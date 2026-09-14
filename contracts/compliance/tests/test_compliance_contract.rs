#![cfg(test)]

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger,
    },
    Address, Env, String,
};

use compliance::{ComplianceContract, ComplianceContractClient, DataKey, KycRecord};
use stellarforge_common::storage::INSTANCE_BUMP_AMOUNT;

const LEVEL_BASIC: u32 = 1;
const LEVEL_FULL: u32 = 2;
const LEVEL_ACCREDITED: u32 = 3;
const NEVER_EXPIRES: u64 = 0;

struct Harness<'a> {
    env: Env,
    admin: Address,
    contract_id: Address,
    client: ComplianceContractClient<'a>,
}

impl Harness<'_> {
    fn ttl_of(&self, subject: &Address) -> u32 {
        let key = DataKey::KycStatus(subject.clone());
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().persistent().get_ttl(&key)
        })
    }

    fn instance_ttl(&self) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().instance().get_ttl()
        })
    }

    /// The network's maximum entry TTL, which is what write paths extend to.
    fn max_ttl(&self) -> u32 {
        self.env
            .as_contract(&self.contract_id, || self.env.storage().max_ttl())
    }

    fn record(&self, level: u32, expires_at: u64) -> KycRecord {
        KycRecord {
            jurisdiction: String::from_str(&self.env, "US"),
            level,
            expires_at,
        }
    }
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ComplianceContract, (&admin,));
    let client = ComplianceContractClient::new(&env, &contract_id);

    Harness {
        env,
        admin,
        contract_id,
        client,
    }
}

// ─── Initialisation ────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let h = setup();
    assert_eq!(h.client.admin(), h.admin);
}

// `test_double_initialize_fails` and `test_admin_before_initialize_is_rejected`
// were removed with the `initialize` entry point. Neither state is reachable
// through a constructor: configuration happens once, inside the deploy
// transaction, and there is no moment at which a registered contract has no
// admin. The guarantee is now structural rather than something a test asserts.

// The constructor's `admin.require_auth()` is not covered here, and cannot be:
// `Env::register` invokes a constructor with authorization mocked, so it
// succeeds whatever the environment is configured to allow. Covering it needs a
// real deploy via `env.deployer()` against uploaded wasm, which no test in this
// repo does yet. Auth on the former `initialize` was equally uncovered — every
// test that called it ran under `mock_all_auths` — so this is a pre-existing
// gap that moved, not one this change introduced.

// ─── Records ───────────────────────────────────────────────────────────────

#[test]
fn test_set_and_get_kyc() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    let stored = h.client.get_kyc(&subject).unwrap();
    assert_eq!(stored.level, LEVEL_FULL);
    assert_eq!(stored.jurisdiction, String::from_str(&h.env, "US"));
    assert_eq!(stored.expires_at, NEVER_EXPIRES);
}

#[test]
fn test_get_kyc_for_unknown_subject_returns_none() {
    let h = setup();
    assert!(h.client.get_kyc(&Address::generate(&h.env)).is_none());
}

#[test]
fn test_set_kyc_overwrites_the_previous_record() {
    let h = setup();
    let subject = Address::generate(&h.env);

    h.client
        .set_kyc(&subject, &h.record(LEVEL_BASIC, NEVER_EXPIRES));
    h.client
        .set_kyc(&subject, &h.record(LEVEL_ACCREDITED, NEVER_EXPIRES));

    assert_eq!(h.client.get_kyc(&subject).unwrap().level, LEVEL_ACCREDITED);
}

#[test]
fn test_revoke_kyc_removes_the_record() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    h.client.revoke_kyc(&subject);

    assert!(h.client.get_kyc(&subject).is_none());
    assert!(!h.client.is_compliant(&subject, &LEVEL_BASIC));
}

#[test]
fn test_revoking_an_unknown_subject_is_a_no_op() {
    let h = setup();
    h.client.revoke_kyc(&Address::generate(&h.env));
}

// ─── Screening ─────────────────────────────────────────────────────────────

