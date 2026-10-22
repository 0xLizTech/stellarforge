#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Events as _, Ledger, MuxedAddress as _,
    },
    Address, Env, MuxedAddress, String,
};

use compliance::{ComplianceContract, ComplianceContractClient, KycRecord};
use soroban_sdk::{Event, IntoVal};
use stellarforge_common::storage::{persistent_bump_amount, INSTANCE_BUMP_AMOUNT};
use vault::{
    AdminTransferred, Approve, ComplianceSet, DataKey, MaxShareSupplySet, Paused, Transfer,
    TransferFrom, VaultConfig, VaultContract, VaultContractClient, VaultError,
    MAX_COMPLIANCE_LEVEL, MAX_LOCKUP_SECS,
};

// ─── Test Helpers & Fixtures ──────────────────────────────────────────────────

#[contract]
struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn decimals(_env: Env) -> u32 {
        7
    }
}

struct TestHarness {
    env: Env,
    admin: Address,
    contract_id: Address,
    client: VaultContractClient<'static>,
    underlying: Address,
    compliance_id: Option<Address>,
}

impl TestHarness {
    fn set_balance(&self, holder: &Address, amount: i128) {
        self.env.as_contract(&self.contract_id, || {
            self.env
                .storage()
                .persistent()
                .set(&DataKey::Balance(holder.clone()), &amount);
            let current_total: i128 = self
                .env
                .storage()
                .persistent()
                .get(&DataKey::TotalSupply)
                .unwrap_or(0);
            self.env
                .storage()
                .persistent()
                .set(&DataKey::TotalSupply, &(current_total + amount));
        });
    }

    fn set_locked_until(&self, holder: &Address, timestamp: u64) {
        self.env.as_contract(&self.contract_id, || {
            self.env
                .storage()
                .persistent()
                .set(&DataKey::LockedUntil(holder.clone()), &timestamp);
        });
    }
}

fn setup_vault(compliance: Option<Address>, min_level: u32) -> TestHarness {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying: underlying.clone(),
        exchange_rate: 2,
        max_share_supply: 1_000_000_000,
        lockup_secs: 86400,
        name: String::from_str(&env, "Institutional RWA Vault Share"),
        symbol: String::from_str(&env, "vRWA-01"),
        compliance: compliance.clone(),
        min_compliance_level: min_level,
    };

    let contract_id = env.register(VaultContract, (&admin, &config));
    let client = VaultContractClient::new(&env, &contract_id);

    TestHarness {
        env,
        admin,
        contract_id,
        client,
        underlying,
        compliance_id: compliance,
    }
}

fn default_setup() -> TestHarness {
    setup_vault(None, 0)
}

fn setup_with_compliance() -> TestHarness {
    let env = Env::default();
    env.mock_all_auths();
    let comp_admin = Address::generate(&env);
    let comp_id = env.register(ComplianceContract, (&comp_admin,));

    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying: underlying.clone(),
        exchange_rate: 1,
        max_share_supply: 10_000_000,
        lockup_secs: 3600,
        name: String::from_str(&env, "Compliant Vault Share"),
        symbol: String::from_str(&env, "cVAULT"),
        compliance: Some(comp_id.clone()),
        min_compliance_level: 2,
    };

    let contract_id = env.register(VaultContract, (&admin, &config));
    let client = VaultContractClient::new(&env, &contract_id);

    TestHarness {
        env,
        admin,
        contract_id,
        client,
        underlying,
        compliance_id: Some(comp_id),
    }
}

// ─── 1. Configuration & Constructor Tests ─────────────────────────────────────

