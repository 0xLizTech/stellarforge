//! Contract event definitions.
//!
//! Without events an indexer, explorer or wallet has no way to reconstruct
//! holder positions short of replaying every transaction and re-simulating
//! contract state. Topic lists follow the Stellar token conventions so
//! existing tooling can consume them unmodified: the operation name first,
//! then the addresses involved, with the amount as the payload.

use soroban_sdk::{contractevent, Address};

#[contractevent(topics = ["transfer"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent(topics = ["mint"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mint {
    #[topic]
    pub issuer: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent(topics = ["burn"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Burn {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

#[contractevent(topics = ["approve"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approve {
    #[topic]
    pub owner: Address,
    #[topic]
    pub spender: Address,
    pub amount: i128,
}

#[contractevent(topics = ["paused"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paused {
    pub paused: bool,
}
