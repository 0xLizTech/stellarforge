# ADR-003: Upgrade Path

**Status:** Accepted
**Date:** 2026-08-23
**Authors:** Community (docs)

---

## Context

StellarForge ships multiple Soroban contracts (`rwa-asset`, `compliance`, `registry`, `governance`, …). NFR-R-1 requires each contract to remain **independently deployable**: operators may upgrade or replace one without redeploying the others. At the same time, error enums and client interfaces are part of the external ABI that SDKs and peer contracts depend on.

## Decision

### Independent WASM deployments

- Each contract crate builds its own WASM and is deployed to its own contract id.
- Cross-contract calls use **client interfaces** (see ADR-004), not compile-time monorepo coupling at runtime.
- There is no single “protocol upgrade” transaction that swaps every contract; operators version each id on their own schedule.

### What “upgrade” means here

Soroban contracts are immutable WASM. An “upgrade” is operationally:

1. Deploy a new WASM (new contract id, or host upgrade mechanism when used),
2. Repoint configuration that stores the peer address (e.g. `rwa-asset.set_compliance(new_compliance_id)`),
3. Migrate or accept dual-running during transition.

Contracts store peer addresses in instance/persistent config rather than baking them into custom types, so repointing does not require a storage-layout migration of user balances.

### Frozen error-enum discriminants

`#[contracterror]` / error enums that cross the SDK or cross-contract boundary treat **numeric discriminants as frozen once released**:

- Never reorder or reuse a discriminant for a different meaning.
- Add new error variants only at the end (new discriminant values).
- Prefer additive changes; removing a variant is a breaking release even if the number is left reserved.

This keeps older clients’ error mapping stable when they match on codes returned in transaction results.

### Storage layout discipline

- `DataKey` / instance symbols introduced in a release are append-only for that contract id’s lifetime.
- If a layout must change incompatibly, deploy a **new** contract id and migrate — do not silently reinterpret old keys (aligned with ADR-001).

## Alternatives Considered

- **Monolithic upgrade proxy** that owns all modules. Rejected for NFR-R-1: a bug or governance freeze in the proxy blocks unrelated modules.
- **Automatic forced upgrade of all peers.** Rejected: compliance engines are jurisdiction-specific and must be swappable without touching asset balances.
- **Semver-only discipline without frozen discriminants.** Rejected: clients often match numeric codes from chain results without recompiling against the latest crate.

## Consequences

- Release notes must call out new error codes and any peer-address migration steps.
- SDKs should treat unknown error codes as opaque failures, not map them to “success.”
- Integration tests should cover repointing `set_compliance` (and similar) across contract ids without balance loss.