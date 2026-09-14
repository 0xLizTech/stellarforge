//! Contract event definitions.
//!
//! Without events an indexer, explorer or wallet has no way to reconstruct
//! holder positions short of replaying every transaction and re-simulating
//! contract state. Topic lists follow the Stellar token conventions so
//! existing tooling can consume them unmodified: the operation name first,
//! then the addresses involved, with the amount as the payload.

use soroban_sdk::{contractevent, Address};

use crate::AssetMetadata;

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

#[contractevent(topics = ["mint"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mint {
    #[topic]
    pub issuer: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent(topics = ["burn"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Burn {
    #[topic]
    pub from: Address,
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

// ─── Administrative events ───────────────────────────────────────────────────
//
// IR-03. These are the changes a monitor most needs to see — who may mint, who
// holds the admin role, what the token claims to represent, and whether
// transfers are screened at all — and before these existed none of them left
// a trace short of re-reading contract state.

#[contractevent(topics = ["set_issuer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerSet {
    #[topic]
    pub issuer: Address,
    pub approved: bool,
}

/// Carries both versions, so the legal document hash an asset used to point
/// at stays recoverable from the event stream alone.
#[contractevent(topics = ["update_metadata"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataUpdated {
    pub previous: AssetMetadata,
    pub metadata: AssetMetadata,
}

#[contractevent(topics = ["transfer_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}

/// `compliance` is `None` when screening was switched off, which is the case
/// ADR-004 singles out for monitoring.
#[contractevent(topics = ["set_compliance"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceSet {
    pub compliance: Option<Address>,
    pub min_level: u32,
}
