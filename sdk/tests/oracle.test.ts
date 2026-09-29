import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, it, expect, vi, afterEach } from "vitest";
import {
  Account,
  Address,
  Keypair,
  Networks,
  nativeToScVal,
  rpc,
  SorobanDataBuilder,
  xdr,
} from "@stellar/stellar-sdk";

import {
  OracleAdapterClient,
  OracleErrorCode,
  OracleInvalidDeviationError,
  OracleOverflowError,
  assetToScVal,
  scValToAsset,
  tickOf,
  checkDeviation,
  isFresh,
  toDecimalString,
} from "../src/oracle.js";
import type { OracleAsset, PriceData } from "../src/oracle.js";
import {
  DEFAULT_AUTH_VALIDITY_LEDGERS,
  authorizeEntries,
} from "../src/sponsored.js";
import { TESTNET_CONFIG } from "../src/types.js";
import type { StellarForgeConfig } from "../src/types.js";
import {
  ORACLE_ID,
  RWA_ID,
  HOLDER as REPORTER_ADDR,
  SIGNER as ADMIN,
  SIGNER_SECRET as ADMIN_SECRET,
  SIGNER_KP as ADMIN_KP,
  SUBMITTED_HASH,
  configWith,
  stubSimulation,
  stubWritePath,
  succeeds,
  fails,
  invocation,
  builtInvocation,
  addr,
  bool,
  i128,
  struct,
  none,
  u32,
  u64,
} from "./helpers.js";

const oracleConfig: StellarForgeConfig = configWith({ oracleAdapter: ORACLE_ID });
const signerConfig: StellarForgeConfig = configWith({ oracleAdapter: ORACLE_ID }, ADMIN_SECRET);

const STELLAR_ASSET: OracleAsset = { type: "stellar", address: RWA_ID };
const OTHER_ASSET: OracleAsset = { type: "other", symbol: "USD" };

/** Reporter keypair used for sponsored auth tests. */
const REPORTER_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 31));
const REPORTER = REPORTER_KP.publicKey();
const NEW_ADMIN_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 32));
const LATEST_LEDGER = 1_000;

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

/** Decoded Asset wire form as `builtInvocation` / `invocation` report it. */
function assetArgs(asset: OracleAsset): unknown {
  return asset.type === "stellar" ? ["Stellar", asset.address] : ["Other", asset.symbol];
}

function unsignedEntry(
  address: string,
  fn: string,
  args: xdr.ScVal[],
  nonce = 7n,
): xdr.SorobanAuthorizationEntry {
  return new xdr.SorobanAuthorizationEntry({
    credentials: xdr.SorobanCredentials.sorobanCredentialsAddressV2(
      new xdr.SorobanAddressCredentials({
        address: new Address(address).toScAddress(),
        nonce,
        signatureExpirationLedger: 0,
        signature: xdr.ScVal.scvVoid(),
      }),
    ),
    rootInvocation: new xdr.SorobanAuthorizedInvocation({
      function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(
        new xdr.InvokeContractArgs({
          contractAddress: new Address(ORACLE_ID).toScAddress(),
          functionName: fn,
          args,
        }),
      ),
      subInvocations: [],
    }),
  });
}

function sourceAccountEntry(fn: string, args: xdr.ScVal[]): xdr.SorobanAuthorizationEntry {
  const base = unsignedEntry(ADMIN, fn, args);
  return new xdr.SorobanAuthorizationEntry({
    credentials: xdr.SorobanCredentials.sorobanCredentialsSourceAccount(),
    rootInvocation: base.rootInvocation,
  });
}

function simulationWith(auth: xdr.SorobanAuthorizationEntry[]): rpc.Api.SimulateTransactionResponse {
  return {
    _parsed: true,
    id: "1",
    latestLedger: LATEST_LEDGER,
    minResourceFee: "5000",
    transactionData: new SorobanDataBuilder(),
    events: [],
    result: { retval: xdr.ScVal.scvVoid(), auth },
  } as unknown as rpc.Api.SimulateTransactionResponse;
}

function stubSponsoredNetwork(simulations: rpc.Api.SimulateTransactionResponse[]) {
  const proto = rpc.Server.prototype;
  const getAccount = vi.spyOn(proto, "getAccount").mockImplementation(async (address: string) => {
    return new Account(address, "7");
  });
  const simulate = vi.spyOn(proto, "simulateTransaction");
  for (const simulation of simulations) simulate.mockResolvedValueOnce(simulation);
  const latestLedger = vi
    .spyOn(proto, "getLatestLedger")
    .mockResolvedValue({ id: "x", sequence: LATEST_LEDGER, protocolVersion: "28" } as never);
  return { getAccount, simulate, latestLedger };
}

