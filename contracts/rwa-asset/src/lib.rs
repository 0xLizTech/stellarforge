#![no_std]

mod compliance;
mod error;
mod events;
mod storage;

pub use compliance::{ComplianceClient, ComplianceInterface};
pub use error::RwaError;
pub use events::{Approve, Burn, Mint, Paused, Transfer};

use storage::{extend_instance, extend_persistent};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Bytes, Env,
    String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
/// Address of the compliance contract, absent when screening is disabled.
const COMPLIANCE_KEY: Symbol = symbol_short!("COMPLY");
/// Minimum verification level a party must hold to send or receive.
const MIN_LEVEL_KEY: Symbol = symbol_short!("MINLVL");

/// Upper bound on `AssetMetadata::decimals`. Beyond this a whole token no
/// longer fits sensibly in `i128` alongside realistic supply figures.
const MAX_DECIMALS: u32 = 18;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Metadata,
    Balance(Address),
    TotalSupply,
    Allowance(Address, Address),
    Issuer(Address),
}

// ─── Data Types ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AssetMetadata {
    /// Human-readable name of the real-world asset
    pub name: String,
    /// Ticker symbol (e.g. "REIT-NYC-001")
    pub symbol: String,
    /// Number of decimal places for fractional ownership
    pub decimals: u32,
    /// Asset class: "real_estate", "commodity", "equity", "debt", etc.
    pub asset_class: String,
    /// Off-chain legal document hash (IPFS CID or SHA-256)
    pub legal_doc_hash: Bytes,
    /// Total cap on issuance (0 = uncapped)
    pub max_supply: i128,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct RwaAssetContract;

