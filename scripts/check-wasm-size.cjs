#!/usr/bin/env node
/**
 * Measures compiled Soroban contract WASM sizes against contracts/size-budget.json,
 * formats a Markdown size & delta report for CI summaries and pull request comments,
 * and exits non-zero if any contract exceeds its configured byte budget (#93, #94).
 *
 * Usage:
 *   node scripts/check-wasm-size.cjs [options]
 *
 * Options:
 *   --budget <path>           Path to size budget JSON (default: contracts/size-budget.json)
 *   --wasm-dir <path>         Directory containing compiled .wasm files
 *                             (default: target/wasm32v1-none/release)
 *   --base <path>             Optional base size-budget.json or sizes JSON for delta comparison
 *   --output-markdown <path>  Write the Markdown report to a file
 *   --output-json <path>      Write the measured byte sizes and deltas as JSON to a file
 *   --post-pr-comment         Create or update a pull request comment when running in GitHub Actions
 */
"use strict";

const fs = require("fs");
const path = require("path");

const COMMENT_MARKER = "<!-- stellarforge-wasm-size-report -->";

/**
 * Normalizes a single contract entry from size-budget.json.
 *
 * @param {string} contractName
 * @param {number | { wasm?: string, max_bytes?: number, budget_bytes?: number, baseline_bytes?: number }} rawEntry
 */
function parseBudgetEntry(contractName, rawEntry) {
  const defaultWasm = `${contractName.replace(/-/g, "_")}.wasm`;
  if (typeof rawEntry === "number") {
    if (!Number.isInteger(rawEntry) || rawEntry <= 0) {
      throw new Error(
        `Invalid budget for "${contractName}": max_bytes must be a positive integer`,
      );
    }
    return {
      contract: contractName,
      wasm: defaultWasm,
      maxBytes: rawEntry,
      baselineBytes: null,
    };
  }

  if (typeof rawEntry !== "object" || rawEntry === null) {
    throw new Error(
      `Invalid budget entry for "${contractName}": expected an object or positive integer`,
    );
  }

  const maxBytes =
    rawEntry.max_bytes !== undefined ? rawEntry.max_bytes : rawEntry.budget_bytes;
  if (!Number.isInteger(maxBytes) || maxBytes <= 0) {
    throw new Error(
      `Invalid budget for "${contractName}": max_bytes must be a positive integer`,
    );
  }

  const baselineBytes =
    rawEntry.baseline_bytes !== undefined && rawEntry.baseline_bytes !== null
      ? rawEntry.baseline_bytes
      : null;
  if (
    baselineBytes !== null &&
    (!Number.isInteger(baselineBytes) || baselineBytes < 0)
  ) {
    throw new Error(
      `Invalid baseline_bytes for "${contractName}": must be a non-negative integer`,
    );
  }

  const wasm =
    typeof rawEntry.wasm === "string" && rawEntry.wasm.trim().length > 0
      ? rawEntry.wasm.trim()
      : defaultWasm;

  return {
    contract: contractName,
    wasm,
    maxBytes,
    baselineBytes,
  };
}

/**
 * Formats a byte delta with explicit sign and KB (1024-byte KiB) representation.
 *
 * @param {number | null} deltaBytes
 * @returns {string}
 */
function formatDelta(deltaBytes) {
  if (deltaBytes === null || deltaBytes === undefined) {
    return "n/a";
  }
  if (deltaBytes === 0) {
    return "0 B (0.00 KB)";
  }
  const sign = deltaBytes > 0 ? "+" : "-";
  const absBytes = Math.abs(deltaBytes);
  const absKb = (absBytes / 1024).toFixed(2);
  return `${sign}${absBytes.toLocaleString("en-US")} B (${sign}${absKb} KB)`;
}

/**
 * Evaluates WASM sizes for all contracts configured in the budget object.
 *
 * @param {Record<string, unknown>} budgetConfig
 * @param {string} wasmDir
 * @param {Record<string, unknown> | null} [baseConfig]
 */
