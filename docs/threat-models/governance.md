# Threat Model: Governance

**Status:** Accepted
**Date:** 2026-10-16
**Authors:** StellarForge Core Team

---

## Context

This document expands the PRD's threat model (§10.1-10.2) and the "Governance
attacker" and "Governance takeover" rows specifically, into the entry-point
level for `contracts/governance`. It is a Stage 2 Phase 1 Hardening and Audit
Follow-up: the internal review
([2026-09-14-internal-review-phase1.md](../audits/2026-09-14-internal-review-phase1.md))
fixed IR-03, IR-04 and IR-14 in code, but none of those findings, nor the PRD,
record the residual risk in one place that an external auditor or an
integrator can read before relying on this contract.

It records what each risk is, what already mitigates it, and what does not
yet, as a planning document rather than a finding: everything here is either a
property the contract already has by design, or a residual risk accepted for
Phase 1. Nothing below should be read as "fix this before merge".

## Scope

`contracts/governance/src/lib.rs` as of this document's date: `propose`,
`vote`, `finalize`, `transfer_admin`, and the read-only `admin`,
`get_proposal`, `has_voted`, `proposal_count`. The contract's own doc comment
is the governing fact for this whole document and is restated here because it
changes how every finding below should be read:

> `vote` accepts the weight its caller declares and verifies it against
> nothing, and `finalize` applies no quorum. Any address can therefore carry
> any proposal. Results are advisory until Phase 3 supplies a voting-power
> source; nothing here should gate a privileged action.

Phase 1 governance decides nothing on its own. The risks below are about the
integrity of the record (can a vote tally or an outcome be misrepresented?)
and about what happens once Phase 3 does attach execution to it, not about
funds or privileges Phase 1 governance itself controls, because it controls
none.

## Threats

### T-1: Voting-weight manipulation

**Actor:** Governance attacker (PRD §10.1).

**Mechanism:** `vote(env, voter, proposal_id, support, weight)` takes `weight`
as a caller-supplied `i128`, checked only for `> 0` (`InvalidWeight`) and for
overflow against the running tally. Nothing ties it to a token balance,
stake, or any other on-chain quantity. A single authorized address can cast
`i128::MAX / 2` once, before an overflow check on the second such vote would
trip `Overflow`.

**Why this is not a Phase 1 finding.** The contract's own doc comment
explains the alternative was considered and rejected on purpose: reading a
live balance at vote time is worse than not checking at all, because a holder
could vote, transfer the same tokens onward, and vote again from the
recipient. Correct weighting needs balances snapshotted at proposal creation,
which needs `SFORGE` to exist first. Since the outcome is advisory and gates
nothing, an unweighted or attacker-chosen weight has no effect beyond a
misleading tally.

**Residual risk (Phase 3+).** Once execution hooks exist, an attacker who
controls, or can cheaply acquire for one transaction, an authorized caller
identity could pass any proposal by declaring a large `weight`. This is the
PRD's "Governance attacker: attempts to pass a malicious proposal by
purchasing votes" row, sharpened: with unweighted voting the attacker does
not even need to purchase anything. The PRD's own mitigation for this phase
is explicit: "Phases 1-2: governance is advisory and executes nothing, so
there is nothing to take over." The mitigation is therefore conditional on
Phase 3 shipping balance-snapshotted weight *before* any execution hook is
wired to a passed proposal, not after. **Recommendation carried forward:**
block on this specifically when Phase 3 planning starts; it should not be
treated as "governance already exists, just attach execution to it."

### T-2: Short voting periods foreclosing real participation

**Actor:** Governance attacker, or a proposer acting in good faith but
carelessly.

**Mechanism:** Before IR-14's fix (`2f736c0`), `propose` accepted
`voting_period_ledgers = 0`, so voting was open only in the ledger the
proposal was created in; a proposer could call `propose` and `finalize` back
to back before anyone else had a realistic chance to see the proposal, let
alone vote.

**Current mitigation.** `propose` now rejects any `voting_period_ledgers`
outside `MIN_VOTING_PERIOD_LEDGERS..=MAX_VOTING_PERIOD_LEDGERS` (about one day
to about 90 days) with `InvalidVotingPeriod`. `finalize` separately requires
`env.ledger().sequence() > proposal.deadline_ledger`
(`VotingNotClosed` otherwise), so the deadline cannot be skipped. Together
these guarantee every proposal stays open for at least a day, which is the
window the fix targeted.

**Residual risk.** A day is a floor, not a guarantee of attention. Nothing
notifies voters that a proposal exists beyond the `ProposalCreated` event; a
proposal timed for a period when the relevant voters are known to be away
(a holiday, a timezone gap) could still finalize with a thin, unrepresentative
tally. This is a quorum problem (see T-3), not a voting-period problem, and is
tracked there instead of here to avoid describing the same gap twice.

### T-3: No quorum, so a thin tally finalizes like a decisive one

**Actor:** Governance attacker; also arises with no attacker at all, from
simple voter apathy.

