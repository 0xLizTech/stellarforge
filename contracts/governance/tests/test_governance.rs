#![cfg(test)]

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger,
    },
    Address, Bytes, Env, String,
};

use governance::{
    DataKey, GovernanceContract, GovernanceContractClient, GovernanceError, ProposalStatus,
};
use stellarforge_common::storage::INSTANCE_BUMP_AMOUNT;

const VOTING_PERIOD: u32 = 1_000;

struct Harness<'a> {
    env: Env,
    admin: Address,
    contract_id: Address,
    client: GovernanceContractClient<'a>,
}

impl Harness<'_> {
    /// Creates an active proposal and returns its id.
    fn propose(&self) -> u64 {
        self.propose_for(VOTING_PERIOD)
    }

    fn propose_for(&self, voting_period: u32) -> u64 {
        let proposer = Address::generate(&self.env);
        self.client.propose(
            &proposer,
            &String::from_str(&self.env, "Raise the protocol fee"),
            &Bytes::from_array(&self.env, &[7u8; 32]),
            &voting_period,
        )
    }

    fn advance_past_deadline(&self) {
        self.env
            .ledger()
            .with_mut(|li| li.sequence_number += VOTING_PERIOD + 1);
    }

    fn ttl_of(&self, key: &DataKey) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().persistent().get_ttl(key)
        })
    }

    fn instance_ttl(&self) -> u32 {
        self.env.as_contract(&self.contract_id, || {
            self.env.storage().instance().get_ttl()
        })
    }

    /// The network's maximum entry TTL, which is what write paths extend to.
    fn max_ttl(&self) -> u32 {
        self.env
            .as_contract(&self.contract_id, || self.env.storage().max_ttl())
    }
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(GovernanceContract, (&admin,));
    let client = GovernanceContractClient::new(&env, &contract_id);

    Harness {
        env,
        admin,
        contract_id,
        client,
    }
}

// ─── Initialisation ────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin_and_zero_count() {
    let h = setup();
    assert_eq!(h.client.admin(), h.admin);
    assert_eq!(h.client.proposal_count(), 0);
}

// Four tests covered states that only a separate `initialize` entry point could
// produce: initializing twice, and reaching `admin` or `propose` beforehand.
// A constructor runs inside the deploy transaction, so none of them exist now.
//
// The id-reuse hazard the last of them guarded is worth recording, because the
// constructor is what closes it rather than any check: `initialize` reset the
// proposal counter to zero, so a proposal made before it ran had its id issued
// a second time — the later proposal overwrote the earlier, while vote records
// keyed to that id survived and barred those voters from the new one. Nothing
// resets the counter now. The test below keeps the observable half of that
// property.

#[test]
fn test_first_proposal_on_a_fresh_deploy_owns_id_one() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let client = GovernanceContractClient::new(&env, &env.register(GovernanceContract, (&admin,)));

    assert_eq!(client.proposal_count(), 0);

    let id = client.propose(
        &Address::generate(&env),
        &String::from_str(&env, "real"),
        &Bytes::from_array(&env, &[1u8; 32]),
        &VOTING_PERIOD,
    );

    // No stale proposal underneath this id, and nobody silently barred from
    // voting on it by a vote record left over from an earlier occupant.
    assert_eq!(id, 1);
    assert_eq!(
        client.get_proposal(&id).unwrap().title,
        String::from_str(&env, "real")
    );
    assert!(!client.has_voted(&id, &voter));
    client.vote(&voter, &id, &true, &100);
    assert_eq!(client.get_proposal(&id).unwrap().votes_for, 100);
}

// ─── Proposals ─────────────────────────────────────────────────────────────

#[test]
fn test_propose_stores_the_proposal() {
    let h = setup();
    let id = h.propose();

    let p = h.client.get_proposal(&id).unwrap();
    assert_eq!(p.id, id);
    assert_eq!(p.votes_for, 0);
    assert_eq!(p.votes_against, 0);
    assert_eq!(p.status, ProposalStatus::Active);
    assert_eq!(h.client.proposal_count(), 1);
}

