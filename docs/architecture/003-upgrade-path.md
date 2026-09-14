# ADR-003: Upgrade Path

**Status:** Accepted
**Date:** 2026-09-14
**Authors:** StellarForge Core Team

---

## Context

It is a common misconception that Soroban contracts are immutable. They are not. `env.deployer().update_current_contract_wasm()` replaces the executable of the running contract in place, keeping its address and all of its storage. A Soroban contract is mutable exactly as far as it chooses to be: it is upgradeable if it exposes an entry point that calls that function, and permanently frozen if it does not.

So "can our contracts be upgraded" is not a question about the platform. It is a question about what we ship, and we have to answer it before the first deployment rather than after, because the answer is not itself upgradeable.

NFR-R-1 states that contracts MUST NOT be upgradeable without explicit admin authorization *and a governance vote*, and marks that requirement Phase 3+. Phase 1 has no governance to speak of: `governance` accepts whatever vote weight its caller declares, verifies it against nothing, and applies no quorum. Gating an upgrade on a Phase 1 governance outcome would be gating it on nothing at all.

## Decision

### Phase 1 contracts expose no upgrade entry point

None of the four contracts calls `update_current_contract_wasm`. There is no `upgrade`, no `set_wasm_hash`, no admin escape hatch. Once deployed, the code at a given contract address is the code that will run there forever.

This is the only honest reading of NFR-R-1 for a phase with no functioning governance. The requirement says upgrades need a vote; there is no vote; therefore there are no upgrades.

### Contracts are independently deployable

Each contract is its own Cargo package and none depends on another at runtime. Cross-contract calls go through Soroban's client interface by address, never through a linked crate (see ADR-004). `contracts/common` is the single exception and only at compile time: it holds the TTL policy that must be identical everywhere, and is never deployed as a contract.

The practical consequence is that the four can be deployed, and replaced, on independent schedules. `rwa-asset` holds the compliance engine's address in instance storage and reaches it via `set_compliance`, so an operator can stand up a new compliance contract and re-point the asset at it in one admin call, with no change to the asset's code.

### Migration, not upgrade, is the Phase 1 answer

Fixing a contract means deploying a new one and moving traffic to it. How well that works differs sharply per contract, and the difference is not incidental:

| Contract | Holds state that matters? | Migration story |
|---|---|---|
| `compliance` | KYC records | Redeploy, re-issue records, `set_compliance` on each asset. Records are re-derivable from the operator's own KYC files |
| `registry` | Asset directory | Redeploy, re-`register` each asset. The directory is a convenience index, re-derivable from the operator's records |
| `governance` | Proposals and votes | Redeploy. History is preserved in events; in-flight proposals are lost |
| `rwa-asset` | **Holder balances** | **There is none** |

That last row is the important one. Balances live in the asset contract's own persistent storage, keyed by holder address. A redeployed asset contract starts at zero supply with zero holders, and nothing in the protocol can move the old balances across — reading another contract's storage directly is not something Soroban permits, and even a migration entry point on the new contract would need every holder to act.

**So `rwa-asset` is, for practical purposes, permanent once it has holders.** Not by an explicit decision to make it so, but as a consequence of two decisions taken together: no upgrade entry point, and state that cannot be exported.

### Error discriminants are frozen

Every `#[contracterror]` enum pins its variants with `#[repr(u32)]` and explicit values, and each carries the same note:

> Discriminants are part of the contract's public interface: clients match on the numeric code, so existing variants must keep their values and new ones must be appended. Renumbering silently changes what a deployed client believes went wrong.

The current allocations are `RwaError` 1–13, `GovernanceError` 1–9, `RegistryError` 1–5, `ComplianceError` 1–2.

The word doing the work is *silently*. A renumbered variant does not break a build or fail a test. A deployed SDK that maps 11 to "not compliant" keeps mapping 11 to "not compliant", and simply starts telling people the wrong thing about why their transfer failed. Nothing anywhere reports an error, which is the property that makes this worth an ADR rather than a comment.

Freezing them is cheap insurance for a case that outlives the no-upgrade decision: even when Phase 3 makes contracts upgradeable, an upgrade that renumbers is a silent breaking change to every client already in the wild.

## Alternatives Considered

**Ship an admin-gated `upgrade` now, and add the governance gate in Phase 3.** Rejected, on two grounds.

The first is what it would mean for holders. An address that can replace the code can mint to itself, zero any balance, or disable compliance entirely. For a protocol whose entire proposition is that a token represents a real-world asset under real-world legal terms, "the admin can rewrite the rules at any time" is not a footnote — it is a description of a different product. Every guarantee in the other ADRs holds only as far as the code stays put.

The second is sequencing. NFR-S-6 requires an external audit before mainnet, and an upgrade entry point is precisely the kind of surface an audit exists to examine. Shipping one now means shipping the most dangerous function in the codebase through the least scrutiny.

**A proxy or delegate pattern**, with a thin immutable front end forwarding to a swappable implementation. Rejected. Soroban has native in-place upgrade, so a proxy adds an invocation hop and its own storage layout to reimplement something the platform already does — and it does not actually avoid the question, it just moves the dangerous privilege into the proxy's admin.

**A migration entry point on `rwa-asset`** — a `claim_from(old_contract, holder)` that reads a balance out of the predecessor and credits it. Not rejected on principle, and it is the shape any future migration would take, but it cannot be retrofitted: the *old* contract would need to expose a burn-and-attest entry point, and the currently deployed one does not. Recorded here so that a future asset contract can be designed with it from the start.

## Consequences

- **A deployed Phase 1 `rwa-asset` cannot be fixed.** SF-2026-001, the self-transfer bug that minted tokens from nothing, was caught pre-deployment. Had it been found after an asset had holders, there would have been no remedy inside the protocol: no patch, no pause-and-upgrade, only a public advisory and an off-chain reissue. This is the cost of the decision, stated plainly. It is acceptable *only* because Phase 1 is explicitly not for production — the contracts are unaudited and not on mainnet — and it stops being acceptable the moment either of those changes.

- **The TTL policy of a deployed contract is permanent too.** `contracts/common` is a compile-time crate, so changing the constants requires rebuilding and redeploying. For `rwa-asset` that is the same dead end as any other fix. SF-2026-002 was a TTL bug; the same bug found a month later would have been unfixable. Worth weighing before Phase 3's upgrade design treats TTL as settled.

- **Compliance is the one piece with a real hot-swap path,** because `set_compliance` re-points by address. That makes it the right place for anything expected to change: jurisdictional rules, sanctions screening, a per-market engine. Design accordingly, and resist the temptation to move that logic into the asset where it would become permanent.

- **The registry's `set_active` is the closest thing to a kill switch for a migration.** It cannot stop a superseded asset contract from working, but it can stop the directory from advertising it, which is what indexers and front ends read.

- **Phase 3 inherits a clean slate.** No upgrade entry point means no upgrade semantics to be compatible with, no proxy storage layout to preserve, and no existing admin privilege to take away from someone. Governance-gated upgradeability can be designed on its merits. The one thing it inherits and must respect is the frozen discriminants.
