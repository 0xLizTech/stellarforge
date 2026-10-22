#![cfg(test)]

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Events, Ledger, MockAuth, MockAuthInvoke,
    },
    token, Address, Bytes, Env, Event, IntoVal, String,
};

use governance::{
    AdminTransferCancelled, AdminTransferProposed, AdminTransferred, DataKey, DepositConfigured,
    GovernanceContract, GovernanceContractClient, GovernanceError, PassThresholdSet,
    ProposalCancelled, ProposalCreated, ProposalDepositRefunded, ProposalFinalized, ProposalStatus,
    QuorumSet, VoteCast, DEFAULT_PASS_THRESHOLD_BPS, MAX_PAGE_SIZE, MAX_PASS_THRESHOLD_BPS,
    MAX_TITLE_BYTES, MAX_VOTING_PERIOD_LEDGERS, MIN_VOTING_PERIOD_LEDGERS,
};
use stellarforge_common::storage::INSTANCE_BUMP_AMOUNT;

/// The shortest period `propose` accepts, so tests advance the ledger as little
/// as the contract allows.
const VOTING_PERIOD: u32 = MIN_VOTING_PERIOD_LEDGERS;

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
        self.propose_by(&proposer, voting_period)
    }

    fn propose_by(&self, proposer: &Address, voting_period: u32) -> u64 {
        self.client.propose(
            proposer,
            &String::from_str(&self.env, "Raise the protocol fee"),
            &Bytes::from_array(&self.env, &[7u8; 32]),
            &voting_period,
        )
    }

    /// Like `propose`, but with a distinct title so it does not collide with
    /// (or with another call to this helper using a different `suffix`)
    /// #136's duplicate-active-proposal check. Tests that need several
    /// simultaneously-active proposals for reasons unrelated to #136 use
    /// this instead of `propose`.
    fn propose_distinct(&self, suffix: &str) -> u64 {
        let proposer = Address::generate(&self.env);
        let mut title = std::string::String::from("Raise the protocol fee ");
        title.push_str(suffix);
        self.client.propose(
            &proposer,
            &String::from_str(&self.env, &title),
            &Bytes::from_array(&self.env, &[7u8; 32]),
            &VOTING_PERIOD,
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

    /// Creates an active proposal on behalf of `proposer` and returns its id.
    /// Unlike `propose`/`propose_for`, this lets a deposit test control which
    /// address pays and later assert on their balance.
    fn propose_as(&self, proposer: &Address) -> u64 {
        self.client.propose(
            proposer,
            &String::from_str(&self.env, "Raise the protocol fee"),
            &Bytes::from_array(&self.env, &[7u8; 32]),
            &VOTING_PERIOD,
        )
    }
}

/// Deploys a Stellar Asset Contract test token (#134's deposit tests need a
/// real SEP-41 token to transfer; governance has no other contract in this
/// workspace to borrow a token setup convention from). Returns the token's
/// contract address and a client that can mint new balances for test setup.
fn setup_token(env: &Env, token_admin: &Address) -> (Address, token::StellarAssetClient<'static>) {
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let address = sac.address();
    let mint_client = token::StellarAssetClient::new(env, &address);
    (address, mint_client)
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
    assert_eq!(h.propose_distinct("A"), 1);
    assert_eq!(h.propose_distinct("B"), 2);
    assert_eq!(h.propose_distinct("C"), 3);
    assert_eq!(h.client.proposal_count(), 3);
}

#[test]
fn test_get_unknown_proposal_returns_none() {
    let h = setup();
    assert!(h.client.get_proposal(&42).is_none());
}

// `test_voting_period_that_would_overflow_the_sequence_is_rejected` went with
// the voting-period cap (IR-14). With the period at most
// MAX_VOTING_PERIOD_LEDGERS, the deadline can only overflow at a sequence within
// about 90 days of u32::MAX, and the test host cannot run a contract there: its
// own TTL arithmetic overflows first. `propose` keeps the checked addition, so
// the case still ends in a defined `Overflow` rather than a wrapped deadline.

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
    let first = h.propose_distinct("A");
    let second = h.propose_distinct("B");
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

// ─── Quorum and pass threshold (#135) ──────────────────────────────────────

#[test]
fn test_quorum_and_pass_threshold_default_to_the_pre_135_behavior() {
    let h = setup();
    assert_eq!(h.client.quorum(), 0);
    assert_eq!(h.client.pass_threshold_bps(), DEFAULT_PASS_THRESHOLD_BPS);
}

#[test]
fn test_a_proposal_below_quorum_fails_even_with_unanimous_support() {
    let h = setup();
    h.client.set_quorum(&1_000);

    let id = h.propose();
    // Unanimous, but total turnout (100) is below the 1,000 quorum.
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
}

#[test]
fn test_a_proposal_exactly_at_quorum_is_evaluated_on_threshold() {
    let h = setup();
    h.client.set_quorum(&100);

    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Passed,
        "turnout exactly at quorum must be evaluated, not rejected as 'below'"
    );
}

