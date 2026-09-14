#![no_std]

mod compliance;
mod error;
mod events;

pub use compliance::{ComplianceClient, ComplianceInterface};
pub use error::RwaError;
pub use events::{
    AdminTransferred, Approve, Burn, ComplianceSet, IssuerSet, MetadataUpdated, Mint, Paused,
    Transfer, TransferFrom,
};

use stellarforge_common::{extend_instance, extend_persistent};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Bytes, Env,
    MuxedAddress, String, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
/// Address of the compliance contract, absent when screening is disabled.
const COMPLIANCE_KEY: Symbol = symbol_short!("COMPLY");
/// Minimum verification level a party must hold to send or receive.
const MIN_LEVEL_KEY: Symbol = symbol_short!("MINLVL");

/// Upper bound on `AssetMetadata::decimals`. Beyond this a whole token no
/// longer fits sensibly in `i128` alongside realistic supply figures.
const MAX_DECIMALS: u32 = 18;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Metadata,
    Balance(Address),
    TotalSupply,
    Allowance(Address, Address),
    Issuer(Address),
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
    /// Off-chain legal document hash (IPFS CID or SHA-256)
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
            .set(&DataKey::TotalSupply, &0_i128);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Metadata);
        extend_persistent(&env, &DataKey::TotalSupply);
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

        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_bal);
        env.storage()
            .persistent()
            .set(&DataKey::TotalSupply, &new_total);

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

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(bal - amount));
        env.storage().persistent().set(
            &DataKey::TotalSupply,
            &Self::checked_sub(&env, total, amount),
        );

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

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(bal - amount));
        env.storage().persistent().set(
            &DataKey::TotalSupply,
            &Self::checked_sub(&env, total, amount),
        );

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

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_new);

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

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &to_new);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Balance(from.clone()));
        extend_persistent(&env, &DataKey::Balance(to.clone()));

        events::TransferFrom { from, to, amount }.publish(&env);
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

    pub fn metadata(env: Env) -> AssetMetadata {
        match env.storage().persistent().get(&DataKey::Metadata) {
            Some(m) => m,
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

    pub fn update_metadata(env: Env, metadata: AssetMetadata) {
        Self::require_admin(&env);
        Self::validate_metadata(&env, &metadata);
        let previous = Self::metadata(env.clone());
        Self::validate_metadata_change(&env, &previous, &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::Metadata, &metadata);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::Metadata);

        events::MetadataUpdated { previous, metadata }.publish(&env);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        new_admin.require_auth();
        let previous = Self::admin(env.clone());
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
        extend_instance(&env);

        events::AdminTransferred {
            previous,
            new_admin,
        }
        .publish(&env);
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
        // is "not bypassable by admin". So the admin can only tighten a cap:
        // never raise it, never lift it to uncapped, and never set it below what
        // is already in circulation. An uncapped asset may be given a cap at or
        // above its supply.
        let supply = Self::total_supply(env.clone());
        let loosens_cap = current.max_supply > 0
            && (next.max_supply == 0 || next.max_supply > current.max_supply);
        let below_supply = next.max_supply > 0 && next.max_supply < supply;
        if loosens_cap || below_supply {
            panic_with_error!(env, RwaError::InvalidSupplyCap);
        }
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
}
