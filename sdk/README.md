# @stellarforge-protocol/sdk

TypeScript client for the [StellarForge](https://github.com/0xLizTech/stellarforge)
Real World Asset tokenization protocol on Stellar/Soroban.

Clients for the Phase 1 contracts (`rwa-asset`, `compliance`, `registry`,
`governance`) and the Phase 2 `oracle-adapter` SEP-40 price feed.

```bash
npm install @stellarforge-protocol/sdk
```

Requires Node.js 22.12 or newer, which is what `@stellar/stellar-sdk` 17 requires. Its CommonJS build loads ESM-only dependencies with `require()`, and Node supports that without a flag only from 22.12.

Supported versions: **Node.js 22.12 or newer and 24 (current LTS)**. CI
(`.github/workflows/sdk-ci.yml`) tests Node 22.13 and 24. Node.js 20 and
older are not supported and are not tested — they are below the `engines` floor
and fail at import time.

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
would lose precision silently. Use `toStroops` and `fromStroops` to convert between
human-readable decimal strings and base units. Both accept a custom `decimals`
parameter (0 to 18, defaulting to Stellar's 7). `toStroops` throws rather than guess:
it rejects anything but plain decimal notation, more decimal places than the asset allows,
and a `number` that is not a safe integer.

To format amounts for UI display, use `formatAmount(amount, decimals, locale)`:

```typescript
import { formatAmount } from "@stellarforge-protocol/sdk";

// Formats base units (bigint) into grouped, locale-aware strings
formatAmount(1_234_567_890_123n, 7, "en-US"); // "123,456.7890123"
formatAmount(1_234_567_890_123n, 7, "de-DE"); // "123.456,7890123"

// Exact for 1e30 values and beyond without floating-point precision loss
formatAmount(10n ** 30n, 0, "en-US"); // "1,000,000,000,000,000,000,000,000,000,000"
```

## Runnable testnet examples

The [`examples/`](examples/) directory has complete mint, transfer, KYC,
governance proposal, and vote flows. They submit real testnet transactions, so
use a funded testnet signer and the matching deployed contract IDs. Never use a
mainnet secret. Install dependencies with `npm ci` in this directory, then run
the chosen script with `npm run example -- examples/mint.ts` (replace `mint.ts`
with `transfer.ts`, `kyc.ts`, `propose.ts`, or `vote.ts`).

Every example needs `SIGNER_SECRET` and the contract-specific ID:
`RWA_ASSET_CONTRACT_ID`, `COMPLIANCE_CONTRACT_ID`, or
`GOVERNANCE_CONTRACT_ID`. `SOROBAN_RPC_URL` optionally overrides the public testnet RPC. CI
typechecks all examples without sending transactions using
`npm run examples:check`.

| Example | Required action variables |
|---|---|
| `mint.ts` | `MINT_TO_ADDRESS`, `MINT_AMOUNT` |
| `transfer.ts` | `TRANSFER_TO_ADDRESS`, `TRANSFER_AMOUNT` |
| `kyc.ts` | `KYC_SUBJECT_ADDRESS`, `KYC_LEVEL` |
| `propose.ts` | `PROPOSAL_TITLE`, `PROPOSAL_DESCRIPTION_SHA256` (optional; defaults shown in code) |
| `vote.ts` | `PROPOSAL_ID`, `VOTE_SUPPORT` |

Alternate runtime smoke coverage is provided for Deno 2 and Bun 1. It imports
the built package and performs a read through an in-memory testnet RPC response;
it does not need a deployed contract or send a transaction.

## Networks

`TESTNET_CONFIG` includes SDF's public testnet RPC. `MAINNET_CONFIG` has no RPC
URL, because SDF doesn't run a public mainnet RPC. Supply one from an
[RPC provider](https://developers.stellar.org/docs/data/apis/rpc/providers); TypeScript won't compile a mainnet config without it.

```typescript
import { MAINNET_CONFIG, RwaAssetClient } from "@stellarforge-protocol/sdk";

const client = new RwaAssetClient({
  ...MAINNET_CONFIG,
  rpcUrl: "https://your-mainnet-rpc-provider.example",
  contracts: { rwaAsset: "C..." },
});
```

## Writing

Every write comes in two forms.

**`build*Tx` returns a prepared but unsigned transaction** for a wallet to
sign. No secret reaches this library, which is the only form usable in a
browser.

```typescript
const tx = await client.buildTransferTx(from, to, toStroops("100", meta.decimals));
const signedXdr = await freighter.signTransaction(tx.toXdr(), { networkPassphrase });
```

**The bare method signs and submits** through a configured signer. `KeypairSigner`
supports server-side automation; `FreighterSigner` adapts Freighter's
`signTransaction` API without adding a wallet dependency to the SDK.

```typescript
import { FreighterSigner, RwaAssetClient, TESTNET_CONFIG } from "@stellarforge-protocol/sdk";

const signer = new FreighterSigner(publicKey, (xdr, options) =>
  freighter.signTransaction(xdr, options),
);
const client = new RwaAssetClient({ ...TESTNET_CONFIG, contracts, signer });
const { hash, ledger } = await client.transfer(from, to, amount);
```

### Freighter wallet & network passphrases

A runnable browser sample page and TypeScript adapter live in
[`examples/freighter-wallet/`](examples/freighter-wallet/README.md)
([`index.html`](examples/freighter-wallet/index.html),
[`freighter-adapter.ts`](examples/freighter-wallet/freighter-adapter.ts)).

Stellar transactions are hashed with the target network's passphrase
(`TESTNET_CONFIG.networkPassphrase` for Testnet, `MAINNET_CONFIG.networkPassphrase`
for Mainnet). `FreighterSigner` forwards the client's `networkPassphrase` to
Freighter's `signTransaction(xdr, { address, networkPassphrase })` and decodes the
signed XDR with that same passphrase. In a browser dApp, also check
`freighter.getNetworkDetails().networkPassphrase` against your client config before
prompting the user to sign, and ask them to switch networks in Freighter if they do
not match.

For server-side keypairs, use `new KeypairSigner(keypair)` or
`KeypairSigner.fromSecret(secret)`. The older `signerSecret` config option remains
supported for compatibility. Every submitting write, including sponsored
submissions, uses the configured signer.

A bare write waits for the transaction to settle, for up to its 180-second
validity window plus 30 seconds. If it has not settled by then, the error type
says whether retrying is safe:

- `TransactionExpiredError`: the window closed without the transaction being
  included. It can never land, so rebuild and resubmit.
- `TransactionOutcomeUnknownError`: it may still land. Look up `err.hash`
  before retrying, or the write may execute twice.

Two things worth knowing before you use either:

- **The authorizing address also sources the transaction** in these forms, so
  its signature satisfies the contract's `require_auth`. To have another account
  pay, see [Sponsored writes](#sponsored-writes).
- **Building simulates.** A call the contract would reject — a bad amount, a
  paused asset, a party failing compliance — fails while building, with a typed
  contract error, before anything is signed or submitted.

## Typed Contract Errors

Contract rejections (`Error(Contract, #N)`) are mapped to a typed error class
per contract — `RwaAssetError`, `ComplianceError`, `RegistryError`,
`GovernanceError`, `OracleAdapterError`, and `VaultContractError` — all
extending `ContractError`. Each exposes the numeric discriminant (`code`) and
its readable enum variant name (`codeName`):

```typescript
import { RwaAssetError, RwaAssetErrorCode } from "@stellarforge-protocol/sdk";

try {
  await client.transfer(from, to, amount);
} catch (err) {
  if (err instanceof RwaAssetError) {
    console.error(`${err.codeName} (#${err.code})`);
    if (err.code === RwaAssetErrorCode.ContractPaused) {
      // handle paused contract
    }
  }
}
```

### Multiplexed Addresses (SEP-23)

Transfer operations (`transfer`, `buildTransferTx`, `buildSponsoredTransferTx`) accept
both standard Stellar addresses (`G...` or contract `C...`) and multiplexed addresses
(`M...`, SEP-23). When transferring to an `M...` address, value is credited to the underlying
`G...` address on-chain and the 64-bit muxed ID travels in the transfer contract event
for recipient routing.

Use `isValidMuxedAddress` and `parseMuxedAddress` to validate and inspect M-addresses:

```typescript
import { isValidMuxedAddress, parseMuxedAddress } from "@stellarforge-protocol/sdk";

if (isValidMuxedAddress(to)) {
  const { address, id } = parseMuxedAddress(to)!;
  console.log(`Routing to underlying account ${address} with memo ID ${id}`);
}
```

## Clients

| Client | Contract | Notes |
|---|---|---|
| `RwaAssetClient` | `rwa-asset` | Balances, metadata, allowances; mint, burn, burnFrom, transfer, approve (with an expiry ledger), transferFrom. Admin: setIssuer, setPaused, updateMetadata, setCompliance |
| `ComplianceClient` | `compliance` | `isCompliant` is the pure query, never the TTL-extending `screen`; `getKyc` reads a stored record. Admin: setKyc, revokeKyc |
| `RegistryClient` | `registry` | Asset directory, paged with `listAssets(start, limit)` or walked with `listAllAssets()`; `register` and `setActive` are admin-only |
| `GovernanceClient` | `governance` | Proposals (walked with `proposals()`) and voting — **see the warning below** |
| `VaultClient` | `vault` | Real estate / RWA share vault: config, balances, historical checkpoints, NAV, sponsored deposits/redeems, and typed event decoders |
| `GovernanceClient` | `governance` | Proposals and voting — **see the warning below** |
| `VaultClient` | `vault` | Real estate / RWA share vault: config, balances, previews (`previewDeposit`/`previewRedeem`), historical checkpoints (`balanceAt`/`totalSupplyAt`/`checkpointCount`/`supplyCheckpointCount`), NAV, admin (`setPaused`/`setOracles`/`transferAdmin`), sponsored deposits/redeems, typed errors (`VaultErrorCode`/`parseVaultError`), and event decoders |
| `OracleAdapterClient` | `oracle-adapter` | SEP-40 NAV feed — **see [Oracle adapter](#oracle-adapter)** |

## Vault & Auth-Tree Verification

`VaultClient` handles tokenized share vaults where users deposit underlying RWA tokens in exchange for vault shares:

```typescript
import {
  VaultClient,
  TESTNET_CONFIG,
  sharesForDeposit,
  underlyingForRedeem,
  maxRedeemable,
  describeAuthTree,
  decodeVaultEvents,
} from "@stellarforge-protocol/sdk";

const client = new VaultClient({
  ...TESTNET_CONFIG,
  contracts: { vault: "C...", rwaAsset: "C..." },
});

// 1. Typed reads & NAV
const config = await client.config();
const navData = await client.nav(); // Throws NavRefusalError if oracles diverge or are stale
const navPerShare = await client.navPerShare();

// 2. Pure rounding and lockup checks
const shares = sharesForDeposit(assets, config.exchangeRate);
const redeemableBalance = maxRedeemable(userBalance, lockedUntilLedger, currentLedger);

// 3. Direct deposit and redeem (build*Tx for wallets, bare method for signers)
const depositTx = await client.buildDepositTx(depositor, depositAmount);
const depositRes = await client.deposit(depositor, depositAmount);
const redeemRes = await client.redeem(depositor, shares);

// 4. Sponsored deposit / redeem with auth-tree verification
const sponsored = await client.buildSponsoredDepositTx(depositor, depositAmount, {
  feeSource: relayer,
});
const sponsoredRedeem = await client.buildSponsoredRedeemTx(depositor, shares, {
  feeSource: relayer,
});

// The authorizer can inspect the entire nested call tree:
// e.g. "Root: <vault>.deposit(depositor, amount) └── <rwa>.transfer(depositor, vault, amount)"
console.log(describeAuthTree(sponsored.authEntries[0]));

// Deep verification runs during finalizeSponsoredTx: verifies root calls AND
// nested child invocations, refusing any tampering of amounts, contracts, or methods.
const finalizedTx = await client.finalizeSponsoredTx(sponsored, signedEntries);

// 4. Typed event decoding
// Query events from RPC for the vault contract using txResult.ledger
const eventsResponse = await server.getEvents({
  startLedger: txResult.ledger,
  filters: [{ type: "contract", contractIds: [vaultAddress] }],
});
const events = decodeVaultEvents(eventsResponse);
for (const event of events) {
  if (event.type === "deposit") {
    console.log(`Deposited ${event.assets} assets for ${event.shares} shares by ${event.depositor}`);
  }
}
```

## Oracle adapter

`OracleAdapterClient` talks to the Phase 2 `oracle-adapter` contract: a SEP-40
price feed appraisers report NAV into. Set `contracts.oracleAdapter` to the
deployed contract id.

```typescript
import {
  OracleAdapterClient,
  TESTNET_CONFIG,
  tickOf,
  checkDeviation,
  isFresh,
  toDecimalString,
  authorizeEntries,
  DEFAULT_AUTH_VALIDITY_LEDGERS,
} from "@stellarforge-protocol/sdk";

const client = new OracleAdapterClient({
  ...TESTNET_CONFIG,
  contracts: { oracleAdapter: "C..." },
});

const base = await client.base(); // { type: "other", symbol: "USD" }
const last = await client.lastprice({ type: "stellar", address: rwaId });
const config = await client.assetConfig({ type: "stellar", address: rwaId });

// Pure helpers mirror the contract's tick rounding and deviation check.
const tick = tickOf(observedAtSecs, await client.resolution());
const withinLimit = last && config
  ? checkDeviation(last.price, nextPrice, config.maxDeviationBps)
  : true;
```

**Reads:** `base`, `assets`, `decimals`, `resolution`, `price`, `prices`,
`lastprice` (SEP-40), plus `admin`, `isReporter`, `assetConfig`. Missing
optional prices decode as `null`.

**Writes**, each as `build*Tx` plus a submitting form: `report`,
`overridePrice`, `addAsset`, `setMaxDeviation`, `setReporter`.

**Sponsored reporting** (reporter authorizes; relayer pays):

```typescript
const sponsored = await client.buildSponsoredReportTx(
  reporter,
  { type: "stellar", address: rwaId },
  price,
  timestamp,
  { feeSource: relayer },
);
const signed = await authorizeEntries(
  sponsored.authEntries,
  reporterKeypair,
  sequence + DEFAULT_AUTH_VALIDITY_LEDGERS,
  networkPassphrase,
);
const { hash } = await client.submitSponsoredTx(sponsored, signed);
```

`buildSponsoredOverridePriceTx` is the same flow for an admin override that
bypasses the deviation limit. Assets encode as the SEP-40 tuple variants
`["Stellar", Address]` / `["Other", Symbol]`.

The `stellarforge-nav-reporter` CLI that signs appraisals and submits sponsored
reports is tracked separately under #62 and is not part of this package yet.

## Sponsored writes

One account can source and pay for a write while another address only
authorizes it, for example an issuer covering fees for its holders. The
authorizer signs an authorization entry rather than the transaction, so the two
keys can live on different machines.

```typescript
import { rpc } from "@stellar/stellar-sdk";
import { DEFAULT_AUTH_VALIDITY_LEDGERS, authorizeEntries } from "@stellarforge-protocol/sdk";

// Fee payer: build the call. Nothing is signed yet.
const sponsored = await client.buildSponsoredTransferTx(holder, recipient, amount, {
  feeSource: relayer,
});

// Authorizer: sign its entries, valid until a ledger it chooses.
const { sequence } = await new rpc.Server(rpcUrl).getLatestLedger();
const signed = await authorizeEntries(
  sponsored.authEntries,
  holderKeypair,
  sequence + DEFAULT_AUTH_VALIDITY_LEDGERS,
  networkPassphrase,
);

// Fee payer: attach the signatures, then sign and submit.
const { hash } = await client.submitSponsoredTx(sponsored, signed);
```

- **Transport.** Entries cross machines as base64 through `authEntriesToXdr`
  and `authEntriesFromXdr`. `finalizeSponsoredTx` returns the prepared
  transaction instead of submitting it, for a fee payer signing with a wallet.
- **Checks before submitting.** Finalizing checks every signed entry against
  the one built, and requires its expiry to be at least
  `MIN_AUTH_REMAINING_LEDGERS` (40) and at most `MAX_AUTH_VALIDITY_LEDGERS`
  (17,280) ledgers ahead. `authorizeEntries` refuses a key the entries don't
  name.
- **Network rejections.** The network refuses a signature made with the wrong
  key, a replayed entry, and an entry reused for other arguments.
- **The authorizer must have an account.** It must already exist on the
  network, even though it pays nothing.

Admin handover (#125, see ADR-006) is a two-step
`buildProposeAdminTx(admin, newAdmin)` / `buildAcceptAdminTx(newAdmin)` pair on
every client, not a sponsored call: each needs only one signature, so neither
needs this sponsorship flow at all. The current admin proposes in its own
transaction; only the proposed admin's own signature can accept it in a
separate one, which is also what makes `cancelAdminTransfer(admin)` a safe
unilateral escape hatch for a mistaken or no-longer-wanted proposal.

### Governance results are advisory in Phase 1

The governance contract accepts whatever vote weight its caller declares and
verifies it against nothing. `finalize` does apply quorum and a pass
threshold now (both admin-configurable via `setQuorum`/`setPassThresholdBps`),
but against that same unverified weight, so **any address can still carry any
proposal** by declaring enough weight. Do not gate anything privileged on an
outcome read through `GovernanceClient`.

Weight is deliberately unverified rather than read from a live balance, which
would be worse: a holder could vote, transfer the same tokens onward, and vote
again from the recipient. Correct weighting needs balances snapshotted at
proposal creation, which arrives with `SFORGE` in Phase 2/3.

## SEP-41 Token Compatibility

`RwaAssetClient` conforms to the Stellar Soroban [SEP-41 Token Interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md), allowing RWA tokens to integrate seamlessly with standard Soroban wallets and DEXs.

| SEP-41 Method | Signature | Supported | Description |
|---|---|---|---|
| `allowance` | `allowance(owner, spender) -> i128` | Yes | Reads spender's remaining allowance |
| `approve` | `approve(owner, spender, amount, liveUntilLedger)` | Yes | Grants allowance spendable until `liveUntilLedger` |
| `balance` | `balance(id) -> i128` | Yes | Reads account balance in base units (stroops) |
| `transfer` | `transfer(from, to, amount)` | Yes | Transfers tokens; accepts standard and MuxedAccount (`M...`) destinations |
| `transfer_from` | `transferFrom(spender, from, to, amount)` | Yes | Spender-authorized transfer from `from` |
| `burn` | `burn(from, amount)` | Yes | Holder-authorized token burn |
| `burn_from` | `burnFrom(spender, from, amount)` | Yes | Spender-authorized token burn drawing on allowance |
| `decimals` | `decimals() -> u32` | Yes | Returns asset decimals (0 to 18) |
| `name` | `name() -> String` | Yes | Returns asset display name |
| `symbol` | `symbol() -> String` | Yes | Returns asset ticker symbol |

### Interoperability Notes

- **Compliance Screening**: Transfer operations (`transfer`, `transferFrom`, `mint`) verify that both sender and recipient are compliant with the asset's configured compliance contract (if active) and not frozen/locked.
- **Multiplexed Addresses**: Destinations (`to`) in `transfer` accept SEP-23 M-addresses; the 64-bit muxed ID is emitted in contract events for off-chain accounting.
- **Generic Clients**: The SDK exports `Sep41TokenClient`, an interface that any SEP-41 compliant token client satisfies.

## Subpath Exports & Tree-Shaking

The SDK is marked with `"sideEffects": false` and provides dedicated subpath exports so modern bundlers can shake out unreferenced modules. Importing a specific client or utility bundle ensures unneeded contract clients (such as governance or vault logic) are omitted from your application bundle.

Available export paths:
- `@stellarforge-protocol/sdk` (full bundle with all clients and utilities)
- `@stellarforge-protocol/sdk/client` (`RwaAssetClient`, `ComplianceClient`)
- `@stellarforge-protocol/sdk/governance` (`GovernanceClient`, types)
- `@stellarforge-protocol/sdk/registry` (`RegistryClient`, types)
- `@stellarforge-protocol/sdk/vault` (`VaultClient`, pure helpers, event decoders)
- `@stellarforge-protocol/sdk/oracle` (`OracleAdapterClient`, pure helpers, codecs)
- `@stellarforge-protocol/sdk/sponsored` (sponsored write auth entry helpers)
- `@stellarforge-protocol/sdk/errors` (structured error classes)
- `@stellarforge-protocol/sdk/types` (type definitions and network configurations)
- `@stellarforge-protocol/sdk/utils` (pure decimal and address utilities)

```typescript
// Granular subpath import - pulls in only the RWA client
import { RwaAssetClient } from "@stellarforge-protocol/sdk/client";
import { toStroops } from "@stellarforge-protocol/sdk/utils";
```

## Bundle Size Tracking & Budgets

Bundle sizes are tracked with `size-limit` to prevent bundle bloat in browser dApps. Budgets are strictly enforced in CI:
- Full SDK (CJS): `<= 100 KB`
- Full SDK (ESM index): `<= 5 KB`
- RWA Client (CJS): `<= 40 KB`
- Vault Client (CJS): `<= 45 KB`
- Oracle Client (CJS): `<= 40 KB`
- Governance Client (CJS): `<= 30 KB`

Run `npm run size` locally to measure and verify bundles against the configured limits.

## Browser Support & Compatibility

The SDK is designed to run in both Node.js (22.12+) and modern browser environments without requiring Node built-ins or silent polyfills.

### Supported Browsers
- **Chrome / Chromium** 109+
- **Edge** 109+
- **Firefox** 115+ (ESR)
- **Safari / WebKit** 16.4+

### Browser Environment Principles
- **No Silent Polyfills:** The SDK avoids Node-specific APIs (`fs`, `child_process`, `net`, `Buffer`). It relies on standard Web Platform APIs (`fetch`, Web Crypto via `crypto.subtle`, `BigInt`).
- **Wallet-Centric Signing:** Browsers should never handle raw signer secrets. Instead, use the unsigned `build*Tx` methods (e.g. `client.buildTransferTx`, `client.buildSponsoredDepositTx`) to prepare transactions, and pass the resulting XDR to browser extension wallets (such as Freighter) for user approval and signing:
  ```typescript
  const tx = await client.buildTransferTx(from, to, amount);
  const signedXdr = await window.freighter.signTransaction(tx.toXdr(), { networkPassphrase });
  ```
- **Automated Verification:** The browser suite runs under a DOM/Web runner (`happy-dom`) via `npm run test:browser` to catch regressions where Node-only APIs might slip in.

## Status

Phase 1. The contracts are unaudited and not deployed to mainnet. See the
[roadmap](https://github.com/0xLizTech/stellarforge/blob/main/ROADMAP.md).

## License

Apache-2.0
