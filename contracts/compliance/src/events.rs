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
