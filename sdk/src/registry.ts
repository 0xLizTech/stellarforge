import { nativeToScVal, scValToNative, xdr } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient } from "./base.js";
import type { AssetClass, AssetEntry, StellarForgeConfig, TxResult } from "./types.js";

/** Largest page `list_assets` accepts. Mirrors `MAX_PAGE_SIZE` in the contract. */
export const REGISTRY_MAX_PAGE_SIZE = 100;

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
   * One page of registered asset addresses, in registration order.
   *
   * Registering an already-known asset updates it in place rather than
   * appending, so an address appears at most once across all pages.
   *
   * @param start - Directory index of the first address to return.
   * @param limit - Page size, at most {@link REGISTRY_MAX_PAGE_SIZE}. The
   *   contract rejects a larger page with `PageTooLarge` rather than clipping.
   * @returns Fewer than `limit` addresses once the end is reached.
   */
  async listAssets(start = 0, limit = REGISTRY_MAX_PAGE_SIZE): Promise<string[]> {
    const result = await this.simulateReadOnly("list_assets", [
      nativeToScVal(start, { type: "u32" }),
      nativeToScVal(limit, { type: "u32" }),
    ]);
    return scValToNative(result) as string[];
  }

  /** How many distinct assets are registered. */
  async assetCount(): Promise<number> {
    const result = await this.simulateReadOnly("asset_count", []);
    return scValToNative(result) as number;
  }

  /**
   * Every registered asset address, fetched page by page.
   *
   * One simulation per {@link REGISTRY_MAX_PAGE_SIZE} assets. Pages are read
   * at different moments, so an asset registered mid-walk may or may not
   * appear; none is ever listed twice, since indices only ever grow.
   */
  async listAllAssets(): Promise<string[]> {
    const all: string[] = [];
    for (;;) {
      const page = await this.listAssets(all.length, REGISTRY_MAX_PAGE_SIZE);
      all.push(...page);
      if (page.length < REGISTRY_MAX_PAGE_SIZE) {
        return all;
      }
    }
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
