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
pub use events::{
    AdminTransferred, ProposalCancelled, ProposalCreated, ProposalFinalized, VoteCast,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, String,
    Symbol, Vec,
    AdminTransferred, DepositConfigured, ProposalCreated, ProposalDepositRefunded,
    ProposalFinalized, VoteCast,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, token, Address, Env,
    String, Symbol,
    AdminTransferCancelled, AdminTransferProposed, AdminTransferred, PassThresholdSet,
    ProposalCreated, ProposalFinalized, QuorumSet, VoteCast,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Bytes, BytesN,
    Env, String, Symbol,
};
use stellarforge_common::{extend_instance, extend_persistent, storage::DAY_IN_LEDGERS};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PENDING_ADMIN_KEY: Symbol = symbol_short!("PADMIN");
const PROP_COUNT: Symbol = symbol_short!("PCOUNT");
const DEPOSIT_TOKEN_KEY: Symbol = symbol_short!("DEP_TOKEN");
const DEPOSIT_AMOUNT_KEY: Symbol = symbol_short!("DEP_AMT");
const QUORUM_KEY: Symbol = symbol_short!("QUORUM");
const PASS_BPS_KEY: Symbol = symbol_short!("PASSBPS");

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

/// Largest page `list_proposals` will return.
///
/// Each listed proposal is its own ledger read, and a read-only simulation is
/// held to the same per-invocation read limits as a transaction, so a page
/// has to stay well inside them. Matches `registry::MAX_PAGE_SIZE`.
pub const MAX_PAGE_SIZE: u32 = 100;
/// `pass_threshold_bps` is a fraction of 10,000 (100.00%).
pub const MAX_PASS_THRESHOLD_BPS: u32 = 10_000;
/// A zero threshold would let a proposal pass with no support at all, which
/// is not a vote, so the floor sits just above it.
pub const MIN_PASS_THRESHOLD_BPS: u32 = 1;

