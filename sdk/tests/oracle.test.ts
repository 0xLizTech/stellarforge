import { describe, it, expect, vi, afterEach } from "vitest";
import { Account, nativeToScVal, rpc, xdr } from "@stellar/stellar-sdk";

import {
  OracleAdapterClient,
  OracleErrorCode,
  assetToScVal,
  scValToAsset,
  tickOf,
  checkDeviation,
  isFresh,
  toDecimalString,
} from "../src/oracle.js";
import type { OracleAsset, PriceData } from "../src/oracle.js";
import { TESTNET_CONFIG } from "../src/types.js";
import type { StellarForgeConfig } from "../src/types.js";
import {
  ORACLE_ID,
  RWA_ID,
  HOLDER as REPORTER,
  SIGNER as ADMIN,
  SIGNER_SECRET as ADMIN_SECRET,
  OTHER_KP,
  SUBMITTED_HASH,
  configWith,
  stubSimulation,
  stubWritePath,
  succeeds,
  fails,
  succeedsWithoutResult,
  invocation,
  builtInvocation,
  addr,
  bool,
  i128,
  str,
  struct,
  none,
  u32,
  u64,
} from "./helpers.js";

const oracleConfig: StellarForgeConfig = configWith({ oracleAdapter: ORACLE_ID });
const signerConfig: StellarForgeConfig = configWith({ oracleAdapter: ORACLE_ID }, ADMIN_SECRET);

const STELLAR_ASSET: OracleAsset = { type: "stellar", address: RWA_ID };
const OTHER_ASSET: OracleAsset = { type: "other", symbol: "USD" };

function priceDataScVal(price: bigint, timestamp: bigint): xdr.ScVal {
  return struct({
    price: i128(price),
    timestamp: u64(timestamp),
  });
}

