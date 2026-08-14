import {
  Contract,
  rpc,
  TransactionBuilder,
  BASE_FEE,
  xdr,
  scValToNative,
  nativeToScVal,
  Keypair,
  Account,
} from "@stellar/stellar-sdk";

import type { AssetMetadata, KycRecord, StellarForgeConfig } from "./types.js";

// ─── RwaAssetClient ───────────────────────────────────────────────────────────

/**
 * Minimal client for interacting with a deployed RwaAsset Soroban contract.
 *
 * For production use, transactions should be signed by a wallet (Freighter,
 * hardware wallet, etc.) rather than a raw secret. This client accepts an
 * optional `signerSecret` for automated/server contexts only.
 */
export class RwaAssetClient {
  private readonly server: rpc.Server;
  private readonly contract: Contract;
  private readonly config: StellarForgeConfig;

  constructor(config: StellarForgeConfig) {
    if (!config.contracts.rwaAsset) {
      throw new Error("contracts.rwaAsset address is required");
    }
    this.config = config;
    this.server = new rpc.Server(config.rpcUrl, { allowHttp: false });
    this.contract = new Contract(config.contracts.rwaAsset);
  }

  // ── Read-only queries ──────────────────────────────────────────────────────

  async balance(ownerAddress: string): Promise<bigint> {
    const result = await this.simulateReadOnly("balance", [
      nativeToScVal(ownerAddress, { type: "address" }),
    ]);
    return scValToNative(result) as bigint;
  }

  async totalSupply(): Promise<bigint> {
    const result = await this.simulateReadOnly("total_supply", []);
    return scValToNative(result) as bigint;
  }

  async metadata(): Promise<AssetMetadata> {
    const result = await this.simulateReadOnly("metadata", []);
    const native = scValToNative(result) as Record<string, unknown>;
    return {
      name: native["name"] as string,
      symbol: native["symbol"] as string,
      decimals: native["decimals"] as number,
      assetClass: native["asset_class"] as AssetMetadata["assetClass"],
      legalDocHash: Buffer.from(native["legal_doc_hash"] as Uint8Array).toString("hex"),
      maxSupply: native["max_supply"] as bigint,
    };
  }

  async isPaused(): Promise<boolean> {
    const result = await this.simulateReadOnly("paused", []);
    return scValToNative(result) as boolean;
  }

  async isIssuer(address: string): Promise<boolean> {
    const result = await this.simulateReadOnly("is_issuer", [
      nativeToScVal(address, { type: "address" }),
    ]);
    return scValToNative(result) as boolean;
  }

  async allowance(ownerAddress: string, spenderAddress: string): Promise<bigint> {
    const result = await this.simulateReadOnly("allowance", [
      nativeToScVal(ownerAddress, { type: "address" }),
      nativeToScVal(spenderAddress, { type: "address" }),
    ]);
    return scValToNative(result) as bigint;
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  private async simulateReadOnly(method: string, args: xdr.ScVal[]): Promise<xdr.ScVal> {
    // Use a throwaway account for simulating read-only calls
    const dummyKeypair = Keypair.random();
    const dummyAccount = new Account(dummyKeypair.publicKey(), "0");

    const tx = new TransactionBuilder(dummyAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const simResult = await this.server.simulateTransaction(tx);

    if (rpc.Api.isSimulationError(simResult)) {
      throw new Error(`Simulation error: ${simResult.error}`);
    }

    if (!simResult.result) {
      throw new Error(`No result returned from ${method}`);
    }

    return simResult.result.retval;
  }
}

// ─── ComplianceClient ─────────────────────────────────────────────────────────

export class ComplianceClient {
  private readonly server: rpc.Server;
  private readonly contract: Contract;
  private readonly config: StellarForgeConfig;

  constructor(config: StellarForgeConfig) {
    if (!config.contracts.compliance) {
      throw new Error("contracts.compliance address is required");
    }
    this.config = config;
    this.server = new rpc.Server(config.rpcUrl, { allowHttp: false });
    this.contract = new Contract(config.contracts.compliance);
  }

  async isCompliant(address: string, minLevel: 0 | 1 | 2 | 3): Promise<boolean> {
    const dummyKeypair = Keypair.random();
    const dummyAccount = new Account(dummyKeypair.publicKey(), "0");

    const tx = new TransactionBuilder(dummyAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(
        this.contract.call(
          "is_compliant",
          nativeToScVal(address, { type: "address" }),
          nativeToScVal(minLevel, { type: "u32" }),
        ),
      )
      .setTimeout(30)
      .build();

    const simResult = await this.server.simulateTransaction(tx);

    if (rpc.Api.isSimulationError(simResult)) {
      throw new Error(`Simulation error: ${simResult.error}`);
    }

    return scValToNative(simResult.result!.retval) as boolean;
  }
}
