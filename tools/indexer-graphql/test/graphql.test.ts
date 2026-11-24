import { describe, it, expect, beforeEach } from "vitest";
import { InMemoryIndexerStore, IndexerGraphQLServer } from "../src/index.js";

describe("IndexerGraphQLServer", () => {
  let store: InMemoryIndexerStore;
  let server: IndexerGraphQLServer;

  beforeEach(() => {
    store = new InMemoryIndexerStore();
    server = new IndexerGraphQLServer(store);
  });

  it("queries holders for an asset with pagination", async () => {
    const asset = "C_ASSET_XYZ";
    store.setBalance(asset, "G_ALICE", 500n);
    store.setBalance(asset, "G_BOB", 1000n);
    store.setBalance(asset, "G_CHARLIE", 1500n);

    // Query page 1 (limit 2)
    const res1 = await server.execute({
      query: `query { holders(asset: "${asset}", limit: 2) { nodes { address balance } pageInfo { hasNextPage endCursor totalCount } } }`,
    });

    expect(res1.errors).toBeUndefined();
    expect(res1.data?.holders.nodes).toHaveLength(2);
    expect(res1.data?.holders.nodes[0].address).toBe("G_ALICE");
    expect(res1.data?.holders.pageInfo.hasNextPage).toBe(true);
    expect(res1.data?.holders.pageInfo.totalCount).toBe(3);

    const cursor = res1.data?.holders.pageInfo.endCursor;

    // Query page 2 using cursor
    const res2 = await server.execute({
      query: `query { holders(asset: "${asset}", limit: 2) }`,
      variables: { asset, limit: 2, cursor },
    });

    expect(res2.errors).toBeUndefined();
    expect(res2.data?.holders.nodes).toHaveLength(1);
    expect(res2.data?.holders.nodes[0].address).toBe("G_CHARLIE");
    expect(res2.data?.holders.pageInfo.hasNextPage).toBe(false);
  });

  it("queries balance and balances", async () => {
    const asset = "C_ASSET_XYZ";
    store.setBalance(asset, "G_ALICE", 500n);

    const resSingle = await server.execute({
      query: `query { balance(asset: "${asset}", owner: "G_ALICE") { balance } }`,
    });
    expect(resSingle.data?.balance.balance).toBe("500");

    const resAll = await server.execute({
      query: `query { balances(asset: "${asset}") { nodes { owner balance } } }`,
    });
    expect(resAll.data?.balances.nodes).toHaveLength(1);
  });

  it("queries proposals with pagination", async () => {
    store.addProposal({
      id: "1",
      title: "Proposal 1",
      descriptionHash: "hash1",
      proposer: "G_PROP",
      status: "Active",
      votesFor: "100",
      votesAgainst: "0",
      createdAtLedger: 100,
      votingClosesLedger: 200,
    });

    const res = await server.execute({
      query: `query { proposals(limit: 10) { nodes { id title status } pageInfo { totalCount } } }`,
    });

    expect(res.data?.proposals.nodes).toHaveLength(1);
    expect(res.data?.proposals.nodes[0].title).toBe("Proposal 1");
    expect(res.data?.proposals.pageInfo.totalCount).toBe(1);
  });
});
