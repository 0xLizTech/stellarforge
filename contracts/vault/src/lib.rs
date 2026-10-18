#![no_std]

//! Institutional-grade fractionalization vault contract for Stellar/Soroban.
//!
//! The vault wraps an underlying RWA asset into fractional shares.
//! Each deployment wraps exactly one underlying `rwa-asset` (ADR-006).
//!
//! Shares follow SEP-41 (balance, spendable allowance, transfer with muxed
//! addresses, name, symbol, decimals) with one deliberate protocol invariant:
//! **there is no public `burn` or `burn_from`**. Burning shares outside of an
//! authenticated redemption would permanently strand underlying assets in
//! the vault that no share could ever redeem.

mod compliance;
mod error;
mod events;
mod oracle;
#[cfg(test)]
mod test;
mod token;

pub use compliance::{ComplianceClient, ComplianceInterface};
pub use error::VaultError;
pub use events::{AdminTransferred, Approve, Deposit, Paused, Redeem, SetOracles, Transfer, TransferFrom};
pub use oracle::{
    Asset, NavData, OracleSet, OracleSource, PriceData, Sep40OracleClient, Sep40OracleInterface,
    BPS_DENOMINATOR, NAV_DECIMALS,
};
pub use token::{UnderlyingTokenClient, UnderlyingTokenInterface};

use stellarforge_common::{extend_instance, extend_persistent};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env,
    MuxedAddress, String, Symbol, Vec,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
const CONFIG_KEY: Symbol = symbol_short!("CONFIG");
const DECIMALS_KEY: Symbol = symbol_short!("DECIMALS");
const ORACLES_KEY: Symbol = symbol_short!("ORACLES");

/// Upper bound on `VaultConfig::lockup_secs`: 10 years (315,360,000 seconds).
/// Beyond this, positions are effectively locked indefinitely.
pub const MAX_LOCKUP_SECS: u64 = 10 * 365 * 24 * 60 * 60;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
    TotalSupply,
    UnderlyingHeld,
    Allowance(Address, Address),
    LockedUntil(Address),
}

/// A spender's allowance and the last ledger it may be spent in (SEP-41).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    /// After this ledger the allowance reads as zero.
    pub live_until_ledger: u32,
}

