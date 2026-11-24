#!/usr/bin/env node
/** Regenerates the SDK vault math fixture by executing the vault's Rust math module. */
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");

const root = path.join(__dirname, "..");
const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "tools/vault-math-parity/Cargo.toml",
    "test_generate_sdk_vault_math_parity_table",
    "--",
    "--exact",
  ],
  {
    cwd: root,
    env: { ...process.env, UPDATE_VAULT_PARITY_FIXTURE: "1" },
    encoding: "utf8",
    shell: process.platform === "win32",
  },
);

if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);
if (result.status !== 0) {
  console.error("Rust vault parity fixture generation failed. Ensure the Rust toolchain is installed.");
  process.exit(result.status === null ? 1 : result.status);
}

console.log("ok: sdk/tests/fixtures/vault-math-parity-table.json refreshed from vault Rust math");
