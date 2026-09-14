#![no_std]

//! Oracle adapter: a SEP-40 price feed for net asset values.
//!
//! The Phase 2 vault prices shares from an asset's NAV (FR-06-4). Public
//! Stellar oracles publish market prices for liquid assets. Nobody publishes
//! the appraised value of a building or a loan book, so a StellarForge asset
//! needs a feed its own appraisers can report to. This contract is that feed.
//!
//! # SEP-40 is the interface
//!
//! The read side is SEP-40's consumer interface, byte for byte: `base`,
//! `assets`, `decimals`, `resolution`, `price`, `prices` and `lastprice`, with
//! its `Asset` and `PriceData` types. A consumer written against SEP-40 reads
//! this feed and any other SEP-40 oracle, such as Reflector, the same way.
//! That is the "standardized interface" the roadmap asks for: the vault binds
//! to SEP-40, not to this contract (ADR-005).
//!
//! # What this contract guards, and what it leaves to consumers
//!
//! It guards what is being recorded:
//!
//! - Only a reporter the admin has authorized **for that asset** can report it.
//! - Prices are positive, and not timestamped after the ledger closed.
//! - History is append-only. A report must land in a later resolution tick
//!   than the latest one, so a past price can never be rewritten.
//! - A reported price may move at most `max_deviation_bps` from the latest
//!   price. A larger move needs the admin's `override_price`.
//!
//! It does not judge freshness. A NAV reported once a quarter is current for
//! one consumer and stale for another, so SEP-40 leaves the staleness check to
//! the consumer, who compares `PriceData::timestamp` against the ledger time.
//!
//! # The deviation limit bounds each report, not each day
//!
//! One report per resolution tick can each move the price by the limit, so a
//! compromised reporter can compound it: at a 10% limit and a one-hour
//! resolution, the price can double in under eight hours. Deploy NAV feeds
//! with a resolution matched to how often the value really changes. Consumers
//! should still apply their own bounds.

mod error;
mod events;

pub use error::OracleError;
pub use events::{
    AdminTransferred, AssetAdded, MaxDeviationSet, PriceOverridden, PriceReported, ReporterSet,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const BASE_KEY: Symbol = symbol_short!("BASE");
const DECIMALS_KEY: Symbol = symbol_short!("DECIMALS");
const RESOLUTION_KEY: Symbol = symbol_short!("RES");

/// Most decimals the constructor accepts. At 18, an `i128` still holds prices
/// up to about 1.7e20 whole units of the base asset, far past any real NAV.
pub const MAX_DECIMALS: u32 = 18;

/// Most assets one feed lists. `assets` returns them all in one read, so the
/// list is bounded to keep that read inside the network's resource limits.
pub const MAX_ASSETS: u32 = 100;

/// Most records `prices` returns. A larger request is clamped, as Reflector
/// does.
pub const MAX_RECORDS: u32 = 20;

/// Basis points in 100%.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// SEP-40's asset identifier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    /// A Stellar asset or Soroban token, by contract address.
    Stellar(Address),
    /// Anything else, such as a fiat currency (`USD`) or an off-chain asset.
    Other(Symbol),
}

/// SEP-40's price record. `price` is scaled by `10^decimals()`, and
/// `timestamp` is Unix seconds rounded down to a multiple of `resolution()`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetConfig {
    /// Largest move from the latest price a reporter may record, in basis
    /// points of the latest price.
    pub max_deviation_bps: u32,
}

/// A stored price, linked to the one recorded before it so `prices` can walk
/// back through history that has gaps.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub price: i128,
    pub previous_timestamp: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// `Vec<Asset>`, in the order the assets were added.
    Assets,
    /// `AssetConfig`. Present exactly when the asset is listed.
    Config(Asset),
    /// `bool`. Present exactly when the address may report the asset.
    Reporter(Asset, Address),
    /// `PriceData`, the most recent price.
    Latest(Asset),
    /// `Report`, the price recorded for one resolution tick.
    Report(Asset, u64),
}

#[contract]
pub struct OracleAdapterContract;

