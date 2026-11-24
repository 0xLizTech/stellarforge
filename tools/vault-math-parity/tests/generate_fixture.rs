use std::fs;
use std::path::PathBuf;

use vault_math_parity::{shares_for_deposit, underlying_for_redeem};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../sdk/tests/fixtures/vault-math-parity-table.json")
}

fn expected_table() -> String {
    let amounts = [0_i128, 1, 2, 3, 10, 99, 1_000, 1_000_000, 1_000_000_000_000];
    let rates = [1_i128, 2, 3, 5, 10, 100, 10_000, 10_000_000];
    let mut rows = Vec::new();

    for amount in amounts {
        for rate in rates {
            let result = shares_for_deposit(amount, rate)
                .map(|value| format!("ok:{value}"))
                .unwrap_or_else(|| "overflow".to_string());
            rows.push(format!(
                "    {{\"operation\":\"deposit\",\"amount\":\"{amount}\",\"rate\":\"{rate}\",\"expected\":\"{result}\"}}"
            ));

            let shares = amount * rate;
            let redeemed = underlying_for_redeem(shares, rate)
                .map(|value| format!("ok:{value}"))
                .unwrap_or_else(|| "not_multiple".to_string());
            rows.push(format!(
                "    {{\"operation\":\"redeem\",\"amount\":\"{shares}\",\"rate\":\"{rate}\",\"expected\":\"{redeemed}\"}}"
            ));

            if rate > 1 {
                let non_multiple = shares + 1;
                let result = underlying_for_redeem(non_multiple, rate)
                    .map(|value| format!("ok:{value}"))
                    .unwrap_or_else(|| "not_multiple".to_string());
                rows.push(format!(
                    "    {{\"operation\":\"redeem\",\"amount\":\"{non_multiple}\",\"rate\":\"{rate}\",\"expected\":\"{result}\"}}"
                ));
            }
        }
    }

    format!(
        "{{\n  \"source\": \"contracts/vault/src/math.rs\",\n  \"cases\": [\n{}\n  ]\n}}\n",
        rows.join(",\n")
    )
}

#[test]
fn test_generate_sdk_vault_math_parity_table() {
    let path = fixture_path();
    let expected = expected_table();
    if std::env::var_os("UPDATE_VAULT_PARITY_FIXTURE").is_some() {
        fs::write(&path, &expected).expect("write vault math parity fixture");
        return;
    }
    let committed = fs::read_to_string(&path).expect(
        "committed vault math parity fixture; regenerate with UPDATE_VAULT_PARITY_FIXTURE=1",
    );
    assert_eq!(committed, expected, "vault SDK parity fixture drifted");
}
