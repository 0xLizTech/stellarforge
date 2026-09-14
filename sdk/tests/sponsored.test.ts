import { describe, it, expect, vi, afterEach } from "vitest";
import {
  Account,
  Address,
  Keypair,
  nativeToScVal,
  Networks,
  rpc,
  SorobanDataBuilder,
  StrKey,
  xdr,
} from "@stellar/stellar-sdk";
import type { MockInstance } from "vitest";

import { RwaAssetClient } from "../src/client.js";
import { RegistryClient } from "../src/registry.js";
import {
  DEFAULT_AUTH_VALIDITY_LEDGERS,
  MAX_AUTH_VALIDITY_LEDGERS,
  MIN_AUTH_REMAINING_LEDGERS,
  authEntriesFromXdr,
  authEntriesToXdr,
  authEntryAddress,
  authorizeEntries,
} from "../src/sponsored.js";
import {
  RWA_ID,
  REGISTRY_ID,
  SIGNER as FEE_PAYER,
  SIGNER_SECRET as FEE_PAYER_SECRET,
  SUBMITTED_HASH,
  builtInvocation,
  configWith,
} from "./helpers.js";

const LATEST_LEDGER = 1_000;
const AUTHORIZER = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 21));
const NEW_ADMIN = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 22));
const RECIPIENT = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 23));

const rwaConfig = configWith({ rwaAsset: RWA_ID });

/** An unsigned CAP-71 entry for `address`, as simulation returns one. */
function unsignedEntry(address: string, fn = "transfer", nonce = 7n): xdr.SorobanAuthorizationEntry {
  return new xdr.SorobanAuthorizationEntry({
    credentials: xdr.SorobanCredentials.sorobanCredentialsAddressV2(
      new xdr.SorobanAddressCredentials({
        address: new Address(address).toScAddress(),
        nonce,
        signatureExpirationLedger: 0,
        signature: xdr.ScVal.scvVoid(),
      }),
    ),
    rootInvocation: new xdr.SorobanAuthorizedInvocation({
      function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(
        new xdr.InvokeContractArgs({
          contractAddress: new Address(RWA_ID).toScAddress(),
          functionName: fn,
          args: [nativeToScVal(1n, { type: "i128" })],
        }),
      ),
      subInvocations: [],
    }),
  });
}

/** An entry the transaction source's own signature covers. */
function sourceAccountEntry(): xdr.SorobanAuthorizationEntry {
  const base = unsignedEntry(FEE_PAYER, "transfer_admin");
  return new xdr.SorobanAuthorizationEntry({
    credentials: xdr.SorobanCredentials.sorobanCredentialsSourceAccount(),
    rootInvocation: base.rootInvocation,
  });
}

function simulationWith(auth: xdr.SorobanAuthorizationEntry[]): rpc.Api.SimulateTransactionResponse {
  return {
    _parsed: true,
    id: "1",
    latestLedger: LATEST_LEDGER,
    minResourceFee: "5000",
    transactionData: new SorobanDataBuilder(),
    events: [],
    result: { retval: xdr.ScVal.scvVoid(), auth },
  } as unknown as rpc.Api.SimulateTransactionResponse;
}

interface Stubs {
  getAccount: MockInstance;
  simulate: MockInstance;
  latestLedger: MockInstance;
  send: MockInstance;
  poll: MockInstance;
}

/**
 * Stubs the network for a sponsored flow. Accounts exist unless listed in
 * `missing`, and simulations answer in order from `simulations`.
 */
