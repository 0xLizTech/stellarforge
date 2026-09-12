/**
 * Live smoke test against a deployed contract over real Soroban RPC.
 *
 * Opt-in. Set `SMOKE_RWA_ASSET_ID` and/or `SMOKE_COMPLIANCE_ID` to a deployed
 * contract id and the matching block runs; leave them unset and it skips, which
 * is what CI does.
 *
 * This exists because `client.test.ts` stubs `simulateTransaction`. Those tests
 * pin our decode logic against fixtures, so they pass identically on any SDK
 * version no matter what changed underneath. Only this file exercises the real
 * wire format, which is what a major `@stellar/stellar-sdk` bump actually needs
 * evidence for.
 *
 * Assertions are shapes and contract invariants rather than fixed values: the
 * deployed state belongs to whoever deployed it, and a test that demanded a
 * particular balance would be useless to everyone else.
 */

import { describe, it, expect, beforeAll } from "vitest";
import { StrKey } from "@stellar/stellar-sdk";

import { RwaAssetClient, ComplianceClient } from "../src/client.js";
import { TESTNET_CONFIG } from "../src/types.js";
import { isValidContractId, isValidStellarAddress } from "../src/utils.js";
import type { StellarForgeConfig } from "../src/types.js";

// ─── Configuration ────────────────────────────────────────────────────────────

const RWA_ASSET_ID = process.env["SMOKE_RWA_ASSET_ID"];
const COMPLIANCE_ID = process.env["SMOKE_COMPLIANCE_ID"];

const RPC_URL = process.env["SOROBAN_RPC_URL"] ?? TESTNET_CONFIG.rpcUrl;
const NETWORK_PASSPHRASE =
  process.env["STELLAR_NETWORK_PASSPHRASE"] ?? TESTNET_CONFIG.networkPassphrase;

/**
 * Any valid address works: an account with no balance decodes as 0, which is
 * as good a shape check as a funded one. Supply a real holder to exercise
 * non-zero paths.
 */
const HOLDER =
  process.env["SMOKE_HOLDER_ADDRESS"] ?? StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 9));
const SPENDER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 10));

/** Well-formed but not deployed, for the error path. */
const UNDEPLOYED_ID = StrKey.encodeContract(Buffer.alloc(32, 0xfe));

/** Network round trips are slower and flakier than anything else in this suite. */
const NETWORK_TIMEOUT = 30_000;

function configFor(contracts: StellarForgeConfig["contracts"]): StellarForgeConfig {
  return {
    network: "testnet",
    rpcUrl: RPC_URL,
    networkPassphrase: NETWORK_PASSPHRASE,
    contracts,
  };
}

/**
 * Fail loudly on a malformed id rather than letting it surface as an opaque
 * SDK error three calls later.
 */
function assertUsableConfig(contractId: string, varName: string): void {
  if (!isValidContractId(contractId)) {
    throw new Error(`${varName} is not a valid contract id: ${contractId}`);
  }
  if (!isValidStellarAddress(HOLDER)) {
    throw new Error(`SMOKE_HOLDER_ADDRESS is not a valid Stellar address: ${HOLDER}`);
  }
  if (!RPC_URL.startsWith("https://")) {
    throw new Error(`SOROBAN_RPC_URL must be https (the clients set allowHttp: false): ${RPC_URL}`);
  }
}

// ─── RwaAsset ─────────────────────────────────────────────────────────────────

