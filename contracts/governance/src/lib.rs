#![no_std]

//! Governance — on-chain proposal & voting skeleton.
//! Phase 1: create proposals and cast votes; execution hooks in Phase 3.
//!
//! # This does not decide anything yet
//!
//! `vote` accepts the weight its caller declares and verifies it against
//! nothing, and `finalize` applies no quorum. Any address can therefore carry
//! any proposal. Results are advisory until Phase 3 supplies a voting-power
//! source; nothing here should gate a privileged action.
//!
//! Weight is deliberately left unverified rather than wired to a token
//! balance, because reading a live balance at vote time is worse than not
//! checking at all: a holder could vote, transfer the same tokens onward and
//! vote again from the recipient. Correct weighting needs balances snapshotted
//! at proposal creation, which is Phase 2/3 work alongside `SFORGE`.

mod error;
mod events;

pub use error::GovernanceError;
pub use events::{AdminTransferred, ProposalCreated, ProposalFinalized, VoteCast};

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
    /// Configures the contract as part of the deploy transaction.
    ///
    /// A constructor rather than a separate `initialize` entry point: the two
    /// are equivalent once the contract is running, but a separate call leaves
    /// a window in which the contract exists with no admin and anyone may name
    /// themselves. `require_auth` on `admin` still applies, so a deployer
    /// cannot hand the role to a key whose holder has not signed for it.
    pub fn __constructor(env: Env, admin: Address) {
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
        // Defensive only, and unreachable since the constructor runs at deploy.
        // It guarded a hazard that no longer has a way to occur: `initialize`
        // reset the counter to zero, so a proposal created beforehand had its
        // id handed out a second time — the later proposal overwrote the
        // earlier one, while the vote records keyed to that id survived and
        // locked out everyone who had already voted. Nothing resets the
        // counter now.
        Self::require_initialized(&env);

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

        events::ProposalCreated {
            proposal_id: id,
            proposer: proposal.proposer,
            title: proposal.title,
            description_hash: proposal.description_hash,
            deadline_ledger,
        }
        .publish(&env);

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

        let vote_key = DataKey::Vote(proposal_id, voter.clone());
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

        events::VoteCast {
            proposal_id,
            voter,
            support,
            weight,
        }
        .publish(&env);
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

        events::ProposalFinalized {
            proposal_id,
            status: proposal.status,
            votes_for: proposal.votes_for,
            votes_against: proposal.votes_against,
        }
        .publish(&env);
    }

    /// A pure query. Proposals must stay readable long after their deadline,
    /// which the network-maximum extension on the write paths provides without
    /// charging every reader for it.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Vote(proposal_id, voter))
            .unwrap_or(false)
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&PROP_COUNT).unwrap_or(0)
    }

    /// Hands the admin role to `new_admin`.
    ///
    /// Both the current and the incoming admin must authorize, in the same
    /// transaction (NFR-S-4), for the reason ADR-002 gives for `rwa-asset`: a
    /// handover to a key nobody controls cannot be undone.
    ///
    /// The admin has no powers in Phase 1, but Phase 3 execution hooks will
    /// hang off it, and a role that cannot be rotated would reach them
    /// already unrecoverable if its key were lost or exposed (IR-04).
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let previous = Self::admin(env.clone());
        previous.require_auth();
        new_admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin,
        }
        .publish(&env);
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, GovernanceError::NotInitialized),
        }
    }

    fn require_initialized(env: &Env) {
        if !env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(env, GovernanceError::NotInitialized);
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
