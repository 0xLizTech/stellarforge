import { describe, it, expect, vi, afterEach } from "vitest";
import { StrKey, rpc, xdr } from "@stellar/stellar-sdk";

import { REGISTRY_MAX_PAGE_SIZE, RegistryClient } from "../src/registry.js";
import { TESTNET_CONFIG } from "../src/types.js";
import type { AssetEntry, StellarForgeConfig } from "../src/types.js";
import {
  REGISTRY_ID,
  RWA_ID,
  COMPLIANCE_ID,
  SIGNER as ADMIN,
  SIGNER_SECRET as ADMIN_SECRET,
  OTHER_KP,
  SUBMITTED_HASH,
  configWith,
  stubSimulation,
  stubWritePath,
  succeeds,
  fails,
  succeedsWithoutResult,
  invocation,
  builtInvocation,
  addr,
  bool,
  str,
  struct,
  none,
  u32,
} from "./helpers.js";

const registryConfig: StellarForgeConfig = configWith({ registry: REGISTRY_ID });
const signerConfig: StellarForgeConfig = configWith({ registry: REGISTRY_ID }, ADMIN_SECRET);

function entryScVal(overrides: Record<string, xdr.ScVal> = {}): xdr.ScVal {
  return struct({
    contract: addr(RWA_ID),
    asset_class: str("real_estate"),
    active: bool(true),
    ...overrides,
  });
}

const SAMPLE_ENTRY: AssetEntry = {
  contract: RWA_ID,
  assetClass: "real_estate",
  active: true,
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RegistryClient constructor", () => {
  it("rejects a config with no registry address", () => {
    expect(() => new RegistryClient({ ...TESTNET_CONFIG, contracts: {} })).toThrow(
      "contracts.registry address is required",
    );
  });

  it("accepts a valid contract id", () => {
    expect(() => new RegistryClient(registryConfig)).not.toThrow();
  });
});

describe("RegistryClient.getAsset", () => {
  it("maps the contract struct onto camelCase", async () => {
    stubSimulation(succeeds(entryScVal()));
    await expect(new RegistryClient(registryConfig).getAsset(RWA_ID)).resolves.toEqual({
      contract: RWA_ID,
      assetClass: "real_estate",
      active: true,
    });
  });

  it("carries an inactive flag through rather than dropping the entry", async () => {
    stubSimulation(succeeds(entryScVal({ active: bool(false) })));
    const entry = await new RegistryClient(registryConfig).getAsset(RWA_ID);
    expect(entry?.active).toBe(false);
  });

  // get_asset returns Option<AssetEntry>. None decodes to null, and an unknown
  // address is an ordinary answer rather than a failure.
  it("returns null for an address that was never registered", async () => {
    stubSimulation(succeeds(none()));
    await expect(new RegistryClient(registryConfig).getAsset(COMPLIANCE_ID)).resolves.toBeNull();
  });

  it("invokes `get_asset` with the queried address", async () => {
    const spy = stubSimulation(succeeds(none()));
    await new RegistryClient(registryConfig).getAsset(RWA_ID);
    expect(invocation(spy)).toEqual({ fn: "get_asset", args: [RWA_ID] });
  });
});

describe("RegistryClient.listAssets", () => {
  it("decodes a vector of addresses", async () => {
    stubSimulation(succeeds(xdr.ScVal.scvVec([addr(RWA_ID), addr(COMPLIANCE_ID)])));
    await expect(new RegistryClient(registryConfig).listAssets()).resolves.toEqual([
      RWA_ID,
      COMPLIANCE_ID,
    ]);
  });

  it("decodes an empty registry as an empty array", async () => {
    stubSimulation(succeeds(xdr.ScVal.scvVec([])));
    await expect(new RegistryClient(registryConfig).listAssets()).resolves.toEqual([]);
  });

  it("requests the first full page by default", async () => {
    const spy = stubSimulation(succeeds(xdr.ScVal.scvVec([])));
    await new RegistryClient(registryConfig).listAssets();
    expect(invocation(spy)).toEqual({
      fn: "list_assets",
      args: [0, REGISTRY_MAX_PAGE_SIZE],
    });
  });

  it("passes an explicit window through", async () => {
    const spy = stubSimulation(succeeds(xdr.ScVal.scvVec([])));
    await new RegistryClient(registryConfig).listAssets(200, 25);
    expect(invocation(spy)).toEqual({ fn: "list_assets", args: [200, 25] });
  });
});

describe("RegistryClient.assetCount", () => {
  it("decodes the count", async () => {
    stubSimulation(succeeds(u32(1_742)));
    await expect(new RegistryClient(registryConfig).assetCount()).resolves.toBe(1_742);
  });
});

describe("RegistryClient.listAllAssets", () => {
  function page(size: number, offset: number): xdr.ScVal {
    return xdr.ScVal.scvVec(
      Array.from({ length: size }, (_, i) =>
        addr(StrKey.encodeContract(Buffer.alloc(32, (offset + i) % 256))),
      ),
    );
  }

  it("walks pages until one comes back short", async () => {
    const spy = vi
      .spyOn(rpc.Server.prototype, "simulateTransaction")
      .mockResolvedValueOnce(succeeds(page(REGISTRY_MAX_PAGE_SIZE, 0)) as never)
      .mockResolvedValueOnce(succeeds(page(REGISTRY_MAX_PAGE_SIZE, 100)) as never)
      .mockResolvedValueOnce(succeeds(page(3, 200)) as never);

    const all = await new RegistryClient(registryConfig).listAllAssets();

    expect(all).toHaveLength(2 * REGISTRY_MAX_PAGE_SIZE + 3);
    expect(spy).toHaveBeenCalledTimes(3);
  });

  it("stops after one call for an empty registry", async () => {
    const spy = stubSimulation(succeeds(xdr.ScVal.scvVec([])));
    await expect(new RegistryClient(registryConfig).listAllAssets()).resolves.toEqual([]);
    expect(spy).toHaveBeenCalledTimes(1);
  });
});

