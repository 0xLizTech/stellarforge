#![cfg(test)]

use contracts_vault::{
    Sep40OracleInterface, UnderlyingTokenClient, UnderlyingTokenInterface, VaultConfig,
    VaultContract, VaultContractClient, VaultError,
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

/// Pure shadow model for tracking expected state across stateful test transitions.
struct VaultShadowModel {
    pub exchange_rate: i128,
    pub balances: [(Address, i128); 3],
    pub underlying_held: i128,
    pub total_supply: i128,
    pub paused: bool,
}

impl VaultShadowModel {
    pub fn new(actors: &[Address; 3], exchange_rate: i128) -> Self {
        Self {
            exchange_rate,
            balances: [
                (actors[0].clone(), 0),
                (actors[1].clone(), 0),
                (actors[2].clone(), 0),
            ],
            underlying_held: 0,
            total_supply: 0,
            paused: false,
        }
    }

    pub fn get_balance(&self, addr: &Address) -> i128 {
        for (a, b) in &self.balances {
            if a == addr {
                return *b;
            }
        }
        0
    }

    pub fn set_balance(&mut self, addr: &Address, new_bal: i128) {
        for (a, b) in &mut self.balances {
            if a == addr {
                *b = new_bal;
                return;
            }
        }
    }

    pub fn assert_invariants(&self, env: &Env, client: &VaultContractClient) {
        // Invariant 1: total_supply == underlying_held * exchange_rate
        let contract_total = client.total_supply();
        let contract_held = client.underlying_held();
        assert_eq!(
            contract_total,
            contract_held * self.exchange_rate,
            "Invariant 1 failed: total_supply != underlying_held * exchange_rate"
        );

        // Invariant 2: sum of balances == total_supply
        let mut sum_balances = 0i128;
        for (actor, _) in &self.balances {
            sum_balances += client.balance(actor);
        }
        assert_eq!(
            sum_balances, contract_total,
            "Invariant 2 failed: sum of balances != total_supply"
        );

        // Invariant 3: every actor's balance matches shadow model
        for (actor, _) in &self.balances {
            assert_eq!(
                client.balance(actor),
                self.get_balance(actor),
                "Invariant 3 failed: balance mismatch for actor"
            );
        }

        // Invariant 4: total_supply matches model
        assert_eq!(
            contract_total, self.total_supply,
            "Invariant 4 failed: total_supply mismatch"
        );
    }
}

#[test]
fn test_stateful_vault_invariants() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(100);

    let admin = Address::generate(&env);
    let underlying_id = env.register(MockUnderlying, ());

    let actors = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    let exchange_rate = 10i128;
    let vault_id = env.register(
        VaultContract,
        (
            admin.clone(),
            VaultConfig {
                underlying: underlying_id.clone(),
                exchange_rate,
                max_share_supply: 0,
                lockup_secs: 0,
                name: String::from_str(&env, "Stateful Vault"),
                symbol: String::from_str(&env, "sVAULT"),
                compliance: None,
                min_compliance_level: 0,
            },
        ),
    );

    let client = VaultContractClient::new(&env, &vault_id);
    let mut shadow = VaultShadowModel::new(&actors, exchange_rate);

    // Initial check
    shadow.assert_invariants(&env, &client);

    // Step 1: Actor 0 deposits 10 units -> 100 shares
    client.deposit(&actors[0], &10);
    shadow.set_balance(&actors[0], 100);
    shadow.underlying_held += 10;
    shadow.total_supply += 100;
    shadow.assert_invariants(&env, &client);

    // Step 2: Actor 1 deposits 25 units -> 250 shares
    client.deposit(&actors[1], &25);
    shadow.set_balance(&actors[1], 250);
    shadow.underlying_held += 25;
    shadow.total_supply += 250;
    shadow.assert_invariants(&env, &client);

    // Step 3: Actor 1 transfers 50 shares to Actor 2
    client.transfer(&actors[1], &MuxedAddress::from(actors[2].clone()), &50);
    shadow.set_balance(&actors[1], 200);
    shadow.set_balance(&actors[2], 50);
    shadow.assert_invariants(&env, &client);

    // Step 4: Actor 0 redeems 30 shares -> receives 3 underlying units
    client.redeem(&actors[0], &30);
    shadow.set_balance(&actors[0], 70);
    shadow.underlying_held -= 3;
    shadow.total_supply -= 30;
    shadow.assert_invariants(&env, &client);

    // Step 5: Actor 2 approves Actor 0 for 20 shares, Actor 0 transfers from Actor 2 to Actor 1
    client.approve(&actors[2], &actors[0], &20, &500);
    client.transfer_from(&actors[0], &actors[2], &actors[1], &20);
    shadow.set_balance(&actors[2], 30);
    shadow.set_balance(&actors[1], 220);
    shadow.assert_invariants(&env, &client);

    // Step 6: Advance ledger and verify historical checkpoints match shadow model
    env.ledger().set_sequence_number(200);
    assert_eq!(client.balance_at(&actors[0], &150), 70);
    assert_eq!(client.balance_at(&actors[1], &150), 220);
    assert_eq!(client.balance_at(&actors[2], &150), 30);
    assert_eq!(client.total_supply_at(&150), 320);

    // Step 7: Final full redemptions drain vault cleanly
    client.redeem(&actors[0], &70);
    client.redeem(&actors[1], &220);
    client.redeem(&actors[2], &30);

    shadow.set_balance(&actors[0], 0);
    shadow.set_balance(&actors[1], 0);
    shadow.set_balance(&actors[2], 0);
    shadow.underlying_held = 0;
    shadow.total_supply = 0;
    shadow.assert_invariants(&env, &client);
}
