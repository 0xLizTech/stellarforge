#![cfg(test)]

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger,
    },
    Address, Env, String,
};

use registry::{AssetEntry, DataKey, RegistryContract, RegistryContractClient, RegistryError};
use stellarforge_common::storage::INSTANCE_BUMP_AMOUNT;

struct Harness<'a> {
    env: Env,
    admin: Address,
    contract_id: Address,
    client: RegistryContractClient<'a>,
}

impl Harness<'_> {
    /// Ledgers remaining before the given persistent entry is archived.
    fn ttl_of(&self, key: &DataKey) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().persistent().get_ttl(key)
        })
    }

    fn instance_ttl(&self) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().instance().get_ttl()
        })
    }

    /// The network's maximum entry TTL, which is what write paths extend to.
    fn max_ttl(&self) -> u32 {
        self.env
            .as_contract(&self.contract_id, || self.env.storage().max_ttl())
    }
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(RegistryContract, (&admin,));
    let client = RegistryContractClient::new(&env, &contract_id);

    Harness {
        env,
        admin,
        contract_id,
        client,
    }
}

fn entry(env: &Env, contract: &Address, asset_class: &str, active: bool) -> AssetEntry {
    AssetEntry {
        contract: contract.clone(),
        asset_class: String::from_str(env, asset_class),
        active,
    }
}

// ─── Initialisation ────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let h = setup();
    assert_eq!(h.client.admin(), h.admin);
}

#[test]
fn test_initialize_starts_with_an_empty_directory() {
    let h = setup();
    assert_eq!(h.client.list_assets().len(), 0);
}

// The double-initialize and uninitialized-admin tests went with the
// `initialize` entry point: a constructor configures the contract inside the
// deploy transaction, so neither state exists to be tested. See the note in
// `contracts/compliance/tests/test_compliance_contract.rs` on why constructor
// auth is not covered either.

// ─── Registration ──────────────────────────────────────────────────────────

#[test]
fn test_register_then_get_asset() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    let stored = h.client.get_asset(&asset).unwrap();
    assert_eq!(stored.contract, asset);
    assert_eq!(stored.asset_class, String::from_str(&h.env, "real_estate"));
    assert!(stored.active);

    assert_eq!(h.client.list_assets().len(), 1);
    assert_eq!(h.client.list_assets().get(0).unwrap(), asset);
}

#[test]
fn test_get_unknown_asset_returns_none() {
    let h = setup();
    assert!(h.client.get_asset(&Address::generate(&h.env)).is_none());
}

#[test]
fn test_registering_distinct_assets_lists_each_once() {
    let h = setup();
    let first = Address::generate(&h.env);
    let second = Address::generate(&h.env);

    h.client
        .register(&entry(&h.env, &first, "real_estate", true));
    h.client
        .register(&entry(&h.env, &second, "commodity", true));

    assert_eq!(h.client.list_assets().len(), 2);
}

#[test]
fn test_reregistering_an_asset_does_not_duplicate_the_directory() {
    let h = setup();
    let asset = Address::generate(&h.env);

    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));
    h.client
        .register(&entry(&h.env, &asset, "infrastructure", true));
    h.client
        .register(&entry(&h.env, &asset, "infrastructure", false));

    // Three registrations, one asset: a consumer iterating the directory must
    // not see the same contract three times.
    let listed = h.client.list_assets();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed.get(0).unwrap(), asset);
}

#[test]
fn test_reregistering_updates_the_stored_entry() {
    let h = setup();
    let asset = Address::generate(&h.env);

    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));
    h.client
        .register(&entry(&h.env, &asset, "infrastructure", false));

    let stored = h.client.get_asset(&asset).unwrap();
    assert_eq!(
        stored.asset_class,
        String::from_str(&h.env, "infrastructure")
    );
    assert!(!stored.active);
}

// ─── Activation ────────────────────────────────────────────────────────────

#[test]
fn test_set_active_toggles_the_entry() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    h.client.set_active(&asset, &false);
    assert!(!h.client.get_asset(&asset).unwrap().active);

    h.client.set_active(&asset, &true);
    assert!(h.client.get_asset(&asset).unwrap().active);
}

#[test]
fn test_set_active_on_unknown_asset_is_rejected() {
    let h = setup();
    let res = h.client.try_set_active(&Address::generate(&h.env), &false);
    assert_eq!(res, Err(Ok(RegistryError::AssetNotFound.into())));
}

#[test]
fn test_set_active_does_not_touch_the_directory() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    h.client.set_active(&asset, &false);
    assert_eq!(h.client.list_assets().len(), 1);
}

// ─── Storage lifetime ──────────────────────────────────────────────────────

// Since protocol 23 the test environment auto-restores archived entries, so
// advancing the ledger past an expiry proves nothing. These tests assert the
// TTL itself, which is what the extension is for.

/// Longer than PERSISTENT_REFRESH_INTERVAL, so a write path actually reissues
/// the extension rather than finding the entry still fresh enough.
const IDLE: u32 = 600_000;

#[test]
fn test_register_extends_to_the_network_maximum() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    let max = h.max_ttl();
    assert_eq!(h.ttl_of(&DataKey::Asset(asset)), max);
    assert_eq!(h.ttl_of(&DataKey::AssetList), max);
    assert_eq!(h.instance_ttl(), INSTANCE_BUMP_AMOUNT);
}

#[test]
fn test_reads_do_not_extend_the_entries_they_touch() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    let expected = h.max_ttl() - IDLE;
    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);

    // Lookups are pure queries. Extending here would charge every reader for
    // a ledger write; the network-maximum bump on the write path is what keeps
    // the entry alive instead.
    h.client.get_asset(&asset);
    h.client.list_assets();

    assert_eq!(h.ttl_of(&DataKey::Asset(asset)), expected);
    assert_eq!(h.ttl_of(&DataKey::AssetList), expected);
}

#[test]
fn test_set_active_extends_the_entry() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);
    h.client.set_active(&asset, &false);

    assert_eq!(h.ttl_of(&DataKey::Asset(asset)), h.max_ttl());
    assert_eq!(h.instance_ttl(), INSTANCE_BUMP_AMOUNT);
}