describe("RegistryClient.admin", () => {
  it("decodes the admin address", async () => {
    stubSimulation(succeeds(addr(ADMIN)));
    await expect(new RegistryClient(registryConfig).admin()).resolves.toBe(ADMIN);
  });
});

describe("RegistryClient read failures", () => {
  it("surfaces the simulation error message", async () => {
    stubSimulation(fails("HostError: Error(Contract, #3)"));
    await expect(new RegistryClient(registryConfig).getAsset(RWA_ID)).rejects.toThrow(
      "Simulation error: HostError: Error(Contract, #3)",
    );
  });

  it("names the method when the simulation returns no result", async () => {
    stubSimulation(succeedsWithoutResult());
    await expect(new RegistryClient(registryConfig).listAssets()).rejects.toThrow(
      "No result returned from list_assets",
    );
  });
});

describe("RegistryClient write builders", () => {
  // The contract takes an AssetEntry struct, so the encoding has to round-trip
  // back to the same fields the caller supplied.
  it("encodes the asset entry as a struct `register` can decode", async () => {
    stubWritePath();
    const tx = await new RegistryClient(registryConfig).buildRegisterTx(ADMIN, SAMPLE_ENTRY);
    const { fn, args } = builtInvocation(tx);

    expect(fn).toBe("register");
    expect(args).toEqual([
      { contract: RWA_ID, asset_class: "real_estate", active: true },
    ]);
  });

  it("preserves an inactive entry through encoding", async () => {
    stubWritePath();
    const tx = await new RegistryClient(registryConfig).buildRegisterTx(ADMIN, {
      ...SAMPLE_ENTRY,
      active: false,
    });
    expect(builtInvocation(tx).args).toEqual([
      { contract: RWA_ID, asset_class: "real_estate", active: false },
    ]);
  });

  it("invokes `set_active` with the address before the flag", async () => {
    stubWritePath();
    const tx = await new RegistryClient(registryConfig).buildSetActiveTx(ADMIN, RWA_ID, false);
    expect(builtInvocation(tx)).toEqual({ fn: "set_active", args: [RWA_ID, false] });
  });

  it.each([
    { name: "register", build: (c: RegistryClient) => c.buildRegisterTx(ADMIN, SAMPLE_ENTRY) },
    { name: "setActive", build: (c: RegistryClient) => c.buildSetActiveTx(ADMIN, RWA_ID, true) },
  ])("$name sources the transaction from the admin", async ({ build }) => {
    const stubs = stubWritePath();
    const tx = await build(new RegistryClient(registryConfig));
    expect(tx.source).toBe(ADMIN);
    expect(stubs.getAccount).toHaveBeenCalledWith(ADMIN);
  });

  it("builds without a signerSecret and leaves the transaction unsigned", async () => {
    stubWritePath();
    const tx = await new RegistryClient(registryConfig).buildRegisterTx(ADMIN, SAMPLE_ENTRY);
    expect(registryConfig.signerSecret).toBeUndefined();
    expect(tx.signatures).toHaveLength(0);
  });

  // set_active panics with AssetNotFound for an unregistered asset, and
  // prepareTransaction simulates, so that surfaces before anything is signed.
  it("surfaces a rejection raised while preparing", async () => {
    const stubs = stubWritePath();
    stubs.prepare.mockRejectedValue(new Error("HostError: Error(Contract, #3)"));
    await expect(
      new RegistryClient(registryConfig).buildSetActiveTx(ADMIN, RWA_ID, true),
    ).rejects.toThrow("HostError: Error(Contract, #3)");
  });
});

describe("RegistryClient write submission", () => {
  it.each([
    { name: "register", call: (c: RegistryClient) => c.register(ADMIN, SAMPLE_ENTRY) },
    { name: "setActive", call: (c: RegistryClient) => c.setActive(ADMIN, RWA_ID, false) },
  ])("$name reports the settled transaction", async ({ call }) => {
    stubWritePath();
    await expect(call(new RegistryClient(signerConfig))).resolves.toEqual({
      hash: SUBMITTED_HASH,
      ledger: 42,
    });
  });

  it("refuses to submit without a signerSecret", async () => {
    stubWritePath();
    await expect(
      new RegistryClient(registryConfig).register(ADMIN, SAMPLE_ENTRY),
    ).rejects.toThrow(/signerSecret is required/);
  });

  it("refuses when the signer is not the admin sourcing the transaction", async () => {
    const stubs = stubWritePath();
    const wrong = configWith({ registry: REGISTRY_ID }, OTHER_KP.secret());
    await expect(new RegistryClient(wrong).register(ADMIN, SAMPLE_ENTRY)).rejects.toThrow(
      /must be authorized by/,
    );
    expect(stubs.send).not.toHaveBeenCalled();
  });

  it("throws when the transaction settles as FAILED", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.FAILED,
        txHash: SUBMITTED_HASH,
        ledger: 42,
      },
    });
    await expect(
      new RegistryClient(signerConfig).setActive(ADMIN, RWA_ID, true),
    ).rejects.toThrow(/did not succeed: FAILED/);
  });
});
