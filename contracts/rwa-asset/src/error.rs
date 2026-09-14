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
    /// `update_metadata` tried to remove a supply cap, or set one below the
    /// circulating supply.
    InvalidSupplyCap = 13,
    /// `approve` set a non-zero allowance whose `live_until_ledger` has
    /// already passed.
    InvalidExpiration = 14,
}