#[test]
fn test_constructor_initial_state() {
    let h = default_setup();
    assert_eq!(h.client.admin(), h.admin);
    assert!(!h.client.paused());
    assert_eq!(h.client.decimals(), 7);
    assert_eq!(h.client.total_supply(), 0);
    assert_eq!(h.client.underlying_held(), 0);
    assert_eq!(
        h.client.name(),
        String::from_str(&h.env, "Institutional RWA Vault Share")
    );
    assert_eq!(h.client.symbol(), String::from_str(&h.env, "vRWA-01"));

    let cfg = h.client.config();
    assert_eq!(cfg.underlying, h.underlying);
    assert_eq!(cfg.exchange_rate, 2);
    assert_eq!(cfg.max_share_supply, 1_000_000_000);
    assert_eq!(cfg.lockup_secs, 86400);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_constructor_rejects_zero_exchange_rate() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 0,
        max_share_supply: 1_000,
        lockup_secs: 60,
        name: String::from_str(&env, "V"),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_constructor_rejects_negative_max_supply() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: -1,
        lockup_secs: 60,
        name: String::from_str(&env, "V"),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_constructor_rejects_excessive_lockup() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: 1_000,
        lockup_secs: MAX_LOCKUP_SECS + 1,
        name: String::from_str(&env, "V"),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_constructor_rejects_empty_name() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: 1_000,
        lockup_secs: 60,
        name: String::from_str(&env, ""),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_constructor_rejects_empty_symbol() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: 1_000,
        lockup_secs: 60,
        name: String::from_str(&env, "V"),
        symbol: String::from_str(&env, ""),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_constructor_rejects_non_token_underlying() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let fake_underlying = Address::generate(&env);

    let config = VaultConfig {
        underlying: fake_underlying,
        exchange_rate: 1,
        max_share_supply: 1_000,
        lockup_secs: 60,
        name: String::from_str(&env, "Vault"),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: 0,
    };

    env.register(VaultContract, (&admin, &config));
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_constructor_rejects_compliance_level_above_max() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: 1_000,
        lockup_secs: 60,
        name: String::from_str(&env, "Vault"),
        symbol: String::from_str(&env, "V"),
        compliance: None,
        min_compliance_level: MAX_COMPLIANCE_LEVEL + 1,
    };

    env.register(VaultContract, (&admin, &config));
}

// ─── 2. Emergency Pause & Admin Handover Tests ────────────────────────────────

#[test]
fn test_set_paused_takes_effect_in_same_ledger() {
    let h = default_setup();
    assert!(!h.client.paused());

    h.client.set_paused(&true);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![Paused { paused: true }.to_xdr(&h.env, &h.contract_id)],
    );
    assert!(h.client.paused());

    h.client.set_paused(&false);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![Paused { paused: false }.to_xdr(&h.env, &h.contract_id)],
    );
    assert!(!h.client.paused());
}

#[test]
fn test_state_changes_rejected_when_paused() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    h.client.set_paused(&true);

    let muxed_bob = MuxedAddress::from(bob.clone());
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::ContractPaused.into()))
    );

    assert_eq!(
        h.client.try_approve(&alice, &bob, &500, &1000),
        Err(Ok(VaultError::ContractPaused.into()))
    );

    assert_eq!(
        h.client.try_transfer_from(&bob, &alice, &bob, &100),
        Err(Ok(VaultError::ContractPaused.into()))
    );
}

