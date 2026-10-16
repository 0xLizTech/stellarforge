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
//! # Declared weight is capped, not verified, against a balance
//!
//! A live balance read at vote time does not fully verify weight either: a
//! holder can vote, transfer the same tokens onward, and vote again from the
//! recipient, since neither read sees the other's point in time. Fully
//! closing that needs balances snapshotted at proposal creation, which in
//! turn needs the weight-source contract to support a historical balance
//! query it does not have today (`rwa-asset::balance` returns only the
//! current balance). That is Phase 2/3 work alongside `SFORGE`.
//!
//! Capping is still worth doing now: it turns "declare any weight, with no
//! tie to a real holding at all" into "declare at most what you currently
//! hold," which is a real restriction even though it does not close the
//! transfer-and-vote-again path above. When no weight source is configured,
//! `vote` accepts the declared weight uncapped, matching `rwa-asset`'s own
//! fail-open-when-unconfigured precedent (ADR-004) so this contract stays
//! usable and testable without a deployed token. `finalize` still applies no
//! quorum, and outcomes remain advisory in Phase 1.

mod error;
mod events;
mod weight_source;

pub use error::GovernanceError;
pub use events::{AdminTransferred, ProposalCreated, ProposalFinalized, VoteCast, WeightSourceSet};
pub use weight_source::{WeightSourceClient, WeightSourceInterface};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol,
};
use stellarforge_common::{extend_instance, extend_persistent, storage::DAY_IN_LEDGERS};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PROP_COUNT: Symbol = symbol_short!("PCOUNT");
/// Address of the contract `vote` queries to cap declared weight, absent when
/// capping is disabled.
const WEIGHT_SOURCE_KEY: Symbol = symbol_short!("WSRC");

/// Shortest voting period `propose` accepts, about a day.
///
/// A proposer could previously pass 0 and have voting open only in the ledger
/// the proposal was created in, so nobody else could realistically see it,
/// let alone vote, before the proposer finalized it (IR-14).
pub const MIN_VOTING_PERIOD_LEDGERS: u32 = DAY_IN_LEDGERS;

/// Longest voting period `propose` accepts, about 90 days.
pub const MAX_VOTING_PERIOD_LEDGERS: u32 = 90 * DAY_IN_LEDGERS;

/// Longest title `propose` accepts, in bytes. The full text belongs in the
/// document `description_hash` points to.
pub const MAX_TITLE_BYTES: u32 = 256;

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

        // Bounded now rather than in Phase 3, so proposals created under looser
        // rules are not inherited once outcomes can execute.
        if !(MIN_VOTING_PERIOD_LEDGERS..=MAX_VOTING_PERIOD_LEDGERS).contains(&voting_period_ledgers)
        {
            panic_with_error!(&env, GovernanceError::InvalidVotingPeriod);
        }
        if title.len() > MAX_TITLE_BYTES {
            panic_with_error!(&env, GovernanceError::TitleTooLong);
        }

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

        // Cap the declared weight at the voter's current balance in the
        // configured weight source, so a vote can carry at most what the
        // voter demonstrably holds right now. Uncapped when no source is
        // configured (module docs explain why, and ADR-004 is the precedent).
        // A voter capped to zero or less holds nothing to vote with, so this
        // is the same InvalidWeight rejection as a non-positive declared
        // weight, not a zero-weight vote that silently consumes their one
        // shot at this proposal.
        let weight = Self::capped_weight(&env, &voter, weight);
        if weight <= 0 {
            panic_with_error!(&env, GovernanceError::InvalidWeight);
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

    /// Point `vote` at a contract to query for a weight cap, or pass `None`
    /// to vote uncapped again.
    ///
    /// Mirrors `rwa-asset::set_compliance` (ADR-004): a structural binding to
    /// whatever address is given, not a dependency on a specific crate, so
    /// the weight source can be swapped (a different token, a mock in tests)
    /// without rebuilding this contract.
    pub fn set_weight_source(env: Env, weight_source: Option<Address>) {
        Self::require_admin(&env);
        match &weight_source {
            Some(addr) => env.storage().instance().set(&WEIGHT_SOURCE_KEY, addr),
            None => env.storage().instance().remove(&WEIGHT_SOURCE_KEY),
        }
        extend_instance(&env);

        events::WeightSourceSet { weight_source }.publish(&env);
    }

    pub fn weight_source(env: Env) -> Option<Address> {
        env.storage().instance().get(&WEIGHT_SOURCE_KEY)
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
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

    /// `weight`, capped at `voter`'s current balance in the configured
    /// weight source. Returns `weight` unchanged when no source is
    /// configured, matching `rwa-asset::require_compliant`'s fail-open
    /// behaviour for the same reason: this contract must stay usable before
    /// an operator has wired up a weight source.
    fn capped_weight(env: &Env, voter: &Address, weight: i128) -> i128 {
        let Some(source) = Self::weight_source(env.clone()) else {
            return weight;
        };
        let client = WeightSourceClient::new(env, &source);
        let balance = client.balance(voter);
        if balance < weight {
            balance.max(0)
        } else {
            weight
        }
    }
}
