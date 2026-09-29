#!/usr/bin/env node
/**
 * Regenerates sdk/tests/fixtures/oracle-parity-table.json from the Rust contract
 * pure math (oracle_adapter::tick_of / check_price_deviation).
 *
 * Source of truth is contracts/oracle-adapter — this script only shells out to
 * cargo. It does not re-implement tick or deviation arithmetic in JS.
 *
 * Usage: node scripts/generate-oracle-parity-table.cjs
 */
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");

const root = path.join(__dirname, "..");
const env = {
  ...process.env,
  UPDATE_ORACLE_PARITY_FIXTURE: "1",
};

const result = spawnSync(
  "cargo",
  [
    "test",
    "-p",
    "oracle-adapter",
    "--features",
    "testutils",
    "test_generate_sdk_parity_table",
    "--",
    "--exact",
  ],
  {
    cwd: root,
    env,
    encoding: "utf8",
    shell: process.platform === "win32",
  },
);

if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);

if (result.status !== 0) {
  console.error(
    "Rust parity fixture generator failed. Ensure the Rust toolchain is installed.",
  );
  process.exit(result.status === null ? 1 : result.status);
}

console.log(
  "ok: sdk/tests/fixtures/oracle-parity-table.json refreshed from contract pure math",
);
