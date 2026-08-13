#![no_std]

//! Governance — on-chain proposal & voting skeleton.
//! Phase 1: create proposals and cast votes; execution hooks in Phase 3.

mod error;

pub use error::GovernanceError;

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PROP_COUNT: Symbol = symbol_short!("PCOUNT");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Proposal(u64),
    Vote(u64, Address),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description_hash: soroban_sdk::Bytes,
    pub votes_for: i128,
    pub votes_against: i128,
    pub deadline_ledger: u32,
    pub status: ProposalStatus,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(&env, GovernanceError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PROP_COUNT, &0_u64);

        extend_instance(&env);
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        title: String,
        description_hash: soroban_sdk::Bytes,
        voting_period_ledgers: u32,
    ) -> u64 {
        proposer.require_auth();
        let count: u64 = env.storage().instance().get(&PROP_COUNT).unwrap_or(0);
        let id = match count.checked_add(1) {
            Some(v) => v,
            None => panic_with_error!(&env, GovernanceError::Overflow),
        };

        // A voting period long enough to overflow the ledger sequence would
        // wrap the deadline into the past and close voting immediately.
        let deadline_ledger = match env.ledger().sequence().checked_add(voting_period_ledgers) {
            Some(v) => v,
            None => panic_with_error!(&env, GovernanceError::Overflow),
        };

        let proposal = Proposal {
            id,
            proposer,
            title,
            description_hash,
            votes_for: 0,
            votes_against: 0,
            deadline_ledger,
            status: ProposalStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), &proposal);
        env.storage().instance().set(&PROP_COUNT, &id);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Proposal(id));

        id
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u64, support: bool, weight: i128) {
        voter.require_auth();
        if weight <= 0 {
            panic_with_error!(&env, GovernanceError::InvalidWeight);
        }

        let mut proposal = Self::load_proposal(&env, proposal_id);

        if proposal.status != ProposalStatus::Active {
            panic_with_error!(&env, GovernanceError::ProposalNotActive);
        }
        if env.ledger().sequence() > proposal.deadline_ledger {
            panic_with_error!(&env, GovernanceError::VotingClosed);
        }

        let vote_key = DataKey::Vote(proposal_id, voter);
        let already_voted: bool = env.storage().persistent().get(&vote_key).unwrap_or(false);
        if already_voted {
            panic_with_error!(&env, GovernanceError::AlreadyVoted);
        }

        let tally = if support {
            &mut proposal.votes_for
        } else {
            &mut proposal.votes_against
        };
        *tally = match tally.checked_add(weight) {
            Some(v) => v,
            None => panic_with_error!(&env, GovernanceError::Overflow),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&vote_key, &true);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Proposal(proposal_id));
        extend_persistent(&env, &vote_key);
    }

    pub fn finalize(env: Env, proposal_id: u64) {
        let mut proposal = Self::load_proposal(&env, proposal_id);

        if env.ledger().sequence() <= proposal.deadline_ledger {
            panic_with_error!(&env, GovernanceError::VotingNotClosed);
        }
        if proposal.status != ProposalStatus::Active {
            panic_with_error!(&env, GovernanceError::ProposalNotActive);
        }

        // A tie fails: passing requires strictly more support than opposition.
        proposal.status = if proposal.votes_for > proposal.votes_against {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        extend_persistent(&env, &DataKey::Proposal(proposal_id));
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        let key = DataKey::Proposal(proposal_id);
        // Proposals must stay queryable long after their deadline, and nothing
        // writes to a finalised proposal again, so the read path is what keeps
        // the record alive.
        extend_persistent(&env, &key);
        env.storage().persistent().get(&key)
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        let key = DataKey::Vote(proposal_id, voter);
        extend_persistent(&env, &key);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&PROP_COUNT).unwrap_or(0)
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, GovernanceError::NotInitialized),
        }
    }

    fn load_proposal(env: &Env, proposal_id: u64) -> Proposal {
        match env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
        {
            Some(p) => p,
            None => panic_with_error!(env, GovernanceError::ProposalNotFound),
        }
    }
}
