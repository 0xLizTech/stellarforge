import { nativeToScVal, scValToNative, xdr } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient, unwrapEnumVariant } from "./base.js";
import type { SponsoredTransaction } from "./sponsored.js";
import type { StellarForgeConfig, TxResult } from "./types.js";

/** Inclusive i128 bounds used by the oracle-adapter contract. */
const I128_MAX = (1n << 127n) - 1n;
const I128_MIN = -(1n << 127n);

// ─── Oracle Types ─────────────────────────────────────────────────────────────

/**
 * SEP-40 asset identifier.
 *
 * Can identify either a Stellar token/contract address or an off-chain asset/currency symbol.
 */
export type OracleAsset =
  | { type: "stellar"; address: string }
  | { type: "other"; symbol: string };

/**
 * SEP-40 price record.
 *
 * `price` is scaled by `10^decimals()`, and `timestamp` is in Unix seconds.
 */
export interface PriceData {
  price: bigint;
  timestamp: bigint;
}

/**
 * Configuration for a listed asset.
 */
export interface AssetConfig {
  /**
   * Largest price movement from the latest recorded price a reporter may submit,
   * in basis points (1 bp = 0.01%, 10_000 bps = 100%).
   */
  maxDeviationBps: number;
}

/**
 * Contract error discriminants matching `OracleError` in `contracts/oracle-adapter/src/error.rs`.
 */
export enum OracleErrorCode {
  NotInitialized = 1,
  InvalidDecimals = 2,
  InvalidResolution = 3,
  UnknownAsset = 4,
  AssetAlreadyExists = 5,
  TooManyAssets = 6,
  NotReporter = 7,
  InvalidPrice = 8,
  TimestampInFuture = 9,
  StaleReport = 10,
  DeviationTooLarge = 11,
  InvalidDeviation = 12,
  Overflow = 13,
}

// ─── ScVal Codecs ─────────────────────────────────────────────────────────────

/**
 * Encodes an `OracleAsset` into the Soroban ScVal representation matching Rust enum `Asset`.
 *
 * Rust representation:
 * - `Asset::Stellar(Address)` -> `Vec[Symbol("Stellar"), Address]`
 * - `Asset::Other(Symbol)` -> `Vec[Symbol("Other"), Symbol]`
 */
export function assetToScVal(asset: OracleAsset): xdr.ScVal {
  if (asset.type === "stellar") {
    return xdr.ScVal.scvVec([
      nativeToScVal("Stellar", { type: "symbol" }),
      nativeToScVal(asset.address, { type: "address" }),
    ]);
  }
  if (asset.type === "other") {
    return xdr.ScVal.scvVec([
      nativeToScVal("Other", { type: "symbol" }),
      nativeToScVal(asset.symbol, { type: "symbol" }),
    ]);
  }
  throw new Error(`Invalid oracle asset: ${JSON.stringify(asset)}`);
}

/**
 * Decodes a native `scValToNative` value into an `OracleAsset`.
 *
 * Shared by {@link scValToAsset} and any path that already holds the decoded
 * tuple, so both enforce the same tag/payload checks. The tag is unwrapped the
 * same way {@link unwrapEnumVariant} handles unit enums.
 */
export function nativeToAsset(native: unknown): OracleAsset {
  if (!Array.isArray(native) || native.length < 2) {
    throw new Error(`Expected tuple enum variant for Asset, got ${JSON.stringify(native)}`);
  }

  const tag = unwrapEnumVariant(native[0]);
  const payload = native[1];
  if (tag === "Stellar" && typeof payload === "string") {
    return { type: "stellar", address: payload };
  }
  if (tag === "Other" && typeof payload === "string") {
    return { type: "other", symbol: payload };
  }

  throw new Error(`Unrecognized Asset variant: ${JSON.stringify(native)}`);
}

/**
 * Decodes an ScVal into an `OracleAsset`.
 */
export function scValToAsset(val: xdr.ScVal): OracleAsset {
  return nativeToAsset(scValToNative(val));
}

function decodePriceData(native: unknown): PriceData {
  const record = native as Record<string, unknown>;
  return {
    price: record["price"] as bigint,
    timestamp: record["timestamp"] as bigint,
  };
}

// ─── Pure Helper Math ─────────────────────────────────────────────────────────

const BPS_DENOMINATOR = 10_000n;

/**
 * Calculates the resolution tick timestamp for a given timestamp and resolution.
 * Matches `timestamp - timestamp % resolution`.
 */
