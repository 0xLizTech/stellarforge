import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { spawnSync } from "node:child_process";

const packageDir = process.cwd();
const temporaryDirs: string[] = [];
afterEach(() => {
  for (const dir of temporaryDirs.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("shell completion generation", () => {
  it("regenerates the committed bash, zsh and fish completions deterministically", () => {
    const outputDir = mkdtempSync(path.join(os.tmpdir(), "nav-reporter-completions-"));
    temporaryDirs.push(outputDir);
    const generated = spawnSync(process.execPath, [
      path.join(packageDir, "scripts/generate-completions.mjs"), "--output-dir", outputDir,
    ], { encoding: "utf8" });

    expect(generated.status, generated.stderr).toBe(0);
    const committedDir = path.join(packageDir, "completions");
    const names = readdirSync(committedDir).sort();
    expect(names).toEqual([
      "_stellarforge-nav-reporter",
      "stellarforge-nav-reporter.bash",
      "stellarforge-nav-reporter.fish",
    ]);
    for (const name of names) {
      expect(readFileSync(path.join(outputDir, name), "utf8"))
        .toBe(readFileSync(path.join(committedDir, name), "utf8"));
    }
  });
});
