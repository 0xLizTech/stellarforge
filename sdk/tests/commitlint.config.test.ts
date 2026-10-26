import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { createRequire } from "node:module";
import { describe, it, expect } from "vitest";
import lint from "@commitlint/lint";
import conventional from "@commitlint/config-conventional";

// Validates the repository's root commitlint.config.cjs (the one the Commitlint
// CI job runs) against the commit style documented in CONTRIBUTING.md. The real
// config file is loaded here, so the test fails if the config drifts from the
// documented types and scopes rather than testing a private copy.
//
// The file `extends` @commitlint/config-conventional. commitlint's own loader
// resolves that relative to the config file's directory (the repo root), which
// has no node_modules in this SDK-scoped test run; CI installs it at the root
// before running. So the test reproduces the same merge explicitly: the base
// conventional rules overlaid with the project file's overrides.

const sdkRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const configFile = resolve(sdkRoot, "..", "commitlint.config.cjs");
const require = createRequire(import.meta.url);

interface CommitlintConfig {
  extends?: string[];
  rules: Record<string, unknown>;
}

const projectConfig = require(configFile) as CommitlintConfig;
const rules = { ...conventional.rules, ...projectConfig.rules };

async function lintMessage(message: string) {
  // Default (conventional) parser; matches how the CLI parses in CI.
  return lint(message, rules);
}

describe("commitlint.config.cjs", () => {
  it("extends @commitlint/config-conventional", () => {
    expect(projectConfig.extends).toContain("@commitlint/config-conventional");
  });

  it("enforces exactly the commit types documented in CONTRIBUTING.md", () => {
    expect(projectConfig.rules["type-enum"]).toEqual([
      2,
      "always",
      ["feat", "fix", "docs", "test", "refactor", "chore", "perf"],
    ]);
  });

  it("accepts a conforming commit with a documented type and scope", async () => {
    const result = await lintMessage("feat(sdk): add registry client read helpers");
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it("accepts a conforming commit with no scope", async () => {
    const result = await lintMessage("docs: clarify smoke test setup");
    expect(result.valid).toBe(true);
  });

  it("accepts the newer workspace scopes (vault, common)", async () => {
    for (const scope of ["vault", "common"]) {
      const result = await lintMessage(`fix(${scope}): correct storage ttl constant`);
      expect(result.valid, `${scope} should be allowed`).toBe(true);
    }
  });

  it("rejects an unknown commit type", async () => {
    const result = await lintMessage("wip(sdk): still working");
    expect(result.valid).toBe(false);
    expect(result.errors.map((e) => e.name)).toContain("type-enum");
  });

  it("rejects an unknown scope", async () => {
    const result = await lintMessage("feat(frontend): add dashboard");
    expect(result.valid).toBe(false);
    expect(result.errors.map((e) => e.name)).toContain("scope-enum");
  });

  it("rejects a subject line longer than 72 characters", async () => {
    const long = `feat(sdk): ${"a".repeat(80)}`;
    const result = await lintMessage(long);
    expect(result.valid).toBe(false);
    expect(result.errors.map((e) => e.name)).toContain("header-max-length");
  });

  it("rejects a non-conventional subject", async () => {
    const result = await lintMessage("updated some files");
    expect(result.valid).toBe(false);
  });
});
