# Freighter Wallet Signer Example

Browser example demonstrating how to pair [`@stellar/freighter-api`](https://docs.freighter.app/) with `FreighterSigner` from `@stellarforge-protocol/sdk`.

## Files

- [`index.html`](index.html) — Self-contained sample page that connects Freighter, validates the active network passphrase, and submits an `rwa-asset` transfer.
- [`freighter-adapter.ts`](freighter-adapter.ts) — Reusable TypeScript helper for bundled dApps (Vite, Next.js, Webpack, etc.).

## Running the Sample Page

1. Install the [Freighter](https://www.freighter.app/) browser extension and switch it to **Testnet**.
2. Serve this directory over HTTP (browser extensions do not inject into `file://` URLs):

```bash
cd sdk/examples/freighter-wallet
npx serve .
```

3. Open the printed local URL in your browser, click **Connect Freighter**, enter a deployed `rwa-asset` contract ID and recipient, and submit a transfer.

## Network Passphrase Handling

Every Stellar transaction is hashed with its network's passphrase before signing:

| Network | Passphrase | SDK Preset |
| --- | --- | --- |
| Testnet | `Test SDF Network ; September 2015` | `TESTNET_CONFIG.networkPassphrase` |
| Mainnet | `Public Global Stellar Network ; September 2015` | `MAINNET_CONFIG.networkPassphrase` |

Two safeguards keep signing consistent:

1. **Pre-flight check with `getNetworkDetails()`:** Before constructing `FreighterSigner`, compare `freighter.getNetworkDetails().networkPassphrase` against your SDK config's `networkPassphrase`. If they differ, prompt the user to switch networks in Freighter rather than letting them sign a transaction that Soroban RPC will reject with `tx_bad_auth`.
2. **Explicit passphrase on `signTransaction()`:** `FreighterSigner` always forwards the client's `networkPassphrase` into `freighter.signTransaction(xdr, { address, networkPassphrase })` and parses the returned `signedTxXdr` with that same passphrase.
