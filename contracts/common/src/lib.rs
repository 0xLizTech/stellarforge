#![no_std]

//! Shared building blocks for the StellarForge contracts.
//!
//! Only policy that must be identical across contracts belongs here. Contract
//! logic stays in its own crate: the protocol's composability guarantee is
//! that each contract is independently deployable, and this crate is a
//! compile-time library, never a deployed contract.

pub mod storage;

pub use storage::{extend_instance, extend_persistent};
