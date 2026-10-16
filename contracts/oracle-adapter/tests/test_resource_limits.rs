//! Resource-limit regression tests under mainnet budget settings (#122).
//!
//! See `contracts/governance/tests/test_resource_limits.rs` for the full
//! rationale shared by both files: the internal review's mainnet-budget
//! proof-of-concept for IR-01/IR-02 was deleted after use
//! (`docs/audits/2026-09-14-internal-review-phase1.md`, Method), and nothing
//! since has tracked whether a contract flow's cost is drifting, only
//! whether a single run fits the limit at all (`test_oracle_adapter.rs`'s
//! `test_prices_under_mainnet_limits_with_max_records` already enforces the
//! limit; it does not print or compare an instruction count).
//!
//! `report` is this contract's price-feed write path and the one most
//! likely to run often on a live deployment, so it is the flow covered here.

#![cfg(test)]

use soroban_sdk::{
    testutils::{cost_estimate::NetworkInvocationResourceLimits, Address as _, Ledger as _},
    Address, Env, Symbol,
};

use oracle_adapter::{Asset, OracleAdapterContract, OracleAdapterContractClient};

const DECIMALS: u32 = 14;
const RESOLUTION: u32 = 300;
const MAX_DEVIATION_BPS: u32 = 1_000;
const NOW: u64 = 1_800_000_123;
const PRICE: i128 = 100 * 10_i128.pow(DECIMALS);

/// How far above its baseline the flow's instruction count may drift before
/// this test fails it as a regression, as a percentage of the baseline. See
/// the governance resource-limit test file for the full rationale.
const REGRESSION_THRESHOLD_PERCENT: u64 = 20;

/// Captured on this development machine's test host; see the module doc
/// comment on why this is a relative, not an absolute, budget check.
const REPORT_BASELINE_INSNS: u64 = 273_498;

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

#[test]
fn test_report_fits_the_mainnet_budget() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = NOW);

    let admin = Address::generate(&env);
    let base = Asset::Other(Symbol::new(&env, "USD"));
    let contract_id = env.register(
        OracleAdapterContract,
        (&admin, &base, &DECIMALS, &RESOLUTION),
    );
    let client = OracleAdapterContractClient::new(&env, &contract_id);

    let asset = Asset::Stellar(Address::generate(&env));
    client.add_asset(&asset, &MAX_DEVIATION_BPS);
    let reporter = Address::generate(&env);
    client.set_reporter(&asset, &reporter, &true);

    env.cost_estimate().budget().reset_default();
    env.cost_estimate()
        .enforce_resource_limits(NetworkInvocationResourceLimits::mainnet());
    client.report(&reporter, &asset, &PRICE, &NOW);

    let insns = env.cost_estimate().budget().cpu_instruction_cost();
    assert_within_budget("oracle-adapter.report", insns, REPORT_BASELINE_INSNS);
}
