/**
 * Shared stubbing for the client suites.
 *
 * The seam is `rpc.Server.prototype` rather than the module, so `Contract`,
 * `TransactionBuilder` and the ScVal codecs stay real and only the network
 * calls are replaced.
 */

import { vi } from "vitest";
import type { MockInstance } from "vitest";
import {
  Account,
  Keypair,
  StrKey,
  Transaction,
  nativeToScVal,
  rpc,
  scValToNative,
  xdr,
} from "@stellar/stellar-sdk";

import type { StellarForgeConfig } from "../src/types.js";
import { TESTNET_CONFIG } from "../src/types.js";

// ─── Fixtures ─────────────────────────────────────────────────────────────────

/** Deterministic C-addresses, so failures name the same contract every run. */
export const RWA_ID = StrKey.encodeContract(Buffer.alloc(32, 1));
export const COMPLIANCE_ID = StrKey.encodeContract(Buffer.alloc(32, 2));
export const REGISTRY_ID = StrKey.encodeContract(Buffer.alloc(32, 5));
export const GOVERNANCE_ID = StrKey.encodeContract(Buffer.alloc(32, 6));
export const ORACLE_ID = StrKey.encodeContract(Buffer.alloc(32, 9));

export const HOLDER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 3));
export const SPENDER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 4));

/**
 * A deterministic signer. The write path requires the signing keypair to be the
 * transaction source, so the public key and the secret have to come from the
 * same seed rather than being independent fixtures.
 */
export const SIGNER_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 7));
export const SIGNER = SIGNER_KP.publicKey();
export const SIGNER_SECRET = SIGNER_KP.secret();

/** A second keypair, for asserting a mismatched signer is refused. */
export const OTHER_KP = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 8));

export const SUBMITTED_HASH = "a".repeat(64);

export function configWith(
  contracts: StellarForgeConfig["contracts"],
  signerSecret?: string,
): StellarForgeConfig {
  return signerSecret === undefined
    ? { ...TESTNET_CONFIG, contracts }
    : { ...TESTNET_CONFIG, contracts, signerSecret };
}

// ─── Read-path stubbing ───────────────────────────────────────────────────────

type SimulateFn = (tx: Transaction) => Promise<rpc.Api.SimulateTransactionResponse>;
export type Spy = MockInstance<SimulateFn>;

export function stubSimulation(response: unknown): Spy {
  const spy = vi.spyOn(
    rpc.Server.prototype as unknown as { simulateTransaction: SimulateFn },
    "simulateTransaction",
  );
  spy.mockResolvedValue(response as rpc.Api.SimulateTransactionResponse);
  return spy;
}

/** A successful simulation carrying `retval`. */
export function succeeds(retval: xdr.ScVal): unknown {
  return { latestLedger: 3, result: { retval, auth: [] } };
}

/**
 * A failed simulation. `rpc.Api.isSimulationError` discriminates purely on the
 * presence of an `error` key, so that is what makes this a failure.
 */
export function fails(message: string): unknown {
  return { latestLedger: 3, error: message };
}

/**
 * A simulation that is not an error yet carries no result. The RPC returns this
 * shape for a restore-required preflight, among others.
 */
export function succeedsWithoutResult(): unknown {
  return { latestLedger: 3 };
}

// ─── Write-path stubbing ──────────────────────────────────────────────────────

type GetAccountFn = (address: string) => Promise<Account>;
type PrepareFn = (tx: Transaction) => Promise<Transaction>;
type SendFn = (tx: Transaction) => Promise<rpc.Api.SendTransactionResponse>;
type PollFn = (
  hash: string,
  opts?: { attempts?: number },
) => Promise<rpc.Api.GetTransactionResponse>;

export interface WriteStubs {
  getAccount: MockInstance<GetAccountFn>;
  prepare: MockInstance<PrepareFn>;
  send: MockInstance<SendFn>;
  poll: MockInstance<PollFn>;
}

/**
 * Stubs the four network calls a write makes. `prepareTransaction` hands back
 * the transaction it was given, so the real builder output stays under test and
 * the submit path has something genuine to sign.
 */
