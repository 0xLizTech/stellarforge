# governance

An on-chain proposal-and-voting skeleton: create a proposal, cast a weighted
vote, finalize against a quorum and pass threshold. Execution hooks are
Phase 3 work — nothing here currently executes anything.

See the root [README](../../README.md#governance) for the full project
context, and [`docs/threat-models/governance.md`](../../docs/threat-models/governance.md)
for the risks this Phase 1 design accepts.

> **Phase 1 caveat.** `vote` accepts a caller-supplied `weight` and verifies
> it against no token balance or other voting-power source. `finalize` does
> apply an admin-configurable quorum and pass threshold, but against that
> unverified weight: any address can carry a vote, so a passed proposal is
> advisory, not binding, until the `SFORGE` token and execution hooks land in
> Phase 3. Nothing in this protocol currently gates a privileged action on a
> governance outcome for this reason — see
> [ADR-007](../../docs/architecture/007-governance-controlled-supply-cap.md)
> for a concrete instance of that design constraint.

## Entry points

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin)` | admin | Runs at deploy; not callable afterwards |
| `propose(proposer, title, description_hash, voting_period_ledgers)` | proposer | Create a proposal; period must be `MIN_VOTING_PERIOD_LEDGERS`–`MAX_VOTING_PERIOD_LEDGERS` (~1–90 days), title at most `MAX_TITLE_BYTES` (256); rejected if an active proposal already shares the same title and description hash |
| `vote(voter, proposal_id, support, weight)` | voter | Cast a vote with caller-declared weight |
| `finalize(proposal_id)` | — | Tally against quorum and the pass threshold once the deadline has passed; refunds any proposal deposit regardless of outcome |
| `cancel_proposal(caller, proposal_id)` | proposer or admin | Withdraw a proposal before it has received a single vote |
| `set_quorum(quorum)` | admin | Minimum total votes before `finalize` evaluates a proposal; `0` disables the check |
| `set_pass_threshold_bps(bps)` | admin | Fraction of cast votes, in basis points, that must be `votes_for` to pass; default 5,001 (>50%) |
| `set_proposal_deposit(token, amount)` | admin | Optional anti-spam deposit `propose` collects and `finalize` always refunds |
| `propose_admin(new_admin)` / `accept_admin()` / `cancel_admin_transfer()` | admin / new_admin / admin | Two-step admin handover; the admin has no other powers in Phase 1 |
| `get_proposal(id)`, `has_voted(id, voter)`, `proposal_count()`, `list_proposals(start, limit)`, `quorum()`, `pass_threshold_bps()`, `deposit_token()`, `deposit_amount()`, `admin()`, `pending_admin()` | — | Read-only queries |

## Errors (`GovernanceError`)

| Variant | Raised by |
|---|---|
| `AlreadyInitialized` | Reserved; unreachable since `initialize` was replaced by the constructor |
| `NotInitialized` | Any entry point, if ever called before construction |
| `ProposalNotFound` | Any lookup against an id that was never assigned |
| `AlreadyVoted` | `vote`, called twice by the same voter on the same proposal |
| `ProposalNotActive` | `vote`/`finalize`/`cancel_proposal`, once a proposal is no longer `Active` |
| `VotingClosed` | `vote`, after the deadline |
| `VotingNotClosed` | `finalize`, before the deadline |
| `InvalidWeight` | `vote`, on a zero or negative weight |
| `Overflow` | Any checked arithmetic operation overflowing |
| `InvalidVotingPeriod` | `propose`, outside `MIN_VOTING_PERIOD_LEDGERS..=MAX_VOTING_PERIOD_LEDGERS` |
| `TitleTooLong` | `propose`, over `MAX_TITLE_BYTES` |
| `NotAuthorizedToCancel` | `cancel_proposal`, called by neither the proposer nor the admin |
| `ProposalAlreadyHasVotes` | `cancel_proposal`, once a vote has been cast |
| `PageTooLarge` | `list_proposals`, over `MAX_PAGE_SIZE` |
| `InvalidDepositAmount` | `set_proposal_deposit`, on a negative amount |
| `InvalidQuorum` | `set_quorum`, on a negative value |
| `InvalidPassThresholdBps` | `set_pass_threshold_bps`, outside `MIN_PASS_THRESHOLD_BPS..=MAX_PASS_THRESHOLD_BPS` |
| `DuplicateActiveProposal` | `propose`, matching another active proposal's title and description hash |
| `NoPendingAdminTransfer` | `accept_admin`/`cancel_admin_transfer`, with nothing proposed |

Discriminants are frozen once a contract using them is deployed (ADR-003) —
listed here without their numeric codes because, as of this writing,
`contracts/governance/src/error.rs` assigns the same discriminant to several
of the variants above (a pre-existing defect tracked separately from #151,
outside this document's scope to fix).

## Events

| Event | Published by |
|---|---|
| `ProposalCreated` | `propose` |
| `VoteCast` | `vote` |
| `ProposalFinalized` | `finalize` |
| `ProposalDepositRefunded` | `finalize`, when a deposit was collected |
| `ProposalCancelled` | `cancel_proposal` |
| `DepositConfigured` | `set_proposal_deposit` |
| `QuorumSet` | `set_quorum` |
| `PassThresholdSet` | `set_pass_threshold_bps` |
| `AdminTransferProposed`, `AdminTransferred`, `AdminTransferCancelled` | `propose_admin`, `accept_admin`, `cancel_admin_transfer` |
