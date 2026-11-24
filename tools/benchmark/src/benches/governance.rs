use governance::{GovernanceContract, GovernanceContractClient, MIN_VOTING_PERIOD_LEDGERS};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, Bytes, String};

use crate::{mainnet_env, measure, Row};

const CONTRACT: &str = "governance";

pub fn run() -> Vec<Row> {
    let mut rows = Vec::new();

    let env = mainnet_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(GovernanceContract, (&admin,));
    rows.push(measure(&env, CONTRACT, "__constructor"));
    let client = GovernanceContractClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let proposal_id = client.propose(
        &proposer,
        &String::from_str(&env, "Raise the protocol fee"),
        &Bytes::from_array(&env, &[7u8; 32]),
        &MIN_VOTING_PERIOD_LEDGERS,
    );
    rows.push(measure(&env, CONTRACT, "propose"));

    let voter = Address::generate(&env);
    client.vote(&voter, &proposal_id, &true, &100);
    rows.push(measure(&env, CONTRACT, "vote"));

    client.get_proposal(&proposal_id);
    rows.push(measure(&env, CONTRACT, "get_proposal"));

    client.has_voted(&proposal_id, &voter);
    rows.push(measure(&env, CONTRACT, "has_voted"));

    client.proposal_count();
    rows.push(measure(&env, CONTRACT, "proposal_count"));

    env.ledger()
        .with_mut(|li| li.sequence_number += MIN_VOTING_PERIOD_LEDGERS + 1);
    client.finalize(&proposal_id);
    rows.push(measure(&env, CONTRACT, "finalize"));

    client.admin();
    rows.push(measure(&env, CONTRACT, "admin"));

    let new_admin = Address::generate(&env);
    client.propose_admin(&new_admin);
    rows.push(measure(&env, CONTRACT, "propose_admin"));
    client.accept_admin();
    rows.push(measure(&env, CONTRACT, "accept_admin"));

    rows
}