export function tickOf(timestamp: bigint, resolution: number): bigint {
  if (resolution <= 0) {
    throw new RangeError(`resolution must be positive, got ${resolution}`);
  }
  const res = BigInt(resolution);
  return timestamp - (timestamp % res);
}

/**
 * Thrown by {@link checkDeviation} when an intermediate product exceeds i128,
 * matching the contract's `OracleError::Overflow` panic.
 */
export class OracleOverflowError extends Error {
  readonly code = OracleErrorCode.Overflow;

  constructor(message = "Oracle deviation check overflowed i128") {
    super(message);
    this.name = "OracleOverflowError";
  }
}

/**
 * Thrown by {@link checkDeviation} when `maxDeviationBps` is 0, matching the
 * contract's `OracleError::InvalidDeviation` (the feed never stores a zero limit).
 */
export class OracleInvalidDeviationError extends Error {
  readonly code = OracleErrorCode.InvalidDeviation;

  constructor(message = "maxDeviationBps must be at least 1") {
    super(message);
    this.name = "OracleInvalidDeviationError";
  }
}

/**
 * Multiplies two integers and throws {@link OracleOverflowError} if the product
 * would not fit in a signed 128-bit integer, matching Soroban's `checked_mul`.
 */
function checkedMulI128(a: bigint, b: bigint): bigint {
  const product = a * b;
  if (product > I128_MAX || product < I128_MIN) {
    throw new OracleOverflowError();
  }
  return product;
}

function assertNonNegativeInteger(name: string, value: number): void {
  if (!Number.isInteger(value) || value < 0) {
    throw new RangeError(`${name} must be a non-negative integer, got ${value}`);
  }
}

/**
 * Checks whether a proposed next price is within `maxDeviationBps` of previous price.
 *
 * Mirrors the contract's `record` deviation check: rejects `maxDeviationBps = 0`
 * ({@link OracleInvalidDeviationError}), and throws {@link OracleOverflowError}
 * when `diff * 10_000` or `previous * maxDeviationBps` exceeds i128.
 *
 * Exact integer arithmetic without floating point:
 * `|next - previous| * 10_000 <= previous * maxDeviationBps`
 */
export function checkDeviation(
  previous: bigint,
  next: bigint,
  maxDeviationBps: number,
): boolean {
  if (previous <= 0n) {
    throw new RangeError(`previous price must be positive, got ${previous}`);
  }
  if (next <= 0n) {
    throw new RangeError(`next price must be positive, got ${next}`);
  }
  if (!Number.isInteger(maxDeviationBps) || maxDeviationBps < 0) {
    throw new RangeError(`maxDeviationBps must be a non-negative integer, got ${maxDeviationBps}`);
  }
  if (maxDeviationBps === 0) {
    throw new OracleInvalidDeviationError(
      `maxDeviationBps must be at least 1 (InvalidDeviation), got ${maxDeviationBps}`,
    );
  }

  // Contract: moved = (price - latest).abs(); both prices are positive so the
  // subtraction cannot overflow i128, then checked_mul for both sides.
  const diff = next >= previous ? next - previous : previous - next;
  const lhs = checkedMulI128(diff, BPS_DENOMINATOR);
  const rhs = checkedMulI128(previous, BigInt(maxDeviationBps));
  return lhs <= rhs;
}

/**
 * Checks if a price record is fresh relative to `nowSecs` and `maxAgeSecs`.
 * Rejects records timestamped in the future.
 */
export function isFresh(price: PriceData, maxAgeSecs: number, nowSecs: number): boolean {
  assertNonNegativeInteger("maxAgeSecs", maxAgeSecs);
  assertNonNegativeInteger("nowSecs", nowSecs);
  const now = BigInt(nowSecs);
  const maxAge = BigInt(maxAgeSecs);
  if (price.timestamp > now) {
    return false;
  }
  return now - price.timestamp <= maxAge;
}

/**
 * Formats a scaled bigint price to a human-readable decimal string without precision loss.
 */
export function toDecimalString(value: bigint, decimals: number): string {
  if (decimals < 0 || !Number.isInteger(decimals)) {
    throw new RangeError(`decimals must be a non-negative integer, got ${decimals}`);
  }
  const sign = value < 0n ? "-" : "";
  const abs = value < 0n ? -value : value;
  if (decimals === 0) {
    return `${sign}${abs.toString()}`;
  }
  const strVal = abs.toString().padStart(decimals + 1, "0");
  const intPart = strVal.slice(0, strVal.length - decimals);
  const fracPart = strVal.slice(strVal.length - decimals).replace(/0+$/, "");
  return `${sign}${intPart}${fracPart ? `.${fracPart}` : ""}`;
}

