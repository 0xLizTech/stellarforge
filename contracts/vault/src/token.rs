//! Binding to the underlying token contract.
//!
//! Used at construction to query the underlying token's `decimals()`, which
//! the vault adopts for its share token and stores immutably, and during
//! `deposit` and `redeem` to transfer the underlying asset.

use soroban_sdk::{contractclient, Address, Env, MuxedAddress};

#[contractclient(name = "UnderlyingTokenClient")]
pub trait UnderlyingTokenInterface {
    /// Returns the number of decimal places for the token.
    fn decimals(env: Env) -> u32;

    /// Returns the balance of the specified account.
    fn balance(env: Env, id: Address) -> i128;

    /// Transfer tokens from `from` to `to`.
    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128);
}
