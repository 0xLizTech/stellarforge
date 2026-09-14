use soroban_sdk::contracterror;

/// Failure modes of the oracle adapter.
///
/// Discriminants are part of the contract's public interface: clients match on
/// the numeric code, so existing variants must keep their values and new ones
/// must be appended. Renumbering silently changes what a deployed client
/// believes went wrong.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    /// Configuration was absent when an entry point needed it. Unreachable in
    /// normal operation, since the constructor sets it during deployment; kept
    /// as a defined failure rather than an `unwrap` so the cause is legible if
    /// it ever does happen.
    NotInitialized = 1,
    /// The constructor was given more than `MAX_DECIMALS` decimals.
    InvalidDecimals = 2,
    /// The constructor was given a resolution of zero seconds.
    InvalidResolution = 3,
    /// The asset has not been added with `add_asset`.
    UnknownAsset = 4,
    /// `add_asset` was given an asset that is already listed.
    AssetAlreadyExists = 5,
    /// `add_asset` would list more than `MAX_ASSETS` assets.
    TooManyAssets = 6,
    /// The caller is not an authorized reporter for this asset.
    NotReporter = 7,
    /// A price was zero or negative.
    InvalidPrice = 8,
    /// A price was timestamped after the current ledger's close time.
    TimestampInFuture = 9,
    /// A price's timestamp, rounded down to the resolution, is not after the
    /// latest recorded price. History is append-only.
    StaleReport = 10,
    /// A reported price moved further from the latest price than the asset's
    /// `max_deviation_bps` allows. Only `override_price` can record it.
    DeviationTooLarge = 11,
    /// A maximum deviation of zero basis points was given.
    InvalidDeviation = 12,
    /// An arithmetic operation overflowed.
    Overflow = 13,
}
