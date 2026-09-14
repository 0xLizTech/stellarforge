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
    Address, Bytes, Env, IntoVal, String, Symbol,
};

use rwa_asset::{AssetMetadata, RwaAssetContract};

fn metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "NYC Real Estate Fund I"),
        symbol: String::from_str(env, "REIT-NYC-001"),
        decimals: 7,
        asset_class: String::from_str(env, "real_estate"),
        legal_doc_hash: Bytes::from_array(env, &[0u8; 32]),
        max_supply: 0,
    }
}

#[test]
fn test_constructor_requires_the_admin_to_authorize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let id = env.register(RwaAssetContract, (&admin, &metadata(&env)));

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id,
                    Symbol::new(&env, "__constructor"),
                    (admin, metadata(&env)).into_val(&env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );
}
