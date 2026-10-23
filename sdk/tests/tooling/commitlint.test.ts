import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { readRepoFile, repoRoot, run } from "./helpers.js";

const configPath = join(repoRoot, "commitlint.config.cjs");

/** Path of the commitlint CLI installed in sdk/ (it is a devDependency there). */
function cliPath(): string {
  const pkgDir = join(repoRoot, "sdk", "node_modules", "@commitlint", "cli");
  const pkg = JSON.parse(readFileSync(join(pkgDir, "package.json"), "utf8")) as {
    bin: string | Record<string, string>;
  };
  const bin = typeof pkg.bin === "string" ? pkg.bin : (pkg.bin["commitlint"] ?? "");
  return join(pkgDir, bin);
}

const cli = cliPath();

/** Lints one commit message exactly as CI and `make commitlint` do. */
function lint(message: string): { ok: boolean; output: string } {
  const result = run(process.execPath, [cli, "--config", configPath], { input: message });
  return { ok: result.status === 0, output: result.output };
}

describe("commitlint accepts conforming commits", () => {
  // The examples in CONTRIBUTING.md "Commit Style".
  it.each([
    "feat(rwa-asset): add transfer_from with allowance deduction",
    "fix(compliance): correct expiry check direction (> not <)",
    "docs(readme): add SDK quick-start example",
    "test(rwa-asset): add pause/unpause integration tests",
    "chore(ci): cache cargo registry for faster builds",
  ])("%s", (message) => {
    const result = lint(message);
    expect(result.ok, result.output).toBe(true);
  });

  it.each(["refactor", "perf"])("allows the %s type", (type) => {
    expect(lint(`${type}(sdk): tidy the client`).ok).toBe(true);
  });

  it("allows a commit with no scope", () => {
    expect(lint("fix: handle an empty holder list").ok).toBe(true);
  });

  it("allows a breaking-change marker", () => {
    expect(lint("feat(sdk)!: rename the client constructor option").ok).toBe(true);
  });

  it("allows a body and a footer separated by blank lines", () => {
    const message = [
      "fix(governance): reject a zero voting period",
      "",
      "A zero period closed the vote in the ledger it opened.",
      "",
      "Closes #123",
    ].join("\n");
    const result = lint(message);
    expect(result.ok, result.output).toBe(true);
  });

  it("ignores a merge commit", () => {
    expect(lint("Merge pull request #12 from someone/branch").ok).toBe(true);
  });
});

describe("commitlint rejects non-conforming commits", () => {
  it.each([
    ["an unknown type", "update(sdk): add a thing"],
    ["a type that is not lower case", "Feat(sdk): add a thing"],
    ["no type at all", "added a thing to the sdk"],
    ["a missing colon", "feat(sdk) add a thing"],
    ["an unknown scope", "feat(frontend): add a thing"],
    ["an empty subject", "feat(sdk): "],
    ["a trailing full stop", "feat(sdk): add a thing."],
    ["a subject over 72 characters", `feat(sdk): ${"x".repeat(70)}`],
    ["a body with no blank line before it", "feat(sdk): add a thing\nbody starts too soon"],
  ])("%s", (_reason, message) => {
    const result = lint(message);
    expect(result.ok, `expected a failure for ${JSON.stringify(message)}`).toBe(false);
    expect(result.output).toMatch(/✖|problems/);
  });

  it("names the broken rule so the author can fix it", () => {
    const result = lint("feat(frontend): add a thing");
    expect(result.output).toContain("scope-enum");
  });
});

describe("commitlint config and CONTRIBUTING.md agree", () => {
  const config = createRequire(import.meta.url)(configPath) as {
    rules: Record<string, [number, string, unknown]>;
  };

  /** The backticked words on the `**Label:**` line of CONTRIBUTING.md. */
  function documented(label: string): string[] {
    const line = readRepoFile("CONTRIBUTING.md")
      .split("\n")
      .find((l) => l.startsWith(`**${label}:**`));
    expect(line, `CONTRIBUTING.md has no **${label}:** line`).toBeDefined();
    return [...(line ?? "").matchAll(/`([^`]+)`/g)].map((m) => m[1] ?? "");
  }

  it("lists the same types", () => {
    const configured = config.rules["type-enum"]?.[2] as string[];
    expect([...documented("Types")].sort()).toEqual([...configured].sort());
  });

  it("lists the same scopes", () => {
    const configured = config.rules["scope-enum"]?.[2] as string[];
    expect([...documented("Scopes")].sort()).toEqual([...configured].sort());
  });
});
