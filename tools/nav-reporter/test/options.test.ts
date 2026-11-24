import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { resolveCliOptions } from "../src/options.js";

const temporaryDirs: string[] = [];
afterEach(() => {
  for (const dir of temporaryDirs.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("resolveCliOptions", () => {
  it("merges config, environment and flags in documented precedence order", () => {
    const cwd = mkdtempSync(path.join(os.tmpdir(), "nav-reporter-options-"));
    temporaryDirs.push(cwd);
    mkdirSync(path.join(cwd, "config"));
    writeFileSync(path.join(cwd, "config", "sforge.config.json"), JSON.stringify({
      appraisal: "files/appraisal.json", oracle: "C_CONFIG", appraiserKey: "G_CONFIG",
      reporterKeyFile: "keys/reporter", relayerKeyFile: "keys/relayer", network: "testnet",
      rpcUrl: "https://config-rpc.invalid", dryRun: false, json: false,
    }));

    const resolved = resolveCliOptions([
      "--config", "config/sforge.config.json", "--oracle", "C_FLAG", "--network", "mainnet", "--dry-run",
    ], {
      ORACLE_CONTRACT_ID: "C_ENV", NETWORK: "testnet", SOROBAN_RPC_URL: "https://env-rpc.invalid",
      DRY_RUN: "false", JSON_OUTPUT: "true", APPRAISER_PUBLIC_KEY: "G_ENV",
    }, cwd);

    expect(resolved).toEqual({
      appraisalPath: path.join(cwd, "config/files/appraisal.json"),
      oracleContractId: "C_FLAG", appraiserKey: "G_ENV",
      reporterKeyFile: path.join(cwd, "config/keys/reporter"),
      relayerKeyFile: path.join(cwd, "config/keys/relayer"),
      networkName: "mainnet", rpcUrl: "https://env-rpc.invalid", dryRun: true, jsonOutput: true,
    });
  });

  it("uses defaults when no config or environment values are present", () => {
    const cwd = mkdtempSync(path.join(os.tmpdir(), "nav-reporter-options-"));
    temporaryDirs.push(cwd);
    expect(resolveCliOptions([], {}, cwd)).toEqual({
      appraisalPath: undefined, oracleContractId: undefined, appraiserKey: undefined,
      reporterKeyFile: undefined, relayerKeyFile: undefined, networkName: "testnet",
      rpcUrl: undefined, dryRun: false, jsonOutput: false,
    });
  });

  it("rejects invalid network names and malformed option values", () => {
    expect(() => resolveCliOptions([], { NETWORK: "futurenet" }, "/tmp"))
      .toThrow("NETWORK must be 'testnet' or 'mainnet'");
    expect(() => resolveCliOptions(["--oracle"], {}, "/tmp")).toThrow("--oracle requires a value");
  });
});
