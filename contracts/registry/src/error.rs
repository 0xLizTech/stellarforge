use soroban_sdk::contracterror;

/// Failure modes of the asset registry.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    /// `initialize` was called on an already-configured contract.
    AlreadyInitialized = 1,
    /// An entry point was reached before `initialize` ran.
    NotInitialized = 2,
    /// No entry exists for the requested contract address.
    AssetNotFound = 3,
}