#[test]
fn test_unverified_subject_is_not_compliant() {
    let h = setup();
    let subject = Address::generate(&h.env);
    assert!(!h.client.is_compliant(&subject, &LEVEL_BASIC));
    // Even a zero minimum requires a record to exist.
    assert!(!h.client.is_compliant(&subject, &0));
}

#[test]
fn test_level_is_a_minimum_not_an_equality() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    assert!(h.client.is_compliant(&subject, &0));
    assert!(h.client.is_compliant(&subject, &LEVEL_BASIC));
    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));
    assert!(!h.client.is_compliant(&subject, &LEVEL_ACCREDITED));
}

#[test]
fn test_expiry_is_evaluated_against_ledger_time() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.env.ledger().with_mut(|li| li.timestamp = 1_000);
    h.client.set_kyc(&subject, &h.record(LEVEL_FULL, 2_000));

    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));

    // Exactly at the expiry timestamp the record is already stale: the
    // contract requires expires_at to be strictly greater than now.
    h.env.ledger().with_mut(|li| li.timestamp = 2_000);
    assert!(!h.client.is_compliant(&subject, &LEVEL_FULL));

    h.env.ledger().with_mut(|li| li.timestamp = 2_001);
    assert!(!h.client.is_compliant(&subject, &LEVEL_FULL));
}

#[test]
fn test_zero_expiry_never_lapses() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    h.env.ledger().with_mut(|li| li.timestamp = u64::MAX);
    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));
}

// ─── Storage lifetime ──────────────────────────────────────────────────────

// Since protocol 23 the test environment auto-restores archived entries, so
// advancing the ledger past an expiry proves nothing. These tests assert the
// TTL itself, which is what the extension is for.

/// Longer than PERSISTENT_REFRESH_INTERVAL, so a write path actually reissues
/// the extension rather than finding the entry still fresh enough.
const IDLE: u32 = 600_000;

#[test]
fn test_set_kyc_extends_to_the_network_maximum() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    assert_eq!(h.ttl_of(&subject), h.max_ttl());
    assert_eq!(h.instance_ttl(), INSTANCE_BUMP_AMOUNT);
}

#[test]
fn test_screen_extends_the_record_it_consults() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    // A record is set once and thereafter only read. Screening is the one
    // moment the holder is demonstrably active, and it happens inside a
    // transfer that is already paying for a ledger write.
    let expected_before = h.max_ttl() - IDLE;
    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);
    assert_eq!(h.ttl_of(&subject), expected_before);

    assert!(h.client.screen(&subject, &LEVEL_FULL));
    assert_eq!(h.ttl_of(&subject), h.max_ttl());
}

#[test]
fn test_is_compliant_is_a_pure_query() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client
        .set_kyc(&subject, &h.record(LEVEL_FULL, NEVER_EXPIRES));

    let expected = h.max_ttl() - IDLE;
    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);

    // The query path must not write to the ledger — that is the whole reason
    // `screen` exists as a separate entry point.
    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));
    assert_eq!(h.ttl_of(&subject), expected);

    h.client.get_kyc(&subject);
    assert_eq!(h.ttl_of(&subject), expected);
}

#[test]
fn test_screen_and_is_compliant_agree() {
    let h = setup();
    let verified = Address::generate(&h.env);
    let unverified = Address::generate(&h.env);
    h.client
        .set_kyc(&verified, &h.record(LEVEL_BASIC, NEVER_EXPIRES));

    // The two entry points must differ only in their ledger side effect.
    for level in [0, LEVEL_BASIC, LEVEL_FULL, LEVEL_ACCREDITED] {
        assert_eq!(
            h.client.screen(&verified, &level),
            h.client.is_compliant(&verified, &level)
        );
        assert_eq!(
            h.client.screen(&unverified, &level),
            h.client.is_compliant(&unverified, &level)
        );
    }
}

#[test]
fn test_screening_an_unverified_subject_does_not_create_an_entry() {
    let h = setup();
    let subject = Address::generate(&h.env);

    // extend_persistent must stay a no-op for a key that was never written,
    // rather than materialising an empty record.
    assert!(!h.client.screen(&subject, &LEVEL_BASIC));
    assert!(h.client.get_kyc(&subject).is_none());
}
