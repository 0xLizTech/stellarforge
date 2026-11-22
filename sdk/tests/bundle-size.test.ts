import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { describe, expect, it } from "vitest";

const SDK_DIR = path.resolve(__dirname, "..");
const DIST_DIR = path.join(SDK_DIR, "dist");

describe("SDK Bundle Size Tracking (#182)", () => {
  it("enforces size-limit configuration and all bundles stay within budgets", () => {
    // Run size-limit against configured package targets
    const output = execSync("npx size-limit", {
      cwd: SDK_DIR,
      encoding: "utf8",
    });

    expect(output).toBeDefined();
    // Size limit output includes file size measurements
    expect(output).toContain("brotlied");
  });

  it("fails when a bundle exceeds its configured budget (fail on regression)", () => {
    // Setting an impossibly low budget (1B) must fail with non-zero exit code
    let threw = false;
    try {
      execSync("npx size-limit --limit 1B dist/index.js", {
        cwd: SDK_DIR,
        encoding: "utf8",
        stdio: "pipe",
      });
    } catch (err: unknown) {
      threw = true;
      const errorOutput = String((err as { stderr?: Buffer; stdout?: Buffer }).stdout || "");
      expect(errorOutput).toContain("Package size limit has exceeded");
    }
    expect(threw).toBe(true);
  });

  it("verifies compiled dist files exist and are within sanity bounds", () => {
    const requiredFiles = [
      "index.js",
      "index.mjs",
      "client.js",
      "client.mjs",
      "governance.js",
      "governance.mjs",
      "vault.js",
      "vault.mjs",
      "oracle.js",
      "oracle.mjs",
      "registry.js",
      "registry.mjs",
    ];

    for (const filename of requiredFiles) {
      const filePath = path.join(DIST_DIR, filename);
      expect(fs.existsSync(filePath), `Expected ${filename} to exist in dist/`).toBe(true);
      const stat = fs.statSync(filePath);
      expect(stat.size).toBeGreaterThan(50);
      // Main bundle should not bloat beyond 150 KB uncompressed
      expect(stat.size).toBeLessThan(150_000);
    }
  });
});
