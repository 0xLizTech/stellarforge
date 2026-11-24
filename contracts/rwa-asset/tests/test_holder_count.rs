#![cfg(test)]

//! Property test for the holder count invariant (#146).
//!
//! After any sequence of mints, burns and transfers, the stored `holder_count`
//! must equal the number of addresses whose `balance` is strictly positive.

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient, RwaError};

const UNIT: i128 = 10_000_000;

fn default_metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "NYC Real Estate Fund I"),
        symbol: String::from_str(env, "REIT-NYC-001"),
        decimals: 7,
        asset_class: String::from_str(env, "real_estate"),
        legal_doc_hash: Bytes::from_array(env, &[1u8; 32]),
        max_supply: 0, // uncapped so mints never hit the supply ceiling
    }
}

fn setup() -> (Env, Address, RwaAssetContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(RwaAssetContract, (&admin, &default_metadata(&env)));
    let client = RwaAssetContractClient::new(&env, &contract_id);

    (env, admin, client)
}

/// Counts addresses with strictly positive balances by querying each holder.
fn count_nonzero_balances(client: &RwaAssetContractClient, holders: &[Address]) -> u32 {
    holders
        .iter()
        .filter(|addr| client.balance(addr) > 0)
        .count() as u32
}

// ── Basic Holder Count Tests ────────────────────────────────────────────────

#[test]
fn test_holder_count_starts_at_zero() {
    let (_, _, client) = setup();
    assert_eq!(client.holder_count(), 0);
}

#[test]
fn test_mint_to_one_new_address_increments_count() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    assert_eq!(client.holder_count(), 0);

    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);
}

#[test]
fn test_mint_to_same_address_does_not_increment_again() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);

    client.mint(&issuer, &alice, &(50 * UNIT));
    assert_eq!(client.holder_count(), 1);
}

#[test]
fn test_burn_to_zero_decrements_count() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);

    client.burn(&alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 0);
}

#[test]
fn test_partial_burn_does_not_decrement() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);

    client.burn(&alice, &(50 * UNIT));
    assert_eq!(client.holder_count(), 1);
    assert_eq!(client.balance(&alice), 50 * UNIT);
}

#[test]
fn test_transfer_to_new_address_increments_count() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);

    client.transfer(&alice, &bob, &(40 * UNIT));
    assert_eq!(client.holder_count(), 2);
}

#[test]
fn test_transfer_all_tokens_decrements_sender_count() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 1);

    client.transfer(&alice, &bob, &(100 * UNIT));
    // Alice has zero, Bob has 100 → count stays 1
    assert_eq!(client.holder_count(), 1);
    assert_eq!(client.balance(&alice), 0);
    assert_eq!(client.balance(&bob), 100 * UNIT);
}

#[test]
fn test_transfer_to_existing_holder_does_not_increment() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    client.mint(&issuer, &bob, &(50 * UNIT));
    assert_eq!(client.holder_count(), 2);

    client.transfer(&alice, &bob, &(10 * UNIT));
    assert_eq!(client.holder_count(), 2);
}

#[test]
fn test_burn_from_to_zero_decrements_count() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let spender = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    client.approve(&alice, &spender, &(100 * UNIT), &1000);
    assert_eq!(client.holder_count(), 1);

    client.burn_from(&spender, &alice, &(100 * UNIT));
    assert_eq!(client.holder_count(), 0);
}

#[test]
fn test_transfer_from_to_new_address_increments() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    client.approve(&alice, &spender, &(100 * UNIT), &1000);
    assert_eq!(client.holder_count(), 1);

    client.transfer_from(&spender, &alice, &bob, &(50 * UNIT));
    assert_eq!(client.holder_count(), 2);
}

#[test]
fn test_transfer_from_all_tokens_replaces_holder() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    client.set_issuer(&issuer, &true);
    client.mint(&issuer, &alice, &(100 * UNIT));
    client.approve(&alice, &spender, &(100 * UNIT), &1000);
    assert_eq!(client.holder_count(), 1);

    client.transfer_from(&spender, &alice, &bob, &(100 * UNIT));
    // Alice 0, Bob 100 → still 1 holder
    assert_eq!(client.holder_count(), 1);
}

// ── Holder Cap Tests ────────────────────────────────────────────────────────

#[test]
fn test_holder_count_cap_starts_none() {
    let (_, _, client) = setup();
    assert_eq!(client.holder_count_cap(), None);
}

#[test]
fn test_set_holder_cap_configures_cap() {
    let (_, _, client) = setup();
    client.set_holder_cap(&Some(5));
    assert_eq!(client.holder_count_cap(), Some(5));
}

#[test]
fn test_set_holder_cap_none_removes_cap() {
    let (_, _, client) = setup();
    client.set_holder_cap(&Some(5));
    assert_eq!(client.holder_count_cap(), Some(5));

    client.set_holder_cap(&None);
    assert_eq!(client.holder_count_cap(), None);
}

#[test]
fn test_mint_respects_holder_cap() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.set_holder_cap(&Some(2));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    client.mint(&issuer, &alice, &(100 * UNIT));
    client.mint(&issuer, &bob, &(100 * UNIT));
    assert_eq!(client.holder_count(), 2);

    // Minting to a third new holder exceeds the cap
    let res = client.try_mint(&issuer, &charlie, &(100 * UNIT));
    assert_eq!(res, Err(Ok(RwaError::ExceedsHolderCap.into())));
    assert_eq!(client.holder_count(), 2);
}

