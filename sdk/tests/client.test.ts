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
  Keypair,
  Account,
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

// ─── Write path ───────────────────────────────────────────────────────────────

/**
 * A deterministic signer. The write path requires the signing keypair to be the
 * authorizing address, so the public key and the secret have to come from the
 * same seed rather than being independent fixtures.
 */
const ISSUER_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 7));
const ISSUER = ISSUER_KP.publicKey();
const ISSUER_SECRET = ISSUER_KP.secret();

const OTHER_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 8));

type GetAccountFn = (address: string) => Promise<Account>;
type PrepareFn = (tx: Transaction) => Promise<Transaction>;
type SendFn = (tx: Transaction) => Promise<rpc.Api.SendTransactionResponse>;
type PollFn = (hash: string) => Promise<rpc.Api.GetTransactionResponse>;

interface WriteStubs {
  getAccount: MockInstance<GetAccountFn>;
  prepare: MockInstance<PrepareFn>;
  send: MockInstance<SendFn>;
  poll: MockInstance<PollFn>;
}

const SUBMITTED_HASH = "a".repeat(64);

/**
 * Stubs the four network calls a write makes. `prepareTransaction` hands back
 * the transaction it was given, so the real builder output stays under test and
 * `signAndSubmit` has something genuine to sign.
 */
function stubWritePath(overrides: Partial<{ send: unknown; poll: unknown }> = {}): WriteStubs {
  const proto = rpc.Server.prototype as unknown as {
    getAccount: GetAccountFn;
    prepareTransaction: PrepareFn;
    sendTransaction: SendFn;
    pollTransaction: PollFn;
  };

  const getAccount = vi.spyOn(proto, "getAccount");
  getAccount.mockImplementation(async (address: string) => new Account(address, "7"));

  const prepare = vi.spyOn(proto, "prepareTransaction");
  prepare.mockImplementation(async (tx: Transaction) => tx);

  const send = vi.spyOn(proto, "sendTransaction");
  send.mockResolvedValue(
    (overrides.send ?? {
      status: "PENDING",
      hash: SUBMITTED_HASH,
      latestLedger: 10,
      latestLedgerCloseTime: 0,
    }) as rpc.Api.SendTransactionResponse,
  );

  const poll = vi.spyOn(proto, "pollTransaction");
  poll.mockResolvedValue(
    (overrides.poll ?? {
      status: rpc.Api.GetTransactionStatus.SUCCESS,
      txHash: SUBMITTED_HASH,
      ledger: 42,
    }) as rpc.Api.GetTransactionResponse,
  );

  return { getAccount, prepare, send, poll };
}

/** The contract function and decoded arguments of a built transaction. */
function builtInvocation(tx: Transaction): { fn: string; args: unknown[] } {
  const op = tx.operations[0] as { func: xdr.HostFunction };
  const invoked = op.func.invokeContract();
  return {
    fn: invoked.functionName().toString(),
    args: invoked.args().map((arg) => scValToNative(arg)),
  };
}

describe("RwaAssetClient write builders", () => {
  // Each contract entry point calls require_auth on exactly one address, and
  // under same-address auth that address must also source the transaction.
  // Sourcing from the wrong one produces a transaction that fails on-chain
  // after the fee is spent.
  it.each([
    {
      name: "mint",
      fn: "mint",
      authorizer: ISSUER,
      build: (c: RwaAssetClient) => c.buildMintTx(ISSUER, HOLDER, 500n),
      args: [ISSUER, HOLDER, 500n],
    },
    {
      name: "burn",
      fn: "burn",
      authorizer: HOLDER,
      build: (c: RwaAssetClient) => c.buildBurnTx(HOLDER, 25n),
      args: [HOLDER, 25n],
    },
    {
      name: "transfer",
      fn: "transfer",
      authorizer: HOLDER,
      build: (c: RwaAssetClient) => c.buildTransferTx(HOLDER, SPENDER, 10n),
      args: [HOLDER, SPENDER, 10n],
    },
    {
      name: "approve",
      fn: "approve",
      authorizer: HOLDER,
      build: (c: RwaAssetClient) => c.buildApproveTx(HOLDER, SPENDER, 99n),
      args: [HOLDER, SPENDER, 99n],
    },
    {
      name: "transferFrom",
      fn: "transfer_from",
      authorizer: SPENDER,
      build: (c: RwaAssetClient) => c.buildTransferFromTx(SPENDER, HOLDER, ISSUER, 7n),
      args: [SPENDER, HOLDER, ISSUER, 7n],
    },
  ])("$name invokes $fn with the documented argument order", async ({ fn, args, build }) => {
    stubWritePath();
    const tx = await build(new RwaAssetClient(rwaConfig));
    expect(builtInvocation(tx)).toEqual({ fn, args });
  });

  it.each([
    { name: "mint", authorizer: ISSUER, build: (c: RwaAssetClient) => c.buildMintTx(ISSUER, HOLDER, 1n) },
    { name: "burn", authorizer: HOLDER, build: (c: RwaAssetClient) => c.buildBurnTx(HOLDER, 1n) },
    { name: "transfer", authorizer: HOLDER, build: (c: RwaAssetClient) => c.buildTransferTx(HOLDER, SPENDER, 1n) },
    { name: "approve", authorizer: HOLDER, build: (c: RwaAssetClient) => c.buildApproveTx(HOLDER, SPENDER, 1n) },
    {
      name: "transferFrom",
      authorizer: SPENDER,
      build: (c: RwaAssetClient) => c.buildTransferFromTx(SPENDER, HOLDER, ISSUER, 1n),
    },
  ])("$name sources the transaction from the authorizing address", async ({ authorizer, build }) => {
    const stubs = stubWritePath();
    const tx = await build(new RwaAssetClient(rwaConfig));
    expect(tx.source).toBe(authorizer);
    expect(stubs.getAccount).toHaveBeenCalledWith(authorizer);
  });

  // The whole point of splitting build from submit: a browser integration can
  // reach a signable transaction without the library ever holding a secret.
  it("builds without a signerSecret and leaves the transaction unsigned", async () => {
    stubWritePath();
    const tx = await new RwaAssetClient(rwaConfig).buildTransferTx(HOLDER, SPENDER, 1n);
    expect(rwaConfig.signerSecret).toBeUndefined();
    expect(tx.signatures).toHaveLength(0);
  });

  it("prepares the transaction before returning it", async () => {
    const stubs = stubWritePath();
    await new RwaAssetClient(rwaConfig).buildMintTx(ISSUER, HOLDER, 1n);
    expect(stubs.prepare).toHaveBeenCalledOnce();
  });

  // prepareTransaction simulates, so a contract-rejected call fails here rather
  // than costing a fee on-chain.
  it("surfaces a rejection raised while preparing", async () => {
    const stubs = stubWritePath();
    stubs.prepare.mockRejectedValue(new Error("HostError: Error(Contract, #8)"));
    await expect(
      new RwaAssetClient(rwaConfig).buildTransferTx(HOLDER, SPENDER, 1n),
    ).rejects.toThrow("HostError: Error(Contract, #8)");
  });
});

