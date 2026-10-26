#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    Address, Bytes, Env, String,
};

use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient};

/// Deterministic Pseudo-Random Number Generator (PCG / LCG) for reproducible fuzzing.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_range(&mut self, min: i128, max: i128) -> i128 {
        if min >= max {
            return min;
        }
        let diff = (max - min) as u64;
        let rand = (self.next_u32() as u64) % diff;
        min + (rand as i128)
    }
}

fn setup_fuzz_env() -> (Env, Address, Address, RwaAssetContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let metadata = AssetMetadata {
        name: String::from_str(&env, "Fuzz Asset Token"),
        symbol: String::from_str(&env, "FUZZ"),
        decimals: 7,
        asset_class: String::from_str(&env, "fuzz"),
        legal_doc_hash: Bytes::from_array(&env, &[0u8; 32]),
        max_supply: 1_000_000_000_000,
    };

    let contract_id = env.register(RwaAssetContract, (&admin, &metadata));
    let client = RwaAssetContractClient::new(&env, &contract_id);
    client.set_issuer(&issuer, &true);

    (env, admin, issuer, client)
}

#[test]
fn test_fuzz_transfers_and_allowance_invariant() {
    let (env, _, issuer, client) = setup_fuzz_env();
    let mut rng = FuzzRng::new(0x1337_CAFE_BEEF);

    // Create 4 distinct user accounts
    let users = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    // Initial funding
    for user in &users {
        client.mint(&issuer, user, &100_000);
    }

    // Invariant check helper
    let assert_supply_invariant = || {
        let total_supply = client.total_supply();
        let sum_balances: i128 = users.iter().map(|u| client.balance(u)).sum();
        assert_eq!(
            total_supply, sum_balances,
            "Supply invariant violated: supply={total_supply}, sum_balances={sum_balances}"
        );
    };

    assert_supply_invariant();

    // Run 100 randomized transaction steps
    for step in 0..100 {
        let op = rng.next_u32() % 5;
        let u1_idx = (rng.next_u32() as usize) % users.len();
        let mut u2_idx = (rng.next_u32() as usize) % users.len();
        if u1_idx == u2_idx {
            u2_idx = (u2_idx + 1) % users.len();
        }

        let user1 = &users[u1_idx];
        let user2 = &users[u2_idx];
        let amount = rng.next_range(1, 10_000);

        match op {
            0 => {
                // Direct transfer
                let bal = client.balance(user1);
                if bal >= amount {
                    client.transfer(user1, user2, &amount);
                } else {
                    assert!(client.try_transfer(user1, user2, &amount).is_err());
                }
            }
            1 => {
                // Approve allowance
                client.approve(user1, user2, &amount, &1000);
                assert_eq!(client.allowance(user1, user2), amount);
            }
            2 => {
                // Transfer from allowance
                let allowed = client.allowance(user1, user2);
                let bal = client.balance(user1);
                if allowed >= amount && bal >= amount {
                    client.transfer_from(user2, user1, user2, &amount);
                } else {
                    assert!(client.try_transfer_from(user2, user1, user2, &amount).is_err());
                }
            }
            3 => {
                // Burn
                let bal = client.balance(user1);
                if bal >= amount {
                    client.burn(user1, &amount);
                } else {
                    assert!(client.try_burn(user1, &amount).is_err());
                }
            }
            4 => {
                // Mint more
                client.mint(&issuer, user1, &amount);
            }
            _ => unreachable!(),
        }

        assert_supply_invariant();
    }
}