function stubNetwork(
  simulations: rpc.Api.SimulateTransactionResponse[],
  missing: string[] = [],
): Stubs {
  const proto = rpc.Server.prototype;
  const getAccount = vi.spyOn(proto, "getAccount").mockImplementation(async (address: string) => {
    if (missing.includes(address)) throw new Error(`Account not found: ${address}`);
    return new Account(address, "7");
  });
  const simulate = vi.spyOn(proto, "simulateTransaction");
  for (const simulation of simulations) simulate.mockResolvedValueOnce(simulation);
  const latestLedger = vi
    .spyOn(proto, "getLatestLedger")
    .mockResolvedValue({ id: "x", sequence: LATEST_LEDGER, protocolVersion: "28" } as never);
  const send = vi.spyOn(proto, "sendTransaction").mockResolvedValue({
    status: "PENDING",
    hash: SUBMITTED_HASH,
    latestLedger: LATEST_LEDGER,
    latestLedgerCloseTime: 0,
  } as never);
  const poll = vi.spyOn(proto, "pollTransaction").mockResolvedValue({
    status: rpc.Api.GetTransactionStatus.SUCCESS,
    txHash: SUBMITTED_HASH,
    ledger: 42,
  } as never);
  return { getAccount, simulate, latestLedger, send, poll };
}

const sign = (entries: xdr.SorobanAuthorizationEntry[], signer = AUTHORIZER, ledgers = DEFAULT_AUTH_VALIDITY_LEDGERS) =>
  authorizeEntries(entries, signer, LATEST_LEDGER + ledgers, Networks.TESTNET);

afterEach(() => {
  vi.restoreAllMocks();
});

describe("building a sponsored write", () => {
  it("sources the transaction from the fee payer and returns the entries to sign", async () => {
    const stubs = stubNetwork([simulationWith([unsignedEntry(AUTHORIZER.publicKey())])]);
    const sponsored = await new RwaAssetClient(rwaConfig).buildSponsoredTransferTx(
      AUTHORIZER.publicKey(),
      RECIPIENT,
      5n,
      { feeSource: FEE_PAYER },
    );

    expect(sponsored.transaction.source).toBe(FEE_PAYER);
    expect(sponsored.transaction.signatures).toHaveLength(0);
    expect(builtInvocation(sponsored.transaction)).toEqual({
      fn: "transfer",
      args: [AUTHORIZER.publicKey(), RECIPIENT, 5n],
    });
    expect(sponsored.authorizers).toEqual([AUTHORIZER.publicKey()]);
    expect(stubs.getAccount).toHaveBeenCalledWith(FEE_PAYER);
    expect(stubs.getAccount).toHaveBeenCalledWith(AUTHORIZER.publicKey());
  });

  // Confirmed on testnet: an account that does not exist cannot authorize.
  it("refuses an authorizer with no account on the network", async () => {
    stubNetwork([simulationWith([unsignedEntry(AUTHORIZER.publicKey())])], [AUTHORIZER.publicKey()]);
    await expect(
      new RwaAssetClient(rwaConfig).buildSponsoredTransferTx(AUTHORIZER.publicKey(), RECIPIENT, 5n, {
        feeSource: FEE_PAYER,
      }),
    ).rejects.toThrow(/has no account on the network/);
  });

  it("surfaces a simulation error", async () => {
    vi.spyOn(rpc.Server.prototype, "getAccount").mockResolvedValue(new Account(FEE_PAYER, "7"));
    vi.spyOn(rpc.Server.prototype, "simulateTransaction").mockResolvedValue({
      latestLedger: 3,
      error: "HostError: Error(Contract, #8)",
    } as never);
    await expect(
      new RwaAssetClient(rwaConfig).buildSponsoredMintTx(AUTHORIZER.publicKey(), RECIPIENT, 5n, {
        feeSource: FEE_PAYER,
      }),
    ).rejects.toThrow("Simulation error: HostError: Error(Contract, #8)");
  });

  it("marks entries the fee payer's signature covers as having no authorizer", async () => {
    stubNetwork([simulationWith([sourceAccountEntry(), unsignedEntry(NEW_ADMIN.publicKey(), "transfer_admin")])]);
    const sponsored = await new RegistryClient(configWith({ registry: REGISTRY_ID })).buildTransferAdminTx(
      FEE_PAYER,
      NEW_ADMIN.publicKey(),
    );

    expect(sponsored.transaction.source).toBe(FEE_PAYER);
    expect(builtInvocation(sponsored.transaction)).toEqual({
      fn: "transfer_admin",
      args: [NEW_ADMIN.publicKey()],
    });
    expect(sponsored.authorizers).toEqual([null, NEW_ADMIN.publicKey()]);
  });
});

