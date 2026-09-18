use soroban_sdk::contracterror;

/// Failure modes of the vault contract.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong (see ADR-003).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    /// Configuration or admin state was absent when an entry point needed it.
    NotInitialized = 1,
    /// `exchange_rate` was less than 1.
    InvalidExchangeRate = 2,
    /// `max_share_supply` was negative.
    InvalidMaxSupply = 3,
    /// `lockup_secs` exceeded `MAX_LOCKUP_SECS`.
    InvalidLockup = 4,
    /// `name` or `symbol` was empty.
    InvalidMetadata = 5,
    /// Underlying contract failed decimals call or is not a valid token.
    InvalidUnderlying = 6,
    /// The contract is paused; state-changing calls are refused.
    ContractPaused = 7,
    /// Amount was zero or negative.
    InvalidAmount = 8,
    /// Holder's balance is below the requested amount.
    InsufficientBalance = 9,
    /// Spender's allowance is below the requested amount.
    InsufficientAllowance = 10,
    /// An arithmetic operation overflowed `i128`.
    Overflow = 11,
    /// `approve` set a non-zero allowance whose `live_until_ledger` has already passed.
    InvalidExpiration = 12,
    /// A party to the transfer lacks the required verification level.
    NotCompliant = 13,
    /// The owner's shares are currently locked (`now < locked_until(from)`).
    SharesLocked = 14,
    /// Minting shares would exceed `max_share_supply`.
    ExceedsMaxSupply = 15,
}
