import { describe, it, expect, vi, afterEach } from "vitest";
import { rpc, xdr } from "@stellar/stellar-sdk";

import { GovernanceClient } from "../src/governance.js";
import { TESTNET_CONFIG } from "../src/types.js";
import type { StellarForgeConfig } from "../src/types.js";
import {
  GOVERNANCE_ID,
  HOLDER,
  SIGNER as PROPOSER,
  SIGNER_SECRET as PROPOSER_SECRET,
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
  bytes,
  enumVariant,
  i128,
  none,
  str,
  struct,
  u32,
  u64,
} from "./helpers.js";

const govConfig: StellarForgeConfig = configWith({ governance: GOVERNANCE_ID });
const signerConfig: StellarForgeConfig = configWith(
  { governance: GOVERNANCE_ID },
  PROPOSER_SECRET,
);

const DESC_HASH_BYTES = Buffer.alloc(32, 0xcd);
const DESC_HASH_HEX = "cd".repeat(32);

function proposalScVal(overrides: Record<string, xdr.ScVal> = {}): xdr.ScVal {
  return struct({
    id: u64(1n),
    proposer: addr(PROPOSER),
    title: str("Raise the compliance floor"),
    description_hash: bytes(DESC_HASH_BYTES),
    votes_for: i128(300n),
    votes_against: i128(120n),
    deadline_ledger: u32(900_000),
    status: enumVariant("Active"),
    ...overrides,
  });
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("GovernanceClient constructor", () => {
  it("rejects a config with no governance address", () => {
    expect(() => new GovernanceClient({ ...TESTNET_CONFIG, contracts: {} })).toThrow(
      "contracts.governance address is required",
    );
  });

  it("accepts a valid contract id", () => {
    expect(() => new GovernanceClient(govConfig)).not.toThrow();
  });
});

describe("GovernanceClient.getProposal", () => {
  it("maps the contract struct onto camelCase", async () => {
    stubSimulation(succeeds(proposalScVal()));
    await expect(new GovernanceClient(govConfig).getProposal(1n)).resolves.toEqual({
      id: 1n,
      proposer: PROPOSER,
      title: "Raise the compliance floor",
      descriptionHash: DESC_HASH_HEX,
      votesFor: 300n,
      votesAgainst: 120n,
      deadlineLedger: 900_000,
      status: "Active",
    });
  });

  // A Soroban unit enum variant decodes to a one-element array, so an
  // unwrapped status would be ["Active"] and fail every comparison a caller
  // writes against it.
  it.each(["Active", "Passed", "Rejected", "Executed"] as const)(
    "unwraps the %s status to a bare string",
    async (status) => {
      stubSimulation(succeeds(proposalScVal({ status: enumVariant(status) })));
      const proposal = await new GovernanceClient(govConfig).getProposal(1n);
      expect(proposal?.status).toBe(status);
      expect(Array.isArray(proposal?.status)).toBe(false);
    },
  );

  it("renders description_hash as hex, not raw bytes", async () => {
    stubSimulation(succeeds(proposalScVal()));
    const proposal = await new GovernanceClient(govConfig).getProposal(1n);
    expect(proposal?.descriptionHash).toBe(DESC_HASH_HEX);
  });

  // Ids and vote tallies exceed Number.MAX_SAFE_INTEGER in principle: u64 and
  // i128 respectively. Decoding either through `number` would lose precision.
  it("keeps the id as bigint and the deadline as number", async () => {
    stubSimulation(succeeds(proposalScVal()));
    const proposal = await new GovernanceClient(govConfig).getProposal(1n);
    expect(typeof proposal?.id).toBe("bigint");
    expect(typeof proposal?.votesFor).toBe("bigint");
    expect(typeof proposal?.deadlineLedger).toBe("number");
  });

  it("returns null for an id that carries no proposal", async () => {
    stubSimulation(succeeds(none()));
    await expect(new GovernanceClient(govConfig).getProposal(999n)).resolves.toBeNull();
  });

  it("invokes `get_proposal` with the id", async () => {
    const spy = stubSimulation(succeeds(none()));
    await new GovernanceClient(govConfig).getProposal(7n);
    expect(invocation(spy)).toEqual({ fn: "get_proposal", args: [7n] });
  });
});

describe("GovernanceClient.hasVoted", () => {
  it.each([true, false])("decodes %s", async (voted) => {
    stubSimulation(succeeds(bool(voted)));
    await expect(new GovernanceClient(govConfig).hasVoted(1n, HOLDER)).resolves.toBe(voted);
  });

  it("invokes `has_voted` with the id before the voter", async () => {
    const spy = stubSimulation(succeeds(bool(false)));
    await new GovernanceClient(govConfig).hasVoted(3n, HOLDER);
    expect(invocation(spy)).toEqual({ fn: "has_voted", args: [3n, HOLDER] });
  });
});

describe("GovernanceClient.proposalCount", () => {
  it("decodes a u64 to bigint", async () => {
    stubSimulation(succeeds(u64(12n)));
    const count = await new GovernanceClient(govConfig).proposalCount();
    expect(count).toBe(12n);
    expect(typeof count).toBe("bigint");
  });

  it("decodes zero for a contract with no proposals", async () => {
    stubSimulation(succeeds(u64(0n)));
    await expect(new GovernanceClient(govConfig).proposalCount()).resolves.toBe(0n);
  });
});

describe("GovernanceClient.pendingAdmin (#125)", () => {
  it("decodes an address when a transfer is pending", async () => {
    stubSimulation(succeeds(addr(HOLDER)));
    await expect(new GovernanceClient(govConfig).pendingAdmin()).resolves.toBe(HOLDER);
  });

  it("decodes null when no transfer is pending", async () => {
    stubSimulation(succeeds(none()));
    await expect(new GovernanceClient(govConfig).pendingAdmin()).resolves.toBeNull();
  });
});

describe("GovernanceClient.quorum (#135)", () => {
  it("decodes an i128 to bigint", async () => {
    stubSimulation(succeeds(i128(1_000n)));
    const quorum = await new GovernanceClient(govConfig).quorum();
    expect(quorum).toBe(1_000n);
    expect(typeof quorum).toBe("bigint");
  });

  it("decodes the default of zero", async () => {
    stubSimulation(succeeds(i128(0n)));
    await expect(new GovernanceClient(govConfig).quorum()).resolves.toBe(0n);
  });
});

describe("GovernanceClient.passThresholdBps (#135)", () => {
  it("decodes a u32", async () => {
    stubSimulation(succeeds(u32(6_000)));
    await expect(new GovernanceClient(govConfig).passThresholdBps()).resolves.toBe(6_000);
  });
});

describe("GovernanceClient read failures", () => {
  it("surfaces the simulation error message", async () => {
    stubSimulation(fails("HostError: Error(Contract, #3)"));
    await expect(new GovernanceClient(govConfig).getProposal(1n)).rejects.toThrow(
      "Simulation error: HostError: Error(Contract, #3)",
    );
  });

  it("names the method when the simulation returns no result", async () => {
    stubSimulation(succeedsWithoutResult());
    await expect(new GovernanceClient(govConfig).proposalCount()).rejects.toThrow(
      "No result returned from proposal_count",
    );
  });
});

describe("GovernanceClient write builders", () => {
  it("invokes `propose` with the documented argument order", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildProposeTx(
      PROPOSER,
      "Raise the compliance floor",
      DESC_HASH_HEX,
      5000,
    );
    const { fn, args } = builtInvocation(tx);

    expect(fn).toBe("propose");
    expect(args[0]).toBe(PROPOSER);
    expect(args[1]).toBe("Raise the compliance floor");
    expect(Buffer.from(args[2] as Uint8Array).toString("hex")).toBe(DESC_HASH_HEX);
    expect(args[3]).toBe(5000);
  });

  // A malformed hash would otherwise reach the contract as truncated bytes.
  it.each(["abc", "zz", "nothex!"])("rejects the malformed hash %s", async (hash) => {
    stubWritePath();
    await expect(
      new GovernanceClient(govConfig).buildProposeTx(PROPOSER, "t", hash, 100),
    ).rejects.toThrow(/even-length hex/);
  });

  it("accepts an empty description hash", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildProposeTx(PROPOSER, "t", "", 100);
    expect(builtInvocation(tx).fn).toBe("propose");
  });

  it("invokes `vote` with the documented argument order", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildVoteTx(HOLDER, 4n, true, 250n);
    expect(builtInvocation(tx)).toEqual({ fn: "vote", args: [HOLDER, 4n, true, 250n] });
  });

  it("carries a vote against through unchanged", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildVoteTx(HOLDER, 4n, false, 1n);
    expect(builtInvocation(tx).args[2]).toBe(false);
  });

  it("invokes `finalize` with only the id", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildFinalizeTx(HOLDER, 4n);
    expect(builtInvocation(tx)).toEqual({ fn: "finalize", args: [4n] });
  });

  it("invokes `set_quorum` with the admin and the quorum (#135)", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildSetQuorumTx(PROPOSER, 1_000n);
    expect(builtInvocation(tx)).toEqual({ fn: "set_quorum", args: [1_000n] });
    expect(tx.source).toBe(PROPOSER);
  });

  it("invokes `set_pass_threshold_bps` with the threshold (#135)", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildSetPassThresholdBpsTx(PROPOSER, 6_000);
    expect(builtInvocation(tx)).toEqual({ fn: "set_pass_threshold_bps", args: [6_000] });
    expect(tx.source).toBe(PROPOSER);
  });

  it("invokes `propose_admin` with the new admin (#125)", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildProposeAdminTx(PROPOSER, HOLDER);
    expect(builtInvocation(tx)).toEqual({ fn: "propose_admin", args: [HOLDER] });
    expect(tx.source).toBe(PROPOSER);
  });

  it("invokes `accept_admin` sourced from the proposed admin, with no args (#125)", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildAcceptAdminTx(HOLDER);
    expect(builtInvocation(tx)).toEqual({ fn: "accept_admin", args: [] });
    expect(tx.source).toBe(HOLDER);
  });

  it("invokes `cancel_admin_transfer` with no args (#125)", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildCancelAdminTransferTx(PROPOSER);
    expect(builtInvocation(tx)).toEqual({ fn: "cancel_admin_transfer", args: [] });
    expect(tx.source).toBe(PROPOSER);
  });

  it.each([
    {
      name: "propose",
      source: PROPOSER,
      build: (c: GovernanceClient) => c.buildProposeTx(PROPOSER, "t", DESC_HASH_HEX, 100),
    },
    { name: "vote", source: HOLDER, build: (c: GovernanceClient) => c.buildVoteTx(HOLDER, 1n, true, 1n) },
    // finalize requires authorization from nobody, so the source is simply
    // whoever pays — but it still signs, because it sources the transaction.
    { name: "finalize", source: HOLDER, build: (c: GovernanceClient) => c.buildFinalizeTx(HOLDER, 1n) },
  ])("$name sources the transaction from $source", async ({ source, build }) => {
    const stubs = stubWritePath();
    const tx = await build(new GovernanceClient(govConfig));
    expect(tx.source).toBe(source);
    expect(stubs.getAccount).toHaveBeenCalledWith(source);
  });

  it("builds without a signerSecret and leaves the transaction unsigned", async () => {
    stubWritePath();
    const tx = await new GovernanceClient(govConfig).buildVoteTx(HOLDER, 1n, true, 1n);
    expect(govConfig.signerSecret).toBeUndefined();
    expect(tx.signatures).toHaveLength(0);
  });
});