/// `finalize`'s behavior before quorum/threshold existed: more for than
/// against, with no minimum turnout. Kept as the default so a contract that
/// never calls `set_quorum`/`set_pass_threshold_bps` behaves exactly as it
/// did before this change.
pub const DEFAULT_QUORUM: i128 = 0;
pub const DEFAULT_PASS_THRESHOLD_BPS: u32 = 5_001;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Proposal(u64),
    Vote(u64, Address),
    /// Hash of (title, description_hash) for the currently active proposal
    /// that produced it, keyed by the hash itself so a second `propose` with
    /// identical content can be rejected by one lookup (#136) rather than a
    /// linear scan of every open proposal.
    ActiveProposalHash(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
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
    /// The token `propose` collected a deposit in, or `None` if no deposit
    /// was configured when this proposal was created. Carried on the
    /// proposal itself, rather than read from `deposit_token()` at refund
    /// time, so `finalize` refunds in the same token this proposal actually
    /// paid in, even if `set_proposal_deposit` later changes or clears the
    /// configured token (#134).
    pub deposit_token: Option<Address>,
    /// The amount collected in `deposit_token`, alongside it for the same
    /// reason; always `0` when `deposit_token` is `None`. The deposit is an
    /// anti-spam cost of proposing, not a stake on the result, so `finalize`
    /// refunds it regardless of outcome.
    pub deposit_amount: i128,
    /// sha256(title ++ description_hash), stored so `finalize` can clear this
    /// proposal's `DataKey::ActiveProposalHash` entry without recomputing it
    /// from a (title, description_hash) that may no longer be in scope.
    pub content_hash: BytesN<32>,
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
        env.storage().instance().set(&QUORUM_KEY, &DEFAULT_QUORUM);
        env.storage()
            .instance()
            .set(&PASS_BPS_KEY, &DEFAULT_PASS_THRESHOLD_BPS);

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

        // Reject a second active proposal with identical content (#136).
        // Hashed rather than compared field-by-field so duplicate detection
        // is one storage lookup regardless of how many proposals are
        // currently active, and so the check covers title+action together:
        // changing either one produces a different hash and a legitimately
        // distinct proposal.
        let content_hash = Self::proposal_content_hash(&env, &title, &description_hash);
        if env
            .storage()
            .persistent()
            .has(&DataKey::ActiveProposalHash(content_hash.clone()))
        {
            panic_with_error!(&env, GovernanceError::DuplicateActiveProposal);
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

        // IR-14's remaining follow-up (#134): an optional admin-configured
        // deposit, so spam proposals cost the proposer rather than only fees
        // and rent. Collected here in whatever token and amount are
        // configured right now, and that same token/amount pair travels with
        // the proposal so `finalize` refunds correctly even if the admin
        // reconfigures or clears the deposit before this proposal closes.
        let configured_token = Self::deposit_token(env.clone());
        let configured_amount = Self::deposit_amount(env.clone());
        let (deposit_token, deposit_amount) = match &configured_token {
            Some(token_address) if configured_amount > 0 => {
                token::Client::new(&env, token_address).transfer(
                    &proposer,
                    env.current_contract_address(),
                    &configured_amount,
                );
                (configured_token, configured_amount)
            }
            _ => (None, 0),
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
            deposit_token,
            deposit_amount,
            content_hash: content_hash.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveProposalHash(content_hash.clone()), &id);
        env.storage().instance().set(&PROP_COUNT, &id);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Proposal(id));
        extend_persistent(&env, &DataKey::ActiveProposalHash(content_hash));

        events::ProposalCreated {
            proposal_id: id,
            proposer: proposal.proposer,
            title: proposal.title,
            description_hash: proposal.description_hash,
            deadline_ledger,
            deposit_amount: proposal.deposit_amount,
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

        let total_votes = match proposal.votes_for.checked_add(proposal.votes_against) {
            Some(v) => v,
            None => panic_with_error!(&env, GovernanceError::Overflow),
        };
        let quorum: i128 = env
            .storage()
            .instance()
            .get(&QUORUM_KEY)
            .unwrap_or(DEFAULT_QUORUM);
        let pass_threshold_bps: u32 = env
            .storage()
            .instance()
            .get(&PASS_BPS_KEY)
            .unwrap_or(DEFAULT_PASS_THRESHOLD_BPS);

        // Below quorum fails outright, before the threshold is even
        // evaluated: a proposal nobody looked at should not pass on the
        // strength of a 1-0 vote just because the one vote was "for". Zero
        // votes is rejected unconditionally, independent of quorum: the
        // threshold comparison below is for_bps/total_votes >= threshold,
        // which is vacuously satisfied at 0/0 and would otherwise let an
        // unvoted proposal pass whenever quorum is left at its default of 0.
        proposal.status = if total_votes == 0 || total_votes < quorum {
            ProposalStatus::Rejected
        } else {
            // votes_for / total_votes >= pass_threshold_bps / 10_000, without
            // the division: cross-multiply to stay in integer arithmetic.
            // total_votes is bounded by i128 (each weight is checked_add'd in
            // `vote`), and pass_threshold_bps <= 10_000, so the product does
            // not approach i128::MAX for any input `vote` itself accepts.
            let for_bps = proposal
                .votes_for
                .saturating_mul(MAX_PASS_THRESHOLD_BPS as i128);
            let required = total_votes.saturating_mul(pass_threshold_bps as i128);
            if for_bps >= required {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected
            }
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        // The slot that blocked a duplicate while this proposal was active is
        // released the moment it stops being active, so a genuinely new
        // proposal with the same content can be made afterward (#136 only
        // bars a *concurrent* duplicate, not a resubmission once the first
        // has been decided).
        env.storage()
            .persistent()
            .remove(&DataKey::ActiveProposalHash(proposal.content_hash.clone()));

        extend_persistent(&env, &DataKey::Proposal(proposal_id));

        // Refunded regardless of status (#134): the deposit is a cost of
        // spamming the proposal list, not a stake on the outcome, so it is
        // returned whether the proposal passed or was rejected, in the same
        // token it was paid in. `finalize` runs at most once per proposal
        // (the `ProposalNotActive` check above), so this cannot
        // double-refund.
        if let Some(token_address) = &proposal.deposit_token {
            token::Client::new(&env, token_address).transfer(
                &env.current_contract_address(),
                &proposal.proposer,
                &proposal.deposit_amount,
            );

            events::ProposalDepositRefunded {
                proposal_id,
                proposer: proposal.proposer.clone(),
                amount: proposal.deposit_amount,
            }
            .publish(&env);
        }

        events::ProposalFinalized {
            proposal_id,
            status: proposal.status,
            votes_for: proposal.votes_for,
            votes_against: proposal.votes_against,
        }
        .publish(&env);
    }

    /// Withdraws a mistaken proposal before anyone has voted on it.
    ///
    /// Restricted to before the first vote rather than before the deadline: a
    /// voter commits weight to an outcome the moment they vote, and letting a
    /// later cancellation erase that vote (with no way for the voter to react)
    /// would make every "yes" or "no" provisional for as long as the proposer
    /// or admin cared to withdraw it. A proposal with zero votes has no such
    /// voter to protect.
    ///
    /// Either the proposer or the admin may call this: the proposer owns a
    /// genuine mistake, and the admin covers a proposer who is unreachable or
    /// no longer holds the key that created it.
    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();

        let mut proposal = Self::load_proposal(&env, proposal_id);

        if caller != proposal.proposer && caller != Self::admin(env.clone()) {
            panic_with_error!(&env, GovernanceError::NotAuthorizedToCancel);
        }
        if proposal.status != ProposalStatus::Active {
            panic_with_error!(&env, GovernanceError::ProposalNotActive);
        }
        if proposal.votes_for > 0 || proposal.votes_against > 0 {
            panic_with_error!(&env, GovernanceError::ProposalAlreadyHasVotes);
        }

        proposal.status = ProposalStatus::Cancelled;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        extend_persistent(&env, &DataKey::Proposal(proposal_id));

        events::ProposalCancelled {
            proposal_id,
            cancelled_by: caller,
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

    /// Proposals in creation order, `limit` of them starting at index `start`.
    ///
    /// `start` is a zero-based index into the directory, not a proposal id:
    /// index 0 is proposal id 1, since ids are assigned starting at 1 and
    /// never reused. A page shorter than `limit` means the end of the
    /// directory was reached; a `start` at or past [`GovernanceContract::proposal_count`]
    /// returns an empty page. `limit` above [`MAX_PAGE_SIZE`] is rejected
    /// rather than quietly truncated, so a caller cannot mistake a clipped
    /// page for the whole directory.
    pub fn list_proposals(env: Env, start: u32, limit: u32) -> Vec<Proposal> {
        if limit > MAX_PAGE_SIZE {
            panic_with_error!(&env, GovernanceError::PageTooLarge);
        }

        let count: u64 = env.storage().instance().get(&PROP_COUNT).unwrap_or(0);
        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let end = start.saturating_add(limit).min(count);

        let mut page = Vec::new(&env);
        for index in start..end {
            // Every index below the count was written by `propose` and
            // nothing removes one (cancellation changes status, not
            // existence), so this is always present. Proposal ids start at
            // 1, one past the zero-based `index`.
            let id = u64::from(index) + 1;
            if let Some(proposal) = env.storage().persistent().get(&DataKey::Proposal(id)) {
                page.push_back(proposal);
            }
        }
        page
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

    /// Proposes `new_admin` as the next admin. Only the current admin may
    /// call this; `new_admin` has not authorized anything yet.
    ///
    /// Two-step handover (#125), replacing the single-transaction dual-auth
    /// `transfer_admin` ADR-002 originally specified. ADR-002 rejected this
    /// shape for Phase 1 only because the SDK could not build a transaction
    /// carrying two distinct signers (#31); #31 is closed, and ADR-006
    /// revisits the tradeoff now that it no longer applies. The underlying
    /// guarantee is unchanged: a handover to a key nobody controls still
    /// cannot complete, because `accept_admin` still requires that key's own
    /// signature. The admin has no powers in Phase 1, but Phase 3 execution
    /// hooks will hang off it, and a role that cannot be rotated would reach
    /// them already unrecoverable if its key were lost or exposed (IR-04).
    ///
    /// Calling this again before `accept_admin` replaces the pending
    /// proposal rather than requiring `cancel_admin_transfer` first.
    pub fn propose_admin(env: Env, new_admin: Address) {
        let current = Self::admin(env.clone());
        current.require_auth();
        env.storage().instance().set(&PENDING_ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferProposed {
            current,
            proposed_admin: new_admin,
        }
        .publish(&env);
    }

    /// Completes a handover `propose_admin` started. Only the proposed admin
    /// may call this, and only its own signature is required: the current
    /// admin already authorized the proposal, which is where the "both
    /// sides consented" guarantee comes from now.
    pub fn accept_admin(env: Env) {
        let pending: Address = match env.storage().instance().get(&PENDING_ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, GovernanceError::NoPendingAdminTransfer),
        };
        pending.require_auth();

        let previous = Self::admin(env.clone());
        env.storage().instance().set(&ADMIN_KEY, &pending);
        env.storage().instance().remove(&PENDING_ADMIN_KEY);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin: pending,
        }
        .publish(&env);
    }

    /// Withdraws a pending proposal before it is accepted. Only the current
    /// admin may call this, which is what makes a mistaken or no-longer-wanted
    /// proposal reversible without waiting for the proposed admin to act (or
    /// refuse to).
    pub fn cancel_admin_transfer(env: Env) {
        let current = Self::admin(env.clone());
        current.require_auth();
        let pending: Address = match env.storage().instance().get(&PENDING_ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, GovernanceError::NoPendingAdminTransfer),
        };
        env.storage().instance().remove(&PENDING_ADMIN_KEY);
        extend_instance(&env);

        events::AdminTransferCancelled {
            cancelled_proposed_admin: pending,
        }
        .publish(&env);
    }

    /// The address `accept_admin` would install if called now, or `None` if
    /// no transfer is pending.
    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&PENDING_ADMIN_KEY)
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, GovernanceError::NotInitialized),
        }
    }

    // ── Proposal Deposit Configuration (#134) ───────────────────────────────

    /// Configures the proposal deposit, or pass `token = None` to disable it.
    ///
    /// `amount` is ignored (and stored as `0`) whenever `token` is `None`, so
    /// a disabled deposit cannot be re-enabled by accident through a stale
    /// `amount` left over from an earlier configuration. Mirrors
    /// `rwa-asset::set_compliance`'s `Option<Address>` shape (ADR-004):
    /// unconfigured means the feature is off, not that it defaults to some
    /// token address.
    pub fn set_proposal_deposit(env: Env, token: Option<Address>, amount: i128) {
        Self::require_admin(&env);
        if amount < 0 {
            panic_with_error!(&env, GovernanceError::InvalidDepositAmount);
        }

        let stored_amount = if token.is_some() { amount } else { 0 };
        match &token {
            Some(addr) => env.storage().instance().set(&DEPOSIT_TOKEN_KEY, addr),
            None => env.storage().instance().remove(&DEPOSIT_TOKEN_KEY),
        }
        env.storage()
            .instance()
            .set(&DEPOSIT_AMOUNT_KEY, &stored_amount);
        extend_instance(&env);

        events::DepositConfigured {
            token,
            amount: stored_amount,
    /// Sets the minimum total votes (`votes_for + votes_against`) a proposal
    /// needs before `finalize` will even evaluate its threshold (#135). Zero
    /// disables the check, matching `finalize`'s pre-#135 behavior.
    pub fn set_quorum(env: Env, quorum: i128) {
        let admin = Self::admin(env.clone());
        admin.require_auth();
        if quorum < 0 {
            panic_with_error!(&env, GovernanceError::InvalidQuorum);
        }
        env.storage().instance().set(&QUORUM_KEY, &quorum);
        extend_instance(&env);

        events::QuorumSet { admin, quorum }.publish(&env);
    }

    pub fn quorum(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&QUORUM_KEY)
            .unwrap_or(DEFAULT_QUORUM)
    }

    /// Sets the fraction of cast votes (in basis points of
    /// `votes_for + votes_against`) that must be `votes_for` for a proposal
    /// to pass (#135). Replaces the old unconditional "strictly more for
    /// than against" rule once quorum is also met.
    pub fn set_pass_threshold_bps(env: Env, pass_threshold_bps: u32) {
        let admin = Self::admin(env.clone());
        admin.require_auth();
        if !(MIN_PASS_THRESHOLD_BPS..=MAX_PASS_THRESHOLD_BPS).contains(&pass_threshold_bps) {
            panic_with_error!(&env, GovernanceError::InvalidPassThresholdBps);
        }
        env.storage()
            .instance()
            .set(&PASS_BPS_KEY, &pass_threshold_bps);
        extend_instance(&env);

        events::PassThresholdSet {
            admin,
            pass_threshold_bps,
        }
        .publish(&env);
    }

    pub fn deposit_token(env: Env) -> Option<Address> {
        env.storage().instance().get(&DEPOSIT_TOKEN_KEY)
    }

    /// The amount `propose` currently collects, in `deposit_token()`'s token.
    /// Always `0` when no deposit token is configured.
    pub fn deposit_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DEPOSIT_AMOUNT_KEY)
            .unwrap_or(0)
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    pub fn pass_threshold_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&PASS_BPS_KEY)
            .unwrap_or(DEFAULT_PASS_THRESHOLD_BPS)
    }

    fn require_initialized(env: &Env) {
        if !env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(env, GovernanceError::NotInitialized);
        }
    }

    /// sha256 of `title`'s bytes followed by `description_hash`, the key
    /// #136's duplicate check stores and looks up. `title` participates
    /// (not just `description_hash`) so renaming a proposal while keeping the
    /// same description document counts as a distinct proposal, matching
    /// what a human reviewing the directory would call "different."
    fn proposal_content_hash(env: &Env, title: &String, description_hash: &Bytes) -> BytesN<32> {
        let mut buf = title.to_bytes();
        buf.append(description_hash);
        env.crypto().sha256(&buf).to_bytes()
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
