//! Integration tests for Stage 1 repository and CI tooling (#91, #93, #94, #95).
//!
//! Exercises:
//! - `deny.toml` license, ban, and source rules (#91)
//! - `contracts/size-budget.json` and `scripts/check-wasm-size.cjs` (#93, #94)
//! - `scripts/check-soroban-sdk-version.cjs` (#95)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts/common must sit two levels below workspace root")
        .to_path_buf()
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn test_deny_toml_allows_permissive_licenses_and_denies_unknown_registries() {
    let root = repo_root();
    let deny_toml = fs::read_to_string(root.join("deny.toml")).expect("deny.toml must exist");

    for allowed in [
        "\"Apache-2.0\"",
        "\"MIT\"",
        "\"BSD-2-Clause\"",
        "\"BSD-3-Clause\"",
    ] {
        assert!(
            deny_toml.contains(allowed),
            "deny.toml must allow {allowed}"
        );
    }

    // Strip comment lines before checking that no GPL/LGPL/AGPL identifier is in `allow`.
    let non_comment: String = deny_toml
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in ["GPL-2.0", "GPL-3.0", "LGPL-", "AGPL-"] {
        assert!(
            !non_comment.contains(forbidden),
            "deny.toml must not allow {forbidden}"
        );
    }

    assert!(deny_toml.contains("unknown-registry = \"deny\""));
    assert!(deny_toml.contains("unknown-git = \"deny\""));
}

#[test]
fn test_size_budget_json_covers_all_five_contracts() {
    let root = repo_root();
    let budget_content = fs::read_to_string(root.join("contracts").join("size-budget.json"))
        .expect("contracts/size-budget.json must exist");

    for contract in [
        "\"rwa-asset\"",
        "\"registry\"",
        "\"compliance\"",
        "\"governance\"",
        "\"oracle-adapter\"",
    ] {
        assert!(
            budget_content.contains(contract),
            "contracts/size-budget.json must include a budget for {contract}"
        );
    }
}

#[test]
fn test_wasm_size_script_reports_10kb_delta_and_enforces_budget() {
    let root = repo_root();
    let script = root.join("scripts").join("check-wasm-size.cjs");
    let tmp = make_temp_dir("stellarforge-wasm-size-test");

    let budget_path = tmp.join("size-budget.json");
    fs::write(
        &budget_path,
        r#"{
  "rwa-asset": { "wasm": "rwa_asset.wasm", "baseline_bytes": 35000, "max_bytes": 44000 },
  "registry": { "wasm": "registry.wasm", "baseline_bytes": 11000, "max_bytes": 25000 }
}"#,
    )
    .expect("write test budget");

    // 1. Registry grows by +10 KB (10,240 bytes) and stays within budget
    fs::write(tmp.join("rwa_asset.wasm"), vec![0u8; 35000]).expect("write rwa_asset.wasm");
    fs::write(tmp.join("registry.wasm"), vec![0u8; 11000 + 10240]).expect("write registry.wasm");

    let pass_out = Command::new("node")
        .arg(&script)
        .arg("--budget")
        .arg(&budget_path)
        .arg("--wasm-dir")
        .arg(&tmp)
        .output()
        .expect("run check-wasm-size.cjs");

    assert!(
        pass_out.status.success(),
        "expected check-wasm-size.cjs to succeed when within budget, stderr: {}",
        String::from_utf8_lossy(&pass_out.stderr)
    );
    let pass_stdout = String::from_utf8_lossy(&pass_out.stdout);
    assert!(
        pass_stdout.contains("+10,240 B (+10.00 KB)"),
        "expected +10 KB delta in report, got:\n{pass_stdout}"
    );

    // 2. rwa-asset grows by +10 KB (35,000 -> 45,240), exceeding its 44,000-byte budget
    fs::write(tmp.join("rwa_asset.wasm"), vec![0u8; 35000 + 10240]).expect("write rwa_asset.wasm");

    let fail_out = Command::new("node")
        .arg(&script)
        .arg("--budget")
        .arg(&budget_path)
        .arg("--wasm-dir")
        .arg(&tmp)
        .output()
        .expect("run check-wasm-size.cjs");

    assert!(
        !fail_out.status.success(),
        "expected check-wasm-size.cjs to fail when contract exceeds budget"
    );
    let fail_stderr = String::from_utf8_lossy(&fail_out.stderr);
    assert!(
        fail_stderr.contains("exceeding its budget of 44000 bytes"),
        "expected budget violation error on stderr, got:\n{fail_stderr}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_soroban_sdk_version_consistency_check_passes_workspace_and_rejects_override() {
    let root = repo_root();
    let script = root.join("scripts").join("check-soroban-sdk-version.cjs");

    // 1. Workspace root must pass
    let ws_out = Command::new("node")
        .arg(&script)
        .arg("--root")
        .arg(&root)
        .output()
        .expect("run check-soroban-sdk-version.cjs on workspace root");

    assert!(
        ws_out.status.success(),
        "workspace root must pass soroban-sdk consistency check, stderr: {}",
        String::from_utf8_lossy(&ws_out.stderr)
    );

    // 2. A member crate pinning its own soroban-sdk version must fail
    let tmp = make_temp_dir("stellarforge-sdk-override-test");
    let member_dir = tmp.join("contracts").join("pinned-crate");
    fs::create_dir_all(&member_dir).expect("create member dir");

    fs::write(
        tmp.join("Cargo.toml"),
        r#"[workspace]
members = ["contracts/pinned-crate"]
resolver = "2"

[workspace.dependencies]
soroban-sdk = { version = "27.0.5" }
"#,
    )
    .expect("write root Cargo.toml");

    fs::write(
        member_dir.join("Cargo.toml"),
        r#"[package]
name = "pinned-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
soroban-sdk = { version = "27.0.5" }
"#,
    )
    .expect("write member Cargo.toml");

    let fail_out = Command::new("node")
        .arg(&script)
        .arg("--root")
        .arg(&tmp)
        .output()
        .expect("run check-soroban-sdk-version.cjs on invalid workspace");

    assert!(
        !fail_out.status.success(),
        "expected check-soroban-sdk-version.cjs to fail when crate pins soroban-sdk"
    );
    let fail_stderr = String::from_utf8_lossy(&fail_out.stderr);
    assert!(
        fail_stderr.contains("contracts/pinned-crate/Cargo.toml"),
        "expected error pointing to contracts/pinned-crate/Cargo.toml, got:\n{fail_stderr}"
    );

    let _ = fs::remove_dir_all(&tmp);
}
