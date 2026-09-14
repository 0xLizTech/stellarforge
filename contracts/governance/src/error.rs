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
}
