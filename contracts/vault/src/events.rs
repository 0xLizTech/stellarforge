//! Contract event definitions for the vault contract.
//!
//! Topic lists and data formats match Stellar token conventions (SEP-41)
//! and `rwa-asset`, ensuring existing indexers and explorers consume them unmodified.

use soroban_sdk::{contractevent, Address};

/// Published by `transfer`, in SEP-41's current format: the amount, plus the
/// recipient's muxed id when `to` was a muxed address.
#[contractevent(topics = ["transfer"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
    pub to_muxed_id: Option<u64>,
}

/// Published by `transfer_from`. SEP-41 specifies the amount alone here,
/// because `transfer_from` takes no muxed address.
#[contractevent(topics = ["transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferFrom {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

/// SEP-41 format: data is `[amount, live_until_ledger]`.
#[contractevent(topics = ["approve"], data_format = "vec")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approve {
    #[topic]
    pub owner: Address,
    #[topic]
    pub spender: Address,
    pub amount: i128,
    pub live_until_ledger: u32,
}

#[contractevent(topics = ["paused"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paused {
    pub paused: bool,
}

#[contractevent(topics = ["transfer_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}

/// Published by `set_compliance`. `compliance` is `None` when screening was
/// switched off, which is the case ADR-004 singles out for monitoring.
#[contractevent(topics = ["set_compliance"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceSet {
    pub compliance: Option<Address>,
    pub min_level: u32,
}

/// Published by `set_max_share_supply`. Carries both the previous and new cap
/// so a monitor can reconstruct cap changes from the event stream alone.
#[contractevent(topics = ["set_max_share_supply"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaxShareSupplySet {
    pub previous: i128,
    pub max_share_supply: i128,
}
