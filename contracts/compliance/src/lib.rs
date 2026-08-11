#![no_std]

//! Compliance Engine — KYC/AML status registry.
//! Phase 1 skeleton: address allowlisting + jurisdiction tagging.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

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
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
    }

    /// Set or update KYC record for an address.
    pub fn set_kyc(env: Env, subject: Address, record: KycRecord) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::KycStatus(subject), &record);
    }

    /// Revoke KYC for an address.
    pub fn revoke_kyc(env: Env, subject: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .remove(&DataKey::KycStatus(subject));
    }

    /// Returns true if the address has at minimum the required verification level
    /// and the record is not expired.
    pub fn is_compliant(env: Env, subject: Address, min_level: u32) -> bool {
        let record: Option<KycRecord> =
            env.storage().persistent().get(&DataKey::KycStatus(subject));

        match record {
            None => false,
            Some(r) => {
                let not_expired = r.expires_at == 0 || r.expires_at > env.ledger().timestamp();
                r.level >= min_level && not_expired
            }
        }
    }

    pub fn get_kyc(env: Env, subject: Address) -> Option<KycRecord> {
        env.storage().persistent().get(&DataKey::KycStatus(subject))
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
