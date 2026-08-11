#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");

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
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PAUSED_KEY, &false);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &0_i128);
    }

    // ── Issuer Management ─────────────────────────────────────────────────────

    /// Grant or revoke issuer role for an address.
    pub fn set_issuer(env: Env, issuer: Address, approved: bool) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Issuer(issuer), &approved);
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
        assert!(
            Self::is_issuer(env.clone(), issuer.clone()),
            "caller is not an issuer"
        );
        assert!(amount > 0, "amount must be positive");

        let meta: AssetMetadata = env.storage().persistent().get(&DataKey::Metadata).unwrap();
        let total: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        if meta.max_supply > 0 {
            assert!(
                total.checked_add(amount).unwrap() <= meta.max_supply,
                "exceeds max supply"
            );
        }

        let bal = Self::balance(env.clone(), to.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(bal + amount));
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &(total + amount));
    }

    /// Burn tokens from caller's balance.
    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);
        assert!(amount > 0, "amount must be positive");

        let bal = Self::balance(env.clone(), from.clone());
        assert!(bal >= amount, "insufficient balance");

        let total: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &(total - amount));
    }

    // ── Transfers ─────────────────────────────────────────────────────────────

    /// Transfer tokens from caller to recipient.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);
        assert!(amount > 0, "amount must be positive");

        let from_bal = Self::balance(env.clone(), from.clone());
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal = Self::balance(env.clone(), to.clone());

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_bal + amount));
    }

    // ── Allowances ────────────────────────────────────────────────────────────

    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner, spender), &amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);
        assert!(amount > 0, "amount must be positive");

        let allowance: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allowance(from.clone(), spender.clone()))
            .unwrap_or(0);
        assert!(allowance >= amount, "allowance exceeded");

        let from_bal = Self::balance(env.clone(), from.clone());
        assert!(from_bal >= amount, "insufficient balance");

        let to_bal = Self::balance(env.clone(), to.clone());

        env.storage().persistent().set(
            &DataKey::Allowance(from.clone(), spender),
            &(allowance - amount),
        );
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_bal + amount));
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
        env.storage()
            .persistent()
            .get(&DataKey::Metadata)
            .expect("not initialized")
    }

    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized")
    }

    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }

    // ── Admin Operations ──────────────────────────────────────────────────────

    pub fn set_paused(env: Env, paused: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&PAUSED_KEY, &paused);
    }

    pub fn update_metadata(env: Env, metadata: AssetMetadata) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        new_admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
    }

    // ── Private Helpers ───────────────────────────────────────────────────────

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized");
        admin.require_auth();
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env.storage().instance().get(&PAUSED_KEY).unwrap_or(false);
        assert!(!paused, "contract is paused");
    }
}
