//! Binding to the external compliance contract.
//!
//! Declared as a client interface rather than a direct dependency on the
//! `compliance` crate: the asset only needs the shape of one call, and an
//! operator may point it at any contract implementing that shape — a
//! jurisdiction-specific engine, or a mock during testing — without this crate
//! being rebuilt.

use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "ComplianceClient")]
pub trait ComplianceInterface {
    /// Returns true when `subject` holds a valid, unexpired verification
    /// record at or above `min_level`.
    ///
    /// Deliberately the screening entry point rather than a pure query: a
    /// verification record is written once and thereafter only read, so the
    /// compliance contract needs a moment at which it may refresh the record's
    /// lifetime, and only that contract can extend its own entries. Screening
    /// happens inside a transfer, which is already paying for a ledger write.
    fn screen(env: Env, subject: Address, min_level: u32) -> bool;
}
