#![cfg(test)]

use contracts_vault::{
    Checkpoint, DataKey, Sep40OracleInterface, UnderlyingTokenClient, UnderlyingTokenInterface,
    VaultConfig, VaultContract, VaultContractClient, VaultError,
};
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    Address, Env, MuxedAddress, String, Vec,
};

#[contract]
struct MockUnderlying;

#[contractimpl]
impl MockUnderlying {
    pub fn decimals(_env: Env) -> u32 {
        7
    }
    pub fn balance(_env: Env, _id: Address) -> i128 {
        1_000_000_000_0000000
    }
    pub fn transfer(_env: Env, _from: Address, _to: MuxedAddress, _amount: i128) {}
}

fn setup_vault<'a>(env: &'a Env) -> (VaultContractClient<'a>, Address, Address) {
    let admin = Address::generate(env);
    let underlying_id = env.register(MockUnderlying, ());

    let vault_id = env.register(
        VaultContract,
        (
            admin.clone(),
            VaultConfig {
                underlying: underlying_id.clone(),
                exchange_rate: 10,
                max_share_supply: 0,
                lockup_secs: 0,
                name: String::from_str(env, "Vault Share"),
                symbol: String::from_str(env, "vSHARE"),
                compliance: None,
                min_compliance_level: 0,
            },
        ),
    );

    let client = VaultContractClient::new(env, &vault_id);
    (client, admin, underlying_id)
}

#[test]
fn test_checkpoint_coalescing_and_boundaries() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let (client, _admin, _underlying) = setup_vault(&env);
    let holder1 = Address::generate(&env);
    let holder2 = Address::generate(&env);

    // Ledger 100: deposit 10 underlying -> 100 shares for holder1
    client.deposit(&holder1, &10);
    assert_eq!(client.balance(&holder1), 100);
    assert_eq!(client.checkpoint_count(&holder1), 1);
    assert_eq!(client.supply_checkpoint_count(), 1);

    // Same ledger (100): another deposit -> 50 more shares (total 150)
    // Coalescing should NOT increment checkpoint count
    client.deposit(&holder1, &5);
    assert_eq!(client.balance(&holder1), 150);
    assert_eq!(client.checkpoint_count(&holder1), 1);
    assert_eq!(client.supply_checkpoint_count(), 1);

    // Advance to ledger 105
    env.ledger().set_sequence_number(105);

    // Current ledger (105) is not closed yet -> balance_at(105) must fail with LedgerNotClosed
    let err = client.try_balance_at(&holder1, &105).unwrap_err().unwrap();
    assert_eq!(err, VaultError::LedgerNotClosed);

    // Future ledger (106) must also fail
    let err_fut = client.try_balance_at(&holder1, &106).unwrap_err().unwrap();
    assert_eq!(err_fut, VaultError::LedgerNotClosed);

    // Boundary before first checkpoint (ledger 99) -> returns 0
    assert_eq!(client.balance_at(&holder1, &99), 0);
    assert_eq!(client.total_supply_at(&99), 0);

    // Ledger 100 closed -> balance is 150, supply is 150
    assert_eq!(client.balance_at(&holder1, &100), 150);
    assert_eq!(client.total_supply_at(&100), 150);

    // Ledgers 101..104 (no changes) -> balance remains 150
    assert_eq!(client.balance_at(&holder1, &101), 150);
    assert_eq!(client.balance_at(&holder1, &104), 150);
    assert_eq!(client.total_supply_at(&104), 150);

    // In ledger 105: transfer 50 shares from holder1 to holder2
    client.transfer(&holder1, &MuxedAddress::from(holder2.clone()), &50);
    assert_eq!(client.balance(&holder1), 100);
    assert_eq!(client.balance(&holder2), 50);
    assert_eq!(client.checkpoint_count(&holder1), 2);
    assert_eq!(client.checkpoint_count(&holder2), 1);
    // Supply did not change, so supply checkpoint count is still 1
    assert_eq!(client.supply_checkpoint_count(), 1);

    // Advance to ledger 110
    env.ledger().set_sequence_number(110);

    // Query ledger 105 (now closed)
    assert_eq!(client.balance_at(&holder1, &105), 100);
    assert_eq!(client.balance_at(&holder2, &105), 50);
    assert_eq!(client.total_supply_at(&105), 150);

    // Address that never held shares returns 0
    let random_user = Address::generate(&env);
    assert_eq!(client.balance_at(&random_user, &105), 0);
}

#[test]
fn test_checkpoints_redeem_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(200);

    let (client, _admin, _underlying) = setup_vault(&env);
    let owner = Address::generate(&env);
    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Ledger 200: Deposit 20 -> 200 shares
    client.deposit(&owner, &20);
    assert_eq!(client.balance(&owner), 200);

    // Approve spender
    client.approve(&owner, &spender, &100, &500);

    // Ledger 210: transfer_from 60 shares
    env.ledger().set_sequence_number(210);
    client.transfer_from(&spender, &owner, &recipient, &60);
    assert_eq!(client.balance(&owner), 140);
    assert_eq!(client.balance(&recipient), 60);

    // Ledger 220: Redeem 40 shares (4 underlying)
    env.ledger().set_sequence_number(220);
    client.redeem(&owner, &40);
    assert_eq!(client.balance(&owner), 100);
    assert_eq!(client.total_supply(), 160);

    // Advance to ledger 230 to close ledger 220
    env.ledger().set_sequence_number(230);

    // Historical checks
    assert_eq!(client.balance_at(&owner, &199), 0);
    assert_eq!(client.balance_at(&owner, &200), 200);
    assert_eq!(client.balance_at(&owner, &209), 200);
    assert_eq!(client.balance_at(&owner, &210), 140);
    assert_eq!(client.balance_at(&owner, &219), 140);
    assert_eq!(client.balance_at(&owner, &220), 100);

    assert_eq!(client.total_supply_at(&200), 200);
    assert_eq!(client.total_supply_at(&219), 200);
    assert_eq!(client.total_supply_at(&220), 160);
}

#[test]
fn test_checkpoint_binary_search_scale() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(20_000);

    let (client, _admin, _underlying) = setup_vault(&env);
    let holder = Address::generate(&env);

    // Simulate 50 sequential checkpoints across ledgers 100..150
    for i in 0..50u32 {
        let seq = 100 + i;
        env.ledger().set_sequence_number(seq);
        client.deposit(&holder, &1); // +10 shares each step
    }

    assert_eq!(client.checkpoint_count(&holder), 50);
    assert_eq!(client.supply_checkpoint_count(), 50);

    // Close ledger 149 by advancing to 10_000
    env.ledger().set_sequence_number(10_000);

    // Test first checkpoint
    assert_eq!(client.balance_at(&holder, &100), 10);
    assert_eq!(client.total_supply_at(&100), 10);

    // Test middle checkpoint
    assert_eq!(client.balance_at(&holder, &125), 260); // 26 steps of 10
    assert_eq!(client.total_supply_at(&125), 260);

    // Test last checkpoint
    assert_eq!(client.balance_at(&holder, &149), 500);
    assert_eq!(client.total_supply_at(&149), 500);

    // Test arbitrary ledger between 149 and 10_000
    assert_eq!(client.balance_at(&holder, &5000), 500);
    assert_eq!(client.total_supply_at(&5000), 500);
}
