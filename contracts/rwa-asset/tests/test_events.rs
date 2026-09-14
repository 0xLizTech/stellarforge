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
    testutils::{Address as _, Events, MuxedAddress as _},
    Address, Bytes, Env, Event, MuxedAddress, String,
};

use rwa_asset::{
    AdminTransferred, Approve, AssetMetadata, Burn, ComplianceSet, IssuerSet, MetadataUpdated,
    Mint, Paused, RwaAssetContract, RwaAssetContractClient, Transfer, TransferFrom,
};

/// Allowance expiry used throughout; the test ledger starts at sequence 0.
const LIVE_UNTIL: u32 = 1_000;

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
    let id = env.register(RwaAssetContract, (&admin, &metadata(&env)));
    let client = RwaAssetContractClient::new(&env, &id);

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
            to_muxed_id: None,
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
    f.client.approve(&alice, &spender, &500, &LIVE_UNTIL);
    f.client.transfer_from(&spender, &alice, &bob, &300);

    // The spender moved the tokens, but the event records the parties whose
    // balances actually changed.
    assert_eq!(
        f.env.events().all(),
        std::vec![TransferFrom {
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

    f.client.approve(&alice, &spender, &750, &LIVE_UNTIL);

    assert_eq!(
        f.env.events().all(),
        std::vec![Approve {
            owner: alice,
            spender,
            amount: 750,
            live_until_ledger: LIVE_UNTIL,
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

// ─── Administrative events (IR-03) ─────────────────────────────────────────

#[test]
fn test_set_issuer_emits_event() {
    let f = setup();
    let issuer = Address::generate(&f.env);

    f.client.set_issuer(&issuer, &false);

    assert_eq!(
        f.env.events().all(),
        std::vec![IssuerSet {
            issuer,
            approved: false,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_update_metadata_emits_both_versions() {
    let f = setup();
    let mut updated = metadata(&f.env);
    updated.legal_doc_hash = Bytes::from_array(&f.env, &[9u8; 32]);

    f.client.update_metadata(&updated);

    // The previous version rides along so the document hash the asset used to
    // name is recoverable without replaying state.
    assert_eq!(
        f.env.events().all(),
        std::vec![MetadataUpdated {
            previous: metadata(&f.env),
            metadata: updated,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_transfer_admin_emits_event() {
    let f = setup();
    let previous = f.client.admin();
    let new_admin = Address::generate(&f.env);

    f.client.transfer_admin(&new_admin);

    assert_eq!(
        f.env.events().all(),
        std::vec![AdminTransferred {
            previous,
            new_admin,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

/// Disabling screening is the change ADR-004 tells monitors to watch for, so
/// the `None` case is asserted as well as the configured one.
#[test]
fn test_set_compliance_emits_event_including_when_disabled() {
    let f = setup();
    let engine = Address::generate(&f.env);

    f.client.set_compliance(&Some(engine.clone()), &2);
    assert_eq!(
        f.env.events().all(),
        std::vec![ComplianceSet {
            compliance: Some(engine),
            min_level: 2,
        }
        .to_xdr(&f.env, &f.id)],
    );

    f.client.set_compliance(&None, &0);
    assert_eq!(
        f.env.events().all(),
        std::vec![ComplianceSet {
            compliance: None,
            min_level: 0,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

// ─── SEP-41 events (IR-09, IR-15) ──────────────────────────────────────────

#[test]
fn test_burn_from_emits_a_burn_for_the_holder() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let spender = Address::generate(&f.env);
    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.approve(&alice, &spender, &500, &LIVE_UNTIL);

    f.client.burn_from(&spender, &alice, &200);

    assert_eq!(
        f.env.events().all(),
        std::vec![Burn {
            from: alice,
            amount: 200,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

/// IR-15. A self-`transfer_from` spends the allowance but moves no balance,
/// so like a self-`transfer` it must not tell indexers a settlement happened.
#[test]
fn test_self_transfer_from_emits_nothing() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let spender = Address::generate(&f.env);
    f.client.mint(&f.issuer, &alice, &1_000);
    f.client.approve(&alice, &spender, &500, &LIVE_UNTIL);

    f.client.transfer_from(&spender, &alice, &alice, &400);

    assert!(
        f.env.events().all().events().is_empty(),
        "self-transfer_from published an event",
    );
}

/// SEP-41: a muxed recipient's id travels in the event, while the balance is
/// credited to the underlying address.
#[test]
fn test_transfer_to_a_muxed_address_carries_its_id() {
    let f = setup();
    let alice = Address::generate(&f.env);
    let bob = MuxedAddress::new(MuxedAddress::generate(&f.env), 42);
    f.client.mint(&f.issuer, &alice, &1_000);

    f.client.transfer(&alice, &bob, &250);

    assert_eq!(
        f.env.events().all(),
        std::vec![Transfer {
            from: alice,
            to: bob.address(),
            amount: 250,
            to_muxed_id: Some(42),
        }
        .to_xdr(&f.env, &f.id)],
    );
    assert_eq!(f.client.balance(&bob.address()), 250);
}