#[contractimpl]
impl OracleAdapterContract {
    /// Configures the feed as part of the deploy transaction.
    ///
    /// `base` is what prices are quoted in, typically `Other("USD")`. `decimals`
    /// and `resolution` are fixed for the life of the contract: changing either
    /// would silently reinterpret every price already recorded.
    pub fn __constructor(env: Env, admin: Address, base: Asset, decimals: u32, resolution: u32) {
        admin.require_auth();
        if decimals > MAX_DECIMALS {
            panic_with_error!(&env, OracleError::InvalidDecimals);
        }
        if resolution == 0 {
            panic_with_error!(&env, OracleError::InvalidResolution);
        }

        let instance = env.storage().instance();
        instance.set(&ADMIN_KEY, &admin);
        instance.set(&BASE_KEY, &base);
        instance.set(&DECIMALS_KEY, &decimals);
        instance.set(&RESOLUTION_KEY, &resolution);

        extend_instance(&env);
    }

    // ─── SEP-40 consumer interface ──────────────────────────────────────────

    /// The asset prices are quoted in.
    pub fn base(env: Env) -> Asset {
        Self::instance_get(&env, &BASE_KEY)
    }

    /// Every listed asset, in the order they were added.
    pub fn assets(env: Env) -> Vec<Asset> {
        env.storage()
            .persistent()
            .get(&DataKey::Assets)
            .unwrap_or(Vec::new(&env))
    }

    /// Decimal places in every `PriceData::price`.
    pub fn decimals(env: Env) -> u32 {
        Self::instance_get(&env, &DECIMALS_KEY)
    }

    /// Length of one tick in seconds. Recorded timestamps are multiples of it.
    pub fn resolution(env: Env) -> u32 {
        Self::instance_get(&env, &RESOLUTION_KEY)
    }

    /// The price recorded in the tick containing `timestamp`, if any.
    ///
    /// This is an exact tick lookup, not "the latest price at or before
    /// `timestamp`". A tick with no report returns `None`, as SEP-40 requires.
    pub fn price(env: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let tick = Self::tick(&env, timestamp);
        env.storage()
            .persistent()
            .get::<_, Report>(&DataKey::Report(asset, tick))
            .map(|report| PriceData {
                price: report.price,
                timestamp: tick,
            })
    }