describe("authorizeEntries", () => {
  it("signs the signer's entries with the expiry given, and leaves others untouched", async () => {
    const mine = unsignedEntry(AUTHORIZER.publicKey());
    const theirs = unsignedEntry(NEW_ADMIN.publicKey(), "transfer", 8n);
    const [signedMine, untouched] = await sign([mine, theirs]);

    const credentials = (signedMine as xdr.SorobanAuthorizationEntry).credentials;
    expect(credentials.type).toBe("sorobanCredentialsAddressV2");
    if (credentials.type !== "sorobanCredentialsAddressV2") throw new Error("unreachable");
    expect(credentials.addressV2.signatureExpirationLedger).toBe(LATEST_LEDGER + DEFAULT_AUTH_VALIDITY_LEDGERS);
    expect(credentials.addressV2.signature.type).not.toBe("scvVoid");
    expect(untouched?.equals(theirs)).toBe(true);
  });

  // authorizeEntry itself would sign with any key.
  it("refuses a signer that none of the entries name", async () => {
    await expect(sign([unsignedEntry(AUTHORIZER.publicKey())], NEW_ADMIN)).rejects.toThrow(
      new RegExp(`${NEW_ADMIN.publicKey()} is not asked to authorize`),
    );
  });

  it.each([0, -5, 1.5])("refuses a validUntilLedger of %s", async (ledger) => {
    await expect(
      authorizeEntries([unsignedEntry(AUTHORIZER.publicKey())], AUTHORIZER, ledger, Networks.TESTNET),
    ).rejects.toThrow(/positive ledger sequence/);
  });

  it("round-trips entries through base64 for another machine", async () => {
    const signed = await sign([unsignedEntry(AUTHORIZER.publicKey())]);
    const back = authEntriesFromXdr(authEntriesToXdr(signed));
    expect(back[0]?.equals(signed[0] as xdr.SorobanAuthorizationEntry)).toBe(true);
    expect(authEntryAddress(back[0] as xdr.SorobanAuthorizationEntry)).toBe(AUTHORIZER.publicKey());
  });
});

