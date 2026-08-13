//! Storage time-to-live policy, shared by every StellarForge contract.
//!
//! Soroban archives ledger entries that are not periodically extended. An
//! archived entry is not lost, but it cannot be read until it is restored, so
//! a contract that never extends its own entries will start failing for
//! long-idle data.
//!
//! Write paths must extend what they touch. Read paths must extend too
//! whenever an entry can be written once and then only read — a KYC record or
//! a registry entry is set once and consulted forever, so extending on write
//! alone would let it archive while still in active use. Extending during a
//! read is a real ledger write, which is why it is applied deliberately rather
//! than everywhere.
//!
//! This policy lives in one crate rather than being copied per contract:
//! SF-2026-002 was a missing extension, and four independent copies of the
//! constants would be four chances to reintroduce it.

use soroban_sdk::{Env, IntoVal, Val};

/// Ledgers closed per day at the ~5 second target close time.
pub const DAY_IN_LEDGERS: u32 = 17_280;

/// How far ahead instance storage (admin, pause flag) is extended.
pub const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
/// Remaining TTL below which an instance extension is issued.
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// How far ahead persistent entries (balances, allowances, metadata) are
/// extended. Longer than instance storage because a holder may legitimately
/// not transact for months.
pub const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Remaining TTL below which a persistent extension is issued.
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Extends the contract instance and its code.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Extends a single persistent entry, if it exists.
///
/// Extending a key that was never written is a no-op rather than an error, so
/// callers do not need to check existence first.
pub fn extend_persistent<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    if env.storage().persistent().has(key) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}