#[contractimpl]
impl RwaAssetContract {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Deploy and configure this asset. Can only be called once.
    pub fn initialize(env: Env, admin: Address, metadata: AssetMetadata) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(&env, RwaError::AlreadyInitialized);
        }
        admin.require_auth();
        Self::validate_metadata(&env, &metadata);

        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PAUSED_KEY, &false);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &0_i128);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Metadata);
        extend_persistent(&env, &DataKey::TotalSupply);
    }

    // ── Issuer Management ─────────────────────────────────────────────────────

    /// Grant or revoke issuer role for an address.
    pub fn set_issuer(env: Env, issuer: Address, approved: bool) {
        Self::require_admin(&env);
        extend_instance(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Issuer(issuer.clone()), &approved);
        extend_persistent(&env, &DataKey::Issuer(issuer));
    }

    pub fn is_issuer(env: Env, issuer: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer))
            .unwrap_or(false)
    }

    // ── Minting & Burning ─────────────────────────────────────────────────────

    /// Mint new tokens to a recipient. Caller must be a registered issuer.
    pub fn mint(env: Env, issuer: Address, to: Address, amount: i128) {
        issuer.require_auth();
        Self::require_not_paused(&env);
        if !Self::is_issuer(env.clone(), issuer.clone()) {
            panic_with_error!(&env, RwaError::NotIssuer);
        }
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &to);

        let meta = Self::metadata(env.clone());
        let total = Self::total_supply(env.clone());
        let new_total = Self::checked_add(&env, total, amount);

        if meta.max_supply > 0 && new_total > meta.max_supply {
            panic_with_error!(&env, RwaError::ExceedsMaxSupply);
        }

        let bal = Self::balance(env.clone(), to.clone());
        let new_bal = Self::checked_add(&env, bal, amount);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_bal);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &new_total);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(to.clone()));
        extend_persistent(&env, &DataKey::TotalSupply);

        events::Mint { issuer, to, amount }.publish(&env);
    }

    /// Burn tokens from caller's balance.
    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);

        let bal = Self::balance(env.clone(), from.clone());
        if bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        let total = Self::total_supply(env.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(bal - amount));
        env.storage().persistent().set(
            &DataKey::TotalSupply,
            &Self::checked_sub(&env, total, amount),
        );

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::TotalSupply);

        events::Burn { from, amount }.publish(&env);
    }

    // ── Transfers ─────────────────────────────────────────────────────────────

    /// Transfer tokens from caller to recipient.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        // A self-transfer must be a no-op. Writing both legs would target the
        // same storage key, and the credit would overwrite the debit and mint
        // `amount` out of nothing.
        if from == to {
            return;
        }

        let to_bal = Self::balance(env.clone(), to.clone());
        let to_new = Self::checked_add(&env, to_bal, amount);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_new);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::Balance(to.clone()));

        events::Transfer { from, to, amount }.publish(&env);
    }

    // ── Allowances ────────────────────────────────────────────────────────────

    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        Self::require_not_paused(&env);
        if amount < 0 {
            panic_with_error!(&env, RwaError::InvalidAmount);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Allowance(owner.clone(), spender.clone()));

        events::Approve {
            owner,
            spender,
            amount,
        }
        .publish(&env);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);

        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        if allowance < amount {
            panic_with_error!(&env, RwaError::InsufficientAllowance);
        }

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        // The spender exercised their authorisation, so the allowance is
        // consumed either way.
        env.storage().persistent().set(
            &DataKey::Allowance(from.clone(), spender.clone()),
            &(allowance - amount),
        );

        // Skip the balance legs when from == to: both writes target the same
        // storage key, and the credit would overwrite the debit and mint
        // `amount` out of nothing.
        if from != to {
            let to_bal = Self::balance(env.clone(), to.clone());
            let to_new = Self::checked_add(&env, to_bal, amount);

            env.storage()
                .persistent()
                .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
            env.storage()
                .persistent()
                .set(&DataKey::Balance(to.clone()), &to_new);

            extend_persistent(&env, &DataKey::Balance(to.clone()));
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::Allowance(from.clone(), spender));

        events::Transfer { from, to, amount }.publish(&env);
    }

    // ── Read-only Views ───────────────────────────────────────────────────────

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(owner, spender))
            .unwrap_or(0)
    }

    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    pub fn metadata(env: Env) -> AssetMetadata {
        match env.storage().persistent().get(&DataKey::Metadata) {
            Some(m) => m,
            None => panic_with_error!(&env, RwaError::NotInitialized),
        }
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, RwaError::NotInitialized),
        }
    }

    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }

    // ── Admin Operations ──────────────────────────────────────────────────────

    pub fn set_paused(env: Env, paused: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&PAUSED_KEY, &paused);
        extend_instance(&env);
        events::Paused { paused }.publish(&env);
    }

    pub fn update_metadata(env: Env, metadata: AssetMetadata) {
        Self::require_admin(&env);
        Self::validate_metadata(&env, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        new_admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
    }

    // ── Compliance Configuration ──────────────────────────────────────────────

    /// Point the asset at a compliance contract, or pass `None` to disable
    /// screening entirely.
    ///
    /// `min_level` is the verification level both parties to a transfer must
    /// meet. It mirrors `ComplianceContract`'s scale: 0 none, 1 basic, 2 full,
    /// 3 accredited.
    pub fn set_compliance(env: Env, compliance: Option<Address>, min_level: u32) {
        Self::require_admin(&env);
        match compliance {
            Some(addr) => env.storage().instance().set(&COMPLIANCE_KEY, &addr),
            None => env.storage().instance().remove(&COMPLIANCE_KEY),
        }
        env.storage().instance().set(&MIN_LEVEL_KEY, &min_level);
        extend_instance(&env);
    }

    pub fn compliance_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&COMPLIANCE_KEY)
    }

    pub fn min_compliance_level(env: Env) -> u32 {
        env.storage().instance().get(&MIN_LEVEL_KEY).unwrap_or(0)
    }

    // ── Private Helpers ───────────────────────────────────────────────────────

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }

    fn require_not_paused(env: &Env) {
        if Self::paused(env.clone()) {
            panic_with_error!(env, RwaError::ContractPaused);
        }
    }

    /// Rejects `party` unless it satisfies the configured compliance policy.
    ///
    /// When no compliance contract is configured the check is skipped, so an
    /// asset can be issued and moved before an operator has stood up a KYC
    /// engine. Screening becomes mandatory the moment `set_compliance` names
    /// a contract.
    fn require_compliant(env: &Env, party: &Address) {
        let Some(compliance) = Self::compliance_contract(env.clone()) else {
            return;
        };
        let min_level = Self::min_compliance_level(env.clone());
        let client = ComplianceClient::new(env, &compliance);
        if !client.is_compliant(party, &min_level) {
            panic_with_error!(env, RwaError::NotCompliant);
        }
    }

    fn require_positive(env: &Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(env, RwaError::InvalidAmount);
        }
    }

    /// Rejects metadata that would make supply accounting unrepresentable.
    fn validate_metadata(env: &Env, metadata: &AssetMetadata) {
        if metadata.decimals > MAX_DECIMALS
            || metadata.max_supply < 0
            || metadata.symbol.is_empty()
            || metadata.name.is_empty()
        {
            panic_with_error!(env, RwaError::InvalidMetadata);
        }
    }

    fn checked_add(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        }
    }

    fn checked_sub(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        }
    }
}
