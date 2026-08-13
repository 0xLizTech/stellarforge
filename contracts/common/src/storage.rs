//! Storage time-to-live policy, shared by every StellarForge contract.
//!
//! Soroban archives ledger entries that are not periodically extended. Since
//! protocol 23 an archived persistent entry is restored automatically by the
//! transaction that accesses it, so archival costs a restore fee charged to
//! whoever touches the entry first rather than failing outright. The goal of
//! this policy is therefore to keep that fee from ever being incurred, not to
//! prevent a hard failure.
//!
//! Write paths extend the entries they touch, and they extend to the network
//! maximum: an entry that is written once and then only read has no other
//! opportunity to be refreshed, and paying the rent up front is cheaper than
//! turning every read into a ledger write. Read paths deliberately do **not**
//! extend, so queries stay pure.
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

/// How far a persistent entry is allowed to decay before a write path
/// refreshes it. Re-extending on every single write would charge rent for
/// ledgers the entry already has.
pub const PERSISTENT_REFRESH_INTERVAL: u32 = 30 * DAY_IN_LEDGERS;

/// How far ahead persistent entries are extended: the network maximum.
///
/// Read from the ledger rather than hardcoded, because the maximum is a
/// network parameter and `extend_ttl` beyond it is not accepted.
pub fn persistent_bump_amount(env: &Env) -> u32 {
    env.storage().max_ttl()
}

/// Remaining TTL below which a persistent extension is issued.
pub fn persistent_lifetime_threshold(env: &Env) -> u32 {
    persistent_bump_amount(env).saturating_sub(PERSISTENT_REFRESH_INTERVAL)
}

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
///
/// Call this from write paths only. Calling it from a read makes that read a
/// ledger write, which costs the caller a fee and breaks the expectation that
/// a query is free of side effects.
pub fn extend_persistent<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    if env.storage().persistent().has(key) {
        env.storage().persistent().extend_ttl(
            key,
            persistent_lifetime_threshold(env),
            persistent_bump_amount(env),
        );
    }
}
