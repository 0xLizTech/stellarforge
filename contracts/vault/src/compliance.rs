//! Binding to the external compliance contract.
//!
//! Declared as a client interface following ADR-004: the vault only needs the
//! shape of one call (`screen`), and an operator may point it at any contract
//! implementing that shape without rebuilding this crate.

use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "ComplianceClient")]
pub trait ComplianceInterface {
    /// Returns true when `subject` holds a valid, unexpired verification
    /// record at or above `min_level`.
    fn screen(env: Env, subject: Address, min_level: u32) -> bool;
}
