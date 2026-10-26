#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::Address as _,
    Address, Env, String,
};

#[contract]
struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn decimals(_env: Env) -> u32 {
        7
    }
}

fn setup() -> (Env, Address, Address, VaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying = env.register(MockToken, ());

    let config = VaultConfig {
        underlying: underlying.clone(),
        exchange_rate: 1,
        max_share_supply: 1_000_000_000,
        lockup_secs: 86400,
        name: String::from_str(&env, "Test Vault"),
        symbol: String::from_str(&env, "tVAULT"),
        compliance: None,
        min_compliance_level: 0,
    };

    let contract_id = env.register(VaultContract, (&admin, &config));
    let client = VaultContractClient::new(&env, &contract_id);

    (env, admin, contract_id, client)
}

#[test]
fn test_internal_mint_and_burn_helpers() {
    let (env, _admin, contract_id, client) = setup();
    let alice = Address::generate(&env);

    env.as_contract(&contract_id, || {
        VaultContract::mint_shares(&env, &alice, 500);
    });

    assert_eq!(client.balance(&alice), 500);
    assert_eq!(client.total_supply(), 500);

    env.as_contract(&contract_id, || {
        VaultContract::burn_shares(&env, &alice, 200);
    });

    assert_eq!(client.balance(&alice), 300);
    assert_eq!(client.total_supply(), 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_internal_mint_exceeds_max_supply_rejected() {
    let (env, _admin, contract_id, _client) = setup();
    let alice = Address::generate(&env);

    env.as_contract(&contract_id, || {
        VaultContract::mint_shares(&env, &alice, 1_000_000_001);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_internal_burn_insufficient_balance_rejected() {
    let (env, _admin, contract_id, _client) = setup();
    let alice = Address::generate(&env);

    env.as_contract(&contract_id, || {
        VaultContract::mint_shares(&env, &alice, 100);
        VaultContract::burn_shares(&env, &alice, 101);
    });
}

#[test]
fn test_internal_set_underlying_held_and_locked_until() {
    let (env, _admin, contract_id, client) = setup();
    let alice = Address::generate(&env);

    env.as_contract(&contract_id, || {
        VaultContract::set_underlying_held(&env, 750);
        VaultContract::set_locked_until(&env, &alice, 12345);
    });

    assert_eq!(client.underlying_held(), 750);
    assert_eq!(client.locked_until(&alice), 12345);
}