export function stubWritePath(
  overrides: Partial<{ send: unknown; poll: unknown }> = {},
): WriteStubs {
  const proto = rpc.Server.prototype as unknown as {
    getAccount: GetAccountFn;
    prepareTransaction: PrepareFn;
    sendTransaction: SendFn;
    pollTransaction: PollFn;
  };

  const getAccount = vi.spyOn(proto, "getAccount");
  getAccount.mockImplementation(async (address: string) => new Account(address, "7"));

  const prepare = vi.spyOn(proto, "prepareTransaction");
  prepare.mockImplementation(async (tx: Transaction) => tx);

  const send = vi.spyOn(proto, "sendTransaction");
  send.mockResolvedValue(
    (overrides.send ?? {
      status: "PENDING",
      hash: SUBMITTED_HASH,
      latestLedger: 10,
      latestLedgerCloseTime: 0,
    }) as rpc.Api.SendTransactionResponse,
  );

  const poll = vi.spyOn(proto, "pollTransaction");
  poll.mockResolvedValue(
    (overrides.poll ?? {
      status: rpc.Api.GetTransactionStatus.SUCCESS,
      txHash: SUBMITTED_HASH,
      ledger: 42,
    }) as rpc.Api.GetTransactionResponse,
  );

  return { getAccount, prepare, send, poll };
}

// ─── Decoding what was built ──────────────────────────────────────────────────

export interface Invocation {
  fn: string;
  args: unknown[];
}

/** The contract function and decoded arguments of a built transaction. */
export function builtInvocation(tx: Transaction): Invocation {
  const invoked = invocationArgs(tx);
  return {
    fn: invoked.functionName.toString(),
    args: invoked.args.map((arg) => scValToNative(arg)),
  };
}

/**
 * The contract call made by a built transaction's first operation.
 *
 * `xdr.HostFunction` is a discriminated union, so the arm is checked before it
 * is read. A test inspecting some other kind of host function fails here
 * instead of passing on whatever that arm happens to hold.
 */
export function invocationArgs(tx: Transaction): xdr.InvokeContractArgs {
  const { func } = tx.operations[0] as { func: xdr.HostFunction };
  if (func.type !== "hostFunctionTypeInvokeContract") {
    throw new Error(`Expected a contract invocation, got ${func.type}`);
  }
  return func.invokeContract;
}

/** The contract function and decoded arguments of the simulated transaction. */
export function invocation(spy: Spy): Invocation {
  return builtInvocation(spy.mock.calls[0]?.[0] as Transaction);
}

// ─── ScVal construction ───────────────────────────────────────────────────────

export const i128 = (v: bigint): xdr.ScVal => nativeToScVal(v, { type: "i128" });
export const u64 = (v: bigint): xdr.ScVal => nativeToScVal(v, { type: "u64" });
export const u32 = (v: number): xdr.ScVal => nativeToScVal(v, { type: "u32" });
export const bool = (v: boolean): xdr.ScVal => nativeToScVal(v);
export const str = (v: string): xdr.ScVal => nativeToScVal(v, { type: "string" });
export const bytes = (v: Buffer): xdr.ScVal => nativeToScVal(v, { type: "bytes" });
export const addr = (v: string): xdr.ScVal => nativeToScVal(v, { type: "address" });

/** Builds the ScMap a Soroban struct decodes from. Keys must be sorted. */
export function struct(fields: Record<string, xdr.ScVal>): xdr.ScVal {
  const entries = Object.keys(fields)
    .sort()
    .map(
      (key) =>
        new xdr.ScMapEntry({
          key: nativeToScVal(key, { type: "symbol" }),
          val: fields[key] as xdr.ScVal,
        }),
    );
  return xdr.ScVal.scvMap(entries);
}

/**
 * Builds a Soroban unit enum variant, which is encoded as a one-element vector
 * holding the variant name as a symbol.
 */
export function enumVariant(name: string): xdr.ScVal {
  return xdr.ScVal.scvVec([nativeToScVal(name, { type: "symbol" })]);
}

/** `Option::None`, which Soroban encodes as void. */
export function none(): xdr.ScVal {
  return xdr.ScVal.scvVoid();
}