const sign = (
  entries: xdr.SorobanAuthorizationEntry[],
  signer: Keypair,
  ledgers = DEFAULT_AUTH_VALIDITY_LEDGERS,
) => authorizeEntries(entries, signer, LATEST_LEDGER + ledgers, Networks.TESTNET);

interface ParityTable {
  tickOf: Array<{ timestamp: string; resolution: number; expected: string }>;
  checkDeviation: Array<{
    previous: string;
    next: string;
    maxDeviationBps: number;
    expected: boolean | "Overflow";
  }>;
}

const parityTable = JSON.parse(
  readFileSync(join(dirname(fileURLToPath(import.meta.url)), "fixtures", "oracle-parity-table.json"), "utf8"),
) as ParityTable;

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

  it("round-trips stellar and other assets", () => {
    expect(scValToAsset(assetToScVal(STELLAR_ASSET))).toEqual(STELLAR_ASSET);
    expect(scValToAsset(assetToScVal(OTHER_ASSET))).toEqual(OTHER_ASSET);
  });
});

describe("Oracle pure math helpers", () => {
  describe("contract parity table", () => {
    it(`has at least 50 cases (tickOf=${parityTable.tickOf.length}, checkDeviation=${parityTable.checkDeviation.length})`, () => {
      expect(parityTable.tickOf.length + parityTable.checkDeviation.length).toBeGreaterThanOrEqual(50);
    });

    it("tickOf matches every committed contract-generated case", () => {
      for (const row of parityTable.tickOf) {
        expect(tickOf(BigInt(row.timestamp), row.resolution)).toBe(BigInt(row.expected));
      }
    });

    it("checkDeviation matches every committed contract-generated case", () => {
      for (const row of parityTable.checkDeviation) {
        const prev = BigInt(row.previous);
        const next = BigInt(row.next);
        if (row.expected === "Overflow") {
          expect(() => checkDeviation(prev, next, row.maxDeviationBps)).toThrow(OracleOverflowError);
        } else {
          expect(checkDeviation(prev, next, row.maxDeviationBps)).toBe(row.expected);
        }
      }
    });
  });

  describe("tickOf", () => {
    it("rejects non-positive resolution", () => {
      expect(() => tickOf(100n, 0)).toThrow(RangeError);
      expect(() => tickOf(100n, -10)).toThrow(RangeError);
    });
  });

  describe("checkDeviation", () => {
    it("rejects maxDeviationBps of 0 as InvalidDeviation", () => {
      expect(() => checkDeviation(1000n, 1000n, 0)).toThrow(OracleInvalidDeviationError);
      try {
        checkDeviation(1000n, 1000n, 0);
      } catch (err) {
        expect(err).toBeInstanceOf(OracleInvalidDeviationError);
        expect((err as OracleInvalidDeviationError).code).toBe(OracleErrorCode.InvalidDeviation);
      }
    });

    it("throws OracleOverflowError past i128 bounds", () => {
      expect(() => checkDeviation(1n, (1n << 127n) - 1n, 1)).toThrow(OracleOverflowError);
      try {
        checkDeviation(1n, (1n << 127n) - 1n, 1);
      } catch (err) {
        expect(err).toBeInstanceOf(OracleOverflowError);
        expect((err as OracleOverflowError).code).toBe(OracleErrorCode.Overflow);
      }
    });

    it("rejects non-positive prices and non-integer deviation", () => {
      expect(() => checkDeviation(0n, 100n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(100n, 0n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(-5n, 100n, 100)).toThrow(RangeError);
      expect(() => checkDeviation(100n, 100n, -1)).toThrow(RangeError);
      expect(() => checkDeviation(100n, 100n, 1.5)).toThrow(RangeError);
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

    it("rejects negative or non-integer maxAgeSecs and nowSecs", () => {
      const price: PriceData = { price: 100n, timestamp: 1000n };
      expect(() => isFresh(price, -1, 1000)).toThrow(RangeError);
      expect(() => isFresh(price, 60, -1)).toThrow(RangeError);
      expect(() => isFresh(price, 1.5, 1000)).toThrow(RangeError);
      expect(() => isFresh(price, 60, 1.5)).toThrow(RangeError);
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

  it("assets() returns list of assets via scValToAsset", async () => {
    const rawList = xdr.ScVal.scvVec([assetToScVal(STELLAR_ASSET), assetToScVal(OTHER_ASSET)]);
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

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("price() asserts full ScVal args for %s asset", async (_label, asset) => {
    const spyFound = stubSimulation(succeeds(priceDataScVal(250_000_000n, 1700000000n)));
    const found = await client.price(asset, 1700000000n);
    expect(found).toEqual({ price: 250_000_000n, timestamp: 1700000000n });
    expect(invocation(spyFound)).toEqual({
      fn: "price",
      args: [assetArgs(asset), 1700000000n],
    });

    stubSimulation(succeeds(none()));
    const notFound = await client.price(asset, 1700000000n);
    expect(notFound).toBeNull();
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("prices() asserts full ScVal args for %s asset", async (_label, asset) => {
    const rawPrices = xdr.ScVal.scvVec([
      priceDataScVal(260_000_000n, 1700003600n),
      priceDataScVal(250_000_000n, 1700000000n),
    ]);
    const spyFound = stubSimulation(succeeds(rawPrices));
    const found = await client.prices(asset, 2);
    expect(found).toEqual([
      { price: 260_000_000n, timestamp: 1700003600n },
      { price: 250_000_000n, timestamp: 1700000000n },
    ]);
    expect(invocation(spyFound)).toEqual({
      fn: "prices",
      args: [assetArgs(asset), 2],
    });

    stubSimulation(succeeds(none()));
    const empty = await client.prices(asset, 5);
    expect(empty).toBeNull();
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("lastprice() asserts full ScVal args for %s asset", async (_label, asset) => {
    const spy = stubSimulation(succeeds(priceDataScVal(300_000_000n, 1700010000n)));
    const found = await client.lastprice(asset);
    expect(found).toEqual({ price: 300_000_000n, timestamp: 1700010000n });
    expect(invocation(spy)).toEqual({
      fn: "lastprice",
      args: [assetArgs(asset)],
    });

    stubSimulation(succeeds(none()));
    const notFound = await client.lastprice(asset);
    expect(notFound).toBeNull();
  });

  it("admin() queries admin address", async () => {
    const spy = stubSimulation(succeeds(addr(ADMIN)));
    const result = await client.admin();
    expect(result).toBe(ADMIN);
    expect(invocation(spy)).toEqual({ fn: "admin", args: [] });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("isReporter() asserts full ScVal args for %s asset", async (_label, asset) => {
    const spy = stubSimulation(succeeds(bool(true)));
    const result = await client.isReporter(asset, REPORTER_ADDR);
    expect(result).toBe(true);
    expect(invocation(spy)).toEqual({
      fn: "is_reporter",
      args: [assetArgs(asset), REPORTER_ADDR],
    });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("assetConfig() asserts full ScVal args for %s asset", async (_label, asset) => {
    const spy = stubSimulation(succeeds(assetConfigScVal(500)));
    const config = await client.assetConfig(asset);
    expect(config).toEqual({ maxDeviationBps: 500 });
    expect(invocation(spy)).toEqual({
      fn: "asset_config",
      args: [assetArgs(asset)],
    });

    stubSimulation(succeeds(none()));
    const notFound = await client.assetConfig(asset);
    expect(notFound).toBeNull();
  });

  it("surfaces a simulation error on a read (UnknownAsset)", async () => {
    stubSimulation(fails("HostError: Error(Contract, #4)"));
    await expect(client.lastprice(STELLAR_ASSET)).rejects.toThrow(/HostError: Error\(Contract, #4\)/);
  });
});

describe("OracleAdapterClient write operations", () => {
  const client = new OracleAdapterClient(signerConfig);

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("report() asserts full args for %s asset", async (_label, asset) => {
    stubWritePath();
    const result = await client.report(ADMIN, asset, 100_000_000n, 1700000000n);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildReportTx(ADMIN, asset, 100_000_000n, 1700000000n);
    expect(builtInvocation(tx)).toEqual({
      fn: "report",
      args: [ADMIN, assetArgs(asset), 100_000_000n, 1700000000n],
    });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("overridePrice() asserts full args for %s asset", async (_label, asset) => {
    stubWritePath();
    const result = await client.overridePrice(ADMIN, asset, 200_000_000n, 1700003600n);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildOverridePriceTx(ADMIN, asset, 200_000_000n, 1700003600n);
    expect(builtInvocation(tx)).toEqual({
      fn: "override_price",
      args: [assetArgs(asset), 200_000_000n, 1700003600n],
    });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("addAsset() asserts full args for %s asset", async (_label, asset) => {
    stubWritePath();
    const result = await client.addAsset(ADMIN, asset, 1000);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildAddAssetTx(ADMIN, asset, 1000);
    expect(builtInvocation(tx)).toEqual({
      fn: "add_asset",
      args: [assetArgs(asset), 1000],
    });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("setMaxDeviation() asserts full args for %s asset", async (_label, asset) => {
    stubWritePath();
    const result = await client.setMaxDeviation(ADMIN, asset, 1500);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildSetMaxDeviationTx(ADMIN, asset, 1500);
    expect(builtInvocation(tx)).toEqual({
      fn: "set_max_deviation",
      args: [assetArgs(asset), 1500],
    });
  });

  it.each([
    ["stellar", STELLAR_ASSET],
    ["other", OTHER_ASSET],
  ] as const)("setReporter() asserts full args for %s asset", async (_label, asset) => {
    stubWritePath();
    const result = await client.setReporter(ADMIN, asset, REPORTER_ADDR, true);
    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

    const tx = await client.buildSetReporterTx(ADMIN, asset, REPORTER_ADDR, true);
    expect(builtInvocation(tx)).toEqual({
      fn: "set_reporter",
      args: [assetArgs(asset), REPORTER_ADDR, true],
    });
  });
});

describe("OracleAdapterClient sponsored writes", () => {
  const client = new OracleAdapterClient(signerConfig);

  it("buildSponsoredReportTx signs with authorizeEntries and finalizes", async () => {
    const reportArgs = [
      nativeToScVal(REPORTER, { type: "address" }),
      assetToScVal(STELLAR_ASSET),
      nativeToScVal(100_000_000n, { type: "i128" }),
      nativeToScVal(1700000000n, { type: "u64" }),
    ];
    stubSponsoredNetwork([
      simulationWith([unsignedEntry(REPORTER, "report", reportArgs)]),
      simulationWith([]),
    ]);

    const sponsored = await client.buildSponsoredReportTx(
      REPORTER,
      STELLAR_ASSET,
      100_000_000n,
      1700000000n,
      { feeSource: ADMIN },
    );

    expect(sponsored.transaction.source).toBe(ADMIN);
    expect(sponsored.authorizers).toEqual([REPORTER]);
    expect(builtInvocation(sponsored.transaction)).toEqual({
      fn: "report",
      args: [REPORTER, assetArgs(STELLAR_ASSET), 100_000_000n, 1700000000n],
    });

    const signed = await sign(sponsored.authEntries, REPORTER_KP);
    const tx = await client.finalizeSponsoredTx(sponsored, signed);
    expect(tx.source).toBe(ADMIN);
    const { auth } = tx.operations[0] as unknown as { auth: xdr.SorobanAuthorizationEntry[] };
    expect(auth).toHaveLength(1);
    expect(auth[0]?.equals(signed[0] as xdr.SorobanAuthorizationEntry)).toBe(true);
  });

  it("buildSponsoredOverridePriceTx signs with authorizeEntries and finalizes", async () => {
    const overrideArgs = [
      assetToScVal(OTHER_ASSET),
      nativeToScVal(200_000_000n, { type: "i128" }),
      nativeToScVal(1700003600n, { type: "u64" }),
    ];
    stubSponsoredNetwork([
      simulationWith([unsignedEntry(ADMIN, "override_price", overrideArgs)]),
      simulationWith([]),
    ]);

    // Fee source is a separate relayer; admin is the authorizer.
    const relayerClient = new OracleAdapterClient(oracleConfig);
    const sponsored = await relayerClient.buildSponsoredOverridePriceTx(
      ADMIN,
      OTHER_ASSET,
      200_000_000n,
      1700003600n,
      { feeSource: REPORTER },
    );

    expect(sponsored.transaction.source).toBe(REPORTER);
    expect(sponsored.authorizers).toEqual([ADMIN]);
    expect(builtInvocation(sponsored.transaction)).toEqual({
      fn: "override_price",
      args: [assetArgs(OTHER_ASSET), 200_000_000n, 1700003600n],
    });

    const signed = await sign(sponsored.authEntries, ADMIN_KP);
    // finalize needs a client that can re-simulate; signer secret not required until submit
    const tx = await relayerClient.finalizeSponsoredTx(sponsored, signed);
    expect(tx.source).toBe(REPORTER);
    const { auth } = tx.operations[0] as unknown as { auth: xdr.SorobanAuthorizationEntry[] };
    expect(auth).toHaveLength(1);
    expect(auth[0]?.equals(signed[0] as xdr.SorobanAuthorizationEntry)).toBe(true);
  });

  it("buildTransferAdminTx lists both authorizers for dual auth", async () => {
    const newAdmin = NEW_ADMIN_KP.publicKey();
    const args = [nativeToScVal(newAdmin, { type: "address" })];
    stubSponsoredNetwork([
      simulationWith([
        sourceAccountEntry("transfer_admin", args),
        unsignedEntry(newAdmin, "transfer_admin", args),
      ]),
    ]);

    const sponsored = await client.buildTransferAdminTx(ADMIN, newAdmin);
    expect(sponsored.transaction.source).toBe(ADMIN);
    expect(builtInvocation(sponsored.transaction)).toEqual({
      fn: "transfer_admin",
      args: [newAdmin],
    });
    // Current admin is covered by the fee-payer source signature (null);
    // incoming admin must sign its own entry.
    expect(sponsored.authorizers).toEqual([null, newAdmin]);
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
