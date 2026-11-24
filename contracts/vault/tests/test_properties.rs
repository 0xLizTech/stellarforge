#![cfg(test)]

//! Randomized property tests for the vault's core accounting invariants (#235).
//!
//! The vault promises that shares are a fixed claim on the underlying asset:
//! `total_supply == underlying_held * exchange_rate`. Every deposit, transfer
//! and redemption must preserve that identity, keep the sum of holder balances
//! equal to the supply, and never reduce the assets backing one share.
//!
//! The cases are generated with a deterministic xorshift PRNG rather than a
//! property-test crate: a failing seed is reproducible from the log, and the
//! test needs no extra dependency in a `no_std` contract workspace.

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, Env, MuxedAddress, String,
};

use vault::{VaultConfig, VaultContract, VaultContractClient};

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

/// Number of randomized operations the test runs.
const CASES: u64 = 10_000;

/// Actors whose balances the shadow model tracks.
const ACTORS: usize = 4;

/// A small non-unit rate, so redemption's "exact multiple" rule is exercised.
const EXCHANGE_RATE: i128 = 7;

/// Deterministic xorshift64* generator.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        assert_ne!(seed, 0, "xorshift seed must be non-zero");
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform-ish value in `0..bound`; `bound` must be positive.
    fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0);
        self.next_u64() % bound
    }
}

/// Expected state, updated alongside each contract call.
struct Shadow {
    balances: [i128; ACTORS],
    underlying_held: i128,
    total_supply: i128,
    /// Assets per share before the latest operation, as a cross-multiplied pair.
    previous_held: i128,
    previous_supply: i128,
}

impl Shadow {
    fn new() -> Self {
        Self {
            balances: [0; ACTORS],
            underlying_held: 0,
            total_supply: 0,
            previous_held: 0,
            previous_supply: 0,
        }
    }

    fn remember_ratio(&mut self) {
        self.previous_held = self.underlying_held;
        self.previous_supply = self.total_supply;
    }

    fn assert_invariants(&self, client: &VaultContractClient, actors: &[Address; ACTORS]) {
        let total_supply = client.total_supply();
        let underlying_held = client.underlying_held();

        assert_eq!(
            total_supply,
            underlying_held * EXCHANGE_RATE,
            "supply is not backed by held underlying"
        );
        assert_eq!(
            total_supply, self.total_supply,
            "supply diverged from the model"
        );
        assert_eq!(
            underlying_held, self.underlying_held,
            "underlying held diverged from the model"
        );

        let mut sum = 0i128;
        for (i, actor) in actors.iter().enumerate() {
            let balance = client.balance(actor);
            assert_eq!(balance, self.balances[i], "balance diverged for actor {i}");
            assert!(balance >= 0, "negative balance for actor {i}");
            sum += balance;
        }
        assert_eq!(sum, total_supply, "balances do not sum to total supply");

        // Assets per share must not fall. With a fixed exchange rate and no
        // NAV change it stays constant; cross-multiplication avoids rounding.
        if self.previous_supply > 0 && total_supply > 0 {
            assert!(
                underlying_held * self.previous_supply >= self.previous_held * total_supply,
                "assets per share decreased"
            );
        }
    }
}

fn deposit(
    client: &VaultContractClient,
    shadow: &mut Shadow,
    actors: &[Address; ACTORS],
    actor: usize,
    amount: i128,
) {
    shadow.remember_ratio();
    client.deposit(&actors[actor], &amount);
    let shares = amount * EXCHANGE_RATE;
    shadow.balances[actor] += shares;
    shadow.underlying_held += amount;
    shadow.total_supply += shares;
}

fn transfer(
    client: &VaultContractClient,
    shadow: &mut Shadow,
    actors: &[Address; ACTORS],
    from: usize,
    to: usize,
    amount: i128,
) {
    shadow.remember_ratio();
    client.transfer(
        &actors[from],
        &MuxedAddress::from(actors[to].clone()),
        &amount,
    );
    shadow.balances[from] -= amount;
    shadow.balances[to] += amount;
}

fn redeem(
    client: &VaultContractClient,
    shadow: &mut Shadow,
    actors: &[Address; ACTORS],
    holder: usize,
    units: i128,
) {
    shadow.remember_ratio();
    let shares = units * EXCHANGE_RATE;
    client.redeem(&actors[holder], &shares);
    shadow.balances[holder] -= shares;
    shadow.underlying_held -= units;
    shadow.total_supply -= shares;
}

#[test]
fn randomized_operations_preserve_the_share_invariant() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying = env.register(MockUnderlying, ());

    let vault_id = env.register(
        VaultContract,
        (
            admin,
            VaultConfig {
                underlying,
                exchange_rate: EXCHANGE_RATE,
                max_share_supply: 0,
                lockup_secs: 0,
                name: String::from_str(&env, "Property Vault"),
                symbol: String::from_str(&env, "pVAULT"),
                compliance: None,
                min_compliance_level: 0,
            },
        ),
    );
    let client = VaultContractClient::new(&env, &vault_id);

    let actors = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    let mut shadow = Shadow::new();
    let mut rng = Rng::new(0x5EED_5F0C_E123_4567);

    shadow.assert_invariants(&client, &actors);

    for _ in 0..CASES {
        match rng.below(3) {
            // Deposit: 1..=50 underlying units.
            0 => {
                let actor = rng.below(ACTORS as u64) as usize;
                let amount = 1 + rng.below(50) as i128;
                deposit(&client, &mut shadow, &actors, actor, amount);
            }
            // Transfer: at most the sender's balance, in whole exchange-rate
            // units, so balances stay a multiple of `EXCHANGE_RATE` and the
            // final sweep can drain the vault to zero.
            1 => {
                let from = rng.below(ACTORS as u64) as usize;
                let to = rng.below(ACTORS as u64) as usize;
                let balance = shadow.balances[from];
                let max_units = (balance / EXCHANGE_RATE).min(50);
                if from != to && max_units > 0 {
                    let units = 1 + rng.below(max_units as u64);
                    let amount = units as i128 * EXCHANGE_RATE;
                    transfer(&client, &mut shadow, &actors, from, to, amount);
                }
            }
            // Redeem: whole underlying units, so an exact share multiple.
            _ => {
                let holder = rng.below(ACTORS as u64) as usize;
                let units = shadow.balances[holder] / EXCHANGE_RATE;
                if units > 0 {
                    let amount = 1 + rng.below(units.min(50) as u64) as i128;
                    redeem(&client, &mut shadow, &actors, holder, amount);
                }
            }
        }

        shadow.assert_invariants(&client, &actors);
    }

    // A final sweep: every holder redeems what remains, leaving no stranded
    // underlying behind and keeping the invariant intact at zero supply.
    for holder in 0..ACTORS {
        let units = shadow.balances[holder] / EXCHANGE_RATE;
        if units > 0 {
            redeem(&client, &mut shadow, &actors, holder, units);
            shadow.assert_invariants(&client, &actors);
        }
    }

    assert_eq!(client.total_supply(), 0);
    assert_eq!(client.underlying_held(), 0);
}
