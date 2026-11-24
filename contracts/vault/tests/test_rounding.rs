#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, MuxedAddress, String,
};

use vault::{VaultConfig, VaultContract, VaultContractClient, VaultError};

// ─── Stateful Mock Underlying Token ──────────────────────────────────────────

#[contract]
struct TrackingMockToken;

#[contractimpl]
impl TrackingMockToken {
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let key = (symbol_short!("BAL"), to);
        let cur: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&key, &cur.checked_add(amount).expect("mock mint overflow"));
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        let key = (symbol_short!("BAL"), id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        let to_addr = to.address();
        let from_key = (symbol_short!("BAL"), from);
        let to_key = (symbol_short!("BAL"), to_addr);
        let from_bal: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
        assert!(from_bal >= amount, "mock token insufficient balance");
        env.storage()
            .persistent()
            .set(&from_key, &(from_bal - amount));
        let to_bal: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);
        env.storage().persistent().set(
            &to_key,
            &to_bal.checked_add(amount).expect("mock transfer overflow"),
        );
    }
}

// ─── Harness ─────────────────────────────────────────────────────────────────

struct RoundingHarness {
    env: Env,
    vault_id: Address,
    client: VaultContractClient<'static>,
    token: TrackingMockTokenClient<'static>,
    exchange_rate: i128,
}

fn setup_vault(exchange_rate: i128, max_share_supply: i128, lockup_secs: u64) -> RoundingHarness {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    let admin = Address::generate(&env);
    let token_id = env.register(TrackingMockToken, ());
    let token = TrackingMockTokenClient::new(&env, &token_id);

    let config = VaultConfig {
        underlying: token_id,
        exchange_rate,
        max_share_supply,
        lockup_secs,
        name: String::from_str(&env, "Rounding Test Vault"),
        symbol: String::from_str(&env, "vRND"),
        compliance: None,
        min_compliance_level: 0,
    };

    let vault_id = env.register(VaultContract, (&admin, &config));
    let client = VaultContractClient::new(&env, &vault_id);

    RoundingHarness {
        env,
        vault_id,
        client,
        token,
        exchange_rate,
    }
}

impl RoundingHarness {
    /// Asserts core solvency and rounding invariants:
    /// 1. `total_supply == underlying_held * exchange_rate` (exact backing; vault never loses value)
    /// 2. `token.balance(vault) >= underlying_held` (actual underlying held matches accounting)
    /// 3. `sum(balances of holders) == total_supply`
    fn assert_vault_invariants(&self, holders: &[&Address]) {
        let total_supply = self.client.total_supply();
        let underlying_held = self.client.underlying_held();
        let vault_token_bal = self.token.balance(&self.vault_id);

        assert_eq!(
            total_supply,
            underlying_held
                .checked_mul(self.exchange_rate)
                .expect("invariant mul overflow"),
            "total_supply must equal underlying_held * exchange_rate"
        );
        assert_eq!(
            vault_token_bal, underlying_held,
            "actual vault token balance must match underlying_held"
        );

        let mut sum_shares: i128 = 0;
        let mut sum_redeemable_underlying: i128 = 0;
        for h in holders {
            let bal = self.client.balance(h);
            assert!(bal >= 0, "holder balance must be non-negative");
            sum_shares = sum_shares.checked_add(bal).expect("sum_shares overflow");
            let redeemable_shares = self.client.redeemable(h);
            assert!(
                redeemable_shares <= bal,
                "redeemable shares must never exceed holder balance"
            );
            assert_eq!(
                redeemable_shares % self.exchange_rate,
                0,
                "redeemable shares must be an exact multiple of exchange_rate"
            );
            sum_redeemable_underlying = sum_redeemable_underlying
                .checked_add(self.client.preview_redeem(&redeemable_shares))
                .expect("sum_redeemable overflow");
        }

        assert_eq!(
            sum_shares, total_supply,
            "sum of holder shares must equal total_supply"
        );
        assert!(
            sum_redeemable_underlying <= underlying_held,
            "aggregate redeemable underlying must never exceed vault underlying_held"
        );
    }
}

// ─── 1. First Deposit Rounding Tests ─────────────────────────────────────────

#[test]
fn test_first_deposit_tiny_amount_across_exchange_rates() {
    for rate in [1_i128, 2, 3, 7, 13, 1_000_000] {
        let h = setup_vault(rate, 0, 0);
        let alice = Address::generate(&h.env);
        h.token.mint(&alice, &1);

        assert_eq!(h.client.total_supply(), 0);
        assert_eq!(h.client.underlying_held(), 0);

        let preview = h.client.preview_deposit(&1);
        let shares = h.client.deposit(&alice, &1);
        assert_eq!(shares, rate);
        assert_eq!(shares, preview);
        assert_eq!(h.client.balance(&alice), rate);
        assert_eq!(h.client.redeemable(&alice), rate);
        h.assert_vault_invariants(&[&alice]);

        let redeemed = h.client.redeem(&alice, &shares);
        assert_eq!(redeemed, 1);
        assert_eq!(h.token.balance(&alice), 1);
        assert_eq!(h.client.total_supply(), 0);
        assert_eq!(h.client.underlying_held(), 0);
        h.assert_vault_invariants(&[&alice]);
    }
}

