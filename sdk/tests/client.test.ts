import { describe, it, expect, vi, afterEach } from "vitest";
import { Address, nativeToScVal, Networks, rpc, Transaction, xdr } from "@stellar/stellar-sdk";

import { RwaAssetClient, ComplianceClient } from "../src/client.js";
import { TransactionExpiredError, TransactionOutcomeUnknownError } from "../src/errors.js";
import { MAINNET_CONFIG, TESTNET_CONFIG } from "../src/types.js";
import type { StellarForgeConfig } from "../src/types.js";
import {
  RWA_ID,
  COMPLIANCE_ID,
  HOLDER,
  SPENDER,
  SIGNER as ISSUER,
  SIGNER_SECRET as ISSUER_SECRET,
  OTHER_KP,
  SUBMITTED_HASH,
  configWith,
  stubSimulation,
  stubWritePath,
  succeeds,
  fails,
  succeedsWithoutResult,
  invocation,
  invocationArgs,
  builtInvocation,
  i128,
  bool,
  struct,
} from "./helpers.js";

// ─── Fixtures local to this suite ─────────────────────────────────────────────

/** 32 bytes of 0xab, so the expected hex is unmistakable. */
const DOC_HASH_BYTES = Buffer.alloc(32, 0xab);
const DOC_HASH_HEX = "ab".repeat(32);

const rwaConfig: StellarForgeConfig = configWith({ rwaAsset: RWA_ID });
const complianceConfig: StellarForgeConfig = configWith({ compliance: COMPLIANCE_ID });

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
      return Address.fromScAddress(invocationArgs(tx).contractAddress).toString();
    };

    expect(contractOf(0)).toBe(RWA_ID);
    expect(contractOf(1)).toBe(COMPLIANCE_ID);
  });
});

