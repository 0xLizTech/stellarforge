/**
 * Tokenize a commercial real estate property on Stellar testnet with @stellarforge-protocol/sdk.
 *
 * Companion script to docs/guides/tokenized-real-estate.md.
 *
 * Steps performed:
 *   1. Register KYC records for property sponsor (level 3) and investor (level 2).
 *   2. Link real estate rwa-asset contract to compliance (minimum level 2).
 *   3. Authorize property sponsor as issuer and set asset metadata (real_estate).
 *   4. List the property in the registry directory.
 *   5. Mint initial tokenized equity units to the investor and verify balances.
 *
 * Required environment:
 *   RWA_ASSET_ID     deployed rwa-asset contract id for the property (C...)
 *   COMPLIANCE_ID    deployed compliance contract id (C...)
 *   ADMIN_SECRET     secret key of the protocol admin (S...)
 *   ISSUER_SECRET    secret key of the property sponsor account (S...)
 *   INVESTOR_ADDRESS public key of the KYC-cleared investor (G...)
 * Optional:
 *   REGISTRY_ID      deployed registry contract id (C...)
 *   ISSUE_AMOUNT     whole tokens in the initial offering (default 500000)
 */

import { createHash } from "node:crypto";

import { Keypair } from "@stellar/stellar-sdk";

import {
  ComplianceClient,
  RegistryClient,
  RwaAssetClient,
  TESTNET_CONFIG,
} from "../src/index.js";
import type { ContractAddresses, StellarForgeConfig } from "../src/index.js";

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`Missing required environment variable ${name}`);
  return value;
}

const DECIMALS = 7n;
const units = (whole: number): bigint => BigInt(whole) * 10n ** DECIMALS;

async function main(): Promise<void> {
  const rwaAssetId = requiredEnv("RWA_ASSET_ID");
  const complianceId = requiredEnv("COMPLIANCE_ID");
  const registryId = process.env["REGISTRY_ID"];
  const contracts: ContractAddresses = {
    rwaAsset: rwaAssetId,
    compliance: complianceId,
    ...(registryId ? { registry: registryId } : {}),
  };
  const adminSecret = requiredEnv("ADMIN_SECRET");
  const issuerSecret = requiredEnv("ISSUER_SECRET");
  const investor = requiredEnv("INVESTOR_ADDRESS");
  const tranche = Number(process.env["ISSUE_AMOUNT"] ?? "500000");

  const admin = Keypair.fromSecret(adminSecret).publicKey();
  const issuer = Keypair.fromSecret(issuerSecret).publicKey();

  const asAdmin: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: adminSecret };
  const asIssuer: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: issuerSecret };

  const compliance = new ComplianceClient(asAdmin);
  const assetAsAdmin = new RwaAssetClient(asAdmin);
  const assetAsIssuer = new RwaAssetClient(asIssuer);

  // 1. Compliance KYC Setup
  await compliance.setKyc(admin, issuer, { jurisdiction: "US", level: 3, expiresAt: 0 });
  await compliance.setKyc(admin, investor, { jurisdiction: "US", level: 2, expiresAt: 0 });

  if (!(await compliance.isCompliant(investor, 2))) {
    throw new Error("Investor is not compliant at level 2 after setKyc — aborting issuance.");
  }
  console.log("Compliance: issuer and investor KYC verified.");

  // 2. Wire compliance into the real estate token
  await assetAsAdmin.setCompliance(admin, complianceId, 2);
  await assetAsAdmin.setIssuer(admin, issuer, true);
  console.log("Asset: compliance configured (level 2 min), issuer authorized.");

  // 3. Property metadata
  const deedSummary = "Grandview Plaza — 100 Main St, Commercial REIT Equity Units (Offering Memorandum v1)";
  const legalDocHash = createHash("sha256").update(deedSummary).digest("hex");
  await assetAsAdmin.updateMetadata(admin, {
    name: "Grandview Plaza REIT Units",
    symbol: "REIT-BLD1",
    decimals: 7,
    assetClass: "real_estate",
    legalDocHash,
    maxSupply: units(10_000_000),
  });
  console.log("Asset: metadata configured for REIT-BLD1.");

  // 4. Registry listing
  if (registryId) {
    const registry = new RegistryClient(asAdmin);
    await registry.register(admin, {
      contract: rwaAssetId,
      assetClass: "real_estate",
      active: true,
    });
    console.log("Registry: property listed in registry.");
  }

  // 5. Mint initial equity tranche
  const issued = await assetAsIssuer.mint(issuer, investor, units(tranche));
  console.log(`Minted ${tranche} REIT-BLD1 to ${investor} in tx ${issued.hash}.`);

  const balance = await assetAsIssuer.balance(investor);
  const supply = await assetAsIssuer.totalSupply();
  console.log(`Investor balance: ${balance / 10n ** DECIMALS} REIT-BLD1`);
  console.log(`Total supply:     ${supply / 10n ** DECIMALS} REIT-BLD1`);
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
