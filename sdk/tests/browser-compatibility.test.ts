// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import {
  ComplianceClient,
  DEFAULT_AUTH_VALIDITY_LEDGERS,
  GovernanceClient,
  OracleAdapterClient,
  RegistryClient,
  RwaAssetClient,
  TESTNET_CONFIG,
  TransactionExpiredError,
  TransactionOutcomeUnknownError,
  VaultClient,
  checkDeviation,
  fromStroops,
  hexToBytes32,
  isFresh,
  isValidContractId,
  isValidStellarAddress,
  maxRedeemable,
  sha256Hex,
  sharesForDeposit,
  tickOf,
  toDecimalString,
  toStroops,
  truncateAddress,
  underlyingForRedeem,
} from "../src/index.js";

const dummyContractId = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM";

describe("Browser Compatibility Suite (#184)", () => {
  it("runs in a real browser-like environment with window and document", () => {
    expect(typeof window).toBe("object");
    expect(typeof document).toBe("object");
    expect(typeof navigator).toBe("object");
    expect(window.crypto).toBeDefined();
    expect(window.crypto.subtle).toBeDefined();
  });

  it("does not silently polyfill Node-only modules into browser globals", () => {
    // Neither fs, child_process, nor net should be leaked as globals
    const g = globalThis as Record<string, unknown>;
    expect(g.fs).toBeUndefined();
    expect(g.child_process).toBeUndefined();
    expect(g.net).toBeUndefined();
    expect(g.tls).toBeUndefined();
  });

  it("instantiates all protocol clients in browser environment", () => {
    const rwa = new RwaAssetClient({
      ...TESTNET_CONFIG,
      contracts: { rwaAsset: dummyContractId },
    });
    expect(rwa).toBeInstanceOf(RwaAssetClient);

    const compliance = new ComplianceClient({
      ...TESTNET_CONFIG,
      contracts: { compliance: dummyContractId },
    });
    expect(compliance).toBeInstanceOf(ComplianceClient);

    const gov = new GovernanceClient({
      ...TESTNET_CONFIG,
      contracts: { governance: dummyContractId },
    });
    expect(gov).toBeInstanceOf(GovernanceClient);

    const registry = new RegistryClient({
      ...TESTNET_CONFIG,
      contracts: { registry: dummyContractId },
    });
    expect(registry).toBeInstanceOf(RegistryClient);

    const vault = new VaultClient({
      ...TESTNET_CONFIG,
      contracts: { vault: dummyContractId, rwaAsset: dummyContractId },
    });
    expect(vault).toBeInstanceOf(VaultClient);

    const oracle = new OracleAdapterClient({
      ...TESTNET_CONFIG,
      contracts: { oracleAdapter: dummyContractId },
    });
    expect(oracle).toBeInstanceOf(OracleAdapterClient);
  });

  it("executes all pure conversion and math utilities in browser runtime", () => {
    // Stroop conversions
    expect(toStroops("100.5", 7)).toBe(1005000000n);
    expect(fromStroops(1005000000n, 7)).toBe("100.5");

    // Vault pure calculations: sharesForDeposit is assets * exchangeRate
    const shares = sharesForDeposit(1000n, 10_000_000n);
    expect(shares).toBe(10_000_000_000n);

    const underlying = underlyingForRedeem(10_000_000_000n, 10_000_000n);
    expect(underlying).toBe(1000n);

    const redeemable = maxRedeemable(1000n, 100, 150);
    expect(redeemable).toBe(1000n);

    // Oracle pure calculations
    const tick = tickOf(1700000025n, 300);
    expect(tick).toBe(1700000025n - (1700000025n % 300n));

    const deviationOk = checkDeviation(100n, 102n, 300);
    expect(deviationOk).toBe(true);

    const fresh = isFresh({ price: 1000n, timestamp: 1000n }, 100, 1050);
    expect(fresh).toBe(true);

    const decimalStr = toDecimalString(12345678n, 6);
    expect(decimalStr).toBe("12.345678");

    // Address utilities
    const addr = "GBZXN7PIRZGNMHGA7MUUUF4GWPY5AYPV6LY4UV2GL6VJGIQRXFDNMADI";
    expect(truncateAddress(addr)).toBe("GBZXN...NMADI");
    expect(truncateAddress(addr, 4)).toBe("GBZX...MADI");
    expect(isValidStellarAddress(addr)).toBe(true);
    expect(isValidContractId(dummyContractId)).toBe(true);

    // Hex and hash utilities
    const hash = sha256Hex("stellarforge");
    expect(typeof hash).toBe("string");
    expect(hash.length).toBe(64);

    const bytes = hexToBytes32(hash);
    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBe(32);
  });

  it("supports browser wallet transaction preparation without secrets", async () => {
    const client = new RwaAssetClient({
      ...TESTNET_CONFIG,
      contracts: { rwaAsset: dummyContractId },
    });

    // Client provides build*Tx methods designed for browser wallets (Freighter, etc.)
    expect(typeof client.buildTransferTx).toBe("function");
    expect(typeof client.buildApproveTx).toBe("function");
    expect(typeof client.buildProposeAdminTx).toBe("function");
    expect(typeof client.buildAcceptAdminTx).toBe("function");
  });

  it("handles errors and invalid inputs cleanly without silent polyfills", () => {
    expect(() => toStroops("invalid_decimal", 7)).toThrow();
    expect(() => toStroops("1.12345678", 7)).toThrow(); // Exceeds decimals

    const expiredErr = new TransactionExpiredError("tx-123");
    expect(expiredErr.name).toBe("TransactionExpiredError");
    expect(expiredErr.hash).toBe("tx-123");

    const unknownErr = new TransactionOutcomeUnknownError("tx-456");
    expect(unknownErr.name).toBe("TransactionOutcomeUnknownError");
    expect(unknownErr.hash).toBe("tx-456");

    expect(DEFAULT_AUTH_VALIDITY_LEDGERS).toBeGreaterThan(0);
  });
});
