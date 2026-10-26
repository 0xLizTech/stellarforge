import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { describe, it, expect } from "vitest";
import { ESLint } from "eslint";

// Validates the SDK's eslint.config.mjs (the config the pre-commit hook and SDK
// CI both run). It asserts the config loads, passes clean code, and flags the
// policy violations CONTRIBUTING.md calls out ("no `any` types").

const sdkRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

// One ESLint instance for the suite; building it loads and type-resolves the
// flat config, which is the slow part, so it is not repeated per assertion.
// cwd pins config resolution to eslint.config.mjs.
const eslint = new ESLint({ cwd: sdkRoot });

async function lint(code: string) {
  // A src/ path so the file-scoped rule block applies.
  const results = await eslint.lintText(code, { filePath: resolve(sdkRoot, "src/__fixture.ts") });
  const result = results[0];
  if (!result) throw new Error("ESLint returned no result");
  return result;
}

describe("eslint.config.mjs", () => {
  it("passes clean, strictly-typed TypeScript", async () => {
    const result = await lint("export const double = (n: number): number => n * 2;\n");
    expect(result.errorCount).toBe(0);
    expect(result.warningCount).toBe(0);
  }, 30000);

  it("flags explicit any", async () => {
    const result = await lint("export const parse = (value: any): string => String(value);\n");
    expect(result.errorCount).toBeGreaterThan(0);
    expect(result.messages.map((m) => m.ruleId)).toContain(
      "@typescript-eslint/no-explicit-any",
    );
  }, 30000);

  it("flags unused variables but allows the underscore convention", async () => {
    const unused = await lint("const unusedValue = 42;\nexport const x = 1;\n");
    expect(unused.messages.map((m) => m.ruleId)).toContain(
      "@typescript-eslint/no-unused-vars",
    );

    const allowed = await lint("export const handler = (_event: string): void => {};\n");
    expect(allowed.errorCount).toBe(0);
  }, 30000);
});
