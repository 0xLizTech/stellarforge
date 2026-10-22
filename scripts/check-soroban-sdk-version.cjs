#!/usr/bin/env node
/**
 * Verifies that `soroban-sdk` is pinned once in the root `[workspace.dependencies]`
 * and that every workspace member crate inherits it via `workspace = true` without
 * per-crate version, path, or git overrides (#95).
 *
 * Usage:
 *   node scripts/check-soroban-sdk-version.cjs [--root <workspace-root>]
 */
"use strict";

const fs = require("fs");
const path = require("path");

/**
 * Strips TOML comments from a line while respecting double- and single-quoted strings.
 *
 * @param {string} line
 * @returns {string}
 */
function stripTomlComment(line) {
  let inSingle = false;
  let inDouble = false;
  for (let i = 0; i < line.length; i += 1) {
    const ch = line[i];
    if (ch === '"' && !inSingle && line[i - 1] !== "\\") {
      inDouble = !inDouble;
    } else if (ch === "'" && !inDouble) {
      inSingle = !inSingle;
    } else if (ch === "#" && !inSingle && !inDouble) {
      return line.slice(0, i);
    }
  }
  return line;
}

/**
 * Parses a Cargo.toml file into section headers and key-value assignments,
 * coalescing multi-line arrays and inline tables.
 *
 * @param {string} content
 * @returns {Array<{ section: string, key: string, rawValue: string, line: number }>}
 */
function parseTomlEntries(content) {
  const rawLines = content.split(/\r?\n/);
  const entries = [];
  let currentSection = "";
  let i = 0;

  while (i < rawLines.length) {
    const lineNum = i + 1;
    const cleaned = stripTomlComment(rawLines[i]).trim();
    i += 1;

    if (!cleaned) {
      continue;
    }

    const sectionMatch = cleaned.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      currentSection = sectionMatch[1].trim();
      continue;
    }

    const eqIdx = cleaned.indexOf("=");
    if (eqIdx === -1) {
      continue;
    }

    const key = cleaned.slice(0, eqIdx).trim();
    let valueAccum = cleaned.slice(eqIdx + 1).trim();

    // Balance brackets/braces across multiple lines if needed
    const countDelims = (str) => {
      let sq = 0;
      let cu = 0;
      let inS = false;
      let inD = false;
      for (let j = 0; j < str.length; j += 1) {
        const c = str[j];
        if (c === '"' && !inS && str[j - 1] !== "\\") inD = !inD;
        else if (c === "'" && !inD) inS = !inS;
        else if (!inS && !inD) {
          if (c === "[") sq += 1;
          else if (c === "]") sq -= 1;
          else if (c === "{") cu += 1;
          else if (c === "}") cu -= 1;
        }
      }
      return { sq, cu };
    };

    let delims = countDelims(valueAccum);
    while ((delims.sq > 0 || delims.cu > 0) && i < rawLines.length) {
      const nextCleaned = stripTomlComment(rawLines[i]).trim();
      i += 1;
      if (nextCleaned) {
        valueAccum += " " + nextCleaned;
        delims = countDelims(valueAccum);
      }
    }

    entries.push({
      section: currentSection,
      key,
      rawValue: valueAccum,
      line: lineNum,
    });
  }

  return entries;
}

/**
 * Parses an inline TOML table `{ k = v, ... }` into a key-value map.
 *
 * @param {string} rawValue
 * @returns {Record<string, string> | null}
 */
function parseInlineTable(rawValue) {
  const trimmed = rawValue.trim();
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) {
    return null;
  }
  const inner = trimmed.slice(1, -1).trim();
  const result = {};
  if (!inner) {
    return result;
  }

  // Split by top-level commas
  const parts = [];
  let current = "";
  let depth = 0;
  let inSingle = false;
  let inDouble = false;
  for (let i = 0; i < inner.length; i += 1) {
    const ch = inner[i];
    if (ch === '"' && !inSingle && inner[i - 1] !== "\\") inDouble = !inDouble;
    else if (ch === "'" && !inDouble) inSingle = !inSingle;
    else if (!inSingle && !inDouble) {
      if (ch === "[" || ch === "{") depth += 1;
      else if (ch === "]" || ch === "}") depth -= 1;
      else if (ch === "," && depth === 0) {
        parts.push(current.trim());
        current = "";
        continue;
      }
    }
    current += ch;
  }
  if (current.trim()) {
    parts.push(current.trim());
  }

  for (const part of parts) {
    const eq = part.indexOf("=");
    if (eq !== -1) {
      const k = part.slice(0, eq).trim();
      const v = part.slice(eq + 1).trim();
      result[k] = v;
    }
  }

  return result;
}

/**
 * Checks whether a TOML section is a dependency section in a member crate.
 *
 * @param {string} section
 * @returns {boolean}
 */
function isDependencySection(section) {
  if (
    section === "dependencies" ||
    section === "dev-dependencies" ||
    section === "build-dependencies"
  ) {
    return true;
  }
  return /^target\.[^.]+\.(dependencies|dev-dependencies|build-dependencies)$/.test(
    section,
  );
}

