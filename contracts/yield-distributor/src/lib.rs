#![no_std]

pub mod error;

#[cfg(test)]
mod test;

use error::YieldError;
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Distribution {
    pub id: u32,
    pub token: Address,
    pub total_amount: i128,
    pub snapshot_ledger: u32,
    pub share_token: Address,
    pub total_shares_at_snapshot: i128,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Paused,
    DistributionCount,
    Distribution(u32),
    Claimed(u32, Address),
}

#[contract]
pub struct YieldDistributorContract;

#[contractimpl]
impl YieldDistributorContract {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::DistributionCount, &0u32);
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&DataKey::Admin) {
            Some(a) => a,
            None => panic_with_error!(&env, YieldError::NotInitialized),
        }
    }

    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    pub fn set_paused(env: Env, paused: bool) {
        let admin = Self::admin(env.clone());
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &paused);
    }

    pub fn distribution_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::DistributionCount)
            .unwrap_or(0)
    }

    pub fn get_distribution(env: Env, id: u32) -> Option<Distribution> {
        env.storage().persistent().get(&DataKey::Distribution(id))
    }

    pub fn is_claimed(env: Env, id: u32, account: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(id, account))
            .unwrap_or(false)
    }
}
