//! Resource-limit regression tests under mainnet budget settings (#122).
//!
//! The internal review's proof-of-concept for IR-01 and IR-02 ran under
//! `InvocationResourceLimits::mainnet()` and was deleted afterwards
//! (`docs/audits/2026-09-14-internal-review-phase1.md`, Method). Every other
//! test in `test_governance.rs` runs with the test host's default, generous
//! budget, so none of them would catch a flow quietly growing past what
//! mainnet actually allows, let alone growing it gradually enough that no
//! single change crosses the mainnet ceiling outright.
//!
//! This file enforces `NetworkInvocationResourceLimits::mainnet()` on
//! governance's key flows (`propose`, `vote`, `finalize`) and prints the
//! instruction count each one consumes, so it is visible in the CI log
//! (`cargo test` runs with `--nocapture` are not required: Rust's test
//! harness prints captured stdout for a test on failure, and the regression
//! assertion below is what turns a budget increase into a failure). Each
//! flow also has a baseline below; crossing that baseline by more than
//! `REGRESSION_THRESHOLD_PERCENT` fails the test, so a quiet jump in cost
//! cannot merge unnoticed.
//!
//! The baselines are captured on this test's own development machine running
//! this crate's own test host, not on-chain: `soroban-sdk`'s own doc comment
//! on `cpu_instruction_cost` warns costs "are likely to be underestimated
//! when running Rust code compared to running the WASM equivalent." They are
//! for catching a *relative* regression between runs of the same host, not
//! for asserting an absolute mainnet budget number; the `mainnet()` resource
//! limits enforced below are what actually gates "fits in one mainnet
//! invocation."
//!
//! To refresh a baseline after a deliberate change to one of these flows,
//! run this file once, read the printed instruction count for the flow that
//! changed, and update its constant below with a short note on why it moved.

#![cfg(test)]

use soroban_sdk::{
    testutils::{cost_estimate::NetworkInvocationResourceLimits, Address as _, Ledger as _},
    Address, Bytes, Env, String,
};

use governance::{GovernanceContract, GovernanceContractClient};

const VOTING_PERIOD: u32 = 17_280; // governance::MIN_VOTING_PERIOD_LEDGERS

/// How far above its baseline a flow's instruction count may drift before
/// this test fails it as a regression, as a percentage of the baseline.
const REGRESSION_THRESHOLD_PERCENT: u64 = 20;

// Baselines captured on this development machine's test host (see the module
// doc comment). Updated after #134 (deposit fields), #135 (quorum and pass
// threshold evaluation), and #136 (SHA-256 content-hash duplicate check on
// propose and active-hash removal on finalize).
const PROPOSE_BASELINE_INSNS: u64 = 194_672;
const VOTE_BASELINE_INSNS: u64 = 179_636;
const FINALIZE_BASELINE_INSNS: u64 = 145_748;

struct Harness<'a> {
    env: Env,
    client: GovernanceContractClient<'a>,
}

fn setup() -> Harness<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(GovernanceContract, (&admin,));
    let client = GovernanceContractClient::new(&env, &contract_id);

    Harness { env, client }
}

/// Asserts `actual` is at or under `baseline` plus `REGRESSION_THRESHOLD_PERCENT`,
/// and prints both so a CI log shows the real number even when the test
/// passes comfortably under the threshold.
fn assert_within_budget(flow: &str, actual: u64, baseline: u64) {
    let allowed = baseline + (baseline * REGRESSION_THRESHOLD_PERCENT) / 100;
    println!(
        "[resource-limits] {flow}: {actual} instructions (baseline {baseline}, \
         allowed up to {allowed} at {REGRESSION_THRESHOLD_PERCENT}% over baseline)"
    );
    assert!(
        actual <= allowed,
        "{flow} consumed {actual} instructions, more than {REGRESSION_THRESHOLD_PERCENT}% \
         over its {baseline}-instruction baseline (allowed up to {allowed}). If this is a \
         deliberate change, update {flow}'s baseline constant in this file with a note on why \
         it moved; if not, this flow regressed."
    );
}

fn enforce_mainnet_limits(env: &Env) {
    env.cost_estimate().budget().reset_default();
    env.cost_estimate()
        .enforce_resource_limits(NetworkInvocationResourceLimits::mainnet());
}

#[test]
fn test_propose_fits_the_mainnet_budget() {
    let h = setup();
    let proposer = Address::generate(&h.env);

    enforce_mainnet_limits(&h.env);
    h.client.propose(
        &proposer,
        &String::from_str(&h.env, "Raise the protocol fee"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );

    let insns = h.env.cost_estimate().budget().cpu_instruction_cost();
    assert_within_budget("governance.propose", insns, PROPOSE_BASELINE_INSNS);
}

#[test]
fn test_vote_fits_the_mainnet_budget() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.client.propose(
        &proposer,
        &String::from_str(&h.env, "Raise the protocol fee"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );
    let voter = Address::generate(&h.env);

    enforce_mainnet_limits(&h.env);
    h.client.vote(&voter, &id, &true, &10_i128);

    let insns = h.env.cost_estimate().budget().cpu_instruction_cost();
    assert_within_budget("governance.vote", insns, VOTE_BASELINE_INSNS);
}

#[test]
fn test_finalize_fits_the_mainnet_budget() {
    let h = setup();
    let proposer = Address::generate(&h.env);
    let id = h.client.propose(
        &proposer,
        &String::from_str(&h.env, "Raise the protocol fee"),
        &Bytes::from_array(&h.env, &[7u8; 32]),
        &VOTING_PERIOD,
    );
    h.client
        .vote(&Address::generate(&h.env), &id, &true, &10_i128);
    h.env
        .ledger()
        .with_mut(|li| li.sequence_number += VOTING_PERIOD + 1);

    enforce_mainnet_limits(&h.env);
    h.client.finalize(&id);

    let insns = h.env.cost_estimate().budget().cpu_instruction_cost();
    assert_within_budget("governance.finalize", insns, FINALIZE_BASELINE_INSNS);
}
