#![no_std]

//! Asset Registry — tracks all deployed RWA asset contracts.
//! Phase 1 skeleton: registration + lookup only.

mod error;
mod events;

pub use error::RegistryError;
pub use events::{ActiveSet, AdminTransferred, AssetRegistered};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

/// Largest page `list_assets` will return.
///
/// Each listed address is its own ledger read, and a read-only simulation is
/// held to the same per-invocation read limits as a transaction, so a page has
/// to stay well inside them.
pub const MAX_PAGE_SIZE: u32 = 100;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Asset(Address),
    /// Number of distinct assets ever registered, which is also the next free
    /// directory index.
    AssetCount,
    /// The asset registered `n`th, counting from zero. Indices are never
    /// reused or removed.
    AssetAt(u32),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AssetEntry {
    pub contract: Address,
    pub asset_class: soroban_sdk::String,
    pub active: bool,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Configures the contract as part of the deploy transaction.
    ///
    /// A constructor rather than a separate `initialize` entry point: the two
    /// are equivalent once the contract is running, but a separate call leaves
    /// a window in which the contract exists with no admin and anyone may name
    /// themselves. `require_auth` on `admin` still applies, so a deployer
    /// cannot hand the role to a key whose holder has not signed for it.
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().persistent().set(&DataKey::AssetCount, &0_u32);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::AssetCount);
    }

    /// Record an asset contract, or update an already-registered one.
    ///
    /// The directory is one entry per asset rather than a single list. A list
    /// held in one ledger entry hit the 64 KiB entry size limit at around
    /// 1,600 assets, after which every registration failed permanently
    /// (IR-01), and each registration rewrote the whole list on the way there.
    pub fn register(env: Env, entry: AssetEntry) {
        Self::require_admin(&env);
        let key = DataKey::Asset(entry.contract.clone());

        // The directory index is appended to only for a genuinely new asset.
        // Appending unconditionally listed the same address once per call, so
        // re-registering an asset to correct its class silently duplicated it
        // for every consumer iterating the directory.
        let is_new = !env.storage().persistent().has(&key);
        let contract = entry.contract.clone();
        env.storage().persistent().set(&key, &entry);

        if is_new {
            let index = Self::asset_count(env.clone());
            let next = match index.checked_add(1) {
                Some(v) => v,
                None => panic_with_error!(&env, RegistryError::Overflow),
            };
            let at = DataKey::AssetAt(index);
            env.storage().persistent().set(&at, &contract);
            env.storage().persistent().set(&DataKey::AssetCount, &next);
            extend_persistent(&env, &at);
            extend_persistent(&env, &DataKey::AssetCount);
        }

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::AssetRegistered {
            contract: entry.contract,
            asset_class: entry.asset_class,
            active: entry.active,
            is_new,
        }
        .publish(&env);
    }

    /// A pure query: registry lookups do not extend the entries they read, so
    /// they cost the caller nothing. Write paths extend to the network maximum
    /// instead.
    pub fn get_asset(env: Env, contract: Address) -> Option<AssetEntry> {
        env.storage().persistent().get(&DataKey::Asset(contract))
    }

    /// Registered assets in registration order, `limit` of them starting at
    /// index `start`.
    ///
    /// A page shorter than `limit` means the end of the directory was reached;
    /// a `start` at or past [`RegistryContract::asset_count`] returns an empty
    /// page. `limit` above [`MAX_PAGE_SIZE`] is rejected rather than quietly
    /// truncated, so a caller cannot mistake a clipped page for the whole
    /// directory.
    pub fn list_assets(env: Env, start: u32, limit: u32) -> Vec<Address> {
        if limit > MAX_PAGE_SIZE {
            panic_with_error!(&env, RegistryError::PageTooLarge);
        }

        let end = start
            .saturating_add(limit)
            .min(Self::asset_count(env.clone()));
        let mut page = Vec::new(&env);
        for index in start..end {
            // Every index below the count was written by `register` and
            // nothing removes one, so this is always present.
            if let Some(asset) = env.storage().persistent().get(&DataKey::AssetAt(index)) {
                page.push_back(asset);
            }
        }
        page
    }

    /// How many distinct assets have been registered. Valid `list_assets`
    /// indices run from 0 to one less than this.
    pub fn asset_count(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetCount)
            .unwrap_or(0)
    }

    pub fn set_active(env: Env, contract: Address, active: bool) {
        Self::require_admin(&env);
        let key = DataKey::Asset(contract.clone());
        let mut entry: AssetEntry = match env.storage().persistent().get(&key) {
            Some(e) => e,
            None => panic_with_error!(&env, RegistryError::AssetNotFound),
        };
        entry.active = active;
        env.storage().persistent().set(&key, &entry);

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::ActiveSet { contract, active }.publish(&env);
    }

    /// Hands the admin role to `new_admin`.
    ///
    /// Both the current and the incoming admin must authorize, in the same
    /// transaction (NFR-S-4), for the reason ADR-002 gives for `rwa-asset`: a
    /// handover to a key nobody controls cannot be undone.
    ///
    /// Without this the constructor's admin held the role for the contract's
    /// whole life, so a compromised key could never be rotated out of control
    /// of what the directory advertises (IR-04).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        let previous = Self::admin(env.clone());
        new_admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin,
        }
        .publish(&env);
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, RegistryError::NotInitialized),
        }
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }
}
