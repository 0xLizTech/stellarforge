//! Generates and validates the committed SDK parity table for `tickOf` and
//! `checkDeviation`.
//!
//! Expected values come from the **same** pure functions the contract uses
//! (`oracle_adapter::tick_of`, `oracle_adapter::check_price_deviation`) — not a
//! hand-copied mirror. Regenerate the fixture with:
//!
//! ```text
//! UPDATE_ORACLE_PARITY_FIXTURE=1 cargo test -p oracle-adapter --features testutils \
//!   test_generate_sdk_parity_table -- --exact
//! ```
//!
//! Without that env var, the test fails if
//! `sdk/tests/fixtures/oracle-parity-table.json` drifts from a fresh run.

#![cfg(test)]

use oracle_adapter::{check_price_deviation, tick_of, DeviationCheck, BPS_DENOMINATOR};
use std::fmt::Write as _;
use std::path::PathBuf;

fn check_deviation(previous: i128, next: i128, max_bps: u32) -> Result<bool, &'static str> {
    // Contract refuses max_bps == 0 at validate_deviation before record.
    assert!(max_bps > 0, "generator must not emit max_bps=0");
    assert!(previous > 0 && next > 0, "prices must be positive");
    match check_price_deviation(previous, next, max_bps) {
        DeviationCheck::WithinLimit => Ok(true),
        DeviationCheck::TooLarge => Ok(false),
        DeviationCheck::Overflow => Err("Overflow"),
    }
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../sdk/tests/fixtures/oracle-parity-table.json")
}