#[test]
fn test_first_deposit_cannot_dilute_second_depositor() {
    let rate = 7_i128;
    let h = setup_vault(rate, 0, 0);
    let attacker = Address::generate(&h.env);
    let victim = Address::generate(&h.env);
    let dust_sink = Address::generate(&h.env);

    h.token.mint(&attacker, &1_000);
    h.token.mint(&victim, &1_000);

    // First deposit by attacker of 1 unit
    let attacker_shares = h.client.deposit(&attacker, &1);
    assert_eq!(attacker_shares, rate);

    // Attacker splits shares into dust (1 share to dust_sink, rate-1 kept)
    h.client
        .transfer(&attacker, &MuxedAddress::from(dust_sink.clone()), &1);
    h.assert_vault_invariants(&[&attacker, &victim, &dust_sink]);

    // Victim deposits 1 unit and then 999 units
    let v_shares_1 = h.client.deposit(&victim, &1);
    assert_eq!(v_shares_1, rate);
    let v_shares_2 = h.client.deposit(&victim, &999);
    assert_eq!(v_shares_2, 999 * rate);
    h.assert_vault_invariants(&[&attacker, &victim, &dust_sink]);

    // Victim can redeem 100% of their deposited underlying without any rounding loss
    let victim_redeemable = h.client.redeemable(&victim);
    assert_eq!(victim_redeemable, 1_000 * rate);
    let victim_out = h.client.redeem(&victim, &victim_redeemable);
    assert_eq!(victim_out, 1_000);
    assert_eq!(h.token.balance(&victim), 1_000);

    // Vault still holds exact backing for attacker + dust_sink
    assert_eq!(h.client.underlying_held(), 1);
    h.assert_vault_invariants(&[&attacker, &victim, &dust_sink]);
}

// ─── 2. Tiny Amounts & Dust Share Rounding ───────────────────────────────────

#[test]
fn test_preview_redeem_and_redeemable_round_down_for_dust() {
    let rate = 7_i128;
    let h = setup_vault(rate, 0, 0);
    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    h.token.mint(&alice, &10);

    // Alice deposits 10 units -> 70 shares
    h.client.deposit(&alice, &10);

    // Transfer non-multiple dust shares (1..rate) to Bob and check rounding down
    for dust in 1..rate {
        assert_eq!(h.client.preview_redeem(&dust), 0);
        assert_eq!(h.client.preview_redeem(&(rate + dust)), 1);
    }

    // Transfer 10 shares to Bob (10 = 1*7 + 3 dust; Alice left with 60 = 8*7 + 4 dust)
    h.client
        .transfer(&alice, &MuxedAddress::from(bob.clone()), &10);

    assert_eq!(h.client.redeemable(&bob), 7);
    assert_eq!(h.client.redeemable(&alice), 56);
    // Sum of redeemable underlying = 1 + 8 = 9 <= 10 (vault never loses value; 1 unit backs the 3+4=7 dust shares)
    h.assert_vault_invariants(&[&alice, &bob]);

    // Attempting to redeem non-multiple shares must fail with NotMultipleOfExchangeRate
    let err_bob = h.client.try_redeem(&bob, &10).unwrap_err().unwrap();
    assert_eq!(err_bob, VaultError::NotMultipleOfExchangeRate.into());

    let err_dust = h.client.try_redeem(&bob, &3).unwrap_err().unwrap();
    assert_eq!(err_dust, VaultError::NotMultipleOfExchangeRate.into());

    // Redeeming the rounded-down redeemable amounts succeeds
    assert_eq!(h.client.redeem(&bob, &7), 1);
    assert_eq!(h.client.redeem(&alice, &56), 8);
    assert_eq!(h.client.balance(&bob), 3);
    assert_eq!(h.client.balance(&alice), 4);
    assert_eq!(h.client.underlying_held(), 1);
    h.assert_vault_invariants(&[&alice, &bob]);

    // Re-aggregating the dust shares allows clean final redemption with zero loss
    h.client
        .transfer(&bob, &MuxedAddress::from(alice.clone()), &3);
    assert_eq!(h.client.balance(&alice), 7);
    assert_eq!(h.client.redeem(&alice, &7), 1);
    assert_eq!(h.client.total_supply(), 0);
    assert_eq!(h.client.underlying_held(), 0);
    assert_eq!(h.token.balance(&alice) + h.token.balance(&bob), 10);
    h.assert_vault_invariants(&[&alice, &bob]);
}

// ─── 3. Huge Amounts & Overflow Boundaries ───────────────────────────────────

