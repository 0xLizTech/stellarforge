#![no_std]

//! Compliance Engine — KYC/AML status registry.
//! Phase 1 skeleton: address allowlisting + jurisdiction tagging.

mod error;

pub use error::ComplianceError;

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    KycStatus(Address),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct KycRecord {
    /// ISO-3166-1 alpha-2 country code
    pub jurisdiction: String,
    /// Verification level: 0=none, 1=basic, 2=full, 3=accredited
    pub level: u32,
    /// Unix timestamp of expiry (0 = never)
    pub expires_at: u64,
}

#[contract]
pub struct ComplianceContract;

#[contractimpl]
impl ComplianceContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(&env, ComplianceError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);

        extend_instance(&env);
    }

    /// Set or update KYC record for an address.
    pub fn set_kyc(env: Env, subject: Address, record: KycRecord) {
        Self::require_admin(&env);
        let key = DataKey::KycStatus(subject);
        env.storage().persistent().set(&key, &record);

        extend_instance(&env);
        extend_persistent(&env, &key);
    }

    /// Revoke KYC for an address.
    pub fn revoke_kyc(env: Env, subject: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .remove(&DataKey::KycStatus(subject));

        extend_instance(&env);
    }

    /// Returns true if the address has at minimum the required verification level
    /// and the record is not expired.
    pub fn is_compliant(env: Env, subject: Address, min_level: u32) -> bool {
        let key = DataKey::KycStatus(subject);
        // A KYC record is written once and then only read — every subsequent
        // transfer screens against it without ever writing it back. Extending
        // here is what keeps a verified holder's record from being archived
        // out from under them, which would block their transfers until
        // someone restored it.
        extend_persistent(&env, &key);

        let record: Option<KycRecord> = env.storage().persistent().get(&key);

        match record {
            None => false,
            Some(r) => {
                let not_expired = r.expires_at == 0 || r.expires_at > env.ledger().timestamp();
                r.level >= min_level && not_expired
            }
        }
    }

    pub fn get_kyc(env: Env, subject: Address) -> Option<KycRecord> {
        let key = DataKey::KycStatus(subject);
        extend_persistent(&env, &key);
        env.storage().persistent().get(&key)
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, ComplianceError::NotInitialized),
        }
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }
}
