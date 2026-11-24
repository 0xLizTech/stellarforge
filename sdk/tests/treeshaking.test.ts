import path from "path";
import * as esbuild from "esbuild";
import { describe, expect, it } from "vitest";

const SDK_DIR = path.resolve(__dirname, "..");
const DIST_DIR = path.join(SDK_DIR, "dist");
const EXTERNAL_MODULES = ["@stellar/stellar-sdk", "crypto", "node:crypto"];

describe("SDK Tree-Shaking (#183)", () => {
  it("bundles only RwaAssetClient from root index omitting GovernanceClient and other clients", async () => {
    const entryCode = `
      import { RwaAssetClient } from "${path.join(DIST_DIR, "index.mjs")}";
      console.log(RwaAssetClient);
    `;

    const result = await esbuild.build({
      stdin: {
        contents: entryCode,
        resolveDir: SDK_DIR,
        sourcefile: "virtual-entry-rwa.js",
        loader: "js",
      },
      bundle: true,
      format: "esm",
      treeShaking: true,
      write: false,
      external: EXTERNAL_MODULES,
    });

    const outputText = result.outputFiles?.[0]?.text ?? "";

    // RwaAssetClient should be present
    expect(outputText).toContain("RwaAssetClient");

    // Other clients and their domain-specific methods should be omitted
    expect(outputText).not.toContain("GovernanceClient");
    expect(outputText).not.toContain("VaultClient");
    expect(outputText).not.toContain("OracleAdapterClient");
    expect(outputText).not.toContain("RegistryClient");
    expect(outputText).not.toContain("sharesForDeposit");
    expect(outputText).not.toContain("underlyingForRedeem");
  });

  it("bundles only GovernanceClient from subpath omitting RwaAssetClient, VaultClient, and Oracle", async () => {
    const entryCode = `
      import { GovernanceClient } from "${path.join(DIST_DIR, "governance.mjs")}";
      console.log(GovernanceClient);
    `;

    const result = await esbuild.build({
      stdin: {
        contents: entryCode,
        resolveDir: SDK_DIR,
        sourcefile: "virtual-entry-gov.js",
        loader: "js",
      },
      bundle: true,
      format: "esm",
      treeShaking: true,
      write: false,
      external: EXTERNAL_MODULES,
    });

    const outputText = result.outputFiles?.[0]?.text ?? "";

    expect(outputText).toContain("GovernanceClient");
    expect(outputText).not.toContain("RwaAssetClient");
    expect(outputText).not.toContain("VaultClient");
    expect(outputText).not.toContain("OracleAdapterClient");
    expect(outputText).not.toContain("sharesForDeposit");
  });

  it("bundles only pure math utilities from utils omitting all contract clients", async () => {
    const entryCode = `
      import { toStroops, fromStroops } from "${path.join(DIST_DIR, "utils.mjs")}";
      console.log(toStroops, fromStroops);
    `;

    const result = await esbuild.build({
      stdin: {
        contents: entryCode,
        resolveDir: SDK_DIR,
        sourcefile: "virtual-entry-utils.js",
        loader: "js",
      },
      bundle: true,
      format: "esm",
      treeShaking: true,
      write: false,
      external: EXTERNAL_MODULES,
    });

    const outputText = result.outputFiles?.[0]?.text ?? "";

    expect(outputText).toContain("toStroops");
    expect(outputText).toContain("fromStroops");
    expect(outputText).not.toContain("RwaAssetClient");
    expect(outputText).not.toContain("GovernanceClient");
    expect(outputText).not.toContain("VaultClient");
    expect(outputText).not.toContain("OracleAdapterClient");
    expect(outputText).not.toContain("ContractClient");
  });
});
