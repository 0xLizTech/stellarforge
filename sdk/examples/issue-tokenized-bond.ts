/**
 * Issue a tokenized bond on Stellar testnet with @stellarforge-protocol/sdk.
 *
 * This is the runnable companion to docs/guides/issue-tokenized-bond.md.
 * It assumes the StellarForge contracts are already deployed to testnet
 * (see scripts/deploy.sh) and walks the issuance flow end to end:
 *
 *   1. KYC the issuer and the investor in the compliance contract.
 *   2. Point the bond's rwa-asset contract at compliance (minimum level 2).
 *   3. Authorise the issuer and set the bond's metadata (terms + supply cap).
 *   4. Register the bond in the registry (when REGISTRY_ID is set).
 *   5. Mint the first tranche to the investor and verify balances.
 *
 * Required environment:
 *   RWA_ASSET_ID     deployed rwa-asset contract id for the bond (C...)
 *   COMPLIANCE_ID    deployed compliance contract id (C...)
 *   ADMIN_SECRET     secret key of the contracts' admin (S...)
 *   ISSUER_SECRET    secret key of the bond issuer account (S...)
 *   INVESTOR_ADDRESS public key of the first investor (G...)
 * Optional:
 *   REGISTRY_ID      deployed registry contract id (C...)
 *   ISSUE_AMOUNT     whole bonds in the first tranche (default 250000)
 *
 * Run from sdk/ after `npm install`:
 *   npx tsx examples/issue-tokenized-bond.ts
 *
 * Testnet only: the admin and issuer accounts must be funded first
 * (https://friendbot.stellar.org). Never use a mainnet secret with this
 * script; for production, sign with a wallet instead of a raw secret.
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

/** Bond amounts use Stellar's standard 7 decimals: 1 bond = 10^7 base units. */
const DECIMALS = 7n;
const units = (whole: number): bigint => BigInt(whole) * 10n ** DECIMALS;

async function main(): Promise<void> {
  const rwaAssetId = requiredEnv("RWA_ASSET_ID");
  const complianceId = requiredEnv("COMPLIANCE_ID");
  const registryId = process.env["REGISTRY_ID"];
  const contracts: ContractAddresses = {
    rwaAsset: rwaAssetId,
    compliance: complianceId,
    // exactOptionalPropertyTypes: only set the key when a registry exists.
    ...(registryId ? { registry: registryId } : {}),
  };
  const adminSecret = requiredEnv("ADMIN_SECRET");
  const issuerSecret = requiredEnv("ISSUER_SECRET");
  const investor = requiredEnv("INVESTOR_ADDRESS");
  const tranche = Number(process.env["ISSUE_AMOUNT"] ?? "250000");

  const admin = Keypair.fromSecret(adminSecret).publicKey();
  const issuer = Keypair.fromSecret(issuerSecret).publicKey();

  // One client set per signing role: a write's signer must be its source,
  // so admin calls and issuer calls use different signerSecret values.
  const asAdmin: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: adminSecret };
  const asIssuer: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: issuerSecret };

  const compliance = new ComplianceClient(asAdmin);
  const assetAsAdmin = new RwaAssetClient(asAdmin);
  const assetAsIssuer = new RwaAssetClient(asIssuer);

  // ── 1. Compliance setup ──────────────────────────────────────────────────
  // The issuer is an accredited entity (level 3); investors need full KYC
  // (level 2). expiresAt 0 means the records never expire.
  await compliance.setKyc(admin, issuer, { jurisdiction: "US", level: 3, expiresAt: 0 });
  await compliance.setKyc(admin, investor, { jurisdiction: "US", level: 2, expiresAt: 0 });

  if (!(await compliance.isCompliant(investor, 2))) {
    throw new Error("Investor is not compliant at level 2 after setKyc — aborting issuance.");
  }
  console.log("Compliance: issuer (level 3) and investor (level 2) registered.");

  // ── 2. Wire compliance into the bond and authorise the issuer ────────────
  // minLevel 2 means both parties to every mint/transfer must hold at least
  // a full-KYC record. Screening stays on until setCompliance(admin, null, …).
  await assetAsAdmin.setCompliance(admin, complianceId, 2);
  await assetAsAdmin.setIssuer(admin, issuer, true);
  console.log("Asset: compliance screening on (min level 2), issuer authorised.");

  // ── 3. Bond terms as metadata ────────────────────────────────────────────
  // legalDocHash anchors the offering document on-chain: the SHA-256 of the
  // term sheet. maxSupply caps the whole programme at 1,000,000 bonds; the
  // cap can only be tightened later, and decimals can never change.
  const termSheet = "Acme Corp — 5.00% Senior Unsecured Bond, due 2030-06-15 (testnet demo)";
  const legalDocHash = createHash("sha256").update(termSheet).digest("hex");
  await assetAsAdmin.updateMetadata(admin, {
    name: "Acme Corp 5.00% Senior Bond 2030",
    symbol: "ACME30",
    decimals: 7,
    assetClass: "debt",
    legalDocHash,
    maxSupply: units(1_000_000),
  });
  console.log("Asset: metadata set (ACME30, cap 1,000,000 bonds).");

  // ── 4. Register the bond (optional) ──────────────────────────────────────
  if (registryId) {
    const registry = new RegistryClient(asAdmin);
    await registry.register(admin, {
      contract: rwaAssetId,
      assetClass: "debt",
      active: true,
    });
    console.log("Registry: bond registered as an active debt asset.");
  }

  // ── 5. Issue the first tranche ───────────────────────────────────────────
  const issued = await assetAsIssuer.mint(issuer, investor, units(tranche));
  console.log(`Minted ${tranche} ACME30 to ${investor} in tx ${issued.hash}.`);

  const balance = await assetAsIssuer.balance(investor);
  const supply = await assetAsIssuer.totalSupply();
  console.log(`Investor balance: ${balance / 10n ** DECIMALS} ACME30`);
  console.log(`Total supply:     ${supply / 10n ** DECIMALS} ACME30`);
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