    /// Up to `records` of the most recent prices, newest first.
    ///
    /// Returns reports, not ticks: a tick nobody reported in is skipped rather
    /// than counted, which is what a sparsely updated NAV feed needs. Requests
    /// above `MAX_RECORDS` are clamped. `None` when there is nothing to return.
    pub fn prices(env: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let storage = env.storage().persistent();
        let limit = records.min(MAX_RECORDS);
        let mut out = Vec::new(&env);
        let mut next = storage
            .get::<_, PriceData>(&DataKey::Latest(asset.clone()))
            .map(|latest| latest.timestamp);

        while out.len() < limit {
            let Some(timestamp) = next else { break };
            let Some(report) = storage.get::<_, Report>(&DataKey::Report(asset.clone(), timestamp))
            else {
                break;
            };
            out.push_back(PriceData {
                price: report.price,
                timestamp,
            });
            next = report.previous_timestamp;
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    /// The most recent price. Check its `timestamp` before relying on it.
    pub fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
        env.storage().persistent().get(&DataKey::Latest(asset))
    }

    // ─── Reporting ──────────────────────────────────────────────────────────

    /// Records a price for `asset`, from a reporter authorized for it.
    ///
    /// `timestamp` is when the value was observed, in Unix seconds. It is
    /// rounded down to the resolution, and the rounded tick must be later than
    /// the latest recorded one. The price may differ from the latest by at most
    /// the asset's `max_deviation_bps`.
    pub fn report(env: Env, reporter: Address, asset: Asset, price: i128, timestamp: u64) {
        reporter.require_auth();
        let config = Self::load_config(&env, &asset);

        let reporter_key = DataKey::Reporter(asset.clone(), reporter.clone());
        let allowed: bool = env
            .storage()
            .persistent()
            .get(&reporter_key)
            .unwrap_or(false);
        if !allowed {
            panic_with_error!(&env, OracleError::NotReporter);
        }

        let (timestamp, previous_price) = Self::record(
            &env,
            &asset,
            price,
            timestamp,
            Some(config.max_deviation_bps),
        );

        extend_persistent(&env, &DataKey::Config(asset.clone()));
        extend_persistent(&env, &reporter_key);

        events::PriceReported {
            asset,
            reporter,
            price,
            timestamp,
            previous_price,
        }
        .publish(&env);
    }

    /// Records a price the deviation limit would refuse. Admin only.
    ///
    /// This is the manual override PRD §17 calls for: a genuine revaluation
    /// larger than the limit would otherwise freeze the feed at a price
    /// everyone knows is wrong. Every other rule still applies, so an override
    /// cannot rewrite history or record a non-positive price.
    pub fn override_price(env: Env, asset: Asset, price: i128, timestamp: u64) {
        Self::admin(env.clone()).require_auth();
        Self::load_config(&env, &asset);

        let (timestamp, previous_price) = Self::record(&env, &asset, price, timestamp, None);

        extend_persistent(&env, &DataKey::Config(asset.clone()));

        events::PriceOverridden {
            asset,
            price,
            timestamp,
            previous_price,
        }
        .publish(&env);
    }

    // ─── Administration ─────────────────────────────────────────────────────

    /// Lists `asset`, so reporters can be authorized for it.
    ///
    /// `max_deviation_bps` must be at least 1. There is no "unlimited" zero: a
    /// feed with no limit is one typo away from any price, so disabling the
    /// check has to be spelled out as a very large value.
    pub fn add_asset(env: Env, asset: Asset, max_deviation_bps: u32) {
        Self::admin(env.clone()).require_auth();
        Self::validate_deviation(&env, max_deviation_bps);

        let config_key = DataKey::Config(asset.clone());
        if env.storage().persistent().has(&config_key) {
            panic_with_error!(&env, OracleError::AssetAlreadyExists);
        }
        let mut assets = Self::assets(env.clone());
        if assets.len() >= MAX_ASSETS {
            panic_with_error!(&env, OracleError::TooManyAssets);
        }
        assets.push_back(asset.clone());

        env.storage().persistent().set(&DataKey::Assets, &assets);
        env.storage()
            .persistent()
            .set(&config_key, &AssetConfig { max_deviation_bps });

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Assets);
        extend_persistent(&env, &config_key);

        events::AssetAdded {
            asset,
            max_deviation_bps,
        }
        .publish(&env);
    }

    /// Changes a listed asset's deviation limit.
    pub fn set_max_deviation(env: Env, asset: Asset, max_deviation_bps: u32) {
        Self::admin(env.clone()).require_auth();
        Self::validate_deviation(&env, max_deviation_bps);
        Self::load_config(&env, &asset);

        let config_key = DataKey::Config(asset.clone());
        env.storage()
            .persistent()
            .set(&config_key, &AssetConfig { max_deviation_bps });

        extend_instance(&env);
        extend_persistent(&env, &config_key);

        events::MaxDeviationSet {
            asset,
            max_deviation_bps,
        }
        .publish(&env);
    }

    /// Grants or revokes `reporter`'s right to report `asset`.
    ///
    /// Authorization is per asset, so an appraiser for one property cannot
    /// move the NAV of another.
    pub fn set_reporter(env: Env, asset: Asset, reporter: Address, allowed: bool) {
        Self::admin(env.clone()).require_auth();
        Self::load_config(&env, &asset);

        let key = DataKey::Reporter(asset.clone(), reporter.clone());
        if allowed {
            env.storage().persistent().set(&key, &true);
            extend_persistent(&env, &key);
        } else {
            env.storage().persistent().remove(&key);
        }
        extend_instance(&env);

        events::ReporterSet {
            asset,
            reporter,
            allowed,
        }
        .publish(&env);
    }

