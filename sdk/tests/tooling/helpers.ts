import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Repository root: sdk/tests/tooling -> three levels up. */
export const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

/** Reads a repo file as text with CRLF folded to LF, so checks pass on any checkout. */
export function readRepoFile(relativePath: string): string {
  return readFileSync(join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
}

export interface RunResult {
  status: number | null;
  stdout: string;
  stderr: string;
  output: string;
}

/** Runs a command to completion and never throws on a non-zero exit. */
export function run(
  command: string,
  args: string[],
  options: { cwd?: string; input?: string; env?: NodeJS.ProcessEnv } = {},
): RunResult {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    encoding: "utf8",
    ...(options.input !== undefined ? { input: options.input } : {}),
    ...(options.env !== undefined ? { env: options.env } : {}),
  });
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  return { status: result.status, stdout, stderr, output: `${stdout}${stderr}` };
}

/** True when `bash` can be started (it cannot on a bare Windows shell). */
export const hasBash = run("bash", ["-c", "true"]).status === 0;
