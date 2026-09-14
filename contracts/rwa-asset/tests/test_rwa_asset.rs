#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient, RwaError};

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
    let contract_id = env.register(RwaAssetContract, (&admin, &default_metadata(&env)));
    let client = RwaAssetContractClient::new(&env, &contract_id);

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

// `test_double_initialize_fails` was removed with the `initialize` entry point.
// A constructor configures the contract inside the deploy transaction, so a
// second configuration call is not something the contract exposes.

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
fn test_mint_fails_for_non_issuer() {
    let (env, _, client) = setup();
    let non_issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let res = client.try_mint(&non_issuer, &recipient, &100);
    assert_eq!(res, Err(Ok(RwaError::NotIssuer.into())));
}

#[test]
fn test_mint_over_cap_fails() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    // Minting a single token beyond the ceiling must be rejected.
    let res = client.try_mint(&issuer, &recipient, &(MAX_SUPPLY + UNIT));
    assert_eq!(res, Err(Ok(RwaError::ExceedsMaxSupply.into())));
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
fn test_transfer_insufficient_balance_fails() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let res = client.try_transfer(&alice, &bob, &1);
    assert_eq!(res, Err(Ok(RwaError::InsufficientBalance.into())));
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
fn test_pause_blocks_transfers() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &500);
    client.set_paused(&true);

    let res = client.try_transfer(&alice, &bob, &100);
    assert_eq!(res, Err(Ok(RwaError::ContractPaused.into())));
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

// ─── Self-Transfer Invariant Tests ─────────────────────────────────────────

#[test]
fn test_self_transfer_does_not_create_tokens() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1_000);

    client.transfer(&alice, &alice, &400);

    assert_eq!(
        client.balance(&alice),
        1_000,
        "self-transfer changed balance"
    );
    assert_eq!(client.total_supply(), 1_000, "supply desynchronised");
}

#[test]
fn test_self_transfer_from_does_not_create_tokens() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let spender = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1_000);
    client.approve(&alice, &spender, &500);

    client.transfer_from(&spender, &alice, &alice, &400);

    assert_eq!(
        client.balance(&alice),
        1_000,
        "self-transfer changed balance"
    );
    assert_eq!(client.total_supply(), 1_000, "supply desynchronised");
    assert_eq!(
        client.allowance(&alice, &spender),
        100,
        "allowance not spent"
    );
}

/// Asserts the protocol's central accounting invariant: the sum of every
/// holder's balance must equal the reported total supply. Both self-transfer
/// bugs broke exactly this relationship while leaving each individual
/// balance query looking plausible.
fn assert_supply_invariant(client: &RwaAssetContractClient, holders: &[&Address]) {
    let sum: i128 = holders.iter().map(|h| client.balance(h)).sum();
    assert_eq!(sum, client.total_supply(), "sum(balances) != total_supply");
}

#[test]
fn test_supply_invariant_holds_across_operations() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    let holders = [&alice, &bob, &carol];

    client.mint(&issuer, &alice, &(1_000 * UNIT));
    assert_supply_invariant(&client, &holders);

    client.transfer(&alice, &bob, &(250 * UNIT));
    assert_supply_invariant(&client, &holders);

    client.transfer(&alice, &alice, &(100 * UNIT));
    assert_supply_invariant(&client, &holders);

    client.approve(&alice, &bob, &(300 * UNIT));
    client.transfer_from(&bob, &alice, &carol, &(200 * UNIT));
    assert_supply_invariant(&client, &holders);

    client.transfer_from(&bob, &alice, &alice, &(50 * UNIT));
    assert_supply_invariant(&client, &holders);

    client.burn(&bob, &(100 * UNIT));
    assert_supply_invariant(&client, &holders);
}

// ─── Validation & Overflow Tests ───────────────────────────────────────────

/// Registers a contract with caller-supplied metadata, bypassing the capped
/// default, so supply-boundary behaviour can be exercised directly.
fn setup_with(metadata: AssetMetadata, env: &Env) -> (Address, RwaAssetContractClient<'static>) {
    let admin = Address::generate(env);
    let contract_id = env.register(RwaAssetContract, (&admin, &metadata));
    let client = RwaAssetContractClient::new(env, &contract_id);
    (admin, client)
}

