# @stellarforge-protocol/sdk

TypeScript client for the [StellarForge](https://github.com/0xLizTech/stellarforge)
Real World Asset tokenization protocol on Stellar/Soroban.

Clients for all four Phase 1 contracts: `rwa-asset`, `compliance`, `registry`
and `governance`.

```bash
npm install @stellarforge-protocol/sdk
```

Requires Node.js 22.12 or newer, which is what `@stellar/stellar-sdk` 17 requires. Its CommonJS build loads ESM-only dependencies with `require()`, and Node supports that without a flag only from 22.12.

## Reading

Every read is a simulation. Nothing is written, nothing is signed, and no
account is needed.

```typescript
import { RwaAssetClient, TESTNET_CONFIG, fromStroops } from "@stellarforge-protocol/sdk";

const client = new RwaAssetClient({
  ...TESTNET_CONFIG,
  contracts: { rwaAsset: "C..." },
});

const meta = await client.metadata();
const supply = await client.totalSupply();

console.log(`${meta.symbol}: ${fromStroops(supply, meta.decimals)}`);
```

Amounts are `bigint`, because the contracts store them as `i128` and a `number`
would lose precision silently. Use `toStroops` and `fromStroops` to convert.
`toStroops` throws rather than guess. It rejects anything but plain decimal
notation, more decimal places than the asset allows, and a `number` that is not
a safe integer, so pass user-entered amounts as strings.

## Writing

Every write comes in two forms.

**`build*Tx` returns a prepared but unsigned transaction** for a wallet to
sign. No secret reaches this library, which is the only form usable in a
browser.

```typescript
const tx = await client.buildTransferTx(from, to, toStroops("100", meta.decimals));
const signedXdr = await freighter.signTransaction(tx.toXdr(), { networkPassphrase });
```

**The bare method signs and submits**, for server-side automation.

```typescript
const client = new RwaAssetClient({ ...TESTNET_CONFIG, contracts, signerSecret });
const { hash, ledger } = await client.transfer(from, to, amount);
```

A bare write waits for the transaction to settle, for up to its 180-second
validity window plus 30 seconds. If it has not settled by then, the error type
says whether retrying is safe:

- `TransactionExpiredError`: the window closed without the transaction being
  included. It can never land, so rebuild and resubmit.
- `TransactionOutcomeUnknownError`: it may still land. Look up `err.hash`
  before retrying, or the write may execute twice.

Two things worth knowing before you use either:

- **The authorizing address must also source the transaction.** Its signature
  is what satisfies the contract's `require_auth`. Paying fees from a separate
  account is not supported yet.
- **Building simulates.** A call the contract would reject — a bad amount, a
  paused asset, a party failing compliance — fails while building, with the
  contract's own error, before anything is signed or submitted.

## Clients

| Client | Contract | Notes |
|---|---|---|
| `RwaAssetClient` | `rwa-asset` | Balances, metadata, allowances; mint, burn, burnFrom, transfer, approve (with an expiry ledger), transferFrom |
| `ComplianceClient` | `compliance` | `isCompliant` is the pure query, never the TTL-extending `screen` |
| `RegistryClient` | `registry` | Asset directory, paged with `listAssets(start, limit)` or walked with `listAllAssets()`; `register` and `setActive` are admin-only |
| `GovernanceClient` | `governance` | Proposals and voting — **see the warning below** |

### Governance results are advisory in Phase 1

The governance contract accepts whatever vote weight its caller declares and
verifies it against nothing, and `finalize` applies no quorum. **Any address
can carry any proposal.** Do not gate anything privileged on an outcome read
through `GovernanceClient`.

Weight is deliberately unverified rather than read from a live balance, which
would be worse: a holder could vote, transfer the same tokens onward, and vote
again from the recipient. Correct weighting needs balances snapshotted at
proposal creation, which arrives with `SFORGE` in Phase 2/3.

## Status

Phase 1. The contracts are unaudited and not deployed to mainnet. See the
[roadmap](https://github.com/0xLizTech/stellarforge/blob/main/ROADMAP.md).

## License

Apache-2.0