// ─── OracleAdapterClient ──────────────────────────────────────────────────────

/**
 * Client for interacting with the SEP-40 Oracle Adapter contract.
 *
 * Implements the SEP-40 consumer query interface (`base`, `assets`, `decimals`,
 * `resolution`, `price`, `prices`, `lastprice`) as well as reporter write and
 * admin management functions.
 */
export class OracleAdapterClient extends ContractClient {
  constructor(config: StellarForgeConfig) {
    super(config, config.contracts.oracleAdapter, "oracleAdapter");
  }

  // ── SEP-40 Queries ──────────────────────────────────────────────────────────

  /**
   * The base asset against which prices in this oracle are quoted (for example `USD`).
   */
  async base(): Promise<OracleAsset> {
    const result = await this.simulateReadOnly("base", []);
    return scValToAsset(result);
  }

  /**
   * List of all listed assets in the oracle feed in the order they were added.
   */
  async assets(): Promise<OracleAsset[]> {
    const result = await this.simulateReadOnly("assets", []);
    const native = scValToNative(result) as unknown;
    if (!Array.isArray(native)) {
      throw new Error(`Expected assets vec, got ${JSON.stringify(native)}`);
    }
    // Decode each entry through the shared Asset decoder (same checks as scValToAsset).
    return native.map((entry) => nativeToAsset(entry));
  }

  /**
   * Number of decimal places used to scale prices.
   */
  async decimals(): Promise<number> {
    const result = await this.simulateReadOnly("decimals", []);
    return scValToNative(result) as number;
  }

  /**
   * Length of one tick resolution in seconds.
   */
  async resolution(): Promise<number> {
    const result = await this.simulateReadOnly("resolution", []);
    return scValToNative(result) as number;
  }

  /**
   * Fetches the price recorded in the resolution tick containing `timestamp`, if any.
   */
  async price(asset: OracleAsset, timestamp: bigint | number): Promise<PriceData | null> {
    const result = await this.simulateReadOnly("price", [
      assetToScVal(asset),
      nativeToScVal(BigInt(timestamp), { type: "u64" }),
    ]);
    const native = scValToNative(result);
    if (native === null || native === undefined) {
      return null;
    }
    return decodePriceData(native);
  }

  /**
   * Fetches up to `records` recent prices for `asset`, newest first.
   */
  async prices(asset: OracleAsset, records: number): Promise<PriceData[] | null> {
    const result = await this.simulateReadOnly("prices", [
      assetToScVal(asset),
      nativeToScVal(records, { type: "u32" }),
    ]);
    const native = scValToNative(result);
    if (native === null || native === undefined) {
      return null;
    }
    const list = native as unknown[];
    return list.map(decodePriceData);
  }

  /**
   * Fetches the latest recorded price for `asset`, or `null` if no price has been reported.
   */
  async lastprice(asset: OracleAsset): Promise<PriceData | null> {
    const result = await this.simulateReadOnly("lastprice", [assetToScVal(asset)]);
    const native = scValToNative(result);
    if (native === null || native === undefined) {
      return null;
    }
    return decodePriceData(native);
  }

  // ── Administrative and Reporter Queries ─────────────────────────────────────

  /**
   * The current admin of the oracle adapter contract.
   */
  async admin(): Promise<string> {
    const result = await this.simulateReadOnly("admin", []);
    return scValToNative(result) as string;
  }

  /**
   * Checks whether `reporter` is authorized to submit price reports for `asset`.
   */
  async isReporter(asset: OracleAsset, reporter: string): Promise<boolean> {
    const result = await this.simulateReadOnly("is_reporter", [
      assetToScVal(asset),
      nativeToScVal(reporter, { type: "address" }),
    ]);
    return scValToNative(result) as boolean;
  }

  /**
   * Fetches the configuration for `asset`, or `null` if the asset is not listed.
   */
  async assetConfig(asset: OracleAsset): Promise<AssetConfig | null> {
    const result = await this.simulateReadOnly("asset_config", [assetToScVal(asset)]);
    const native = scValToNative(result) as Record<string, unknown> | null | undefined;
    if (native === null || native === undefined) {
      return null;
    }
    return {
      maxDeviationBps: native["max_deviation_bps"] as number,
    };
  }

  // ── Reporting Writes ────────────────────────────────────────────────────────

