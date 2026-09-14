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