#[test]
fn test_proposal_ids_are_sequential() {
    let h = setup();
    assert_eq!(h.propose(), 1);
    assert_eq!(h.propose(), 2);
    assert_eq!(h.propose(), 3);
    assert_eq!(h.client.proposal_count(), 3);
}

#[test]
fn test_get_unknown_proposal_returns_none() {
    let h = setup();
    assert!(h.client.get_proposal(&42).is_none());
}

#[test]
fn test_voting_period_that_would_overflow_the_sequence_is_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);

    // The test ledger starts at sequence 0, where nothing can overflow.
    h.env.ledger().with_mut(|li| li.sequence_number = 100);

    // Wrapping would put the deadline in the past and close voting on a
    // proposal the moment it was created.
    let res = h.client.try_propose(
        &proposer,
        &String::from_str(&h.env, "overflow"),
        &Bytes::from_array(&h.env, &[0u8; 32]),
        &u32::MAX,
    );
    assert_eq!(res, Err(Ok(GovernanceError::Overflow.into())));
}

// ─── Voting ────────────────────────────────────────────────────────────────

#[test]
fn test_vote_tallies_support_and_opposition() {
    let h = setup();
    let id = h.propose();

    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.client.vote(&Address::generate(&h.env), &id, &true, &50);
    h.client.vote(&Address::generate(&h.env), &id, &false, &30);

    let p = h.client.get_proposal(&id).unwrap();
    assert_eq!(p.votes_for, 150);
    assert_eq!(p.votes_against, 30);
}

#[test]
fn test_double_voting_is_rejected() {
    let h = setup();
    let id = h.propose();
    let voter = Address::generate(&h.env);

    h.client.vote(&voter, &id, &true, &100);
    assert!(h.client.has_voted(&id, &voter));

    let res = h.client.try_vote(&voter, &id, &false, &100);
    assert_eq!(res, Err(Ok(GovernanceError::AlreadyVoted.into())));

    // The rejected second vote must not have moved the tally.
    let p = h.client.get_proposal(&id).unwrap();
    assert_eq!(p.votes_for, 100);
    assert_eq!(p.votes_against, 0);
}

#[test]
fn test_a_voter_is_only_blocked_on_the_proposal_they_voted_on() {
    let h = setup();
    let first = h.propose();
    let second = h.propose();
    let voter = Address::generate(&h.env);

    h.client.vote(&voter, &first, &true, &100);
    assert!(!h.client.has_voted(&second, &voter));
    h.client.vote(&voter, &second, &true, &100);

    assert_eq!(h.client.get_proposal(&second).unwrap().votes_for, 100);
}

#[test]
fn test_non_positive_weight_is_rejected() {
    let h = setup();
    let id = h.propose();
    let voter = Address::generate(&h.env);

    let expected = Err(Ok(GovernanceError::InvalidWeight.into()));
    assert_eq!(h.client.try_vote(&voter, &id, &true, &0), expected);
    assert_eq!(h.client.try_vote(&voter, &id, &true, &-1), expected);
}

#[test]
fn test_vote_on_unknown_proposal_is_rejected() {
    let h = setup();
    let res = h
        .client
        .try_vote(&Address::generate(&h.env), &99, &true, &100);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotFound.into())));
}

#[test]
fn test_voting_after_the_deadline_is_rejected() {
    let h = setup();
    let id = h.propose();
    h.advance_past_deadline();

    let res = h
        .client
        .try_vote(&Address::generate(&h.env), &id, &true, &100);
    assert_eq!(res, Err(Ok(GovernanceError::VotingClosed.into())));
}

#[test]
fn test_voting_on_the_deadline_ledger_is_still_open() {
    let h = setup();
    let id = h.propose();

    // The deadline ledger itself is inclusive.
    h.env
        .ledger()
        .with_mut(|li| li.sequence_number += VOTING_PERIOD);
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);

    assert_eq!(h.client.get_proposal(&id).unwrap().votes_for, 100);
}

#[test]
fn test_vote_tally_overflow_is_rejected() {
    let h = setup();
    let id = h.propose();

    h.client
        .vote(&Address::generate(&h.env), &id, &true, &i128::MAX);
    let res = h
        .client
        .try_vote(&Address::generate(&h.env), &id, &true, &1);
    assert_eq!(res, Err(Ok(GovernanceError::Overflow.into())));
}

