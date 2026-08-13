#![cfg(test)]

//! Asserts the exact topics and payload of every emitted event.
//!
//! Indexers match on topics, so a silent change to a topic list breaks
//! downstream consumers just as surely as dropping the event entirely. These
//! tests compare against the event's full XDR encoding, pinning the wire
//! format rather than merely checking that something was published.
//!
//! `Events::all()` returns the events of the last contract invocation only, so
//! each assertion sees exactly the events of the call above it.

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Bytes, Env, Event, String,
};

use rwa_asset::{
    Approve, AssetMetadata, Burn, Mint, Paused, RwaAssetContract, RwaAssetContractClient, Transfer,
};

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

struct Fixture<'a> {
    env: Env,
    id: Address,
    client: RwaAssetContractClient<'a>,
    issuer: Address,
}

fn setup() -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let id = env.register(RwaAssetContract, ());
    let client = RwaAssetContractClient::new(&env, &id);
    client.initialize(&admin, &metadata(&env));

    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);

    Fixture {
        env,
        id,
        client,
        issuer,
    }
}

#[test]
fn test_mint_emits_event() {
    let f = setup();
    let alice = Address::generate(&f.env);

    f.client.mint(&f.issuer, &alice, &1_000);

    assert_eq!(
        f.env.events().all(),
        std::vec![Mint {
            issuer: f.issuer.clone(),
            to: alice,
            amount: 1_000,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_transfer_emits_event() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let bob = Address::generate(&f.env);

    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.transfer(&alice, &bob, &250);

    assert_eq!(
        f.env.events().all(),
        std::vec![Transfer {
            from: alice,
            to: bob,
            amount: 250,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_transfer_from_emits_a_transfer_naming_the_counterparties() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let bob = Address::generate(&f.env);
    let spender = Address::generate(&f.env);

    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.approve(&alice, &spender, &500);
    f.client.transfer_from(&spender, &alice, &bob, &300);

    // The spender moved the tokens, but the event records the parties whose
    // balances actually changed.
    assert_eq!(
        f.env.events().all(),
        std::vec![Transfer {
            from: alice,
            to: bob,
            amount: 300,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_burn_emits_event() {
    let f = setup();
    let alice = Address::generate(&f.env);

    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.burn(&alice, &400);

    assert_eq!(
        f.env.events().all(),
        std::vec![Burn {
            from: alice,
            amount: 400,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_approve_emits_event() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let spender = Address::generate(&f.env);

    f.client.approve(&alice, &spender, &750);

    assert_eq!(
        f.env.events().all(),
        std::vec![Approve {
            owner: alice,
            spender,
            amount: 750,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_set_paused_emits_event() {
    let f = setup();

    f.client.set_paused(&true);

    assert_eq!(
        f.env.events().all(),
        std::vec![Paused { paused: true }.to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_self_transfer_no_op_emits_nothing() {
    let f = setup();
    let alice = Address::generate(&f.env);

    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.transfer(&alice, &alice, &400);

    // The no-op path moves no balance, so publishing a Transfer would tell
    // indexers a settlement occurred when none did.
    assert!(
        f.env.events().all().events().is_empty(),
        "self-transfer no-op published an event",
    );
}
