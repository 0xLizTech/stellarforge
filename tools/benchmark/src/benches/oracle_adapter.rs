use oracle_adapter::{Asset, OracleAdapterContract, OracleAdapterContractClient};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Symbol};

use crate::{mainnet_env, measure, Row};

const CONTRACT: &str = "oracle-adapter";
const DECIMALS: u32 = 14;
const RESOLUTION: u32 = 300;
const MAX_DEVIATION_BPS: u32 = 1_000;
const NOW: u64 = 1_800_000_000;
const PRICE: i128 = 100 * 10_i128.pow(DECIMALS);

pub fn run() -> Vec<Row> {
    let mut rows = Vec::new();

    let env = mainnet_env();
    env.ledger().with_mut(|li| li.timestamp = NOW);
    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    let contract_id = env.register(
        OracleAdapterContract,
        (&admin, &base, &DECIMALS, &RESOLUTION),
    );
    rows.push(measure(&env, CONTRACT, "__constructor"));
    let client = OracleAdapterContractClient::new(&env, &contract_id);

    let asset = Asset::Stellar(Address::generate(&env));
    client.add_asset(&asset, &MAX_DEVIATION_BPS);
    rows.push(measure(&env, CONTRACT, "add_asset"));

    let reporter = Address::generate(&env);
    client.set_reporter(&asset, &reporter, &true);
    rows.push(measure(&env, CONTRACT, "set_reporter"));

    client.report(&reporter, &asset, &PRICE, &NOW);
    rows.push(measure(&env, CONTRACT, "report"));

    client.base();
    rows.push(measure(&env, CONTRACT, "base"));

    client.assets();
    rows.push(measure(&env, CONTRACT, "assets"));

    client.decimals();
    rows.push(measure(&env, CONTRACT, "decimals"));

    client.resolution();
    rows.push(measure(&env, CONTRACT, "resolution"));

    client.price(&asset, &NOW);
    rows.push(measure(&env, CONTRACT, "price"));

    client.prices(&asset, &1);
    rows.push(measure(&env, CONTRACT, "prices"));

    client.lastprice(&asset);
    rows.push(measure(&env, CONTRACT, "lastprice"));

    client.admin();
    rows.push(measure(&env, CONTRACT, "admin"));

    client.is_reporter(&asset, &reporter);
    rows.push(measure(&env, CONTRACT, "is_reporter"));

    client.asset_config(&asset);
    rows.push(measure(&env, CONTRACT, "asset_config"));

    client.paused();
    rows.push(measure(&env, CONTRACT, "paused"));

    client.set_paused(&true);
    rows.push(measure(&env, CONTRACT, "set_paused"));
    client.set_paused(&false);

    // A move beyond MAX_DEVIATION_BPS, past the next tick, so override_price
    // is measured exercising the path the deviation limit itself refuses.
    env.ledger()
        .with_mut(|li| li.timestamp += u64::from(RESOLUTION));
    let override_timestamp = env.ledger().timestamp();
    client.override_price(&asset, &(PRICE * 10), &override_timestamp);
    rows.push(measure(&env, CONTRACT, "override_price"));

    client.set_max_deviation(&asset, &500);
    rows.push(measure(&env, CONTRACT, "set_max_deviation"));

    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
    rows.push(measure(&env, CONTRACT, "transfer_admin"));

    rows
}
