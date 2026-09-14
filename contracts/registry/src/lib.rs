#![no_std]

//! Asset Registry — tracks all deployed RWA asset contracts.
//! Phase 1 skeleton: registration + lookup only.

mod error;

pub use error::RegistryError;

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Asset(Address),
    AssetList,
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
        env.storage()
            .persistent()
            .set(&DataKey::AssetList, &Vec::<Address>::new(&env));

        extend_instance(&env);
        extend_persistent(&env, &DataKey::AssetList);
    }

    /// Record an asset contract, or update an already-registered one.
    pub fn register(env: Env, entry: AssetEntry) {
        Self::require_admin(&env);
        let key = DataKey::Asset(entry.contract.clone());

        // The directory index is appended to only for a genuinely new asset.
        // Pushing unconditionally left the same address in `list_assets` once
        // per call, so re-registering an asset to correct its class silently
        // duplicated it for every consumer iterating the directory.
        let is_new = !env.storage().persistent().has(&key);
        let contract = entry.contract.clone();
        env.storage().persistent().set(&key, &entry);

        if is_new {
            let mut list: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::AssetList)
                .unwrap_or(Vec::new(&env));
            list.push_back(contract);
            env.storage().persistent().set(&DataKey::AssetList, &list);
        }

        extend_instance(&env);
        extend_persistent(&env, &key);
        extend_persistent(&env, &DataKey::AssetList);
    }

    /// A pure query: registry lookups do not extend the entries they read, so
    /// they cost the caller nothing. Write paths extend to the network maximum
    /// instead.
    pub fn get_asset(env: Env, contract: Address) -> Option<AssetEntry> {
        env.storage().persistent().get(&DataKey::Asset(contract))
    }

    pub fn list_assets(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetList)
            .unwrap_or(Vec::new(&env))
    }

    pub fn set_active(env: Env, contract: Address, active: bool) {
        Self::require_admin(&env);
        let key = DataKey::Asset(contract);
        let mut entry: AssetEntry = match env.storage().persistent().get(&key) {
            Some(e) => e,
            None => panic_with_error!(&env, RegistryError::AssetNotFound),
        };
        entry.active = active;
        env.storage().persistent().set(&key, &entry);

        extend_instance(&env);
        extend_persistent(&env, &key);
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
