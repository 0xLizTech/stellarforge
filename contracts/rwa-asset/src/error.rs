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
    /// `initialize` was called on an already-configured contract.
    AlreadyInitialized = 1,
    /// An entry point was reached before `initialize` ran.
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
}
