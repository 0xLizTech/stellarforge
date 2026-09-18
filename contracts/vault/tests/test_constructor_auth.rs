#![cfg(test)]

//! The constructor's `admin.require_auth()` stops a deployer naming an
//! admin who never agreed to the role (IR-17, ADR-002).
//!
//! `Env::register` authorizes constructor calls itself, but under `mock_all_auths`
//! it records which authorizations the constructor demanded. Asserting that
//! record ensures that removing `require_auth` fails the test.

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, String, Symbol,
};

use vault::{VaultConfig, VaultContract};

#[contract]
struct MockUnderlying;

#[contractimpl]
impl MockUnderlying {
    pub fn decimals(_env: Env) -> u32 {
        7
    }
}

#[test]
fn test_constructor_requires_the_admin_to_authorize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let underlying = env.register(MockUnderlying, ());

    let config = VaultConfig {
        underlying: underlying.clone(),
        exchange_rate: 1,
        max_share_supply: 1_000_000,
        lockup_secs: 86400,
        name: String::from_str(&env, "Vault Share"),
        symbol: String::from_str(&env, "vRWA"),
        compliance: None,
        min_compliance_level: 0,
    };

    let id = env.register(VaultContract, (&admin, &config));

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id,
                    Symbol::new(&env, "__constructor"),
                    (admin, config).into_val(&env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}