// ─── Data Types ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VaultConfig {
    /// The underlying token contract address.
    pub underlying: Address,
    /// Fixed ratio of shares minted per underlying token (must be >= 1).
    pub exchange_rate: i128,
    /// Maximum share supply cap (0 = uncapped; negative values rejected).
    pub max_share_supply: i128,
    /// Lock-up period in seconds applied to newly minted positions.
    pub lockup_secs: u64,
    /// Human-readable name of the share token.
    pub name: String,
    /// Symbol of the share token (e.g. "vREIT").
    pub symbol: String,
    /// Optional compliance contract for KYC screening.
    pub compliance: Option<Address>,
    /// Minimum verification level required (0..3).
    pub min_compliance_level: u32,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Configures the contract as part of the deploy transaction.
    ///
    /// Constructor requires `admin.require_auth()` (IR-17, ADR-002).
    /// Share decimals are read immutably from the underlying token.
    pub fn __constructor(env: Env, admin: Address, config: VaultConfig) {
        admin.require_auth();
        Self::validate_config(&env, &config);

        // Share decimals: read from the underlying token contract.
        // A non-token underlying that fails this call refuses deployment.
        let decimals = match UnderlyingTokenClient::new(&env, &config.underlying).try_decimals() {
            Ok(Ok(d)) => d,
            _ => panic_with_error!(&env, VaultError::InvalidUnderlying),
        };

        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PAUSED_KEY, &false);
        env.storage().instance().set(&CONFIG_KEY, &config);
        env.storage().instance().set(&DECIMALS_KEY, &decimals);

        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &0_i128);
        env.storage()
            .persistent()
            .set(&DataKey::UnderlyingHeld, &0_i128);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::TotalSupply);
        extend_persistent(&env, &DataKey::UnderlyingHeld);
    }

    // ── SEP-41 Share Token: Reads ─────────────────────────────────────────────

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        Self::read_allowance(&env, &DataKey::Allowance(owner, spender)).amount
    }

    pub fn decimals(env: Env) -> u32 {
        match env.storage().instance().get(&DECIMALS_KEY) {
            Some(d) => d,
            None => panic_with_error!(&env, VaultError::NotInitialized),
        }
    }

    pub fn name(env: Env) -> String {
        Self::config(env).name
    }

    pub fn symbol(env: Env) -> String {
        Self::config(env).symbol
    }

    /// Timestamp until which the holder's shares are locked from transfer and redemption.
    pub fn locked_until(env: Env, holder: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::LockedUntil(holder))
            .unwrap_or(0)
    }

    // ── SEP-41 Share Token: Allowances ────────────────────────────────────────

    /// Set `spender`'s allowance over `owner`'s balance, spendable through
    /// ledger `live_until_ledger` (SEP-41).
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        live_until_ledger: u32,
    ) {
        owner.require_auth();
        Self::require_not_paused(&env);
        if amount < 0 {
            panic_with_error!(&env, VaultError::InvalidAmount);
        }
        if amount > 0 && live_until_ledger < env.ledger().sequence() {
            panic_with_error!(&env, VaultError::InvalidExpiration);
        }

        let key = DataKey::Allowance(owner.clone(), spender.clone());
        env.storage().persistent().set(
            &key,
            &AllowanceValue {
                amount,
                live_until_ledger,
            },
        );

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::Approve {
            owner,
            spender,
            amount,
            live_until_ledger,
        }
        .publish(&env);
    }

    // ── SEP-41 Share Token: Transfers ─────────────────────────────────────────

    /// Transfer shares from caller to recipient.
    ///
    /// Execution order:
    /// 1. Auth: `from.require_auth()`
    /// 2. Not paused: `require_not_paused`
    /// 3. Positive amount: `require_positive`
    /// 4. Lock-up check: `now < locked_until(from)`
    /// 5. Compliance screen: `from` and `to` (if compliance configured)
    /// 6. Checked arithmetic and balance mutation
    pub fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        let to_muxed_id = to.id();
        let to_addr = to.address();

        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_unlocked(&env, &from);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to_addr);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, VaultError::InsufficientBalance);
        }

        // Self-transfer is a no-op; avoid double-write credit overwrite.
        if from == to_addr {
            extend_instance(&env);
            return;
        }

        let to_bal = Self::balance(env.clone(), to_addr.clone());
        let to_new = Self::checked_add(&env, to_bal, amount);

        Self::write_balance(&env, &from, from_bal - amount);
        Self::write_balance(&env, &to_addr, to_new);
        extend_instance(&env);

        events::Transfer {
            from,
            to: to_addr,
            amount,
            to_muxed_id,
        }
        .publish(&env);
    }

    /// Transfer shares from `from` to `to` using caller's (`spender`) allowance.
    ///
    /// The owner's lock is checked, not the spender's.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_unlocked(&env, &from);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);

        Self::spend_allowance(&env, &from, &spender, amount);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, VaultError::InsufficientBalance);
        }

        if from == to {
            extend_instance(&env);
            return;
        }

        let to_bal = Self::balance(env.clone(), to.clone());
        let to_new = Self::checked_add(&env, to_bal, amount);

        Self::write_balance(&env, &from, from_bal - amount);
        Self::write_balance(&env, &to, to_new);
        extend_instance(&env);

        events::TransferFrom { from, to, amount }.publish(&env);
    }

    // ── Value-Moving Core: Deposit & Redeem (#58) ──────────────────────────────

    /// Deposits `amount` of underlying tokens and mints `amount * exchange_rate` shares to `from`.
    ///
    /// Requires authorization from `from`, which forms an authorization tree with
    /// `vault.deposit` at the root and `underlying.transfer` as a sub-invocation.
    pub fn deposit(env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);

        let cfg = Self::config(env.clone());
        let shares = match amount.checked_mul(cfg.exchange_rate) {
            Some(s) => s,
            None => panic_with_error!(&env, VaultError::Overflow),
        };

        let current_total = Self::total_supply(env.clone());
        let new_total = Self::checked_add(&env, current_total, shares);
        if cfg.max_share_supply > 0 && new_total > cfg.max_share_supply {
            panic_with_error!(&env, VaultError::ExceedsMaxSupply);
        }

        // Pull underlying tokens to the vault contract address
        UnderlyingTokenClient::new(&env, &cfg.underlying).transfer(
            &from,
            &MuxedAddress::from(env.current_contract_address()),
            &amount,
        );

        let current_held = Self::underlying_held(env.clone());
        let new_held = Self::checked_add(&env, current_held, amount);
        Self::set_underlying_held(&env, new_held);

        Self::mint_shares(&env, &from, shares);

        let now = env.ledger().timestamp();
        let current_lock = Self::locked_until(env.clone(), from.clone());
        let new_lock_target = match now.checked_add(cfg.lockup_secs) {
            Some(t) => t,
            None => panic_with_error!(&env, VaultError::Overflow),
        };
        let final_lock = if cfg.lockup_secs == 0 {
            current_lock
        } else if current_lock > new_lock_target {
            current_lock
        } else {
            new_lock_target
        };
        Self::set_locked_until(&env, &from, final_lock);

        events::Deposit {
            from,
            amount,
            shares,
            locked_until: final_lock,
        }
        .publish(&env);

        shares
    }

    /// Redeems `shares` fractional shares for `shares / exchange_rate` underlying tokens.
    ///
    /// Holder must not be currently locked (`now >= locked_until(holder)`).
    /// `shares` must be an exact integer multiple of `exchange_rate`.
    pub fn redeem(env: Env, holder: Address, shares: i128) -> i128 {
        holder.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, shares);

        let cfg = Self::config(env.clone());
        if shares % cfg.exchange_rate != 0 {
            panic_with_error!(&env, VaultError::NotMultipleOfExchangeRate);
        }

        Self::require_unlocked(&env, &holder);

        let units = shares / cfg.exchange_rate;
        let current_held = Self::underlying_held(env.clone());
        if current_held < units {
            panic_with_error!(&env, VaultError::InsufficientBalance);
        }

        Self::burn_shares(&env, &holder, shares);
        Self::set_underlying_held(&env, current_held - units);

        // Vault transfers underlying to holder (authorizes as itself)
        UnderlyingTokenClient::new(&env, &cfg.underlying).transfer(
            &env.current_contract_address(),
            &MuxedAddress::from(holder.clone()),
            &units,
        );

        events::Redeem {
            holder,
            shares,
            amount: units,
        }
        .publish(&env);

        units
    }

    /// Returns the maximum number of shares currently redeemable by `holder`.
    pub fn redeemable(env: Env, holder: Address) -> i128 {
        if Self::paused(env.clone()) {
            return 0;
        }
        let now = env.ledger().timestamp();
        let locked = Self::locked_until(env.clone(), holder.clone());
        if now < locked {
            return 0;
        }
        let bal = Self::balance(env.clone(), holder);
        let cfg = Self::config(env.clone());
        if cfg.exchange_rate <= 0 {
            return 0;
        }
        bal - (bal % cfg.exchange_rate)
    }

    // ── Multi-Source NAV & Oracles (#59) ───────────────────────────────────────

    /// Configures the primary and optional secondary SEP-40 oracle sources (admin only).
    pub fn set_oracles(env: Env, config: Option<OracleSet>) {
        Self::require_admin(&env);

        let mut oracles_list = Vec::new(&env);
        if let Some(ref cfg) = config {
            if cfg.max_staleness_secs < 1 {
                panic_with_error!(&env, VaultError::InvalidMetadata);
            }
            oracles_list.push_back(cfg.primary.oracle.clone());

            if let Some(ref sec) = cfg.secondary {
                if cfg.max_divergence_bps < 1 {
                    panic_with_error!(&env, VaultError::InvalidMetadata);
                }
                if cfg.primary.oracle == sec.oracle {
                    panic_with_error!(&env, VaultError::IdenticalOracleSources);
                }

                let base1 = match Sep40OracleClient::new(&env, &cfg.primary.oracle).try_base() {
                    Ok(Ok(b)) => b,
                    _ => panic_with_error!(&env, VaultError::OracleBaseMismatch),
                };
                let base2 = match Sep40OracleClient::new(&env, &sec.oracle).try_base() {
                    Ok(Ok(b)) => b,
                    _ => panic_with_error!(&env, VaultError::OracleBaseMismatch),
                };
                if base1 != base2 {
                    panic_with_error!(&env, VaultError::OracleBaseMismatch);
                }
                oracles_list.push_back(sec.oracle.clone());
            }

            env.storage().instance().set(&ORACLES_KEY, cfg);
        } else {
            env.storage().instance().remove(&ORACLES_KEY);
        }

        extend_instance(&env);

        let admin = Self::admin(env.clone());
        events::SetOracles {
            admin,
            oracles: oracles_list,
        }
        .publish(&env);
    }

    /// Reads the configured oracle set, if set.
    pub fn oracles(env: Env) -> Option<OracleSet> {
        env.storage().instance().get(&ORACLES_KEY)
    }

    /// Pure query returning the total Net Asset Value of `underlying_held` in base asset terms.
    pub fn nav(env: Env) -> NavData {
        let (price, timestamp) = Self::get_effective_price_and_timestamp(&env);
        let held = Self::underlying_held(env.clone());
        let underlying_decimals = match env.storage().instance().get::<_, u32>(&DECIMALS_KEY) {
            Some(d) => d,
            None => panic_with_error!(&env, VaultError::NotInitialized),
        };

        let divisor = match 10_i128.checked_pow(underlying_decimals) {
            Some(d) => d,
            None => panic_with_error!(&env, VaultError::Overflow),
        };
        let product = match held.checked_mul(price) {
            Some(p) => p,
            None => panic_with_error!(&env, VaultError::Overflow),
        };
        let value = product / divisor;

        NavData {
            value,
            decimals: NAV_DECIMALS,
            timestamp,
        }
    }

    /// Pure query returning the Net Asset Value of one whole fractional share.
    pub fn nav_per_share(env: Env) -> NavData {
        let (price, timestamp) = Self::get_effective_price_and_timestamp(&env);
        let cfg = Self::config(env.clone());
        if cfg.exchange_rate <= 0 {
            panic_with_error!(&env, VaultError::InvalidExchangeRate);
        }
        let value = price / cfg.exchange_rate;

        NavData {
            value,
            decimals: NAV_DECIMALS,
            timestamp,
        }
    }

    // ── Administration & State Views ──────────────────────────────────────────

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, VaultError::NotInitialized),
        }
    }

    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }

    pub fn config(env: Env) -> VaultConfig {
        match env.storage().instance().get(&CONFIG_KEY) {
            Some(c) => c,
            None => panic_with_error!(&env, VaultError::NotInitialized),
        }
    }

    pub fn underlying_held(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::UnderlyingHeld)
            .unwrap_or(0)
    }

    /// Emergency pause taking effect in the same ledger (NFR-R-2).
    pub fn set_paused(env: Env, paused: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&PAUSED_KEY, &paused);
        extend_instance(&env);
        events::Paused { paused }.publish(&env);
    }

    /// Hand over admin privileges. Authorized by both existing and new admin (NFR-S-4).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        new_admin.require_auth();
        let previous = Self::admin(env.clone());
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin,
        }
        .publish(&env);
    }

    // ── Central Internal Balance Write Path ───────────────────────────────────

    /// Internal balance writer: the single central point where share balances change.
    ///
    /// Every balance mutation (transfers, mints, burns) routes through this function.
    /// Future issues (#60 checkpoints) hook this exact entry point to record
    /// ledger-bounded historical balances.
    pub(crate) fn write_balance(env: &Env, holder: &Address, new_balance: i128) {
        let key = DataKey::Balance(holder.clone());
        env.storage().persistent().set(&key, &new_balance);
        extend_persistent(env, &key);
    }

    /// Internal share mint helper for deposit operations (#58).
    pub(crate) fn mint_shares(env: &Env, to: &Address, amount: i128) {
        Self::require_positive(env, amount);
        let total = Self::total_supply(env.clone());
        let new_total = Self::checked_add(env, total, amount);

        let cfg = Self::config(env.clone());
        if cfg.max_share_supply > 0 && new_total > cfg.max_share_supply {
            panic_with_error!(env, VaultError::ExceedsMaxSupply);
        }

        let bal = Self::balance(env.clone(), to.clone());
        let new_bal = Self::checked_add(env, bal, amount);

        Self::write_balance(env, to, new_bal);

        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &new_total);
        extend_persistent(env, &DataKey::TotalSupply);
        extend_instance(env);
    }

    /// Internal share burn helper for redeem operations (#58).
    pub(crate) fn burn_shares(env: &Env, from: &Address, amount: i128) {
        Self::require_positive(env, amount);
        let bal = Self::balance(env.clone(), from.clone());
        if bal < amount {
            panic_with_error!(env, VaultError::InsufficientBalance);
        }
        let total = Self::total_supply(env.clone());
        let new_total = Self::checked_sub(env, total, amount);
        let new_bal = bal - amount;

        Self::write_balance(env, from, new_bal);

        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &new_total);
        extend_persistent(env, &DataKey::TotalSupply);
        extend_instance(env);
    }

    /// Internal helper to update underlying held balance for deposit/redeem (#58).
    pub(crate) fn set_underlying_held(env: &Env, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::UnderlyingHeld, &amount);
        extend_persistent(env, &DataKey::UnderlyingHeld);
        extend_instance(env);
    }

    /// Internal helper to set lock-up timestamp for a holder.
    pub(crate) fn set_locked_until(env: &Env, holder: &Address, until: u64) {
        let key = DataKey::LockedUntil(holder.clone());
        env.storage().persistent().set(&key, &until);
        extend_persistent(env, &key);
        extend_instance(env);
    }

    // ── Private Helpers ───────────────────────────────────────────────────────

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }

    fn require_not_paused(env: &Env) {
        if Self::paused(env.clone()) {
            panic_with_error!(env, VaultError::ContractPaused);
        }
    }

    fn require_positive(env: &Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(env, VaultError::InvalidAmount);
        }
    }

    fn require_unlocked(env: &Env, holder: &Address) {
        let now = env.ledger().timestamp();
        let locked_until = Self::locked_until(env.clone(), holder.clone());
        if now < locked_until {
            panic_with_error!(env, VaultError::SharesLocked);
        }
    }

    fn require_compliant(env: &Env, party: &Address) {
        let cfg = Self::config(env.clone());
        let Some(compliance) = cfg.compliance else {
            return;
        };
        let client = ComplianceClient::new(env, &compliance);
        if !client.screen(party, &cfg.min_compliance_level) {
            panic_with_error!(env, VaultError::NotCompliant);
        }
    }

    fn read_allowance(env: &Env, key: &DataKey) -> AllowanceValue {
        match env.storage().persistent().get::<_, AllowanceValue>(key) {
            Some(allowance) if allowance.live_until_ledger >= env.ledger().sequence() => allowance,
            _ => AllowanceValue {
                amount: 0,
                live_until_ledger: 0,
            },
        }
    }

    fn spend_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
        let key = DataKey::Allowance(from.clone(), spender.clone());
        let current = Self::read_allowance(env, &key);
        if current.amount < amount {
            panic_with_error!(env, VaultError::InsufficientAllowance);
        }
        env.storage().persistent().set(
            &key,
            &AllowanceValue {
                amount: current.amount - amount,
                live_until_ledger: current.live_until_ledger,
            },
        );
        extend_persistent(env, &key);
    }

    fn validate_config(env: &Env, config: &VaultConfig) {
        if config.exchange_rate < 1 {
            panic_with_error!(env, VaultError::InvalidExchangeRate);
        }
        if config.max_share_supply < 0 {
            panic_with_error!(env, VaultError::InvalidMaxSupply);
        }
        if config.lockup_secs > MAX_LOCKUP_SECS {
            panic_with_error!(env, VaultError::InvalidLockup);
        }
        if config.name.is_empty() || config.symbol.is_empty() {
            panic_with_error!(env, VaultError::InvalidMetadata);
        }
        if config.min_compliance_level > 3 {
            panic_with_error!(env, VaultError::InvalidComplianceLevel);
        }
    }

    fn fetch_source_price(env: &Env, source: &OracleSource, max_staleness: u64) -> (i128, u64) {
        let client = Sep40OracleClient::new(env, &source.oracle);
        let oracle_decimals = match client.try_decimals() {
            Ok(Ok(d)) => d,
            _ => panic_with_error!(env, VaultError::PriceUnavailable),
        };
        let price_data = match client.try_lastprice(&source.asset) {
            Ok(Ok(Some(p))) => p,
            _ => panic_with_error!(env, VaultError::PriceUnavailable),
        };

        if price_data.price <= 0 {
            panic_with_error!(env, VaultError::InvalidPrice);
        }
        let now = env.ledger().timestamp();
        if price_data.timestamp > now {
            panic_with_error!(env, VaultError::PriceInFuture);
        }
        if now.saturating_sub(price_data.timestamp) > max_staleness {
            panic_with_error!(env, VaultError::StalePrice);
        }

        let normalized = Self::normalize_decimals(env, price_data.price, oracle_decimals, NAV_DECIMALS);
        (normalized, price_data.timestamp)
    }

    fn normalize_decimals(env: &Env, price: i128, from_decimals: u32, to_decimals: u32) -> i128 {
        if from_decimals == to_decimals {
            price
        } else if from_decimals > to_decimals {
            let diff = from_decimals - to_decimals;
            let factor = match 10_i128.checked_pow(diff) {
                Some(f) => f,
                None => panic_with_error!(env, VaultError::Overflow),
            };
            price / factor
        } else {
            let diff = to_decimals - from_decimals;
            let factor = match 10_i128.checked_pow(diff) {
                Some(f) => f,
                None => panic_with_error!(env, VaultError::Overflow),
            };
            match price.checked_mul(factor) {
                Some(v) => v,
                None => panic_with_error!(env, VaultError::Overflow),
            }
        }
    }

    fn get_effective_price_and_timestamp(env: &Env) -> (i128, u64) {
        let oracle_set: OracleSet = match env.storage().instance().get(&ORACLES_KEY) {
            Some(s) => s,
            None => panic_with_error!(env, VaultError::OracleNotConfigured),
        };

        let (p1, t1) = Self::fetch_source_price(env, &oracle_set.primary, oracle_set.max_staleness_secs);

        if let Some(ref sec) = oracle_set.secondary {
            let (p2, t2) = Self::fetch_source_price(env, sec, oracle_set.max_staleness_secs);

            let diff = if p1 >= p2 { p1 - p2 } else { p2 - p1 };
            let min_p = if p1 <= p2 { p1 } else { p2 };

            let lhs = match diff.checked_mul(BPS_DENOMINATOR) {
                Some(v) => v,
                None => panic_with_error!(env, VaultError::Overflow),
            };
            let rhs = match min_p.checked_mul(oracle_set.max_divergence_bps as i128) {
                Some(v) => v,
                None => panic_with_error!(env, VaultError::Overflow),
            };

            if lhs > rhs {
                panic_with_error!(env, VaultError::OracleDivergence);
            }

            let chosen_price = min_p;
            let chosen_timestamp = if t1 <= t2 { t1 } else { t2 };
            (chosen_price, chosen_timestamp)
        } else {
            (p1, t1)
        }
    }

    fn checked_add(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => panic_with_error!(env, VaultError::Overflow),
        }
    }

    fn checked_sub(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => panic_with_error!(env, VaultError::Overflow),
        }
    }
}