#[test]
fn test_transfer_admin_dual_auth() {
    let h = default_setup();
    let new_admin = Address::generate(&h.env);

    h.client.transfer_admin(&new_admin);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![AdminTransferred {
            previous: h.admin.clone(),
            new_admin: new_admin.clone(),
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert_eq!(h.client.admin(), new_admin);
}

#[test]
fn test_set_compliance_rebinds_and_clears_screening() {
    let h = default_setup();
    let comp_admin = Address::generate(&h.env);
    let comp_a = h.env.register(ComplianceContract, (&comp_admin,));
    let comp_b = h.env.register(ComplianceContract, (&comp_admin,));
    let comp_a_client = ComplianceContractClient::new(&h.env, &comp_a);
    let comp_b_client = ComplianceContractClient::new(&h.env, &comp_b);

    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1_000);
    let muxed_bob = MuxedAddress::from(bob.clone());

    // Bind to comp_a at level 2.
    h.client.set_compliance(&Some(comp_a.clone()), &2);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![ComplianceSet {
            compliance: Some(comp_a.clone()),
            min_level: 2,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    let cfg = h.client.config();
    assert_eq!(cfg.compliance, Some(comp_a));
    assert_eq!(cfg.min_compliance_level, 2);

    // Unverified transfer is now refused.
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );

    // Enroll Alice and Bob on comp_b at level 1, then rebind vault to comp_b at level 1.
    comp_b_client.set_kyc(
        &alice,
        &KycRecord {
            jurisdiction: String::from_str(&h.env, "US"),
            level: 1,
            expires_at: 0,
        },
    );
    comp_b_client.set_kyc(
        &bob,
        &KycRecord {
            jurisdiction: String::from_str(&h.env, "US"),
            level: 1,
            expires_at: 0,
        },
    );
    let _ = comp_a_client;
    h.client.set_compliance(&Some(comp_b.clone()), &1);
    assert!(h.client.try_transfer(&alice, &muxed_bob, &100).is_ok());

    // Revoke Bob on comp_b -> blocked again; then clear compliance -> succeeds.
    comp_b_client.revoke_kyc(&bob);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );

    h.client.set_compliance(&None, &0);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![ComplianceSet {
            compliance: None,
            min_level: 0,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert!(h.client.try_transfer(&alice, &muxed_bob, &100).is_ok());
}

#[test]
fn test_set_compliance_rejects_level_above_max() {
    let h = default_setup();
    let comp = Address::generate(&h.env);

    assert_eq!(
        h.client
            .try_set_compliance(&Some(comp), &(MAX_COMPLIANCE_LEVEL + 1)),
        Err(Ok(VaultError::InvalidComplianceLevel.into()))
    );
}

#[test]
fn test_set_compliance_requires_admin_auth() {
    let h = default_setup();
    let comp = Address::generate(&h.env);

    h.env.mock_auths(&[]);
    assert!(h
        .client
        .try_set_compliance(&Some(comp.clone()), &1)
        .is_err());

    h.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &h.admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_compliance",
            args: (Some(comp.clone()), 1_u32).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);
    h.client.set_compliance(&Some(comp.clone()), &1);
    assert_eq!(h.client.config().compliance, Some(comp));
}

#[test]
fn test_set_max_share_supply_tightens_cap_and_caps_uncapped_vault() {
    let h = default_setup();
    assert_eq!(h.client.config().max_share_supply, 1_000_000_000);

    // Tighten from 1_000_000_000 down to 500_000.
    h.client.set_max_share_supply(&500_000);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![MaxShareSupplySet {
            previous: 1_000_000_000,
            max_share_supply: 500_000,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert_eq!(h.client.config().max_share_supply, 500_000);

    // Setting the same cap is a valid no-change tightening.
    h.client.set_max_share_supply(&500_000);
    assert_eq!(h.client.config().max_share_supply, 500_000);

    // Uncapped vault (0) can be given a finite cap at or above total_supply.
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());
    let uncapped_cfg = VaultConfig {
        underlying,
        exchange_rate: 1,
        max_share_supply: 0,
        lockup_secs: 60,
        name: String::from_str(&env, "Uncapped Vault"),
        symbol: String::from_str(&env, "uVAULT"),
        compliance: None,
        min_compliance_level: 0,
    };
    let uncapped_id = env.register(VaultContract, (&admin, &uncapped_cfg));
    let uncapped_client = VaultContractClient::new(&env, &uncapped_id);

    uncapped_client.set_max_share_supply(&250_000);
    assert_eq!(uncapped_client.config().max_share_supply, 250_000);
}

#[test]
fn test_set_max_share_supply_rejects_raising_cap() {
    let h = default_setup();
    h.client.set_max_share_supply(&500_000);

    assert_eq!(
        h.client.try_set_max_share_supply(&500_001),
        Err(Ok(VaultError::InvalidSupplyCap.into()))
    );
    assert_eq!(h.client.config().max_share_supply, 500_000);
}

#[test]
fn test_set_max_share_supply_rejects_removing_cap() {
    let h = default_setup();
    h.client.set_max_share_supply(&500_000);

    assert_eq!(
        h.client.try_set_max_share_supply(&0),
        Err(Ok(VaultError::InvalidSupplyCap.into()))
    );
    assert_eq!(h.client.config().max_share_supply, 500_000);
}

#[test]
fn test_set_max_share_supply_rejects_cap_below_total_supply() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    h.set_balance(&alice, 600);

    assert_eq!(
        h.client.try_set_max_share_supply(&599),
        Err(Ok(VaultError::InvalidSupplyCap.into()))
    );

    // Exact circulating supply is allowed.
    h.client.set_max_share_supply(&600);
    assert_eq!(h.client.config().max_share_supply, 600);
}

#[test]
fn test_set_max_share_supply_rejects_negative_cap() {
    let h = default_setup();
    assert_eq!(
        h.client.try_set_max_share_supply(&-1),
        Err(Ok(VaultError::InvalidSupplyCap.into()))
    );
}

#[test]
fn test_set_max_share_supply_requires_admin_auth() {
    let h = default_setup();

    h.env.mock_auths(&[]);
    assert!(h.client.try_set_max_share_supply(&500_000).is_err());

    h.env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &h.admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_max_share_supply",
            args: (500_000_i128,).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);
    h.client.set_max_share_supply(&500_000);
    assert_eq!(h.client.config().max_share_supply, 500_000);
}

// ─── 3. Amount, Balance & Allowance Validations ───────────────────────────────

#[test]
fn test_transfer_rejects_zero_or_negative_amount() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    let muxed = MuxedAddress::from(bob);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed, &0),
        Err(Ok(VaultError::InvalidAmount.into()))
    );
    assert_eq!(
        h.client.try_transfer(&alice, &muxed, &-50),
        Err(Ok(VaultError::InvalidAmount.into()))
    );
}