describe("finalizing a sponsored write", () => {
  async function built(entries: xdr.SorobanAuthorizationEntry[]) {
    const stubs = stubNetwork([simulationWith(entries), simulationWith([])]);
    const client = new RwaAssetClient(configWith({ rwaAsset: RWA_ID }, FEE_PAYER_SECRET));
    const sponsored = await client.buildSponsoredTransferTx(AUTHORIZER.publicKey(), RECIPIENT, 5n, {
      feeSource: FEE_PAYER,
    });
    return { client, sponsored, stubs };
  }

  it("attaches the signed entries to a transaction the fee payer sources", async () => {
    const { client, sponsored, stubs } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const signed = await sign(sponsored.authEntries);

    const tx = await client.finalizeSponsoredTx(sponsored, signed);

    expect(tx.source).toBe(FEE_PAYER);
    const { auth } = tx.operations[0] as unknown as { auth: xdr.SorobanAuthorizationEntry[] };
    expect(auth).toHaveLength(1);
    expect(auth[0]?.equals(signed[0] as xdr.SorobanAuthorizationEntry)).toBe(true);
    // Re-simulated with the signatures in place, since they change the resource fee.
    expect(stubs.simulate).toHaveBeenCalledTimes(2);
    expect(tx.signatures).toHaveLength(0);
  });

  it("keeps entries the fee payer covers alongside the signed ones", async () => {
    const stubs = stubNetwork([
      simulationWith([sourceAccountEntry(), unsignedEntry(NEW_ADMIN.publicKey(), "transfer_admin")]),
      simulationWith([]),
    ]);
    const client = new RegistryClient(configWith({ registry: REGISTRY_ID }));
    const sponsored = await client.buildTransferAdminTx(FEE_PAYER, NEW_ADMIN.publicKey());
    const signed = await sign(sponsored.authEntries, NEW_ADMIN);

    const tx = await client.finalizeSponsoredTx(sponsored, signed);
    const { auth } = tx.operations[0] as unknown as { auth: xdr.SorobanAuthorizationEntry[] };
    expect(auth.map((e) => e.credentials.type)).toEqual([
      "sorobanCredentialsSourceAccount",
      "sorobanCredentialsAddressV2",
    ]);
    expect(stubs.simulate).toHaveBeenCalledTimes(2);
  });

  it("refuses the wrong number of entries", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    await expect(client.finalizeSponsoredTx(sponsored, [])).rejects.toThrow(/Expected 1 authorization entries, got 0/);
  });

  it("refuses an entry that was never signed", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    await expect(client.finalizeSponsoredTx(sponsored, sponsored.authEntries)).rejects.toThrow(/is not signed/);
  });

  it("refuses an entry signed for a different invocation", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const other = await sign([unsignedEntry(AUTHORIZER.publicKey(), "approve")]);
    await expect(client.finalizeSponsoredTx(sponsored, other)).rejects.toThrow(/different invocation/);
  });

  it("refuses an entry signed with a different nonce", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const other = await sign([unsignedEntry(AUTHORIZER.publicKey(), "transfer", 99n)]);
    await expect(client.finalizeSponsoredTx(sponsored, other)).rejects.toThrow(/different nonce/);
  });

  it("refuses an entry expiring too soon to be included", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const signed = await sign(sponsored.authEntries, AUTHORIZER, MIN_AUTH_REMAINING_LEDGERS - 1);
    await expect(client.finalizeSponsoredTx(sponsored, signed)).rejects.toThrow(/at least 40 ledgers must remain/);
  });

  it("refuses an entry expiring beyond the maximum bound", async () => {
    const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const signed = await sign(sponsored.authEntries, AUTHORIZER, MAX_AUTH_VALIDITY_LEDGERS + 1);
    await expect(client.finalizeSponsoredTx(sponsored, signed)).rejects.toThrow(/more than 17280 ledgers/);
  });

  it("accepts expiries at both bounds", async () => {
    for (const ledgers of [MIN_AUTH_REMAINING_LEDGERS, MAX_AUTH_VALIDITY_LEDGERS]) {
      const { client, sponsored } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
      const signed = await sign(sponsored.authEntries, AUTHORIZER, ledgers);
      await expect(client.finalizeSponsoredTx(sponsored, signed)).resolves.toBeDefined();
      vi.restoreAllMocks();
    }
  });

  it("submits signed by the fee payer and reports the settled transaction", async () => {
    const { client, sponsored, stubs } = await built([unsignedEntry(AUTHORIZER.publicKey())]);
    const signed = await sign(sponsored.authEntries);

    await expect(client.submitSponsoredTx(sponsored, signed)).resolves.toEqual({
      hash: SUBMITTED_HASH,
      ledger: 42,
    });
    const submitted = stubs.send.mock.calls[0]?.[0] as { source: string; signatures: unknown[] };
    expect(submitted.source).toBe(FEE_PAYER);
    expect(submitted.signatures).toHaveLength(1);
  });

  it("refuses to submit with a signerSecret that is not the fee payer's", async () => {
    stubNetwork([simulationWith([unsignedEntry(AUTHORIZER.publicKey())]), simulationWith([])]);
    const client = new RwaAssetClient(configWith({ rwaAsset: RWA_ID }, AUTHORIZER.secret()));
    const sponsored = await client.buildSponsoredTransferTx(AUTHORIZER.publicKey(), RECIPIENT, 5n, {
      feeSource: FEE_PAYER,
    });
    const signed = await sign(sponsored.authEntries);
    await expect(client.submitSponsoredTx(sponsored, signed)).rejects.toThrow(
      /sourced and paid for by .*which must sign it/,
    );
  });
});
