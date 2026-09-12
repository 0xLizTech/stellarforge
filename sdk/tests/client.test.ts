import { describe, it, expect, vi, afterEach } from "vitest";
import type { MockInstance } from "vitest";
import {
  rpc,
  nativeToScVal,
  scValToNative,
  xdr,
  StrKey,
  Transaction,
  Address,
} from "@stellar/stellar-sdk";

import { RwaAssetClient, ComplianceClient } from "../src/client.js";
import { TESTNET_CONFIG } from "../src/types.js";
import type { StellarForgeConfig } from "../src/types.js";

// ─── Fixtures ─────────────────────────────────────────────────────────────────

/** Deterministic C-addresses, so failures name the same contract every run. */
const RWA_ID = StrKey.encodeContract(Buffer.alloc(32, 1));
const COMPLIANCE_ID = StrKey.encodeContract(Buffer.alloc(32, 2));

const HOLDER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 3));
const SPENDER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 4));

/** 32 bytes of 0xab, so the expected hex is unmistakable. */
const DOC_HASH_BYTES = Buffer.alloc(32, 0xab);
const DOC_HASH_HEX = "ab".repeat(32);

const rwaConfig: StellarForgeConfig = {
  ...TESTNET_CONFIG,
  contracts: { rwaAsset: RWA_ID },
};

const complianceConfig: StellarForgeConfig = {
  ...TESTNET_CONFIG,
  contracts: { compliance: COMPLIANCE_ID },
};

// ─── Simulation stubbing ──────────────────────────────────────────────────────

// The clients build a real `rpc.Server`, so the seam is the prototype method
// rather than the module. That keeps `Contract`, `TransactionBuilder` and the
// ScVal codecs real: only the network call is replaced.
type SimulateFn = (tx: Transaction) => Promise<rpc.Api.SimulateTransactionResponse>;
type Spy = MockInstance<SimulateFn>;

function stubSimulation(response: unknown): Spy {
  const spy = vi.spyOn(
    rpc.Server.prototype as unknown as { simulateTransaction: SimulateFn },
    "simulateTransaction",
  );
  spy.mockResolvedValue(response as rpc.Api.SimulateTransactionResponse);
  return spy;
}

/** A successful simulation carrying `retval`. */
function succeeds(retval: xdr.ScVal): unknown {
  return { latestLedger: 3, result: { retval, auth: [] } };
}

/**
 * A failed simulation. `rpc.Api.isSimulationError` discriminates purely on the
 * presence of an `error` key, so that is what makes this a failure.
 */
function fails(message: string): unknown {
  return { latestLedger: 3, error: message };
}

/**
 * A simulation that is not an error yet carries no result. The RPC returns this
 * shape for a restore-required preflight, among others.
 */
function succeedsWithoutResult(): unknown {
  return { latestLedger: 3 };
}

/** The contract function and decoded arguments of the simulated transaction. */
function invocation(spy: Spy): { fn: string; args: unknown[] } {
  const tx = spy.mock.calls[0]?.[0] as Transaction;
  const op = tx.operations[0] as { func: xdr.HostFunction };
  const invoked = op.func.invokeContract();
  return {
    fn: invoked.functionName().toString(),
    args: invoked.args().map((arg) => scValToNative(arg)),
  };
}

const i128 = (v: bigint): xdr.ScVal => nativeToScVal(v, { type: "i128" });
const bool = (v: boolean): xdr.ScVal => nativeToScVal(v);

/** Builds the ScMap a Soroban struct decodes from. Keys must be sorted. */
function struct(fields: Record<string, xdr.ScVal>): xdr.ScVal {
  const entries = Object.keys(fields)
    .sort()
    .map(
      (key) =>
        new xdr.ScMapEntry({
          key: nativeToScVal(key, { type: "symbol" }),
          val: fields[key] as xdr.ScVal,
        }),
    );
  return xdr.ScVal.scvMap(entries);
}

