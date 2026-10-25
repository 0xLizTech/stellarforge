# ADR-004: Cross-Contract Interaction

**Status:** Accepted
**Date:** 2026-09-14
**Authors:** StellarForge Core Team

---

## Context

`rwa-asset` has to refuse a transfer when either party fails compliance screening. The rules that decide that — verification levels, expiry, jurisdiction — are not the asset's business, and they change on a different clock from the asset's. They live in the `compliance` contract.

That makes this the one place in Phase 1 where a contract calls another contract, and the decisions about how it does so set the pattern for every cross-contract call that follows.

There is a second, subtler question underneath it. Screening reads a KYC record, and per ADR-001 a record that is written once and thereafter only read has no natural moment at which its TTL gets refreshed. Only the compliance contract can extend its own entries. So the shape of this call determines whether KYC records stay alive at all.

## Decision

### Bind to an interface, not to the crate

`rwa-asset` declares the shape of the call it needs and generates a client from it:

```rust
#[contractclient(name = "ComplianceClient")]
pub trait ComplianceInterface {
    fn screen(env: Env, subject: Address, min_level: u32) -> bool;
}
```

It does not depend on the `compliance` crate. The trait names one function; `#[contractclient]` turns it into a typed client that invokes whatever address it is handed.

Three things follow, and all three are the point:

**The asset can be pointed at any conforming contract.** A jurisdiction-specific engine, a stricter engine for a particular asset class, a mock during testing. `set_compliance` takes an address, and anything implementing `screen(Address, u32) -> bool` will do.

**The two contracts stay independently deployable** (ADR-003). A change to the compliance contract's internals, its storage layout, its error enum, does not rebuild or redeploy the asset.

**The coupling is exactly one function wide.** `compliance` also exposes `set_kyc`, `revoke_kyc`, `is_compliant`, `get_kyc` and `admin`. The asset knows about none of them, so none of them can break it.

The cost is honest: this is a structural contract, not a compiler-checked one. Nothing verifies at build time that the address in `COMPLIANCE_KEY` implements `screen`. A mismatch surfaces at the first transfer, as a failed invocation.

### `screen` is a write path; `is_compliant` is the query

The compliance contract exposes both, and they differ by one line:

```rust
pub fn is_compliant(env: Env, subject: Address, min_level: u32) -> bool {
    Self::evaluate(&env, subject, min_level)
}

pub fn screen(env: Env, subject: Address, min_level: u32) -> bool {
    extend_persistent(&env, &DataKey::KycStatus(subject.clone()));
    Self::evaluate(&env, subject, min_level)
}
```

The asset binds to `screen`. The SDK and every off-chain reader use `is_compliant`.

The reasoning is ADR-001's, applied: a KYC record is written once and read forever, so it needs a moment to be refreshed, and screening is the only moment at which the holder is demonstrably active. It is also already inside a transfer, which is paying for ledger writes anyway, so the extension is close to free where it lands and would be pure overhead anywhere else. Putting it on the pure query instead would charge every explorer, indexer and balance-checking front end for a ledger write, and would make a read non-pure — which, per ADR-002, is a property we rely on to let the SDK read without an account.

`screen` takes no authorization. It is callable by anyone, and this is safe because it is the weakest possible write: it can only push a TTL further out. It cannot create, alter or revoke a record, and it cannot change what `evaluate` returns. The worst an attacker achieves by calling it in a loop is paying to keep someone else's KYC record alive.

### What gets screened, and what deliberately does not

| Operation | Screened |
|---|---|
| `rwa-asset.mint` | `to` |
| `rwa-asset.transfer` | `from` and `to` |
| `rwa-asset.transfer_from` | `from` and `to` |
| `rwa-asset.burn` | nobody |
| `rwa-asset.burn_from` | nobody |
| `rwa-asset.approve` | nobody |
| `vault.transfer` | `from` and `to` |
| `vault.transfer_from` | `from` and `to` |
| `vault.approve` | nobody |

`transfer_from` screens the owner and the recipient, not the spender. The spender moves value it does not own; the parties to the movement are the ones a regulator cares about.

