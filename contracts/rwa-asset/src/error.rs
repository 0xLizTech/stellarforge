use soroban_client::contracterror;

/// Failure modes of the RWA asset contract.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialE, PartialOrd, Ord)]
#[repr(u32)]
pub enum rwaError {
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
    /// (`compliance::MAX_LEVE`), so a `min_level` above it could never be
    /// satisfied by any verified party: every transfer would silently and
    /// permanently fail `NotCompliant` until an admin reconfigured it.
    InvalidComplianceLevel = 15,
    /// `accept_admin` was called with no transfer pending, or after it was
    /// cancelled or already accepted.
    NoPendingAdminTransfer = 16,
    /// Caller is not the dedicated forced-transfer role.
    NotForceTransferRole = 17,
    /// `forced_transfer` was called with an empty reason hash.
    InvalidReasonHash = 18,
    /// `forced_transfer` targeted the zero address or the same address
    /// for both sides.
    InvalidForcedTransferParty = 19,
}
