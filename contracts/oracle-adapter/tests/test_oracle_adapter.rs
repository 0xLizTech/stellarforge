#![cfg(test)]

use soroban_sdk::{
    testutils::{
        cost_estimate::NetworkInvocationResourceLimits, storage::Persistent as _, Address as _,
        Events, Ledger, MockAuth, MockAuthInvoke,
    },
    Address, Env, Event, IntoVal, Symbol, Vec,
};

use oracle_adapter::{
    AdminTransferred, Asset, AssetAdded, AssetConfig, DataKey, MaxDeviationSet,
    OracleAdapterContract, OracleAdapterContractClient, OracleError, PriceData, PriceOverridden,
    PriceReported, ReporterSet, MAX_ASSETS, MAX_DECIMALS, MAX_RECORDS,
};

const DECIMALS: u32 = 14;
const RESOLUTION: u32 = 300;
/// 10%.
const MAX_DEVIATION_BPS: u32 = 1_000;
/// Deliberately not a multiple of `RESOLUTION`, so rounding is visible.
const NOW: u64 = 1_800_000_123;
/// `NOW` rounded down to the resolution.
const NOW_TICK: u64 = 1_800_000_000;
/// 100.00 in the feed's decimals.
const PRICE: i128 = 100 * 10_i128.pow(DECIMALS);

struct Harness<'a> {
    env: Env,
    admin: Address,
    reporter: Address,
    asset: Asset,
    contract_id: Address,
    client: OracleAdapterContractClient<'a>,
}

impl Harness<'_> {
    fn usd(&self) -> Asset {
        Asset::Other(Symbol::new(&self.env, "USD"))
    }

    /// Moves the ledger clock forward by whole ticks.
    fn advance_ticks(&self, ticks: u64) {
        self.env
            .ledger()
            .with_mut(|li| li.timestamp += ticks * u64::from(RESOLUTION));
    }

    /// Reports `price` at the current ledger time.
    fn report_now(&self, price: i128) {
        let now = self.env.ledger().timestamp();
        self.client
            .report(&self.reporter, &self.asset, &price, &now);
    }

    fn ttl_of(&self, key: &DataKey) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().persistent().get_ttl(key)
        })
    }

    fn max_ttl(&self) -> u32 {
        self.env
            .as_contract(&self.contract_id, || self.env.storage().max_ttl())
    }
}

/// A feed quoting in USD, with one listed asset and one reporter for it.
fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = NOW);

    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    let contract_id = env.register(
        OracleAdapterContract,
        (&admin, &base, &DECIMALS, &RESOLUTION),
    );
    let client = OracleAdapterContractClient::new(&env, &contract_id);

    let asset = Asset::Stellar(Address::generate(&env));
    client.add_asset(&asset, &MAX_DEVIATION_BPS);
    let reporter = Address::generate(&env);
    client.set_reporter(&asset, &reporter, &true);

    Harness {
        env,
        admin,
        reporter,
        asset,
        contract_id,
        client,
    }
}

// ─── Constructor and SEP-40 configuration ───────────────────────────────────

#[test]
fn test_constructor_stores_the_sep40_configuration() {
    let h = setup();
    assert_eq!(h.client.admin(), h.admin);
    assert_eq!(h.client.base(), h.usd());
    assert_eq!(h.client.decimals(), DECIMALS);
    assert_eq!(h.client.resolution(), RESOLUTION);
}

