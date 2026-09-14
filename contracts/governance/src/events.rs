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

/// Published by `transfer_admin`. The incoming admin is data rather than a
/// topic, matching `rwa-asset`, so one topic filter on the outgoing address
/// finds every handover away from a key being watched.
#[contractevent(topics = ["transfer_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}
