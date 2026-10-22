import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { hasBash, readRepoFile, repoRoot, run } from "./helpers.js";

/**
 * A small offline stand-in for cargo-machete: a dependency counts as used when
 * some `.rs` file of the crate mentions it as `name::` or `use name`. It is
 * stricter than a bare word search, looser than the compiler. It exists so
 * `npm test` can guard the manifests on a machine without cargo-machete; the
 * real check is the `Unused Dependencies` workflow.
 */
function findUnusedDependencies(crateDir: string): string[] {
  const manifest = readFileSync(join(crateDir, "Cargo.toml"), "utf8").replace(/\r\n/g, "\n");
  const declared = new Set<string>();
  let inDependencies = false;
  for (const line of manifest.split("\n")) {
    const section = /^\s*\[(.+)\]\s*$/.exec(line);
    if (section) {
      inDependencies = /^(dev-|build-)?dependencies$/.test(section[1] ?? "");
      continue;
    }
    const entry = /^([A-Za-z0-9_-]+)\s*=/.exec(line);
    if (inDependencies && entry) declared.add(entry[1] ?? "");
  }

  const sources = readdirSync(crateDir, { recursive: true, withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".rs"))
    .filter((e) => !e.parentPath.split(/[\\/]/).includes("target"))
    .map((e) => readFileSync(join(e.parentPath, e.name), "utf8"))
    .join("\n");

  return [...declared].filter((name) => {
    const ident = name.replace(/-/g, "_");
    return !new RegExp(`(^|[^\\w])${ident}\\s*::|\\buse\\s+${ident}\\b|extern\\s+crate\\s+${ident}\\b`).test(sources);
  });
}

/** Directories of every workspace member listed in the root Cargo.toml. */
function workspaceMembers(): string[] {
  const root = readRepoFile("Cargo.toml");
  const block = /members\s*=\s*\[([^\]]*)\]/.exec(root)?.[1] ?? "";
  return [...block.matchAll(/"([^"]+)"/g)].map((m) => m[1] ?? "");
}

describe("workspace manifests", () => {
  it("lists the members this check expects to find", () => {
    const members = workspaceMembers();
    expect(members).toContain("contracts/rwa-asset");
    expect(members).toContain("tools/benchmark");
  });

  it.each(workspaceMembers())("%s declares no unused dependency", (member) => {
    expect(findUnusedDependencies(join(repoRoot, member))).toEqual([]);
  });
});

describe("findUnusedDependencies", () => {
  let dir = "";

  /** A throwaway crate with the given manifest dependencies and lib.rs. */
  function crate(dependencies: string, source: string, extraSources: Record<string, string> = {}): string {
    mkdirSync(join(dir, "src"), { recursive: true });
    writeFileSync(
      join(dir, "Cargo.toml"),
      `[package]\nname = "fixture"\nversion = "0.0.0"\nedition = "2021"\n\n${dependencies}`,
    );
    writeFileSync(join(dir, "src", "lib.rs"), source);
    for (const [path, text] of Object.entries(extraSources)) {
      mkdirSync(join(dir, path, ".."), { recursive: true });
      writeFileSync(join(dir, path), text);
    }
    return dir;
  }

  beforeEach(() => {
    dir = mkdtempSync(join(tmpdir(), "unused-deps-"));
  });

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  it("reports nothing when every dependency is used", () => {
    crate('[dependencies]\nserde = "1"\n', "use serde::Serialize;\n");
    expect(findUnusedDependencies(dir)).toEqual([]);
  });

  it("reports a dependency that no source mentions", () => {
    crate('[dependencies]\nserde = "1"\nrand = "0.8"\n', "use serde::Serialize;\n");
    expect(findUnusedDependencies(dir)).toEqual(["rand"]);
  });

  it("maps a hyphenated crate name to its underscored identifier", () => {
    crate('[dependencies]\nsoroban-sdk = { workspace = true }\n', "use soroban_sdk::Env;\n");
    expect(findUnusedDependencies(dir)).toEqual([]);
  });

  it("counts a dependency used only from tests/", () => {
    crate('[dev-dependencies]\ncompliance = { path = "../compliance" }\n', "", {
      "tests/it.rs": "use compliance::ComplianceContract;\n",
    });
    expect(findUnusedDependencies(dir)).toEqual([]);
  });

  it("reports an unused dev-dependency", () => {
    crate('[dev-dependencies]\ncompliance = { path = "../compliance" }\n', "pub fn f() {}\n");
    expect(findUnusedDependencies(dir)).toEqual(["compliance"]);
  });

  it("is not fooled by a dependency name that is only a substring", () => {
    crate('[dependencies]\nserde = "1"\n', "use serde_json::Value;\n");
    expect(findUnusedDependencies(dir)).toEqual(["serde"]);
  });

  it("ignores keys outside the dependency tables", () => {
    crate('[features]\ntestutils = []\n\n[dependencies]\n', "pub fn f() {}\n");
    expect(findUnusedDependencies(dir)).toEqual([]);
  });
});

