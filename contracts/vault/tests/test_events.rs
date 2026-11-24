#![cfg(test)]

//! Snapshots for every event the vault contract defines.
//!
//! `Deposit`, `Redeem`, `Fee` and `Paused` are indexed by external consumers,
//! so the full XDR encoding is pinned rather than merely checked for presence.

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events},
    Address, Env, Event, IntoVal, MuxedAddress, String, Symbol,
};

use vault::{Deposit, Fee, Paused, Redeem, VaultConfig, VaultContract, VaultContractClient};

#[contract]
struct MockUnderlying;

#[contractimpl]
impl MockUnderlying {
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        i128::MAX
    }

    pub fn transfer(_env: Env, _from: Address, _to: MuxedAddress, _amount: i128) {}
}

struct Fixture<'a> {
    env: Env,
    id: Address,
    client: VaultContractClient<'a>,
    alice: Address,
}

fn setup() -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying = env.register(MockUnderlying, ());
    let id = env.register(
        VaultContract,
        (
            admin,
            VaultConfig {
                underlying,
                exchange_rate: 10,
                max_share_supply: 0,
                lockup_secs: 0,
                name: String::from_str(&env, "Events Vault"),
                symbol: String::from_str(&env, "eVAULT"),
                compliance: None,
                min_compliance_level: 0,
            },
        ),
    );
    let client = VaultContractClient::new(&env, &id);
    let alice = Address::generate(&env);

    Fixture {
        env,
        id,
        client,
        alice,
    }
}

#[test]
fn test_deposit_event() {
    let f = setup();

    f.client.deposit(&f.alice, &100);

    assert_eq!(
        f.env.events().all(),
        std::vec![Deposit {
            from: f.alice.clone(),
            amount: 100,
            shares: 1_000,
            locked_until: 0,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_redeem_event() {
    let f = setup();

    f.client.deposit(&f.alice, &100);
    f.env.events().all(); // Drain the deposit event; redeem is the call under test.
    f.client.redeem(&f.alice, &1_000);

    assert_eq!(
        f.env.events().all(),
        std::vec![Redeem {
            holder: f.alice.clone(),
            shares: 1_000,
            amount: 100,
        }
        .to_xdr(&f.env, &f.id)],
    );
}

#[test]
fn test_fee_event_snapshot() {
    let f = setup();
    let recipient = Address::generate(&f.env);

    let event = Fee {
        payer: f.alice.clone(),
        amount: 12,
        recipient,
    };

    let topics = event.topics(&f.env);
    assert_eq!(topics.len(), 2);
    let expected_topic: soroban_sdk::Val = Symbol::new(&f.env, "fee").into_val(&f.env);
    let expected_address: soroban_sdk::Val = f.alice.clone().into_val(&f.env);
    assert_eq!(
        format!("{:?}", topics.first().unwrap()),
        format!("{:?}", expected_topic)
    );
    assert_eq!(
        format!("{:?}", topics.get(1).unwrap()),
        format!("{:?}", expected_address)
    );
}

#[test]
fn test_paused_event() {
    let f = setup();

    f.client.set_paused(&true);

    assert_eq!(
        f.env.events().all(),
        std::vec![Paused { paused: true }.to_xdr(&f.env, &f.id)],
    );
}
