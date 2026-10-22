import fs from "fs";
import os from "os";
import path from "path";
import { spawnSync } from "child_process";
import { afterEach, describe, expect, it } from "vitest";

// eslint-disable-next-line @typescript-eslint/no-require-imports
const wasmSize = require("../../scripts/check-wasm-size.cjs") as {
  COMMENT_MARKER: string;
  parseBudgetEntry: (
    name: string,
    raw: unknown,
  ) => {
    contract: string;
    wasm: string;
    maxBytes: number;
    baselineBytes: number | null;
  };
  formatDelta: (delta: number | null) => string;
  evaluateWasmSizes: (
    budget: Record<string, unknown>,
    wasmDir: string,
    base?: Record<string, unknown> | null,
  ) => {
    ok: boolean;
    results: Array<{
      contract: string;
      wasm: string;
      sizeBytes: number;
      baselineBytes: number | null;
      deltaBytes: number | null;
      maxBytes: number;
      headroomBytes: number;
      utilizationPct: number;
      overBudget: boolean;
    }>;
    errors: string[];
  };
  renderMarkdownReport: (evaluation: unknown) => string;
  postOrUpdatePrComment: (
    markdown: string,
    env: Record<string, string | undefined>,
    fetchImpl: (
      url: string,
      init?: { method?: string; headers?: Record<string, string>; body?: string },
    ) => Promise<{ ok: boolean; status: number; json: () => Promise<unknown> }>,
  ) => Promise<{ posted: boolean; updated?: boolean; reason?: string }>;
};

// eslint-disable-next-line @typescript-eslint/no-require-imports
const sdkCheck = require("../../scripts/check-soroban-sdk-version.cjs") as {
  checkWorkspaceSorobanSdk: (rootDir: string) => {
    ok: boolean;
    workspaceVersion: string | null;
    checkedMembers: string[];
    errors: string[];
  };
};

const REPO_ROOT = path.resolve(__dirname, "..", "..");