#[test]
fn test_mint_to_existing_holder_bypasses_cap() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.set_holder_cap(&Some(2));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.mint(&issuer, &alice, &(100 * UNIT));
    client.mint(&issuer, &bob, &(100 * UNIT));
    assert_eq!(client.holder_count(), 2);

    // Minting more to Alice is allowed — she's already a holder
    client.mint(&issuer, &alice, &(50 * UNIT));
    assert_eq!(client.balance(&alice), 150 * UNIT);
    assert_eq!(client.holder_count(), 2);
}

#[test]
fn test_transfer_respects_holder_cap() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.set_holder_cap(&Some(2));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    client.mint(&issuer, &alice, &(100 * UNIT));
    client.mint(&issuer, &bob, &(100 * UNIT));
    assert_eq!(client.holder_count(), 2);

    let res = client.try_transfer(&alice, &charlie, &(10 * UNIT));
    assert_eq!(res, Err(Ok(RwaError::ExceedsHolderCap.into())));
}

#[test]
fn test_cap_below_current_count_prevents_new_holders_but_allows_existing() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    client.mint(&issuer, &alice, &(100 * UNIT));
    client.mint(&issuer, &bob, &(100 * UNIT));
    client.mint(&issuer, &charlie, &(100 * UNIT));
    assert_eq!(client.holder_count(), 3);

    // Set cap below the current count
    client.set_holder_cap(&Some(2));
    assert_eq!(client.holder_count(), 3);
    assert_eq!(client.holder_count_cap(), Some(2));

    // New holders are rejected
    let dave = Address::generate(&env);
    let res = client.try_mint(&issuer, &dave, &(10 * UNIT));
    assert_eq!(res, Err(Ok(RwaError::ExceedsHolderCap.into())));

    // Existing holders can still receive
    client.mint(&issuer, &alice, &(50 * UNIT));
    assert_eq!(client.balance(&alice), 150 * UNIT);
}

// ── Property Test: Holder Count Invariant ──────────────────────────────────

/// Property test: after any sequence of operations, `holder_count` equals the
/// number of addresses whose balance is strictly positive.
///
/// Performs 50 random operations (mint/burn/transfer) and checks the invariant
/// after each one. The operations intentionally:
/// - Mint to both new and existing holders
/// - Burn partially and fully
/// - Transfer to new and existing holders, including full balance transfers
///
/// This covers the edge cases: balance crossing 0 → >0, >0 → 0, and staying >0.
#[test]
fn test_holder_count_invariant() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);

    // Pre-generate a fixed pool of addresses
    let holders: Vec<Address> = (0..10).map(|_| Address::generate(&env)).collect();

    // Perform a sequence of random operations
    for i in 0..50 {
        let addr_idx = i % holders.len();
        let addr = &holders[addr_idx];
        let other_idx = (i + 1) % holders.len();
        let other = &holders[other_idx];

        match i % 4 {
            0 => {
                // Mint to a holder
                client.mint(&issuer, addr, &(10 * UNIT));
            }
            1 => {
                // Burn if balance is positive
                let bal = client.balance(addr);
                if bal > 0 {
                    let burn_amount = if bal >= 5 * UNIT { 5 * UNIT } else { bal };
                    client.burn(addr, &burn_amount);
                }
            }
            2 => {
                // Transfer to another holder
                let bal = client.balance(addr);
                if bal > 0 {
                    let transfer_amount = if bal >= 3 * UNIT { 3 * UNIT } else { bal };
                    client.transfer(addr, other, &transfer_amount);
                }
            }
            _ => {
                // Mint to a different holder
                client.mint(&issuer, other, &(7 * UNIT));
            }
        }

        // Invariant check: holder_count == number of non-zero balances
        let expected_count = count_nonzero_balances(&client, &holders);
        let actual_count = client.holder_count();
        assert_eq!(
            actual_count, expected_count,
            "Holder count mismatch at step {}: actual={}, expected={}",
            i, actual_count, expected_count
        );
    }
}

/// Property test under an active holder cap.
///
/// Sets a cap of 5 holders and performs 40 operations. Operations that would
/// exceed the cap are expected to fail; all others succeed. After each
/// operation the invariant still holds.
#[test]
fn test_holder_count_invariant_with_cap() {
    let (env, _, client) = setup();
    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    client.set_holder_cap(&Some(5));

    let holders: Vec<Address> = (0..8).map(|_| Address::generate(&env)).collect();

    for i in 0..40 {
        let addr_idx = i % holders.len();
        let addr = &holders[addr_idx];
        let other_idx = (i + 1) % holders.len();
        let other = &holders[other_idx];

        match i % 4 {
            0 => {
                // Mint: may fail if it would add a new holder beyond cap
                let _ = client.try_mint(&issuer, addr, &(10 * UNIT));
            }
            1 => {
                let bal = client.balance(addr);
                if bal > 0 {
                    let burn_amount = if bal >= 5 * UNIT { 5 * UNIT } else { bal };
                    client.burn(addr, &burn_amount);
                }
            }
            2 => {
                let bal = client.balance(addr);
                if bal > 0 {
                    let transfer_amount = if bal >= 3 * UNIT { 3 * UNIT } else { bal };
                    // May fail if `other` has zero balance and count is at cap
                    let _ = client.try_transfer(addr, other, &transfer_amount);
                }
            }
            _ => {
                let _ = client.try_mint(&issuer, other, &(7 * UNIT));
            }
        }

        // Invariant holds even under a cap
        let expected_count = count_nonzero_balances(&client, &holders);
        let actual_count = client.holder_count();
        assert_eq!(
            actual_count, expected_count,
            "Holder count mismatch at step {} with cap: actual={}, expected={}",
            i, actual_count, expected_count
        );
    }
}
