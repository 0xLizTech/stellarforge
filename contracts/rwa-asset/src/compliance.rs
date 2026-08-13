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
    fn is_compliant(env: Env, subject: Address, min_level: u32) -> bool;
}