/**
 * Checks whether a TOML section is a sub-table for a specific dependency,
 * e.g. `[dependencies.soroban-sdk]` or `[dev-dependencies.soroban-sdk]`.
 *
 * @param {string} section
 * @returns {{ tableType: string, depName: string } | null}
 */
function parseDependencySubtable(section) {
  const direct = section.match(
    /^(dependencies|dev-dependencies|build-dependencies)\.(.+)$/,
  );
  if (direct) {
    return { tableType: direct[1], depName: direct[2].trim() };
  }
  const targetMatch = section.match(
    /^target\.[^.]+\.(dependencies|dev-dependencies|build-dependencies)\.(.+)$/,
  );
  if (targetMatch) {
    return { tableType: targetMatch[1], depName: targetMatch[2].trim() };
  }
  return null;
}

/**
 * Validates the root Cargo.toml and all workspace member Cargo.toml files
 * to ensure `soroban-sdk` is centrally managed in `[workspace.dependencies]`
 * and never overridden per crate.
 *
 * @param {string} rootDir
 * @returns {{ ok: boolean, workspaceVersion: string | null, checkedMembers: string[], errors: string[] }}
 */
function checkWorkspaceSorobanSdk(rootDir) {
  const errors = [];
  const rootManifestPath = path.join(rootDir, "Cargo.toml");

  if (!fs.existsSync(rootManifestPath)) {
    return {
      ok: false,
      workspaceVersion: null,
      checkedMembers: [],
      errors: [`Missing root Cargo.toml at ${rootManifestPath}`],
    };
  }

  const rootContent = fs.readFileSync(rootManifestPath, "utf8");
  const rootEntries = parseTomlEntries(rootContent);

  // 1. Extract workspace members
  const membersEntry = rootEntries.find(
    (e) => e.section === "workspace" && e.key === "members",
  );
  if (!membersEntry) {
    errors.push("Root Cargo.toml is missing [workspace] members list");
  }

  const members = [];
  if (membersEntry) {
    const strMatches = membersEntry.rawValue.matchAll(/"([^"]+)"|'([^']+)'/g);
    for (const m of strMatches) {
      members.push(m[1] || m[2]);
    }
  }

  // 2. Check [workspace.dependencies] for soroban-sdk
  let workspaceVersion = null;
  const wsSdkEntry = rootEntries.find(
    (e) =>
      (e.section === "workspace.dependencies" && e.key === "soroban-sdk") ||
      (e.section === "workspace.dependencies.soroban-sdk" &&
        e.key === "version"),
  );

  if (!wsSdkEntry) {
    errors.push(
      "Root Cargo.toml must define `soroban-sdk` under `[workspace.dependencies]`",
    );
  } else if (wsSdkEntry.section === "workspace.dependencies") {
    const raw = wsSdkEntry.rawValue;
    if (/^["'][^"']+["']$/.test(raw)) {
      workspaceVersion = raw.slice(1, -1);
    } else {
      const table = parseInlineTable(raw);
      if (table && table.version && /^["'][^"']+["']$/.test(table.version)) {
        workspaceVersion = table.version.slice(1, -1);
      }
    }
    if (!workspaceVersion || workspaceVersion === "*") {
      errors.push(
        `Root Cargo.toml [workspace.dependencies] soroban-sdk must pin an explicit version (found: ${raw})`,
      );
    }
  } else {
    const raw = wsSdkEntry.rawValue;
    if (/^["'][^"']+["']$/.test(raw)) {
      workspaceVersion = raw.slice(1, -1);
    }
    if (!workspaceVersion || workspaceVersion === "*") {
      errors.push(
        `Root Cargo.toml [workspace.dependencies.soroban-sdk] must pin an explicit version (found: ${raw})`,
      );
    }
  }

  // 3. Check each member crate's Cargo.toml
  const checkedMembers = [];
  for (const memberRelPath of members) {
    const memberManifestPath = path.join(rootDir, memberRelPath, "Cargo.toml");
    if (!fs.existsSync(memberManifestPath)) {
      errors.push(
        `Workspace member "${memberRelPath}" is missing Cargo.toml at ${memberManifestPath}`,
      );
      continue;
    }

    checkedMembers.push(memberRelPath);
    const memberContent = fs.readFileSync(memberManifestPath, "utf8");
    const memberEntries = parseTomlEntries(memberContent);

    // Track subtable entries like [dependencies.soroban-sdk]
    const subtables = new Map();

    for (const entry of memberEntries) {
      // Case A: Entry inside [dependencies], [dev-dependencies], etc.
      if (isDependencySection(entry.section)) {
        // Dotted key e.g. `soroban-sdk.version = "27.0.5"` or `soroban-sdk.workspace = true`
        if (entry.key.startsWith("soroban-sdk.")) {
          const subKey = entry.key.slice("soroban-sdk.".length).trim();
          const mapKey = `${entry.section}::soroban-sdk`;
          if (!subtables.has(mapKey)) {
            subtables.set(mapKey, {
              section: entry.section,
              line: entry.line,
              props: {},
            });
          }
          subtables.get(mapKey).props[subKey] = entry.rawValue;
          continue;
        }

        const inlineTable = parseInlineTable(entry.rawValue);
        const isSorobanSdkAlias =
          inlineTable &&
          inlineTable.package &&
          inlineTable.package.replace(/["']/g, "") === "soroban-sdk";

        if (entry.key === "soroban-sdk" || isSorobanSdkAlias) {
          if (!inlineTable) {
            errors.push(
              `${memberRelPath}/Cargo.toml:${entry.line} [${entry.section}] pins "${entry.key} = ${entry.rawValue}" directly; must use "{ workspace = true }"`,
            );
            continue;
          }

          if (inlineTable.workspace !== "true") {
            errors.push(
              `${memberRelPath}/Cargo.toml:${entry.line} [${entry.section}] "${entry.key}" must set "workspace = true"`,
            );
          }

          for (const forbidden of ["version", "git", "path", "rev", "branch", "tag"]) {
            if (Object.prototype.hasOwnProperty.call(inlineTable, forbidden)) {
              errors.push(
                `${memberRelPath}/Cargo.toml:${entry.line} [${entry.section}] "${entry.key}" overrides "${forbidden} = ${inlineTable[forbidden]}"; version/source must only be set in root [workspace.dependencies]`,
              );
            }
          }
        }
      }

      // Case B: Subtable section e.g. [dependencies.soroban-sdk]
      const sub = parseDependencySubtable(entry.section);
      if (sub) {
        const mapKey = `${sub.tableType}::${sub.depName}`;
        if (!subtables.has(mapKey)) {
          subtables.set(mapKey, {
            section: entry.section,
            depName: sub.depName,
            line: entry.line,
            props: {},
          });
        }
        subtables.get(mapKey).props[entry.key] = entry.rawValue;
      }
    }

    for (const [, info] of subtables.entries()) {
      const isSoroban =
        info.depName === "soroban-sdk" ||
        info.section.endsWith("soroban-sdk") ||
        (info.props.package &&
          info.props.package.replace(/["']/g, "") === "soroban-sdk");

      if (!isSoroban) {
        continue;
      }

      if (info.props.workspace !== "true") {
        errors.push(
          `${memberRelPath}/Cargo.toml:${info.line} [${info.section}] must set "workspace = true"`,
        );
      }

      for (const forbidden of ["version", "git", "path", "rev", "branch", "tag"]) {
        if (Object.prototype.hasOwnProperty.call(info.props, forbidden)) {
          errors.push(
            `${memberRelPath}/Cargo.toml:${info.line} [${info.section}] overrides "${forbidden} = ${info.props[forbidden]}"; version/source must only be set in root [workspace.dependencies]`,
          );
        }
      }
    }
  }

  // 4. Verify Cargo.lock (if present) does not resolve multiple versions of soroban-sdk
  const lockfilePath = path.join(rootDir, "Cargo.lock");
  if (fs.existsSync(lockfilePath)) {
    const lockContent = fs.readFileSync(lockfilePath, "utf8");
    const sdkVersions = new Set();
    const pkgBlocks = lockContent.split("[[package]]");
    for (const block of pkgBlocks.slice(1)) {
      const nameMatch = block.match(/^\s*name\s*=\s*"([^"]+)"/m);
      const versionMatch = block.match(/^\s*version\s*=\s*"([^"]+)"/m);
      if (nameMatch && nameMatch[1] === "soroban-sdk" && versionMatch) {
        sdkVersions.add(versionMatch[1]);
      }
    }
    if (sdkVersions.size > 1) {
      errors.push(
        `Cargo.lock resolves multiple versions of soroban-sdk: ${[...sdkVersions].join(", ")}`,
      );
    }
  }

  return {
    ok: errors.length === 0,
    workspaceVersion,
    checkedMembers,
    errors,
  };
}

function main(argv = process.argv.slice(2)) {
  let rootDir = path.join(__dirname, "..");
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--root" && i + 1 < argv.length) {
      rootDir = path.resolve(argv[++i]);
    } else {
      throw new Error(`Unknown or incomplete argument: ${argv[i]}`);
    }
  }

  const result = checkWorkspaceSorobanSdk(rootDir);
  if (!result.ok) {
    for (const err of result.errors) {
      console.error(`error: ${err}`);
    }
    process.exit(1);
  }

  console.log(
    `ok: soroban-sdk (${result.workspaceVersion}) is consistently inherited via [workspace.dependencies] across ${result.checkedMembers.length} workspace members`,
  );
}

if (require.main === module) {
  try {
    main();
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(1);
  }
}

module.exports = {
  stripTomlComment,
  parseTomlEntries,
  parseInlineTable,
  checkWorkspaceSorobanSdk,
};