fn uncapped_metadata(env: &Env) -> AssetMetadata {
    let mut m = default_metadata(env);
    m.max_supply = 0; // 0 means uncapped
    m
}

#[test]
fn test_mint_overflow_is_rejected() {
    let env = create_env();
    env.mock_all_auths();
    let (_, client) = setup_with(uncapped_metadata(&env), &env);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    client.set_issuer(&issuer, &true);

    client.mint(&issuer, &alice, &i128::MAX);
    assert_eq!(client.total_supply(), i128::MAX);

    // One more unit would overflow i128 rather than wrap.
    let res = client.try_mint(&issuer, &alice, &1);
    assert_eq!(res, Err(Ok(RwaError::Overflow.into())));
}

#[test]
fn test_zero_and_negative_amounts_are_rejected() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1_000);

    let expected = Err(Ok(RwaError::InvalidAmount.into()));
    assert_eq!(client.try_transfer(&alice, &bob, &0), expected);
    assert_eq!(client.try_transfer(&alice, &bob, &-1), expected);
    assert_eq!(client.try_mint(&issuer, &bob, &0), expected);
    assert_eq!(client.try_burn(&alice, &0), expected);
    assert_eq!(client.try_approve(&alice, &bob, &-1), expected);
}

#[test]
fn test_transfer_from_without_allowance_is_rejected() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &1_000);

    let res = client.try_transfer_from(&bob, &alice, &bob, &100);
    assert_eq!(res, Err(Ok(RwaError::InsufficientAllowance.into())));
}

fn invalid_metadata_cases(env: &Env) -> [AssetMetadata; 3] {
    let mut too_many_decimals = default_metadata(env);
    too_many_decimals.decimals = 19;

    let mut negative_cap = default_metadata(env);
    negative_cap.max_supply = -1;

    let mut empty_symbol = default_metadata(env);
    empty_symbol.symbol = String::from_str(env, "");

    [too_many_decimals, negative_cap, empty_symbol]
}

/// Case-by-case coverage of `validate_metadata` moved here when `initialize`
/// became a constructor. `update_metadata` runs the same validation and is
/// still callable through `try_`, so each rule keeps a typed-error assertion;
/// a constructor can only be observed panicking, which cannot distinguish
/// `InvalidMetadata` from any other failure.
#[test]
fn test_update_metadata_rejects_bad_input() {
    let (env, _, client) = setup();

    for meta in invalid_metadata_cases(&env) {
        let res = client.try_update_metadata(&meta);
        assert_eq!(res, Err(Ok(RwaError::InvalidMetadata.into())));
    }
}

/// Guards the negative tests above: were `update_metadata` to reject
/// everything, they would all still pass.
#[test]
fn test_update_metadata_accepts_valid_input() {
    let (env, _, client) = setup();

    let mut renamed = default_metadata(&env);
    renamed.symbol = String::from_str(&env, "REIT-NYC-002");
    client.update_metadata(&renamed);

    assert_eq!(
        client.metadata().symbol,
        String::from_str(&env, "REIT-NYC-002")
    );
}

/// The constructor must validate too, or a contract could be deployed holding
/// metadata that `update_metadata` would refuse. Asserted as a panic because
/// `Env::register` has no fallible form.
#[test]
#[should_panic]
fn test_constructor_rejects_invalid_metadata() {
    let env = create_env();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let mut too_many_decimals = default_metadata(&env);
    too_many_decimals.decimals = 19;

    env.register(RwaAssetContract, (&admin, &too_many_decimals));
}

#[test]
fn test_self_transfer_still_validates_balance() {
    let (env, _, client) = setup();
    let alice = Address::generate(&env);

    // The no-op path must not become a way to bypass the balance check.
    let res = client.try_transfer(&alice, &alice, &100);
    assert_eq!(res, Err(Ok(RwaError::InsufficientBalance.into())));
}
