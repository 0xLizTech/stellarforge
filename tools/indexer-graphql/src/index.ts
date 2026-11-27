export interface PageInfo {
  hasNextPage: boolean;
  endCursor: string | null;
  totalCount: number;
}

export interface Holder {
  address: string;
  balance: string;
}

export interface HoldersConnection {
  nodes: Holder[];
  pageInfo: PageInfo;
}

export interface BalanceRecord {
  asset: string;
  owner: string;
  balance: string;
}

export interface BalancesConnection {
  nodes: BalanceRecord[];
  pageInfo: PageInfo;
}

export interface ProposalRecord {
  id: string;
  title: string;
  descriptionHash: string;
  proposer: string;
  status: string;
  votesFor: string;
  votesAgainst: string;
  createdAtLedger: number;
  votingClosesLedger: number;
}

export interface ProposalsConnection {
  nodes: ProposalRecord[];
  pageInfo: PageInfo;
}

export interface IndexerStore {
  getHolders(asset: string, limit?: number, cursor?: string): HoldersConnection;
  getBalance(asset: string, owner: string): BalanceRecord | null;
  getBalances(asset: string, limit?: number, cursor?: string): BalancesConnection;
  getProposals(limit?: number, cursor?: string): ProposalsConnection;
  getProposal(id: string): ProposalRecord | null;
}

export class InMemoryIndexerStore implements IndexerStore {
  private holdersByAsset: Map<string, Map<string, bigint>> = new Map();
  private proposals: ProposalRecord[] = [];

  setBalance(asset: string, owner: string, balance: bigint): void {
    let assetMap = this.holdersByAsset.get(asset);
    if (!assetMap) {
      assetMap = new Map();
      this.holdersByAsset.set(asset, assetMap);
    }
    if (balance <= 0n) {
      assetMap.delete(owner);
    } else {
      assetMap.set(owner, balance);
    }
  }

  addProposal(proposal: ProposalRecord): void {
    this.proposals.push(proposal);
  }

  getHolders(asset: string, limit: number = 20, cursor?: string): HoldersConnection {
    const assetMap = this.holdersByAsset.get(asset);
    if (!assetMap) {
      return { nodes: [], pageInfo: { hasNextPage: false, endCursor: null, totalCount: 0 } };
    }

    const allHolders: Holder[] = Array.from(assetMap.entries()).map(([address, balance]) => ({
      address,
      balance: balance.toString(),
    }));

    return paginateNodes(allHolders, limit, cursor, (h) => h.address);
  }

  getBalance(asset: string, owner: string): BalanceRecord | null {
    const assetMap = this.holdersByAsset.get(asset);
    const bal = assetMap?.get(owner) ?? 0n;
    return { asset, owner, balance: bal.toString() };
  }

  getBalances(asset: string, limit: number = 20, cursor?: string): BalancesConnection {
    const assetMap = this.holdersByAsset.get(asset);
    if (!assetMap) {
      return { nodes: [], pageInfo: { hasNextPage: false, endCursor: null, totalCount: 0 } };
    }

    const records: BalanceRecord[] = Array.from(assetMap.entries()).map(([owner, bal]) => ({
      asset,
      owner,
      balance: bal.toString(),
    }));

    return paginateNodes(records, limit, cursor, (r) => r.owner);
  }

  getProposals(limit: number = 20, cursor?: string): ProposalsConnection {
    return paginateNodes(this.proposals, limit, cursor, (p) => p.id);
  }

  getProposal(id: string): ProposalRecord | null {
    return this.proposals.find((p) => p.id === id) ?? null;
  }
}

function paginateNodes<T>(
  items: T[],
  limit: number,
  cursor: string | undefined,
  getId: (item: T) => string
): { nodes: T[]; pageInfo: PageInfo } {
  let startIndex = 0;
  if (cursor) {
    const decoded = Buffer.from(cursor, "base64").toString("utf-8");
    const found = items.findIndex((item) => getId(item) === decoded);
    if (found !== -1) {
      startIndex = found + 1;
    }
  }

  const pageItems = items.slice(startIndex, startIndex + limit);
  const hasNextPage = startIndex + limit < items.length;
  const lastItem = pageItems[pageItems.length - 1];
  const endCursor = lastItem ? Buffer.from(getId(lastItem), "utf-8").toString("base64") : null;

  return {
    nodes: pageItems,
    pageInfo: {
      hasNextPage,
      endCursor,
      totalCount: items.length,
    },
  };
}

export interface GraphQLRequest {
  query: string;
  variables?: Record<string, unknown>;
}

export interface GraphQLResponse<T = unknown> {
  data?: T;
  errors?: { message: string }[];
}

export class IndexerGraphQLServer {
  constructor(private store: IndexerStore) {}

  async execute(request: GraphQLRequest): Promise<GraphQLResponse> {
    const { query, variables = {} } = request;
    const trimmed = query.trim();

    try {
      if (trimmed.includes("holders")) {
        const asset = (variables.asset as string) || this.extractArgument(trimmed, "asset") || "";
        const limit = Number(variables.limit || this.extractArgument(trimmed, "limit") || 20);
        const cursor = (variables.cursor as string) || this.extractArgument(trimmed, "cursor");
        const holders = this.store.getHolders(asset, limit, cursor);
        return { data: { holders } };
      }

      if (trimmed.includes("proposals")) {
        const limit = Number(variables.limit || this.extractArgument(trimmed, "limit") || 20);
        const cursor = (variables.cursor as string) || this.extractArgument(trimmed, "cursor");
        const proposals = this.store.getProposals(limit, cursor);
        return { data: { proposals } };
      }

      if (trimmed.includes("balances")) {
        const asset = (variables.asset as string) || this.extractArgument(trimmed, "asset") || "";
        const limit = Number(variables.limit || this.extractArgument(trimmed, "limit") || 20);
        const cursor = (variables.cursor as string) || this.extractArgument(trimmed, "cursor");
        const balances = this.store.getBalances(asset, limit, cursor);
        return { data: { balances } };
      }

      if (trimmed.includes("balance")) {
        const asset = (variables.asset as string) || this.extractArgument(trimmed, "asset") || "";
        const owner = (variables.owner as string) || this.extractArgument(trimmed, "owner") || "";
        const balance = this.store.getBalance(asset, owner);
        return { data: { balance } };
      }

      return { errors: [{ message: "Unsupported query root field" }] };
    } catch (err) {
      return { errors: [{ message: (err as Error).message }] };
    }
  }

  private extractArgument(query: string, name: string): string | undefined {
    const match = query.match(new RegExp(`${name}\\s*:\\s*"?([^"\\s,)]+)"?`));
    return match ? match[1] : undefined;
  }
}
