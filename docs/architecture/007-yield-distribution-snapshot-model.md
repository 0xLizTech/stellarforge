# ADR-007: Yield Distribution Snapshot Model

**Status:** Accepted
**Date:** 2026-10-24
**Authors:** StellarForge Core Team

---

## Context

Phase 2 introduces the `yield-distributor` contract to distribute dividends and yield to token holders (PRD §5.5, FR-07). A foundational requirement for yield distribution is determining token ownership deterministically as of a specific snapshot point (historical ledger sequence), ensuring distributions cannot be gamed by flash-loaning or transferring tokens immediately around payment intervals.

Three primary architectural models exist for snapshot-at-ledger distribution on Soroban:
1. **On-chain Historical Checkpoints (Binary Search over Historic Ledger Entries):** Asset/vault balances record `(ledger, balance)` tuples on balance mutation.
2. **Off-chain Merkle Tree Roots with On-chain Claim Proof Verification:** An off-chain worker snapshots ledger state, generates a Merkle tree of claims, and posts the Merkle root on-chain.
3. **Pull-based Global Accumulator (Synthetix / MasterChef Staking Model):** A cumulative reward-per-token scalar updates globally, and accounts track their settled reward index on interactions.

---

## Evaluation of Approaches

| Criteria | On-chain Checkpoints | Merkle Root Distribution | Pull-based Accumulator |
| :--- | :--- | :--- | :--- |
| **Write Cost (Transfer/Mint/Burn)** | Low-to-Medium (appends checkpoint entry per change) | Zero (no balance mutation overhead) | Low (updates global & user debt) |
| **Read/Claim Cost** | Low ($O(\log N)$ binary search across checkpoints) | Medium ($O(\log K)$ Merkle proof verification) | Lowest ($O(1)$ arithmetic) |
| **Storage Overhead** | Medium ($O(K \times \text{mutations})$ bounded by ring buffer / compaction) | Minimal (single 32-byte root per distribution) | Low ($O(\text{holders})$ persistent debt entries) |
| **Trust Model & Decentralization** | Fully trustless, native on-chain verification | Requires off-chain snapshot operator & trust in root publisher | Fully trustless, native on-chain |
| **Continuous vs Discrete Snapshots** | Discrete snapshots at any arbitrary historical ledger | Discrete fixed-interval distributions | Continuous stream or step distributions |
| **Integration Complexity with Vault/Asset** | Direct integration via `balance_at(holder, ledger)` | Independent off-chain pipeline; requires proof generation SDK | High coupling; requires tracking balance transitions continuously |

---

## Decision

We adopt **On-chain Historical Checkpoints** (`balance_at(holder, ledger)`) in the vault and share token architecture (as specified in ADR-006 §7 and Issue #256).

### Key Reasons:
1. **Trust-minimization:** Eliminates reliance on off-chain snapshot workers, indexers, or custodial Merkle root signers.
2. **Arbitrary Historical Ledger Querying:** Enables `yield-distributor` to query balances at any past distribution ledger sequence deterministically.
3. **Soroban Storage Efficiency:** Checkpoints are stored in persistent storage using bounded vectors with oldest-entry compaction, preventing unbounded state bloat.

### Limits on Holder Count and Checkpoint Bounds:
- **Holder Limit per Batch Claim:** To respect Soroban instruction and memory budgets per transaction, `yield-distributor` claim and payout execution is capped at **100 claims per batch execution**.
- **Checkpoint Ring Buffer Bound:** Each holder account retains at most **1,000 checkpoint entries**. When the checkpoint vector reaches capacity, oldest entries are trimmed or compacted, ensuring bounded rent and storage footprint.

---

## Alternatives Considered

- **Merkle Roots:** Rejected for primary distributions due to reliance on off-chain indexing infrastructure and poor UX requiring claimants to provide cryptographic proofs from external APIs.
- **Pull-based Accumulators:** Evaluated as viable for continuous staking, but suboptimal for discrete institutional dividend events where payouts occur at exact calendar/snapshot ledgers across non-staked circulating holdings.

---

## Consequences

- The `vault` and token contracts maintain checkpoint records on balance adjustments via `write_balance`.
- The `yield-distributor` contract queries `balance_at(holder, ledger)` directly to calculate prorated distributions.
- Storage rent is bounded by ring-buffer compaction.
