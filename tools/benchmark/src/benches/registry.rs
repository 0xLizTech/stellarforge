use registry::{AssetEntry, RegistryContract, RegistryContractClient, MAX_PAGE_SIZE};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String};

use crate::{mainnet_env, measure, Row};

const CONTRACT: &str = "registry";

pub fn run() -> Vec<Row> {
    let mut rows = Vec::new();

    let env = mainnet_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(RegistryContract, (&admin,));
    rows.push(measure(&env, CONTRACT, "__constructor"));
    let client = RegistryContractClient::new(&env, &contract_id);

    let asset = Address::generate(&env);
    client.register(&AssetEntry {
        contract: asset.clone(),
        asset_class: String::from_str(&env, "real_estate"),
        active: true,
    });
    rows.push(measure(&env, CONTRACT, "register"));

    client.get_asset(&asset);
    rows.push(measure(&env, CONTRACT, "get_asset"));

    client.list_assets(&0, &MAX_PAGE_SIZE);
    rows.push(measure(&env, CONTRACT, "list_assets"));

    client.asset_count();
    rows.push(measure(&env, CONTRACT, "asset_count"));

    client.set_active(&asset, &false);
    rows.push(measure(&env, CONTRACT, "set_active"));

    client.admin();
    rows.push(measure(&env, CONTRACT, "admin"));

    client.paused();
    rows.push(measure(&env, CONTRACT, "paused"));

    client.set_paused(&true);
    rows.push(measure(&env, CONTRACT, "set_paused"));
    client.set_paused(&false);

    let new_admin = Address::generate(&env);
    client.propose_admin(&new_admin);
    rows.push(measure(&env, CONTRACT, "propose_admin"));
    client.accept_admin();
    rows.push(measure(&env, CONTRACT, "accept_admin"));

    rows
}
