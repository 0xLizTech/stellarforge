# ADR-002: Authorization Patterns

**Status:** Accepted
**Date:** 2026-09-14
**Authors:** StellarForge Core Team

---

## Context

Soroban has no ambient caller identity. There is no `msg.sender`: a contract can ask which *contract* invoked it, but nothing in the environment tells it which *account* signed the transaction. Authorization is instead something a contract asks for explicitly, by calling `require_auth()` on an `Address` it already holds.

That shifts a decision onto us that other chains make for you. Every privileged entry point has to name the address it expects to hear from, and that address has to arrive from somewhere — as a function parameter, or out of storage. NFR-S-1 requires that every privileged function call `require_auth()` on the relevant authority; it does not say which authority, or where it comes from.

Four contracts make this decision independently, and an inconsistency between them is exactly the kind of thing an auditor finds and an author does not.

## Decision

### The authorizing address is always an explicit parameter

Entry points that act on behalf of a party take that party's `Address` as their first argument and call `require_auth()` on it:

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    // ...
}
```

This is the SEP-41 token convention, and we follow it everywhere rather than inventing a house style. It has one property worth stating plainly: `from` is *asserted* by the caller and *verified* by `require_auth`. A caller may name any address; only a transaction carrying that address's authorization will get past the first line.

The alternative — reading the authority out of storage and never taking it as an argument — is used only where the authority is a singleton, which in practice means the admin.

### Authorization map

Every entry point across the four Phase 1 contracts, and who must authenticate:

| Contract | Entry point | Must authenticate | Notes |
|---|---|---|---|
| `rwa-asset` | `initialize` | the named `admin` | See *Deployment window* below |
| | `set_issuer` | admin | The grantee does **not** sign |
| | `mint` | `issuer` | Plus a stored `is_issuer` check |
| | `burn` | `from` | |
| | `transfer` | `from` | |
| | `approve` | `owner` | |
| | `transfer_from` | `spender` | Not `from` — the allowance is the authority |
| | `set_paused` | admin | |
| | `update_metadata` | admin | |
| | `transfer_admin` | admin **and** `new_admin` | NFR-S-4; see below |
| | `set_compliance` | admin | |
| | `balance`, `allowance`, `total_supply`, `metadata`, `admin`, `paused`, `is_issuer`, `compliance_contract`, `min_compliance_level` | nobody | Pure reads |
| `compliance` | `initialize` | the named `admin` | |
| | `set_kyc`, `revoke_kyc` | admin | |
| | `is_compliant`, `get_kyc` | nobody | Pure reads |
| | `screen` | nobody | Writes, but only a TTL extension — see ADR-004 |
| `registry` | `initialize` | the named `admin` | |
| | `register`, `set_active` | admin | |
| | `get_asset`, `list_assets` | nobody | |
| `governance` | `initialize` | the named `admin` | |
| | `propose` | `proposer` | |
| | `vote` | `voter` | |
| | `finalize` | nobody | Deliberately permissionless |

### Two roles in `rwa-asset`, and only one of them is a singleton

**Admin** lives in instance storage under `ADMIN_KEY`. One per contract, read back by the private `require_admin` helper, which is the only way a privileged entry point obtains it.

**Issuer** is a set, stored per-address as `DataKey::Issuer(Address) -> bool` in persistent storage. Only `mint` consults it. The split exists because minting is the operation an operator most wants to delegate — to a treasury service, a subscription desk — without handing over the ability to pause the contract or reassign its admin.

There is no shared role registry across contracts. Each of the four holds its own admin, and nothing requires them to be the same address. An operator may run the compliance engine under a different key from the asset, which is the point: the party that decides who is KYC-verified need not be the party that can pause trading.

### `transfer_admin` requires both signatures; `set_issuer` requires one

NFR-S-4 requires admin transfer to authenticate both the current and the incoming admin, and `transfer_admin` does:

```rust
pub fn transfer_admin(env: Env, new_admin: Address) {
    Self::require_admin(&env);
    new_admin.require_auth();
    env.storage().instance().set(&ADMIN_KEY, &new_admin);
}
```

The asymmetry with `set_issuer`, which the admin performs alone, is deliberate and rests on one question: **is the mistake recoverable?**

Granting the issuer role to a wrong address is recoverable. The admin calls `set_issuer(addr, false)` and it is undone. Transferring admin to a wrong address is not. A typo, an address on a chain the operator does not control, a key that was never generated — any of these ends with a contract nobody can pause, re-point at a compliance engine, or ever administer again. Requiring the incoming admin to sign proves the key exists and its holder is present, which is the only property that matters here.

Both signatures must appear in the **same transaction**. This is a single atomic invocation, not a two-step propose-and-accept handshake across ledgers.

### Reads never authenticate, and never write

Every getter is callable by anyone, including over simulation with no account at all. This is what lets the TypeScript SDK read balances and metadata without a keypair. It is also, per ADR-001, why reads do not extend TTL: a read that writes is neither free to simulate nor honest about what it does.

## Alternatives Considered

**A shared `access-control` contract holding roles for all four.** Rejected. It makes every privileged call a cross-contract invocation, adds a single point of failure across contracts that are otherwise independently deployable (see ADR-003), and buys nothing at four contracts with one admin and one delegable role. Revisit if the role model grows past that.

**Deriving the authority from the invoking contract rather than a parameter.** Workable for contract-to-contract calls and useless for the case that dominates, which is a user signing a transaction. It would also break SEP-41 compatibility for the token surface.

**A two-step admin handover** — `propose_admin` then `accept_admin` in a later transaction. This is the Ownable2Step pattern and it achieves the same guarantee as dual auth. Rejected for Phase 1 because it costs an extra storage key, an extra entry point, and a pending-transfer state that must be reasoned about (can it be cancelled? does it expire?), in exchange for a tooling convenience: collecting two signatures into one Soroban transaction is harder today than sending two transactions. That trade may be worth revisiting, and the note under *Consequences* is the reason it might be.

**A timelock on admin operations.** Not rejected on merit, just out of scope. It belongs with governance in Phase 3, where there is a body to appeal to during the delay. A timelock with no governance behind it delays the admin without constraining them.

## Consequences

- **The authorizing address must also source the transaction.** Soroban can carry authorization for an address other than the transaction source via `SorobanAuthorizationEntry`, but the SDK does not build those today, so in practice every write is signed by one key that both authorizes and pays. This is tracked as [#31](https://github.com/0xLizTech/stellarforge/issues/31).

- **`transfer_admin` is therefore not reachable from the SDK at all.** It needs two distinct signatures on one transaction, and the SDK's write path supports exactly one. Admin handover is a `stellar-cli` operation until #31 lands. This is a real gap, not a deliberate omission, and it is the strongest argument for revisiting the two-step alternative above.

- **Auditing is a table lookup.** Because the authority is always the first parameter or `require_admin`, "does this function check auth" is answerable by reading its first two lines. Three entry points break the pattern deliberately — `transfer_from` (authenticates the spender, not the owner), `screen` (writes without auth), and `finalize` (no auth at all) — and each is documented where it is defined. An auditor should confirm those three are the *only* exceptions; `burn` is not one of them, it authenticates `from` like any other holder operation.

- **Nothing stops the four admins from being the same key.** In a default deployment they will be, because that is what the deploy script produces. The separation is available, not enforced, and a deployment that uses one key for all four gets one key's blast radius.

- **Deployment window.** `initialize` authenticates the admin it is given, not the account that deployed the contract. It is protected against being run twice, not against being run first by someone else. Deploy and initialize must therefore be treated as one operation — atomically where the tooling allows — and a contract that has been deployed but not yet initialized should not be treated as owned. This is a property of the sequence, not of the contract, and it is the operator's to get right.

## Amendment (2026-09) — `initialize` became `__constructor`

The *Deployment window* consequence above described a hazard and then left it
to operators:

> Deploy and initialize must therefore be treated as one operation — atomically
> where the tooling allows — and a contract that has been deployed but not yet
> initialized should not be treated as owned.

That was an accurate description and an inadequate answer. Correctness rested on
every operator, now and in future, knowing something the contracts did not
enforce. The tooling did allow it atomically, and we were not using it.

All four contracts now configure themselves in a `__constructor`, which
`soroban-sdk` runs as part of the deploy transaction. The `initialize` entry
point is gone. Where the authorization map above lists `initialize`, read
`__constructor`; the authenticating address is unchanged in every case.

**What this closes.** There is no longer a moment at which a deployed contract
exists without an admin, so there is nothing for anyone to claim. The guarantee
moved from the operator's procedure into the contract's shape.

**What it does not change.** `admin.require_auth()` still applies, for the same
reason `transfer_admin` needs the incoming admin's signature: it proves the key
exists and its holder consented, rather than letting a deployer name an address
nobody controls.

### The `AlreadyInitialized` variants are now unreachable

They are kept anyway. ADR-003 freezes discriminants, so removing a variant and
letting later ones shift up would be a silent breaking change to any client
already matching on the numeric code. Each is marked reserved where it is
defined. `NotInitialized` is likewise unreachable in normal operation and is
kept as a defined failure rather than an `unwrap`, so an archived-instance case
would still report something legible.

### Constructor authorization is not covered by tests

`Env::register` invokes a constructor with authorization mocked, so it succeeds
regardless of what the environment permits, and the SDK documents that it
therefore cannot be used to test constructor auth. Doing so needs a real deploy
through `env.deployer()` against uploaded wasm, which no test in this repo does
yet.

This is worth stating plainly rather than leaving to be discovered: the
`require_auth` call in each constructor is asserted by no test. It was equally
unasserted on `initialize`, where every test that called it ran under
`mock_all_auths`, so nothing regressed — but "nothing regressed" is not the same
as "covered", and an audit should treat this as an untested control.

## Amendment (2026-09) — every contract can rotate its admin

The authorization map above gave `transfer_admin` to `rwa-asset` alone. In the
other three contracts, the admin named at deployment held the role for the
contract's whole life. The internal security review recorded this as IR-04.
Nothing could rotate out a compromised compliance key, which is the key that
decides who may transfer, and nothing could replace a lost one.

`compliance`, `registry` and `governance` now expose `transfer_admin(new_admin)`.
Like the original, each authenticates **the admin and `new_admin`** in the same
transaction, and each publishes `AdminTransferred`. Add these rows to the map:

| Contract | Entry point | Must authenticate | Notes |
|---|---|---|---|
| `compliance` | `transfer_admin` | admin **and** `new_admin` | NFR-S-4 |
| `registry` | `transfer_admin` | admin **and** `new_admin` | NFR-S-4 |
| | `list_assets(start, limit)`, `asset_count` | nobody | Pure reads. `list_assets` became paginated under IR-01 |
| `governance` | `transfer_admin` | admin **and** `new_admin` | NFR-S-4. The admin has no other powers in Phase 1 |

The *Consequences* note on the SDK applies to all four. `transfer_admin` needs
two signatures on one transaction, so it remains a `stellar-cli` operation until
[#31](https://github.com/0xLizTech/stellarforge/issues/31) lands.

The dual-authorization requirement is now asserted by tests in all four
contracts, where before it was asserted in none. One test supplies only the
current admin's signature and expects the call to be rejected. Another shows
that a rotated-out admin cannot use its old powers, even with a valid signature.

## Amendment (2026-09) — SEP-41 surface and constructor coverage

IR-09 aligned `rwa-asset` with SEP-41. `approve` now takes
`live_until_ledger`, and `transfer` takes `to` as a `MuxedAddress`; neither
changes who authenticates. Two new entry points extend the map:

| Contract | Entry point | Must authenticate | Notes |
|---|---|---|---|
| `rwa-asset` | `burn_from` | `spender` | Not `from`: as with `transfer_from`, the allowance is the authority |
| | `decimals`, `name`, `symbol` | nobody | Pure reads of `AssetMetadata` |

That makes `burn_from` a fourth deliberate exception, alongside
`transfer_from`, `screen` and `finalize`.

The *Constructor authorization is not covered by tests* section above is
resolved (IR-17). `Env::register` still authorizes the constructor itself, but
under `mock_all_auths` it records the authorization the constructor demanded,
and each contract's `tests/test_constructor_auth.rs` asserts that record
exactly. Removing a constructor's `require_auth` now fails a test. What no unit
test can show is that the network rejects a deploy lacking the signature; that
is host behaviour.
