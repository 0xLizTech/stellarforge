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
