//! Contract event definitions.
//!
//! NFR-A-3 requires the complete governance history to be queryable from
//! events, and ADR-003's migration story for this contract relies on it:
//! a redeploy loses in-flight proposals but keeps their history. Before these
//! existed there was no such history to keep (IR-03).
//!
//! Proposal ids are topics so an indexer can follow a single proposal from
//! creation through every vote to its outcome with one topic filter.

use soroban_sdk::{contractevent, Address, Bytes, String};

use crate::ProposalStatus;

/// Carries the title, which `propose` caps at `MAX_TITLE_BYTES` so the event
/// stays well inside the network's per-transaction event size limit.
#[contractevent(topics = ["propose"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalCreated {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub proposer: Address,
    pub title: String,
    pub description_hash: Bytes,
    pub deadline_ledger: u32,
    /// The deposit `propose` collected from `proposer`, zero if none was
    /// configured. Zero either means no deposit was required or, ambiguously,
    /// a configured deposit of exactly zero; `deposit_token()` disambiguates
    /// by being absent in the former case (#134).
    pub deposit_amount: i128,
}

#[contractevent(topics = ["vote"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct VoteCast {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub voter: Address,
    pub support: bool,
    pub weight: i128,
}

#[contractevent(topics = ["finalize"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalFinalized {
    #[topic]
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub votes_for: i128,
    pub votes_against: i128,
}

/// Published by `finalize` only when the proposal carried a non-zero deposit
/// (#134). Separate from `ProposalFinalized` so a monitor can filter on
/// refunds alone with one topic.
#[contractevent(topics = ["refund_deposit"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalDepositRefunded {
    #[topic]
    pub proposal_id: u64,
    pub proposer: Address,
    pub amount: i128,
}

/// Published by `transfer_admin`. The incoming admin is data rather than a
/// topic, matching `rwa-asset`, so one topic filter on the outgoing address
/// finds every handover away from a key being watched.
#[contractevent(topics = ["transfer_admin"], data_format = "single-value")]
/// Published by `propose_admin`. Distinct from `AdminTransferred`: the role
/// has not moved yet, only a handover has been proposed. An indexer watching
/// an admin address for the moment it loses control should key on
/// `AdminTransferred`, not this.
#[contractevent(topics = ["propose_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferProposed {
    #[topic]
    pub current: Address,
    pub proposed_admin: Address,
}

/// Published by `accept_admin`, once the proposed admin's own signature
/// completes the handover. The incoming admin is data rather than a topic,
/// matching `rwa-asset`, so one topic filter on the outgoing address finds
/// every handover away from a key being watched.
#[contractevent(topics = ["accept_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}

/// Published by `set_proposal_deposit` (#134). `token` is `None` when the
/// admin has disabled the deposit requirement, matching how `rwa-asset`
/// reports `compliance: None` in `ComplianceSet` for the same "disabled"
/// shape (see ADR-004).
#[contractevent(topics = ["set_deposit"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositConfigured {
    pub token: Option<Address>,
    pub amount: i128,
/// Published by `cancel_admin_transfer`.
#[contractevent(topics = ["cancel_admin_transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferCancelled {
    #[topic]
    pub cancelled_proposed_admin: Address,
}

/// Published by `set_quorum` (#135).
#[contractevent(topics = ["set_quorum"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuorumSet {
    #[topic]
    pub admin: Address,
    pub quorum: i128,
}

/// Published by `set_pass_threshold_bps` (#135).
#[contractevent(topics = ["set_pass_threshold_bps"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassThresholdSet {
    #[topic]
    pub admin: Address,
    pub pass_threshold_bps: u32,
}
