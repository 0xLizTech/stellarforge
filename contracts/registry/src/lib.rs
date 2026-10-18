#![no_std]

//! Asset Registry — tracks all deployed RWA asset contracts.
//! Phase 1 skeleton: registration + lookup only.

mod error;
mod events;

pub use error::RegistryError;
pub use events::{
    ActiveSet, AdminTransferCancelled, AdminTransferProposed, AdminTransferred, AssetRegistered,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};
use stellarforge_common::{extend_instance, extend_persistent};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const PENDING_ADMIN_KEY: Symbol = symbol_short!("PADMIN");

/// Largest page `list_assets` will return.
///
/// Each listed address is its own ledger read, and a read-only simulation is
/// held to the same per-invocation read limits as a transaction, so a page has
/// to stay well inside them.
pub const MAX_PAGE_SIZE: u32 = 100;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Asset(Address),
    /// Number of distinct assets ever registered, which is also the next free
    /// directory index.
    AssetCount,
    /// The asset registered `n`th, counting from zero. Indices are never
    /// reused or removed.
    AssetAt(u32),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AssetEntry {
    pub contract: Address,
    pub asset_class: soroban_sdk::String,
    pub active: bool,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
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
        env.storage().persistent().set(&DataKey::AssetCount, &0_u32);

        extend_instance(&env);
        extend_persistent(&env, &DataKey::AssetCount);
    }

    /// Record an asset contract, or update an already-registered one.
    ///
    /// An entry records what the admin asserted, and nothing more. `register`
    /// does not check that `contract` is a deployed `rwa-asset` or that
    /// `asset_class` matches that asset's metadata, and no contract reads
    /// `active`. Integrators must not treat an entry as proof that an asset is
    /// genuine or current (IR-16).
    ///
    /// The directory is one entry per asset rather than a single list. A list
    /// held in one ledger entry hit the 64 KiB entry size limit at around
    /// 1,600 assets, after which every registration failed permanently
    /// (IR-01), and each registration rewrote the whole list on the way there.
    pub fn register(env: Env, entry: AssetEntry) {
        Self::require_admin(&env);
        let key = DataKey::Asset(entry.contract.clone());

        // The directory index is appended to only for a genuinely new asset.
        // Appending unconditionally listed the same address once per call, so
        // re-registering an asset to correct its class silently duplicated it
        // for every consumer iterating the directory.
        let is_new = !env.storage().persistent().has(&key);
        let contract = entry.contract.clone();
        env.storage().persistent().set(&key, &entry);

        if is_new {
            let index = Self::asset_count(env.clone());
            let next = match index.checked_add(1) {
                Some(v) => v,
                None => panic_with_error!(&env, RegistryError::Overflow),
            };
            let at = DataKey::AssetAt(index);
            env.storage().persistent().set(&at, &contract);
            env.storage().persistent().set(&DataKey::AssetCount, &next);
            extend_persistent(&env, &at);
            extend_persistent(&env, &DataKey::AssetCount);
        }

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::AssetRegistered {
            contract: entry.contract,
            asset_class: entry.asset_class,
            active: entry.active,
            is_new,
        }
        .publish(&env);
    }

    /// A pure query: registry lookups do not extend the entries they read, so
    /// they cost the caller nothing. Write paths extend to the network maximum
    /// instead.
    pub fn get_asset(env: Env, contract: Address) -> Option<AssetEntry> {
        env.storage().persistent().get(&DataKey::Asset(contract))
    }

    /// Registered assets in registration order, `limit` of them starting at
    /// index `start`.
    ///
    /// A page shorter than `limit` means the end of the directory was reached;
    /// a `start` at or past [`RegistryContract::asset_count`] returns an empty
    /// page. `limit` above [`MAX_PAGE_SIZE`] is rejected rather than quietly
    /// truncated, so a caller cannot mistake a clipped page for the whole
    /// directory.
    pub fn list_assets(env: Env, start: u32, limit: u32) -> Vec<Address> {
        if limit > MAX_PAGE_SIZE {
            panic_with_error!(&env, RegistryError::PageTooLarge);
        }

        let end = start
            .saturating_add(limit)
            .min(Self::asset_count(env.clone()));
        let mut page = Vec::new(&env);
        for index in start..end {
            // Every index below the count was written by `register` and
            // nothing removes one, so this is always present.
            if let Some(asset) = env.storage().persistent().get(&DataKey::AssetAt(index)) {
                page.push_back(asset);
            }
        }
        page
    }

    /// How many distinct assets have been registered. Valid `list_assets`
    /// indices run from 0 to one less than this.
    pub fn asset_count(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetCount)
            .unwrap_or(0)
    }

    pub fn set_active(env: Env, contract: Address, active: bool) {
        Self::require_admin(&env);
        let key = DataKey::Asset(contract.clone());
        let mut entry: AssetEntry = match env.storage().persistent().get(&key) {
            Some(e) => e,
            None => panic_with_error!(&env, RegistryError::AssetNotFound),
        };
        entry.active = active;
        env.storage().persistent().set(&key, &entry);

        extend_instance(&env);
        extend_persistent(&env, &key);

        events::ActiveSet { contract, active }.publish(&env);
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
            None => panic_with_error!(&env, RegistryError::NoPendingAdminTransfer),
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
            None => panic_with_error!(&env, RegistryError::NoPendingAdminTransfer),
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
            None => panic_with_error!(&env, RegistryError::NotInitialized),
        }
    }

    fn require_admin(env: &Env) {
        Self::admin(env.clone()).require_auth();
    }
}