#[test]
fn test_approve_rejects_negative_amount() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    assert_eq!(
        h.client.try_approve(&alice, &bob, &-10, &100),
        Err(Ok(VaultError::InvalidAmount.into()))
    );
}

#[test]
fn test_approve_rejects_past_expiration() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);

    h.env.ledger().set_sequence_number(50);
    assert_eq!(
        h.client.try_approve(&alice, &bob, &100, &49),
        Err(Ok(VaultError::InvalidExpiration.into()))
    );

    // Revocation with 0 amount is allowed even in the past.
    assert!(h.client.try_approve(&alice, &bob, &0, &10).is_ok());
}

#[test]
fn test_transfer_rejects_insufficient_balance() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 100);

    let muxed = MuxedAddress::from(bob);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed, &101),
        Err(Ok(VaultError::InsufficientBalance.into()))
    );
}

#[test]
fn test_transfer_from_rejects_insufficient_allowance() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let spender = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    h.client.approve(&alice, &spender, &200, &1000);

    assert_eq!(
        h.client.try_transfer_from(&spender, &alice, &bob, &201),
        Err(Ok(VaultError::InsufficientAllowance.into()))
    );
}

// ─── 4. Lock-up Exact Boundary Tests ──────────────────────────────────────────

#[test]
fn test_lockup_exact_boundary_enforcement() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 500);

    let lock_expiry = 10_000_u64;
    h.set_locked_until(&alice, lock_expiry);

    let muxed = MuxedAddress::from(bob.clone());

    // 1 second before lock expiry: MUST be refused with SharesLocked.
    h.env.ledger().set_timestamp(lock_expiry - 1);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed, &100),
        Err(Ok(VaultError::SharesLocked.into()))
    );

    // Exactly at lock expiry: MUST succeed.
    h.env.ledger().set_timestamp(lock_expiry);
    assert!(h.client.try_transfer(&alice, &muxed, &100).is_ok());
    assert_eq!(h.client.balance(&alice), 400);
    assert_eq!(h.client.balance(&bob), 100);
}

#[test]
fn test_transfer_from_refused_for_locked_owner() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let spender = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 500);

    let lock_expiry = 50_000_u64;
    h.set_locked_until(&alice, lock_expiry);

    h.client.approve(&alice, &spender, &300, &1000);

    h.env.ledger().set_timestamp(lock_expiry - 1);
    assert_eq!(
        h.client.try_transfer_from(&spender, &alice, &bob, &100),
        Err(Ok(VaultError::SharesLocked.into()))
    );

    h.env.ledger().set_timestamp(lock_expiry);
    assert!(h
        .client
        .try_transfer_from(&spender, &alice, &bob, &100)
        .is_ok());
    assert_eq!(h.client.balance(&bob), 100);
    assert_eq!(h.client.allowance(&alice, &spender), 200);
}

// ─── 5. Compliance Screening Tests ────────────────────────────────────────────

