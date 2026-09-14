#![cfg(test)]

//! The constructor's `admin.require_auth()` is what stops a deployer naming an
//! admin who never agreed to the role (IR-17, applied to the Phase 2 contracts).
//!
//! `Env::register` authorizes constructor calls itself, so registration
//! succeeds whether or not that call is present, and no test could tell the
//! difference. Under `mock_all_auths`, though, the environment still records
//! which authorizations the constructor demanded. Asserting that record pins
//! the control: remove the `require_auth` and this test fails.

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

use oracle_adapter::{Asset, OracleAdapterContract};

#[test]
fn test_constructor_requires_the_admin_to_authorize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));

    let id = env.register(OracleAdapterContract, (&admin, &base, &14_u32, &300_u32));

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id,
                    Symbol::new(&env, "__constructor"),
                    (admin, base, 14_u32, 300_u32).into_val(&env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}
