#!/usr/bin/env node

import * as fs from "fs";
import * as path from "path";
import { Keypair } from "@stellar/stellar-sdk";
import { MAINNET_CONFIG, OracleAdapterClient, TESTNET_CONFIG } from "@stellarforge-protocol/sdk";
import { Keypair, Networks } from "@stellar/stellar-sdk";
import { OracleAdapterClient } from "@stellarforge-protocol/sdk";
import { submitSignedAppraisal } from "./reporter.js";
import { ExitCode, type SignedAppraisalFile } from "./types.js";
import { resolveCliOptions } from "./options.js";

function usageText(): string {
  return `
stellarforge-nav-reporter: Institutional NAV Appraisal Reporter CLI

Usage:
  stellarforge-nav-reporter --appraisal <file> --oracle <contract-id> [options]

Required Options:
  --appraisal <path>           Path to signed appraisal JSON file
  --oracle <contract-id>       Soroban address of the deployed oracle-adapter contract

Authentication (Environment Variables or Files):
  APPRAISER_PUBLIC_KEY         Expected public key of the appraiser (or --appraiser-key)
  REPORTER_SECRET_KEY          Secret key of the authorized reporter (or --reporter-key-file)
  RELAYER_SECRET_KEY           Secret key of fee relayer account (or --relayer-key-file)

Operational Options:
  --dry-run                    Perform full validation and pre-flight checks without submitting
  --json                       Emit structured machine-readable JSON output
  --network <name>             Network name: 'testnet' (default) or 'mainnet'
  --rpc-url <url>              Custom Soroban RPC URL
  --config <path>              Load options from sforge.config.json (default: ./sforge.config.json)
  -h, --help                   Display this help message

Configuration precedence: command-line flags > environment variables > config file > defaults.

Exit Codes:
  0: Success (or dry-run verified, or idempotent duplicate)
  2: Authentication / signature failure
  3: Pre-flight validation failure
  4: Submission failure
  5: Unknown transaction outcome
`;
}

function loadSecret(envVar: string, filePath?: string): string | undefined {
  if (filePath && fs.existsSync(filePath)) {
    return fs.readFileSync(filePath, "utf8").trim();
  }
  return process.env[envVar]?.trim();
}