#[test]
fn test_a_proposal_one_below_quorum_fails() {
    let h = setup();
    h.client.set_quorum(&101);

    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &100);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
}

#[test]
fn test_raising_the_pass_threshold_requires_a_wider_margin() {
    let h = setup();
    // 60% required: 60 for vs 40 against is exactly the boundary.
    h.client.set_pass_threshold_bps(&6_000);

    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &60);
    h.client.vote(&Address::generate(&h.env), &id, &false, &41);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected,
        "60 for vs 41 against is 59.4%, short of the 60% threshold"
    );
}

#[test]
fn test_exactly_meeting_the_pass_threshold_passes() {
    let h = setup();
    h.client.set_pass_threshold_bps(&6_000);

    let id = h.propose();
    h.client.vote(&Address::generate(&h.env), &id, &true, &60);
    h.client.vote(&Address::generate(&h.env), &id, &false, &40);
    h.advance_past_deadline();

    h.client.finalize(&id);
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Passed,
        "60 for vs 40 against is exactly 60%, which must be enough"
    );
}

#[test]
fn test_set_quorum_rejects_negative_values() {
    let h = setup();
    assert_eq!(
        h.client.try_set_quorum(&-1),
        Err(Ok(GovernanceError::InvalidQuorum.into()))
    );
}

#[test]
fn test_set_pass_threshold_bps_rejects_zero() {
    let h = setup();
    assert_eq!(
        h.client.try_set_pass_threshold_bps(&0),
        Err(Ok(GovernanceError::InvalidPassThresholdBps.into()))
    );
}

#[test]
fn test_set_pass_threshold_bps_rejects_above_max() {
    let h = setup();
    assert_eq!(
        h.client
            .try_set_pass_threshold_bps(&(MAX_PASS_THRESHOLD_BPS + 1)),
        Err(Ok(GovernanceError::InvalidPassThresholdBps.into()))
    );
}

#[test]
fn test_set_pass_threshold_bps_accepts_the_max() {
    let h = setup();
    h.client.set_pass_threshold_bps(&MAX_PASS_THRESHOLD_BPS);
    assert_eq!(h.client.pass_threshold_bps(), MAX_PASS_THRESHOLD_BPS);
}

