//! Contract event definitions.
//!
//! The registry is what indexers and front ends read to decide which assets to
//! show, and ADR-003 names `set_active` as the closest thing to a kill switch
//! during a migration. Both changes must be observable as they happen rather
//! than by re-reading every entry (IR-03). Topic lists follow the convention
//! used across the protocol: the entry point name first, then the asset.

use soroban_sdk::{contractevent, Address, String};

/// `is_new` distinguishes an asset joining the directory from a correction to
/// one already listed, so an indexer can track the directory's size without
/// querying `asset_count`.
#[contractevent(topics = ["register"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistered {
    #[topic]
    pub contract: Address,
    pub asset_class: String,
    pub active: bool,
    pub is_new: bool,
}

#[contractevent(topics = ["set_active"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveSet {
    #[topic]
    pub contract: Address,
    pub active: bool,
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
