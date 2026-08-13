# ADR-001: Storage Key Design

**Status:** Accepted
**Date:** 2026-05-11
**Authors:** StellarForge Core Team

---

## Context

Soroban provides three storage tiers: `instance`, `persistent`, and `temporary`. Each has different cost, durability, and accessibility characteristics. We need a consistent pattern for deciding which tier to use for each type of data across all contracts.

## Decision

### Tier assignments

| Data type | Tier | Rationale |
|---|---|---|
| Admin address | `instance` | Always needed for auth checks; small; should never be archived |
| Pause flag | `instance` | Must be read on every write operation; small |
| Per-user balances | `persistent` | User data; must survive archival; can be loaded on demand |
| Per-user KYC records | `persistent` | Same as balances |
| Per-proposal voting records | `persistent` | Historical; must be queryable post-deadline |
| Asset metadata | `persistent` | Rarely changes; needs to be available for indexers |
| Allowances | `persistent` | Must persist across transactions |
| Temporary nonces | `temporary` | Short-lived; auto-expiry reduces state rent costs |

### Key naming convention

All storage keys use a `DataKey` enum with `#[contracttype]`. String keys are avoided to prevent typos and enable better tooling.

```rust
#[contracttype]
pub enum DataKey {
    Balance(Address),       // persistent
    Metadata,               // persistent
    Issuer(Address),        // persistent
    Allowance(Address, Address), // persistent
}

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");    // instance
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");  // instance
```

## Consequences

- Consistent patterns across all contracts make auditing easier.
- `instance` storage is always loaded with the contract — keeping it small (admin + pause only) minimizes per-invocation overhead.
- `persistent` storage entries are subject to Soroban state rent, and an entry whose TTL lapses is archived and unreadable until restored. Contracts must therefore extend the TTL of entries they touch; this cannot be deferred to `stellar-cli`, which only covers entries a CLI invocation happens to write.

## Amendment (2026-06) — TTL extension is the contract's job

This ADR originally assumed TTL extension was handled by tooling and could be
deferred to Phase 2. That was wrong, and shipped as **SF-2026-002**: no entry
point extended the entries it wrote, so a holder who did not transact for long
enough would have found their balance archived and transfers involving them
failing.

The policy now lives in
[`contracts/common/src/storage.rs`](../../contracts/common/src/storage.rs) —
instance storage extended 7 days, persistent entries 30 days — and every Phase
1 contract depends on it. It is a shared crate rather than a copy per contract
because four independent copies of the constants are four chances to
reintroduce the same bug. `stellarforge-common` is a compile-time library
only; it is never deployed, so the contracts remain independently deployable.

### Write paths are not sufficient on their own

Extending only on write covers balances, which are rewritten by every
transfer. It does not cover an entry that is written once and thereafter only
read — a `KycRecord`, a registry entry, a finalised proposal. Those would age
into archival while still in active use.

`compliance` is the sharpest case: an archived `KycRecord` cannot be read at
all, so the cross-contract `is_compliant` call from `rwa-asset` fails until
the record is restored, blocking transfers for a holder who is in fact
verified. `is_compliant`, `get_kyc`, `get_asset`, `list_assets` and
`get_proposal` therefore extend the entries they read. This makes those calls
ledger-writing rather than pure reads, which is the deliberate trade.

**Still outstanding:** `rwa-asset`'s own read views (`balance`, `allowance`,
`metadata`) do not extend. A holder who neither sends nor receives for long
enough can still have their balance archived — the SF-2026-002 scenario,
narrowed but not closed.
