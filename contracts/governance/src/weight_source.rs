//! Binding to an external balance source used to cap declared vote weight.
//!
//! Declared as a client interface rather than a direct dependency on the
//! `rwa-asset` crate, for the same reasons ADR-004 gives for `rwa-asset`'s own
//! binding to `compliance`: governance only needs the shape of one call, and
//! an operator may point it at any contract implementing that shape, without
//! this crate being rebuilt when the token contract changes.

use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "WeightSourceClient")]
pub trait WeightSourceInterface {
    /// Returns `id`'s current balance in the weight-source contract.
    fn balance(env: Env, id: Address) -> i128;
}
