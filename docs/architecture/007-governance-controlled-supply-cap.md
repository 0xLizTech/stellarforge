# ADR-007: Governance-Controlled Supply Cap Loosening

**Status:** Accepted
**Date:** 2026-10-19
**Authors:** StellarForge Core Team

---

## Context

IR-02 (see `docs/audits/2026-09-14-internal-review-phase1.md`) fixed
`update_metadata` so the admin's own signature can tighten a supply cap but
never raise it, lift it to uncapped, or set it below circulating supply.
PRD §10.2 records the resulting guarantee as "[the cap is] not bypassable by
admin." That fix was correct for what it closed: a single admin key silently
diluting every holder in one call.

It also left no path at all for a cap to grow. In practice an issuer's
circulating demand can legitimately outgrow a cap set at launch — a real
estate fund raising a second tranche, a tokenized note being reissued at a
larger size. Today the only way to raise a cap is to redeploy a new asset
contract and migrate every holder, which is disruptive and, per ADR-003, not
something this protocol can do non-destructively (`rwa-asset` has no upgrade
path).

Issue #148 asks for the actual rule PRD §10.2 was always reaching for:
tightening is uncontroversial and stays single-signature, but loosening is
real dilution and needs an approval the admin cannot produce alone.

## Decision

### A second, separately configured address approves loosening; the admin alone still cannot

`rwa-asset` gains `set_governance(governance: Option<Address>)`, admin-only,
naming the address whose signature can approve raising a cap. Loosening then
goes through two steps, mirroring the existing `propose_admin`/`accept_admin`
shape rather than inventing a new one:

1. `propose_cap_increase(new_max_supply)` — admin only. Validates that a
   finite cap already exists and that `new_max_supply` is genuinely higher
   (or `0`, uncapped). Stores the proposal; does not touch `AssetMetadata`.
2. `approve_cap_increase()` — `governance`'s signature only. Re-validates
   against current circulating supply, then applies the new cap through the
   same `replace_metadata` path `update_metadata` uses — so it bumps
   `metadata_version` and publishes the same `MetadataUpdated` event (#149)
   rather than a parallel code path a monitor would also have to learn.

`cancel_cap_increase()` (admin-only) withdraws a proposal before approval,
mirroring `cancel_admin_transfer`. `update_metadata` itself is unchanged: it
still refuses any change that loosens a cap, so there remains exactly one
way to raise one.

### Why a configurable address, not the `governance` contract by name

The `governance` crate's own module docs are explicit: "Phase 1: create
proposals and cast votes; execution hooks in Phase 3... `vote` accepts the
weight its caller declares and verifies it against nothing, and `finalize`
applies no quorum [by default]. Results are advisory." Hard-wiring supply-cap
approval to that contract today would gate real dilution power on a vote
nobody has to win, which is worse than not gating it at all — it would look
governed while being whatever the last caller declared.

Naming an arbitrary `Address` instead lets an operator point this at whatever
already provides real multi-party assurance — a multisig account is the
natural Phase 1/2 choice — and repoint it at the `governance` contract once
Phase 3 makes its outcomes binding, with no change to `rwa-asset` itself.
This is the same reasoning ADR-006 gives for the admin-recovery guardian:
the contract does not need to know whether the address behind a role is a
single key, a multisig, or eventually a DAO.

### Why two steps in one address pair, not a single dual-signature call

A single entry point requiring both `admin.require_auth()` and
`governance.require_auth()` in one transaction is possible in principle, but
every other "both sides must consent" flow in this codebase
(`propose_admin`/`accept_admin` here and in every other Phase 1/2 contract)
already uses two steps for the same reason ADR-002/`propose_admin`'s doc
comment gives: it does not require the SDK to build a transaction carrying
two distinct signers in lockstep, and the two signatures can come from
parties who are not coordinating in real time. Matching that shape means one
mental model across the codebase instead of two.

### Why `update_metadata` keeps refusing loosening unconditionally

Allowing `update_metadata` to loosen a cap *when governance happens to also
sign the same call* would work, but it would mean an integrator reading
`update_metadata`'s rule ("tightens only") is no longer reliably true — it
would depend on who else signed a given transaction. Keeping the two entry
points separate means `update_metadata`'s contract is unconditional: it
never loosens, full stop, and the only way to loosen is the explicitly
named, explicitly approved path.

## Alternatives Considered

**Gate loosening on an actual `governance.finalize`d proposal.** Rejected for
now, for the reason given above: Phase 1/2 governance verifies no vote
weight and applies no quorum unless an admin opts in, so a passed proposal
is not yet a trustworthy signal (see the Phase 1 caveat in `governance`'s own
README). Once Phase 3 makes outcomes binding, an operator can simply set
`governance` to the `governance` contract's address and have `finalize`
call `approve_cap_increase` — no change to this ADR's shape is needed, only
to what address is configured.

**A fixed multisig threshold built into `rwa-asset` itself (M-of-N
signers).** Rejected on the same basis ADR-006 rejects replacing the
single-admin model outright: it is a materially larger storage and rotation
surface than this issue's scope, and an `Address` configured here can
already *be* a multisig contract, which gets the same guarantee without
`rwa-asset` needing to know anything about thresholds.

**Let `governance` itself be `None` by default but allow `propose_admin`'s
proposer to immediately self-approve.** Rejected: that collapses back to a
single signature deciding dilution, which is exactly what PRD §10.2 and
IR-02 closed.

## Consequences

- **A supply cap can now grow**, but only through a path that needs a second,
  independently configured signature. An operator who never calls
  `set_governance` gets the exact pre-#148 behavior: a cap can only ever be
  tightened, by the admin alone.
- **One more address for an operator to manage and secure**, with the same
  operational weight as the admin key itself — whoever (or whatever
  contract) `governance` points at can unilaterally raise the cap once the
  admin has proposed a value, so it should be held with at least as much
  care as a cap-tightening admin key.
- **No change to any existing entry point's behavior.** `update_metadata`'s
  rule is exactly what IR-02 shipped; everything new is additive.
