# @stellarforge-protocol/indexer-graphql

GraphQL API over the StellarForge indexer exposing token balances, asset holders, and governance proposals.

## Features

- Query token holders for an asset with pagination
- Query account balances
- Query governance proposals with pagination
- Filter and slice with cursors (`hasNextPage`, `endCursor`, `totalCount`)

## Usage

```typescript
import { InMemoryIndexerStore, IndexerGraphQLServer } from "@stellarforge-protocol/indexer-graphql";

const store = new InMemoryIndexerStore();
store.setBalance("CASSET123", "GUSER123", 10000000n);

const server = new IndexerGraphQLServer(store);

const response = await server.execute({
  query: `
    query GetHolders($asset: String!, $limit: Int) {
      holders(asset: $asset, limit: $limit) {
        nodes {
          address
          balance
        }
        pageInfo {
          hasNextPage
          endCursor
          totalCount
        }
      }
    }
  `,
  variables: { asset: "CASSET123", limit: 10 },
});

console.log(response.data.holders.nodes);
```