function metadataScVal(overrides: Record<string, xdr.ScVal> = {}): xdr.ScVal {
  return struct({
    name: nativeToScVal("Manhattan REIT", { type: "string" }),
    symbol: nativeToScVal("REIT-NYC-001", { type: "string" }),
    decimals: nativeToScVal(7, { type: "u32" }),
    asset_class: nativeToScVal("real_estate", { type: "string" }),
    legal_doc_hash: nativeToScVal(DOC_HASH_BYTES, { type: "bytes" }),
    max_supply: i128(1_000_000n),
    ...overrides,
  });
}

afterEach(() => {
  vi.restoreAllMocks();
});

// ─── RwaAssetClient ───────────────────────────────────────────────────────────

describe("RwaAssetClient constructor", () => {
  it("rejects a config with no rwaAsset address", () => {
    expect(() => new RwaAssetClient({ ...TESTNET_CONFIG, contracts: {} })).toThrow(
      "contracts.rwaAsset address is required",
    );
  });

  it("rejects an empty rwaAsset address", () => {
    expect(
      () => new RwaAssetClient({ ...TESTNET_CONFIG, contracts: { rwaAsset: "" } }),
    ).toThrow("contracts.rwaAsset address is required");
  });

  it("accepts a valid contract id", () => {
    expect(() => new RwaAssetClient(rwaConfig)).not.toThrow();
  });
});

describe("RwaAssetClient.balance", () => {
  it("decodes an i128 to bigint", async () => {
    stubSimulation(succeeds(i128(42_000_000n)));
    await expect(new RwaAssetClient(rwaConfig).balance(HOLDER)).resolves.toBe(42_000_000n);
  });

  it("decodes a zero balance", async () => {
    stubSimulation(succeeds(i128(0n)));
    await expect(new RwaAssetClient(rwaConfig).balance(HOLDER)).resolves.toBe(0n);
  });

  // The contract stores balances as i128. Decoding through `number` would lose
  // precision silently, so pick a value no double can represent.
  it("preserves precision beyond Number.MAX_SAFE_INTEGER", async () => {
    const huge = 170141183460469231731687303715884105727n; // i128 max
    stubSimulation(succeeds(i128(huge)));
    const result = await new RwaAssetClient(rwaConfig).balance(HOLDER);
    expect(result).toBe(huge);
    expect(typeof result).toBe("bigint");
  });

  it("invokes `balance` with the owner address", async () => {
    const spy = stubSimulation(succeeds(i128(1n)));
    await new RwaAssetClient(rwaConfig).balance(HOLDER);
    expect(invocation(spy)).toEqual({ fn: "balance", args: [HOLDER] });
  });
});

describe("RwaAssetClient.totalSupply", () => {
  it("decodes an i128 to bigint", async () => {
    stubSimulation(succeeds(i128(999n)));
    await expect(new RwaAssetClient(rwaConfig).totalSupply()).resolves.toBe(999n);
  });

  it("invokes `total_supply` with no arguments", async () => {
    const spy = stubSimulation(succeeds(i128(0n)));
    await new RwaAssetClient(rwaConfig).totalSupply();
    expect(invocation(spy)).toEqual({ fn: "total_supply", args: [] });
  });
});

describe("RwaAssetClient.allowance", () => {
  it("decodes an i128 to bigint", async () => {
    stubSimulation(succeeds(i128(250n)));
    await expect(
      new RwaAssetClient(rwaConfig).allowance(HOLDER, SPENDER),
    ).resolves.toBe(250n);
  });

  // Owner and spender are the same type, so a swapped pair would return another
  // account's allowance rather than fail. Pin the order.
  it("invokes `allowance` with owner before spender", async () => {
    const spy = stubSimulation(succeeds(i128(1n)));
    await new RwaAssetClient(rwaConfig).allowance(HOLDER, SPENDER);
    expect(invocation(spy)).toEqual({ fn: "allowance", args: [HOLDER, SPENDER] });
  });
});

