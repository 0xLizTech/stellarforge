use compliance::{ComplianceContract, ComplianceContractClient, KycRecord};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String};

use crate::{mainnet_env, measure, Row};

const CONTRACT: &str = "compliance";
const LEVEL_FULL: u32 = 2;
const NEVER_EXPIRES: u64 = 0;

fn record(env: &soroban_sdk::Env) -> KycRecord {
    KycRecord {
        jurisdiction: String::from_str(env, "US"),
        level: LEVEL_FULL,
        expires_at: NEVER_EXPIRES,
    }
}

pub fn run() -> Vec<Row> {
    let mut rows = Vec::new();

    let env = mainnet_env();
    let admin = Address::generate(&env);
    let contract_id = env.register(ComplianceContract, (&admin,));
    rows.push(measure(&env, CONTRACT, "__constructor"));
    let client = ComplianceContractClient::new(&env, &contract_id);

    let subject = Address::generate(&env);
    client.set_kyc(&subject, &record(&env));
    rows.push(measure(&env, CONTRACT, "set_kyc"));

    client.get_kyc(&subject);
    rows.push(measure(&env, CONTRACT, "get_kyc"));

    client.is_compliant(&subject, &LEVEL_FULL);
    rows.push(measure(&env, CONTRACT, "is_compliant"));

    client.screen(&subject, &LEVEL_FULL);
    rows.push(measure(&env, CONTRACT, "screen"));

    client.admin();
    rows.push(measure(&env, CONTRACT, "admin"));

    client.paused();
    rows.push(measure(&env, CONTRACT, "paused"));

    client.set_paused(&true);
    rows.push(measure(&env, CONTRACT, "set_paused"));
    client.set_paused(&false);

    client.revoke_kyc(&subject);
    rows.push(measure(&env, CONTRACT, "revoke_kyc"));

    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
    rows.push(measure(&env, CONTRACT, "transfer_admin"));

    rows
}
