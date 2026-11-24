#![no_std]

mod compliance;
mod error;
mod events;

pub use compliance::{ComplianceClient, ComplianceInterface};
pub use error::RwaError;
pub use events::{
    AdminTransferCancelled, AdminTransferProposed, AdminTransferred, Approve, Burn,
    CapIncreaseCancelled, CapIncreaseProposed, ComplianceSet, ForcedTransfer, FrozenSet,
    GovernanceSet, HolderCapSet, IssuerSet, LockupDateSet, MetadataUpdated, Mint, Paused, Transfer,
    TransferFrom,
};

use stellarforge_common::{extend_instance, extend_persistent};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Bytes, Env,
    MuxedAddress, String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PENDING_ADMIN_KEY: Symbol = symbol_short!("PADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
/// Address of the compliance contract, absent when screening is disabled.
const COMPLIANCE_KEY: Symbol = symbol_short!("COMPLY");
/// Minimum verification level a party must hold to send or receive.
const MIN_LEVEL_KEY: Symbol = symbol_short!("MINLVL");
/// Address authorized to approve loosening a supply cap, absent when no cap
/// may be loosened at all (#148).
const GOVERNANCE_KEY: Symbol = symbol_short!("GOV");

/// Per-contract ceiling on the number of distinct token holders, absent when
/// uncapped. Stored in instance storage alongside the other small admin flags.
const HOLDER_CAP_KEY: Symbol = symbol_short!("HOLDCAP");

/// Upper bound on `AssetMetadata::decimals`. Beyond this a whole token no
/// longer fits sensibly in `i128` alongside realistic supply figures.
const MAX_DECIMALS: u32 = 18;

/// Required length of `AssetMetadata::legal_doc_hash`: a raw SHA-256 digest.
const LEGAL_DOC_HASH_LEN: u32 = 32;

/// Upper bound on `set_compliance`'s `min_level`: 0 none, 1 basic, 2 full,
/// 3 accredited, matching `compliance::MAX_LEVEL`'s scale. Declared locally
/// rather than imported from the `compliance` crate: this contract binds
/// only to the minimal `ComplianceInterface` trait in `src/compliance.rs`,
/// deliberately not to a specific compliance implementation (see that
/// module's doc comment), so it cannot read a constant from one. A
/// `min_level` above this ceiling could never be satisfied by any verified
/// party, since a compliance record's own level cannot exceed it either;
/// every transfer would then fail `NotCompliant` until an admin
/// reconfigured it (#139).
pub const MAX_COMPLIANCE_LEVEL: u32 = 3;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Metadata,
    /// How many times `Metadata` has been written: 1 at construction,
    /// incremented by one on every subsequent change (#149). Kept apart from
    /// `AssetMetadata` itself rather than as one of its fields, so a caller
    /// updating metadata never has to read back and resupply the current
    /// version just to avoid clobbering it.
    MetadataVersion,
    Balance(Address),
    TotalSupply,
    Allowance(Address, Address),
    Issuer(Address),
    /// The `max_supply` a `propose_cap_increase` is waiting on `governance`
    /// to approve, if any (#148).
    PendingCapIncrease,
    /// Addresses permitted to call `forced_transfer` (#144).
    ForcedTransferRole(Address),
    /// Whether an address is frozen and cannot send tokens.
    Frozen(Address),
    /// The ledger sequence until which an address's tokens are locked.
    LockupDate(Address),
    /// Running count of addresses whose balance is strictly greater than zero.
    /// Incremented when a balance transitions 0 → >0, decremented on >0 → 0.
    HolderCount,
}

/// A spender's allowance and the last ledger it may be spent in (SEP-41).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    /// After this ledger the allowance reads as zero.
    pub live_until_ledger: u32,
}

// ─── Data Types ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AssetMetadata {
    /// Human-readable name of the real-world asset
    pub name: String,
    /// Ticker symbol (e.g. "REIT-NYC-001")
    pub symbol: String,
    /// Number of decimal places for fractional ownership
    pub decimals: u32,
    /// Asset class: "real_estate", "commodity", "equity", "debt", etc.
    pub asset_class: String,
    /// SHA-256 digest (32 raw bytes, not hex) of the off-chain legal document
    /// this asset represents. Must be exactly [`LEGAL_DOC_HASH_LEN`] bytes
    /// and not all-zero: either would make the on-chain token point at no
    /// verifiable legal anchor (#150).
    pub legal_doc_hash: Bytes,
    /// Total cap on issuance (0 = uncapped)
    pub max_supply: i128,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct RwaAssetContract;

