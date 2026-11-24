#![no_main]

use contracts_vault::{
    UnderlyingTokenInterface, VaultConfig, VaultContract, VaultContractClient,
};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    Address, Env, MuxedAddress, String,
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

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying_id = env.register(MockUnderlying, ());

    let vault_id = env.register(
        VaultContract,
        (
            admin.clone(),
            VaultConfig {
                underlying: underlying_id.clone(),
                exchange_rate: 10,
                max_share_supply: 1_000_000,
                lockup_secs: 0,
                name: String::from_str(&env, "Fuzz Vault"),
                symbol: String::from_str(&env, "fVAULT"),
                compliance: None,
                min_compliance_level: 0,
            },
        ),
    );

    let client = VaultContractClient::new(&env, &vault_id);
    let user = Address::generate(&env);

    for chunk in data.chunks(4) {
        let op = chunk[0] % 4;
        let val = (u16::from_be_bytes([chunk[1], chunk[2]]) as i128) % 1000 + 1;

        match op {
            0 => {
                let _ = client.try_deposit(&user, &val);
            }
            1 => {
                let shares = val * 10;
                let _ = client.try_redeem(&user, &shares);
            }
            2 => {
                let muxed = MuxedAddress::from(user.clone());
                let _ = client.try_transfer(&user, &muxed, &val);
            }
            3 => {
                let _ = client.try_redeemable(&user);
            }
            _ => unreachable!(),
        }

        // Global invariant check after every step
        let total = client.total_supply();
        let held = client.underlying_held();
        assert_eq!(total, held * 10);
    }
});
