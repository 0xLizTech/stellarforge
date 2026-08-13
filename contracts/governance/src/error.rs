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
    /// `initialize` was called on an already-configured contract.
    AlreadyInitialized = 1,
    /// An entry point was reached before `initialize` ran.
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
}
