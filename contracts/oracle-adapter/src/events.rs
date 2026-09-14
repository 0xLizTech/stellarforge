//! Contract event definitions.
//!
//! A NAV feed is only as trustworthy as the ability to audit who moved it and
//! when, so every state change publishes an event whose first topic is the
//! entry point's name, matching the Phase 1 contracts. The asset is a topic on
//! each, so an indexer can follow one feed with a single filter.

use soroban_sdk::{contractevent, Address};

use crate::Asset;

#[contractevent(topics = ["add_asset"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAdded {
    #[topic]
    pub asset: Asset,
    pub max_deviation_bps: u32,
}

#[contractevent(topics = ["set_max_deviation"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaxDeviationSet {
    #[topic]
    pub asset: Asset,
    pub max_deviation_bps: u32,
}

#[contractevent(topics = ["set_reporter"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReporterSet {
    #[topic]
    pub asset: Asset,
    #[topic]
    pub reporter: Address,
    pub allowed: bool,
}

/// `timestamp` is the recorded one, already rounded down to the resolution.
#[contractevent(topics = ["report"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceReported {
    #[topic]
    pub asset: Asset,
    #[topic]
    pub reporter: Address,
    pub price: i128,
    pub timestamp: u64,
    pub previous_price: Option<i128>,
}

/// Published by `override_price`, which bypasses the deviation limit. A
/// separate event rather than a flag on `PriceReported`, so a monitor can alert
/// on every override with one topic filter.
#[contractevent(topics = ["override_price"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceOverridden {
    #[topic]
    pub asset: Asset,
    pub price: i128,
    pub timestamp: u64,
    pub previous_price: Option<i128>,
}

/// Published by `transfer_admin`. The incoming admin is data rather than a
/// topic, matching the Phase 1 contracts.
#[contractevent(topics = ["transfer_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}
