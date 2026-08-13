#![cfg(test)]

//! Integration tests exercising the asset against a real ComplianceContract
//! rather than a stub, so the cross-contract call, the KYC level comparison
//! and the expiry clock are all covered end to end.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, String,
};

use compliance::{ComplianceContract, ComplianceContractClient, KycRecord};
use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient, RwaError};

/// Verification levels as defined by ComplianceContract.
const LEVEL_BASIC: u32 = 1;
const LEVEL_FULL: u32 = 2;

const NEVER_EXPIRES: u64 = 0;

struct Harness<'a> {
    env: Env,
    asset: RwaAssetContractClient<'a>,
    compliance: ComplianceContractClient<'a>,
    issuer: Address,
}

impl Harness<'_> {
    /// Deploys both contracts and configures the asset to screen at
    /// `min_level`. The issuer is registered but deliberately left unverified;
    /// individual tests grant whatever status they need.
    fn new(min_level: u32) -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let compliance_id = env.register(ComplianceContract, ());
        let compliance = ComplianceContractClient::new(&env, &compliance_id);
        compliance.initialize(&admin);

        let asset_id = env.register(RwaAssetContract, ());
        let asset = RwaAssetContractClient::new(&env, &asset_id);
        asset.initialize(&admin, &metadata(&env));

        asset.set_compliance(&Some(compliance_id), &min_level);

        let issuer = Address::generate(&env);
        asset.set_issuer(&issuer, &true);

        Self {
            env,
            asset,
            compliance,
            issuer,
        }
    }

    fn verify(&self, subject: &Address, level: u32, expires_at: u64) {
        self.compliance.set_kyc(
            subject,
            &KycRecord {
                jurisdiction: String::from_str(&self.env, "US"),
                level,
                expires_at,
            },
        );
    }
}

fn metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "NYC Real Estate Fund I"),
        symbol: String::from_str(env, "REIT-NYC-001"),
        decimals: 7,
        asset_class: String::from_str(env, "real_estate"),
        legal_doc_hash: Bytes::from_array(env, &[0u8; 32]),
        max_supply: 0,
    }
}

// ─── Configuration ─────────────────────────────────────────────────────────

#[test]
fn test_compliance_is_configurable_and_readable() {
    let h = Harness::new(LEVEL_BASIC);
    assert!(h.asset.compliance_contract().is_some());
    assert_eq!(h.asset.min_compliance_level(), LEVEL_BASIC);

    h.asset.set_compliance(&None, &0);
    assert!(h.asset.compliance_contract().is_none());
}

#[test]
fn test_transfers_are_unrestricted_when_compliance_is_unset() {
    let h = Harness::new(LEVEL_BASIC);
    h.asset.set_compliance(&None, &0);

    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    // Neither party holds a KYC record.
    h.asset.mint(&h.issuer, &alice, &1_000);
    h.asset.transfer(&alice, &bob, &400);

    assert_eq!(h.asset.balance(&bob), 400);
}

// ─── Screening ─────────────────────────────────────────────────────────────

#[test]
fn test_mint_to_unverified_recipient_is_rejected() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);

    let res = h.asset.try_mint(&h.issuer, &alice, &1_000);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));
    assert_eq!(h.asset.balance(&alice), 0);
}

#[test]
fn test_transfer_to_unverified_recipient_is_rejected() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    h.verify(&alice, LEVEL_BASIC, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);

    let res = h.asset.try_transfer(&alice, &bob, &400);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));
    assert_eq!(h.asset.balance(&alice), 1_000);
}

#[test]
fn test_transfer_from_unverified_sender_is_rejected() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    h.verify(&alice, LEVEL_BASIC, NEVER_EXPIRES);
    h.verify(&bob, LEVEL_BASIC, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);

    // Alice's verification is withdrawn after she acquired the position.
    h.compliance.revoke_kyc(&alice);

    let res = h.asset.try_transfer(&alice, &bob, &400);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));
}

#[test]
fn test_insufficient_level_is_rejected_until_upgraded() {
    let h = Harness::new(LEVEL_FULL);
    let alice = Address::generate(&h.env);

    // Basic verification sits below the asset's required level.
    h.verify(&alice, LEVEL_BASIC, NEVER_EXPIRES);
    let res = h.asset.try_mint(&h.issuer, &alice, &1_000);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));

    h.verify(&alice, LEVEL_FULL, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);
    assert_eq!(h.asset.balance(&alice), 1_000);
}

#[test]
fn test_expired_verification_is_rejected() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    h.env.ledger().set_timestamp(1_000);
    h.verify(&alice, LEVEL_BASIC, 2_000);
    h.verify(&bob, LEVEL_BASIC, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);

    // Still inside the validity window.
    h.asset.transfer(&alice, &bob, &100);
    assert_eq!(h.asset.balance(&bob), 100);

    // Past expiry the same transfer is refused.
    h.env.ledger().set_timestamp(2_001);
    let res = h.asset.try_transfer(&alice, &bob, &100);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));
}

#[test]
fn test_transfer_from_screens_both_counterparties_not_the_spender() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    let spender = Address::generate(&h.env);

    h.verify(&alice, LEVEL_BASIC, NEVER_EXPIRES);
    h.verify(&bob, LEVEL_BASIC, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);
    h.asset.approve(&alice, &spender, &500);

    // The spender holds no KYC record and never holds the asset, so the
    // transfer must still settle.
    h.asset.transfer_from(&spender, &alice, &bob, &300);
    assert_eq!(h.asset.balance(&bob), 300);

    // The recipient, by contrast, is screened.
    let carol = Address::generate(&h.env);
    let res = h.asset.try_transfer_from(&spender, &alice, &carol, &100);
    assert_eq!(res, Err(Ok(RwaError::NotCompliant.into())));
}

#[test]
fn test_burn_is_allowed_after_verification_lapses() {
    let h = Harness::new(LEVEL_BASIC);
    let alice = Address::generate(&h.env);

    h.verify(&alice, LEVEL_BASIC, NEVER_EXPIRES);
    h.asset.mint(&h.issuer, &alice, &1_000);
    h.compliance.revoke_kyc(&alice);

    // Exiting a position must not depend on still being verified, or a lapsed
    // holder's tokens would be permanently stranded.
    h.asset.burn(&alice, &400);
    assert_eq!(h.asset.balance(&alice), 600);
    assert_eq!(h.asset.total_supply(), 600);
}
