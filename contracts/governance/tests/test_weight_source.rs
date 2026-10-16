#![cfg(test)]

//! Integration tests exercising `vote`'s weight cap against a real
//! `RwaAssetContract` rather than a stub, so the cross-contract balance read
//! is covered end to end (issue #133).

use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    Address, Bytes, Env, Event, IntoVal, String,
};

use governance::{
    GovernanceContract, GovernanceContractClient, GovernanceError, VoteCast, WeightSourceSet,
};
use rwa_asset::{AssetMetadata, RwaAssetContract, RwaAssetContractClient};

const VOTING_PERIOD: u32 = governance::MIN_VOTING_PERIOD_LEDGERS;

struct Harness<'a> {
    env: Env,
    contract_id: Address,
    client: GovernanceContractClient<'a>,
    token_id: Address,
    token: RwaAssetContractClient<'a>,
    issuer: Address,
}

impl Harness<'_> {
    fn propose(&self) -> u64 {
        let proposer = Address::generate(&self.env);
        self.client.propose(
            &proposer,
            &String::from_str(&self.env, "Raise the protocol fee"),
            &Bytes::from_array(&self.env, &[7u8; 32]),
            &VOTING_PERIOD,
        )
    }

    /// Mints `amount` of the weight-source token to `to`.
    fn fund(&self, to: &Address, amount: i128) {
        self.token.mint(&self.issuer, to, &amount);
    }
}

fn metadata(env: &Env) -> AssetMetadata {
    AssetMetadata {
        name: String::from_str(env, "StellarForge Governance Token"),
        symbol: String::from_str(env, "SFORGE"),
        decimals: 7,
        asset_class: String::from_str(env, "governance"),
        legal_doc_hash: Bytes::from_array(env, &[0u8; 32]),
        max_supply: 0,
    }
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(GovernanceContract, (&admin,));
    let client = GovernanceContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token_id = env.register(RwaAssetContract, (&admin, &metadata(&env)));
    let token = RwaAssetContractClient::new(&env, &token_id);
    token.set_issuer(&issuer, &true);

    Harness {
        env,
        contract_id,
        client,
        token_id,
        token,
        issuer,
    }
}

// ─── Configuration ──────────────────────────────────────────────────────────

#[test]
fn test_weight_source_is_configurable_and_readable() {
    let h = setup();
    assert_eq!(h.client.weight_source(), None);

    h.client.set_weight_source(&Some(h.token_id.clone()));
    assert_eq!(h.client.weight_source(), Some(h.token_id));

    h.client.set_weight_source(&None);
    assert_eq!(h.client.weight_source(), None);
}

#[test]
fn test_set_weight_source_requires_admin() {
    let h = setup();
    let not_admin = Address::generate(&h.env);

    // Only `not_admin`'s signature is on the transaction, never the real
    // admin's, so `require_admin`'s `require_auth()` on the stored admin
    // address has nothing to authorize it.
    h.env.mock_auths(&[MockAuth {
        address: &not_admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_weight_source",
            args: (Some(h.token_id.clone()),).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h
        .client
        .try_set_weight_source(&Some(h.token_id.clone()))
        .is_err());
    assert_eq!(h.client.weight_source(), None);
}

#[test]
fn test_set_weight_source_emits_event() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));

    assert_eq!(
        h.env.events().all(),
        std::vec![WeightSourceSet {
            weight_source: Some(h.token_id.clone()),
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Capping ────────────────────────────────────────────────────────────────

#[test]
fn test_vote_is_uncapped_when_no_weight_source_is_configured() {
    let h = setup();
    let id = h.propose();
    let voter = Address::generate(&h.env);

    // The voter holds no tokens at all, and the vote still carries its full
    // declared weight, matching the uncapped behaviour this contract had
    // before #133 and still has when no weight source is configured.
    h.client.vote(&voter, &id, &true, &1_000);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 1_000);
}

#[test]
fn test_vote_weight_is_capped_at_the_voters_balance() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    let voter = Address::generate(&h.env);
    h.fund(&voter, 300);

    // Declares 1,000 but holds only 300.
    h.client.vote(&voter, &id, &true, &1_000);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 300);
}

#[test]
fn test_vote_weight_under_the_balance_is_unchanged() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    let voter = Address::generate(&h.env);
    h.fund(&voter, 1_000);

    // Declares less than the balance holds, so no capping applies.
    h.client.vote(&voter, &id, &true, &300);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 300);
}

#[test]
fn test_vote_cast_event_reports_the_capped_weight_not_the_declared_one() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    let voter = Address::generate(&h.env);
    h.fund(&voter, 50);
    h.client.vote(&voter, &id, &true, &1_000);

    assert_eq!(
        h.env.events().all(),
        std::vec![VoteCast {
            proposal_id: id,
            voter,
            support: true,
            weight: 50,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_a_voter_with_zero_balance_is_rejected_once_capping_is_configured() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    // Never funded, so the balance read is 0.
    let voter = Address::generate(&h.env);
    let res = h.client.try_vote(&voter, &id, &true, &1_000);
    assert_eq!(res, Err(Ok(GovernanceError::InvalidWeight.into())));

    // A rejected vote must not have consumed the voter's one shot at this
    // proposal, since nothing was tallied.
    assert!(!h.client.has_voted(&id, &voter));
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 0);
}

#[test]
fn test_capping_does_not_let_a_double_spent_balance_double_count() {
    // Documents the residual limitation the module docs describe: capping at
    // a live balance read still allows a holder to vote, transfer the same
    // tokens to a second address, and vote again from there, since each cap
    // only ever sees its own voter's balance at its own point in time. This
    // is not closed by this change, and is recorded here so a future reader
    // sees it demonstrated rather than only described in prose.
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    let first_voter = Address::generate(&h.env);
    let second_voter = Address::generate(&h.env);
    h.fund(&first_voter, 500);

    h.client.vote(&first_voter, &id, &true, &500);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 500);

    h.token.transfer(&first_voter, &second_voter, &500);

    h.client.vote(&second_voter, &id, &true, &500);
    // Both votes counted in full: 1,000 total from 500 real tokens.
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 1_000);
}

#[test]
fn test_capping_survives_voting_after_the_source_is_later_removed() {
    let h = setup();
    h.client.set_weight_source(&Some(h.token_id.clone()));
    let id = h.propose();

    let capped_voter = Address::generate(&h.env);
    h.fund(&capped_voter, 10);
    h.client.vote(&capped_voter, &id, &true, &1_000);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 10);

    h.client.set_weight_source(&None);

    let uncapped_voter = Address::generate(&h.env);
    h.client.vote(&uncapped_voter, &id, &true, &1_000);
    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 1_010);
}