describe("Unused Dependencies workflow", () => {
  const workflow = readRepoFile(".github/workflows/unused-deps.yml");

  it("runs on a schedule and can be started by hand", () => {
    expect(workflow).toMatch(/^\s*schedule:/m);
    expect(workflow).toMatch(/^\s*- cron: "(\S+ ){4}\S+"/m);
    expect(workflow).toMatch(/^\s*workflow_dispatch:/m);
  });

  it("also gates pull requests and main", () => {
    expect(workflow).toMatch(/^\s*pull_request:/m);
    expect(workflow).toMatch(/^\s*push:/m);
  });

  it("runs the self-test and the real check", () => {
    expect(workflow).toContain("bash scripts/check-unused-deps.sh --self-test");
    expect(workflow).toMatch(/run: bash scripts\/check-unused-deps\.sh\s*$/m);
  });

  it("pins the tool version, and the script, workflow and docs agree on it", () => {
    const fromWorkflow = /CARGO_MACHETE_VERSION: "(\d+\.\d+\.\d+)"/.exec(workflow)?.[1];
    const fromScript = /MACHETE_VERSION="(\d+\.\d+\.\d+)"/.exec(readRepoFile("scripts/check-unused-deps.sh"))?.[1];
    const fromDocs = /cargo-machete --locked --version (\d+\.\d+\.\d+)/.exec(readRepoFile("CONTRIBUTING.md"))?.[1];
    const fromMakefile = /--version (\d+\.\d+\.\d+)/.exec(readRepoFile("Makefile"))?.[1];
    expect(fromWorkflow).toBeDefined();
    expect(fromScript).toBe(fromWorkflow);
    expect(fromDocs).toBe(fromWorkflow);
    expect(fromMakefile).toBe(fromWorkflow);
  });
});

describe("new workflows pin their actions", () => {
  it.each([".github/workflows/unused-deps.yml", ".github/workflows/commitlint.yml"])("%s", (path) => {
    const uses = [...readRepoFile(path).matchAll(/^\s*(?:- )?uses:\s*(\S+)/gm)].map((m) => m[1] ?? "");
    expect(uses.length).toBeGreaterThan(0);
    for (const action of uses) {
      // owner/repo@vN, owner/repo@vN.N.N or owner/repo@<40-hex sha>; never a branch.
      expect(action, action).toMatch(/@(v\d+(\.\d+){0,2}|[0-9a-f]{40})$/);
    }
  });
});

describe("Commitlint workflow", () => {
  const workflow = readRepoFile(".github/workflows/commitlint.yml");

  it("runs on pull requests and checks the whole PR range", () => {
    expect(workflow).toMatch(/^\s*pull_request:/m);
    expect(workflow).toContain("fetch-depth: 0");
    expect(workflow).toContain("--from \"$BASE_SHA\"");
    expect(workflow).toContain("--to \"$HEAD_SHA\"");
    expect(workflow).toContain("commitlint.config.cjs");
  });

  it("does not splice PR-controlled values into the shell script", () => {
    const script = /run: \|([\s\S]*)$/.exec(workflow)?.[1] ?? "";
    expect(script).not.toContain("${{");
  });
});

// With cargo-machete installed, run the real thing too.
const hasMachete = run("cargo-machete", ["--version"]).status === 0;

describe.skipIf(!hasBash || !hasMachete)("scripts/check-unused-deps.sh (needs cargo-machete)", () => {
  it("proves it can pass and fail", () => {
    const result = run("bash", ["scripts/check-unused-deps.sh", "--self-test"]);
    expect(result.status, result.output).toBe(0);
    expect(result.stdout).toContain("passes");
    expect(result.stdout).toContain("fails");
  });

  it("passes on this workspace", () => {
    const result = run("bash", ["scripts/check-unused-deps.sh"]);
    expect(result.status, result.output).toBe(0);
  });
});
