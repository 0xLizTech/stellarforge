#!/usr/bin/env node
import fs from "fs";
import os from "os";
import path from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SDK_DIR = path.resolve(__dirname, "..");
const REPO_ROOT = path.resolve(SDK_DIR, "..");

console.log("[test-package-imports] Packaging SDK with npm pack...");

// 1. Pack the package into a tarball
const packOutput = execSync("npm pack --json", {
  cwd: SDK_DIR,
  encoding: "utf8",
});
const packInfo = JSON.parse(packOutput);
const tarballFilename = packInfo[0].filename;
const tarballPath = path.join(SDK_DIR, tarballFilename);

if (!fs.existsSync(tarballPath)) {
  throw new Error(`Tarball not found at ${tarballPath}`);
}
console.log(`[test-package-imports] Tarball created: ${tarballFilename}`);

// 2. Create isolated temporary directory
const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "sf-sdk-import-test-"));
console.log(`[test-package-imports] Testing in temporary workspace: ${tempDir}`);

try {
  // Extract tarball
  execSync(`tar -xzf "${tarballPath}" -C "${tempDir}"`, { stdio: "inherit" });
  const packageDir = path.join(tempDir, "package");

  // Setup node_modules in tempDir
  const nodeModulesDir = path.join(tempDir, "node_modules");
  const sfScopeDir = path.join(nodeModulesDir, "@stellarforge-protocol");
  const stellarScopeDir = path.join(nodeModulesDir, "@stellar");
  fs.mkdirSync(sfScopeDir, { recursive: true });
  fs.mkdirSync(stellarScopeDir, { recursive: true });

  // Symlink the packed package as @stellarforge-protocol/sdk
  const sdkLink = path.join(sfScopeDir, "sdk");
  fs.symlinkSync(packageDir, sdkLink, "dir");

  // Symlink @stellar/stellar-sdk from parent SDK
  const stellarSdkSource = path.join(SDK_DIR, "node_modules", "@stellar", "stellar-sdk");
  const stellarSdkLink = path.join(stellarScopeDir, "stellar-sdk");
  if (fs.existsSync(stellarSdkSource)) {
    fs.symlinkSync(stellarSdkSource, stellarSdkLink, "dir");
  }

  // 3. Test ESM imports
  console.log("[test-package-imports] Verifying ESM imports...");
  const esmTestFile = path.join(tempDir, "test-esm.mjs");
  fs.writeFileSync(
    esmTestFile,
    `
import {
  RwaAssetClient,
  ComplianceClient,
  GovernanceClient,
  RegistryClient,
  VaultClient,
  OracleAdapterClient,
  TESTNET_CONFIG,
  MAINNET_CONFIG,
  toStroops,
  fromStroops,
  ContractClient,
  TransactionExpiredError,
  TransactionOutcomeUnknownError,
  DEFAULT_AUTH_VALIDITY_LEDGERS,
  MAX_AUTH_VALIDITY_LEDGERS,
  MIN_AUTH_REMAINING_LEDGERS,
} from "@stellarforge-protocol/sdk";

import { RwaAssetClient as RwaSub, ComplianceClient as CompSub } from "@stellarforge-protocol/sdk/client";
import { GovernanceClient as GovSub } from "@stellarforge-protocol/sdk/governance";
import { RegistryClient as RegSub } from "@stellarforge-protocol/sdk/registry";
import { VaultClient as VaultSub, sharesForDeposit, underlyingForRedeem, maxRedeemable } from "@stellarforge-protocol/sdk/vault";
import { OracleAdapterClient as OracleSub, tickOf, checkDeviation, isFresh } from "@stellarforge-protocol/sdk/oracle";
import { toStroops as toStroopsSub, fromStroops as fromStroopsSub } from "@stellarforge-protocol/sdk/utils";
import { TransactionExpiredError as ExpiredSub } from "@stellarforge-protocol/sdk/errors";
import { DEFAULT_AUTH_VALIDITY_LEDGERS as AuthSub } from "@stellarforge-protocol/sdk/sponsored";
import { ContractClient as BaseSub } from "@stellarforge-protocol/sdk/base";

// Assert root imports
console.assert(typeof RwaAssetClient === "function", "RwaAssetClient is function");
console.assert(typeof ComplianceClient === "function", "ComplianceClient is function");
console.assert(typeof GovernanceClient === "function", "GovernanceClient is function");
console.assert(typeof RegistryClient === "function", "RegistryClient is function");
console.assert(typeof VaultClient === "function", "VaultClient is function");
console.assert(typeof OracleAdapterClient === "function", "OracleAdapterClient is function");
console.assert(typeof toStroops === "function", "toStroops is function");
console.assert(typeof fromStroops === "function", "fromStroops is function");
console.assert(typeof ContractClient === "function", "ContractClient is function");
console.assert(typeof TransactionExpiredError === "function", "TransactionExpiredError is function");
console.assert(typeof DEFAULT_AUTH_VALIDITY_LEDGERS === "number", "DEFAULT_AUTH_VALIDITY_LEDGERS is number");

// Assert subpath imports
console.assert(typeof RwaSub === "function", "RwaSub is function");
console.assert(typeof CompSub === "function", "CompSub is function");
console.assert(typeof GovSub === "function", "GovSub is function");
console.assert(typeof RegSub === "function", "RegSub is function");
console.assert(typeof VaultSub === "function", "VaultSub is function");
console.assert(typeof sharesForDeposit === "function", "sharesForDeposit is function");
console.assert(typeof OracleSub === "function", "OracleSub is function");
console.assert(typeof tickOf === "function", "tickOf is function");
console.assert(typeof toStroopsSub === "function", "toStroopsSub is function");
console.assert(typeof ExpiredSub === "function", "ExpiredSub is function");
console.assert(typeof AuthSub === "number", "AuthSub is number");
console.assert(typeof BaseSub === "function", "BaseSub is function");

console.log("✓ ESM imports and subpath exports passed.");
`
  );
  execSync(`node "${esmTestFile}"`, { stdio: "inherit", cwd: tempDir });

  // 4. Test CommonJS require
  console.log("[test-package-imports] Verifying CJS requires...");
  const cjsTestFile = path.join(tempDir, "test-cjs.cjs");
  fs.writeFileSync(
    cjsTestFile,
    `
const sdk = require("@stellarforge-protocol/sdk");
const clientSub = require("@stellarforge-protocol/sdk/client");
const govSub = require("@stellarforge-protocol/sdk/governance");
const regSub = require("@stellarforge-protocol/sdk/registry");
const vaultSub = require("@stellarforge-protocol/sdk/vault");
const oracleSub = require("@stellarforge-protocol/sdk/oracle");
const utilsSub = require("@stellarforge-protocol/sdk/utils");
const errorsSub = require("@stellarforge-protocol/sdk/errors");
const sponsoredSub = require("@stellarforge-protocol/sdk/sponsored");
const baseSub = require("@stellarforge-protocol/sdk/base");

// Assert root CJS
console.assert(typeof sdk.RwaAssetClient === "function", "sdk.RwaAssetClient is function");
console.assert(typeof sdk.GovernanceClient === "function", "sdk.GovernanceClient is function");
console.assert(typeof sdk.VaultClient === "function", "sdk.VaultClient is function");
console.assert(typeof sdk.OracleAdapterClient === "function", "sdk.OracleAdapterClient is function");
console.assert(typeof sdk.RegistryClient === "function", "sdk.RegistryClient is function");
console.assert(typeof sdk.ComplianceClient === "function", "sdk.ComplianceClient is function");
console.assert(typeof sdk.toStroops === "function", "sdk.toStroops is function");
console.assert(typeof sdk.ContractClient === "function", "sdk.ContractClient is function");

// Assert subpath CJS
console.assert(typeof clientSub.RwaAssetClient === "function", "clientSub.RwaAssetClient is function");
console.assert(typeof clientSub.ComplianceClient === "function", "clientSub.ComplianceClient is function");
console.assert(typeof govSub.GovernanceClient === "function", "govSub.GovernanceClient is function");
console.assert(typeof regSub.RegistryClient === "function", "regSub.RegistryClient is function");
console.assert(typeof vaultSub.VaultClient === "function", "vaultSub.VaultClient is function");
console.assert(typeof vaultSub.sharesForDeposit === "function", "vaultSub.sharesForDeposit is function");
console.assert(typeof oracleSub.OracleAdapterClient === "function", "oracleSub.OracleAdapterClient is function");
console.assert(typeof oracleSub.tickOf === "function", "oracleSub.tickOf is function");
console.assert(typeof utilsSub.toStroops === "function", "utilsSub.toStroops is function");
console.assert(typeof errorsSub.TransactionExpiredError === "function", "errorsSub.TransactionExpiredError is function");
console.assert(typeof sponsoredSub.DEFAULT_AUTH_VALIDITY_LEDGERS === "number", "sponsoredSub.DEFAULT_AUTH_VALIDITY_LEDGERS is number");
console.assert(typeof baseSub.ContractClient === "function", "baseSub.ContractClient is function");

console.log("✓ CJS requires and subpath exports passed.");
`
  );
  execSync(`node "${cjsTestFile}"`, { stdio: "inherit", cwd: tempDir });

  // 5. Test TypeScript type resolution
  console.log("[test-package-imports] Verifying TypeScript type resolution...");
  const tsConfigFile = path.join(tempDir, "tsconfig.json");
  fs.writeFileSync(
    tsConfigFile,
    JSON.stringify(
      {
        compilerOptions: {
          module: "NodeNext",
          moduleResolution: "NodeNext",
          target: "ES2022",
          strict: true,
          noEmit: true,
          skipLibCheck: true,
        },
        files: ["test-types.ts"],
      },
      null,
      2
    )
  );

  const tsTestFile = path.join(tempDir, "test-types.ts");
  fs.writeFileSync(
    tsTestFile,
    `
import {
  RwaAssetClient,
  TESTNET_CONFIG,
  MAINNET_CONFIG,
  type AssetMetadata,
  type KycRecord,
  type TxResult,
  type StellarForgeConfig,
} from "@stellarforge-protocol/sdk";

import { RwaAssetClient as RwaSub, ComplianceClient as CompSub } from "@stellarforge-protocol/sdk/client";
import { GovernanceClient as GovSub } from "@stellarforge-protocol/sdk/governance";
import { RegistryClient as RegSub } from "@stellarforge-protocol/sdk/registry";
import { VaultClient as VaultSub, type VaultConfig, type VaultEvent } from "@stellarforge-protocol/sdk/vault";
import { OracleAdapterClient as OracleSub, type OracleAsset, type AssetConfig } from "@stellarforge-protocol/sdk/oracle";
import { toStroops, fromStroops } from "@stellarforge-protocol/sdk/utils";
import { TransactionExpiredError } from "@stellarforge-protocol/sdk/errors";
import { ContractClient } from "@stellarforge-protocol/sdk/base";
import type { Proposal } from "@stellarforge-protocol/sdk/types";

const config: StellarForgeConfig = {
  ...TESTNET_CONFIG,
  contracts: { rwaAsset: "CB..." },
};

const rwaClient: RwaAssetClient = new RwaSub(config);
const govClient: GovSub = new GovSub({ ...TESTNET_CONFIG, contracts: { governance: "CB..." } });
const vaultClient: VaultSub = new VaultSub({ ...TESTNET_CONFIG, contracts: { vault: "CB...", rwaAsset: "CB..." } });
const oracleClient: OracleSub = new OracleSub({ ...TESTNET_CONFIG, contracts: { oracleAdapter: "CB..." } });

export { rwaClient, govClient, vaultClient, oracleClient };
`
  );

  const tscBin = path.join(SDK_DIR, "node_modules", ".bin", "tsc");
  execSync(`"${tscBin}" -p "${tsConfigFile}"`, {
    stdio: "inherit",
    cwd: tempDir,
  });
  console.log("✓ TypeScript type resolution passed.");

  console.log("[test-package-imports] All package import and type tests passed successfully!");
} finally {
  // Cleanup
  if (fs.existsSync(tarballPath)) {
    fs.rmSync(tarballPath, { force: true });
  }
  if (fs.existsSync(tempDir)) {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}
