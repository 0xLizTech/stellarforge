import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { copyFileSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { devNull, tmpdir } from "node:os";
import { join } from "node:path";
import { repoRoot, run } from "./helpers.js";

// Keep the user's global git config (core.autocrlf in particular) out of the
// experiments, so a result never depends on the machine running the test.
const isolatedEnv: NodeJS.ProcessEnv = {
  ...process.env,
  GIT_CONFIG_GLOBAL: devNull,
  GIT_CONFIG_SYSTEM: devNull,
  GIT_CONFIG_NOSYSTEM: "1",
};

/** `git check-attr` for several attributes of several paths -> { path: { attr: value } }. */
function checkAttr(attrs: string[], paths: string[]): Record<string, Record<string, string>> {
  const out = run("git", ["check-attr", ...attrs, "--", ...paths], { env: isolatedEnv });
  expect(out.status, out.output).toBe(0);
  const parsed: Record<string, Record<string, string>> = {};
  for (const line of out.stdout.split("\n")) {
    const m = /^(.+): (\S+): (.*)$/.exec(line.trim());
    if (!m) continue;
    const [, path = "", attr = "", value = ""] = m;
    (parsed[path] ??= {})[attr] = value;
  }
  return parsed;
}

describe(".gitattributes in this repository", () => {
  const lfPaths = [
    "scripts/deploy.sh",
    "contracts/rwa-asset/src/lib.rs",
    "sdk/src/index.ts",
    "README.md",
  ];

  it("forces LF for *.sh, *.rs, *.ts and *.md", () => {
    const attrs = checkAttr(["text", "eol"], lfPaths);
    for (const path of lfPaths) {
      expect(attrs[path]?.["eol"], path).toBe("lf");
      expect(attrs[path]?.["text"], path).toBe("set");
    }
  });

  it("forces LF for the other files the repo executes or parses", () => {
    const paths = ["Makefile", ".github/workflows/sdk-ci.yml", "Cargo.toml", "sdk/package.json"];
    const attrs = checkAttr(["eol"], paths);
    for (const path of paths) expect(attrs[path]?.["eol"], path).toBe("lf");
  });

  it("marks Cargo.lock and the npm lockfile as generated", () => {
    const paths = ["Cargo.lock", "sdk/package-lock.json"];
    const attrs = checkAttr(["linguist-generated"], paths);
    for (const path of paths) expect(attrs[path]?.["linguist-generated"], path).toBe("true");
  });

  it("does not mark hand-written source as generated", () => {
    const attrs = checkAttr(["linguist-generated"], ["sdk/src/index.ts", "Cargo.toml"]);
    expect(attrs["sdk/src/index.ts"]?.["linguist-generated"]).toBe("unspecified");
    expect(attrs["Cargo.toml"]?.["linguist-generated"]).toBe("unspecified");
  });

  it("leaves no CRLF and no mixed endings in any tracked text file", () => {
    const out = run("git", ["ls-files", "--eol"], { env: isolatedEnv });
    expect(out.status, out.output).toBe(0);
    const bad = out.stdout.split("\n").filter((line) => /^i\/(crlf|mixed)\b/.test(line));
    expect(bad).toEqual([]);
  });
});

describe(".gitattributes normalizes line endings on `git add`", () => {
  let dir = "";

  const git = (...args: string[]) => run("git", args, { cwd: dir, env: isolatedEnv });

  /** The index line ending git reports for a file: lf, crlf, mixed or -text. */
  const indexEol = (file: string): string => {
    const out = git("ls-files", "--eol", "--", file);
    return /^i\/(\S+)/.exec(out.stdout)?.[1] ?? `no output: ${out.output}`;
  };

  beforeEach(() => {
    dir = mkdtempSync(join(tmpdir(), "gitattributes-"));
    expect(git("init", "-q").status).toBe(0);
    // Windows default; the whole point is to hold even then.
    expect(git("config", "core.autocrlf", "false").status).toBe(0);
  });

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  it.each(["deploy.sh", "lib.rs", "index.ts", "README.md"])("commits %s with LF", (name) => {
    copyFileSync(join(repoRoot, ".gitattributes"), join(dir, ".gitattributes"));
    writeFileSync(join(dir, name), "line one\r\nline two\r\n");
    expect(git("add", "--", ".gitattributes", name).status).toBe(0);
    expect(indexEol(name)).toBe("lf");
  });

  it("commits a file that already had LF unchanged", () => {
    copyFileSync(join(repoRoot, ".gitattributes"), join(dir, ".gitattributes"));
    writeFileSync(join(dir, "ok.sh"), "echo hi\n");
    git("add", "--", ".gitattributes", "ok.sh");
    expect(indexEol("ok.sh")).toBe("lf");
  });

  it("does not rewrite binary files", () => {
    copyFileSync(join(repoRoot, ".gitattributes"), join(dir, ".gitattributes"));
    writeFileSync(join(dir, "logo.png"), Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
    git("add", "--", ".gitattributes", "logo.png");
    expect(indexEol("logo.png")).toBe("-text");
  });

  it("control: without .gitattributes the same CRLF script stays CRLF", () => {
    writeFileSync(join(dir, "deploy.sh"), "line one\r\nline two\r\n");
    git("add", "--", "deploy.sh");
    expect(indexEol("deploy.sh")).toBe("crlf");
  });
});
