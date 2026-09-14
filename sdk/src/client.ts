import { nativeToScVal, scValToNative, xdr } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient, hexToScVal } from "./base.js";
import type { AssetMetadata, KycRecord, StellarForgeConfig, TxResult } from "./types.js";

// ─── Struct codecs ────────────────────────────────────────────────────────────

/** One field of the ScMap a Soroban struct decodes from. */
function structField(key: string, val: xdr.ScVal): xdr.ScMapEntry {
  return new xdr.ScMapEntry({ key: nativeToScVal(key, { type: "symbol" }), val });
}

/** Encodes `AssetMetadata` as the struct `update_metadata` decodes. Keys must be sorted. */
function assetMetadataToScVal(metadata: AssetMetadata): xdr.ScVal {
  return xdr.ScVal.scvMap([
    structField("asset_class", nativeToScVal(metadata.assetClass, { type: "string" })),
    structField("decimals", nativeToScVal(metadata.decimals, { type: "u32" })),
    structField("legal_doc_hash", hexToScVal(metadata.legalDocHash)),
    structField("max_supply", nativeToScVal(metadata.maxSupply, { type: "i128" })),
    structField("name", nativeToScVal(metadata.name, { type: "string" })),
    structField("symbol", nativeToScVal(metadata.symbol, { type: "string" })),
  ]);
}

/** Encodes a `KycRecord` as the struct `set_kyc` decodes. Keys must be sorted. */
function kycRecordToScVal(record: KycRecord): xdr.ScVal {
  return xdr.ScVal.scvMap([
    structField("expires_at", nativeToScVal(BigInt(record.expiresAt), { type: "u64" })),
    structField("jurisdiction", nativeToScVal(record.jurisdiction, { type: "string" })),
    structField("level", nativeToScVal(record.level, { type: "u32" })),
  ]);
}

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

  /** The asset's admin: the address every admin operation below is authorized by. */
  async admin(): Promise<string> {
    const result = await this.simulateReadOnly("admin", []);
    return scValToNative(result) as string;
  }

  /**
   * The compliance contract that screens transfers, or `null` when screening is
   * off. An asset with no compliance contract transfers between anyone
   * (ADR-004), so this is the check to make before relying on screening.
   */
  async complianceContract(): Promise<string | null> {
    const result = await this.simulateReadOnly("compliance_contract", []);
    return (scValToNative(result) as string | null | undefined) ?? null;
  }

  /** The verification level both parties to a screened transfer must hold. */
  async minComplianceLevel(): Promise<number> {
    const result = await this.simulateReadOnly("min_compliance_level", []);
    return scValToNative(result) as number;
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

  // ── Admin operations ───────────────────────────────────────────────────────
  //
  // Authorized by the asset's admin, who also sources and pays for the
  // transaction, exactly as the writes above do. `admin()` returns the address.
  //
  // `transfer_admin` is not offered. It needs the current and the incoming admin
  // to sign one transaction, which this same-address write path cannot build.
  // Until separately signed authorization entries land (#31), hand the role over
  // with `stellar-cli`.

  /** Grant (`approved` true) or revoke the issuer role for `issuer`. */
  async buildSetIssuerTx(admin: string, issuer: string, approved: boolean): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_issuer", [
      nativeToScVal(issuer, { type: "address" }),
      nativeToScVal(approved),
    ]);
  }

  async setIssuer(admin: string, issuer: string, approved: boolean): Promise<TxResult> {
    return this.submit(await this.buildSetIssuerTx(admin, issuer, approved), admin);
  }

  /**
   * Pause (`true`) or unpause the asset. While it is paused, mint, burn,
   * transfer and approve all fail. Admin operations keep working.
   */
  async buildSetPausedTx(admin: string, paused: boolean): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_paused", [nativeToScVal(paused)]);
  }

  async setPaused(admin: string, paused: boolean): Promise<TxResult> {
    return this.submit(await this.buildSetPausedTx(admin, paused), admin);
  }

  /**
   * Replace the asset's metadata.
   *
   * The contract refuses a different `decimals`, a cap below circulating supply,
   * and lifting a cap back to uncapped. Building simulates, so those surface here
   * before anything is signed. `legalDocHash` must be even-length hex, and is
   * checked before any network call.
   */
  async buildUpdateMetadataTx(admin: string, metadata: AssetMetadata): Promise<Transaction> {
    return this.buildWriteTx(admin, "update_metadata", [assetMetadataToScVal(metadata)]);
  }

  async updateMetadata(admin: string, metadata: AssetMetadata): Promise<TxResult> {
    return this.submit(await this.buildUpdateMetadataTx(admin, metadata), admin);
  }

  /**
   * Point the asset at a compliance contract, or pass `null` to switch screening
   * off. `minLevel` is the verification level both parties to a transfer must
   * hold.
   *
   * Level 0 is not "screening off": every party must still hold a record
   * (ADR-004). Only `null` turns screening off.
   */
  async buildSetComplianceTx(
    admin: string,
    compliance: string | null,
    minLevel: 0 | 1 | 2 | 3,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_compliance", [
      compliance === null ? xdr.ScVal.scvVoid() : nativeToScVal(compliance, { type: "address" }),
      nativeToScVal(minLevel, { type: "u32" }),
    ]);
  }

  async setCompliance(
    admin: string,
    compliance: string | null,
    minLevel: 0 | 1 | 2 | 3,
  ): Promise<TxResult> {
    return this.submit(await this.buildSetComplianceTx(admin, compliance, minLevel), admin);
  }
}

