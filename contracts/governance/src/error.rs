use soroban_sdk::contracterror;

/// Failure modes of the governance contract.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovernanceError {
    /// Reserved. Raised by the former `initialize` entry point, which a
    /// constructor replaced; a deployed contract is configured exactly once,
    /// by the deploy transaction itself, so nothing can raise this now.
    /// Retained rather than removed because discriminants are frozen
    /// (see ADR-003) and 1 must not be reused for anything else.
    AlreadyInitialized = 1,
    /// Admin state was absent when an entry point needed it. Unreachable in
    /// normal operation now that the constructor sets it during deployment;
    /// kept as a defined failure rather than an `unwrap` so the cause is
    /// legible if it ever does happen.
    NotInitialized = 2,
    /// No proposal exists with the given id.
    ProposalNotFound = 3,
    /// This voter has already cast a vote on this proposal.
    AlreadyVoted = 4,
    /// The proposal is no longer accepting votes or finalisation.
    ProposalNotActive = 5,
    /// The voting deadline has passed.
    VotingClosed = 6,
    /// The voting deadline has not yet passed.
    VotingNotClosed = 7,
    /// Vote weight was zero or negative.
    InvalidWeight = 8,
    /// An arithmetic operation overflowed.
    Overflow = 9,
    /// `propose` was given a voting period outside the allowed range.
    InvalidVotingPeriod = 10,
    /// `propose` was given a title longer than `MAX_TITLE_BYTES`.
    TitleTooLong = 11,
    /// `cancel_proposal` was called by neither the proposer nor the admin.
    NotAuthorizedToCancel = 12,
    /// `cancel_proposal` was called after the proposal had already received a
    /// vote.
    ProposalAlreadyHasVotes = 13,
    /// `list_proposals` was given a `limit` above `MAX_PAGE_SIZE`.
    PageTooLarge = 14,
    /// `set_proposal_deposit` was given a negative amount.
    InvalidDepositAmount = 15,
    /// `accept_admin` was called with no transfer pending, or after it was
    /// cancelled or already accepted.
    NoPendingAdminTransfer = 16,
    /// `set_quorum` was given a negative value.
    InvalidQuorum = 17,
    /// `set_pass_threshold_bps` was given a value outside
    /// `MIN_PASS_THRESHOLD_BPS..=MAX_PASS_THRESHOLD_BPS`.
    InvalidPassThresholdBps = 18,
    /// `propose` matches another currently-active proposal's title and
    /// description hash.
    DuplicateActiveProposal = 19,
}
