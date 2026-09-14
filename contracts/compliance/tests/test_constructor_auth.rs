#![cfg(test)]

//! IR-17. The constructor's `admin.require_auth()` is what stops a deployer
//! naming an admin who never agreed to the role.
//!
//! `Env::register` authorizes constructor calls itself, so registration
//! succeeds whether or not that call is present, and no test could tell the
//! difference. Under `mock_all_auths`, though, the environment still records
//! which authorizations the constructor demanded. Asserting that record pins
//! the control: remove the `require_auth` and this test fails.
//!
//! It shows the contract asks for the admin's signature. That the network then
//! refuses a deploy lacking it is the host's job, and not something a unit test
//! of this contract can show.

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

use compliance::ComplianceContract;

#[test]
fn test_constructor_requires_the_admin_to_authorize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let id = env.register(ComplianceContract, (&admin,));

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id,
                    Symbol::new(&env, "__constructor"),
                    (admin,).into_val(&env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}
