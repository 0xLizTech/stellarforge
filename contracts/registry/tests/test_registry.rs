#![cfg(test)]

use soroban_sdk::{
    testutils::{
        cost_estimate::NetworkInvocationResourceLimits,
        storage::{Instance as _, Persistent as _},
        Address as _, Events, Ledger,
    },
    Address, Env, Event, String,
};

use registry::{
    ActiveSet, AssetEntry, AssetRegistered, DataKey, RegistryContract, RegistryContractClient,
    RegistryError, MAX_PAGE_SIZE,
};
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
    assert_eq!(h.client.list_assets(&0, &MAX_PAGE_SIZE).len(), 0);
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

    assert_eq!(h.client.list_assets(&0, &MAX_PAGE_SIZE).len(), 1);
    assert_eq!(
        h.client.list_assets(&0, &MAX_PAGE_SIZE).get(0).unwrap(),
        asset
    );
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

    assert_eq!(h.client.list_assets(&0, &MAX_PAGE_SIZE).len(), 2);
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
    let listed = h.client.list_assets(&0, &MAX_PAGE_SIZE);
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
    assert_eq!(h.client.list_assets(&0, &MAX_PAGE_SIZE).len(), 1);
}

// ─── Directory paging ──────────────────────────────────────────────────────

fn register_many(h: &Harness, n: u32) -> std::vec::Vec<Address> {
    (0..n)
        .map(|_| {
            let asset = Address::generate(&h.env);
            h.client
                .register(&entry(&h.env, &asset, "real_estate", true));
            asset
        })
        .collect()
}

#[test]
fn test_asset_count_counts_distinct_assets() {
    let h = setup();
    assert_eq!(h.client.asset_count(), 0);

    let assets = register_many(&h, 3);
    h.client
        .register(&entry(&h.env, &assets[0], "commodity", false));

    assert_eq!(h.client.asset_count(), 3);
}

#[test]
fn test_pages_partition_the_directory_in_registration_order() {
    let h = setup();
    let assets = register_many(&h, 7);

    let mut walked = std::vec::Vec::new();
    let mut start = 0;
    loop {
        let page = h.client.list_assets(&start, &3);
        walked.extend(page.iter());
        if page.len() < 3 {
            break;
        }
        start += 3;
    }

    assert_eq!(walked, assets);
}

#[test]
fn test_a_page_past_the_end_is_empty() {
    let h = setup();
    register_many(&h, 2);

    assert_eq!(h.client.list_assets(&2, &MAX_PAGE_SIZE).len(), 0);
    assert_eq!(h.client.list_assets(&u32::MAX, &MAX_PAGE_SIZE).len(), 0);
}

#[test]
fn test_an_oversized_page_is_rejected() {
    let h = setup();
    let res = h.client.try_list_assets(&0, &(MAX_PAGE_SIZE + 1));
    assert_eq!(res, Err(Ok(RegistryError::PageTooLarge.into())));
}

/// Writes `prefilled` directory entries straight to storage, then registers
/// and pages through the contract with mainnet per-invocation limits enforced.
///
/// The prefill bypasses `register` because the test host's cost grows with its
/// whole in-memory ledger, and it runs as one frame, so limits are lifted for
/// that setup alone and reinstated before the calls under test.
fn assert_directory_accepts_and_pages_after(prefilled: u32) {
    let h = setup();

    h.env.cost_estimate().disable_resource_limits();
    h.env.cost_estimate().budget().reset_unlimited();
    h.env.as_contract(&h.contract_id, || {
        for index in 0..prefilled {
            h.env
                .storage()
                .persistent()
                .set(&DataKey::AssetAt(index), &Address::generate(&h.env));
        }
        h.env
            .storage()
            .persistent()
            .set(&DataKey::AssetCount, &prefilled);
    });
    h.env.cost_estimate().budget().reset_default();
    h.env
        .cost_estimate()
        .enforce_resource_limits(NetworkInvocationResourceLimits::mainnet());

    let newest = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &newest, "real_estate", true));

    assert_eq!(h.client.asset_count(), prefilled + 1);
    let last_page = h
        .client
        .list_assets(&(prefilled + 1 - MAX_PAGE_SIZE), &MAX_PAGE_SIZE);
    assert_eq!(last_page.len(), MAX_PAGE_SIZE);
    assert_eq!(last_page.get(MAX_PAGE_SIZE - 1).unwrap(), newest);
}

/// IR-01. The directory used to be one `Vec` in one ledger entry, which
/// crossed the 64 KiB entry limit at around 1,600 assets and then refused
/// every registration for good. 2,000 is past that ceiling.
#[test]
fn test_directory_scales_past_the_former_entry_size_ceiling() {
    assert_directory_accepts_and_pages_after(2_000);
}

/// NFR-P-3 requires 10,000 assets. No entry grows with the directory, so this
/// exercises the same per-call footprint as the test above; it is kept
/// runnable rather than in the default suite because the prefill alone takes
/// minutes in the test host. Run with `cargo test -p registry -- --ignored`.
#[test]
#[ignore = "slow: prefills 10,000 entries; run with --ignored"]
fn test_directory_scales_to_the_nfr_p_3_size() {
    assert_directory_accepts_and_pages_after(10_000);
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
    assert_eq!(h.ttl_of(&DataKey::AssetAt(0)), max);
    assert_eq!(h.ttl_of(&DataKey::AssetCount), max);
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
    h.client.list_assets(&0, &MAX_PAGE_SIZE);

    assert_eq!(h.ttl_of(&DataKey::Asset(asset)), expected);
    assert_eq!(h.ttl_of(&DataKey::AssetAt(0)), expected);
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

// ─── Events (IR-03) ────────────────────────────────────────────────────────

#[test]
fn test_register_emits_event_distinguishing_new_from_updated() {
    let h = setup();
    let asset = Address::generate(&h.env);

    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));
    assert_eq!(
        h.env.events().all(),
        std::vec![AssetRegistered {
            contract: asset.clone(),
            asset_class: String::from_str(&h.env, "real_estate"),
            active: true,
            is_new: true,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );

    h.client
        .register(&entry(&h.env, &asset, "infrastructure", false));
    assert_eq!(
        h.env.events().all(),
        std::vec![AssetRegistered {
            contract: asset,
            asset_class: String::from_str(&h.env, "infrastructure"),
            active: false,
            is_new: false,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_set_active_emits_event() {
    let h = setup();
    let asset = Address::generate(&h.env);
    h.client
        .register(&entry(&h.env, &asset, "real_estate", true));

    h.client.set_active(&asset, &false);

    assert_eq!(
        h.env.events().all(),
        std::vec![ActiveSet {
            contract: asset,
            active: false,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}
