import { describe, it, expect, vi } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Keypair } from "@stellar/stellar-sdk";
import { beforeEach, describe, it, expect, vi } from "vitest";
import { Keypair, rpc } from "@stellar/stellar-sdk";
import * as sdk from "@stellarforge-protocol/sdk";
import type { OracleAdapterClient, OracleAsset } from "@stellarforge-protocol/sdk";
import {
  canonicalJsonStringify,
  parseExactPrice,
  submitSignedAppraisal,
} from "../src/index.js";
import { ExitCode, type SignedAppraisalFile } from "../src/types.js";
import { runCli } from "../src/cli.js";

describe("NAV reporter CLI JSON output", () => {
  it("keeps required-argument errors as one parseable JSON stdout line", async () => {
    const stdout = vi.spyOn(console, "log").mockImplementation(() => {});
    const stderr = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const code = await runCli(["--json"]);
      expect(code).toBe(ExitCode.PreflightFailure);
      expect(stdout).toHaveBeenCalledTimes(1);
      expect(JSON.parse(String(stdout.mock.calls[0]?.[0]))).toMatchObject({
        success: false,
        exitCode: ExitCode.PreflightFailure,
      });
      expect(stderr).not.toHaveBeenCalled();
    } finally {
      stdout.mockRestore();
      stderr.mockRestore();
    }
  });

  it("keeps malformed-file errors parseable and sends no diagnostics to stdout", async () => {
    const dir = mkdtempSync(join(tmpdir(), "nav-reporter-cli-"));
    const appraisal = join(dir, "appraisal.json");
    writeFileSync(appraisal, "not-json");
    const stdout = vi.spyOn(console, "log").mockImplementation(() => {});
    const stderr = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const code = await runCli(["--json", "--appraisal", appraisal, "--oracle", "C..."]);
      expect(code).toBe(ExitCode.PreflightFailure);
      expect(JSON.parse(String(stdout.mock.calls[0]?.[0]))).toMatchObject({
        success: false,
        exitCode: ExitCode.PreflightFailure,
        error: expect.stringContaining("Malformed appraisal JSON"),
      });
      expect(stderr).not.toHaveBeenCalled();
    } finally {
      stdout.mockRestore();
      stderr.mockRestore();
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("returns help as JSON when --json is present", async () => {
    const stdout = vi.spyOn(console, "log").mockImplementation(() => {});
    try {
      expect(await runCli(["--help", "--json"])).toBe(ExitCode.Success);
      expect(JSON.parse(String(stdout.mock.calls[0]?.[0]))).toMatchObject({
        success: true,
        exitCode: ExitCode.Success,
        help: expect.stringContaining("--appraisal"),
      });
    } finally {
      stdout.mockRestore();
    }
  });
});

beforeEach(() => {
  vi.spyOn(rpc.Server.prototype, "getLatestLedger").mockResolvedValue({ sequence: 1234 } as any);
  vi.spyOn(sdk, "authorizeEntries").mockImplementation(async (entries) => [...entries]);
});

function createMockSignedAppraisal(
  appraiserKeypair: Keypair,
  overrides?: Partial<SignedAppraisalFile["payload"]>
): SignedAppraisalFile {
  const payload = {
    asset: { type: "other", symbol: "REALESTATE" } as OracleAsset,
    price: "1234.56",
    observedAt: new Date(Date.now() - 60_000).toISOString(),
    appraiser: "Certified Institutional Appraiser",
    documentSha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ...overrides,
  };

  const canonicalBytes = Buffer.from(canonicalJsonStringify(payload), "utf8");
  const signature = Buffer.from(appraiserKeypair.sign(canonicalBytes)).toString("hex");

  return {
    payload,
    signature,
    publicKey: appraiserKeypair.publicKey(),
  };
}

describe("parseExactPrice", () => {
  it("converts exact decimal strings without float precision loss", () => {
    expect(parseExactPrice("123.45", 2)).toBe(12345n);
    expect(parseExactPrice("123.45", 7)).toBe(1234500000n);
    expect(parseExactPrice("0.0000001", 7)).toBe(1n);
    expect(parseExactPrice("100", 7)).toBe(1000000000n);
  });

  it("refuses lossy conversions exceeding decimals", () => {
    expect(() => parseExactPrice("123.456", 2)).toThrow("Lossy price conversion");
    expect(() => parseExactPrice("0.00000001", 7)).toThrow("Lossy price conversion");
  });
});

describe("submitSignedAppraisal", () => {
  const appraiser = Keypair.random();
  const reporter = Keypair.random();
  const relayer = Keypair.random();

  function createMockOracleClient(overrides?: Partial<Record<string, any>>) {
    return {
      decimals: vi.fn().mockResolvedValue(7),
      resolution: vi.fn().mockResolvedValue(86400n),
      isReporter: vi.fn().mockResolvedValue(true),
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        timestamp: BigInt(Math.floor(Date.now() / 1000) - 100_000),
      }),
      assetConfig: vi.fn().mockResolvedValue({
        maxDeviationBps: 500, // 5%
      }),
      buildSponsoredReportTx: vi.fn().mockResolvedValue({ authEntries: [] } as any),
      authorizeEntries: vi.fn().mockReturnValue({} as any),
      finalizeSponsoredTx: vi.fn().mockReturnValue({
        sign: vi.fn(),
      } as any),
      submitSponsoredTx: vi.fn().mockResolvedValue({
        hash: "tx_mock_hash_123456",
        ledger: 1234,
      }),
      price: vi.fn().mockResolvedValue({
        price: 1234_5600000n,
        timestamp: 1790769600n,
      }),
      ...overrides,
    } as unknown as OracleAdapterClient;
  }

  it("succeeds on valid signed appraisal", async () => {
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Date.now() - 60000).toISOString(),
      price: "1020.00", // 2% move from 1000, within 5%
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        timestamp: BigInt(Math.floor(Date.now() / 1000) - 100000), // earlier tick
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
      relayerKeypair: relayer,
      expectedAppraiserPublicKey: appraiser.publicKey(),
    });

    expect(res.success).toBe(true);
    expect(res.exitCode).toBe(ExitCode.Success);
    expect(res.action).toBe("submitted");
    expect(res.txHash).toBe("tx_mock_hash_123456");
  });

  it("refuses tampered payload with authentication failure", async () => {
    const signed = createMockSignedAppraisal(appraiser);
    signed.payload.price = "999999.99"; // tampered price!

    const mockClient = createMockOracleClient();
    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
      expectedAppraiserPublicKey: appraiser.publicKey(),
    });

    expect(res.success).toBe(false);
    expect(res.exitCode).toBe(ExitCode.AuthFailure);
    expect(res.error).toContain("Authentication failed");
  });

  it("refuses unauthorized reporter", async () => {
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Date.now() - 60000).toISOString(),
    });

    const mockClient = createMockOracleClient({
      isReporter: vi.fn().mockResolvedValue(false),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(false);
    expect(res.exitCode).toBe(ExitCode.PreflightFailure);
    expect(res.error).toContain("not authorized");
  });

  it("refuses future timestamp", async () => {
    const futureDate = new Date(Date.now() + 1000000).toISOString();
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: futureDate,
    });

    const mockClient = createMockOracleClient();
    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(false);
    expect(res.exitCode).toBe(ExitCode.PreflightFailure);
    expect(res.error).toContain("in the future");
  });

  it("refuses deviation exceeding limit", async () => {
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Date.now() - 60000).toISOString(),
      price: "1200.00", // 20% increase from 1000, limit is 5%
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        // Keep this in the preceding daily tick at any time of day.
        timestamp: BigInt(Math.floor(Date.now() / 1000) - 200_000),
      }),
      assetConfig: vi.fn().mockResolvedValue({
        maxDeviationBps: 500, // 5%
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(false);
    expect(res.exitCode).toBe(ExitCode.PreflightFailure);
    expect(res.error).toContain("Deviation limit exceeded");
  });

  it("handles dry-run mode without submitting", async () => {
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Date.now() - 60000).toISOString(),
      price: "1010.00",
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        timestamp: BigInt(Math.floor(Date.now() / 1000) - 100000),
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
      dryRun: true,
    });

    expect(res.success).toBe(true);
    expect(res.exitCode).toBe(ExitCode.Success);
    expect(res.action).toBe("dry_run");
    expect(mockClient.submitSponsoredTx).not.toHaveBeenCalled();
  });

  it("treats identical duplicate report at same tick as idempotent success", async () => {
    const nowSecs = BigInt(Math.floor(Date.now() / 1000) - 1000);
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Number(nowSecs) * 1000).toISOString(),
      price: "1000.00",
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        timestamp: nowSecs,
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(true);
    expect(res.exitCode).toBe(ExitCode.Success);
    expect(res.action).toBe("idempotent_duplicate");
  });

  it("alerts on conflicting report at same tick", async () => {
    const nowSecs = BigInt(Math.floor(Date.now() / 1000) - 1000);
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Number(nowSecs) * 1000).toISOString(),
      price: "1020.00",
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n, // Different price already at this tick!
        timestamp: nowSecs,
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(false);
    expect(res.exitCode).toBe(ExitCode.PreflightFailure);
    expect(res.error).toContain("Conflicting price report");
  });

  it("recovers from unknown outcome if price is recorded on-chain", async () => {
    const nowSecs = BigInt(Math.floor(Date.now() / 1000) - 100000);
    const signed = createMockSignedAppraisal(appraiser, {
      observedAt: new Date(Date.now() - 5000).toISOString(),
      price: "1010.00",
    });

    const mockClient = createMockOracleClient({
      lastprice: vi.fn().mockResolvedValue({
        price: 1000_0000000n,
        timestamp: nowSecs,
      }),
      submitSponsoredTx: vi.fn().mockRejectedValue({
        name: "TransactionOutcomeUnknownError",
      }),
      price: vi.fn().mockResolvedValue({
        price: 1010_0000000n, // Price made it on chain despite timeout!
      }),
    });

    const res = await submitSignedAppraisal(signed, {
      oracleClient: mockClient,
      reporterKeypair: reporter,
    });

    expect(res.success).toBe(true);
    expect(res.exitCode).toBe(ExitCode.Success);
    expect(res.details?.recoveredFromUnknownOutcome).toBe(true);
  });
});
