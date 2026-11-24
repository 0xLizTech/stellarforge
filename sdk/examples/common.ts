import { Keypair } from "@stellar/stellar-sdk";
import { TESTNET_CONFIG, type StellarForgeConfig } from "../src/index.js";

/** Read a required environment variable, with an actionable error when absent. */
export function required(name: string): string {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`Set ${name} before running this example.`);
  return value;
}

/** Build the shared testnet config for a single contract client. */
export function configFor(contract: "rwaAsset" | "compliance" | "governance"): StellarForgeConfig {
  if (process.env.STELLAR_NETWORK_PASSPHRASE && process.env.STELLAR_NETWORK_PASSPHRASE !== TESTNET_CONFIG.networkPassphrase) {
    throw new Error("These examples are restricted to Stellar testnet.");
  }
  const contractEnv = {
    rwaAsset: "RWA_ASSET_CONTRACT_ID",
    compliance: "COMPLIANCE_CONTRACT_ID",
    governance: "GOVERNANCE_CONTRACT_ID",
  }[contract];
  return {
    ...TESTNET_CONFIG,
    rpcUrl: process.env.SOROBAN_RPC_URL ?? TESTNET_CONFIG.rpcUrl,
    contracts: { [contract]: required(contractEnv) },
    signerSecret: required("SIGNER_SECRET"),
  };
}

/** Resolve the configured signer address. */
export function signerAddress(): string {
  return Keypair.fromSecret(required("SIGNER_SECRET")).publicKey();
}

/** Print a submitted transaction in a format useful for copying into a block explorer. */
export function printResult(result: { hash: string; ledger: number }): void {
  console.log(`Transaction: ${result.hash}\nLedger: ${result.ledger}`);
}
