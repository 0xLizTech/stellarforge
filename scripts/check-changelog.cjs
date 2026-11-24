#!/usr/bin/env node
"use strict";

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");

const base = process.argv[2];
if (!base) {
  console.error("Usage: node scripts/check-changelog.cjs <base-sha>");
  process.exit(2);
}

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" });
}

const changedFiles = git(
  "diff",
  "--name-only",
  "--diff-filter=ACMRTD",
  base,
)
  .split(/\r?\n/)
  .filter(Boolean);
const codeExtensions = /\.(?:rs|ts|tsx|js|jsx|mjs|cjs|sh|toml|json|ya?ml|html|css|scss)$/i;
const hasCodeChanges = changedFiles.some(
  (file) =>
    codeExtensions.test(file) ||
    /(^|\/)(?:Cargo\.lock|Dockerfile|Makefile)$/.test(file),
);

if (!hasCodeChanges) {
  console.log("No code changes; changelog entry not required.");
  process.exit(0);
}

function unreleasedEntries(content) {
  const lines = content
    .replace(/<!--[\s\S]*?-->/g, "")
    .split(/\r?\n/);
  const start = lines.findIndex((line) => /^##\s+\[Unreleased\]\s*$/i.test(line));
  if (start === -1) {
    throw new Error("CHANGELOG.md has no ## [Unreleased] section.");
  }

  const categories = new Set([
    "Added",
    "Changed",
    "Deprecated",
    "Removed",
    "Fixed",
    "Security",
  ]);
  const entries = [];
  let category = null;

  for (const line of lines.slice(start + 1)) {
    if (/^##\s+\[/.test(line)) break;
    const heading = line.match(/^###\s+(.+?)\s*$/);
    if (heading) {
      category = categories.has(heading[1]) ? heading[1] : null;
      continue;
    }
    if (category && /^\s*-\s+\S/.test(line)) {
      entries.push(`${category}: ${line.trim()}`);
    }
  }

  return entries;
}

try {
  const baseEntries = unreleasedEntries(git("show", `${base}:CHANGELOG.md`));
  const headEntries = unreleasedEntries(fs.readFileSync("CHANGELOG.md", "utf8"));
  const baseCounts = new Map();
  for (const entry of baseEntries) {
    baseCounts.set(entry, (baseCounts.get(entry) || 0) + 1);
  }

  const addedEntry = headEntries.some((entry) => {
    const count = baseCounts.get(entry) || 0;
    if (count > 0) {
      baseCounts.set(entry, count - 1);
      return false;
    }
    return true;
  });

  if (!addedEntry) {
    throw new Error(
      "Code changes require a new bullet under a recognized category in CHANGELOG.md's [Unreleased] section.",
    );
  }
  console.log("Found a new categorized entry under CHANGELOG.md [Unreleased].");
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
