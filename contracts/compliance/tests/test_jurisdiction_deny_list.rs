#![cfg(test)]

//! Integration tests for jurisdiction allow/deny screening (#141): a holder
//! can hold a verified, sufficient-level record and still be blocked when
//! their jurisdiction is on the deny list.

use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    Address, Env, Event, IntoVal, String,
};

use compliance::{
    ComplianceContract, ComplianceContractClient, ComplianceError, JurisdictionDeniedSet, KycRecord,
};

const LEVEL_FULL: u32 = 2;
const NEVER_EXPIRES: u64 = 0;

struct Harness<'a> {
    env: Env,
    contract_id: Address,
    client: ComplianceContractClient<'a>,
}

impl Harness<'_> {
    fn record(&self, jurisdiction: &str, level: u32) -> KycRecord {
        KycRecord {
            jurisdiction: String::from_str(&self.env, jurisdiction),
            level,
            expires_at: NEVER_EXPIRES,
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
        contract_id,
        client,
    }
}

// ─── Configuration ──────────────────────────────────────────────────────────

#[test]
fn test_jurisdiction_is_not_denied_by_default() {
    let h = setup();
    assert!(!h
        .client
        .is_jurisdiction_denied(&String::from_str(&h.env, "KP")));
}

#[test]
fn test_set_jurisdiction_denied_is_readable_and_reversible() {
    let h = setup();
    let kp = String::from_str(&h.env, "KP");

    h.client.set_jurisdiction_denied(&kp, &true);
    assert!(h.client.is_jurisdiction_denied(&kp));

    h.client.set_jurisdiction_denied(&kp, &false);
    assert!(!h.client.is_jurisdiction_denied(&kp));
}

#[test]
fn test_set_jurisdiction_denied_requires_admin() {
    let h = setup();
    let not_admin = Address::generate(&h.env);
    let kp = String::from_str(&h.env, "KP");

    h.env.mock_auths(&[MockAuth {
        address: &not_admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_jurisdiction_denied",
            args: (kp.clone(), true).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h.client.try_set_jurisdiction_denied(&kp, &true).is_err());
    assert!(!h.client.is_jurisdiction_denied(&kp));
}

#[test]
fn test_set_jurisdiction_denied_rejects_a_malformed_code() {
    let h = setup();

    let too_short = String::from_str(&h.env, "K");
    let lowercase = String::from_str(&h.env, "kp");

    assert_eq!(
        h.client.try_set_jurisdiction_denied(&too_short, &true),
        Err(Ok(ComplianceError::InvalidJurisdiction.into())),
    );
    assert_eq!(
        h.client.try_set_jurisdiction_denied(&lowercase, &true),
        Err(Ok(ComplianceError::InvalidJurisdiction.into())),
    );
}

#[test]
fn test_set_jurisdiction_denied_emits_event() {
    let h = setup();
    let kp = String::from_str(&h.env, "KP");

    h.client.set_jurisdiction_denied(&kp, &true);

    assert_eq!(
        h.env.events().all(),
        std::vec![JurisdictionDeniedSet {
            jurisdiction: kp,
            denied: true,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Screening ──────────────────────────────────────────────────────────────

#[test]
fn test_a_verified_holder_in_a_denied_jurisdiction_fails_screening() {
    let h = setup();
    let subject = Address::generate(&h.env);

    h.client.set_kyc(&subject, &h.record("KP", LEVEL_FULL));
    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "KP"), &true);

    // A sufficient level alone no longer passes once the jurisdiction itself
    // is denied, which is exactly the gap #141 was filed to close.
    assert!(!h.client.is_compliant(&subject, &LEVEL_FULL));
    assert!(!h.client.screen(&subject, &LEVEL_FULL));
}

#[test]
fn test_denying_a_jurisdiction_after_the_fact_blocks_existing_holders_too() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client.set_kyc(&subject, &h.record("KP", LEVEL_FULL));

    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));

    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "KP"), &true);
    assert!(!h.client.is_compliant(&subject, &LEVEL_FULL));
}

#[test]
fn test_undenying_a_jurisdiction_restores_screening() {
    let h = setup();
    let subject = Address::generate(&h.env);
    let kp = String::from_str(&h.env, "KP");
    h.client.set_kyc(&subject, &h.record("KP", LEVEL_FULL));
    h.client.set_jurisdiction_denied(&kp, &true);
    assert!(!h.client.is_compliant(&subject, &LEVEL_FULL));

    h.client.set_jurisdiction_denied(&kp, &false);
    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));
}

#[test]
fn test_a_holder_in_an_undenied_jurisdiction_is_unaffected() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client.set_kyc(&subject, &h.record("US", LEVEL_FULL));
    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "KP"), &true);

    assert!(h.client.is_compliant(&subject, &LEVEL_FULL));
}

#[test]
fn test_denying_several_jurisdictions_is_independent() {
    let h = setup();
    let in_kp = Address::generate(&h.env);
    let in_ir = Address::generate(&h.env);
    let in_us = Address::generate(&h.env);

    h.client.set_kyc(&in_kp, &h.record("KP", LEVEL_FULL));
    h.client.set_kyc(&in_ir, &h.record("IR", LEVEL_FULL));
    h.client.set_kyc(&in_us, &h.record("US", LEVEL_FULL));

    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "KP"), &true);
    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "IR"), &true);

    assert!(!h.client.is_compliant(&in_kp, &LEVEL_FULL));
    assert!(!h.client.is_compliant(&in_ir, &LEVEL_FULL));
    assert!(h.client.is_compliant(&in_us, &LEVEL_FULL));
}

#[test]
fn test_screen_and_is_compliant_still_agree_with_a_deny_list_configured() {
    let h = setup();
    let subject = Address::generate(&h.env);
    h.client.set_kyc(&subject, &h.record("KP", LEVEL_FULL));
    h.client
        .set_jurisdiction_denied(&String::from_str(&h.env, "KP"), &true);

    assert_eq!(
        h.client.is_compliant(&subject, &LEVEL_FULL),
        h.client.screen(&subject, &LEVEL_FULL),
    );
}