// ─── Write path ───────────────────────────────────────────────────────────────

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
      name: "burnFrom",
      fn: "burn_from",
      authorizer: SPENDER,
      build: (c: RwaAssetClient) => c.buildBurnFromTx(SPENDER, HOLDER, 12n),
      args: [SPENDER, HOLDER, 12n],
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
      build: (c: RwaAssetClient) => c.buildApproveTx(HOLDER, SPENDER, 99n, 5_000),
      args: [HOLDER, SPENDER, 99n, 5_000],
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
    {
      name: "burnFrom",
      authorizer: SPENDER,
      build: (c: RwaAssetClient) => c.buildBurnFromTx(SPENDER, HOLDER, 1n),
    },
    { name: "transfer", authorizer: HOLDER, build: (c: RwaAssetClient) => c.buildTransferTx(HOLDER, SPENDER, 1n) },
    {
      name: "approve",
      authorizer: HOLDER,
      build: (c: RwaAssetClient) => c.buildApproveTx(HOLDER, SPENDER, 1n, 5_000),
    },
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

  // IR-05. The RPC has seen a ledger close after the transaction's maxTime and
  // still does not know the hash, so it can never land and a retry is safe.
  it("reports a transaction that expired unincluded as safe to retry", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.NOT_FOUND,
        txHash: SUBMITTED_HASH,
        latestLedger: 99,
        latestLedgerCloseTime: 9_999_999_999,
      },
    });
    const err = await new RwaAssetClient(signerConfig)
      .mint(ISSUER, HOLDER, 1n)
      .catch((e: unknown) => e);

    expect(err).toBeInstanceOf(TransactionExpiredError);
    expect((err as TransactionExpiredError).hash).toBe(SUBMITTED_HASH);
  });

  // IR-05. With no ledger yet past maxTime the transaction may still land, so
  // a caller that retried on an ordinary failure would execute it twice.
  it("reports an unsettled transaction as outcome unknown, not as failed", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.NOT_FOUND,
        txHash: SUBMITTED_HASH,
        latestLedger: 11,
        latestLedgerCloseTime: 0,
      },
    });
    const err = await new RwaAssetClient(signerConfig)
      .mint(ISSUER, HOLDER, 1n)
      .catch((e: unknown) => e);

    expect(err).toBeInstanceOf(TransactionOutcomeUnknownError);
    expect((err as TransactionOutcomeUnknownError).hash).toBe(SUBMITTED_HASH);
  });

  // IR-05. The stellar-sdk default is 30 attempts, a sixth of the 180-second
  // validity window every write is built with.
  it("polls for the transaction's whole validity window", async () => {
    const stubs = stubWritePath();
    await new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n);

    const [hash, opts] = stubs.poll.mock.calls[0] ?? [];
    expect(hash).toBe(SUBMITTED_HASH);
    expect(opts?.attempts).toBeGreaterThanOrEqual(180);
  });

  it("refuses TRY_AGAIN_LATER without polling, since nothing was queued", async () => {
    const stubs = stubWritePath({
      send: {
        status: "TRY_AGAIN_LATER",
        hash: SUBMITTED_HASH,
        latestLedger: 10,
        latestLedgerCloseTime: 0,
      },
    });
    await expect(new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n)).rejects.toThrow(
      /TRY_AGAIN_LATER/,
    );
    expect(stubs.poll).not.toHaveBeenCalled();
  });

  // DUPLICATE means this exact transaction is already queued, so the right
  // response is to wait for it, not to report a failure a caller might retry.
  it("waits for a DUPLICATE submission to settle", async () => {
    stubWritePath({
      send: { status: "DUPLICATE", hash: SUBMITTED_HASH, latestLedger: 10, latestLedgerCloseTime: 0 },
    });
    await expect(new RwaAssetClient(signerConfig).mint(ISSUER, HOLDER, 1n)).resolves.toEqual({
      hash: SUBMITTED_HASH,
      ledger: 42,
    });
  });

  it.each([
    { name: "burn", call: (c: RwaAssetClient) => c.burn(ISSUER, 1n) },
    { name: "burnFrom", call: (c: RwaAssetClient) => c.burnFrom(ISSUER, HOLDER, 1n) },
    { name: "transfer", call: (c: RwaAssetClient) => c.transfer(ISSUER, HOLDER, 1n) },
    { name: "approve", call: (c: RwaAssetClient) => c.approve(ISSUER, SPENDER, 1n, 5_000) },
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

// ─── Network presets ──────────────────────────────────────────────────────────

describe("network presets", () => {
  it("TESTNET_CONFIG names the testnet passphrase and SDF's public testnet RPC", () => {
    expect(TESTNET_CONFIG.networkPassphrase).toBe(Networks.TESTNET);
    expect(TESTNET_CONFIG.rpcUrl).toBe("https://soroban-testnet.stellar.org");
  });

  // In 0.1.0 MAINNET_CONFIG named https://soroban-rpc.stellar.org, which does
  // not resolve. There is no public mainnet RPC to default to instead.
  it("MAINNET_CONFIG names the mainnet passphrase and no RPC endpoint", () => {
    expect(MAINNET_CONFIG.networkPassphrase).toBe(Networks.PUBLIC);
    expect("rpcUrl" in MAINNET_CONFIG).toBe(false);
  });

  it("refuses a config with no rpcUrl, and says where to find one", () => {
    // @ts-expect-error rpcUrl is required, and MAINNET_CONFIG deliberately has none.
    const incomplete: StellarForgeConfig = { ...MAINNET_CONFIG, contracts: { rwaAsset: RWA_ID } };

    expect(() => new RwaAssetClient(incomplete)).toThrow(
      /config\.rpcUrl is required.*docs\/data\/apis\/rpc\/providers/,
    );
  });

  it("accepts MAINNET_CONFIG once an RPC URL is supplied", () => {
    expect(
      () =>
        new RwaAssetClient({
          ...MAINNET_CONFIG,
          rpcUrl: "https://mainnet-rpc.example.org",
          contracts: { rwaAsset: RWA_ID },
        }),
    ).not.toThrow();
  });
});