#[test]
fn test_constructor_accepts_max_decimals() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    let id = env.register(
        OracleAdapterContract,
        (&admin, &base, &MAX_DECIMALS, &1_u32),
    );
    assert_eq!(
        OracleAdapterContractClient::new(&env, &id).decimals(),
        MAX_DECIMALS
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_constructor_rejects_too_many_decimals() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    env.register(
        OracleAdapterContract,
        (&admin, &base, &(MAX_DECIMALS + 1), &RESOLUTION),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_constructor_rejects_a_zero_resolution() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    env.register(OracleAdapterContract, (&admin, &base, &DECIMALS, &0_u32));
}

// ─── Listing assets ─────────────────────────────────────────────────────────

#[test]
fn test_assets_lists_in_the_order_added() {
    let h = setup();
    let usd = h.usd();
    h.client.add_asset(&usd, &500);

    let expected = Vec::from_array(&h.env, [h.asset.clone(), usd.clone()]);
    assert_eq!(h.client.assets(), expected);
    assert_eq!(
        h.client.asset_config(&usd),
        Some(AssetConfig {
            max_deviation_bps: 500
        })
    );
}

#[test]
fn test_add_asset_refuses_a_listed_asset() {
    let h = setup();
    assert_eq!(
        h.client.try_add_asset(&h.asset, &MAX_DEVIATION_BPS),
        Err(Ok(OracleError::AssetAlreadyExists.into()))
    );
    assert_eq!(h.client.assets().len(), 1);
}

#[test]
fn test_add_asset_refuses_a_zero_deviation_limit() {
    let h = setup();
    let usd = h.usd();
    assert_eq!(
        h.client.try_add_asset(&usd, &0),
        Err(Ok(OracleError::InvalidDeviation.into()))
    );
    assert_eq!(h.client.asset_config(&usd), None);
}

#[test]
fn test_add_asset_stops_at_max_assets() {
    let h = setup();
    for _ in 1..MAX_ASSETS {
        h.client.add_asset(
            &Asset::Stellar(Address::generate(&h.env)),
            &MAX_DEVIATION_BPS,
        );
    }
    assert_eq!(h.client.assets().len(), MAX_ASSETS);

    let one_more = Asset::Stellar(Address::generate(&h.env));
    assert_eq!(
        h.client.try_add_asset(&one_more, &MAX_DEVIATION_BPS),
        Err(Ok(OracleError::TooManyAssets.into()))
    );
}

#[test]
fn test_add_asset_requires_the_admin() {
    let h = setup();
    let usd = h.usd();
    h.client.add_asset(&usd, &500);
    assert_eq!(
        h.env.auths(),
        std::vec![(
            h.admin.clone(),
            soroban_sdk::testutils::AuthorizedInvocation {
                function: soroban_sdk::testutils::AuthorizedFunction::Contract((
                    h.contract_id.clone(),
                    Symbol::new(&h.env, "add_asset"),
                    (usd, 500_u32).into_val(&h.env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}

#[test]
fn test_add_asset_by_anyone_else_is_rejected() {
    let h = setup();
    let usd = h.usd();
    let outsider = Address::generate(&h.env);
    h.env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "add_asset",
            args: (usd.clone(), 500_u32).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);
    assert!(h.client.try_add_asset(&usd, &500).is_err());
}

#[test]
fn test_add_asset_emits_event() {
    let h = setup();
    let usd = h.usd();
    h.client.add_asset(&usd, &500);
    assert_eq!(
        h.env.events().all(),
        std::vec![AssetAdded {
            asset: usd,
            max_deviation_bps: 500,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_set_max_deviation_replaces_the_limit() {
    let h = setup();
    h.client.set_max_deviation(&h.asset, &2_500);
    // Events first: `events().all()` holds only the latest invocation's, and
    // any read in between replaces them.
    assert_eq!(
        h.env.events().all(),
        std::vec![MaxDeviationSet {
            asset: h.asset.clone(),
            max_deviation_bps: 2_500,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert_eq!(
        h.client.asset_config(&h.asset),
        Some(AssetConfig {
            max_deviation_bps: 2_500
        })
    );
}

#[test]
fn test_set_max_deviation_refuses_zero_and_unknown_assets() {
    let h = setup();
    assert_eq!(
        h.client.try_set_max_deviation(&h.asset, &0),
        Err(Ok(OracleError::InvalidDeviation.into()))
    );
    assert_eq!(
        h.client.try_set_max_deviation(&h.usd(), &500),
        Err(Ok(OracleError::UnknownAsset.into()))
    );
}

// ─── Reporters ──────────────────────────────────────────────────────────────

#[test]
fn test_set_reporter_grants_and_revokes() {
    let h = setup();
    assert!(h.client.is_reporter(&h.asset, &h.reporter));

    h.client.set_reporter(&h.asset, &h.reporter, &false);
    assert_eq!(
        h.env.events().all(),
        std::vec![ReporterSet {
            asset: h.asset.clone(),
            reporter: h.reporter.clone(),
            allowed: false,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert!(!h.client.is_reporter(&h.asset, &h.reporter));

    assert_eq!(
        h.client.try_report(&h.reporter, &h.asset, &PRICE, &NOW),
        Err(Ok(OracleError::NotReporter.into()))
    );
}

#[test]
fn test_set_reporter_refuses_an_unknown_asset() {
    let h = setup();
    assert_eq!(
        h.client.try_set_reporter(&h.usd(), &h.reporter, &true),
        Err(Ok(OracleError::UnknownAsset.into()))
    );
}

#[test]
fn test_an_unauthorized_address_cannot_report() {
    let h = setup();
    let stranger = Address::generate(&h.env);
    assert_eq!(
        h.client.try_report(&stranger, &h.asset, &PRICE, &NOW),
        Err(Ok(OracleError::NotReporter.into()))
    );
    assert_eq!(h.client.lastprice(&h.asset), None);
}

#[test]
fn test_a_reporter_for_one_asset_cannot_report_another() {
    let h = setup();
    let other = Asset::Stellar(Address::generate(&h.env));
    h.client.add_asset(&other, &MAX_DEVIATION_BPS);

    assert_eq!(
        h.client.try_report(&h.reporter, &other, &PRICE, &NOW),
        Err(Ok(OracleError::NotReporter.into()))
    );
}

#[test]
fn test_report_requires_the_reporter_to_authorize() {
    let h = setup();
    h.report_now(PRICE);
    assert_eq!(
        h.env.auths(),
        std::vec![(
            h.reporter.clone(),
            soroban_sdk::testutils::AuthorizedInvocation {
                function: soroban_sdk::testutils::AuthorizedFunction::Contract((
                    h.contract_id.clone(),
                    Symbol::new(&h.env, "report"),
                    (h.reporter.clone(), h.asset.clone(), PRICE, NOW).into_val(&h.env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}

#[test]
fn test_report_refuses_an_unknown_asset() {
    let h = setup();
    assert_eq!(
        h.client.try_report(&h.reporter, &h.usd(), &PRICE, &NOW),
        Err(Ok(OracleError::UnknownAsset.into()))
    );
}

// ─── Recording prices ───────────────────────────────────────────────────────

#[test]
fn test_report_records_the_price_at_its_tick() {
    let h = setup();
    h.report_now(PRICE);

    let expected = PriceData {
        price: PRICE,
        timestamp: NOW_TICK,
    };
    assert_eq!(h.client.lastprice(&h.asset), Some(expected.clone()));
    // Any timestamp inside the tick finds it; the neighbouring ticks do not.
    assert_eq!(h.client.price(&h.asset, &NOW_TICK), Some(expected.clone()));
    assert_eq!(
        h.client
            .price(&h.asset, &(NOW_TICK + u64::from(RESOLUTION) - 1)),
        Some(expected)
    );
    assert_eq!(
        h.client
            .price(&h.asset, &(NOW_TICK + u64::from(RESOLUTION))),
        None
    );
    assert_eq!(h.client.price(&h.asset, &(NOW_TICK - 1)), None);
}

#[test]
fn test_report_emits_event() {
    let h = setup();
    h.report_now(PRICE);
    h.advance_ticks(1);
    h.report_now(PRICE + 1);

    assert_eq!(
        h.env.events().all(),
        std::vec![PriceReported {
            asset: h.asset.clone(),
            reporter: h.reporter.clone(),
            price: PRICE + 1,
            timestamp: NOW_TICK + u64::from(RESOLUTION),
            previous_price: Some(PRICE),
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_reads_of_an_unlisted_or_unreported_asset_return_none() {
    let h = setup();
    let usd = h.usd();
    for asset in [h.asset.clone(), usd] {
        assert_eq!(h.client.lastprice(&asset), None);
        assert_eq!(h.client.price(&asset, &NOW), None);
        assert_eq!(h.client.prices(&asset, &5), None);
    }
}

#[test]
fn test_report_refuses_a_non_positive_price() {
    let h = setup();
    let expected = Err(Ok(OracleError::InvalidPrice.into()));
    assert_eq!(
        h.client.try_report(&h.reporter, &h.asset, &0, &NOW),
        expected
    );
    assert_eq!(
        h.client.try_report(&h.reporter, &h.asset, &-PRICE, &NOW),
        expected
    );
}

#[test]
fn test_report_refuses_a_timestamp_after_the_ledger() {
    let h = setup();
    assert_eq!(
        h.client
            .try_report(&h.reporter, &h.asset, &PRICE, &(NOW + 1)),
        Err(Ok(OracleError::TimestampInFuture.into()))
    );
}

#[test]
fn test_history_cannot_be_rewritten() {
    let h = setup();
    h.report_now(PRICE);
    let expected = Err(Ok(OracleError::StaleReport.into()));

    // The same tick, even at a later second inside it.
    h.env
        .ledger()
        .with_mut(|li| li.timestamp = NOW_TICK + u64::from(RESOLUTION) - 1);
    let later_same_tick = h.env.ledger().timestamp();
    assert_eq!(
        h.client
            .try_report(&h.reporter, &h.asset, &PRICE, &later_same_tick),
        expected
    );

    // An earlier tick, after time has moved on.
    h.advance_ticks(5);
    assert_eq!(
        h.client
            .try_report(&h.reporter, &h.asset, &PRICE, &(NOW_TICK - 1)),
        expected
    );
    assert_eq!(h.client.prices(&h.asset, &MAX_RECORDS).unwrap().len(), 1);
}

#[test]
fn test_a_move_within_the_deviation_limit_is_recorded() {
    let h = setup();
    h.report_now(PRICE);

    // Exactly 10% up, then exactly 10% down from there.
    let up = PRICE + PRICE / 10;
    h.advance_ticks(1);
    h.report_now(up);
    let down = up - up / 10;
    h.advance_ticks(1);
    h.report_now(down);

    assert_eq!(h.client.lastprice(&h.asset).unwrap().price, down);
}

#[test]
fn test_a_move_beyond_the_deviation_limit_is_refused() {
    let h = setup();
    h.report_now(PRICE);
    h.advance_ticks(1);
    let now = h.env.ledger().timestamp();
    let expected = Err(Ok(OracleError::DeviationTooLarge.into()));

    assert_eq!(
        h.client
            .try_report(&h.reporter, &h.asset, &(PRICE + PRICE / 10 + 1), &now),
        expected
    );
    assert_eq!(
        h.client
            .try_report(&h.reporter, &h.asset, &(PRICE - PRICE / 10 - 1), &now),
        expected
    );
    assert_eq!(h.client.lastprice(&h.asset).unwrap().price, PRICE);
}

#[test]
fn test_the_first_report_is_not_deviation_checked() {
    let h = setup();
    h.report_now(1);
    assert_eq!(h.client.lastprice(&h.asset).unwrap().price, 1);
}

#[test]
fn test_a_price_too_large_to_check_is_refused_with_overflow() {
    let h = setup();
    h.report_now(PRICE);
    h.advance_ticks(1);
    let now = h.env.ledger().timestamp();
    assert_eq!(
        h.client.try_report(&h.reporter, &h.asset, &i128::MAX, &now),
        Err(Ok(OracleError::Overflow.into()))
    );
}

// ─── Override ───────────────────────────────────────────────────────────────

#[test]
fn test_override_records_a_move_the_limit_refuses() {
    let h = setup();
    h.report_now(PRICE);
    h.advance_ticks(1);
    let now = h.env.ledger().timestamp();

    h.client.override_price(&h.asset, &(PRICE * 3), &now);

    assert_eq!(
        h.env.events().all(),
        std::vec![PriceOverridden {
            asset: h.asset.clone(),
            price: PRICE * 3,
            timestamp: NOW_TICK + u64::from(RESOLUTION),
            previous_price: Some(PRICE),
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert_eq!(
        h.client.lastprice(&h.asset),
        Some(PriceData {
            price: PRICE * 3,
            timestamp: NOW_TICK + u64::from(RESOLUTION),
        })
    );

    // Reporters continue from the overridden price.
    h.advance_ticks(1);
    h.report_now(PRICE * 3 + PRICE / 10);
}

#[test]
fn test_override_keeps_every_other_rule() {
    let h = setup();
    h.report_now(PRICE);

    assert_eq!(
        h.client.try_override_price(&h.asset, &PRICE, &NOW),
        Err(Ok(OracleError::StaleReport.into()))
    );
    h.advance_ticks(1);
    let now = h.env.ledger().timestamp();
    assert_eq!(
        h.client.try_override_price(&h.asset, &0, &now),
        Err(Ok(OracleError::InvalidPrice.into()))
    );
    assert_eq!(
        h.client.try_override_price(&h.asset, &PRICE, &(now + 1)),
        Err(Ok(OracleError::TimestampInFuture.into()))
    );
    assert_eq!(
        h.client.try_override_price(&h.usd(), &PRICE, &now),
        Err(Ok(OracleError::UnknownAsset.into()))
    );
}

#[test]
fn test_override_by_a_reporter_is_rejected() {
    let h = setup();
    h.report_now(PRICE);
    h.advance_ticks(1);
    let now = h.env.ledger().timestamp();

    h.env.mock_auths(&[MockAuth {
        address: &h.reporter,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "override_price",
            args: (h.asset.clone(), PRICE * 3, now).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);
    assert!(h
        .client
        .try_override_price(&h.asset, &(PRICE * 3), &now)
        .is_err());
    assert_eq!(h.client.lastprice(&h.asset).unwrap().price, PRICE);
}

// ─── History ────────────────────────────────────────────────────────────────

#[test]
fn test_prices_walks_back_newest_first_across_gaps() {
    let h = setup();
    let mut expected = std::vec::Vec::new();
    for i in 0..5_i128 {
        h.report_now(PRICE + i);
        expected.push(PriceData {
            price: PRICE + i,
            timestamp: h.env.ledger().timestamp()
                - h.env.ledger().timestamp() % u64::from(RESOLUTION),
        });
        // Uneven gaps: unreported ticks are skipped, not counted.
        h.advance_ticks(1 + i as u64 * 7);
    }
    expected.reverse();

    let all = h.client.prices(&h.asset, &10).unwrap();
    assert_eq!(all.len(), 5);
    for (i, price) in expected.iter().enumerate() {
        assert_eq!(all.get(i as u32).unwrap(), *price);
    }

    let two = h.client.prices(&h.asset, &2).unwrap();
    assert_eq!(two.len(), 2);
    assert_eq!(two.get(0).unwrap(), expected[0]);
    assert_eq!(two.get(1).unwrap(), expected[1]);

    assert_eq!(h.client.prices(&h.asset, &0), None);
}

#[test]
fn test_prices_is_clamped_to_max_records_under_mainnet_limits() {
    let h = setup();
    for i in 0..(MAX_RECORDS + 5) {
        h.report_now(PRICE + i128::from(i));
        h.advance_ticks(1);
    }

    h.env
        .cost_estimate()
        .enforce_resource_limits(NetworkInvocationResourceLimits::mainnet());
    let records = h.client.prices(&h.asset, &u32::MAX).unwrap();

    assert_eq!(records.len(), MAX_RECORDS);
    assert_eq!(
        records.get(0).unwrap().price,
        PRICE + i128::from(MAX_RECORDS + 4)
    );
}

// ─── Storage lifetime ───────────────────────────────────────────────────────

#[test]
fn test_report_extends_the_entries_it_writes_to_the_network_maximum() {
    let h = setup();
    h.report_now(PRICE);

    let max = h.max_ttl();
    assert_eq!(h.ttl_of(&DataKey::Latest(h.asset.clone())), max);
    assert_eq!(h.ttl_of(&DataKey::Report(h.asset.clone(), NOW_TICK)), max);
    assert_eq!(
        h.ttl_of(&DataKey::Reporter(h.asset.clone(), h.reporter.clone())),
        max
    );
    assert_eq!(h.ttl_of(&DataKey::Config(h.asset.clone())), max);
}

// ─── Admin handover ─────────────────────────────────────────────────────────

#[test]
fn test_transfer_admin_moves_the_role() {
    let h = setup();
    let new_admin = Address::generate(&h.env);

    h.client.transfer_admin(&new_admin);

    assert_eq!(
        h.env.events().all(),
        std::vec![AdminTransferred {
            previous: h.admin.clone(),
            new_admin: new_admin.clone(),
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
    assert_eq!(h.client.admin(), new_admin);
}

#[test]
fn test_transfer_admin_without_the_incoming_signature_is_rejected() {
    let h = setup();
    let new_admin = Address::generate(&h.env);

    h.env.mock_auths(&[MockAuth {
        address: &h.admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "transfer_admin",
            args: (new_admin.clone(),).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h.client.try_transfer_admin(&new_admin).is_err());
    assert_eq!(h.client.admin(), h.admin);
}
