#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient};

/// One whole token in base units. The asset uses 7 decimals, matching the
/// Stellar convention where 1 XLM = 10_000_000 stroops.
const UNIT: i128 = 10_000_000;

/// Supply ceiling configured by [`default_metadata`], in base units.
const MAX_SUPPLY: i128 = 1_000_000 * UNIT;

fn create_env() -> Env {
    Env::default()
}

fn default_metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "NYC Real Estate Fund I"),
        symbol: String::from_str(env, "REIT-NYC-001"),
        decimals: 7,
        asset_class: String::from_str(env, "real_estate"),
        legal_doc_hash: Bytes::from_array(env, &[0u8; 32]),
        max_supply: MAX_SUPPLY,
    }
}

fn setup() -> (Env, Address, RwaAssetContractClient<'static>) {
    let env = create_env();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(RwaAssetContract, ());
    let client = RwaAssetContractClient::new(&env, &contract_id);

    client.initialize(&admin, &default_metadata(&env));

    (env, admin, client)
}

// ─── Initialization Tests ──────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let (_, admin, client) = setup();
    assert_eq!(client.admin(), admin);
}

#[test]
fn test_initialize_sets_metadata() {
    let (env, _, client) = setup();
    let meta = client.metadata();
    assert_eq!(meta.symbol, String::from_str(&env, "REIT-NYC-001"));
    assert_eq!(meta.decimals, 7);
}

#[test]
fn test_initialize_not_paused() {
    let (_, _, client) = setup();
    assert!(!client.paused());
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_fails() {
    let (env, admin, client) = setup();
    client.initialize(&admin, &default_metadata(&env));
}

// ─── Issuer Tests ──────────────────────────────────────────────────────────

#[test]
fn test_set_issuer() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);

    assert!(!client.is_issuer(&issuer));
    client.set_issuer(&issuer, &true);
    assert!(client.is_issuer(&issuer));

    client.set_issuer(&issuer, &false);
    assert!(!client.is_issuer(&issuer));
}

// ─── Mint Tests ────────────────────────────────────────────────────────────

#[test]
fn test_mint_increases_balance_and_supply() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &recipient, &(1_000 * UNIT));

    assert_eq!(client.balance(&recipient), 1_000 * UNIT);
    assert_eq!(client.total_supply(), 1_000 * UNIT);
}

#[test]
#[should_panic(expected = "caller is not an issuer")]
fn test_mint_fails_for_non_issuer() {
    let (env, _, client) = setup();
    let non_issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.mint(&non_issuer, &recipient, &100);
}

#[test]
#[should_panic(expected = "exceeds max supply")]
fn test_mint_over_cap_fails() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    // Minting a single token beyond the ceiling must be rejected.
    client.mint(&issuer, &recipient, &(MAX_SUPPLY + UNIT));
}

// ─── Transfer Tests ────────────────────────────────────────────────────────

#[test]
fn test_transfer_moves_balance() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &500);

    client.transfer(&alice, &bob, &200);

    assert_eq!(client.balance(&alice), 300);
    assert_eq!(client.balance(&bob), 200);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_transfer_insufficient_balance_fails() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.transfer(&alice, &bob, &1);
}

// ─── Allowance Tests ───────────────────────────────────────────────────────

#[test]
fn test_approve_and_transfer_from() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1000);
    client.approve(&alice, &bob, &400);

    assert_eq!(client.allowance(&alice, &bob), 400);

    client.transfer_from(&bob, &alice, &carol, &300);

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&carol), 300);
    assert_eq!(client.allowance(&alice, &bob), 100);
}

// ─── Burn Tests ────────────────────────────────────────────────────────────

#[test]
fn test_burn_reduces_balance_and_supply() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1000);
    client.burn(&alice, &400);

    assert_eq!(client.balance(&alice), 600);
    assert_eq!(client.total_supply(), 600);
}

// ─── Pause Tests ───────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "contract is paused")]
fn test_pause_blocks_transfers() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &500);
    client.set_paused(&true);

    client.transfer(&alice, &bob, &100);
}

#[test]
fn test_unpause_allows_transfers() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &500);
    client.set_paused(&true);
    client.set_paused(&false);
    client.transfer(&alice, &bob, &100);

    assert_eq!(client.balance(&bob), 100);
}
