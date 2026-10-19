//! Contract event definitions.
//!
//! Without events an indexer, explorer or wallet has no way to reconstruct
//! holder positions short of replaying every transaction and re-simulating
//! contract state. Topic lists follow the Stellar token conventions so
//! existing tooling can consume them unmodified: the operation name first,
//! then the addresses involved, with the amount as the payload.

use soroban_sdk::{contractevent, Address, Bytes};

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

/// Carries both the previous and new metadata in full, so the legal document
/// hash an asset used to point at stays recoverable from the event stream
/// alone, and `version`, so an integrator can tell two reads of `metadata()`
/// apart (or confirm nothing changed) without diffing every field (#149).
/// `version` is `metadata()`'s version *after* this change: 2 the first time
/// this event fires, since construction itself counts as version 1.
#[contractevent(topics = ["update_metadata"], data_format = "map")]
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataUpdated {
    pub previous: AssetMetadata,
    pub metadata: AssetMetadata,
    pub version: u32,
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

#[contractevent(topics = ["accept_admin"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub previous: Address,
    pub new_admin: Address,
}

/// Published by `cancel_admin_transfer`.
#[contractevent(topics = ["cancel_admin_transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferCancelled {
    #[topic]
    pub cancelled_proposed_admin: Address,
}

/// `compliance` is `None` when screening was switched off, which is the case
/// ADR-004 singles out for monitoring.
#[contractevent(topics = ["set_compliance"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceSet {
    pub compliance: Option<Address>,
    pub min_level: u32,
}

// ─── Supply cap governance (#148) ────────────────────────────────────────────

/// Published by `set_governance`. `governance` is `None` when loosening a
/// supply cap was disabled, which — like `ComplianceSet`'s `None` — is worth
/// a monitor's attention: it is a privileged capability turning on or off.
#[contractevent(topics = ["set_governance"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceSet {
    pub governance: Option<Address>,
}

/// Published by `propose_cap_increase`. Approval, once it happens, is visible
/// as an ordinary `MetadataUpdated` (with the new `max_supply` in `metadata`)
/// rather than a second event type, so a monitor only has to watch one event
/// for every way metadata can change.
#[contractevent(topics = ["propose_cap_increase"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapIncreaseProposed {
    pub proposed_max_supply: i128,
}

/// Published by `cancel_cap_increase`.
#[contractevent(topics = ["cancel_cap_increase"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapIncreaseCancelled {
    pub cancelled_proposed_max_supply: i128,
}

// ─── Forced transfer (#144) ──────────────────────────────────────────────────

/// Published by `forced_transfer`, a court-ordered recovery executed by the
/// dedicated forced-transfer role. Distinct from `Transfer` so indexers and
/// monitors can isolate legally compelled movements from ordinary ones. The
/// `reason_hash` commits to the off-chain legal order without publishing it.
/// `Bytes`, not a fixed-size hash type, matching `forced_transfer`'s
/// parameter: `InvalidReasonHash` rejects it empty, but does not require any
/// particular length.
#[contractevent(topics = ["forced_transfer"], data_format = "map")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForcedTransfer {
    #[topic]
    pub operator: Address,
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
    pub reason_hash: Bytes,
}

// ─── Frozen Address & Lock-up Date ────────────────────────────────────────────

/// Published by `set_frozen`. Monitors can track which addresses have been
/// frozen to comply with sanctions or other regulatory requirements.
#[contractevent(topics = ["set_frozen"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenSet {
    #[topic]
    pub address: Address,
    pub frozen: bool,
}

/// Published by `set_lockup_date`. Monitors can track lock-up periods for
/// holders, which are often required for regulatory or vesting purposes.
#[contractevent(topics = ["set_lockup_date"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockupDateSet {
    #[topic]
    pub address: Address,
    pub lockup_until_ledger: u32,
}