const tempDirs: string[] = [];
function makeTempDir(prefix: string): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(() => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop()!;
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

describe("Stage 1 repository and CI tooling (#91, #93, #94, #95)", () => {
  describe("#91 cargo-deny configuration (deny.toml & security.yml)", () => {
    it("configures deny.toml to allow Apache-2.0, MIT, BSD and reject GPL and unknown registries", () => {
      const denyPath = path.join(REPO_ROOT, "deny.toml");
      expect(fs.existsSync(denyPath)).toBe(true);

      const content = fs.readFileSync(denyPath, "utf8");
      expect(content).toContain('"Apache-2.0"');
      expect(content).toContain('"MIT"');
      expect(content).toContain('"BSD-2-Clause"');
      expect(content).toContain('"BSD-3-Clause"');
      expect(content).not.toMatch(/"A?L?GPL-[0-9.]+/);
      expect(content).toContain('unknown-registry = "deny"');
      expect(content).toContain('unknown-git = "deny"');
      expect(content).toContain("https://github.com/rust-lang/crates.io-index");
    });

    it("wires cargo deny check into .github/workflows/security.yml", () => {
      const workflowPath = path.join(
        REPO_ROOT,
        ".github",
        "workflows",
        "security.yml",
      );
      const workflow = fs.readFileSync(workflowPath, "utf8");
      expect(workflow).toContain("deny.toml");
      expect(workflow).toContain("cargo install cargo-deny --locked --version");
      expect(workflow).toContain("cargo deny check");
    });
  });

  describe("#94 contracts/size-budget.json & #93 WASM size tracking", () => {
    const EXPECTED_CONTRACTS = [
      "rwa-asset",
      "registry",
      "compliance",
      "governance",
      "oracle-adapter",
    ];

    it("defines valid size budgets for all five contracts in contracts/size-budget.json", () => {
      const budgetPath = path.join(REPO_ROOT, "contracts", "size-budget.json");
      expect(fs.existsSync(budgetPath)).toBe(true);

      const budget = JSON.parse(fs.readFileSync(budgetPath, "utf8")) as Record<
        string,
        { wasm: string; baseline_bytes: number; max_bytes: number }
      >;

      for (const name of EXPECTED_CONTRACTS) {
        expect(budget[name]).toBeDefined();
        const parsed = wasmSize.parseBudgetEntry(name, budget[name]);
        expect(parsed.maxBytes).toBeGreaterThan(0);
        // Must stay under Soroban's 64 KiB mainnet contract size ceiling
        expect(parsed.maxBytes).toBeLessThanOrEqual(65536);
        expect(parsed.baselineBytes).toBeGreaterThan(0);
        expect(parsed.baselineBytes!).toBeLessThanOrEqual(parsed.maxBytes);
      }
    });

    it("shows the exact +10 KB delta when a PR adds 10 KB to a contract and fails if above budget", () => {
      const wasmDir = makeTempDir("stellarforge-wasm-");
      const budget = {
        "rwa-asset": {
          wasm: "rwa_asset.wasm",
          baseline_bytes: 35000,
          max_bytes: 44000,
        },
        registry: {
          wasm: "registry.wasm",
          baseline_bytes: 11000,
          max_bytes: 25000,
        },
      };

      // registry grows by +10 KB (10,240 bytes) from 11,000 -> 21,240 (still within 25,000 budget)
      fs.writeFileSync(
        path.join(wasmDir, "rwa_asset.wasm"),
        Buffer.alloc(35000),
      );
      fs.writeFileSync(
        path.join(wasmDir, "registry.wasm"),
        Buffer.alloc(11000 + 10240),
      );

      const evalPass = wasmSize.evaluateWasmSizes(budget, wasmDir);
      expect(evalPass.ok).toBe(true);
      const regRow = evalPass.results.find((r) => r.contract === "registry")!;
      expect(regRow.deltaBytes).toBe(10240);

      const reportPass = wasmSize.renderMarkdownReport(evalPass);
      expect(reportPass).toContain(wasmSize.COMMENT_MARKER);
      expect(reportPass).toContain("+10,240 B (+10.00 KB)");
      expect(reportPass).toContain("| `registry` | `registry.wasm` | 21,240 | 20.74 KB | +10,240 B (+10.00 KB) | 25,000 | 85.0% | PASS |");

      // Now rwa-asset also grows by +10 KB (35,000 -> 45,240), exceeding its 44,000 budget
      fs.writeFileSync(
        path.join(wasmDir, "rwa_asset.wasm"),
        Buffer.alloc(35000 + 10240),
      );

      const evalFail = wasmSize.evaluateWasmSizes(budget, wasmDir);
      expect(evalFail.ok).toBe(false);
      expect(evalFail.errors[0]).toContain(
        'Contract "rwa-asset" (rwa_asset.wasm) is 45240 bytes, exceeding its budget of 44000 bytes by 1240 bytes',
      );

      const reportFail = wasmSize.renderMarkdownReport(evalFail);
      expect(reportFail).toContain("FAIL (+1,240 B over)");
      expect(reportFail).toContain("### Budget Violations");
    });

    it("handles negative deltas, missing WASM artifacts, and invalid budget entries", () => {
      expect(wasmSize.formatDelta(0)).toBe("0 B (0.00 KB)");
      expect(wasmSize.formatDelta(-2048)).toBe("-2,048 B (-2.00 KB)");
      expect(wasmSize.formatDelta(null)).toBe("n/a");

      const wasmDir = makeTempDir("stellarforge-wasm-missing-");
      const evaluation = wasmSize.evaluateWasmSizes(
        { registry: { wasm: "registry.wasm", max_bytes: 20480 } },
        wasmDir,
      );
      expect(evaluation.ok).toBe(false);
      expect(evaluation.errors[0]).toContain('Missing WASM artifact for "registry"');

      expect(() => wasmSize.parseBudgetEntry("bad", 0)).toThrow(
        /positive integer/,
      );
      expect(() => wasmSize.parseBudgetEntry("bad", { max_bytes: -5 })).toThrow(
        /positive integer/,
      );
    });

    it("creates, updates, or gracefully skips PR comments on fork PRs", async () => {
      const tmp = makeTempDir("stellarforge-pr-event-");
      const eventPath = path.join(tmp, "event.json");
      fs.writeFileSync(
        eventPath,
        JSON.stringify({ pull_request: { number: 42 } }),
      );

      const env = {
        GITHUB_TOKEN: "test-token",
        GITHUB_REPOSITORY: "0xLizTech/stellarforge",
        GITHUB_EVENT_PATH: eventPath,
        GITHUB_API_URL: "https://api.github.example",
      };

      const calls: Array<{
        url: string;
        method: string | undefined;
        body: string | undefined;
      }> = [];

      // 1. Creates a new comment when none exists
      const resCreate = await wasmSize.postOrUpdatePrComment(
        `${wasmSize.COMMENT_MARKER}\nhello`,
        env,
        async (url, init) => {
          calls.push({ url, method: init?.method, body: init?.body });
          if (init?.method === "GET") {
            return { ok: true, status: 200, json: async () => [] };
          }
          return { ok: true, status: 201, json: async () => ({ id: 101 }) };
        },
      );
      expect(resCreate).toEqual({ posted: true, updated: false });
      expect(calls).toHaveLength(2);
      expect(calls[1]?.method).toBe("POST");

      // 2. Updates existing comment when marker is found
      calls.length = 0;
      const resUpdate = await wasmSize.postOrUpdatePrComment(
        `${wasmSize.COMMENT_MARKER}\nupdated`,
        env,
        async (url, init) => {
          calls.push({ url, method: init?.method, body: init?.body });
          if (init?.method === "GET") {
            return {
              ok: true,
              status: 200,
              json: async () => [
                { id: 777, body: `${wasmSize.COMMENT_MARKER}\nold` },
              ],
            };
          }
          return { ok: true, status: 200, json: async () => ({ id: 777 }) };
        },
      );
      expect(resUpdate).toEqual({ posted: true, updated: true });
      expect(calls[1]?.method).toBe("PATCH");
      expect(calls[1]?.url).toContain("/issues/comments/777");

      // 3. Gracefully skips when fork GITHUB_TOKEN returns 403
      const resForbidden = await wasmSize.postOrUpdatePrComment(
        `${wasmSize.COMMENT_MARKER}\nfork`,
        env,
        async (_url, init) => {
          if (init?.method === "GET") {
            return { ok: true, status: 200, json: async () => [] };
          }
          return { ok: false, status: 403, json: async () => ({}) };
        },
      );
      expect(resForbidden).toEqual({
        posted: false,
        reason: "insufficient-permissions",
      });
    });
  });

  describe("#95 soroban-sdk version consistency check", () => {
    it("passes on the real StellarForge workspace", () => {
      const res = sdkCheck.checkWorkspaceSorobanSdk(REPO_ROOT);
      expect(res.ok).toBe(true);
      expect(res.errors).toEqual([]);
      expect(res.workspaceVersion).toBe("27.0.5");
      expect(res.checkedMembers).toContain("contracts/rwa-asset");
      expect(res.checkedMembers).toContain("contracts/registry");
      expect(res.checkedMembers).toContain("contracts/compliance");
      expect(res.checkedMembers).toContain("contracts/governance");
      expect(res.checkedMembers).toContain("contracts/oracle-adapter");
    });

    it("fails when a workspace crate pins its own soroban-sdk version", () => {
      const wsDir = makeTempDir("stellarforge-sdk-drift-");
      fs.mkdirSync(path.join(wsDir, "contracts", "alpha"), { recursive: true });
      fs.mkdirSync(path.join(wsDir, "contracts", "beta"), { recursive: true });

      fs.writeFileSync(
        path.join(wsDir, "Cargo.toml"),
        `[workspace]
members = ["contracts/alpha", "contracts/beta"]
resolver = "2"

[workspace.dependencies]
soroban-sdk = { version = "27.0.5" }
`,
      );

      // alpha uses workspace = true properly
      fs.writeFileSync(
        path.join(wsDir, "contracts", "alpha", "Cargo.toml"),
        `[package]
name = "alpha"
version = "0.1.0"
edition = "2021"

[dependencies]
soroban-sdk = { workspace = true }
`,
      );

      // beta pins its own version in [dependencies] and [dev-dependencies]
      fs.writeFileSync(
        path.join(wsDir, "contracts", "beta", "Cargo.toml"),
        `[package]
name = "beta"
version = "0.1.0"
edition = "2021"

[dependencies]
soroban-sdk = "27.0.5"

[dev-dependencies]
soroban-sdk = { version = "27.0.0", features = ["testutils"] }
`,
      );

      const res = sdkCheck.checkWorkspaceSorobanSdk(wsDir);
      expect(res.ok).toBe(false);
      expect(res.errors.length).toBeGreaterThanOrEqual(2);
      expect(res.errors.join("\n")).toContain("contracts/beta/Cargo.toml");

      // Also verify CLI exits with code 1
      const cliRun = spawnSync(
        process.execPath,
        [
          path.join(REPO_ROOT, "scripts", "check-soroban-sdk-version.cjs"),
          "--root",
          wsDir,
        ],
        { encoding: "utf8" },
      );
      expect(cliRun.status).toBe(1);
    });

    it("fails when a crate uses a [dependencies.soroban-sdk] subtable or dotted version override", () => {
      const wsDir = makeTempDir("stellarforge-sdk-subtable-");
      fs.mkdirSync(path.join(wsDir, "contracts", "gamma"), { recursive: true });

      fs.writeFileSync(
        path.join(wsDir, "Cargo.toml"),
        `[workspace]
members = ["contracts/gamma"]

[workspace.dependencies]
soroban-sdk = { version = "27.0.5" }
`,
      );

      fs.writeFileSync(
        path.join(wsDir, "contracts", "gamma", "Cargo.toml"),
        `[package]
name = "gamma"
version = "0.1.0"
edition = "2021"

[dependencies.soroban-sdk]
version = "27.0.5"
`,
      );

      const res = sdkCheck.checkWorkspaceSorobanSdk(wsDir);
      expect(res.ok).toBe(false);
      expect(res.errors.join("\n")).toContain(
        'contracts/gamma/Cargo.toml:7 [dependencies.soroban-sdk] must set "workspace = true"',
      );
    });

    it("fails when Cargo.lock resolves multiple versions of soroban-sdk", () => {
      const wsDir = makeTempDir("stellarforge-sdk-lock-");
      fs.mkdirSync(path.join(wsDir, "contracts", "alpha"), { recursive: true });

      fs.writeFileSync(
        path.join(wsDir, "Cargo.toml"),
        `[workspace]
members = ["contracts/alpha"]

[workspace.dependencies]
soroban-sdk = { version = "27.0.5" }
`,
      );
      fs.writeFileSync(
        path.join(wsDir, "contracts", "alpha", "Cargo.toml"),
        `[package]
name = "alpha"
version = "0.1.0"

[dependencies]
soroban-sdk = { workspace = true }
`,
      );
      fs.writeFileSync(
        path.join(wsDir, "Cargo.lock"),
        `[[package]]
name = "soroban-sdk"
version = "27.0.5"

[[package]]
name = "soroban-sdk"
version = "27.0.6"
`,
      );

      const res = sdkCheck.checkWorkspaceSorobanSdk(wsDir);
      expect(res.ok).toBe(false);
      expect(res.errors.join("\n")).toContain(
        "Cargo.lock resolves multiple versions of soroban-sdk: 27.0.5, 27.0.6",
      );
    });
  });
});
