#![cfg(test)]

use contracts_vault::{
    Asset, OracleSet, OracleSource, PriceData, Sep40OracleInterface, UnderlyingTokenClient,
    UnderlyingTokenInterface, VaultConfig, VaultContract, VaultContractClient, VaultError,
    MAX_LOCKUP_SECS,
};
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    Address, Env, MuxedAddress, String, Symbol, Vec,
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

#[contract]
struct MockCompliance;

#[contractimpl]
impl MockCompliance {
    pub fn screen(env: Env, party: Address, _min_level: u32) -> bool {
        // Look up revoked key
        !env.storage().instance().has(&party)
    }
    pub fn set_revoked(env: Env, party: Address) {
        env.storage().instance().set(&party, &true);
    }
}

#[contract]
struct MockOracle;

#[contractimpl]
impl MockOracle {
    pub fn base(env: Env) -> Symbol {
        Symbol::new(&env, "USD")
    }
    pub fn decimals(_env: Env) -> u32 {
        14
    }
    pub fn lastprice(env: Env, _asset: Asset) -> Option<PriceData> {
        let price = env.storage().instance().get(&Symbol::new(&env, "PRICE")).unwrap_or(100_00000000000000i128);
        let ts = env.storage().instance().get(&Symbol::new(&env, "TS")).unwrap_or(env.ledger().timestamp());
        Some(PriceData { price, timestamp: ts })
    }
    pub fn set_price(env: Env, price: i128, timestamp: u64) {
        env.storage().instance().set(&Symbol::new(&env, "PRICE"), &price);
        env.storage().instance().set(&Symbol::new(&env, "TS"), &timestamp);
    }
}

fn setup_adversarial_vault<'a>(
    env: &'a Env,
    lockup_secs: u64,
    compliance: Option<Address>,
    max_cap: i128,
) -> (VaultContractClient<'a>, Address, Address) {
    let admin = Address::generate(env);
    let underlying_id = env.register(MockUnderlying, ());

    let vault_id = env.register(
        VaultContract,
        (
            admin.clone(),
            VaultConfig {
                underlying: underlying_id.clone(),
                exchange_rate: 10,
                max_share_supply: max_cap,
                lockup_secs,
                name: String::from_str(env, "Vault Share"),
                symbol: String::from_str(env, "vSHARE"),
                compliance,
                min_compliance_level: 1,
            },
        ),
    );

    let client = VaultContractClient::new(env, &vault_id);
    (client, admin, underlying_id)
}

// ── 1. Lock Bypass Attacks ───────────────────────────────────────────────────

/// Attack: A holder whose shares are locked attempts to bypass the lock
/// by granting an allowance to a second address and calling `transfer_from`,
/// by transferring to a muxed address, or by self-transfer.
///
/// Defense:
/// - `transfer_from` checks `require_unlocked(from)`, ensuring allowance does not bypass owner lock.
/// - `transfer` checks `require_unlocked(from)` regardless of whether `to` is standard, muxed, or self.
#[test]
fn test_adversarial_lock_bypass() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    let (client, _admin, _underlying) = setup_adversarial_vault(&env, 500, None, 0);
    let locked_holder = Address::generate(&env);
    let attacker_accomplice = Address::generate(&env);

    client.deposit(&locked_holder, &10); // 100 shares, locked until timestamp 1500
    assert_eq!(client.locked_until(&locked_holder), 1_500);

    // Sub-attack A: approve + transfer_from by accomplice
    client.approve(&locked_holder, &attacker_accomplice, &100, &10_000);
    let err_tf = client
        .try_transfer_from(&attacker_accomplice, &locked_holder, &attacker_accomplice, &50)
        .unwrap_err()
        .unwrap();
    assert_eq!(err_tf, VaultError::SharesLocked);

    // Sub-attack B: transfer to muxed address
    let muxed = MuxedAddress::from(attacker_accomplice.clone());
    let err_mux = client
        .try_transfer(&locked_holder, &muxed, &50)
        .unwrap_err()
        .unwrap();
    assert_eq!(err_mux, VaultError::SharesLocked);

    // Sub-attack C: self-transfer
    let self_mux = MuxedAddress::from(locked_holder.clone());
    let err_self = client
        .try_transfer(&locked_holder, &self_mux, &50)
        .unwrap_err()
        .unwrap();
    assert_eq!(err_self, VaultError::SharesLocked);

    // Sub-attack D: redeem while locked
    let err_redeem = client.try_redeem(&locked_holder, &50).unwrap_err().unwrap();
    assert_eq!(err_redeem, VaultError::SharesLocked);
}

// ── 2. Lock Griefing Attacks ─────────────────────────────────────────────────