#[contractimpl]
impl RwaAssetContract {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Configures the contract as part of the deploy transaction.
    ///
    /// A constructor rather than a separate `initialize` entry point: the two
    /// are equivalent once the contract is running, but a separate call leaves
    /// a window in which the contract exists with no admin and anyone may name
    /// themselves. `require_auth` on `admin` still applies, so a deployer
    /// cannot hand the role to a key whose holder has not signed for it.
    pub fn __constructor(env: Env, admin: Address, metadata: AssetMetadata) {
        admin.require_auth();
        Self::validate_metadata(&env, &metadata);

        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&PAUSED_KEY, &false);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::MetadataVersion, &1_u32);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &0_i128);

        // Holder count starts at zero; no addresses hold tokens at construction.
        env.storage()
            .persistent()
            .set(&DataKey::HolderCount, &0_u32);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Metadata);
        extend_persistent(&env, &DataKey::MetadataVersion);
        extend_persistent(&env, &DataKey::TotalSupply);
        extend_persistent(&env, &DataKey::HolderCount);
    }

    // ── Forced Transfer Role ──────────────────────────────────────────────────

    /// Grant or revoke the forced-transfer role for an address.
    ///
    /// Only the admin may call this. The role is deliberately separate from
    /// the issuer role: an issuer mints and burns, whereas a forced-transfer
    /// holder can move any holder's balance without their consent, which is
    /// the power a court order needs and nothing else should have.
    pub fn set_forced_transfer_role(env: Env, account: Address, approved: bool) {
        Self::require_admin(&env);
        extend_instance(&env);
        env.storage()
            .persistent()
            .set(&DataKey::ForcedTransferRole(account.clone()), &approved);
        extend_persistent(&env, &DataKey::ForcedTransferRole(account.clone()));
    }

    pub fn is_forced_transfer_role(env: Env, account: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::ForcedTransferRole(account))
            .unwrap_or(false)
    }

    // ── Frozen Address Management ──────────────────────────────────────────────

    /// Freeze or unfreeze an address. Frozen addresses cannot send tokens.
    ///
    /// Only the admin may call this. This is used to comply with sanctions or
    /// other regulatory requirements that require blocking specific addresses.
    pub fn set_frozen(env: Env, address: Address, frozen: bool) {
        Self::require_admin(&env);
        extend_instance(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Frozen(address.clone()), &frozen);
        extend_persistent(&env, &DataKey::Frozen(address.clone()));

        events::FrozenSet { address, frozen }.publish(&env);
    }

    pub fn is_frozen(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Frozen(address))
            .unwrap_or(false)
    }

    // ── Lock-up Date Management ────────────────────────────────────────────────

    /// Set the lock-up date for an address. Tokens are locked until the
    /// specified ledger sequence.
    ///
    /// Only the admin may call this. This is used for vesting periods or
    /// regulatory holding periods. Setting to 0 removes the lock-up.
    pub fn set_lockup_date(env: Env, address: Address, lockup_until_ledger: u32) {
        Self::require_admin(&env);
        extend_instance(&env);
        env.storage()
            .persistent()
            .set(&DataKey::LockupDate(address.clone()), &lockup_until_ledger);
        extend_persistent(&env, &DataKey::LockupDate(address.clone()));

        events::LockupDateSet {
            address,
            lockup_until_ledger,
        }
        .publish(&env);
    }

    pub fn lockup_date(env: Env, address: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::LockupDate(address))
            .unwrap_or(0)
    }

    // ── Issuer Management ─────────────────────────────────────────────────────

    /// Grant or revoke issuer role for an address.
    pub fn set_issuer(env: Env, issuer: Address, approved: bool) {
        Self::require_admin(&env);
        extend_instance(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Issuer(issuer.clone()), &approved);
        extend_persistent(&env, &DataKey::Issuer(issuer.clone()));

        events::IssuerSet { issuer, approved }.publish(&env);
    }

    pub fn is_issuer(env: Env, issuer: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer))
            .unwrap_or(false)
    }

    // ── Minting & Burning ─────────────────────────────────────────────────────

    /// Mint new tokens to a recipient. Caller must be a registered issuer.
    pub fn mint(env: Env, issuer: Address, to: Address, amount: i128) {
        issuer.require_auth();
        Self::require_not_paused(&env);
        if !Self::is_issuer(env.clone(), issuer.clone()) {
            panic_with_error!(&env, RwaError::NotIssuer);
        }
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &to);

        let meta = Self::metadata(env.clone());
        let total = Self::total_supply(env.clone());
        let new_total = Self::checked_add(&env, total, amount);

        if meta.max_supply > 0 && new_total > meta.max_supply {
            panic_with_error!(&env, RwaError::ExceedsMaxSupply);
        }

        let bal = Self::balance(env.clone(), to.clone());
        let new_bal = Self::checked_add(&env, bal, amount);

        // Reject if this mint would add a new holder and exceed the cap.
        if bal == 0 {
            Self::require_holder_cap_not_exceeded(&env);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_bal);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &new_total);

        // A new holder is anyone whose balance just crossed zero → positive.
        if bal == 0 {
            Self::increment_holder_count(&env);
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(to.clone()));
        extend_persistent(&env, &DataKey::TotalSupply);

        events::Mint { issuer, to, amount }.publish(&env);
    }

    /// Burn tokens from caller's balance.
    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);

        let bal = Self::balance(env.clone(), from.clone());
        if bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        let total = Self::total_supply(env.clone());
        let new_bal = bal - amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_bal);
        env.storage().persistent().set(
            &DataKey::TotalSupply,
            &Self::checked_sub(&env, total, amount),
        );

        // Holder count decrements when a balance hits zero.
        if new_bal == 0 {
            Self::decrement_holder_count(&env);
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::TotalSupply);

        events::Burn { from, amount }.publish(&env);
    }

    /// Burn `amount` from `from`, drawing on `spender`'s allowance (SEP-41).
    ///
    /// Screens nobody, for the reason `burn` does not (ADR-004): value is
    /// destroyed rather than moved, so there is no counterparty to screen.
    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::spend_allowance(&env, &from, &spender, amount);

        let bal = Self::balance(env.clone(), from.clone());
        if bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        let total = Self::total_supply(env.clone());
        let new_bal = bal - amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_bal);
        env.storage().persistent().set(
            &DataKey::TotalSupply,
            &Self::checked_sub(&env, total, amount),
        );

        // Holder count decrements when a balance hits zero.
        if new_bal == 0 {
            Self::decrement_holder_count(&env);
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::TotalSupply);

        events::Burn { from, amount }.publish(&env);
    }

    // ── Transfers ─────────────────────────────────────────────────────────────

    /// Transfer tokens from caller to recipient.
    ///
    /// `to` may be a muxed address, as SEP-41 requires. The balance belongs to
    /// the underlying address; the muxed id is carried in the event for the
    /// recipient's off-chain bookkeeping.
    pub fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        let to_muxed_id = to.id();
        let to = to.address();
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);
        Self::require_not_frozen(&env, &from);
        Self::require_not_locked(&env, &from);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        // A self-transfer must be a no-op. Writing both legs would target the
        // same storage key, and the credit would overwrite the debit and mint
        // `amount` out of nothing.
        if from == to {
            return;
        }

        let to_bal = Self::balance(env.clone(), to.clone());
        let to_new = Self::checked_add(&env, to_bal, amount);
        let from_new = from_bal - amount;

        // Reject if this transfer would add a new holder and exceed the cap.
        if to_bal == 0 {
            Self::require_holder_cap_not_exceeded(&env);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &from_new);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_new);

        // Update holder count: `to` gains a position (0 → >0), `from` may lose one.
        if to_bal == 0 {
            Self::increment_holder_count(&env);
        }
        if from_new == 0 {
            Self::decrement_holder_count(&env);
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::Balance(to.clone()));

        events::Transfer {
            from,
            to,
            amount,
            to_muxed_id,
        }
        .publish(&env);
    }

    // ── Allowances ────────────────────────────────────────────────────────────

    /// Set `spender`'s allowance over `owner`'s balance, spendable through
    /// ledger `live_until_ledger` (SEP-41).
    ///
    /// The expiry bounds how long a forgotten approval stays spendable (IR-09).
    /// It may already be past only when revoking with `amount` 0.
    ///
    /// A new amount replaces the old one outright, so a spender who sees the
    /// change coming can spend the old allowance first. To lower an allowance
    /// safely, set it to 0, confirm, then set the new amount.
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        live_until_ledger: u32,
    ) {
        owner.require_auth();
        Self::require_not_paused(&env);
        if amount < 0 {
            panic_with_error!(&env, RwaError::InvalidAmount);
        }
        if amount > 0 && live_until_ledger < env.ledger().sequence() {
            panic_with_error!(&env, RwaError::InvalidExpiration);
        }

        let key = DataKey::Allowance(owner.clone(), spender.clone());
        env.storage().persistent().set(
            &key,
            &AllowanceValue {
                amount,
                live_until_ledger,
            },
        );

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::Approve {
            owner,
            spender,
            amount,
            live_until_ledger,
        }
        .publish(&env);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::require_not_paused(&env);
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);
        Self::require_not_frozen(&env, &from);
        Self::require_not_locked(&env, &from);

        // The spender exercised their authorisation, so the allowance is
        // consumed even when the transfer turns out to be a no-op.
        Self::spend_allowance(&env, &from, &spender, amount);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        // A self-transfer moves nothing. Writing both legs would target the
        // same storage key, and the credit would overwrite the debit and mint
        // `amount` out of nothing. Like `transfer`'s no-op it publishes
        // nothing either, so indexers do not record a settlement that did not
        // happen (IR-15).
        if from == to {
            extend_instance(&env);
            return;
        }

        let to_bal = Self::balance(env.clone(), to.clone());
        let to_new = Self::checked_add(&env, to_bal, amount);
        let from_new = from_bal - amount;

        // Reject if this transfer would add a new holder and exceed the cap.
        if to_bal == 0 {
            Self::require_holder_cap_not_exceeded(&env);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &from_new);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_new);

        // Update holder count: `to` gains a position, `from` may lose one.
        if to_bal == 0 {
            Self::increment_holder_count(&env);
        }
        if from_new == 0 {
            Self::decrement_holder_count(&env);
        }

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::Balance(to.clone()));

        events::TransferFrom { from, to, amount }.publish(&env);
    }

    // ── Forced Transfers ──────────────────────────────────────────────────────

    /// Move `amount` from `from` to `to` without `from`'s authorisation.
    ///
    /// Callable only by an address holding the forced-transfer role, which the
    /// admin grants through `set_forced_transfer_role`. This is the on-chain
    /// hook for court-ordered recovery: a regulated issuer must be able to
    /// return tokens to a rightful owner when a legal order says so, even if
    /// the current holder cannot or will not sign.
    ///
    /// `reason_hash` is a hash of the off-chain legal order (e.g. a SHA-256 of
    /// the court document). It is not interpreted on-chain; it is emitted in
    /// the [`events::ForcedTransfer`] event so that every forced move is
    /// auditable against the order that justified it. Every call emits the
    /// event, including a self-transfer, so the log is complete.
    ///
    /// Pausing does not block this: a pause is an operational safety valve,
    /// not a legal one, and a court order must still be executable while
    /// transfers are otherwise halted. Compliance screening still applies to
    /// both parties, matching `transfer`.
    pub fn forced_transfer(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        amount: i128,
        reason_hash: Bytes,
    ) {
        operator.require_auth();
        if !Self::is_forced_transfer_role(env.clone(), operator.clone()) {
            panic_with_error!(&env, RwaError::NotForcedTransferRole);
        }
        if reason_hash.is_empty() {
            panic_with_error!(&env, RwaError::InvalidReasonHash);
        }
        Self::require_positive(&env, amount);
        Self::require_compliant(&env, &from);
        Self::require_compliant(&env, &to);

        let from_bal = Self::balance(env.clone(), from.clone());
        if from_bal < amount {
            panic_with_error!(&env, RwaError::InsufficientBalance);
        }

        // A self-transfer moves nothing. The event is still published so the
        // call is logged, but no balance is written: writing both legs would
        // target the same storage key and mint `amount` out of nothing.
        if from != to {
            let to_bal = Self::balance(env.clone(), to.clone());
            let to_new = Self::checked_add(&env, to_bal, amount);
            let from_new = from_bal - amount;

            // Reject if this forced transfer would add a new holder and exceed the cap.
            if to_bal == 0 {
                Self::require_holder_cap_not_exceeded(&env);
            }

            env.storage()
                .persistent()
                .set(&DataKey::Balance(from.clone()), &from_new);
            env.storage()
                .persistent()
                .set(&DataKey::Balance(to.clone()), &to_new);

            // Update holder count: `to` gains a position, `from` may lose one.
            if to_bal == 0 {
                Self::increment_holder_count(&env);
            }
            if from_new == 0 {
                Self::decrement_holder_count(&env);
            }

            extend_persistent(&env, &DataKey::Balance(from.clone()));
            extend_persistent(&env, &DataKey::Balance(to.clone()));
        }

        extend_instance(&env);

        events::ForcedTransfer {
            operator,
            from,
            to,
            amount,
            reason_hash,
        }
        .publish(&env);
    }

    // ── Read-only Views ───────────────────────────────────────────────────────

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner))
            .unwrap_or(0)
    }

    /// What `spender` may still spend from `owner`: zero once the allowance's
    /// `live_until_ledger` has passed.
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        Self::read_allowance(&env, &DataKey::Allowance(owner, spender)).amount
    }

    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0)
    }

    /// The number of addresses currently holding a strictly positive balance.
    ///
    /// Incremented when a balance transitions 0 → >0 (first receive), and
    /// decremented when a balance hits exactly 0. Used for Reg D–style
    /// holder-count limits; an optional ceiling is set via `set_holder_cap`.
    pub fn holder_count(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::HolderCount)
            .unwrap_or(0)
    }

    /// The maximum number of distinct holders the admin has configured, or
    /// `None` if the contract is uncapped.
    pub fn holder_count_cap(env: Env) -> Option<u32> {
        env.storage().instance().get(&HOLDER_CAP_KEY)
    }

    pub fn metadata(env: Env) -> AssetMetadata {
        match env.storage().persistent().get(&DataKey::Metadata) {
            Some(m) => m,
            None => panic_with_error!(&env, RwaError::NotInitialized),
        }
    }

    /// How many times metadata has been written: 1 right after construction,
    /// incremented by one on every `update_metadata` or governance-approved
    /// cap increase (#149). An integrator diffing two reads can tell from the
    /// version alone whether anything changed, without comparing every field.
    pub fn metadata_version(env: Env) -> u32 {
        match env.storage().persistent().get(&DataKey::MetadataVersion) {
            Some(v) => v,
            None => panic_with_error!(&env, RwaError::NotInitialized),
        }
    }

    /// SEP-41 metadata getters, each a field of [`AssetMetadata`].
    pub fn decimals(env: Env) -> u32 {
        Self::metadata(env).decimals
    }

    pub fn name(env: Env) -> String {
        Self::metadata(env).name
    }

    pub fn symbol(env: Env) -> String {
        Self::metadata(env).symbol
    }

    pub fn admin(env: Env) -> Address {
        match env.storage().instance().get(&ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, RwaError::NotInitialized),
        }
    }

    pub fn paused(env: Env) -> bool {
        env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
    }

    // ── Admin Operations ──────────────────────────────────────────────────────

    pub fn set_paused(env: Env, paused: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&PAUSED_KEY, &paused);
        extend_instance(&env);
        events::Paused { paused }.publish(&env);
    }

    // ── Holder Cap (#146) ─────────────────────────────────────────────────────

    /// Set or remove a ceiling on the number of distinct token holders.
    ///
    /// Pass `Some(n)` to cap the number of addresses that may hold a positive
    /// balance at `n`. Any `mint`, `transfer`, `transfer_from` or
    /// `forced_transfer` that would create a *new* holder when the count
    /// already equals the cap is rejected with `ExceedsHolderCap`.
    ///
    /// Pass `None` to lift the cap entirely. The cap may be tightened below the
    /// current holder count — doing so freezes new entries rather than evicting
    /// existing ones.
    ///
    /// Only the admin may call this.
    pub fn set_holder_cap(env: Env, cap: Option<u32>) {
        Self::require_admin(&env);
        match cap {
            Some(c) => env.storage().instance().set(&HOLDER_CAP_KEY, &c),
            None => env.storage().instance().remove(&HOLDER_CAP_KEY),
        }
        extend_instance(&env);
        events::HolderCapSet { cap }.publish(&env);
    }

    pub fn update_metadata(env: Env, metadata: AssetMetadata) {
        Self::require_admin(&env);
        Self::validate_metadata(&env, &metadata);
        let previous = Self::metadata(env.clone());
        Self::validate_metadata_change(&env, &previous, &metadata);
        Self::replace_metadata(&env, previous, metadata);
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
    /// signature. What changes is that the two signatures no longer need to
    /// land in the same transaction, which is what makes this reachable from
    /// the SDK at all, rather than `stellar-cli` only.
    ///
    /// Calling this again before `accept_admin` replaces the pending
    /// proposal rather than requiring `cancel_admin_transfer` first.
    pub fn propose_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        let current = Self::admin(env.clone());
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
            None => panic_with_error!(&env, RwaError::NoPendingAdminTransfer),
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
        Self::require_admin(&env);
        let pending: Address = match env.storage().instance().get(&PENDING_ADMIN_KEY) {
            Some(a) => a,
            None => panic_with_error!(&env, RwaError::NoPendingAdminTransfer),
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

    // ── Compliance Configuration ──────────────────────────────────────────────

    /// Point the asset at a compliance contract, or pass `None` to disable
    /// screening entirely.
    ///
    /// `min_level` is the verification level both parties to a transfer must
    /// meet. It mirrors `ComplianceContract`'s scale: 0 none, 1 basic, 2 full,
    /// 3 accredited.
    pub fn set_compliance(env: Env, compliance: Option<Address>, min_level: u32) {
        Self::require_admin(&env);
        if min_level > MAX_COMPLIANCE_LEVEL {
            panic_with_error!(&env, RwaError::InvalidComplianceLevel);
        }
        match &compliance {
            Some(addr) => env.storage().instance().set(&COMPLIANCE_KEY, addr),
            None => env.storage().instance().remove(&COMPLIANCE_KEY),
        }
        env.storage().instance().set(&MIN_LEVEL_KEY, &min_level);
        extend_instance(&env);

        events::ComplianceSet {
            compliance,
            min_level,
        }
        .publish(&env);
    }

    pub fn compliance_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&COMPLIANCE_KEY)
    }

    pub fn min_compliance_level(env: Env) -> u32 {
        env.storage().instance().get(&MIN_LEVEL_KEY).unwrap_or(0)
    }

    // ── Supply Cap Governance (#148) ──────────────────────────────────────────
    //
    // `update_metadata` (above) only ever tightens a cap, enforcing PRD §10.2's
    // "not bypassable by admin" on the admin's own, single-signature path. That
    // promise was never meant to make a cap permanently stuck once circulating
    // supply legitimately needs to grow beyond it — only to stop one key doing
    // it alone. This section adds the one path that can loosen a cap: the admin
    // proposes a specific new ceiling, and a second, independently configured
    // address (`governance`) must separately authorize it before it takes
    // effect. Tightening never needed this; raising a cap dilutes every holder
    // and so is held to the higher bar `update_metadata` deliberately refuses.

    /// Names the address that may approve loosening a supply cap, or pass
    /// `None` to disable loosening entirely (the default: a cap can then only
    /// ever be tightened, exactly as before this feature existed).
    ///
    /// This address is deliberately not required to be the `governance`
    /// contract: Phase 1/2 governance verifies no vote weight and executes
    /// nothing (see `governance`'s crate docs), so gating real dilution power
    /// on it would gate it on nothing. An operator can point this at a
    /// multisig today and at the governance contract once Phase 3 makes its
    /// outcomes binding — the contract does not need to know which.
    pub fn set_governance(env: Env, governance: Option<Address>) {
        Self::require_admin(&env);
        match &governance {
            Some(addr) => env.storage().instance().set(&GOVERNANCE_KEY, addr),
            None => env.storage().instance().remove(&GOVERNANCE_KEY),
        }
        extend_instance(&env);

        events::GovernanceSet { governance }.publish(&env);
    }

    pub fn governance(env: Env) -> Option<Address> {
        env.storage().instance().get(&GOVERNANCE_KEY)
    }

    /// Admin-initiated first step of loosening a supply cap: proposes
    /// `new_max_supply` as the next cap, replacing the field `update_metadata`
    /// would otherwise refuse to touch. Replaces any proposal already pending
    /// rather than requiring `cancel_cap_increase` first, mirroring
    /// `propose_admin`.
    ///
    /// Rejects anything that is not an actual loosening of a cap that exists:
    /// there must already be a finite cap, and the proposed value must be
    /// `0` (uncapped) or strictly greater than the current cap. A value that
    /// only tightens belongs on `update_metadata`, which needs no second
    /// signature.
    pub fn propose_cap_increase(env: Env, new_max_supply: i128) {
        Self::require_admin(&env);
        if new_max_supply < 0 {
            panic_with_error!(&env, RwaError::InvalidSupplyCap);
        }

        let current = Self::metadata(env.clone());
        let loosens_cap =
            current.max_supply > 0 && (new_max_supply == 0 || new_max_supply > current.max_supply);
        if !loosens_cap {
            panic_with_error!(&env, RwaError::InvalidSupplyCap);
        }

        env.storage()
            .persistent()
            .set(&DataKey::PendingCapIncrease, &new_max_supply);
        extend_instance(&env);
        extend_persistent(&env, &DataKey::PendingCapIncrease);

        events::CapIncreaseProposed {
            proposed_max_supply: new_max_supply,
        }
        .publish(&env);
    }

    /// Second step: `governance`'s own signature finalizes the pending
    /// proposal, applying it the same way `update_metadata` would — through
    /// `replace_metadata`, so this counts as a metadata version bump and
    /// publishes the same `MetadataUpdated` event (#149), rather than a
    /// parallel code path an integrator would also have to watch.
    ///
    /// Re-validates against current circulating supply rather than trusting
    /// the proposal unconditionally: nothing stops supply from moving between
    /// `propose_cap_increase` and this call, within whatever the cap at
    /// proposal time still allowed.
    pub fn approve_cap_increase(env: Env) {
        let governance = match Self::governance(env.clone()) {
            Some(addr) => addr,
            None => panic_with_error!(&env, RwaError::GovernanceNotConfigured),
        };
        governance.require_auth();

        let new_max_supply: i128 =
            match env.storage().persistent().get(&DataKey::PendingCapIncrease) {
                Some(v) => v,
                None => panic_with_error!(&env, RwaError::NoPendingCapIncrease),
            };

        let previous = Self::metadata(env.clone());
        let supply = Self::total_supply(env.clone());
        if new_max_supply > 0 && new_max_supply < supply {
            panic_with_error!(&env, RwaError::InvalidSupplyCap);
        }

        let mut next = previous.clone();
        next.max_supply = new_max_supply;

        env.storage()
            .persistent()
            .remove(&DataKey::PendingCapIncrease);
        extend_instance(&env);

        Self::replace_metadata(&env, previous, next);
    }

    /// Withdraws a pending cap increase before `governance` approves it. Only
    /// the current admin may call this, mirroring `cancel_admin_transfer`:
    /// the admin proposed it and may change its mind, and nothing here
    /// requires `governance`'s signature to withdraw what has not taken
    /// effect yet.
    pub fn cancel_cap_increase(env: Env) {
        Self::require_admin(&env);
        let proposed_max_supply: i128 =
            match env.storage().persistent().get(&DataKey::PendingCapIncrease) {
                Some(v) => v,
                None => panic_with_error!(&env, RwaError::NoPendingCapIncrease),
            };
        env.storage()
            .persistent()
            .remove(&DataKey::PendingCapIncrease);
        extend_instance(&env);

        events::CapIncreaseCancelled {
            cancelled_proposed_max_supply: proposed_max_supply,
        }
        .publish(&env);
    }

    /// The `max_supply` `approve_cap_increase` would install if called now,
    /// or `None` if no proposal is pending.
    pub fn pending_cap_increase(env: Env) -> Option<i128> {
        env.storage().persistent().get(&DataKey::PendingCapIncrease)
    }

    // ── Private Helpers ───────────────────────────────────────────────────────

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }

    /// The allowance stored at `key`, reading as zero once it has lapsed.
    fn read_allowance(env: &Env, key: &DataKey) -> AllowanceValue {
        match env.storage().persistent().get::<_, AllowanceValue>(key) {
            Some(allowance) if allowance.live_until_ledger >= env.ledger().sequence() => allowance,
            _ => AllowanceValue {
                amount: 0,
                live_until_ledger: 0,
            },
        }
    }

    /// Deducts `amount` from `spender`'s allowance over `from`, keeping its
    /// expiry. Shared by `transfer_from` and `burn_from`.
    fn spend_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
        let key = DataKey::Allowance(from.clone(), spender.clone());
        let current = Self::read_allowance(env, &key);
        if current.amount < amount {
            panic_with_error!(env, RwaError::InsufficientAllowance);
        }
        env.storage().persistent().set(
            &key,
            &AllowanceValue {
                amount: current.amount - amount,
                live_until_ledger: current.live_until_ledger,
            },
        );
        extend_persistent(env, &key);
    }

    fn require_not_paused(env: &Env) {
        if Self::paused(env.clone()) {
            panic_with_error!(env, RwaError::ContractPaused);
        }
    }

    /// Rejects `party` unless it satisfies the configured compliance policy.
    ///
    /// When no compliance contract is configured the check is skipped, so an
    /// asset can be issued and moved before an operator has stood up a KYC
    /// engine. Screening becomes mandatory the moment `set_compliance` names
    /// a contract.
    fn require_compliant(env: &Env, party: &Address) {
        let Some(compliance) = Self::compliance_contract(env.clone()) else {
            return;
        };
        let min_level = Self::min_compliance_level(env.clone());
        let client = ComplianceClient::new(env, &compliance);
        if !client.screen(party, &min_level) {
            panic_with_error!(env, RwaError::NotCompliant);
        }
    }

    /// Rejects transfers from frozen addresses.
    fn require_not_frozen(env: &Env, from: &Address) {
        if Self::is_frozen(env.clone(), from.clone()) {
            panic_with_error!(env, RwaError::AddressFrozen);
        }
    }

    /// Rejects transfers from addresses whose tokens are still locked.
    fn require_not_locked(env: &Env, from: &Address) {
        let lockup_until = Self::lockup_date(env.clone(), from.clone());
        if lockup_until > 0 && lockup_until > env.ledger().sequence() {
            panic_with_error!(env, RwaError::AddressLocked);
        }
    }

    fn require_positive(env: &Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(env, RwaError::InvalidAmount);
        }
    }

    /// Rejects metadata that would make supply accounting unrepresentable.
    fn validate_metadata(env: &Env, metadata: &AssetMetadata) {
        if metadata.decimals > MAX_DECIMALS
            || metadata.max_supply < 0
            || metadata.symbol.is_empty()
            || metadata.name.is_empty()
        {
            panic_with_error!(env, RwaError::InvalidMetadata);
        }
        Self::validate_legal_doc_hash(env, &metadata.legal_doc_hash);
    }

    /// Rejects a `legal_doc_hash` that cannot be a real SHA-256 digest: the
    /// wrong length, or all-zero. Either would let the asset claim a legal
    /// anchor that resolves to nothing (#150); a hash this check accepts can
    /// still be wrong, but it can no longer be absent in disguise.
    fn validate_legal_doc_hash(env: &Env, hash: &Bytes) {
        if hash.len() != LEGAL_DOC_HASH_LEN || hash.iter().all(|byte| byte == 0) {
            panic_with_error!(env, RwaError::InvalidLegalDocHash);
        }
    }

    /// Rejects a metadata change that would alter what holders already own.
    ///
    /// `validate_metadata` judges a value in isolation; this judges it against
    /// the asset as it stands. Without it the admin could re-denominate every
    /// balance or lift the supply cap in one silent call (IR-02), a narrower
    /// form of the "rewrite the rules at any time" power ADR-003 declined to
    /// ship as an upgrade entry point.
    fn validate_metadata_change(env: &Env, current: &AssetMetadata, next: &AssetMetadata) {
        // Balances are stored in base units, so a different `decimals` changes
        // the size of every position without moving a single token.
        if next.decimals != current.decimals {
            panic_with_error!(env, RwaError::DecimalsImmutable);
        }

        // The cap is the holders' ceiling on dilution, and PRD §10.2 promises it
        // is "not bypassable by admin [alone]". So the admin's own signature can
        // only tighten a cap here: never raise it, never lift it to uncapped,
        // and never set it below what is already in circulation. An uncapped
        // asset may be given a cap at or above its supply. Raising a cap is
        // still possible, but only through `propose_cap_increase` /
        // `approve_cap_increase` (#148), which requires a second, separately
        // configured `governance` signature the admin alone cannot produce.
        let supply = Self::total_supply(env.clone());
        let loosens_cap = current.max_supply > 0
            && (next.max_supply == 0 || next.max_supply > current.max_supply);
        let below_supply = next.max_supply > 0 && next.max_supply < supply;
        if loosens_cap || below_supply {
            panic_with_error!(env, RwaError::InvalidSupplyCap);
        }
    }

    /// Writes `next` as the current metadata, bumps the version counter, and
    /// publishes `MetadataUpdated` carrying both. Shared by `update_metadata`
    /// and `approve_cap_increase` (#148), so every path that changes metadata
    /// keeps the version and the event in lockstep with what is actually
    /// stored, rather than each path reimplementing it slightly differently.
    ///
    /// Callers are responsible for validating `next` first; this only writes.
    fn replace_metadata(env: &Env, previous: AssetMetadata, next: AssetMetadata) {
        let version = match Self::metadata_version(env.clone()).checked_add(1) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        };

        env.storage().persistent().set(&DataKey::Metadata, &next);
        env.storage()
            .persistent()
            .set(&DataKey::MetadataVersion, &version);

        extend_instance(env);
        extend_persistent(env, &DataKey::Metadata);
        extend_persistent(env, &DataKey::MetadataVersion);

        events::MetadataUpdated {
            previous,
            metadata: next,
            version,
        }
        .publish(env);
    }

    fn checked_add(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        }
    }

    fn checked_sub(env: &Env, a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        }
    }

    // ── Holder count helpers (#146) ───────────────────────────────────────────

    /// Increments the persistent holder count by one. Called when a balance
    /// transitions 0 → >0. The counter is stored as `u32` because Reg D's own
    /// ceiling (2,000 US accredited investors, 500 non-accredited) fits in one
    /// byte; `u32::MAX` is an unreachable upper bound, not a useful cap.
    fn increment_holder_count(env: &Env) {
        let current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::HolderCount)
            .unwrap_or(0);
        let next = match current.checked_add(1) {
            Some(v) => v,
            None => panic_with_error!(env, RwaError::Overflow),
        };
        env.storage().persistent().set(&DataKey::HolderCount, &next);
        extend_persistent(env, &DataKey::HolderCount);
    }

    /// Decrements the persistent holder count by one. Called when a balance
    /// transitions >0 → 0. Saturates at zero rather than underflowing: the
    /// counter is a derived value, and a corrupt zero is already a problem;
    /// there is nothing useful a contract can do from a negative count.
    fn decrement_holder_count(env: &Env) {
        let current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::HolderCount)
            .unwrap_or(0);
        let next = current.saturating_sub(1);
        env.storage().persistent().set(&DataKey::HolderCount, &next);
        extend_persistent(env, &DataKey::HolderCount);
    }

    /// Fails with `ExceedsHolderCap` when a holder cap is configured and the
    /// current count has reached it. Called before any balance write that
    /// would add a new holder (i.e. the recipient's balance is zero).
    fn require_holder_cap_not_exceeded(env: &Env) {
        let Some(cap) = env.storage().instance().get::<_, u32>(&HOLDER_CAP_KEY) else {
            return; // no cap configured
        };
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::HolderCount)
            .unwrap_or(0);
        if count >= cap {
            panic_with_error!(env, RwaError::ExceedsHolderCap);
        }
    }
}
