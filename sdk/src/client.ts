import { scValToNative, nativeToScVal } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient } from "./base.js";
import type { AssetMetadata, StellarForgeConfig, TxResult } from "./types.js";

// ─── RwaAssetClient ───────────────────────────────────────────────────────────

/**
 * Minimal client for interacting with a deployed RwaAsset Soroban contract.
 *
 * For production use, transactions should be signed by a wallet (Freighter,
 * hardware wallet, etc.) rather than a raw secret. This client accepts an
 * optional `signerSecret` for automated/server contexts only.
 */
export class RwaAssetClient extends ContractClient {
  constructor(config: StellarForgeConfig) {
    super(config, config.contracts.rwaAsset, "rwaAsset");
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
    return this.submit(await this.buildMintTx(issuer, to, amount), issuer);
  }

  /** Burn `amount` from `from`'s own balance. */
  async buildBurnTx(from: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(from, "burn", [
      nativeToScVal(from, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async burn(from: string, amount: bigint): Promise<TxResult> {
    return this.submit(await this.buildBurnTx(from, amount), from);
  }

  /** Burn `amount` from `from`, drawing on `spender`'s allowance. Authorized by `spender`. */
  async buildBurnFromTx(spender: string, from: string, amount: bigint): Promise<Transaction> {
    return this.buildWriteTx(spender, "burn_from", [
      nativeToScVal(spender, { type: "address" }),
      nativeToScVal(from, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
    ]);
  }

  async burnFrom(spender: string, from: string, amount: bigint): Promise<TxResult> {
    return this.submit(await this.buildBurnFromTx(spender, from, amount), spender);
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
    return this.submit(await this.buildTransferTx(from, to, amount), from);
  }

  /**
   * Set `spender`'s allowance over `owner`'s balance to `amount`, spendable
   * through ledger `liveUntilLedger` inclusive (SEP-41). After that ledger the
   * allowance reads as zero. A ledger already past is accepted only with
   * `amount` 0, to revoke.
   */
  async buildApproveTx(
    owner: string,
    spender: string,
    amount: bigint,
    liveUntilLedger: number,
  ): Promise<Transaction> {
    return this.buildWriteTx(owner, "approve", [
      nativeToScVal(owner, { type: "address" }),
      nativeToScVal(spender, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
      nativeToScVal(liveUntilLedger, { type: "u32" }),
    ]);
  }

  async approve(
    owner: string,
    spender: string,
    amount: bigint,
    liveUntilLedger: number,
  ): Promise<TxResult> {
    return this.submit(await this.buildApproveTx(owner, spender, amount, liveUntilLedger), owner);
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
    return this.submit(await this.buildTransferFromTx(spender, from, to, amount), spender);
  }

}

// ─── ComplianceClient ─────────────────────────────────────────────────────────

export class ComplianceClient extends ContractClient {
  constructor(config: StellarForgeConfig) {
    super(config, config.contracts.compliance, "compliance");
  }

  /**
   * Whether `address` holds a valid, unexpired record at or above `minLevel`.
   *
   * The pure query, never the TTL-extending `screen`: this costs the caller
   * nothing and writes nothing. See ADR-001.
   *
   * Returns false for an address with no record at all, including at level 0.
   */
  async isCompliant(address: string, minLevel: 0 | 1 | 2 | 3): Promise<boolean> {
    const result = await this.simulateReadOnly("is_compliant", [
      nativeToScVal(address, { type: "address" }),
      nativeToScVal(minLevel, { type: "u32" }),
    ]);
    return scValToNative(result) as boolean;
  }
}