describe("RwaAssetClient.isPaused", () => {
  it("decodes true", async () => {
    stubSimulation(succeeds(bool(true)));
    await expect(new RwaAssetClient(rwaConfig).isPaused()).resolves.toBe(true);
  });

  it("decodes false", async () => {
    stubSimulation(succeeds(bool(false)));
    await expect(new RwaAssetClient(rwaConfig).isPaused()).resolves.toBe(false);
  });

  it("invokes `paused`", async () => {
    const spy = stubSimulation(succeeds(bool(false)));
    await new RwaAssetClient(rwaConfig).isPaused();
    expect(invocation(spy)).toEqual({ fn: "paused", args: [] });
  });
});

describe("RwaAssetClient.isIssuer", () => {
  it("decodes true", async () => {
    stubSimulation(succeeds(bool(true)));
    await expect(new RwaAssetClient(rwaConfig).isIssuer(HOLDER)).resolves.toBe(true);
  });

  it("decodes false", async () => {
    stubSimulation(succeeds(bool(false)));
    await expect(new RwaAssetClient(rwaConfig).isIssuer(HOLDER)).resolves.toBe(false);
  });

  it("invokes `is_issuer` with the candidate address", async () => {
    const spy = stubSimulation(succeeds(bool(true)));
    await new RwaAssetClient(rwaConfig).isIssuer(HOLDER);
    expect(invocation(spy)).toEqual({ fn: "is_issuer", args: [HOLDER] });
  });
});

describe("RwaAssetClient.metadata", () => {
  it("maps every field off the contract struct", async () => {
    stubSimulation(succeeds(metadataScVal()));
    await expect(new RwaAssetClient(rwaConfig).metadata()).resolves.toEqual({
      name: "Manhattan REIT",
      symbol: "REIT-NYC-001",
      decimals: 7,
      assetClass: "real_estate",
      legalDocHash: DOC_HASH_HEX,
      maxSupply: 1_000_000n,
    });
  });

  // The contract emits snake_case; the SDK surface is camelCase. A rename that
  // drifts would surface here as undefined rather than as a type error.
  it("renames asset_class to assetClass", async () => {
    stubSimulation(
      succeeds(
        metadataScVal({
          asset_class: nativeToScVal("infrastructure", { type: "string" }),
        }),
      ),
    );
    const meta = await new RwaAssetClient(rwaConfig).metadata();
    expect(meta.assetClass).toBe("infrastructure");
    expect(meta).not.toHaveProperty("asset_class");
  });

  it("renders legal_doc_hash as lowercase hex, not raw bytes", async () => {
    stubSimulation(succeeds(metadataScVal()));
    const meta = await new RwaAssetClient(rwaConfig).metadata();
    expect(meta.legalDocHash).toBe(DOC_HASH_HEX);
    expect(meta.legalDocHash).toMatch(/^[0-9a-f]{64}$/);
  });

  it("keeps max_supply as bigint, including the uncapped sentinel", async () => {
    stubSimulation(succeeds(metadataScVal({ max_supply: i128(0n) })));
    const meta = await new RwaAssetClient(rwaConfig).metadata();
    expect(meta.maxSupply).toBe(0n);
    expect(typeof meta.maxSupply).toBe("bigint");
  });

  it("invokes `metadata`", async () => {
    const spy = stubSimulation(succeeds(metadataScVal()));
    await new RwaAssetClient(rwaConfig).metadata();
    expect(invocation(spy)).toEqual({ fn: "metadata", args: [] });
  });
});

