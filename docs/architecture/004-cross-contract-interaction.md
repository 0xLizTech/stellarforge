# ADR-004: Cross-Contract Interaction

**Status:** Accepted
**Date:** 2026-08-23
**Authors:** Community (docs)

---

## Context

`rwa-asset` must screen transfer parties against KYC/verification state owned by a **compliance** contract. Hard-depending on the `compliance` crate at runtime would couple deployability and force every asset deployment onto one concrete engine. We also need a split between **pure queries** (safe for wallets/indexers) and **screening during transfers** (may refresh TTL — see ADR-001 amendment).

## Decision

### Client interface, not a hard dependency

`rwa-asset` declares a Soroban client trait (e.g. `ComplianceInterface` in `contracts/rwa-asset/src/compliance.rs`) with the shape it needs:

```rust
fn screen(env: Env, subject: Address, min_level: u32) -> bool;
```

The asset stores a compliance **contract address** in config (`set_compliance`). Operators may point it at:

- the stock `compliance` contract,
- a jurisdiction-specific engine that implements the same interface, or
- a mock during tests,

without rebuilding the asset crate against a different package.

### `screen` vs `is_compliant`

| Entry point | Ledger side effects | Intended caller |
|-------------|---------------------|-----------------|
| `is_compliant` | **None** (pure query) | SDKs, explorers, off-chain policy checks |
| `screen` | Extends TTL of the KYC record consulted | On-chain transfer path (`rwa-asset`) |

Rationale (also in ADR-001):

- A KYC record is written once and then mostly read; without a refresh moment it ages toward archival.
- Only the compliance contract can extend **its own** entries; the asset cannot keep them alive.
- Transfers already pay for a ledger write, so TTL extension on `screen` does not invent a new write path for pure readers.
- Keeping `is_compliant` pure avoids charging explorers and preserves read-only simulation semantics.

`rwa-asset` binds to `screen` inside transfer/mint paths that must enforce policy. Views that only display status should call `is_compliant` (or off-chain equivalents).

### Failure and configuration

- If compliance is unset/`None`, screening is disabled (permissive) or rejected — follow the contract’s documented `set_compliance` behavior; do not hardcode a default peer id.
- `min_level` is configured on the asset and passed into `screen`, so the same compliance deployment can serve assets with different thresholds.

## Alternatives Considered

- **Wasm import / static linking of compliance.** Rejected: breaks independent deployability and jurisdiction swap.
- **Extend TTL inside `is_compliant`.** Rejected: makes a query non-pure and taxes every reader (ADR-001).
- **Permissionless `extend_kyc` keeper only.** Optional later; not sufficient alone for the transfer path’s “holder is active now” signal.

## Consequences

- New compliance engines must implement at least `screen` with the same semantics (boolean at-or-above level, unexpired).
- Asset upgrades that change the interface shape are breaking for every peer engine; prefer additive methods.
- Auditors should verify that every balance-moving path that claims compliance actually calls `screen`, not only `is_compliant`.