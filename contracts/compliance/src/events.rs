//! Contract event definitions.
//!
//! A KYC decision is the input every screened transfer depends on. Without an
//! event, a monitor can only notice an address being approved or revoked by
//! polling `get_kyc` for every address it cares about (IR-03). Topic lists
//! follow the same convention as `rwa-asset`: the entry point name first, then
//! the address acted on, with the record as the payload.

use soroban_sdk::{contractevent, Address};

use crate::KycRecord;

#[contractevent(topics = ["set_kyc"], data_format = "single-value")]
#[derive(Clone, Debug, PartialEq)]
pub struct KycSet {
    #[topic]
    pub subject: Address,
    pub record: KycRecord,
}

/// Carries the record that was removed. Published only when one existed:
/// revoking an unknown subject changes nothing, so it announces nothing.
#[contractevent(topics = ["revoke_kyc"], data_format = "single-value")]
#[derive(Clone, Debug, PartialEq)]
pub struct KycRevoked {
    #[topic]
    pub subject: Address,
    pub record: KycRecord,
}

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

/// Published by `set_paused`, matching `rwa-asset::events::Paused`.
#[contractevent(topics = ["paused"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paused {
    pub paused: bool,
/// Published by `cancel_admin_transfer`.
#[contractevent(topics = ["cancel_admin_transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferCancelled {
    #[topic]
    pub cancelled_proposed_admin: Address,
}