#[test]
fn test_compliance_screening_enforced() {
    let h = setup_with_compliance();
    let comp_client = ComplianceContractClient::new(&h.env, h.compliance_id.as_ref().unwrap());

    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    let muxed_bob = MuxedAddress::from(bob.clone());

    // Neither is compliant -> fails.
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );
    assert_eq!(h.client.balance(&alice), 1000);
    assert_eq!(h.client.balance(&bob), 0);

    // Set Alice compliant (level 2) but Bob unverified -> fails.
    comp_client.set_kyc(
        &alice,
        &KycRecord {
            jurisdiction: String::from_str(&h.env, "US"),
            level: 2,
            expires_at: 0, // never expires
        },
    );
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );
    assert_eq!(h.client.balance(&alice), 1000);
    assert_eq!(h.client.balance(&bob), 0);

    // Set Bob to level 1 (below min level 2) -> fails.
    comp_client.set_kyc(
        &bob,
        &KycRecord {
            jurisdiction: String::from_str(&h.env, "US"),
            level: 1,
            expires_at: 0,
        },
    );
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );
    assert_eq!(h.client.balance(&alice), 1000);
    assert_eq!(h.client.balance(&bob), 0);

    // Set Bob to level 2 -> transfer succeeds.
    comp_client.set_kyc(
        &bob,
        &KycRecord {
            jurisdiction: String::from_str(&h.env, "US"),
            level: 2,
            expires_at: 0,
        },
    );
    assert!(h.client.try_transfer(&alice, &muxed_bob, &100).is_ok());
    assert_eq!(h.client.balance(&alice), 900);
    assert_eq!(h.client.balance(&bob), 100);

    // Revoke Alice -> transfer fails and changes no state.
    comp_client.revoke_kyc(&alice);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );
    assert_eq!(h.client.balance(&alice), 900);
    assert_eq!(h.client.balance(&bob), 100);
}

#[test]
fn test_unscreened_when_no_compliance_configured() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 500);

    let muxed = MuxedAddress::from(bob.clone());
    assert!(h.client.try_transfer(&alice, &muxed, &250).is_ok());
    assert_eq!(h.client.balance(&bob), 250);
}

// ─── 6. Muxed Address, Events & Overflow Tests ───────────────────────────────

#[test]
fn test_muxed_transfer_credits_base_account() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let muxed = MuxedAddress::generate(&h.env);
    let bob = muxed.address();
    let muxed_id = muxed.id();
    h.set_balance(&alice, 1000);

    h.client.transfer(&alice, &muxed, &300);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![Transfer {
            from: alice.clone(),
            to: bob.clone(),
            amount: 300,
            to_muxed_id: muxed_id,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );

    assert_eq!(h.client.balance(&alice), 700);
    assert_eq!(h.client.balance(&bob), 300);
}

#[test]
fn test_approve_and_transfer_from_publish_sep41_events() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let spender = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    h.client.approve(&alice, &spender, &400, &1000);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![Approve {
            owner: alice.clone(),
            spender: spender.clone(),
            amount: 400,
            live_until_ledger: 1000,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );

    h.client.transfer_from(&spender, &alice, &bob, &250);
    assert_eq!(
        h.env.events().all().events(),
        std::vec![TransferFrom {
            from: alice,
            to: bob,
            amount: 250,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_transfer_overflow_rejected() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.env.as_contract(&h.contract_id, || {
        h.env
            .storage()
            .persistent()
            .set(&DataKey::Balance(alice.clone()), &100_i128);
        h.env
            .storage()
            .persistent()
            .set(&DataKey::Balance(bob.clone()), &i128::MAX);
    });

    let muxed_bob = MuxedAddress::from(bob);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &1),
        Err(Ok(VaultError::Overflow.into()))
    );
}

#[test]
fn test_self_transfer_is_noop() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    let muxed = MuxedAddress::from(alice.clone());
    h.client.transfer(&alice, &muxed, &300);
    assert!(
        h.env.events().all().events().is_empty(),
        "self-transfer no-op published an event"
    );

    assert_eq!(h.client.balance(&alice), 1000);
}

// ─── 7. Storage TTL Policy ───────────────────────────────────────────────────

#[test]
fn test_storage_instance_and_persistent_ttl_bumped() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);
    h.client.approve(&alice, &bob, &200, &1000);
    h.client
        .transfer(&alice, MuxedAddress::from(bob.clone()), &100);

    let instance_ttl = h
        .env
        .as_contract(&h.contract_id, || h.env.storage().instance().get_ttl());
    assert_eq!(instance_ttl, INSTANCE_BUMP_AMOUNT);

    let expected_persistent = persistent_bump_amount(&h.env);
    let balance_ttl = h.env.as_contract(&h.contract_id, || {
        h.env
            .storage()
            .persistent()
            .get_ttl(&DataKey::Balance(bob.clone()))
    });
    assert_eq!(balance_ttl, expected_persistent);

    let allowance_ttl = h.env.as_contract(&h.contract_id, || {
        h.env
            .storage()
            .persistent()
            .get_ttl(&DataKey::Allowance(alice.clone(), bob.clone()))
    });
    assert_eq!(allowance_ttl, expected_persistent);
}
