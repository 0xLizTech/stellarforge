import {
  Account,
  BASE_FEE,
  Contract,
  Keypair,
  rpc,
  TransactionBuilder,
  xdr,
} from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import type { StellarForgeConfig, TxResult } from "./types.js";

/**
 * Validity window for a write transaction, in seconds.
 *
 * Long enough to survive a slow submission, short enough that an unsubmitted
 * signed transaction stops being replayable reasonably soon.
 */
const WRITE_TX_TIMEOUT_SECONDS = 180;

/** Validity window for a simulation, which is never submitted. */
const SIMULATION_TIMEOUT_SECONDS = 30;

/** What a submitted transaction yielded, before a caller shapes it. */
export interface SubmitOutcome {
  hash: string;
  ledger: number;
  /** The contract's return value. Absent for entry points returning `()`. */
  returnValue: xdr.ScVal | undefined;
}

/**
 * Shared plumbing for a client bound to one deployed contract.
 *
 * Every StellarForge client needs the same four things — an RPC server, a
 * contract handle, read-only simulation, and the write path — so they live
 * here rather than being copied per contract.
 */
export abstract class ContractClient {
  protected readonly server: rpc.Server;
  protected readonly contract: Contract;
  protected readonly config: StellarForgeConfig;

  /**
   * @param contractId - Deployed contract address, from `config.contracts`.
   * @param configKey - Name of the `contracts` field, used only so a missing
   *   address names itself in the error.
   */
  protected constructor(
    config: StellarForgeConfig,
    contractId: string | undefined,
    configKey: string,
  ) {
    if (!contractId) {
      throw new Error(`contracts.${configKey} address is required`);
    }
    this.config = config;
    this.server = new rpc.Server(config.rpcUrl, { allowHttp: false });
    this.contract = new Contract(contractId);
  }

  /** Simulates `method` and returns its raw return value. Touches no ledger state. */
  protected async simulateReadOnly(method: string, args: xdr.ScVal[]): Promise<xdr.ScVal> {
    // A throwaway account is enough: a simulation is never submitted, so the
    // source needs neither funding nor a real sequence number.
    const dummyKeypair = Keypair.random();
    const dummyAccount = new Account(dummyKeypair.publicKey(), "0");

    const tx = new TransactionBuilder(dummyAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(SIMULATION_TIMEOUT_SECONDS)
      .build();

    const simResult = await this.server.simulateTransaction(tx);

    if (rpc.Api.isSimulationError(simResult)) {
      throw new Error(`Simulation error: ${simResult.error}`);
    }

    if (!simResult.result) {
      throw new Error(`No result returned from ${method}`);
    }

    return simResult.result.retval;
  }

  /**
   * Builds and prepares an invocation sourced from `source`.
   *
   * For an entry point calling `require_auth`, `source` must be the address it
   * authorizes, so the transaction signature satisfies the check. For a
   * permissionless entry point, any funded account will do.
   */
  protected async buildWriteTx(
    source: string,
    method: string,
    args: xdr.ScVal[],
  ): Promise<Transaction> {
    const account = await this.server.getAccount(source);

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(WRITE_TX_TIMEOUT_SECONDS)
      .build();

    // Simulates, then attaches the footprint, authorization entries and
    // resource fee the transaction needs to be accepted.
    return this.server.prepareTransaction(tx);
  }

  /**
   * Signs with the configured secret and waits for the transaction to settle.
   *
   * @param source - The address sourcing the transaction, which must sign it:
   *   it supplies the sequence number and pays the fee, whatever the contract
   *   requires. For an entry point calling `require_auth`, this is also the
   *   address being authorized. For a permissionless one it is simply whoever
   *   is paying.
   */
  protected async signAndSubmit(tx: Transaction, source: string): Promise<SubmitOutcome> {
    const secret = this.config.signerSecret;
    if (!secret) {
      throw new Error(
        "config.signerSecret is required to submit a transaction. " +
          "Use the matching build*Tx method to sign with a wallet instead.",
      );
    }

    const keypair = Keypair.fromSecret(secret);

    // The signer has to be the transaction source. Catching it here beats a
    // protocol-level bad-auth failure, or a require_auth failure after the fee
    // is spent.
    if (keypair.publicKey() !== source) {
      throw new Error(
        `signerSecret is for ${keypair.publicKey()} but this call must be authorized by ` +
          `${source}. Paying from a different account is not supported.`,
      );
    }

    tx.sign(keypair);

    const sent = await this.server.sendTransaction(tx);
    if (sent.status === "ERROR") {
      throw new Error(`Transaction ${sent.hash} was rejected on submission`);
    }

    const settled = await this.server.pollTransaction(sent.hash);
    if (settled.status !== rpc.Api.GetTransactionStatus.SUCCESS) {
      throw new Error(`Transaction ${sent.hash} did not succeed: ${settled.status}`);
    }

    return { hash: sent.hash, ledger: settled.ledger, returnValue: settled.returnValue };
  }

  /**
   * Signs, submits, and discards the return value.
   *
   * The common case: most entry points return `()`, so there is nothing to
   * carry back beyond the hash and ledger.
   */
  protected async submit(tx: Transaction, source: string): Promise<TxResult> {
    const { hash, ledger } = await this.signAndSubmit(tx, source);
    return { hash, ledger };
  }
}

/**
 * Unwraps a Soroban unit enum variant.
 *
 * `scValToNative` decodes `ProposalStatus::Active` to the one-element array
 * `["Active"]`, not to `"Active"`. Handing that straight to a caller gives them
 * a value that silently fails every comparison they write.
 */
export function unwrapEnumVariant(decoded: unknown): string {
  if (Array.isArray(decoded) && typeof decoded[0] === "string") {
    return decoded[0];
  }
  if (typeof decoded === "string") {
    return decoded;
  }
  throw new Error(`Expected a unit enum variant, got ${JSON.stringify(decoded)}`);
}