describe("GovernanceClient.propose", () => {
  // propose returns the id it assigned. Reading proposalCount afterwards would
  // race with anyone else proposing in the same ledger, so the id has to come
  // from the transaction's own return value.
  it("returns the proposal id from the transaction return value", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.SUCCESS,
        txHash: SUBMITTED_HASH,
        ledger: 42,
        returnValue: u64(9n),
      },
    });

    await expect(
      new GovernanceClient(signerConfig).propose(PROPOSER, "t", DESC_HASH_HEX, 100),
    ).resolves.toEqual({ hash: SUBMITTED_HASH, ledger: 42, proposalId: 9n });
  });

  it("keeps the proposal id as bigint", async () => {
    stubWritePath({
      poll: {
        status: rpc.Api.GetTransactionStatus.SUCCESS,
        txHash: SUBMITTED_HASH,
        ledger: 42,
        returnValue: u64(9n),
      },
    });
    const { proposalId } = await new GovernanceClient(signerConfig).propose(
      PROPOSER,
      "t",
      DESC_HASH_HEX,
      100,
    );
    expect(typeof proposalId).toBe("bigint");
  });

  // Reporting success without the id would leave the caller with no way to
  // find the proposal they just created.
  it("throws when the transaction succeeds but carries no return value", async () => {
    stubWritePath();
    await expect(
      new GovernanceClient(signerConfig).propose(PROPOSER, "t", DESC_HASH_HEX, 100),
    ).rejects.toThrow(/returned no proposal id/);
  });
});

describe("GovernanceClient write submission", () => {
  it.each([
    { name: "vote", call: (c: GovernanceClient) => c.vote(PROPOSER, 1n, true, 5n) },
    { name: "finalize", call: (c: GovernanceClient) => c.finalize(PROPOSER, 1n) },
  ])("$name reports the settled transaction", async ({ call }) => {
    stubWritePath();
    await expect(call(new GovernanceClient(signerConfig))).resolves.toEqual({
      hash: SUBMITTED_HASH,
      ledger: 42,
    });
  });

  it("refuses to submit without a signerSecret", async () => {
    stubWritePath();
    await expect(new GovernanceClient(govConfig).vote(PROPOSER, 1n, true, 1n)).rejects.toThrow(
      /signerSecret is required/,
    );
  });

  it("refuses when the signer is not the transaction source", async () => {
    const stubs = stubWritePath();
    const wrong = configWith({ governance: GOVERNANCE_ID }, OTHER_KP.secret());
    await expect(new GovernanceClient(wrong).vote(PROPOSER, 1n, true, 1n)).rejects.toThrow(
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
    await expect(new GovernanceClient(signerConfig).finalize(PROPOSER, 1n)).rejects.toThrow(
      /did not succeed: FAILED/,
    );
  });
});