**`burn` screening nobody is deliberate.** A holder whose KYC has lapsed — expired record, revoked verification — can still destroy their own tokens. The alternative traps them: unable to transfer, unable to exit, holding a position they cannot close. Burning reduces supply and moves value to no one, so there is no counterparty to screen and no transfer to prevent. An audit checklist that flags "a balance-changing operation with no compliance check" will flag this line; it is correct as written. `burn_from`, added with SEP-41 alignment (IR-09), screens nobody for the same reason: it destroys a holder's tokens rather than moving them to anyone.

**`approve` screening nobody** follows from the same reasoning. Granting an allowance moves nothing. The screen happens when the allowance is exercised, at `transfer_from`, which is where the value actually moves and where compliance status is current rather than however old the approval is.

### No compliance contract configured means no screening

```rust
fn require_compliant(env: &Env, party: &Address) {
    let Some(compliance) = Self::compliance_contract(env.clone()) else {
        return;
    };
    // ...
}
```

Unconfigured is **permissive**, not restrictive. This is a decision, not an oversight, and it exists so an asset can be deployed, initialized and tested before an operator has stood up a KYC engine. Screening becomes mandatory the moment `set_compliance` names a contract, and `set_compliance(None, _)` turns it off again.

The exposure is the obvious one: an asset that was never configured transfers freely between anyone. Configuring compliance is an operational step with no on-chain enforcement behind it, and `compliance_contract()` returning `None` is the check an operator, an auditor or a monitoring system should be making.

## Alternatives Considered

**Depend on the `compliance` crate directly.** Rejected. It would give compile-time checking of the call shape, at the price of the three properties above — one deployable engine, rebuilt assets on every compliance change, and a coupling as wide as the crate's public surface.

**Put compliance rules inside `rwa-asset`.** Rejected, and ADR-003 explains why it would have been the worse mistake of the two: the asset contract cannot be upgraded once it has holders, so embedded rules would be frozen permanently. Jurisdictional rules are the single most change-prone thing in the protocol, which makes the asset contract the single worst place to put them.

**One `screen` entry point, no pure query.** Rejected. Every off-chain read would then write to the ledger, and could not be simulated without an account.

**One `is_compliant` entry point, no screening variant,** with KYC records kept alive by a permissionless keeper. Rejected for Phase 1 as the more moving parts; ADR-001 records the keeper idea as deferred rather than refused.

**Treat an unconfigured compliance contract as failing closed.** Rejected for Phase 1 because it makes an asset unusable between initialization and compliance setup, including in tests and local development. Worth revisiting for a production profile, where "fails open by default" is the wrong default to have shipped.

## Consequences

- **The compliance contract is a hard dependency at runtime once configured.** If its address is wrong, if it does not implement `screen`, or if it panics, every `mint`, `transfer` and `transfer_from` fails. `burn` and `approve` keep working, so holders can still exit. Recovery is an admin call to `set_compliance` pointing at a working contract — which is available precisely because ADR-003 kept that indirection.

- **`min_level = 0` does not mean "allow everyone".** `evaluate` returns `false` for a subject with no record at all, before it ever compares levels. So a compliance contract configured at level 0 requires every party to hold *some* record, not none. This is the most likely operational surprise in the whole design: the configuration that reads like "screening off" is in fact "everyone must be enrolled". Screening off is `set_compliance(None, _)`.

- **A record with `expires_at = 0` never expires.** Combined with the point above, the two ways to be permanently compliant are an unconfigured asset or a non-expiring record, and only the first is visible from the asset side.

- **Every screened transfer costs two contract invocations and a ledger write.** This is the price of the split and it lands on the transfer path, which NFR-P-2 budgets at two ledgers. Read performance (NFR-P-1, sub-500ms simulation) is unaffected, because reads go to `is_compliant` and touch nothing.

- **Any future cross-contract call should follow this shape**: a `#[contractclient]` trait naming only the functions needed, an address in configurable storage, and a pure/impure split at the boundary if the callee has entries whose lifetime depends on being called.
