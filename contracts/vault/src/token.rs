//! Binding to the underlying token contract.
//!
//! Used at construction to query the underlying token's `decimals()`, which
//! the vault adopts for its share token and stores immutably.

use soroban_sdk::{contractclient, Env};

#[contractclient(name = "UnderlyingTokenClient")]
pub trait UnderlyingTokenInterface {
    /// Returns the number of decimal places for the token.
    fn decimals(env: Env) -> u32;
}
