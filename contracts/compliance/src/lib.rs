#![no_std]

//! Compliance Engine — KYC/AML status registry.
//! Phase 1 skeleton: address allowlisting + jurisdiction tagging.

mod error;
mod events;

pub use error::ComplianceError;
pub use events::{AdminTransferred, JurisdictionDeniedSet, KycRevoked, KycSet};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

/// Highest verification level on the scale: 0 none, 1 basic, 2 full,
/// 3 accredited.
pub const MAX_LEVEL: u32 = 3;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    KycStatus(Address),
    /// Present and `true` when transfers to or from this ISO 3166-1 alpha-2
    /// jurisdiction are blocked regardless of verification level. Absent
    /// (not merely `false`) is the default, unblocked state.
    JurisdictionDenied(String),
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
        Self::validate_record(&env, &record);
        let key = DataKey::KycStatus(subject.clone());
        env.storage().persistent().set(&key, &record);

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::KycSet { subject, record }.publish(&env);
    }

    /// Revoke KYC for an address.
    pub fn revoke_kyc(env: Env, subject: Address) {
        Self::require_admin(&env);
        let key = DataKey::KycStatus(subject.clone());
        let removed: Option<KycRecord> = env.storage().persistent().get(&key);

        extend_instance(&env);

        if let Some(record) = removed {
            env.storage().persistent().remove(&key);
            events::KycRevoked { subject, record }.publish(&env);
        }
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

    /// Block or unblock an entire jurisdiction, regardless of any individual
    /// holder's verification level there.
    ///
    /// Issuers must be able to exclude a jurisdiction outright (sanctions, a
    /// market they are not licensed in) rather than only gate it by level,
    /// since a denied jurisdiction can hold verified, high-level KYC records
    /// and still need blocking (#141). This lives here, not in `rwa-asset`,
    /// for the reason ADR-004 gives for keeping jurisdictional rules out of
    /// the asset contract: they are the most change-prone rule in the
    /// protocol, and the asset contract has no upgrade path.
    pub fn set_jurisdiction_denied(env: Env, jurisdiction: String, denied: bool) {
        Self::require_admin(&env);
        Self::validate_jurisdiction_shape(&env, &jurisdiction);

        let key = DataKey::JurisdictionDenied(jurisdiction.clone());
        if denied {
            env.storage().persistent().set(&key, &true);
            extend_persistent(&env, &key);
        } else {
            env.storage().persistent().remove(&key);
        }
        extend_instance(&env);

        events::JurisdictionDeniedSet {
            jurisdiction,
            denied,
        }
        .publish(&env);
    }

    pub fn is_jurisdiction_denied(env: Env, jurisdiction: String) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::JurisdictionDenied(jurisdiction))
            .unwrap_or(false)
    }

    /// Hands the admin role to `new_admin`.
    ///
    /// Both the current and the incoming admin must authorize, in the same
    /// transaction (NFR-S-4), for the reason ADR-002 gives for `rwa-asset`: a
    /// handover to a key nobody controls cannot be undone.
    ///
    /// Without this the constructor's admin held the role for the contract's
    /// whole life (IR-04). For this contract that is the widest blast radius in
    /// the protocol: a compromised key can mark any address permanently
    /// compliant on every asset screening against it, and a lost one leaves
    /// records unrenewable until every dependent asset's transfers fail.
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
            None => panic_with_error!(&env, ComplianceError::NotInitialized),
        }
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }

    /// Rejects a record that could only be an input error (IR-08).
    ///
    /// Each of these used to be stored as given and then screen wrongly, with
    /// nothing to tell the admin that the record was not what they meant.
    fn validate_record(env: &Env, record: &KycRecord) {
        // A level above the scale satisfies every `min_level` an asset could
        // configure, including tiers that do not exist yet.
        if record.level > MAX_LEVEL {
            panic_with_error!(env, ComplianceError::InvalidLevel);
        }

        // An already-expired record screens as non-compliant from the moment
        // it is written. Zero is the explicit "never expires".
        if record.expires_at != 0 && record.expires_at <= env.ledger().timestamp() {
            panic_with_error!(env, ComplianceError::AlreadyExpired);
        }

        Self::validate_jurisdiction_shape(env, &record.jurisdiction);
    }

    /// ISO 3166-1 alpha-2: exactly two ASCII uppercase letters. This checks
    /// shape, not membership, so a well-formed but unassigned code passes.
    ///
    /// Shared by `set_kyc`'s record validation and `set_jurisdiction_denied`,
    /// so a deny-list entry can never be stored in a shape `evaluate` could
    /// never match against a real record's `jurisdiction` field.
    fn validate_jurisdiction_shape(env: &Env, jurisdiction: &String) {
        let mut code = [0u8; 2];
        if jurisdiction.len() != 2 {
            panic_with_error!(env, ComplianceError::InvalidJurisdiction);
        }
        jurisdiction.copy_into_slice(&mut code);
        if !code.iter().all(u8::is_ascii_uppercase) {
            panic_with_error!(env, ComplianceError::InvalidJurisdiction);
        }
    }

    fn evaluate(env: &Env, subject: Address, min_level: u32) -> bool {
        let record: Option<KycRecord> =
            env.storage().persistent().get(&DataKey::KycStatus(subject));

        match record {
            None => false,
            Some(r) => {
                let not_expired = r.expires_at == 0 || r.expires_at > env.ledger().timestamp();
                let jurisdiction_allowed =
                    !Self::is_jurisdiction_denied(env.clone(), r.jurisdiction.clone());
                r.level >= min_level && not_expired && jurisdiction_allowed
            }
        }
    }
}
