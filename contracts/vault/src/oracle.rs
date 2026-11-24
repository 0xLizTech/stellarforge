//! SEP-40 Oracle Interface and Multi-Source Aggregation types for the Vault.

use soroban_sdk::{contractclient, contracttype, Address, Env, Symbol};

/// Basis points in 100%.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// Fixed output decimals for all NAV reads (7 decimals matching standard Stellar precision).
pub const NAV_DECIMALS: u32 = 7;

/// SEP-40 asset identifier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

/// SEP-40 price record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

/// An oracle source consisting of the oracle contract address and asset identifier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSource {
    pub oracle: Address,
    pub asset: Asset,
}

/// Multi-source oracle configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSet {
    pub primary: OracleSource,
    pub secondary: Option<OracleSource>,
    pub max_staleness_secs: u64,
    pub max_divergence_bps: u32,
}

/// Structured NAV response.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavData {
    pub value: i128,
    pub decimals: u32,
    pub timestamp: u64,
}

#[contractclient(name = "Sep40OracleClient")]
pub trait Sep40OracleInterface {
    fn base(env: Env) -> Asset;
    fn decimals(env: Env) -> u32;
    fn lastprice(env: Env, asset: Asset) -> Option<PriceData>;
}