    /// Hands the admin role to `new_admin`.
    ///
    /// Both the current and the incoming admin must authorize, in the same
    /// transaction (NFR-S-4), for the reason ADR-002 gives: a handover to a key
    /// nobody controls cannot be undone.
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let previous = Self::admin(env.clone());
        previous.require_auth();
        new_admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin,
        }
        .publish(&env);
    }

    // ─── Queries ────────────────────────────────────────────────────────────

    pub fn admin(env: Env) -> Address {
        Self::instance_get(&env, &ADMIN_KEY)
    }

    pub fn is_reporter(env: Env, asset: Asset, reporter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Reporter(asset, reporter))
            .unwrap_or(false)
    }

    /// The asset's configuration, or `None` if it is not listed.
    pub fn asset_config(env: Env, asset: Asset) -> Option<AssetConfig> {
        env.storage().persistent().get(&DataKey::Config(asset))
    }

    // ─── Internals ──────────────────────────────────────────────────────────

    /// Validates and stores a price, returning the recorded (rounded) timestamp
    /// and the price it replaced. `max_deviation_bps` of `None` skips only the
    /// deviation check.
    fn record(
        env: &Env,
        asset: &Asset,
        price: i128,
        timestamp: u64,
        max_deviation_bps: Option<u32>,
    ) -> (u64, Option<i128>) {
        if price <= 0 {
            panic_with_error!(env, OracleError::InvalidPrice);
        }
        if timestamp > env.ledger().timestamp() {
            panic_with_error!(env, OracleError::TimestampInFuture);
        }
        let tick = Self::tick(env, timestamp);

        let latest_key = DataKey::Latest(asset.clone());
        let latest: Option<PriceData> = env.storage().persistent().get(&latest_key);

        if let Some(latest) = &latest {
            if tick <= latest.timestamp {
                panic_with_error!(env, OracleError::StaleReport);
            }
            if let Some(max_bps) = max_deviation_bps {
                // |price - latest| / latest > max_bps / 10_000, cross-multiplied
                // so no precision is lost to division. Both prices are positive,
                // so the subtraction cannot overflow.
                let moved = (price - latest.price).abs();
                let (Some(lhs), Some(rhs)) = (
                    moved.checked_mul(BPS_DENOMINATOR),
                    latest.price.checked_mul(i128::from(max_bps)),
                ) else {
                    panic_with_error!(env, OracleError::Overflow);
                };
                if lhs > rhs {
                    panic_with_error!(env, OracleError::DeviationTooLarge);
                }
            }
        }

        let report_key = DataKey::Report(asset.clone(), tick);
        env.storage().persistent().set(
            &report_key,
            &Report {
                price,
                previous_timestamp: latest.as_ref().map(|l| l.timestamp),
            },
        );
        env.storage().persistent().set(
            &latest_key,
            &PriceData {
                price,
                timestamp: tick,
            },
        );

        extend_instance(env);
        extend_persistent(env, &report_key);
        extend_persistent(env, &latest_key);

        (tick, latest.map(|l| l.price))
    }

    /// Rounds `timestamp` down to a multiple of the resolution.
    fn tick(env: &Env, timestamp: u64) -> u64 {
        let resolution = u64::from(Self::resolution(env.clone()));
        timestamp - timestamp % resolution
    }

    fn validate_deviation(env: &Env, max_deviation_bps: u32) {
        if max_deviation_bps == 0 {
            panic_with_error!(env, OracleError::InvalidDeviation);
        }
    }

    fn load_config(env: &Env, asset: &Asset) -> AssetConfig {
        match env
            .storage()
            .persistent()
            .get(&DataKey::Config(asset.clone()))
        {
            Some(config) => config,
            None => panic_with_error!(env, OracleError::UnknownAsset),
        }
    }

    fn instance_get<V: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
        env: &Env,
        key: &Symbol,
    ) -> V {
        match env.storage().instance().get(key) {
            Some(v) => v,
            None => panic_with_error!(env, OracleError::NotInitialized),
        }
    }
}