export async function runCli(args: string[]): Promise<number> {
  const jsonOutput = args.includes("--json");
  let appraisalPath: string | undefined;
  let oracleContractId: string | undefined;
  let appraiserKey: string | undefined = process.env.APPRAISER_PUBLIC_KEY;
  let reporterKeyFile: string | undefined;
  let relayerKeyFile: string | undefined;
  let networkName: "testnet" | "mainnet" =
    (process.env.NETWORK as "testnet" | "mainnet") || "testnet";
  let rpcUrl: string | undefined = process.env.SOROBAN_RPC_URL;
  let dryRun = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "-h" || arg === "--help") {
      if (jsonOutput) {
        console.log(JSON.stringify({ success: true, exitCode: ExitCode.Success, help: usageText().trim() }));
      } else {
        console.log(usageText());
      }
      return 0;
    }
    if (arg === "--appraisal" && i + 1 < args.length) {
      appraisalPath = args[++i];
    } else if (arg === "--oracle" && i + 1 < args.length) {
      oracleContractId = args[++i];
    } else if (arg === "--appraiser-key" && i + 1 < args.length) {
      appraiserKey = args[++i];
    } else if (arg === "--reporter-key-file" && i + 1 < args.length) {
      reporterKeyFile = args[++i];
    } else if (arg === "--relayer-key-file" && i + 1 < args.length) {
      relayerKeyFile = args[++i];
    } else if (arg === "--network" && i + 1 < args.length) {
      networkName = args[++i] as "testnet" | "mainnet";
    } else if (arg === "--rpc-url" && i + 1 < args.length) {
      rpcUrl = args[++i];
    } else if (arg === "--dry-run") {
      dryRun = true;
    }
  if (args.includes("-h") || args.includes("--help")) {
    printUsage();
    return 0;
  }
  let options;
  try {
    options = resolveCliOptions(args);
  } catch (error) {
    console.error(`Error: ${error instanceof Error ? error.message : String(error)}`);
    return ExitCode.PreflightFailure;
  }
  const { appraisalPath, oracleContractId, appraiserKey, reporterKeyFile, relayerKeyFile,
    networkName, rpcUrl, dryRun, jsonOutput } = options;

  if (!appraisalPath || !oracleContractId) {
    if (!jsonOutput) {
      console.error("Error: --appraisal and --oracle arguments are required.");
      console.error(usageText());
    } else {
      console.log(
        JSON.stringify({
          success: false,
          exitCode: ExitCode.PreflightFailure,
          error: "Missing required --appraisal or --oracle arguments",
        })
      );
    }
    return ExitCode.PreflightFailure;
  }

  if (!fs.existsSync(appraisalPath)) {
    const errMsg = `Appraisal file not found: ${appraisalPath}`;
    if (jsonOutput) {
      console.log(
        JSON.stringify({
          success: false,
          exitCode: ExitCode.PreflightFailure,
          error: errMsg,
        })
      );
    } else {
      console.error(`Error: ${errMsg}`);
    }
    return ExitCode.PreflightFailure;
  }

  const fileRaw = fs.readFileSync(appraisalPath, "utf8");
  let signedAppraisal: SignedAppraisalFile;
  try {
    signedAppraisal = JSON.parse(fileRaw);
  } catch (err) {
    const errMsg = `Malformed appraisal JSON: ${err instanceof Error ? err.message : String(err)}`;
    if (jsonOutput) {
      console.log(
        JSON.stringify({
          success: false,
          exitCode: ExitCode.PreflightFailure,
          error: errMsg,
        })
      );
    } else {
      console.error(`Error: ${errMsg}`);
    }
    return ExitCode.PreflightFailure;
  }

  const reporterSecret = loadSecret("REPORTER_SECRET_KEY", reporterKeyFile);
  if (!reporterSecret) {
    const errMsg = "Missing REPORTER_SECRET_KEY environment variable or --reporter-key-file";
    if (jsonOutput) {
      console.log(
        JSON.stringify({
          success: false,
          exitCode: ExitCode.AuthFailure,
          error: errMsg,
        })
      );
    } else {
      console.error(`Error: ${errMsg}`);
    }
    return ExitCode.AuthFailure;
  }

  if (networkName === "mainnet" && !rpcUrl) {
    const errMsg = "Mainnet reporting requires --rpc-url or SOROBAN_RPC_URL";
    if (jsonOutput) {
      console.log(JSON.stringify({ success: false, exitCode: ExitCode.PreflightFailure, error: errMsg }));
    } else {
      console.error(`Error: ${errMsg}`);
    }
    return ExitCode.PreflightFailure;
  }

  let reporterKeypair: Keypair;
  try {
    reporterKeypair = Keypair.fromSecret(reporterSecret);
  } catch {
    const errMsg = "Invalid reporter secret key format";
    if (jsonOutput) {
      console.log(
        JSON.stringify({
          success: false,
          exitCode: ExitCode.AuthFailure,
          error: errMsg,
        })
      );
    } else {
      console.error(`Error: ${errMsg}`);
    }
    return ExitCode.AuthFailure;
  }

  let relayerKeypair: Keypair | undefined;
  const relayerSecret = loadSecret("RELAYER_SECRET_KEY", relayerKeyFile);
  if (relayerSecret) {
    try {
      relayerKeypair = Keypair.fromSecret(relayerSecret);
    } catch {
      const errMsg = "Invalid relayer secret key format";
      if (jsonOutput) {
        console.log(
          JSON.stringify({
            success: false,
            exitCode: ExitCode.AuthFailure,
            error: errMsg,
          })
        );
      } else {
        console.error(`Error: ${errMsg}`);
      }
      return ExitCode.AuthFailure;
    }
  }

  if (networkName === "mainnet" && !rpcUrl) {
    const error = "Mainnet reporting requires --rpc-url or SOROBAN_RPC_URL.";
    if (jsonOutput) console.log(JSON.stringify({ success: false, exitCode: ExitCode.PreflightFailure, error }));
    else console.error(`Error: ${error}`);
    return ExitCode.PreflightFailure;
  }

  const oracleClient = new OracleAdapterClient({
    network: networkName,
    networkPassphrase:
      networkName === "mainnet" ? MAINNET_CONFIG.networkPassphrase : TESTNET_CONFIG.networkPassphrase,
    contracts: { oracleAdapter: oracleContractId },
    rpcUrl: rpcUrl ?? TESTNET_CONFIG.rpcUrl,
    networkPassphrase: networkName === "mainnet" ? Networks.PUBLIC : Networks.TESTNET,
    rpcUrl: rpcUrl ?? "https://soroban-testnet.stellar.org",
    contracts: { oracleAdapter: oracleContractId },
    signerSecret: relayerSecret ?? reporterSecret,
  });

  const result = await submitSignedAppraisal(signedAppraisal, {
    oracleClient,
    reporterKeypair,
    relayerKeypair,
    rpcUrl,
    networkName,
    expectedAppraiserPublicKey: appraiserKey,
    dryRun,
  });

  if (jsonOutput) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    if (result.success) {
      console.error(`\n✓ Appraisal reporting succeeded! (${result.action})`);
      if (result.txHash) console.error(`  Transaction: ${result.txHash}`);
      if (result.ledger) console.error(`  Ledger:      ${result.ledger}`);
      if (result.tick) console.error(`  Tick:        ${result.tick}`);
      if (result.details) console.error(`  Details:     ${JSON.stringify(result.details)}`);
    } else {
      console.error(`\n✗ Appraisal reporting failed (code ${result.exitCode}):`);
      console.error(`  ${result.error}`);
    }
  }

  return result.exitCode;
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(__filename)) {
  runCli(process.argv.slice(2)).then((code) => {
    process.exit(code);
  });
}
