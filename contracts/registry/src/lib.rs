#![no_std]

//! Asset Registry — tracks all deployed RWA asset contracts.
//! Phase 1 skeleton: registration + lookup only.

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

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
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::AssetList, &Vec::<Address>::new(&env));
    }

    pub fn register(env: Env, entry: AssetEntry) {
        Self::require_admin(&env);
        let key = DataKey::Asset(entry.contract.clone());
        env.storage().persistent().set(&key, &entry.clone());

        let mut list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetList)
            .unwrap_or(Vec::new(&env));
        list.push_back(entry.contract);
        env.storage()
            .persistent()
            .set(&DataKey::AssetList, &list);
    }

    pub fn get_asset(env: Env, contract: Address) -> Option<AssetEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::Asset(contract))
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
        let mut entry: AssetEntry = env
            .storage()
            .persistent()
            .get(&key)
            .expect("asset not found");
        entry.active = active;
        env.storage().persistent().set(&key, &entry);
    }

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized");
        admin.require_auth();
    }
}