fn build_parity_json() -> String {
    let mut tick_cases: Vec<(u64, u32, u64)> = Vec::new();
    let mut deviation_cases: Vec<(i128, i128, u32, Result<bool, &'static str>)> = Vec::new();

    // ── tickOf: exact multiples, one-off either side, varied resolutions ──
    let resolutions = [1u32, 2, 60, 300, 3600, 86_400];
    for &res in &resolutions {
        let res_u = u64::from(res);
        for k in [0u64, 1, 2, 7, 100, 1_000_000] {
            let ts = k.saturating_mul(res_u);
            tick_cases.push((ts, res, tick_of(ts, res)));
        }
        if res > 1 {
            let mid = 10u64.saturating_mul(res_u);
            tick_cases.push((mid - 1, res, tick_of(mid - 1, res)));
            tick_cases.push((mid + 1, res, tick_of(mid + 1, res)));
            tick_cases.push((mid + res_u / 2, res, tick_of(mid + res_u / 2, res)));
        }
    }
    tick_cases.push((u64::MAX - 10, 60, tick_of(u64::MAX - 10, 60)));
    tick_cases.push((1_800_000_123, 300, tick_of(1_800_000_123, 300)));
    tick_cases.push((1_800_000_000, 300, tick_of(1_800_000_000, 300)));
    tick_cases.push((1_700_000_045, 60, tick_of(1_700_000_045, 60)));
    tick_cases.push((1_700_000_060, 60, tick_of(1_700_000_060, 60)));
    tick_cases.push((1_700_000_000, 3600, tick_of(1_700_000_000, 3600)));

    // ── checkDeviation: equality, exact limit both ways, one unit past ──
    let bases: &[i128] = &[
        1,
        2,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_i128.pow(14),
        100 * 10_i128.pow(14),
        10_i128.pow(18),
        i128::MAX / 20_000,
    ];
    let bps_values: &[u32] = &[1, 2, 10, 100, 500, 1_000, 2_500, 5_000, 10_000];

    for &prev in bases {
        for &bps in bps_values {
            deviation_cases.push((prev, prev, bps, check_deviation(prev, prev, bps)));

            let max_delta = prev.saturating_mul(i128::from(bps)) / BPS_DENOMINATOR;
            if max_delta > 0 {
                let up = prev.saturating_add(max_delta);
                let down = prev.saturating_sub(max_delta);
                if up > 0 {
                    deviation_cases.push((prev, up, bps, check_deviation(prev, up, bps)));
                    if up < i128::MAX {
                        let past_up = up + 1;
                        if past_up > 0 {
                            deviation_cases.push((
                                prev,
                                past_up,
                                bps,
                                check_deviation(prev, past_up, bps),
                            ));
                        }
                    }
                }
                if down > 0 {
                    deviation_cases.push((prev, down, bps, check_deviation(prev, down, bps)));
                    if down > 1 {
                        deviation_cases.push((
                            prev,
                            down - 1,
                            bps,
                            check_deviation(prev, down - 1, bps),
                        ));
                    }
                }
            } else if prev < i128::MAX {
                deviation_cases.push((prev, prev + 1, bps, check_deviation(prev, prev + 1, bps)));
            }
        }
    }

    // Overflow cases near i128 bounds
    deviation_cases.push((
        i128::MAX / 5_000,
        i128::MAX / 5_000 + 1,
        10_000,
        check_deviation(i128::MAX / 5_000, i128::MAX / 5_000 + 1, 10_000),
    ));
    let big_prev = i128::MAX / 100 + 1;
    deviation_cases.push((
        big_prev,
        big_prev,
        10_000,
        check_deviation(big_prev, big_prev, 10_000),
    ));
    deviation_cases.push((1_i128, i128::MAX, 1, check_deviation(1, i128::MAX, 1)));
    let prev_for_bps_overflow = i128::MAX / 2;
    deviation_cases.push((
        prev_for_bps_overflow,
        prev_for_bps_overflow,
        3,
        check_deviation(prev_for_bps_overflow, prev_for_bps_overflow, 3),
    ));
    let near_max = i128::MAX / 10_001;
    deviation_cases.push((
        near_max,
        near_max + 1,
        1,
        check_deviation(near_max, near_max + 1, 1),
    ));
    deviation_cases.push((
        near_max,
        near_max,
        1,
        check_deviation(near_max, near_max, 1),
    ));

    assert!(
        tick_cases.len() + deviation_cases.len() >= 50,
        "need >= 50 cases, got tick={} deviation={}",
        tick_cases.len(),
        deviation_cases.len()
    );

    // Pretty-print to match `JSON.stringify(table, null, 2) + "\n"` so the
    // committed fixture and a fresh generator run compare byte-for-byte.
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(
        "  \"generatedBy\": \"contracts/oracle-adapter/tests/test_sdk_parity_table.rs\",\n",
    );
    out.push_str(
        "  \"note\": \"Expected values from oracle_adapter::tick_of and oracle_adapter::check_price_deviation (same pure fns as OracleAdapterContract::tick / record). BPS_DENOMINATOR=10000. Regenerate: UPDATE_ORACLE_PARITY_FIXTURE=1 cargo test -p oracle-adapter --features testutils test_generate_sdk_parity_table -- --exact\",\n",
    );
    out.push_str("  \"bpsDenominator\": 10000,\n");
    out.push_str("  \"tickOf\": [\n");
    for (i, (ts, res, exp)) in tick_cases.iter().enumerate() {
        let comma = if i + 1 == tick_cases.len() { "" } else { "," };
        write!(
            out,
            "    {{\n      \"timestamp\": \"{ts}\",\n      \"resolution\": {res},\n      \"expected\": \"{exp}\"\n    }}{comma}\n"
        )
        .unwrap();
    }
    out.push_str("  ],\n");
    out.push_str("  \"checkDeviation\": [\n");
    for (i, (prev, next, bps, result)) in deviation_cases.iter().enumerate() {
        let expected = match result {
            Ok(v) => format!("{v}"),
            Err(e) => format!("\"{e}\""),
        };
        let comma = if i + 1 == deviation_cases.len() {
            ""
        } else {
            ","
        };
        write!(
            out,
            "    {{\n      \"previous\": \"{prev}\",\n      \"next\": \"{next}\",\n      \"maxDeviationBps\": {bps},\n      \"expected\": {expected}\n    }}{comma}\n"
        )
        .unwrap();
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

#[test]
fn test_generate_sdk_parity_table() {
    let generated = build_parity_json();

    assert_eq!(tick_of(1_800_000_123, 300), 1_800_000_000);
    assert_eq!(check_deviation(10_000, 11_000, 1_000), Ok(true));
    assert_eq!(check_deviation(10_000, 11_001, 1_000), Ok(false));
    assert_eq!(check_deviation(1, i128::MAX, 1), Err("Overflow"));

    let path = fixture_path();
    if std::env::var_os("UPDATE_ORACLE_PARITY_FIXTURE").is_some() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture dir");
        }
        std::fs::write(&path, &generated).expect("write oracle-parity-table.json");
        eprintln!("updated {}", path.display());
        return;
    }

    let committed = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing committed fixture at {}: {e}\n\
             regenerate with UPDATE_ORACLE_PARITY_FIXTURE=1 cargo test -p oracle-adapter \
             --features testutils test_generate_sdk_parity_table -- --exact",
            path.display()
        )
    });
    // Normalize CRLF so a Windows checkout still matches Linux-generated LF.
    let committed = committed.replace("\r\n", "\n").replace('\r', "\n");
    let generated = generated.replace("\r\n", "\n").replace('\r', "\n");
    assert_eq!(
        committed, generated,
        "sdk/tests/fixtures/oracle-parity-table.json drifted from contract math.\n\
         Regenerate with:\n\
         UPDATE_ORACLE_PARITY_FIXTURE=1 cargo test -p oracle-adapter --features testutils \
         test_generate_sdk_parity_table -- --exact"
    );
}