function evaluateWasmSizes(budgetConfig, wasmDir, baseConfig = null) {
  if (
    typeof budgetConfig !== "object" ||
    budgetConfig === null ||
    Array.isArray(budgetConfig)
  ) {
    throw new Error(
      "Budget configuration must be a JSON object mapping contract names to budgets",
    );
  }

  const contractNames = Object.keys(budgetConfig);
  if (contractNames.length === 0) {
    throw new Error("Budget configuration contains no contracts");
  }

  const results = [];
  const errors = [];

  for (const contractName of contractNames) {
    const entry = parseBudgetEntry(contractName, budgetConfig[contractName]);
    const wasmPath = path.join(wasmDir, entry.wasm);

    if (!fs.existsSync(wasmPath)) {
      errors.push(
        `Missing WASM artifact for "${contractName}": expected ${wasmPath}`,
      );
      continue;
    }

    const stat = fs.statSync(wasmPath);
    const sizeBytes = stat.size;

    let baselineBytes = entry.baselineBytes;
    if (
      baseConfig &&
      Object.prototype.hasOwnProperty.call(baseConfig, contractName)
    ) {
      const baseItem = baseConfig[contractName];
      if (typeof baseItem === "number") {
        baselineBytes = baseItem;
      } else if (typeof baseItem === "object" && baseItem !== null) {
        if (typeof baseItem.size_bytes === "number") {
          baselineBytes = baseItem.size_bytes;
        } else if (typeof baseItem.baseline_bytes === "number") {
          baselineBytes = baseItem.baseline_bytes;
        }
      }
    }

    const deltaBytes =
      baselineBytes !== null && baselineBytes !== undefined
        ? sizeBytes - baselineBytes
        : null;
    const headroomBytes = entry.maxBytes - sizeBytes;
    const overBudget = sizeBytes > entry.maxBytes;

    if (overBudget) {
      errors.push(
        `Contract "${contractName}" (${entry.wasm}) is ${sizeBytes} bytes, exceeding its budget of ${entry.maxBytes} bytes by ${sizeBytes - entry.maxBytes} bytes`,
      );
    }

    results.push({
      contract: contractName,
      wasm: entry.wasm,
      sizeBytes,
      baselineBytes,
      deltaBytes,
      maxBytes: entry.maxBytes,
      headroomBytes,
      utilizationPct: Number(((sizeBytes / entry.maxBytes) * 100).toFixed(1)),
      overBudget,
    });
  }

  return {
    ok: errors.length === 0,
    results,
    errors,
  };
}

/**
 * Renders the evaluation result as a Markdown comment / summary table.
 *
 * @param {{ ok: boolean, results: Array<any>, errors: Array<string> }} evaluation
 * @returns {string}
 */
function renderMarkdownReport(evaluation) {
  const lines = [
    COMMENT_MARKER,
    "## Contract WASM Size Report",
    "",
    "| Contract | Artifact | Size (bytes) | Size (KB) | Delta | Budget (bytes) | Usage | Status |",
    "|---|---|---:|---:|---:|---:|---:|---|",
  ];

  for (const row of evaluation.results) {
    const sizeKb = (row.sizeBytes / 1024).toFixed(2);
    const deltaStr = formatDelta(row.deltaBytes);
    const statusStr = row.overBudget
      ? `FAIL (+${(row.sizeBytes - row.maxBytes).toLocaleString("en-US")} B over)`
      : "PASS";
    lines.push(
      `| \`${row.contract}\` | \`${row.wasm}\` | ${row.sizeBytes.toLocaleString("en-US")} | ${sizeKb} KB | ${deltaStr} | ${row.maxBytes.toLocaleString("en-US")} | ${row.utilizationPct.toFixed(1)}% | ${statusStr} |`,
    );
  }

  lines.push("");
  if (evaluation.ok) {
    lines.push(
      "All contracts are within their configured WASM size budgets (`contracts/size-budget.json`).",
    );
  } else {
    lines.push("### Budget Violations");
    lines.push("");
    for (const err of evaluation.errors) {
      lines.push(`- ${err}`);
    }
  }

  return lines.join("\n") + "\n";
}

/**
 * Posts or updates the WASM size report comment on a GitHub Pull Request.
 * Gracefully handles read-only GITHUB_TOKEN on fork pull requests.
 *
 * @param {string} markdown
 * @param {NodeJS.ProcessEnv} [env]
 * @param {typeof fetch} [fetchImpl]
 * @returns {Promise<{ posted: boolean, updated?: boolean, reason?: string }>}
 */