#[test]
fn test_huge_amounts_round_trip_without_rounding_loss() {
    for rate in [1_i128, 7, 1_000_000_000] {
        let h = setup_vault(rate, 0, 0);
        let whale = Address::generate(&h.env);
        let max_safe_deposit = i128::MAX / rate;
        h.token.mint(&whale, &max_safe_deposit);

        let shares = h.client.deposit(&whale, &max_safe_deposit);
        assert_eq!(shares, max_safe_deposit * rate);
        assert_eq!(h.client.underlying_held(), max_safe_deposit);
        assert_eq!(h.client.redeemable(&whale), shares);
        h.assert_vault_invariants(&[&whale]);

        let redeemed = h.client.redeem(&whale, &shares);
        assert_eq!(redeemed, max_safe_deposit);
        assert_eq!(h.token.balance(&whale), max_safe_deposit);
        assert_eq!(h.client.total_supply(), 0);
        assert_eq!(h.client.underlying_held(), 0);
        h.assert_vault_invariants(&[&whale]);
    }
}

#[test]
fn test_huge_amount_overflow_rejected_without_state_corruption() {
    let rate = 7_i128;
    let h = setup_vault(rate, 0, 0);
    let alice = Address::generate(&h.env);
    let whale = Address::generate(&h.env);

    h.token.mint(&alice, &100);
    h.client.deposit(&alice, &100);

    // Single deposit that overflows amount * exchange_rate
    let overflow_amount = (i128::MAX / rate) + 1;
    let err = h
        .client
        .try_deposit(&whale, &overflow_amount)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, VaultError::Overflow.into());

    // Deposit that fits in amount * rate individually, but overflows total_supply + shares
    let almost_max = i128::MAX / rate;
    h.token.mint(&whale, &almost_max);
    let err2 = h
        .client
        .try_deposit(&whale, &almost_max)
        .unwrap_err()
        .unwrap();
    assert_eq!(err2, VaultError::Overflow.into());

    // Alice's position and vault invariants are completely unaffected
    h.assert_vault_invariants(&[&alice, &whale]);
    assert_eq!(h.client.redeem(&alice, &(100 * rate)), 100);
    assert_eq!(h.token.balance(&alice), 100);
}

// ─── 4. Multi-Holder Rounding Sequence (No Value Extraction) ─────────────────

#[test]
fn test_rounding_sequence_never_reduces_other_holders_value() {
    let rate = 13_i128;
    let h = setup_vault(rate, 0, 0);

    let alice = Address::generate(&h.env);
    let bob = Address::generate(&h.env);
    let attacker = Address::generate(&h.env);
    let accomplice = Address::generate(&h.env);

    let alice_initial = 5_000_i128;
    let bob_initial = 12_345_i128;
    let attacker_initial = 500_i128;

    h.token.mint(&alice, &alice_initial);
    h.token.mint(&bob, &bob_initial);
    h.token.mint(&attacker, &attacker_initial);

    // Honest holders deposit
    h.client.deposit(&alice, &alice_initial);
    h.client.deposit(&bob, &bob_initial);
    h.assert_vault_invariants(&[&alice, &bob, &attacker, &accomplice]);

    // Attacker runs repeated deposit -> dust-split -> redeem cycles
    for i in 1..=20_i128 {
        let dep = (i % 5) + 1; // tiny deposits: 1..=5
        h.client.deposit(&attacker, &dep);

        // Split off prime dust to accomplice
        let dust = (i % (rate - 1)) + 1;
        if h.client.balance(&attacker) >= dust {
            h.client
                .transfer(&attacker, &MuxedAddress::from(accomplice.clone()), &dust);
        }

        // Redeem whatever is redeemable for both attacker and accomplice
        let att_red = h.client.redeemable(&attacker);
        if att_red > 0 {
            h.client.redeem(&attacker, &att_red);
        }
        let acc_red = h.client.redeemable(&accomplice);
        if acc_red > 0 {
            h.client.redeem(&accomplice, &acc_red);
        }

        // Invariants must hold at every single step
        h.assert_vault_invariants(&[&alice, &bob, &attacker, &accomplice]);

        // Honest holders' redeemable value is strictly preserved
        assert_eq!(
            h.client.preview_redeem(&h.client.redeemable(&alice)),
            alice_initial
        );
        assert_eq!(
            h.client.preview_redeem(&h.client.redeemable(&bob)),
            bob_initial
        );
    }

    // Honest holders redeem their full positions
    let alice_out = h.client.redeem(&alice, &h.client.redeemable(&alice));
    let bob_out = h.client.redeem(&bob, &h.client.redeemable(&bob));
    assert_eq!(alice_out, alice_initial);
    assert_eq!(bob_out, bob_initial);
    assert_eq!(h.token.balance(&alice), alice_initial);
    assert_eq!(h.token.balance(&bob), bob_initial);

    // Attacker + accomplice consolidate remaining dust and redeem
    let acc_bal = h.client.balance(&accomplice);
    if acc_bal > 0 {
        h.client
            .transfer(&accomplice, &MuxedAddress::from(attacker.clone()), &acc_bal);
    }
    let final_att_red = h.client.redeemable(&attacker);
    if final_att_red > 0 {
        h.client.redeem(&attacker, &final_att_red);
    }

    // Attacker + accomplice never extracted more underlying than attacker started with
    let attacker_total_underlying = h.token.balance(&attacker) + h.token.balance(&accomplice);
    assert!(
        attacker_total_underlying <= attacker_initial,
        "attacker must never gain underlying from rounding sequences"
    );
    h.assert_vault_invariants(&[&alice, &bob, &attacker, &accomplice]);
}
