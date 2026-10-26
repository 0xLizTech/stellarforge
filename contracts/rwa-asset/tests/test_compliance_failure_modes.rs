#![cfg(test)]

//! Proves, rather than only documents, the compliance failure modes ADR-004
//! and the audit handover describe: a transfer fails closed once a
//! compliance contract is configured but misbehaves, and fails open only
//! when none is configured at all (#143).
//!
//! `TrapContract` and `WrongShapeContract` below are deliberately broken
//! stand-ins for a real compliance engine, registered the same way the
//! integration tests in `test_compliance.rs` register the real one. A
//! panicking cross-contract call is a genuine Rust panic, not a catchable
//! contract error, so these are asserted with `#[should_panic]` (the same
//! convention `test_rwa_asset.rs::test_constructor_rejects_invalid_metadata`
//! uses for a failure with no fallible `try_*` form), rather than inspecting
//! a `try_*` result.

use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Bytes, Env, String};

use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient};

/// Stands in for a compliance contract that panics on every call, as a
/// wrong address, a redeployed-and-broken contract, or a contract mid
/// upgrade might.
#[contract]
struct TrapContract;

#[contractimpl]
impl TrapContract {
    pub fn screen(_env: Env, _subject: Address, _min_level: u32) -> bool {
        panic!("trap contract: always panics");
    }
}

/// Implements some entry point, but not `screen`, standing in for an address
/// that is a deployed contract yet not a conforming compliance engine at
/// all (the address of an unrelated contract, or a typo that happens to
/// resolve).
#[contract]
struct WrongShapeContract;

#[contractimpl]
impl WrongShapeContract {
    pub fn ping(_env: Env) -> bool {
        true
    }
}

struct Harness<'a> {
    env: Env,
    asset: RwaAssetContractClient<'a>,
    issuer: Address,
}

impl Harness<'_> {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let asset_id = env.register(RwaAssetContract, (&admin, &metadata(&env)));
        let asset = RwaAssetContractClient::new(&env, &asset_id);

        let issuer = Address::generate(&env);
        asset.set_issuer(&issuer, &true);

        Self { env, asset, issuer }
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

// ─── Unconfigured: fails open ───────────────────────────────────────────────

#[test]
fn test_unconfigured_compliance_fails_open() {
    let h = Harness::new();
    assert!(h.asset.compliance_contract().is_none());

    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);
    h.asset.transfer(&alice, &bob, &400);

    assert_eq!(h.asset.balance(&bob), 400);
}

// ─── Configured but broken: fails closed ───────────────────────────────────

#[test]
#[should_panic]
fn test_a_panicking_compliance_contract_fails_mint_closed() {
    let h = Harness::new();
    let trap_id = h.env.register(TrapContract, ());
    h.asset.set_compliance(&Some(trap_id), &0);

    let alice = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);
}

#[test]
#[should_panic]
fn test_a_panicking_compliance_contract_fails_transfer_closed() {
    let h = Harness::new();

    // Mint before the trap is wired up, so the transfer attempt below is the
    // only call that reaches the broken compliance contract.
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);

    let trap_id = h.env.register(TrapContract, ());
    h.asset.set_compliance(&Some(trap_id), &0);

    h.asset.transfer(&alice, &bob, &400);
}

#[test]
#[should_panic]
fn test_a_contract_not_implementing_screen_fails_transfer_closed() {
    let h = Harness::new();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);

    let wrong_shape_id = h.env.register(WrongShapeContract, ());
    h.asset.set_compliance(&Some(wrong_shape_id), &0);

    h.asset.transfer(&alice, &bob, &400);
}

#[test]
#[should_panic]
fn test_an_address_with_no_deployed_contract_fails_transfer_closed() {
    let h = Harness::new();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);

    // A plain generated address, never registered as any contract.
    let nonexistent = Address::generate(&h.env);
    h.asset.set_compliance(&Some(nonexistent), &0);

    h.asset.transfer(&alice, &bob, &400);
}

// ─── Recovery and what keeps working in the meantime ───────────────────────

#[test]
fn test_recovery_is_removing_or_replacing_the_broken_compliance_contract() {
    let h = Harness::new();
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);

    let trap_id = h.env.register(TrapContract, ());
    h.asset.set_compliance(&Some(trap_id), &0);

    // The admin's only recovery is `set_compliance` again: either remove it
    // (fail open) or point it at a contract that actually implements
    // `screen` (ADR-004's "Recovery is an admin call to `set_compliance`").
    // Removing it here, rather than calling the broken contract again,
    // is the point: this test never triggers the panic, it only shows the
    // asset is usable again once the admin acts.
    h.asset.set_compliance(&None, &0);
    h.asset.transfer(&alice, &bob, &400);
    assert_eq!(h.asset.balance(&bob), 400);
}

#[test]
fn test_burn_and_approve_still_work_while_compliance_is_broken() {
    let h = Harness::new();
    let alice = Address::generate(&h.env);
    let spender = Address::generate(&h.env);
    h.asset.mint(&h.issuer, &alice, &1_000);

    let trap_id = h.env.register(TrapContract, ());
    h.asset.set_compliance(&Some(trap_id), &0);

    // ADR-004: burn and approve screen nobody, so a holder stuck behind a
    // broken compliance contract can still exit their position.
    h.asset.burn(&alice, &300);
    assert_eq!(h.asset.balance(&alice), 700);

    h.asset.approve(&alice, &spender, &100, &1_000);
    assert_eq!(h.asset.allowance(&alice, &spender), 100);
}
