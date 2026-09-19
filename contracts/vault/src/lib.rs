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
#[cfg(test)]
mod test;
mod token;

pub use compliance::{ComplianceClient, ComplianceInterface};
pub use error::VaultError;
pub use events::{AdminTransferred, Approve, Paused, Transfer, TransferFrom};
pub use token::{UnderlyingTokenClient, UnderlyingTokenInterface};

use stellarforge_common::{extend_instance, extend_persistent};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env,
    MuxedAddress, String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
const CONFIG_KEY: Symbol = symbol_short!("CONFIG");
const DECIMALS_KEY: Symbol = symbol_short!("DECIMALS");

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
    #[allow(dead_code)] // Called by deposit (#58) and balance checkpoints (#60)
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
    #[allow(dead_code)] // Called by redeem (#58) and balance checkpoints (#60)
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
    #[allow(dead_code)] // Called by deposit and redeem (#58)
    pub(crate) fn set_underlying_held(env: &Env, amount: i128) {
        env.storage()
            .persistent()
            .set(&DataKey::UnderlyingHeld, &amount);
        extend_persistent(env, &DataKey::UnderlyingHeld);
        extend_instance(env);
    }

    /// Internal helper to set lock-up timestamp for a holder.
    #[allow(dead_code)] // Called by deposit lock-up enforcement (#58)
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
    }

    fn checked_add(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => panic_with_error!(env, VaultError::Overflow),
        }
    }

    #[allow(dead_code)] // Called by burn_shares for redeem (#58)
    fn checked_sub(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => panic_with_error!(env, VaultError::Overflow),
        }
    }
}