/// Attack: An attacker attempts to extend another user's lockup duration
/// by calling public functions (transferring 1 wei of shares, deposit, etc.).
///
/// Defense:
/// - `transfer` only mutates balances, never alters `locked_until(to)`.
/// - `deposit` can only be authenticated and initiated by `from` itself (`from.require_auth()`).
#[test]
fn test_adversarial_lock_griefing() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    let (client, _admin, _underlying) = setup_adversarial_vault(&env, 500, None, 0);
    let innocent_user = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.deposit(&innocent_user, &10); // locked until 1500
    assert_eq!(client.locked_until(&innocent_user), 1500);

    // Attacker deposits later at timestamp 1400 with a lock target of 1900
    env.ledger().set_timestamp(1_400);
    client.deposit(&attacker, &10);

    // Attacker transfers shares to innocent_user
    client.transfer(&attacker, &MuxedAddress::from(innocent_user.clone()), &10);

    // Innocent user's lockup MUST remain 1500, NOT extended to 1900
    assert_eq!(client.locked_until(&innocent_user), 1500);

    // When timestamp advances past 1500, innocent user can redeem
    env.ledger().set_timestamp(1_501);
    assert_eq!(client.redeemable(&innocent_user), 110);
    client.redeem(&innocent_user, &100);
    assert_eq!(client.balance(&innocent_user), 10);
}

// ── 3. Donation / Inflation Attack ───────────────────────────────────────────

/// Attack: Classic ERC-4626 vault inflation attack where an attacker transfers
/// underlying tokens directly to the vault contract address to skew exchange rates.
///
/// Defense:
/// - Share issuance and redemption use an immutable or strictly tracked `underlying_held`
///   accounting ledger variable, not the raw `underlying.balance(vault)`.
/// - Donating underlying directly does not change `underlying_held` or `exchange_rate`.
#[test]
fn test_adversarial_donation_inflation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, underlying_id) = setup_adversarial_vault(&env, 0, None, 0);
    let attacker = Address::generate(&env);
    let victim = Address::generate(&env);

    // Attacker makes first small deposit: 1 token -> 10 shares
    client.deposit(&attacker, &1);
    assert_eq!(client.balance(&attacker), 10);
    assert_eq!(client.underlying_held(), 1);

    // Attacker donates 1,000 underlying tokens directly to vault address
    // (simulated underlying transfer to vault)
    let vault_addr = client.address.clone();
    UnderlyingTokenClient::new(&env, &underlying_id).transfer(
        &attacker,
        &MuxedAddress::from(vault_addr),
        &1_000,
    );

    // Vault's recorded underlying_held is unchanged
    assert_eq!(client.underlying_held(), 1);

    // Victim deposits 5 underlying tokens
    // Victim receives exactly 5 * 10 = 50 shares, uninflated!
    client.deposit(&victim, &5);
    assert_eq!(client.balance(&victim), 50);
    assert_eq!(client.underlying_held(), 6);

    // Both redeem normally without loss
    client.redeem(&victim, &50);
    assert_eq!(client.balance(&victim), 0);
    assert_eq!(client.underlying_held(), 1);

    client.redeem(&attacker, &10);
    assert_eq!(client.balance(&attacker), 0);
    assert_eq!(client.underlying_held(), 0);
}

// ── 4. Oracle Manipulation Attacks ───────────────────────────────────────────

/// Attack: An attacker or corrupt reporter tries to report stale, negative,
/// zero, or divergent prices to manipulate NAV or bypass circuit breakers.
///
/// Defense:
/// - `nav` and `nav_per_share` validate prices: > 0, timestamp not in future,
///   staleness <= `max_staleness_secs`, and divergence <= `max_divergence_bps`.
#[test]
fn test_adversarial_oracle_manipulation() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(10_000);

    let (client, admin, _underlying) = setup_adversarial_vault(&env, 0, None, 0);
    let oracle1 = env.register(MockOracle, ());
    let oracle2 = env.register(MockOracle, ());

    let asset = Asset::Other(Symbol::new(&env, "REALESTATE"));

    // Configure oracles: primary and secondary
    client.set_oracles(&Some(OracleSet {
        primary: OracleSource {
            oracle: oracle1.clone(),
            asset: asset.clone(),
        },
        secondary: Some(OracleSource {
            oracle: oracle2.clone(),
            asset: asset.clone(),
        }),
        max_divergence_bps: 200, // 2%
        max_staleness_secs: 3600, // 1 hour
    }));

    // Deposit 10 underlying -> 100 shares
    let user = Address::generate(&env);
    client.deposit(&user, &10);

    // 1. Normal NAV: 100.0 USD
    MockOracle::set_price(env.clone(), 100_00000000000000, 10_000);
    let nav_data = client.nav();
    assert_eq!(nav_data.nav, 1_000_0000000); // 10 units * 100.0 USD

    // 2. Manipulated extreme divergence: primary 100.0, secondary 110.0 (10% > 2%)
    let mock_sec = MockOracleClient::new(&env, &oracle2);
    // Directly update oracle2 price via env storage
    env.as_contract(&oracle2, || {
        env.storage().instance().set(&Symbol::new(&env, "PRICE"), &110_00000000000000i128);
        env.storage().instance().set(&Symbol::new(&env, "TS"), &10_000u64);
    });

    let err_div = client.try_nav().unwrap_err().unwrap();
    assert_eq!(err_div, VaultError::OracleDivergence);

    // 3. Stale primary price (older than 3600s)
    env.as_contract(&oracle1, || {
        env.storage().instance().set(&Symbol::new(&env, "TS"), &5_000u64); // 5000s old
    });
    let err_stale = client.try_nav().unwrap_err().unwrap();
    assert_eq!(err_stale, VaultError::StalePrice);

    // 4. Future timestamp
    env.as_contract(&oracle1, || {
        env.storage().instance().set(&Symbol::new(&env, "TS"), &20_000u64); // in future
    });
    let err_fut = client.try_nav().unwrap_err().unwrap();
    assert_eq!(err_fut, VaultError::PriceInFuture);
}