#[test]
fn test_voting_on_a_finalized_proposal_is_rejected() {
    let h = setup();
    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.advance_past_deadline();
    h.client.finalize(&id);

    let res = h
        .client
        .try_vote(&Address::generate(&h.env), &id, &true, &100);
    // Status is checked before the deadline, so a finalised proposal reports
    // that it is closed to votes rather than that time ran out.
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotActive.into())));
}

// ─── Finalisation ──────────────────────────────────────────────────────────

#[test]
fn test_finalize_passes_a_supported_proposal() {
    let h = setup();
    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.client.vote(&Address::generate(&h.env), &id, &false, &99);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Passed
    );
}

#[test]
fn test_finalize_rejects_an_opposed_proposal() {
    let h = setup();
    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &10);
    h.client.vote(&Address::generate(&h.env), &id, &false, &11);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
}

#[test]
fn test_a_tie_is_rejected_not_passed() {
    let h = setup();
    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.client.vote(&Address::generate(&h.env), &id, &false, &100);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
}

#[test]
fn test_an_unvoted_proposal_is_rejected() {
    let h = setup();
    let id = h.propose();
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
}

#[test]
fn test_finalize_before_the_deadline_is_rejected() {
    let h = setup();
    let id = h.propose();

    assert_eq!(
        h.client.try_finalize(&id),
        Err(Ok(GovernanceError::VotingNotClosed.into()))
    );
}

#[test]
fn test_double_finalize_is_rejected() {
    let h = setup();
    let id = h.propose();
    h.advance_past_deadline();
    h.client.finalize(&id);

    assert_eq!(
        h.client.try_finalize(&id),
        Err(Ok(GovernanceError::ProposalNotActive.into()))
    );
}

#[test]
fn test_finalize_unknown_proposal_is_rejected() {
    let h = setup();
    assert_eq!(
        h.client.try_finalize(&99),
        Err(Ok(GovernanceError::ProposalNotFound.into()))
    );
}

// ─── Storage lifetime ──────────────────────────────────────────────────────

// Since protocol 23 the test environment auto-restores archived entries, so
// advancing the ledger past an expiry proves nothing. These tests assert the
// TTL itself, which is what the extension is for.

/// Longer than PERSISTENT_REFRESH_INTERVAL, so a write path actually reissues
/// the extension rather than finding the entry still fresh enough.
const IDLE: u32 = 600_000;

#[test]
fn test_propose_extends_to_the_network_maximum() {
    let h = setup();
    let id = h.propose();

    assert_eq!(h.ttl_of(&DataKey::Proposal(id)), h.max_ttl());
    assert_eq!(h.instance_ttl(), INSTANCE_BUMP_AMOUNT);
}

#[test]
fn test_vote_extends_the_proposal_and_the_vote_record() {
    let h = setup();
    // Long enough that voting is still open after the idle period below.
    let id = h.propose_for(IDLE + 1_000);
    let voter = Address::generate(&h.env);

    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);
    h.client.vote(&voter, &id, &true, &100);

    assert_eq!(h.ttl_of(&DataKey::Proposal(id)), h.max_ttl());
    assert_eq!(h.ttl_of(&DataKey::Vote(id, voter)), h.max_ttl());
}

#[test]
fn test_reading_a_finalized_proposal_does_not_extend_it() {
    let h = setup();
    let id = h.propose();
    h.advance_past_deadline();
    h.client.finalize(&id);

    // Measured rather than assumed: finalisation only reissues the extension
    // if the entry has decayed past the refresh threshold, which it has not.
    let before = h.ttl_of(&DataKey::Proposal(id));
    h.env.ledger().with_mut(|li| li.sequence_number += IDLE);

    // Reading history is a pure query. The network-maximum bump applied on the
    // write paths is what keeps the record queryable.
    h.client.get_proposal(&id);
    h.client.has_voted(&id, &Address::generate(&h.env));

    assert_eq!(h.ttl_of(&DataKey::Proposal(id)), before - IDLE);
}
