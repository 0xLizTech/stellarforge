#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_yield_distributor_scaffold_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(YieldDistributorContract, (&admin,));
    let client = YieldDistributorContractClient::new(&env, &contract_id);

    assert_eq!(client.admin(), admin);
    assert!(!client.paused());
    assert_eq!(client.distribution_count(), 0);

    client.set_paused(&true);
    assert!(client.paused());
}
