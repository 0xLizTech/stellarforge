#![no_std]

//! Governance — on-chain proposal & voting skeleton.
//! Phase 1: create proposals and cast votes; execution hooks in Phase 3.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

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
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PROP_COUNT, &0_u64);
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        title: String,
        description_hash: soroban_sdk::Bytes,
        voting_period_ledgers: u32,
    ) -> u64 {
        proposer.require_auth();
        let id: u64 = env
            .storage()
            .instance()
            .get(&PROP_COUNT)
            .unwrap_or(0)
            + 1;

        let proposal = Proposal {
            id,
            proposer,
            title,
            description_hash,
            votes_for: 0,
            votes_against: 0,
            deadline_ledger: env.ledger().sequence() + voting_period_ledgers,
            status: ProposalStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), &proposal);
        env.storage().instance().set(&PROP_COUNT, &id);
        id
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u64, support: bool, weight: i128) {
        voter.require_auth();
        assert!(weight > 0, "weight must be positive");

        let already_voted: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Vote(proposal_id, voter.clone()))
            .unwrap_or(false);
        assert!(!already_voted, "already voted");

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        assert!(
            proposal.status == ProposalStatus::Active,
            "proposal not active"
        );
        assert!(
            env.ledger().sequence() <= proposal.deadline_ledger,
            "voting closed"
        );

        if support {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Vote(proposal_id, voter), &true);
    }

    pub fn finalize(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        assert!(
            env.ledger().sequence() > proposal.deadline_ledger,
            "voting not closed"
        );
        assert!(
            proposal.status == ProposalStatus::Active,
            "already finalized"
        );

        proposal.status = if proposal.votes_for > proposal.votes_against {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&PROP_COUNT).unwrap_or(0)
    }
}
