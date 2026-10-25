use soroban_sdk::contracterror;

/// Failure modes of the RWA asset contract.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RwaError {
    /// Reserved. Raised by the former `initialize` entry point, which a
    /// constructor replaced; a deployed contract is configured exactly once,
    /// by the deploy transaction itself, so nothing can raise this now.
    /// Retained rather than removed because discriminants are frozen
    /// (see ADR-003) and 1 must not be reused for anything else.
    AlreadyInitialized = 1,
    /// Admin state was absent when an entry point needed it. Unreachable in
    /// normal operation now that the constructor sets it during deployment;
    /// kept as a defined failure rather than an `unwrap` so the cause is
    /// legible if it ever does happen.
    NotInitialized = 2,
    /// Caller is not a registered issuer.
    NotIssuer = 3,
    /// Amount was zero or negative.
    InvalidAmount = 4,
    /// Holder's balance is below the requested amount.
    InsufficientBalance = 5,
    /// Spender's allowance is below the requested amount.
    InsufficientAllowance = 6,
    /// Minting the amount would push total supply past `max_supply`.
    ExceedsMaxSupply = 7,
    /// The contract is paused; state-changing calls are refused.
    ContractPaused = 8,
    /// An arithmetic operation overflowed `i128`.
    Overflow = 9,
    /// Supplied metadata failed validation.
    InvalidMetadata = 10,
    /// A party to the transfer lacks the required verification level.
    NotCompliant = 11,
    /// `update_metadata` tried to change `decimals`, which would re-denominate
    /// every existing balance.
    DecimalsImmutable = 12,
    /// `update_metadata` tried to raise or remove a supply cap, or set one below
    /// the circulating supply.
    InvalidSupplyCap = 13,
    /// `approve` set a non-zero allowance whose `live_until_ledger` has
    /// already passed.
    InvalidExpiration = 14,
    /// `set_compliance` was given a `min_level` above `MAX_COMPLIANCE_LEVEL`.
    /// A compliance record's own level is capped at the same scale
    /// (`compliance::MAX_LEVEL`), so a `min_level` above it could never be
    /// satisfied by any verified party: every transfer would silently and
    /// permanently fail `NotCompliant` until an admin reconfigured it.
    InvalidComplianceLevel = 15,
    /// `legal_doc_hash` was not exactly 32 bytes, or was all-zero.
    InvalidLegalDocHash = 16,
    /// `approve_cap_increase` was called with no `governance` address
    /// configured via `set_governance`.
    GovernanceNotConfigured = 17,
    /// `approve_cap_increase` or `cancel_cap_increase` was called with no
    /// cap increase proposed, or after it was already approved or cancelled.
    NoPendingCapIncrease = 18,
    /// `accept_admin` was called with no transfer pending, or after it was
    /// cancelled or already accepted. Previously misnumbered `15`, colliding
    /// with `InvalidComplianceLevel` (two variants sharing one on-chain error
    /// code, and a hard `E0081` compile error under current Rust). Moved
    /// here rather than left colliding: nothing has been deployed with the
    /// old value, so ADR-003's discriminant freeze (which protects a
    /// *deployed* contract's clients) does not apply yet.
    NoPendingAdminTransfer = 19,
    /// Caller is not the dedicated forced-transfer role (#144).
    NotForcedTransferRole = 20,
    /// `forced_transfer` was called with an empty reason hash.
    InvalidReasonHash = 21,
    /// Reserved. Not currently raised: `forced_transfer` treats a
    /// same-address call as a no-op that still publishes its event, the same
    /// way `transfer`/`transfer_from` treat a self-transfer elsewhere in this
    /// contract, rather than rejecting it.
    InvalidForcedTransferParty = 22,
    /// Sender address is frozen and cannot transfer tokens.
    AddressFrozen = 23,
    /// Sender address's tokens are locked until the lock-up date passes.
    AddressLocked = 24,
    /// Minting or transferring would push the number of distinct holders past
    /// the configured holder cap (`set_holder_cap`). Reg D and similar rules
    /// bound the holder count, not the supply.
    ExceedsHolderCap = 25,
}