  /**
   * Builds an invocation of `report` authorized and paid for by `reporter`.
   */
  async buildReportTx(
    reporter: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
  ): Promise<Transaction> {
    return this.buildWriteTx(reporter, "report", [
      nativeToScVal(reporter, { type: "address" }),
      assetToScVal(asset),
      nativeToScVal(price, { type: "i128" }),
      nativeToScVal(BigInt(timestamp), { type: "u64" }),
    ]);
  }

  /**
   * Submits a price report for `asset`, authorized by `reporter`.
   */
  async report(
    reporter: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
  ): Promise<TxResult> {
    return this.submit(await this.buildReportTx(reporter, asset, price, timestamp), reporter);
  }

  /**
   * Builds a sponsored invocation of `report` paid for by `feeSource` and authorized by `reporter`.
   */
  async buildSponsoredReportTx(
    reporter: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
    { feeSource }: { feeSource: string },
  ): Promise<SponsoredTransaction> {
    return this.buildSponsoredWriteTx(feeSource, "report", [
      nativeToScVal(reporter, { type: "address" }),
      assetToScVal(asset),
      nativeToScVal(price, { type: "i128" }),
      nativeToScVal(BigInt(timestamp), { type: "u64" }),
    ]);
  }

  /**
   * Builds an admin price override invocation bypassing deviation limits.
   */
  async buildOverridePriceTx(
    admin: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "override_price", [
      assetToScVal(asset),
      nativeToScVal(price, { type: "i128" }),
      nativeToScVal(BigInt(timestamp), { type: "u64" }),
    ]);
  }

  /**
   * Submits an admin price override for `asset`.
   */
  async overridePrice(
    admin: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
  ): Promise<TxResult> {
    return this.submit(await this.buildOverridePriceTx(admin, asset, price, timestamp), admin);
  }

  /**
   * Builds a sponsored price override paid for by `feeSource` and authorized by `admin`.
   */
  async buildSponsoredOverridePriceTx(
    admin: string,
    asset: OracleAsset,
    price: bigint,
    timestamp: bigint | number,
    { feeSource }: { feeSource: string },
  ): Promise<SponsoredTransaction> {
    return this.buildSponsoredWriteTx(feeSource, "override_price", [
      assetToScVal(asset),
      nativeToScVal(price, { type: "i128" }),
      nativeToScVal(BigInt(timestamp), { type: "u64" }),
    ]);
  }

  // ── Admin Writes ────────────────────────────────────────────────────────────

  /**
   * Builds an invocation of `add_asset` authorized by `admin`.
   */
  async buildAddAssetTx(
    admin: string,
    asset: OracleAsset,
    maxDeviationBps: number,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "add_asset", [
      assetToScVal(asset),
      nativeToScVal(maxDeviationBps, { type: "u32" }),
    ]);
  }

  /**
   * Adds an asset to the oracle feed with specified max deviation threshold.
   */
  async addAsset(admin: string, asset: OracleAsset, maxDeviationBps: number): Promise<TxResult> {
    return this.submit(await this.buildAddAssetTx(admin, asset, maxDeviationBps), admin);
  }

  /**
   * Builds an invocation of `set_max_deviation` authorized by `admin`.
   */
  async buildSetMaxDeviationTx(
    admin: string,
    asset: OracleAsset,
    maxDeviationBps: number,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_max_deviation", [
      assetToScVal(asset),
      nativeToScVal(maxDeviationBps, { type: "u32" }),
    ]);
  }

  /**
   * Updates max deviation threshold for `asset`.
   */
  async setMaxDeviation(
    admin: string,
    asset: OracleAsset,
    maxDeviationBps: number,
  ): Promise<TxResult> {
    return this.submit(await this.buildSetMaxDeviationTx(admin, asset, maxDeviationBps), admin);
  }

  /**
   * Builds an invocation of `set_reporter` authorized by `admin`.
   */
  async buildSetReporterTx(
    admin: string,
    asset: OracleAsset,
    reporter: string,
    allowed: boolean,
  ): Promise<Transaction> {
    return this.buildWriteTx(admin, "set_reporter", [
      assetToScVal(asset),
      nativeToScVal(reporter, { type: "address" }),
      nativeToScVal(allowed),
    ]);
  }

  /**
   * Grants or revokes permission for `reporter` to report prices for `asset`.
   */
  async setReporter(
    admin: string,
    asset: OracleAsset,
    reporter: string,
    allowed: boolean,
  ): Promise<TxResult> {
    return this.submit(await this.buildSetReporterTx(admin, asset, reporter, allowed), admin);
  }
}