function assetConfigScVal(maxDeviationBps: number): xdr.ScVal {
  return struct({
    max_deviation_bps: u32(maxDeviationBps),
  });
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("OracleAdapterClient constructor", () => {
  it("rejects a config with no oracleAdapter address", () => {
    expect(() => new OracleAdapterClient({ ...TESTNET_CONFIG, contracts: {} })).toThrow(
      "contracts.oracleAdapter address is required",
    );
  });

  it("accepts a valid contract id", () => {
    expect(() => new OracleAdapterClient(oracleConfig)).not.toThrow();
  });
});

describe("Asset ScVal codecs", () => {
  it("encodes and decodes stellar asset", () => {
    const scVal = assetToScVal(STELLAR_ASSET);
    const decoded = scValToAsset(scVal);
    expect(decoded).toEqual(STELLAR_ASSET);
  });

  it("encodes and decodes other asset", () => {
    const scVal = assetToScVal(OTHER_ASSET);
    const decoded = scValToAsset(scVal);
    expect(decoded).toEqual(OTHER_ASSET);
  });

  it("throws when decoding invalid ScVal", () => {
    expect(() => scValToAsset(xdr.ScVal.scvVoid())).toThrow("Expected tuple enum variant for Asset");
    expect(() => scValToAsset(xdr.ScVal.scvVec([nativeToScVal("Unknown", { type: "symbol" })]))).toThrow(
      "Expected tuple enum variant for Asset",
    );
    expect(() =>
      scValToAsset(
        xdr.ScVal.scvVec([
          nativeToScVal("Unknown", { type: "symbol" }),
          nativeToScVal("Value", { type: "string" }),
        ]),
      ),
    ).toThrow("Unrecognized Asset variant");
  });
});

describe("Oracle pure math helpers", () => {
  describe("tickOf", () => {
    it("rounds down timestamps to the nearest resolution multiple", () => {
      expect(tickOf(1700000045n, 60)).toBe(1700000040n);
      expect(tickOf(1700000060n, 60)).toBe(1700000040n);
      expect(tickOf(1700000000n, 3600)).toBe(1699999200n);
      expect(tickOf(120n, 60)).toBe(120n);
      expect(tickOf(125n, 60)).toBe(120n);
    });

    it("rejects non-positive resolution", () => {
      expect(() => tickOf(100n, 0)).toThrow(RangeError);
      expect(() => tickOf(100n, -10)).toThrow(RangeError);
    });
  });

  describe("checkDeviation", () => {
    it("accepts exact equality", () => {
      expect(checkDeviation(1000n, 1000n, 100)).toBe(true);
    });

    it("accepts moves within allowed basis points", () => {
      // 10% limit = 1000 bps
      expect(checkDeviation(10_000n, 11_000n, 1000)).toBe(true);
      expect(checkDeviation(10_000n, 9_000n, 1000)).toBe(true);
      expect(checkDeviation(10_000n, 10_500n, 1000)).toBe(true);
    });

    it("rejects moves exceeding basis points", () => {
      expect(checkDeviation(10_000n, 11_001n, 1000)).toBe(false);
      expect(checkDeviation(10_000n, 8_999n, 1000)).toBe(false);
    });

    it("handles large integers without precision loss", () => {
      const prev = 100_000_000_000_000_000_000n;
      const next = 105_000_000_000_000_000_000n; // 5% increase = 500 bps
      expect(checkDeviation(prev, next, 500)).toBe(true);
      expect(checkDeviation(prev, next, 499)).toBe(false);
    });

    it("rejects non-positive prices and negative deviation", () => {
      expect(() => checkDeviation(0n, 100n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(100n, 0n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(-5n, 100n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(100n, 100n, -1)).toThrow(RangeError);
    });
  });

  describe("isFresh", () => {
    it("returns true when record is within max age", () => {
      const price: PriceData = { price: 100n, timestamp: 1000n };
      expect(isFresh(price, 60, 1050)).toBe(true);
      expect(isFresh(price, 60, 1060)).toBe(true);
    });

    it("returns false when record exceeds max age", () => {
      const price: PriceData = { price: 100n, timestamp: 1000n };
      expect(isFresh(price, 60, 1061)).toBe(false);
    });

    it("returns false when record timestamp is in the future", () => {
      const price: PriceData = { price: 100n, timestamp: 1000n };
      expect(isFresh(price, 60, 999)).toBe(false);
    });
  });

  describe("toDecimalString", () => {
    it("formats integer prices with decimals", () => {
      expect(toDecimalString(10500000n, 7)).toBe("1.05");
      expect(toDecimalString(10000000n, 7)).toBe("1");
      expect(toDecimalString(1234567n, 7)).toBe("0.1234567");
      expect(toDecimalString(5n, 7)).toBe("0.0000005");
      expect(toDecimalString(0n, 7)).toBe("0");
    });

    it("formats zero decimals", () => {
      expect(toDecimalString(500n, 0)).toBe("500");
    });

    it("formats negative prices", () => {
      expect(toDecimalString(-10500000n, 7)).toBe("-1.05");
    });

    it("rejects invalid decimals", () => {
      expect(() => toDecimalString(100n, -1)).toThrow(RangeError);
      expect(() => toDecimalString(100n, 1.5)).toThrow(RangeError);
    });
  });
});

describe("OracleAdapterClient read-only queries", () => {
  const client = new OracleAdapterClient(oracleConfig);

  it("base() queries base asset", async () => {
    const spy = stubSimulation(succeeds(assetToScVal(OTHER_ASSET)));
    const result = await client.base();
    expect(result).toEqual(OTHER_ASSET);
    expect(invocation(spy)).toEqual({ fn: "base", args: [] });
  });

  it("assets() returns list of assets", async () => {
    const rawList = xdr.ScVal.scvVec([
      assetToScVal(STELLAR_ASSET),
      assetToScVal(OTHER_ASSET),
    ]);
    const spy = stubSimulation(succeeds(rawList));
    const result = await client.assets();
    expect(result).toEqual([STELLAR_ASSET, OTHER_ASSET]);
    expect(invocation(spy)).toEqual({ fn: "assets", args: [] });
  });

  it("decimals() returns contract decimals", async () => {
    const spy = stubSimulation(succeeds(u32(14)));
    const result = await client.decimals();
    expect(result).toBe(14);
    expect(invocation(spy)).toEqual({ fn: "decimals", args: [] });
  });

  it("resolution() returns contract resolution in seconds", async () => {
    const spy = stubSimulation(succeeds(u32(3600)));
    const result = await client.resolution();
    expect(result).toBe(3600);
    expect(invocation(spy)).toEqual({ fn: "resolution", args: [] });
  });

  it("price() returns price record or null if not found", async () => {
    const spyFound = stubSimulation(succeeds(priceDataScVal(250_000_000n, 1700000000n)));
    const found = await client.price(STELLAR_ASSET, 1700000000n);
    expect(found).toEqual({ price: 250_000_000n, timestamp: 1700000000n });
    expect(invocation(spyFound).fn).toBe("price");

    stubSimulation(succeeds(none()));
    const notFound = await client.price(STELLAR_ASSET, 1700000000n);
    expect(notFound).toBeNull();
  });

  it("prices() returns historical price records or null if empty", async () => {
    const rawPrices = xdr.ScVal.scvVec([
      priceDataScVal(260_000_000n, 1700003600n),
      priceDataScVal(250_000_000n, 1700000000n),
    ]);
    const spyFound = stubSimulation(succeeds(rawPrices));
    const found = await client.prices(OTHER_ASSET, 2);
    expect(found).toEqual([
      { price: 260_000_000n, timestamp: 1700003600n },
      { price: 250_000_000n, timestamp: 1700000000n },
    ]);
    expect(invocation(spyFound).fn).toBe("prices");

    stubSimulation(succeeds(none()));
    const empty = await client.prices(OTHER_ASSET, 5);
    expect(empty).toBeNull();
  });

  it("lastprice() returns most recent price record or null", async () => {
    stubSimulation(succeeds(priceDataScVal(300_000_000n, 1700010000n)));
    const found = await client.lastprice(STELLAR_ASSET);
    expect(found).toEqual({ price: 300_000_000n, timestamp: 1700010000n });

    stubSimulation(succeeds(none()));
    const notFound = await client.lastprice(STELLAR_ASSET);
    expect(notFound).toBeNull();
  });

  it("admin() queries admin address", async () => {
    const spy = stubSimulation(succeeds(addr(ADMIN)));
    const result = await client.admin();
    expect(result).toBe(ADMIN);
    expect(invocation(spy)).toEqual({ fn: "admin", args: [] });
  });

  it("isReporter() checks authorization status", async () => {
    const spy = stubSimulation(succeeds(bool(true)));
    const result = await client.isReporter(STELLAR_ASSET, REPORTER);
    expect(result).toBe(true);
    expect(invocation(spy).fn).toBe("is_reporter");
  });

  it("assetConfig() returns asset config or null", async () => {
    stubSimulation(succeeds(assetConfigScVal(500)));
    const config = await client.assetConfig(STELLAR_ASSET);
    expect(config).toEqual({ maxDeviationBps: 500 });

    stubSimulation(succeeds(none()));
    const notFound = await client.assetConfig(OTHER_ASSET);
    expect(notFound).toBeNull();
  });
});

describe("OracleAdapterClient write operations", () => {
  const client = new OracleAdapterClient(signerConfig);

  it("report() builds and submits report transaction", async () => {
    stubWritePath();
    const result = await client.report(ADMIN, STELLAR_ASSET, 100_000_000n, 1700000000n);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildReportTx(ADMIN, STELLAR_ASSET, 100_000_000n, 1700000000n);
    const built = builtInvocation(tx);
    expect(built.fn).toBe("report");
    expect(built.args[0]).toBe(ADMIN);
    expect(built.args[2]).toBe(100_000_000n);
    expect(built.args[3]).toBe(1700000000n);
  });

  it("buildSponsoredReportTx() builds sponsored report transaction", async () => {
    const proto = rpc.Server.prototype as unknown as {
      getAccount: (addr: string) => Promise<unknown>;
      simulateTransaction: () => Promise<unknown>;
    };
    vi.spyOn(proto, "getAccount").mockImplementation(async (address: string) => new Account(address, "7"));
    vi.spyOn(proto, "simulateTransaction").mockResolvedValue({
      latestLedger: 10,
      result: { auth: [] },
    });

    const sponsored = await client.buildSponsoredReportTx(
      REPORTER,
      STELLAR_ASSET,
      100_000_000n,
      1700000000n,
      { feeSource: ADMIN },
    );
    expect(sponsored.transaction.source).toBe(ADMIN);
    expect(builtInvocation(sponsored.transaction).fn).toBe("report");
  });

  it("overridePrice() builds and submits admin price override", async () => {
    stubWritePath();
    const result = await client.overridePrice(ADMIN, OTHER_ASSET, 200_000_000n, 1700003600n);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildOverridePriceTx(ADMIN, OTHER_ASSET, 200_000_000n, 1700003600n);
    const built = builtInvocation(tx);
    expect(built.fn).toBe("override_price");
    expect(built.args[1]).toBe(200_000_000n);
    expect(built.args[2]).toBe(1700003600n);
  });

  it("addAsset() builds and submits add_asset transaction", async () => {
    stubWritePath();
    const result = await client.addAsset(ADMIN, STELLAR_ASSET, 1000);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildAddAssetTx(ADMIN, STELLAR_ASSET, 1000);
    const built = builtInvocation(tx);
    expect(built.fn).toBe("add_asset");
    expect(built.args[1]).toBe(1000);
  });

  it("setMaxDeviation() builds and submits set_max_deviation transaction", async () => {
    stubWritePath();
    const result = await client.setMaxDeviation(ADMIN, STELLAR_ASSET, 1500);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildSetMaxDeviationTx(ADMIN, STELLAR_ASSET, 1500);
    const built = builtInvocation(tx);
    expect(built.fn).toBe("set_max_deviation");
    expect(built.args[1]).toBe(1500);
  });

  it("setReporter() builds and submits set_reporter transaction", async () => {
    stubWritePath();
    const result = await client.setReporter(ADMIN, STELLAR_ASSET, REPORTER, true);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildSetReporterTx(ADMIN, STELLAR_ASSET, REPORTER, true);
    const built = builtInvocation(tx);
    expect(built.fn).toBe("set_reporter");
    expect(built.args[1]).toBe(REPORTER);
    expect(built.args[2]).toBe(true);
  });

  it("buildTransferAdminTx() transfers admin role with dual auth", async () => {
    const proto = rpc.Server.prototype as unknown as {
      getAccount: (addr: string) => Promise<unknown>;
      simulateTransaction: () => Promise<unknown>;
    };
    vi.spyOn(proto, "getAccount").mockImplementation(async (address: string) => new Account(address, "7"));
    vi.spyOn(proto, "simulateTransaction").mockResolvedValue({
      latestLedger: 10,
      result: { auth: [] },
    });

    const sponsored = await client.buildTransferAdminTx(ADMIN, REPORTER);
    expect(sponsored.transaction.source).toBe(ADMIN);
    expect(builtInvocation(sponsored.transaction).fn).toBe("transfer_admin");
  });
});

describe("OracleErrorCode enum", () => {
  it("has exact discriminants matching contract", () => {
    expect(OracleErrorCode.NotInitialized).toBe(1);
    expect(OracleErrorCode.InvalidDecimals).toBe(2);
    expect(OracleErrorCode.InvalidResolution).toBe(3);
    expect(OracleErrorCode.UnknownAsset).toBe(4);
    expect(OracleErrorCode.AssetAlreadyExists).toBe(5);
    expect(OracleErrorCode.TooManyAssets).toBe(6);
    expect(OracleErrorCode.NotReporter).toBe(7);
    expect(OracleErrorCode.InvalidPrice).toBe(8);
    expect(OracleErrorCode.TimestampInFuture).toBe(9);
    expect(OracleErrorCode.StaleReport).toBe(10);
    expect(OracleErrorCode.DeviationTooLarge).toBe(11);
    expect(OracleErrorCode.InvalidDeviation).toBe(12);
    expect(OracleErrorCode.Overflow).toBe(13);
  });
});


describe("Oracle asset encoding", () => {
  it("round-trips stellar and other assets", () => {
    const stellar = { type: "stellar" as const, address: RWA_ID };
    const other = { type: "other" as const, symbol: "USD" };
    expect(scValToAsset(assetToScVal(stellar))).toEqual(stellar);
    expect(scValToAsset(assetToScVal(other))).toEqual(other);
  });
});