// ─── ComplianceClient ─────────────────────────────────────────────────────────

/**
 * Client for the deployed compliance contract, which holds the KYC records
 * `rwa-asset` screens transfers against.
 *
 * Writes are admin-only and follow the same build-or-submit split as
 * {@link RwaAssetClient}. `transfer_admin` is not offered, for the same reason.
 */
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

  /**
   * `subject`'s stored verification record, or `null` when there is none.
   *
   * Returned as stored, expired or not. Use {@link isCompliant} to evaluate a
   * record against the ledger's time and a required level.
   */
  async getKyc(subject: string): Promise<KycRecord | null> {
    const result = await this.simulateReadOnly("get_kyc", [
      nativeToScVal(subject, { type: "address" }),
    ]);
    const native = scValToNative(result) as Record<string, unknown> | null | undefined;
    if (native == null) {
      return null;
    }
    return {
      jurisdiction: native["jurisdiction"] as string,
      level: native["level"] as KycRecord["level"],
      expiresAt: Number(native["expires_at"] as bigint),
    };
  }

  /** The address permitted to set and revoke records. */
  async admin(): Promise<string> {
    const result = await this.simulateReadOnly("admin", []);
    return scValToNative(result) as string;
  }

  // ── Admin operations ───────────────────────────────────────────────────────

  /**
   * Set or replace `subject`'s verification record. Authorized and paid for by
   * the compliance contract's admin.
   *
   * The contract refuses a level above 3, an expiry that has already passed
   * (`expiresAt` 0 means never), and a jurisdiction that is not two uppercase
   * letters. Building simulates, so those surface here before anything is signed.
   */
  async buildSetKycTx(admin: string, subject: string, record: KycRecord): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_kyc", [
      nativeToScVal(subject, { type: "address" }),
      kycRecordToScVal(record),
    ]);
  }

  async setKyc(admin: string, subject: string, record: KycRecord): Promise<TxResult> {
    return this.submit(await this.buildSetKycTx(admin, subject, record), admin);
  }

  /**
   * Remove `subject`'s record. Revoking a subject with no record succeeds and
   * changes nothing.
   */
  async buildRevokeKycTx(admin: string, subject: string): Promise<Transaction> {
    return this.buildWriteTx(admin, "revoke_kyc", [nativeToScVal(subject, { type: "address" })]);
  }

  async revokeKyc(admin: string, subject: string): Promise<TxResult> {
    return this.submit(await this.buildRevokeKycTx(admin, subject), admin);
  }
}