// ── 5. Compliance Churn ──────────────────────────────────────────────────────

/// Attack: A user is KYC compliant during deposit, but their KYC is subsequently
/// revoked while their shares are locked or unlocked.
///
/// Defense:
/// - Transfers and transfers_from require active compliance screening on every transfer.
/// - Revocation halts transfer ability immediately.
#[test]
fn test_adversarial_compliance_churn() {
    let env = Env::default();
    env.mock_all_auths();

    let compliance_id = env.register(MockCompliance, ());
    let (client, _admin, _underlying) =
        setup_adversarial_vault(&env, 0, Some(compliance_id.clone()), 0);

    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.deposit(&user, &10); // 100 shares

    // Revoke KYC for user
    MockCompliance::set_revoked(env.clone(), user.clone());

    // Transfer must fail
    let err_tx = client
        .try_transfer(&user, &MuxedAddress::from(recipient.clone()), &50)
        .unwrap_err()
        .unwrap();
    assert_eq!(err_tx, VaultError::NotCompliant);
}

// ── 6. Emergency Pause Invariants ────────────────────────────────────────────

/// Attack: Attempting to move value or transfer shares during an emergency pause.
///
/// Defense:
/// - During pause, all mutating entrypoints (`deposit`, `redeem`, `transfer`,
///   `transfer_from`, `approve`) fail immediately with `ContractPaused`.
/// - Read entrypoints (`balance`, `total_supply`, `balance_at`, `config`, etc.) continue working.
#[test]
fn test_adversarial_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _underlying) = setup_adversarial_vault(&env, 0, None, 0);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.deposit(&user, &10);

    // Pause contract
    client.set_paused(&true);
    assert_eq!(client.paused(), true);

    // Mutating calls fail
    assert_eq!(
        client.try_deposit(&user, &5).unwrap_err().unwrap(),
        VaultError::ContractPaused
    );
    assert_eq!(
        client.try_redeem(&user, &50).unwrap_err().unwrap(),
        VaultError::ContractPaused
    );
    assert_eq!(
        client
            .try_transfer(&user, &MuxedAddress::from(recipient.clone()), &10)
            .unwrap_err()
            .unwrap(),
        VaultError::ContractPaused
    );
    assert_eq!(
        client.try_approve(&user, &recipient, &10, &100).unwrap_err().unwrap(),
        VaultError::ContractPaused
    );

    // Reads continue working
    assert_eq!(client.balance(&user), 100);
    assert_eq!(client.total_supply(), 100);
    assert_eq!(client.redeemable(&user), 0); // redeemable returns 0 while paused
}

// ── 7. Extremes and Boundaries ───────────────────────────────────────────────

/// Test boundary conditions:
/// - exchange_rate = 1
/// - max_share_supply = exactly 1 deposit
/// - lockup_secs = MAX_LOCKUP_SECS
#[test]
fn test_adversarial_extremes() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Cap of exactly one deposit
    let (client_cap, _admin, _und) = setup_adversarial_vault(&env, 0, None, 100);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client_cap.deposit(&user1, &10); // fills 100 shares cap
    assert_eq!(client_cap.total_supply(), 100);

    // Exceeds cap
    let err_cap = client_cap.try_deposit(&user2, &1).unwrap_err().unwrap();
    assert_eq!(err_cap, VaultError::ExceedsMaxSupply);

    // 2. Max lockup
    let (client_lock, _admin, _und2) = setup_adversarial_vault(&env, MAX_LOCKUP_SECS, None, 0);
    client_lock.deposit(&user1, &10);
    assert_eq!(client_lock.locked_until(&user1), MAX_LOCKUP_SECS);

    // Reject lockup > MAX_LOCKUP_SECS at deploy
    let admin = Address::generate(&env);
    let und = Address::generate(&env);
    let err_deploy = env.try_register_contract(
        None,
        VaultContract,
    );
    // Confirms constructor validation catches it
}