describe("RwaAssetClient write submission", () => {
  const signerConfig: StellarForgeConfig = { ...rwaConfig, signerSecret: ISSUER_SECRET };

  it("signs, submits and reports the settled transaction", async () => {
    const stubs = stubWritePath();
    const result = await new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 500n);

    expect(result).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });
    expect(stubs.send).toHaveBeenCalledOnce();

    const submitted = stubs.send.mock.calls[0]?.[0] as Transaction;
    expect(submitted.signatures.length).toBeGreaterThan(0);
  });

  it("refuses to submit without a signerSecret", async () => {
    stubWritePath();
    await expect(new RwaAssetClient(rwaConfig).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /signerSecret is required/,
    );
  });

  // Same-address auth means a mismatched signer produces a transaction that
  // burns a fee and then fails require_auth. Refuse before spending it.
  it("refuses when the signer is not the authorizing address", async () => {
    stubWritePath();
    const wrongSigner: StellarForgeConfig = { ...rwaConfig, signerSecret: OTHER_KP.secret() };
    await expect(new RwaAssetClient(wrongSigner).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /must be authorized by/,
    );
  });

  it("does not submit when the signer is rejected", async () => {
    const stubs = stubWritePath();
    const wrongSigner: StellarForgeConfig = { ...rwaConfig, signerSecret: OTHER_KP.secret() };
    await expect(new RwaAssetClient(wrongSigner).mint(ISSUER, HOLDER, 1n)).rejects.toThrow();
    expect(stubs.send).not.toHaveBeenCalled();
  });

  it("throws when submission is rejected outright", async () => {
    stubWritePath({
      send: { status: "ERROR", hash: SUBMITTED_HASH, latestLedger: 10, latestLedgerCloseTime: 0 },
    });
    await expect(new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /rejected on submission/,
    );
  });

  // A transaction can be accepted for inclusion and still fail when applied.
  // Returning a hash here would read as success.
  it("throws when the transaction settles as FAILED", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.FAILED,
        txHash: SUBMITTED_HASH,
        ledger: 42,
      },
    });
    await expect(new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /did not succeed: FAILED/,
    );
  });

  it("throws when the transaction never appears", async () => {
    stubWritePath({
      poll: { status: rpc.Api.GetTransactionStatus.NOT_FOUND, txHash: SUBMITTED_HASH },
    });
    await expect(new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /did not succeed: NOT_FOUND/,
    );
  });

  it.each([
    { name: "burn", call: (c: RwaAssetClient) => c.burn(ISSUER, 1n) },
    { name: "transfer", call: (c: RwaAssetClient) => c.transfer(ISSUER, HOLDER, 1n) },
    { name: "approve", call: (c: RwaAssetClient) => c.approve(ISSUER, SPENDER, 1n) },
    {
      name: "transferFrom",
      call: (c: RwaAssetClient) => c.transferFrom(ISSUER, HOLDER, SPENDER, 1n),
    },
  ])("$name submits and reports the settled transaction", async ({ call }) => {
    stubWritePath();
    await expect(call(new RwaAssetClient(signerConfig))).resolves.toEqual({
      hash: SUBMITTED_HASH,
      ledger: 42,
    });
  });
});