describe("RwaAssetClient simulation failures", () => {
  it("surfaces the simulation error message", async () => {
    stubSimulation(fails("HostError: Error(Contract, #8)"));
    await expect(new RwaAssetClient(rwaConfig).balance(HOLDER)).rejects.toThrow(
      "Simulation error: HostError: Error(Contract, #8)",
    );
  });

  it("names the method when the simulation returns no result", async () => {
    stubSimulation(succeedsWithoutResult());
    await expect(new RwaAssetClient(rwaConfig).totalSupply()).rejects.toThrow(
      "No result returned from total_supply",
    );
  });

  it("fails every read path rather than returning a default", async () => {
    stubSimulation(fails("boom"));
    const client = new RwaAssetClient(rwaConfig);
    await Promise.all(
      [
        client.balance(HOLDER),
        client.totalSupply(),
        client.metadata(),
        client.isPaused(),
        client.isIssuer(HOLDER),
        client.allowance(HOLDER, SPENDER),
      ].map((p) => expect(p).rejects.toThrow("Simulation error: boom")),
    );
  });
});

// ─── ComplianceClient ─────────────────────────────────────────────────────────

describe("ComplianceClient constructor", () => {
  it("rejects a config with no compliance address", () => {
    expect(() => new ComplianceClient({ ...TESTNET_CONFIG, contracts: {} })).toThrow(
      "contracts.compliance address is required",
    );
  });

  it("accepts a valid contract id", () => {
    expect(() => new ComplianceClient(complianceConfig)).not.toThrow();
  });
});

describe("ComplianceClient.isCompliant", () => {
  it("decodes true", async () => {
    stubSimulation(succeeds(bool(true)));
    await expect(new ComplianceClient(complianceConfig).isCompliant(HOLDER, 2)).resolves.toBe(
      true,
    );
  });

  it("decodes false", async () => {
    stubSimulation(succeeds(bool(false)));
    await expect(new ComplianceClient(complianceConfig).isCompliant(HOLDER, 2)).resolves.toBe(
      false,
    );
  });

  // min_level is a u32 on the contract. Sending the wrong one silently screens
  // against the wrong threshold, so check every level the type allows.
  it.each([0, 1, 2, 3] as const)("passes minLevel %i through unchanged", async (level) => {
    const spy = stubSimulation(succeeds(bool(true)));
    await new ComplianceClient(complianceConfig).isCompliant(HOLDER, level);
    expect(invocation(spy)).toEqual({ fn: "is_compliant", args: [HOLDER, level] });
  });

  it("calls the pure query, never the TTL-extending screen", async () => {
    const spy = stubSimulation(succeeds(bool(true)));
    await new ComplianceClient(complianceConfig).isCompliant(HOLDER, 1);
    expect(invocation(spy).fn).toBe("is_compliant");
    expect(invocation(spy).fn).not.toBe("screen");
  });

  it("surfaces the simulation error message", async () => {
    stubSimulation(fails("HostError: Error(Contract, #2)"));
    await expect(
      new ComplianceClient(complianceConfig).isCompliant(HOLDER, 1),
    ).rejects.toThrow("Simulation error: HostError: Error(Contract, #2)");
  });

  it("reports a missing result rather than dereferencing it", async () => {
    stubSimulation(succeedsWithoutResult());
    await expect(
      new ComplianceClient(complianceConfig).isCompliant(HOLDER, 1),
    ).rejects.toThrow("No result returned from is_compliant");
  });
});

// ─── Contract targeting ───────────────────────────────────────────────────────

describe("contract targeting", () => {
  it("addresses each client at its own configured contract", async () => {
    const spy = stubSimulation(succeeds(bool(true)));

    const both: StellarForgeConfig = {
      ...TESTNET_CONFIG,
      contracts: { rwaAsset: RWA_ID, compliance: COMPLIANCE_ID },
    };

    await new RwaAssetClient(both).isPaused();
    await new ComplianceClient(both).isCompliant(HOLDER, 1);

    const contractOf = (call: number): string => {
      const tx = spy.mock.calls[call]?.[0] as Transaction;
      const op = tx.operations[0] as { func: xdr.HostFunction };
      return Address.fromScAddress(
        op.func.invokeContract().contractAddress(),
      ).toString();
    };

    expect(contractOf(0)).toBe(RWA_ID);
    expect(contractOf(1)).toBe(COMPLIANCE_ID);
  });
});
