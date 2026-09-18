#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger,
    },
    Address, Env, MuxedAddress, String,
};

use compliance::{ComplianceContract, ComplianceContractClient};
use stellarforge_common::storage::INSTANCE_BUMP_AMOUNT;
use vault::{
    events, DataKey, VaultConfig, VaultContract, VaultContractClient, VaultError, MAX_LOCKUP_SECS,
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
    assert_eq!(h.client.paused(), false);
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidExchangeRate.into())));
}

#[test]
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidMaxSupply.into())));
}

#[test]
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidLockup.into())));
}

#[test]
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidMetadata.into())));
}

#[test]
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidMetadata.into())));
}

#[test]
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

    let res = env.register_at(&Address::generate(&env), VaultContract, (&admin, &config));
    assert_eq!(res, Err(Ok(VaultError::InvalidUnderlying.into())));
}

// ─── 2. Emergency Pause & Admin Handover Tests ────────────────────────────────

#[test]
fn test_set_paused_takes_effect_in_same_ledger() {
    let h = default_setup();
    assert_eq!(h.client.paused(), false);

    h.client.set_paused(&true);
    assert_eq!(h.client.paused(), true);

    h.client.set_paused(&false);
    assert_eq!(h.client.paused(), false);
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
    assert_eq!(h.client.admin(), new_admin);
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
    assert!(h.client.try_transfer_from(&spender, &alice, &bob, &100).is_ok());
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

    // Set Alice compliant (level 2) but Bob unverified -> fails.
    comp_client.set_kyc(
        &alice,
        &String::from_str(&h.env, "US"),
        &2,
        &0, // never expires
    );
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );

    // Set Bob to level 1 (below min level 2) -> fails.
    comp_client.set_kyc(&bob, &String::from_str(&h.env, "US"), &1, &0);
    assert_eq!(
        h.client.try_transfer(&alice, &muxed_bob, &100),
        Err(Ok(VaultError::NotCompliant.into()))
    );

    // Set Bob to level 2 -> transfer succeeds.
    comp_client.set_kyc(&bob, &String::from_str(&h.env, "US"), &2, &0);
    assert!(h.client.try_transfer(&alice, &muxed_bob, &100).is_ok());
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

// ─── 6. Muxed Address & Self-Transfer Tests ───────────────────────────────────

#[test]
fn test_muxed_transfer_credits_base_account() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    let muxed = MuxedAddress::Muxed {
        id: 9999,
        address: bob.clone(),
    };

    h.client.transfer(&alice, &muxed, &300);

    assert_eq!(h.client.balance(&alice), 700);
    assert_eq!(h.client.balance(&bob), 300);
}

#[test]
fn test_self_transfer_is_noop() {
    let h = default_setup();
    let alice = Address::generate(&h.env);
    h.set_balance(&alice, 1000);

    let muxed = MuxedAddress::from(alice.clone());
    h.client.transfer(&alice, &muxed, &300);

    assert_eq!(h.client.balance(&alice), 1000);
}

// ─── 7. Internal Mint/Burn Helpers Tests ───────────────────────────────────────

#[test]
fn test_internal_mint_and_burn_helpers() {
    let h = default_setup();
    let alice = Address::generate(&h.env);

    h.env.as_contract(&h.contract_id, || {
        VaultContract::mint_shares(&h.env, &alice, 500);
    });

    assert_eq!(h.client.balance(&alice), 500);
    assert_eq!(h.client.total_supply(), 500);

    h.env.as_contract(&h.contract_id, || {
        VaultContract::burn_shares(&h.env, &alice, 200);
    });

    assert_eq!(h.client.balance(&alice), 300);
    assert_eq!(h.client.total_supply(), 300);
}

#[test]
fn test_internal_mint_exceeds_max_supply_rejected() {
    let h = default_setup();
    let alice = Address::generate(&h.env);

    let res = h.env.as_contract(&h.contract_id, || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            VaultContract::mint_shares(&h.env, &alice, 1_000_000_001);
        }))
    });
    assert!(res.is_err());
}

// ─── 8. Storage TTL Policy ───────────────────────────────────────────────────

#[test]
fn test_storage_instance_ttl_bumped() {
    let h = default_setup();
    let instance_ttl = h.env.as_contract(&h.contract_id, || {
        h.env.storage().instance().get_ttl()
    });
    assert!(instance_ttl >= INSTANCE_BUMP_AMOUNT - 100);
}