async function postOrUpdatePrComment(
  markdown,
  env = process.env,
  fetchImpl = globalThis.fetch,
) {
  const token = env.GITHUB_TOKEN;
  const repo = env.GITHUB_REPOSITORY;
  const eventPath = env.GITHUB_EVENT_PATH;
  const apiUrl = (env.GITHUB_API_URL || "https://api.github.com").replace(
    /\/$/,
    "",
  );

  if (!token || !repo || !eventPath || !fs.existsSync(eventPath)) {
    return { posted: false, reason: "missing-github-env" };
  }

  const event = JSON.parse(fs.readFileSync(eventPath, "utf8"));
  const prNumber = event.pull_request && event.pull_request.number;
  if (!prNumber) {
    return { posted: false, reason: "not-a-pull-request" };
  }

  const headers = {
    Authorization: `Bearer ${token}`,
    Accept: "application/vnd.github+json",
    "Content-Type": "application/json",
    "X-GitHub-Api-Version": "2022-11-28",
  };

  const listUrl = `${apiUrl}/repos/${repo}/issues/${prNumber}/comments?per_page=100`;
  const listRes = await fetchImpl(listUrl, { method: "GET", headers });
  if (!listRes.ok) {
    if (listRes.status === 401 || listRes.status === 403) {
      console.log(
        `Skipping PR comment: GITHUB_TOKEN lacks permission to list comments (HTTP ${listRes.status}).`,
      );
      return { posted: false, reason: "insufficient-permissions" };
    }
    throw new Error(`Failed to list PR comments: HTTP ${listRes.status}`);
  }

  const comments = await listRes.json();
  const existing = Array.isArray(comments)
    ? comments.find(
        (c) => typeof c.body === "string" && c.body.includes(COMMENT_MARKER),
      )
    : null;

  if (existing) {
    const updateUrl = `${apiUrl}/repos/${repo}/issues/comments/${existing.id}`;
    const updateRes = await fetchImpl(updateUrl, {
      method: "PATCH",
      headers,
      body: JSON.stringify({ body: markdown }),
    });
    if (!updateRes.ok) {
      if (updateRes.status === 401 || updateRes.status === 403) {
        console.log(
          `Skipping PR comment update: GITHUB_TOKEN is read-only for fork pull requests (HTTP ${updateRes.status}).`,
        );
        return { posted: false, reason: "insufficient-permissions" };
      }
      throw new Error(`Failed to update PR comment: HTTP ${updateRes.status}`);
    }
    return { posted: true, updated: true };
  }

  const createUrl = `${apiUrl}/repos/${repo}/issues/${prNumber}/comments`;
  const createRes = await fetchImpl(createUrl, {
    method: "POST",
    headers,
    body: JSON.stringify({ body: markdown }),
  });
  if (!createRes.ok) {
    if (createRes.status === 401 || createRes.status === 403) {
      console.log(
        `Skipping PR comment creation: GITHUB_TOKEN is read-only for fork pull requests (HTTP ${createRes.status}).`,
      );
      return { posted: false, reason: "insufficient-permissions" };
    }
    throw new Error(`Failed to create PR comment: HTTP ${createRes.status}`);
  }

  return { posted: true, updated: false };
}

function parseArgs(argv) {
  const root = path.join(__dirname, "..");
  const opts = {
    budgetPath: path.join(root, "contracts", "size-budget.json"),
    wasmDir: path.join(root, "target", "wasm32v1-none", "release"),
    basePath: null,
    outputMarkdown: null,
    outputJson: null,
    postPrComment: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--budget" && i + 1 < argv.length) {
      opts.budgetPath = path.resolve(argv[++i]);
    } else if (arg === "--wasm-dir" && i + 1 < argv.length) {
      opts.wasmDir = path.resolve(argv[++i]);
    } else if (arg === "--base" && i + 1 < argv.length) {
      opts.basePath = path.resolve(argv[++i]);
    } else if (arg === "--output-markdown" && i + 1 < argv.length) {
      opts.outputMarkdown = path.resolve(argv[++i]);
    } else if (arg === "--output-json" && i + 1 < argv.length) {
      opts.outputJson = path.resolve(argv[++i]);
    } else if (arg === "--post-pr-comment") {
      opts.postPrComment = true;
    } else {
      throw new Error(`Unknown or incomplete argument: ${arg}`);
    }
  }

  return opts;
}

async function main(argv = process.argv.slice(2)) {
  const opts = parseArgs(argv);
  const budgetConfig = JSON.parse(fs.readFileSync(opts.budgetPath, "utf8"));
  const baseConfig =
    opts.basePath && fs.existsSync(opts.basePath)
      ? JSON.parse(fs.readFileSync(opts.basePath, "utf8"))
      : null;

  const evaluation = evaluateWasmSizes(budgetConfig, opts.wasmDir, baseConfig);
  const markdown = renderMarkdownReport(evaluation);

  process.stdout.write(markdown);

  if (process.env.GITHUB_STEP_SUMMARY) {
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown, "utf8");
  }

  if (opts.outputMarkdown) {
    fs.writeFileSync(opts.outputMarkdown, markdown, "utf8");
  }

  if (opts.outputJson) {
    fs.writeFileSync(
      opts.outputJson,
      JSON.stringify(evaluation, null, 2) + "\n",
      "utf8",
    );
  }

  if (opts.postPrComment) {
    await postOrUpdatePrComment(markdown);
  }

  if (!evaluation.ok) {
    for (const err of evaluation.errors) {
      console.error(`error: ${err}`);
    }
    process.exit(1);
  }
}

if (require.main === module) {
  main().catch((err) => {
    console.error(`error: ${err.message}`);
    process.exit(1);
  });
}

module.exports = {
  COMMENT_MARKER,
  parseBudgetEntry,
  formatDelta,
  evaluateWasmSizes,
  renderMarkdownReport,
  postOrUpdatePrComment,
};
