use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env, String};

use crate::{mainnet_env, measure, Row};

const CONTRACT: &str = "rwa-asset";
const UNIT: i128 = 10_000_000;
const MAX_SUPPLY: i128 = 1_000_000 * UNIT;
const LIVE_UNTIL: u32 = 1_000;

fn metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "NYC Real Estate Fund I"),
        symbol: String::from_str(env, "REIT-NYC-001"),
        decimals: 7,
        asset_class: String::from_str(env, "real_estate"),
        legal_doc_hash: Bytes::from_array(env, &[0u8; 32]),
        max_supply: MAX_SUPPLY,
    }
}

pub fn run() -> Vec<Row> {
    let mut rows = Vec::new();

    let env = mainnet_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(RwaAssetContract, (&admin, &metadata(&env)));
    rows.push(measure(&env, CONTRACT, "__constructor"));
    let client = RwaAssetContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    client.set_issuer(&issuer, &true);
    rows.push(measure(&env, CONTRACT, "set_issuer"));

    client.is_issuer(&issuer);
    rows.push(measure(&env, CONTRACT, "is_issuer"));

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.mint(&issuer, &alice, &(500 * UNIT));
    rows.push(measure(&env, CONTRACT, "mint"));

    client.balance(&alice);
    rows.push(measure(&env, CONTRACT, "balance"));

    client.transfer(&alice, &bob, &(100 * UNIT));
    rows.push(measure(&env, CONTRACT, "transfer"));

    client.approve(&alice, &bob, &(50 * UNIT), &LIVE_UNTIL);
    rows.push(measure(&env, CONTRACT, "approve"));

    client.allowance(&alice, &bob);
    rows.push(measure(&env, CONTRACT, "allowance"));

    client.transfer_from(&bob, &alice, &bob, &(10 * UNIT));
    rows.push(measure(&env, CONTRACT, "transfer_from"));

    client.burn(&alice, &(10 * UNIT));
    rows.push(measure(&env, CONTRACT, "burn"));

    client.approve(&alice, &bob, &(10 * UNIT), &LIVE_UNTIL);
    client.burn_from(&bob, &alice, &(5 * UNIT));
    rows.push(measure(&env, CONTRACT, "burn_from"));

    client.total_supply();
    rows.push(measure(&env, CONTRACT, "total_supply"));

    client.metadata();
    rows.push(measure(&env, CONTRACT, "metadata"));

    client.decimals();
    rows.push(measure(&env, CONTRACT, "decimals"));

    client.name();
    rows.push(measure(&env, CONTRACT, "name"));

    client.symbol();
    rows.push(measure(&env, CONTRACT, "symbol"));

    client.admin();
    rows.push(measure(&env, CONTRACT, "admin"));

    client.paused();
    rows.push(measure(&env, CONTRACT, "paused"));

    client.set_paused(&true);
    rows.push(measure(&env, CONTRACT, "set_paused"));
    client.set_paused(&false);

    client.update_metadata(&metadata(&env));
    rows.push(measure(&env, CONTRACT, "update_metadata"));

    let compliance = Address::generate(&env);
    client.set_compliance(&Some(compliance.clone()), &1);
    rows.push(measure(&env, CONTRACT, "set_compliance"));

    client.compliance_contract();
    rows.push(measure(&env, CONTRACT, "compliance_contract"));

    client.min_compliance_level();
    rows.push(measure(&env, CONTRACT, "min_compliance_level"));

    // Clear compliance before transferring again, since the benchmark's
    // compliance address above is not a real deployed contract and would
    // make every subsequent transfer fail on the cross-contract `screen`
    // call rather than measuring the intended entry point.
    client.set_compliance(&None, &0);

    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
    rows.push(measure(&env, CONTRACT, "transfer_admin"));

    rows
}