describe.runIf(RWA_ASSET_ID)("live RwaAssetClient", () => {
  let client: RwaAssetClient;

  beforeAll(() => {
    assertUsableConfig(RWA_ASSET_ID as string, "SMOKE_RWA_ASSET_ID");
    client = new RwaAssetClient(configFor({ rwaAsset: RWA_ASSET_ID as string }));
  });

  it(
    "returns metadata the contract would have accepted",
    async () => {
      const meta = await client.metadata();

      // validate_metadata rejects an empty name or symbol, so a deployed
      // contract cannot hold either.
      expect(meta.name.length).toBeGreaterThan(0);
      expect(meta.symbol.length).toBeGreaterThan(0);

      // MAX_DECIMALS is 18 in rwa-asset.
      expect(Number.isInteger(meta.decimals)).toBe(true);
      expect(meta.decimals).toBeGreaterThanOrEqual(0);
      expect(meta.decimals).toBeLessThanOrEqual(18);

      expect(typeof meta.assetClass).toBe("string");

      // Bytes of any length, rendered as hex, so an even-length hex string.
      expect(meta.legalDocHash).toMatch(/^[0-9a-f]*$/);
      expect(meta.legalDocHash.length % 2).toBe(0);

      expect(typeof meta.maxSupply).toBe("bigint");
      expect(meta.maxSupply).toBeGreaterThanOrEqual(0n);
    },
    NETWORK_TIMEOUT,
  );

  it(
    "returns a non-negative total supply",
    async () => {
      const supply = await client.totalSupply();
      expect(typeof supply).toBe("bigint");
      expect(supply).toBeGreaterThanOrEqual(0n);
    },
    NETWORK_TIMEOUT,
  );

  it(
    "returns a non-negative balance",
    async () => {
      const balance = await client.balance(HOLDER);
      expect(typeof balance).toBe("bigint");
      expect(balance).toBeGreaterThanOrEqual(0n);
    },
    NETWORK_TIMEOUT,
  );

  it(
    "returns a non-negative allowance",
    async () => {
      const allowance = await client.allowance(HOLDER, SPENDER);
      expect(typeof allowance).toBe("bigint");
      expect(allowance).toBeGreaterThanOrEqual(0n);
    },
    NETWORK_TIMEOUT,
  );

  it(
    "returns booleans for the flag queries",
    async () => {
      expect(typeof (await client.isPaused())).toBe("boolean");
      expect(typeof (await client.isIssuer(HOLDER))).toBe("boolean");
    },
    NETWORK_TIMEOUT,
  );

  // mint panics with ExceedsMaxSupply past the cap, so a live contract cannot
  // be over it. 0 means uncapped and is exempt.
  it(
    "holds total supply within max supply",
    async () => {
      const [meta, supply] = await Promise.all([client.metadata(), client.totalSupply()]);
      if (meta.maxSupply > 0n) {
        expect(supply).toBeLessThanOrEqual(meta.maxSupply);
      }
    },
    NETWORK_TIMEOUT,
  );

  // Balances sum to total supply, so no single holder can exceed it.
  it(
    "holds any single balance within total supply",
    async () => {
      const [balance, supply] = await Promise.all([client.balance(HOLDER), client.totalSupply()]);
      expect(balance).toBeLessThanOrEqual(supply);
    },
    NETWORK_TIMEOUT,
  );

  // The stubbed suite cannot prove the failure branch fires on a real RPC
  // response, only on a fixture shaped like one.
  //
  // Matching the "Simulation error:" prefix matters: the client adds it only
  // after a successful round trip that the host rejected. A bare toThrow()
  // would also pass on a DNS failure or a proxy 403, making this test green
  // precisely when the network is broken.
  it(
    "rejects a contract that is not deployed with a simulation error",
    async () => {
      const missing = new RwaAssetClient(configFor({ rwaAsset: UNDEPLOYED_ID }));
      await expect(missing.totalSupply()).rejects.toThrow(/Simulation error:/);
    },
    NETWORK_TIMEOUT,
  );
});

// ─── Compliance ───────────────────────────────────────────────────────────────

describe.runIf(COMPLIANCE_ID)("live ComplianceClient", () => {
  let client: ComplianceClient;

  beforeAll(() => {
    assertUsableConfig(COMPLIANCE_ID as string, "SMOKE_COMPLIANCE_ID");
    client = new ComplianceClient(configFor({ compliance: COMPLIANCE_ID as string }));
  });

  it(
    "returns a boolean at every verification level",
    async () => {
      for (const level of [0, 1, 2, 3] as const) {
        expect(typeof (await client.isCompliant(HOLDER, level))).toBe("boolean");
      }
    },
    NETWORK_TIMEOUT,
  );

  /**
   * `evaluate` is `record.level >= min_level && not_expired`, so raising the
   * required level can only ever turn a true into a false. Note it returns
   * false at level 0 for a subject with no record at all, so the sequence may
   * be false throughout.
   */
  it(
    "never becomes more compliant as the required level rises",
    async () => {
      const results: boolean[] = [];
      for (const level of [0, 1, 2, 3] as const) {
        results.push(await client.isCompliant(HOLDER, level));
      }

      for (let i = 1; i < results.length; i++) {
        if (results[i] === true) {
          expect(results[i - 1]).toBe(true);
        }
      }
    },
    NETWORK_TIMEOUT,
  );

  // As above: the prefix is what distinguishes a rejection by the host from a
  // failure to reach it at all.
  it(
    "rejects a contract that is not deployed with a simulation error",
    async () => {
      const missing = new ComplianceClient(configFor({ compliance: UNDEPLOYED_ID }));
      await expect(missing.isCompliant(HOLDER, 1)).rejects.toThrow(/Simulation error:/);
    },
    NETWORK_TIMEOUT,
  );
});
