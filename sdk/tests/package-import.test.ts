import path from "path";
import { execSync } from "child_process";
import { describe, expect, it } from "vitest";

const SDK_DIR = path.resolve(__dirname, "..");

describe("Package Import Verification (#181)", () => {
  it(
    "verifies ESM, CJS, and TypeScript type resolution from packed tarball",
    () => {
      const scriptPath = path.join(SDK_DIR, "scripts", "test-package-imports.mjs");
      const output = execSync(`node "${scriptPath}"`, {
        cwd: SDK_DIR,
        encoding: "utf8",
      });

      expect(output).toContain("ESM imports and subpath exports passed.");
      expect(output).toContain("CJS requires and subpath exports passed.");
      expect(output).toContain("TypeScript type resolution passed.");
      expect(output).toContain("All package import and type tests passed successfully!");
    },
    30_000
  );
});