**Mechanism:** `finalize` computes the outcome as
`votes_for > proposal.votes_against`, with no floor on `votes_for +
votes_against`. A proposal with one vote for and zero against passes exactly
as a proposal with a thousand votes for and none against. `test_an_unvoted_
proposal_is_rejected` confirms a proposal with no votes at all is `Rejected`
(a tie, since `0 > 0` is false), but a single vote already escapes that case.

**Why this is not a Phase 1 finding.** Identical reasoning to T-1: the result
is advisory and gates nothing in Phase 1, so a thin tally has no on-chain
consequence beyond the record itself.

**Residual risk (Phase 3+).** This is exactly the gap the PRD names for this
phase: "With execution hooks (Phase 3+): minimum quorum requirements and an
optimistic timelock, then a guardian veto (Phase 5)." A quorum floor (a
minimum of `votes_for + votes_against`, separate from the `votes_for >
votes_against` comparison already present) must land in the same change that
attaches execution to `finalize`, not as a follow-up, or a single vote could
trigger a privileged action the day execution ships.

### T-4: Admin-key compromise

**Actor:** An attacker who obtains the governance contract's admin key (PRD
§10.1's "Admin key compromise" row is protocol-wide; this is the
governance-specific instance).

**Mechanism.** Before IR-04's fix (`2021ed5`), governance had no
`transfer_admin` at all: the admin set at construction held the role for the
contract's entire lifetime, so a compromised or lost key could never be
recovered or rotated. The admin role on this contract currently has exactly
one power: `transfer_admin` itself (`admin` is read-only elsewhere;
`propose`/`vote`/`finalize` take no admin role).

**Current mitigation.** `transfer_admin(env, new_admin)` requires both the
previous admin's and the new admin's `require_auth` (NFR-S-4 dual
authorization), matching `rwa-asset` (ADR-002) and `compliance`/`registry`.
A compromised key alone cannot hand the role to an attacker-controlled
address without the legitimate admin also having signed; conversely, a key
holder cannot be locked out by an attacker who has only observed, not
compromised, the key.

**Residual risk.** If the admin key is lost (not compromised, but
genuinely inaccessible) there is no recovery path: dual authorization that
requires the lost key's signature cannot be satisfied by anyone. No contract
in this repository currently has a guardian or timelock recovery path for a
lost (as opposed to compromised) admin key; governance shares that gap with
`compliance`, `registry` and `rwa-asset`. The cost of this gap is low today
because governance's admin controls nothing beyond its own `transfer_admin`.
**Recommendation carried forward:** revisit once governance's admin controls
more than its own rotation (i.e., once Phase 3 execution hooks exist), since
the cost of an unrecoverable admin grows with what the role can do.

### T-5: Proposal spam

**Actor:** Any address; no privilege required, since `propose` has no access
control beyond `proposer.require_auth()`.

**Mechanism.** Before any mitigation, `propose` cost only the transaction fee
and the rent on one persistent `Proposal` entry; nothing scaled with how many
proposals an address had already created. `title` was also unbounded in size
before IR-14's fix, so a proposal could additionally carry an
oversized title (now capped at `MAX_TITLE_BYTES`, 256 bytes) and crowd out the
timeline an indexer or front end shows for genuine proposals. GitHub issue
#134, "Add a proposal deposit option to governance," tracks the deposit
mitigation separately.

**Current mitigation.** `MAX_TITLE_BYTES` bounds the one unbounded input.
An optional, admin-configured proposal deposit (refunded on finalization) is
the direct mitigation for the cost side of spam and is tracked and
implemented as its own change (#134) rather than folded into this threat
model, because it is a contract behavior change with its own tests and
`CHANGELOG.md` entry, not a documentation-only finding.

**Residual risk.** Even with a deposit, spam is throttled by cost, not
eliminated: an attacker willing to pay enough deposits (recovered on
finalization, so the true cost is only the ledger rent and the time value of
the locked deposit) can still create many proposals. A deposit set to 0 by
the admin (the "optional" in "optional deposit") reverts to today's
unmitigated state; that is an admin configuration choice, not a contract
defect, and is recorded here so an auditor does not need to re-derive it.

## Assumptions

- Phase 1 governance gates no privileged action anywhere in this protocol.
  Every "residual risk" above that is deferred to Phase 3+ is deferred
  specifically because of this; if any future change wires a `finalize`
  outcome to a privileged call before T-1 and T-3 are closed, this document's
  risk ratings for T-1 and T-3 are void and must be revisited first.
- `SFORGE` (or whatever token eventually supplies voting weight) snapshots
  balances at proposal creation, not at vote time, per the contract's own
  doc comment. This document assumes that design holds; a live-balance
  design would reopen the double-vote hazard the comment describes.
- The Security Council and its emergency powers (PRD §10.4) do not currently
  extend to this contract: `rwa-asset`'s `set_paused` is the only Phase 1
  circuit breaker. Governance has no pause and no override.

## References

- [PRD §10: Security Considerations](../prd/PRODUCT_REQUIREMENTS.md)
- [Internal Security Review - Phase 1](../audits/2026-09-14-internal-review-phase1.md)
  (IR-03, IR-04, IR-14)
- [ADR-002: Auth Patterns](../architecture/002-auth-patterns.md)
- [SECURITY.md](../SECURITY.md)
