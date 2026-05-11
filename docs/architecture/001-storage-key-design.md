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
- `persistent` storage entries are subject to Soroban state rent; callers must extend TTL for long-lived entries (handled automatically by `stellar-cli` for now; we will add explicit TTL extension in Phase 2).
