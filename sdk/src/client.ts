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

import type { Transaction } from "@stellar/stellar-sdk";

import type { AssetMetadata, KycRecord, StellarForgeConfig, TxResult } from "./types.js";

/**
 * Validity window for a write transaction, in seconds.
 *
 * Long enough to survive a slow submission, short enough that an unsubmitted
 * signed transaction stops being replayable reasonably soon.
 */
const WRITE_TX_TIMEOUT_SECONDS = 180;

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

  // ── Write path ─────────────────────────────────────────────────────────────
  //
  // Each write comes in two forms.
  //
  // `build*Tx` returns a prepared but unsigned transaction, for a wallet
  // (Freighter, hardware, multisig) to sign. No secret ever reaches this
  // library, which is the only form usable in a browser.
  //
  // The bare method signs with `config.signerSecret` and submits, for
  // server-side automation. It is a thin wrapper over the builder.
  //
  // Both assume the authorizing address is also the transaction source, so the
  // source signature satisfies the contract's `require_auth`. Paying fees from
  // a different account needs signed authorization entries and is not supported
  // here.
  //
  // Note that `build*Tx` simulates as part of preparing, so a call the contract
  // would reject — a bad amount, a paused asset, a party failing compliance —
  // fails at build time with the contract's own error, before anything is
  // signed or submitted.

  /** Mint `amount` to `to`. Authorized and paid for by `issuer`. */
  async buildMintTx(issuer: string, to: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(issuer, "mint", [
      nativeToScVal(issuer, { type: "address" }),
      nativeToScVal(to, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async mint(issuer: string, to: string, amount: bigint): Promise<TxResult> {
    return this.signAndSubmit(await this.buildMintTx(issuer, to, amount), issuer);
  }

  /** Burn `amount` from `from`'s own balance. */
  async buildBurnTx(from: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(from, "burn", [
      nativeToScVal(from, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async burn(from: string, amount: bigint): Promise<TxResult> {
    return this.signAndSubmit(await this.buildBurnTx(from, amount), from);
  }

  /** Transfer `amount` from `from` to `to`. */
  async buildTransferTx(from: string, to: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(from, "transfer", [
      nativeToScVal(from, { type: "address" }),
      nativeToScVal(to, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async transfer(from: string, to: string, amount: bigint): Promise<TxResult> {
    return this.signAndSubmit(await this.buildTransferTx(from, to, amount), from);
  }

  /** Set `spender`'s allowance over `owner`'s balance to `amount`. */
  async buildApproveTx(owner: string, spender: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(owner, "approve", [
      nativeToScVal(owner, { type: "address" }),
      nativeToScVal(spender, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async approve(owner: string, spender: string, amount: bigint): Promise<TxResult> {
    return this.signAndSubmit(await this.buildApproveTx(owner, spender, amount), owner);
  }

  /** Move `amount` from `from` to `to`, drawing on `spender`'s allowance. */
  async buildTransferFromTx(
    spender: string,
    from: string,
    to: string,
    amount: bigint,
  ): Promise<Transaction> {
    return this.buildWriteTx(spender, "transfer_from", [
      nativeToScVal(spender, { type: "address" }),
      nativeToScVal(from, { type: "address" }),
      nativeToScVal(to, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async transferFrom(
    spender: string,
    from: string,
    to: string,
    amount: bigint,
  ): Promise<TxResult> {
    return this.signAndSubmit(await this.buildTransferFromTx(spender, from, to, amount), spender);
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  /**
   * Builds and prepares an invocation sourced from `authorizer`, whose
   * signature is what satisfies the contract's `require_auth`.
   */
  private async buildWriteTx(
    authorizer: string,
    method: string,
    args: xdr.ScVal[],
  ): Promise<Transaction> {
    const account = await this.server.getAccount(authorizer);

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(WRITE_TX_TIMEOUT_SECONDS)
      .build();

    // Simulates, then attaches the footprint, authorization entries and
    // resource fee the transaction needs to be accepted.
    return this.server.prepareTransaction(tx);
  }

  /** Signs with the configured secret and waits for the transaction to settle. */
  private async signAndSubmit(tx: Transaction, authorizer: string): Promise<TxResult> {
    const secret = this.config.signerSecret;
    if (!secret) {
      throw new Error(
        "config.signerSecret is required to submit a transaction. " +
          "Use the matching build*Tx method to sign with a wallet instead.",
      );
    }

    const keypair = Keypair.fromSecret(secret);

    // Under same-address auth the signer must be the authorizing address.
    // Catching it here beats a require_auth failure after the fee is spent.
    if (keypair.publicKey() !== authorizer) {
      throw new Error(
        `signerSecret is for ${keypair.publicKey()} but this call must be authorized by ` +
          `${authorizer}. Paying from a different account is not supported.`,
      );
    }

    tx.sign(keypair);

    const sent = await this.server.sendTransaction(tx);
    if (sent.status === "ERROR") {
      throw new Error(`Transaction ${sent.hash} was rejected on submission`);
    }

    const settled = await this.server.pollTransaction(sent.hash);
    if (settled.status !== rpc.Api.GetTransactionStatus.SUCCESS) {
      throw new Error(`Transaction ${sent.hash} did not succeed: ${settled.status}`);
    }

    return { hash: sent.hash, ledger: settled.ledger };
  }

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

    if (!simResult.result) {
      throw new Error("No result returned from is_compliant");
    }

    return scValToNative(simResult.result.retval) as boolean;
  }
}