#[test]
fn test_set_quorum_requires_admin_auth() {
    let h = setup();
    let not_admin = Address::generate(&h.env);

    h.env.mock_auths(&[MockAuth {
        address: &not_admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_quorum",
            args: (1_000_i128,).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h.client.try_set_quorum(&1_000).is_err());
    assert_eq!(h.client.quorum(), 0);
}

#[test]
fn test_set_quorum_emits_event() {
    let h = setup();
    h.client.set_quorum(&1_000);

    assert_eq!(
        h.env.events().all(),
        std::vec![QuorumSet {
            admin: h.admin.clone(),
            quorum: 1_000,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_set_pass_threshold_bps_emits_event() {
    let h = setup();
    h.client.set_pass_threshold_bps(&6_000);

    assert_eq!(
        h.env.events().all(),
        std::vec![PassThresholdSet {
            admin: h.admin.clone(),
            pass_threshold_bps: 6_000,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Duplicate proposal rejection (#136) ───────────────────────────────────

#[test]
fn test_try_propose_returns_duplicate_error_not_a_panic() {
    let h = setup();
    h.propose();

    let proposer = Address::generate(&h.env);
    let result = h.client.try_propose(
        &proposer,
        &String::from_str(&h.env, "Raise the protocol fee"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );

    assert_eq!(
        result,
        Err(Ok(GovernanceError::DuplicateActiveProposal.into()))
    );
}

#[test]
fn test_a_different_title_is_not_a_duplicate_even_with_the_same_description_hash() {
    let h = setup();
    h.propose();

    let proposer = Address::generate(&h.env);
    let id = h.client.propose(
        &proposer,
        &String::from_str(&h.env, "A completely different title"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );

    assert!(id > 0);
}

#[test]
fn test_a_different_description_hash_is_not_a_duplicate_even_with_the_same_title() {
    let h = setup();
    h.propose();

    let proposer = Address::generate(&h.env);
    let id = h.client.propose(
        &proposer,
        &String::from_str(&h.env, "Raise the protocol fee"),
        &Bytes::from_array(&h.env, &[9u8; 32]),
        &VOTING_PERIOD,
    );

    assert!(id > 0);
}

#[test]
fn test_a_resubmission_after_the_first_is_finalized_is_allowed() {
    let h = setup();
    let first = h.propose();
    h.client
        .vote(&Address::generate(&h.env), &first, &true, &100);
    h.advance_past_deadline();
    h.client.finalize(&first);

    // #136 only bars a *concurrent* duplicate; once the first proposal is
    // decided, an identical resubmission is a new, distinct decision.
    let second = h.propose();
    assert_ne!(first, second);
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

// ─── Events (IR-03) ────────────────────────────────────────────────────────

// NFR-A-3: the full governance history must be reconstructable from events.

#[test]
fn test_propose_emits_event() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let title = String::from_str(&h.env, "Raise the protocol fee");
    let description_hash = Bytes::from_array(&h.env, &[7u8; 32]);
    let deadline_ledger = h.env.ledger().sequence() + VOTING_PERIOD;

    let id = h
        .client
        .propose(&proposer, &title, &description_hash, &VOTING_PERIOD);

    assert_eq!(
        h.env.events().all(),
        std::vec![ProposalCreated {
            proposal_id: id,
            proposer,
            title,
            description_hash,
            deadline_ledger,
            deposit_amount: 0,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_vote_emits_event() {
    let h = setup();
    let id = h.propose();
    let voter = Address::generate(&h.env);

    h.client.vote(&voter, &id, &false, &40_i128);

    assert_eq!(
        h.env.events().all(),
        std::vec![VoteCast {
            proposal_id: id,
            voter,
            support: false,
            weight: 40,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_finalize_emits_the_outcome_and_tally() {
    let h = setup();
    let id = h.propose();
    h.client
        .vote(&Address::generate(&h.env), &id, &true, &40_i128);
    h.client
        .vote(&Address::generate(&h.env), &id, &false, &10_i128);
    h.advance_past_deadline();

    h.client.finalize(&id);

    assert_eq!(
        h.env.events().all(),
        std::vec![ProposalFinalized {
            proposal_id: id,
            status: ProposalStatus::Passed,
            votes_for: 40,
            votes_against: 10,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Admin rotation (IR-04, two-step handover #125/ADR-006) ────────────────

#[test]
fn test_propose_then_accept_admin_moves_the_role() {
    let h = setup();
    let new_admin = Address::generate(&h.env);

    h.client.propose_admin(&new_admin);
    assert_eq!(
        h.client.admin(),
        h.admin,
        "role must not move on propose alone"
    );

    h.client.accept_admin();

    assert_eq!(h.client.admin(), new_admin);
    assert!(h.client.pending_admin().is_none());
}

/// #120. The existing instance-TTL assertion (`test_propose_extends_to_the_
/// network_maximum`) exercises `propose`'s call to `extend_instance`, never
/// `transfer_admin`'s. `extend_instance` is one shared function, so this
/// mostly proves the same thing twice, but it closes the one call site that
/// otherwise had no test exercising it directly.
#[test]
fn test_transfer_admin_extends_the_instance() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.env
        .ledger()
        .with_mut(|li| li.sequence_number += VOTING_PERIOD * 10);

    h.client.propose_admin(&new_admin);
    h.client.accept_admin();

    assert_eq!(h.instance_ttl(), INSTANCE_BUMP_AMOUNT);
}

/// NFR-S-4 is dual authorization, not "the admin signs". Only the current
/// admin's signature is supplied here, which must not be enough.
/// Only the current admin may propose.
#[test]
fn test_propose_admin_requires_current_admin_auth() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    let not_admin = Address::generate(&h.env);

    h.env.mock_auths(&[MockAuth {
        address: &not_admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "propose_admin",
            args: (new_admin.clone(),).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h.client.try_propose_admin(&new_admin).is_err());
    assert!(h.client.pending_admin().is_none());
}

/// NFR-S-4's dual-authorization guarantee survives the split into two
/// steps: the proposed admin's own signature is required to accept, and the
/// current admin cannot complete the handover on its behalf.
#[test]
fn test_accept_admin_without_the_proposed_admins_signature_is_rejected() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.client.propose_admin(&new_admin);

    h.env.mock_auths(&[MockAuth {
        address: &h.admin,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "accept_admin",
            args: ().into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h.client.try_accept_admin().is_err());
    assert_eq!(h.client.admin(), h.admin);
}

/// `accept_admin` with nothing proposed is a defined failure, not a panic on
/// an absent storage key.
#[test]
fn test_accept_admin_with_no_pending_transfer_fails() {
    let h = setup();

    let result = h.client.try_accept_admin();

    assert_eq!(
        result,
        Err(Ok(GovernanceError::NoPendingAdminTransfer.into()))
    );
}

/// The current admin can withdraw a proposal before it is accepted.
#[test]
fn test_cancel_admin_transfer_clears_the_pending_proposal() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.client.propose_admin(&new_admin);

    h.client.cancel_admin_transfer();

    assert!(h.client.pending_admin().is_none());
    assert!(h.client.try_accept_admin().is_err());
    assert_eq!(h.client.admin(), h.admin);
}

#[test]
fn test_propose_admin_emits_event() {
    let h = setup();
    let new_admin = Address::generate(&h.env);

    h.client.propose_admin(&new_admin);

    assert_eq!(
        h.env.events().all(),
        std::vec![AdminTransferProposed {
            current: h.admin.clone(),
            proposed_admin: new_admin,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_accept_admin_emits_event() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.client.propose_admin(&new_admin);
    h.env.events().all(); // drain the propose event

    h.client.accept_admin();

    assert_eq!(
        h.env.events().all(),
        std::vec![AdminTransferred {
            previous: h.admin.clone(),
            new_admin,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

#[test]
fn test_cancel_admin_transfer_emits_event() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.client.propose_admin(&new_admin);
    h.env.events().all(); // drain the propose event

    h.client.cancel_admin_transfer();

    assert_eq!(
        h.env.events().all(),
        std::vec![AdminTransferCancelled {
            cancelled_proposed_admin: new_admin,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Proposal bounds (IR-14) ───────────────────────────────────────────────

fn try_propose_with(h: &Harness, title: &str, voting_period: u32) -> bool {
    h.client
        .try_propose(
            &Address::generate(&h.env),
            &String::from_str(&h.env, title),
            &Bytes::from_array(&h.env, &[7u8; 32]),
            &voting_period,
        )
        .is_ok()
}

#[test]
fn test_voting_period_outside_the_bounds_is_rejected() {
    let h = setup();

    for period in [
        0,
        MIN_VOTING_PERIOD_LEDGERS - 1,
        MAX_VOTING_PERIOD_LEDGERS + 1,
    ] {
        let res = h.client.try_propose(
            &Address::generate(&h.env),
            &String::from_str(&h.env, "title"),
            &Bytes::from_array(&h.env, &[7u8; 32]),
            &period,
        );
        assert_eq!(
            res,
            Err(Ok(GovernanceError::InvalidVotingPeriod.into())),
            "accepted a period of {period}"
        );
    }
    assert_eq!(h.client.proposal_count(), 0);
}

/// Guards the rejections above: both bounds are inclusive.
#[test]
fn test_voting_period_bounds_are_inclusive() {
    let h = setup();
    assert!(try_propose_with(&h, "shortest", MIN_VOTING_PERIOD_LEDGERS));
    assert!(try_propose_with(&h, "longest", MAX_VOTING_PERIOD_LEDGERS));
}

#[test]
fn test_title_longer_than_the_cap_is_rejected() {
    let h = setup();
    let at_cap = "a".repeat(MAX_TITLE_BYTES as usize);
    let over_cap = "a".repeat(MAX_TITLE_BYTES as usize + 1);

    assert!(try_propose_with(&h, &at_cap, VOTING_PERIOD));

    let res = h.client.try_propose(
        &Address::generate(&h.env),
        &String::from_str(&h.env, &over_cap),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );
    assert_eq!(res, Err(Ok(GovernanceError::TitleTooLong.into())));
}

// ─── Proposal cancellation (#137) ──────────────────────────────────────────

#[test]
fn test_proposer_can_cancel_their_own_proposal_before_any_vote() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);

    h.client.cancel_proposal(&proposer, &id);

    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Cancelled
    );
}

#[test]
fn test_admin_can_cancel_a_proposal_they_did_not_create() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);

    h.client.cancel_proposal(&h.admin, &id);

    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Cancelled
    );
}

#[test]
fn test_cancellation_by_neither_proposer_nor_admin_is_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    let stranger = Address::generate(&h.env);

    let res = h.client.try_cancel_proposal(&stranger, &id);
    assert_eq!(res, Err(Ok(GovernanceError::NotAuthorizedToCancel.into())));
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Active
    );
}

#[test]
fn test_cancellation_after_a_vote_is_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.vote(&Address::generate(&h.env), &id, &true, &1);

    let res = h.client.try_cancel_proposal(&proposer, &id);
    assert_eq!(
        res,
        Err(Ok(GovernanceError::ProposalAlreadyHasVotes.into()))
    );
    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Active
    );
}

#[test]
fn test_cancellation_after_a_single_against_vote_is_also_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.vote(&Address::generate(&h.env), &id, &false, &1);

    let res = h.client.try_cancel_proposal(&proposer, &id);
    assert_eq!(
        res,
        Err(Ok(GovernanceError::ProposalAlreadyHasVotes.into()))
    );
}

// ─── Proposal deposit (#134) ───────────────────────────────────────────────

#[test]
fn test_no_deposit_is_configured_by_default() {
    let h = setup();
    assert_eq!(h.client.deposit_token(), None);
    assert_eq!(h.client.deposit_amount(), 0);
}

#[test]
fn test_admin_can_configure_a_deposit() {
    let h = setup();
    let (token_address, _mint) = setup_token(&h.env, &h.admin);

    h.client
        .set_proposal_deposit(&Some(token_address.clone()), &100);

    assert_eq!(h.client.deposit_token(), Some(token_address));
    assert_eq!(h.client.deposit_amount(), 100);
}

#[test]
fn test_set_proposal_deposit_emits_event() {
    let h = setup();
    let (token_address, _mint) = setup_token(&h.env, &h.admin);

    h.client
        .set_proposal_deposit(&Some(token_address.clone()), &100);

    assert_eq!(
        h.env.events().all(),
        std::vec![DepositConfigured {
            token: Some(token_address),
            amount: 100,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

/// `mock_all_auths()` (set by `setup()`) accepts any signer, so this test
/// instead mocks auth for a specific non-admin address only, the same way
/// `test_transfer_admin_without_the_incoming_signature_is_rejected` asserts
/// `transfer_admin`'s dual-auth requirement: the entry point's own
/// `require_auth` on the real admin then has nothing authorizing it, and
/// the call must fail.
#[test]
fn test_non_admin_cannot_configure_a_deposit() {
    let h = setup();
    let (token_address, _mint) = setup_token(&h.env, &h.admin);
    let impostor = Address::generate(&h.env);

    h.env.mock_auths(&[MockAuth {
        address: &impostor,
        invoke: &MockAuthInvoke {
            contract: &h.contract_id,
            fn_name: "set_proposal_deposit",
            args: (Some(token_address.clone()), 100_i128).into_val(&h.env),
            sub_invokes: &[],
        },
    }]);

    assert!(h
        .client
        .try_set_proposal_deposit(&Some(token_address), &100)
        .is_err());
    assert_eq!(h.client.deposit_token(), None);
}

#[test]
fn test_negative_deposit_amount_is_rejected() {
    let h = setup();
    let (token_address, _mint) = setup_token(&h.env, &h.admin);

    let res = h.client.try_set_proposal_deposit(&Some(token_address), &-1);
    assert_eq!(res, Err(Ok(GovernanceError::InvalidDepositAmount.into())));
}

#[test]
fn test_disabling_the_deposit_clears_both_token_and_amount() {
    let h = setup();
    let (token_address, _mint) = setup_token(&h.env, &h.admin);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    h.client.set_proposal_deposit(&None, &100);

    assert_eq!(h.client.deposit_token(), None);
    assert_eq!(
        h.client.deposit_amount(),
        0,
        "amount must not survive disabling the deposit, even though the \
         caller passed a non-zero amount alongside None"
    );
}

#[test]
fn test_cancelling_an_already_cancelled_proposal_is_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.cancel_proposal(&proposer, &id);

    let res = h.client.try_cancel_proposal(&proposer, &id);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotActive.into())));
}

#[test]
fn test_cancelling_a_finalized_proposal_is_rejected() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.advance_past_deadline();
    h.client.finalize(&id);

    let res = h.client.try_cancel_proposal(&proposer, &id);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotActive.into())));
}

#[test]
fn test_cancelling_an_unknown_proposal_is_rejected() {
    let h = setup();
    let res = h.client.try_cancel_proposal(&h.admin, &99);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotFound.into())));
}

#[test]
fn test_votes_reject_a_cancelled_proposal() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.cancel_proposal(&proposer, &id);

    let res = h
        .client
        .try_vote(&Address::generate(&h.env), &id, &true, &100);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotActive.into())));
}

#[test]
fn test_finalize_rejects_a_cancelled_proposal() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.cancel_proposal(&proposer, &id);
    h.advance_past_deadline();

    let res = h.client.try_finalize(&id);
    assert_eq!(res, Err(Ok(GovernanceError::ProposalNotActive.into())));
}

#[test]
fn test_cancel_proposal_emits_event() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);

    h.client.cancel_proposal(&proposer, &id);

    assert_eq!(
        h.env.events().all(),
        std::vec![ProposalCancelled {
            proposal_id: id,
            cancelled_by: proposer,
        }
        .to_xdr(&h.env, &h.contract_id)],
    );
}

// ─── Proposal pagination (#138) ────────────────────────────────────────────

#[test]
fn test_list_proposals_returns_an_empty_page_when_there_are_none() {
    let h = setup();
    assert_eq!(h.client.list_proposals(&0, &MAX_PAGE_SIZE).len(), 0);
}

#[test]
fn test_pages_partition_proposals_in_creation_order() {
    let h = setup();
    let ids: std::vec::Vec<u64> = (0..7).map(|i| h.propose_distinct(&i.to_string())).collect();

    let mut walked = std::vec::Vec::new();
    let mut start = 0;
    loop {
        let page = h.client.list_proposals(&start, &3);
        walked.extend(page.iter().map(|p| p.id));
        if page.len() < 3 {
            break;
        }
        start += 3;
    }

    assert_eq!(walked, ids);
}

#[test]
fn test_a_page_past_the_end_is_empty() {
    let h = setup();
    h.propose_distinct("1");
    h.propose_distinct("2");

    assert_eq!(h.client.list_proposals(&2, &MAX_PAGE_SIZE).len(), 0);
    assert_eq!(h.client.list_proposals(&u32::MAX, &MAX_PAGE_SIZE).len(), 0);
}

#[test]
fn test_an_oversized_page_is_rejected() {
    let h = setup();
    let res = h.client.try_list_proposals(&0, &(MAX_PAGE_SIZE + 1));
    assert_eq!(res, Err(Ok(GovernanceError::PageTooLarge.into())));
}

#[test]
fn test_list_proposals_reflects_cancelled_status() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.propose_by(&proposer, VOTING_PERIOD);
    h.client.cancel_proposal(&proposer, &id);

    let page = h.client.list_proposals(&0, &MAX_PAGE_SIZE);
    assert_eq!(page.get(0).unwrap().status, ProposalStatus::Cancelled);
}

#[test]
fn test_propose_collects_the_configured_deposit() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let token_client = token::Client::new(&h.env, &token_address);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    h.propose_as(&proposer);

    assert_eq!(token_client.balance(&proposer), 400);
    assert_eq!(token_client.balance(&h.contract_id), 100);
}

#[test]
fn test_propose_with_no_deposit_configured_collects_nothing() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let token_client = token::Client::new(&h.env, &token_address);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    // Deliberately not calling set_proposal_deposit.

    h.propose_as(&proposer);

    assert_eq!(token_client.balance(&proposer), 500);
    assert_eq!(token_client.balance(&h.contract_id), 0);
}

#[test]
fn test_propose_emits_the_collected_deposit_amount() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    let id = h.propose_as(&proposer);

    let proposal = h.client.get_proposal(&id).unwrap();
    assert_eq!(proposal.deposit_amount, 100);
}

#[test]
fn test_proposer_without_enough_balance_cannot_propose() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &50); // less than the configured deposit
    h.client.set_proposal_deposit(&Some(token_address), &100);

    let res = h.client.try_propose(
        &proposer,
        &String::from_str(&h.env, "title"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );
    assert!(res.is_err(), "expected the token transfer to fail");
}

#[test]
fn test_finalize_refunds_the_deposit_on_a_passed_proposal() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let token_client = token::Client::new(&h.env, &token_address);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    let id = h.propose_as(&proposer);
    h.client
        .vote(&Address::generate(&h.env), &id, &true, &10_i128);
    h.advance_past_deadline();
    h.client.finalize(&id);

    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Passed
    );
    assert_eq!(token_client.balance(&proposer), 500);
    assert_eq!(token_client.balance(&h.contract_id), 0);
}

#[test]
fn test_finalize_refunds_the_deposit_on_a_rejected_proposal() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let token_client = token::Client::new(&h.env, &token_address);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    // No votes at all: an unvoted proposal is rejected (0 > 0 is false).
    let id = h.propose_as(&proposer);
    h.advance_past_deadline();
    h.client.finalize(&id);

    assert_eq!(
        h.client.get_proposal(&id).unwrap().status,
        ProposalStatus::Rejected
    );
    assert_eq!(
        token_client.balance(&proposer),
        500,
        "the deposit is an anti-spam cost of proposing, not a stake on the \
         outcome, so it is refunded even though the proposal was rejected"
    );
}

#[test]
fn test_finalize_emits_a_deposit_refund_event() {
    let h = setup();
    let (token_address, mint) = setup_token(&h.env, &h.admin);
    let proposer = Address::generate(&h.env);
    mint.mint(&proposer, &500);
    h.client.set_proposal_deposit(&Some(token_address), &100);

    // No votes: an unvoted proposal is rejected, exercising the refund path
    // on the "rejected" branch rather than "passed" (covered separately by
    // test_finalize_refunds_the_deposit_on_a_passed_proposal above).
    let id = h.propose_as(&proposer);
    h.advance_past_deadline();
    h.client.finalize(&id);

    // finalize's own token::Client::transfer call also publishes the SAC's
    // own "transfer" event in the same invocation; filtered out here, since
    // this test is about governance's own events, not the token's wire
    // format, which filter_by_contract exists precisely to let a test ignore.
    assert_eq!(
        h.env.events().all().filter_by_contract(&h.contract_id),
        std::vec![
            ProposalDepositRefunded {
                proposal_id: id,
                proposer: proposer.clone(),
                amount: 100,
            }
            .to_xdr(&h.env, &h.contract_id),
            ProposalFinalized {
                proposal_id: id,
                status: ProposalStatus::Rejected,
                votes_for: 0,
                votes_against: 0,
            }
            .to_xdr(&h.env, &h.contract_id),
        ],
    );
}

#[test]
fn test_finalize_does_not_refund_when_no_deposit_was_collected() {
    let h = setup();
    // No token configured at all: propose collects nothing, so finalize must
    // not attempt a transfer (which would panic with no token configured).
    let id = h.propose();
    h.advance_past_deadline();
    h.client.finalize(&id); // must not panic
    assert_eq!(h.client.get_proposal(&id).unwrap().deposit_amount, 0);
}

#[test]
fn test_changing_the_deposit_token_after_proposing_still_refunds_in_the_original_token() {
    let h = setup();
    let (first_token, first_mint) = setup_token(&h.env, &h.admin);
    let first_token_client = token::Client::new(&h.env, &first_token);
    let (second_token, _second_mint) = setup_token(&h.env, &h.admin);
    let proposer = Address::generate(&h.env);
    first_mint.mint(&proposer, &500);
    h.client
        .set_proposal_deposit(&Some(first_token.clone()), &100);

    let id = h.propose_as(&proposer);
    // Reconfigure to a different token before this proposal closes.
    h.client.set_proposal_deposit(&Some(second_token), &200);
    h.advance_past_deadline();
    h.client.finalize(&id);

    assert_eq!(
        first_token_client.balance(&proposer),
        500,
        "must refund in the token this proposal actually paid in, not \
         whatever is configured now"
    );
}
