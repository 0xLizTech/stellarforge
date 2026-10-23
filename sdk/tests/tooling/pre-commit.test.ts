import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { hasBash, readRepoFile, repoRoot, run } from "./helpers.js";

const config = readRepoFile(".pre-commit-config.yaml");

/** The text of one hook block: from its `- id:` line to the next hook or the end. */
function hookBlock(id: string): string {
  const start = config.indexOf(`- id: ${id}`);
  expect(start, `no hook with id ${id}`).toBeGreaterThanOrEqual(0);
  const rest = config.slice(start + 1);
  const next = rest.search(/\n\s*- id: /);
  return next === -1 ? rest : rest.slice(0, next);
}

/** The value of a `key:` line inside a hook block. */
function field(block: string, key: string): string {
  const m = new RegExp(`^\\s*${key}:\\s*(.+?)\\s*$`, "m").exec(block);
  expect(m, `hook has no ${key}`).not.toBeNull();
  return m?.[1] ?? "";
}

describe(".pre-commit-config.yaml", () => {
  it("runs cargo fmt in check mode over the whole workspace", () => {
    const block = hookBlock("cargo-fmt");
    expect(field(block, "entry")).toBe("cargo fmt --all -- --check");
    expect(field(block, "language")).toBe("system");
    // cargo fmt takes no file list; handing it staged paths would be wrong.
    expect(field(block, "pass_filenames")).toBe("false");
  });

  it("runs the cargo fmt hook only when a Rust file is staged", () => {
    const files = new RegExp(field(hookBlock("cargo-fmt"), "files"));
    expect(files.test("contracts/rwa-asset/src/lib.rs")).toBe(true);
    expect(files.test("tools/benchmark/src/main.rs")).toBe(true);
    expect(files.test("sdk/src/index.ts")).toBe(false);
    expect(files.test("README.md")).toBe(false);
  });

  it("runs eslint through the staged-files wrapper script", () => {
    const block = hookBlock("eslint");
    expect(field(block, "entry")).toBe("bash scripts/pre-commit-eslint.sh");
    expect(field(block, "language")).toBe("system");
  });

  it("runs the eslint hook only for SDK source files", () => {
    const files = new RegExp(field(hookBlock("eslint"), "files"));
    expect(files.test("sdk/src/utils.ts")).toBe(true);
    expect(files.test("sdk/tests/utils.test.ts")).toBe(false);
    expect(files.test("contracts/rwa-asset/src/lib.rs")).toBe(false);
    expect(files.test("sdk/src/notes.md")).toBe(false);
  });
});

// The wrapper is what the eslint hook actually executes, so exercise it for real.
describe.skipIf(!hasBash)("scripts/pre-commit-eslint.sh", () => {
  let dir = "";

  beforeEach(() => {
    // Inside sdk/ so the file falls under sdk/eslint.config.mjs; removed afterwards.
    dir = mkdtempSync(join(repoRoot, "sdk", "tests", "tooling", "tmp-eslint-"));
  });

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  /** Runs the hook exactly as pre-commit does: repo-relative, forward-slash path. */
  function hook(...files: string[]) {
    const paths = files.map((f) => relative(repoRoot, f).split("\\").join("/"));
    return run("bash", ["scripts/pre-commit-eslint.sh", ...paths]);
  }

  it("accepts a clean file", () => {
    const file = join(dir, "clean.ts");
    writeFileSync(file, "export const answer = 42;\n");
    const result = hook(file);
    expect(result.status, result.output).toBe(0);
  });

  it("rejects a file with lint errors", () => {
    const file = join(dir, "bad.ts");
    writeFileSync(file, "var answer: any = 42;\nexport { answer };\n");
    const result = hook(file);
    expect(result.status, result.output).not.toBe(0);
    expect(result.output).toMatch(/no-var|no-explicit-any/);
  });

  it("fails the whole commit when only one of several files is bad", () => {
    const good = join(dir, "good.ts");
    const bad = join(dir, "bad.ts");
    writeFileSync(good, "export const ok = true;\n");
    writeFileSync(bad, "export const x: any = 1;\n");
    expect(hook(good, bad).status).not.toBe(0);
  });

  it("does nothing, successfully, when no files are passed", () => {
    const result = run("bash", ["scripts/pre-commit-eslint.sh"]);
    expect(result.status, result.output).toBe(0);
  });
});
