#![no_std]

//! Compliance Engine — KYC/AML status registry.
//! Phase 1 skeleton: address allowlisting + jurisdiction tagging.

mod error;
mod events;

pub use error::ComplianceError;
pub use events::{
    AdminTransferCancelled, AdminTransferProposed, AdminTransferred, KycRevoked, KycSet,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol, Vec,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PENDING_ADMIN_KEY: Symbol = symbol_short!("PADMIN");

/// Highest verification level on the scale: 0 none, 1 basic, 2 full,
/// 3 accredited.
pub const MAX_LEVEL: u32 = 3;

/// Largest batch `set_kyc_batch`/`revoke_kyc_batch` will accept in one call.
///
/// Each record is its own ledger write, and a transaction is held to the
/// network's per-invocation resource limits, so a batch has to stay well
/// inside them.
pub const MAX_BATCH_SIZE: u32 = 50;

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
        Self::validate_record(&env, &record);
        let key = DataKey::KycStatus(subject.clone());
        env.storage().persistent().set(&key, &record);

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::KycSet { subject, record }.publish(&env);
    }

    /// Set or update KYC records for up to [`MAX_BATCH_SIZE`] addresses in one
    /// call.
    ///
    /// Onboarding many holders one `set_kyc` per transaction does not scale
    /// past a handful of addresses; this amortises the admin's signature and
    /// the transaction's base fee across the whole batch.
    ///
    /// Every record is validated before any of them are written: a batch
    /// either lands in full or not at all, so operators never have to
    /// reconcile which prefix of a partially-applied batch actually took
    /// effect. One [`events::KycSet`] is published per record, exactly as if
    /// [`ComplianceContract::set_kyc`] had been called once per entry, so a
    /// listener watching for individual decisions sees the same events either
    /// way.
    pub fn set_kyc_batch(env: Env, records: Vec<(Address, KycRecord)>) {
        Self::require_admin(&env);

        if records.len() > MAX_BATCH_SIZE {
            panic_with_error!(&env, ComplianceError::BatchTooLarge);
        }
        for (_, record) in records.iter() {
            Self::validate_record(&env, &record);
        }

        extend_instance(&env);

        for (subject, record) in records.iter() {
            let key = DataKey::KycStatus(subject.clone());
            env.storage().persistent().set(&key, &record);
            extend_persistent(&env, &key);

            events::KycSet { subject, record }.publish(&env);
        }
    }

    /// Revoke KYC for up to [`MAX_BATCH_SIZE`] addresses in one call.
    ///
    /// A subject with no existing record is silently skipped, matching
    /// [`ComplianceContract::revoke_kyc`]'s own behavior: revoking an unknown
    /// subject changes nothing, so it publishes nothing.
    pub fn revoke_kyc_batch(env: Env, subjects: Vec<Address>) {
        Self::require_admin(&env);

        if subjects.len() > MAX_BATCH_SIZE {
            panic_with_error!(&env, ComplianceError::BatchTooLarge);
        }

        extend_instance(&env);

        for subject in subjects.iter() {
            let key = DataKey::KycStatus(subject.clone());
            let removed: Option<KycRecord> = env.storage().persistent().get(&key);
            if let Some(record) = removed {
                env.storage().persistent().remove(&key);
                events::KycRevoked { subject, record }.publish(&env);
            }
        }
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

    /// Proposes `new_admin` as the next admin. Only the current admin may
    /// call this; `new_admin` has not authorized anything yet.
    ///
    /// Two-step handover (#125), replacing the single-transaction dual-auth
    /// `transfer_admin` ADR-002 originally specified. ADR-002 rejected this
    /// shape for Phase 1 only because the SDK could not build a transaction
    /// carrying two distinct signers (#31); #31 is closed, and ADR-006
    /// revisits the tradeoff now that it no longer applies. The underlying
    /// guarantee is unchanged: a handover to a key nobody controls still
    /// cannot complete, because `accept_admin` still requires that key's own
    /// signature. For this contract that guarantee matters most: a
    /// compromised key can mark any address permanently compliant on every
    /// asset screening against it, and a lost one leaves records unrenewable
    /// until every dependent asset's transfers fail (IR-04).
    ///
    /// Calling this again before `accept_admin` replaces the pending
    /// proposal rather than requiring `cancel_admin_transfer` first.
    pub fn propose_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        let current = Self::admin(env.clone());
        env.storage().instance().set(&PENDING_ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferProposed {
            current,
            proposed_admin: new_admin,
        }
        .publish(&env);
    }

    /// Completes a handover `propose_admin` started. Only the proposed admin
    /// may call this, and only its own signature is required: the current
    /// admin already authorized the proposal, which is where the "both
    /// sides consented" guarantee comes from now.
    pub fn accept_admin(env: Env) {
        let pending: Address = match env.storage().instance().get(&PENDING_ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, ComplianceError::NoPendingAdminTransfer),
        };
        pending.require_auth();

        let previous = Self::admin(env.clone());
        env.storage().instance().set(&ADMIN_KEY, &pending);
        env.storage().instance().remove(&PENDING_ADMIN_KEY);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin: pending,
        }
        .publish(&env);
    }

    /// Withdraws a pending proposal before it is accepted. Only the current
    /// admin may call this, which is what makes a mistaken or no-longer-wanted
    /// proposal reversible without waiting for the proposed admin to act (or
    /// refuse to).
    pub fn cancel_admin_transfer(env: Env) {
        Self::require_admin(&env);
        let pending: Address = match env.storage().instance().get(&PENDING_ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, ComplianceError::NoPendingAdminTransfer),
        };
        env.storage().instance().remove(&PENDING_ADMIN_KEY);
        extend_instance(&env);

        events::AdminTransferCancelled {
            cancelled_proposed_admin: pending,
        }
        .publish(&env);
    }

    /// The address `accept_admin` would install if called now, or `None` if
    /// no transfer is pending.
    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&PENDING_ADMIN_KEY)
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

        // ISO 3166-1 alpha-2: exactly two ASCII uppercase letters. This checks
        // shape, not membership, so a well-formed but unassigned code passes.
        let jurisdiction = &record.jurisdiction;
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
                r.level >= min_level && not_expired
            }
        }
    }
}
