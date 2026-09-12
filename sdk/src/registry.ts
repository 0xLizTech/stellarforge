import { nativeToScVal, scValToNative, xdr } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient } from "./base.js";
import type { AssetClass, AssetEntry, StellarForgeConfig, TxResult } from "./types.js";

/** Encodes an `AssetEntry` as the ScMap the contract decodes a struct from. */
function assetEntryToScVal(entry: AssetEntry): xdr.ScVal {
  const field = (key: string, val: xdr.ScVal): xdr.ScMapEntry =>
    new xdr.ScMapEntry({ key: nativeToScVal(key, { type: "symbol" }), val });

  // Soroban requires map keys in sorted order.
  return xdr.ScVal.scvMap([
    field("active", nativeToScVal(entry.active)),
    field("asset_class", nativeToScVal(entry.assetClass, { type: "string" })),
    field("contract", nativeToScVal(entry.contract, { type: "address" })),
  ]);
}

/**
 * Client for the deployed registry contract: the protocol's directory of
 * known `rwa-asset` contracts.
 *
 * Reads are pure queries that extend no TTLs, so they cost the caller nothing.
 * Writes are admin-only and follow the same build-or-submit split as
 * {@link RwaAssetClient}.
 */
export class RegistryClient extends ContractClient {
  constructor(config: StellarForgeConfig) {
    super(config, config.contracts.registry, "registry");
  }

  // ── Read-only queries ────────────────────────────────────────────────────

  /**
   * Looks up a single registered asset.
   *
   * @returns The entry, or `null` when the contract has never been registered.
   *   An unknown address is not an error.
   */
  async getAsset(contractAddress: string): Promise<AssetEntry | null> {
    const result = await this.simulateReadOnly("get_asset", [
      nativeToScVal(contractAddress, { type: "address" }),
    ]);

    // get_asset returns Option<AssetEntry>, and None decodes to null.
    const native = scValToNative(result) as Record<string, unknown> | null;
    if (native === null) {
      return null;
    }

    return {
      contract: native["contract"] as string,
      assetClass: native["asset_class"] as AssetClass,
      active: native["active"] as boolean,
    };
  }

  /**
   * Every registered asset address, in registration order.
   *
   * Registering an already-known asset updates it in place rather than
   * appending, so an address appears at most once.
   */
  async listAssets(): Promise<string[]> {
    const result = await this.simulateReadOnly("list_assets", []);
    return scValToNative(result) as string[];
  }

  /** The address permitted to register assets and change their active flag. */
  async admin(): Promise<string> {
    const result = await this.simulateReadOnly("admin", []);
    return scValToNative(result) as string;
  }

  // ── Write path ───────────────────────────────────────────────────────────

  /**
   * Record an asset, or update one already recorded.
   *
   * @param admin - The registry admin, which sources and signs the
   *   transaction. Read it from {@link admin} if you do not already know it.
   */
  async buildRegisterTx(admin: string, entry: AssetEntry): Promise<Transaction> {
    return this.buildWriteTx(admin, "register", [assetEntryToScVal(entry)]);
  }

  async register(admin: string, entry: AssetEntry): Promise<TxResult> {
    return this.submit(await this.buildRegisterTx(admin, entry), admin);
  }

  /**
   * Flip an asset's active flag.
   *
   * Fails with `AssetNotFound` if the asset was never registered, which
   * surfaces while building rather than on-chain.
   */
  async buildSetActiveTx(
    admin: string,
    contractAddress: string,
    active: boolean,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_active", [
      nativeToScVal(contractAddress, { type: "address" }),
      nativeToScVal(active),
    ]);
  }

  async setActive(admin: string, contractAddress: string, active: boolean): Promise<TxResult> {
    return this.submit(await this.buildSetActiveTx(admin, contractAddress, active), admin);
  }
}
