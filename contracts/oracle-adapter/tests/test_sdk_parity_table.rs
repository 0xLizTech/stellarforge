//! Generates the committed SDK parity table for `tickOf` and `checkDeviation`.
//!
//! The arithmetic here is a byte-for-byte copy of `OracleAdapterContract::tick`
//! and the deviation branch of `record` in `src/lib.rs`. Run with:
//!
//! ```text
//! cargo test -p oracle-adapter --features testutils \
//!   test_generate_sdk_parity_table -- --nocapture
//! ```
//!
//! and copy the single JSON object into `sdk/tests/fixtures/oracle-parity-table.json`.

#![cfg(test)]

use oracle_adapter::BPS_DENOMINATOR;
use std::fmt::Write as _;

/// Mirrors `OracleAdapterContract::tick`.
fn tick_of(timestamp: u64, resolution: u32) -> u64 {
    let resolution = u64::from(resolution);
    timestamp - timestamp % resolution
}

/// Mirrors the deviation check inside `OracleAdapterContract::record`.
///
/// Returns `Ok(true)` when the move is within the limit, `Ok(false)` when it
/// would raise `DeviationTooLarge`, and `Err("Overflow")` when either
/// `checked_mul` fails (contract panics with `OracleError::Overflow`).
fn check_deviation(previous: i128, next: i128, max_bps: u32) -> Result<bool, &'static str> {
    // The contract refuses max_bps == 0 at validate_deviation before record.
    // The SDK mirrors that; this generator only emits positive bps cases plus
    // overflow cases, never zero.
    assert!(max_bps > 0, "generator must not emit max_bps=0");
    assert!(previous > 0 && next > 0, "prices must be positive");

    let moved = (next - previous).abs();
    let (Some(lhs), Some(rhs)) = (
        moved.checked_mul(BPS_DENOMINATOR),
        previous.checked_mul(i128::from(max_bps)),
    ) else {
        return Err("Overflow");
    };
    Ok(lhs <= rhs)
}

#[test]
fn test_generate_sdk_parity_table() {
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

    let mut out = String::from("SDK_PARITY_TABLE_BEGIN\n");
    out.push_str(
        "{\n  \"generatedBy\": \"contracts/oracle-adapter/tests/test_sdk_parity_table.rs\",\n",
    );
    out.push_str("  \"bpsDenominator\": 10000,\n");
    out.push_str("  \"tickOf\": [\n");
    for (i, (ts, res, exp)) in tick_cases.iter().enumerate() {
        write!(
            out,
            "    {{\"timestamp\": \"{}\", \"resolution\": {}, \"expected\": \"{}\"}}{}",
            ts,
            res,
            exp,
            if i + 1 == tick_cases.len() {
                "\n"
            } else {
                ",\n"
            }
        )
        .unwrap();
    }
    out.push_str("  ],\n  \"checkDeviation\": [\n");
    for (i, (prev, next, bps, result)) in deviation_cases.iter().enumerate() {
        let expected = match result {
            Ok(v) => format!("{v}"),
            Err(e) => format!("\"{e}\""),
        };
        write!(
            out,
            "    {{\"previous\": \"{}\", \"next\": \"{}\", \"maxDeviationBps\": {}, \"expected\": {}}}{}",
            prev,
            next,
            bps,
            expected,
            if i + 1 == deviation_cases.len() { "\n" } else { ",\n" }
        )
        .unwrap();
    }
    out.push_str("  ]\n}\n");
    out.push_str("SDK_PARITY_TABLE_END\n");
    print!("{out}");

    assert_eq!(tick_of(1_800_000_123, 300), 1_800_000_000);
    assert_eq!(check_deviation(10_000, 11_000, 1_000), Ok(true));
    assert_eq!(check_deviation(10_000, 11_001, 1_000), Ok(false));
    assert_eq!(check_deviation(1, i128::MAX, 1), Err("Overflow"));
}
