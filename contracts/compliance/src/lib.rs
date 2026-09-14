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
    ///
    /// A pure query: it does not touch the ledger. Contracts screening a
    /// transfer should call [`ComplianceContract::screen`] instead.
    pub fn is_compliant(env: Env, subject: Address, min_level: u32) -> bool {
        Self::evaluate(&env, subject, min_level)
    }

    /// Screens `subject` and refreshes the lifetime of the record consulted.
    ///
    /// Identical to [`ComplianceContract::is_compliant`] except that it
    /// extends the record's TTL. A KYC record is written once and thereafter
    /// only read, so it has no other opportunity to be refreshed; screening is
    /// the moment the holder is demonstrably active. This is the entry point
    /// `rwa-asset` binds to, where the caller is already paying for a ledger
    /// write, which is why the extension does not belong on the pure query.
    pub fn screen(env: Env, subject: Address, min_level: u32) -> bool {
        extend_persistent(&env, &DataKey::KycStatus(subject.clone()));
        Self::evaluate(&env, subject, min_level)
    }

    pub fn get_kyc(env: Env, subject: Address) -> Option<KycRecord> {
        env.storage().persistent().get(&DataKey::KycStatus(subject))
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

    fn evaluate(env: &Env, subject: Address, min_level: u32) -> bool {
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
}
