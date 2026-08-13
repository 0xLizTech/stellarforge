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

**How bad archival actually is.** Since protocol 23 an archived persistent
entry is restored automatically by the transaction that accesses it, provided
the submitter's tooling puts it in the restore footprint — RPC simulation does.
So archival is a restore *fee* charged to whoever touches the entry first, not
a permanent brick and not a failed transfer. Extending on read is therefore a
cost trade, not a correctness fix, and a bad one on its own terms: it converts
one rare restore fee into a small ledger write charged to every reader, and it
makes a query non-pure.

**The constraint that shapes the design:** a contract can only extend its
*own* entries. `rwa-asset` cannot keep a `KycRecord` alive no matter what it
does on its write path, so any fix for the compliance case has to live inside
the compliance contract.

### Decision

1. **Write paths extend to the network maximum**, read from the ledger via
   `env.storage().max_ttl()` rather than hardcoded, since the maximum is a
   network parameter and extending beyond it is not accepted. An entry is
   refreshed only once it has decayed by `PERSISTENT_REFRESH_INTERVAL`
   (30 days), so repeated writes do not pay for ledgers the entry already has.

2. **Reads stay pure.** `get_kyc`, `get_asset`, `list_assets`, `get_proposal`
   and `has_voted` do not touch the ledger.

3. **Screening is a write path, and is separated from the query.**
   `ComplianceContract::is_compliant` is a pure query for SDK and off-chain
   use. `ComplianceContract::screen` is identical except that it extends the
   record it consults, and it is what `rwa-asset`'s `ComplianceInterface` binds
   to. Screening happens inside a transfer, which is already paying for a
   ledger write, and it is the one moment at which the holder is demonstrably
   active — so the record is refreshed exactly when it is in use, which is the
   self-maintaining property balances get for free.

### Alternatives not taken

- **Permissionless keeper entry points** (`extend_kyc(subject)` and friends,
  callable by anyone willing to pay). Deferred until there is evidence anything
  needs them; the maximum-TTL bump covers roughly a year.
- **Relying on auto-restoration alone**, with no extension at all. Rejected
  because the restore fee lands arbitrarily on whichever party transacts first,
  which is a poor experience for the holder who happens to go next.

### Residual exposure, and why it cannot be eliminated

`rwa-asset`'s read views (`balance`, `allowance`, `metadata`) are pure, in
line with point 2. A balance is refreshed to the network maximum by every
mint, burn and transfer it takes part in, so the remaining exposure is a
holder who neither sends nor receives for longer than the maximum TTL — about
a year. Their next transfer then pays a restore fee.

This is not an unfixed defect. Entries live only as long as someone pays rent,
so there is no arrangement in which the exposure disappears; there is only a
choice of who pays and when:

| Who pays | Mechanism | Cost profile |
|---|---|---|
| Whoever transacts next | Auto-restore on access — **chosen** | Lazy. Nothing is spent on holders who never return |
| Every reader | Extend inside `balance()` | Continuous. Explorers and indexers subsidise idle holders, and the view stops being pure |
| Issuer or keeper | Permissionless `extend_balance(addr)` | Proactive. O(holders) writes per year regardless of need |

The first is chosen because the cost falls on the party who benefits, at the
moment they benefit. Note that Soroban's own token reference extends TTL on
read; that was the right call before protocol 23, when archival was a hard
failure rather than a fee, and it is worth knowing that the convention
predates auto-restoration rather than disagreeing with this decision.

**Known limitation.** Idle periods beyond a year are ordinary for a
buy-and-hold real-world asset, so a meaningful share of holders will meet a
restore fee rather than this being a rare edge. The fee is small — a balance is
a single `i128`, and restore cost scales with entry size — and is paid once. If
it should be made invisible to holders, the third row is the way to do it, as
an explicit issuer cost; the read path is not. Revisit when there is deployment
data to measure it against.

Auto-restoration also depends on the submitting client putting the archived
entry in the restore footprint. RPC simulation does this, so the standard flow
is unaffected; a hand-built or cached footprint would fail instead.
