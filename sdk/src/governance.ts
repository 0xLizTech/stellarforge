import { nativeToScVal, scValToNative } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import { ContractClient, unwrapEnumVariant } from "./base.js";
import type {
  Proposal,
  ProposalCreated,
  ProposalStatus,
  StellarForgeConfig,
  TxResult,
} from "./types.js";

/** Converts a hex string to the `Bytes` the contract expects. */
function hexToScVal(hex: string): ReturnType<typeof nativeToScVal> {
  if (hex.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(hex)) {
    throw new Error(`Expected an even-length hex string, got: ${hex}`);
  }
  return nativeToScVal(Buffer.from(hex, "hex"), { type: "bytes" });
}

/**
 * Client for the deployed governance contract.
 *
 * # Results here are advisory
 *
 * The Phase 1 contract does not weigh votes against anything. `vote` accepts
 * whatever weight its caller declares and verifies it against no balance, and
 * `finalize` applies no quorum — a proposal passes on a simple majority of
 * declared weight. **Any address can therefore carry any proposal.**
 *
 * Weight is deliberately unverified rather than read from a live token
 * balance, which would be worse: a holder could vote, transfer the same tokens
 * onward and vote again from the recipient. Correct weighting needs balances
 * snapshotted at proposal creation, which arrives with `SFORGE` in Phase 2/3.
 *
 * Do not gate anything privileged on an outcome read through this client.
 */
export class GovernanceClient extends ContractClient {
  constructor(config: StellarForgeConfig) {
    super(config, config.contracts.governance, "governance");
  }

  // ── Read-only queries ────────────────────────────────────────────────────

  /**
   * Fetches a proposal by id.
   *
   * @returns The proposal, or `null` when no proposal carries that id.
   */
  async getProposal(proposalId: bigint): Promise<Proposal | null> {
    const result = await this.simulateReadOnly("get_proposal", [
      nativeToScVal(proposalId, { type: "u64" }),
    ]);

    // get_proposal returns Option<Proposal>, and None decodes to null.
    const native = scValToNative(result) as Record<string, unknown> | null;
    if (native === null) {
      return null;
    }

    return {
      id: native["id"] as bigint,
      proposer: native["proposer"] as string,
      title: native["title"] as string,
      descriptionHash: Buffer.from(native["description_hash"] as Uint8Array).toString("hex"),
      votesFor: native["votes_for"] as bigint,
      votesAgainst: native["votes_against"] as bigint,
      deadlineLedger: native["deadline_ledger"] as number,
      // A unit enum variant decodes to a one-element array, not a string.
      status: unwrapEnumVariant(native["status"]) as ProposalStatus,
    };
  }

  /** Whether `voter` has already cast a vote on `proposalId`. */
  async hasVoted(proposalId: bigint, voter: string): Promise<boolean> {
    const result = await this.simulateReadOnly("has_voted", [
      nativeToScVal(proposalId, { type: "u64" }),
      nativeToScVal(voter, { type: "address" }),
    ]);
    return scValToNative(result) as boolean;
  }

  /** How many proposals exist. Ids run from 1 to this value. */
  async proposalCount(): Promise<bigint> {
    const result = await this.simulateReadOnly("proposal_count", []);
    return scValToNative(result) as bigint;
  }

  /** The contract admin. */
  async admin(): Promise<string> {
    const result = await this.simulateReadOnly("admin", []);
    return scValToNative(result) as string;
  }

  // ── Write path ───────────────────────────────────────────────────────────

  /**
   * Opens a proposal, authorized and paid for by `proposer`.
   *
   * @param descriptionHash - Hex of the off-chain document's hash.
   * @param votingPeriodLedgers - Ledgers from now until voting closes.
   */
  async buildProposeTx(
    proposer: string,
    title: string,
    descriptionHash: string,
    votingPeriodLedgers: number,
  ): Promise<Transaction> {
    return this.buildWriteTx(proposer, "propose", [
      nativeToScVal(proposer, { type: "address" }),
      nativeToScVal(title, { type: "string" }),
      hexToScVal(descriptionHash),
      nativeToScVal(votingPeriodLedgers, { type: "u32" }),
    ]);
  }

  /**
   * Submits a proposal and returns the id the contract assigned.
   *
   * The id comes from the transaction's return value. Reading `proposalCount`
   * afterwards would race with anyone else proposing in the same ledger.
   */
  async propose(
    proposer: string,
    title: string,
    descriptionHash: string,
    votingPeriodLedgers: number,
  ): Promise<ProposalCreated> {
    const tx = await this.buildProposeTx(proposer, title, descriptionHash, votingPeriodLedgers);
    const { hash, ledger, returnValue } = await this.signAndSubmit(tx, proposer);

    if (!returnValue) {
      throw new Error(`Transaction ${hash} succeeded but returned no proposal id`);
    }

    return { hash, ledger, proposalId: scValToNative(returnValue) as bigint };
  }

  /**
   * Casts a vote.
   *
   * `weight` is accepted as declared and checked against nothing beyond being
   * positive. See the class documentation before treating a tally as meaningful.
   */
  async buildVoteTx(
    voter: string,
    proposalId: bigint,
    support: boolean,
    weight: bigint,
  ): Promise<Transaction> {
    return this.buildWriteTx(voter, "vote", [
      nativeToScVal(voter, { type: "address" }),
      nativeToScVal(proposalId, { type: "u64" }),
      nativeToScVal(support),
      nativeToScVal(weight, { type: "i128" }),
    ]);
  }

  async vote(
    voter: string,
    proposalId: bigint,
    support: boolean,
    weight: bigint,
  ): Promise<TxResult> {
    return this.submit(await this.buildVoteTx(voter, proposalId, support, weight), voter);
  }

  /**
   * Closes voting and settles a proposal as Passed or Rejected. A tie fails.
   *
   * The contract calls `require_auth` on nobody, so anyone may finalize. The
   * `source` here is simply whoever pays the fee, and must sign because it
   * sources the transaction.
   *
   * Fails with `VotingNotClosed` before the deadline ledger, which surfaces
   * while building rather than on-chain.
   */
  async buildFinalizeTx(source: string, proposalId: bigint): Promise<Transaction> {
    return this.buildWriteTx(source, "finalize", [nativeToScVal(proposalId, { type: "u64" })]);
  }

  async finalize(source: string, proposalId: bigint): Promise<TxResult> {
    return this.submit(await this.buildFinalizeTx(source, proposalId), source);
  }
}
