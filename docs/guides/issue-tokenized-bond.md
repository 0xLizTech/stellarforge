# Recipe: Issue a Tokenized Bond

This recipe walks through issuing a fixed-rate corporate bond as a StellarForge `rwa-asset` on **testnet**, using the TypeScript SDK (`@stellarforge-protocol/sdk`): KYC the issuer and an investor in the `compliance` contract, wire compliance screening into the bond, set the bond's terms as metadata, register it, and mint the first tranche.

The full runnable script lives in [`sdk/examples/issue-tokenized-bond.ts`](../../sdk/examples/issue-tokenized-bond.ts); this guide explains each step. The bond in the example is *Acme Corp 5.00% Senior Bond 2030* (`ACME30`), capped at 1,000,000 bonds.

---

## 1. Prerequisites

- **Deployed contracts.** A deployment of `rwa-asset` (one instance per bond), `compliance`, and optionally `registry` on testnet. Deploy with `./scripts/deploy.sh` (see the [Vault Operations Guide](vault-operations.md) for the deterministic deployment flow) and note the contract IDs.
- **Two funded testnet accounts.** An *admin* account (the contracts' admin) and an *issuer* account. Create them with `stellar keys generate --fund` or fund existing keys at [friendbot](https://friendbot.stellar.org). You also need the public key of the first *investor*.
- **The SDK.**

```bash
npm install @stellarforge-protocol/sdk
```

- **Environment.**

| Variable | What it is |
|---|---|
| `RWA_ASSET_ID` | Contract ID of this bond's `rwa-asset` instance |
| `COMPLIANCE_ID` | Contract ID of the `compliance` contract |
| `REGISTRY_ID` | Contract ID of the `registry` contract (optional) |
| `ADMIN_SECRET` | Secret key of the contracts' admin |
| `ISSUER_SECRET` | Secret key of the issuer account |
| `INVESTOR_ADDRESS` | Public key of the first investor |
| `ISSUE_AMOUNT` | Whole bonds in the first tranche (default `250000`) |

> The example signs with raw secret keys because it is a server-side recipe. The SDK accepts a `signerSecret` for automated/server contexts only — in production, build transactions with the `build…Tx` methods and sign them with a wallet (Freighter, hardware wallet) instead.

Bond amounts use Stellar's standard 7 decimals, so 1 bond = `10^7` base units:

```ts
const units = (whole: number): bigint => BigInt(whole) * 10n ** 7n;
```

---

## 2. Configure the Clients

A write transaction's signer must be its source account, and admin calls are sourced by the admin while `mint` is sourced by the issuer — so build one config per role, sharing the contract addresses:

```ts
import {
  ComplianceClient,
  RegistryClient,
  RwaAssetClient,
  TESTNET_CONFIG,
} from "@stellarforge-protocol/sdk";
import type { StellarForgeConfig } from "@stellarforge-protocol/sdk";

const contracts = {
  rwaAsset: RWA_ASSET_ID,
  compliance: COMPLIANCE_ID,
  ...(REGISTRY_ID ? { registry: REGISTRY_ID } : {}),
};
const asAdmin: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: ADMIN_SECRET };
const asIssuer: StellarForgeConfig = { ...TESTNET_CONFIG, contracts, signerSecret: ISSUER_SECRET };

const compliance = new ComplianceClient(asAdmin);
const assetAsAdmin = new RwaAssetClient(asAdmin);
const assetAsIssuer = new RwaAssetClient(asIssuer);
```

---

## 3. Compliance Setup

No one can receive the bond until the `compliance` contract holds a KYC record for them at the level the bond will require. Register the issuer as an accredited entity (level 3) and the investor with full KYC (level 2). `expiresAt: 0` means the record never expires.

```ts
await compliance.setKyc(admin, issuer, { jurisdiction: "US", level: 3, expiresAt: 0 });
await compliance.setKyc(admin, investor, { jurisdiction: "US", level: 2, expiresAt: 0 });

// Fail fast instead of discovering a missing record at mint time.
if (!(await compliance.isCompliant(investor, 2))) {
  throw new Error("Investor is not compliant at level 2 after setKyc — aborting issuance.");
}
```

Then point the bond at the compliance contract and set the minimum level both parties to any mint or transfer must hold:

```ts
await assetAsAdmin.setCompliance(admin, COMPLIANCE_ID, 2);
```

Two details worth knowing:

- Level 0 is **not** "screening off" — every party must still hold a record. Only `setCompliance(admin, null, …)` disables screening.
- Records are per address: every future investor needs a `setKyc` call before they can receive bonds, and `revokeKyc` immediately blocks an address from receiving more.

Finally, authorise the issuer account to mint:

```ts
await assetAsAdmin.setIssuer(admin, issuer, true);
```

---

## 4. Set the Bond's Terms (Metadata)

The bond's terms live in the asset metadata. `legalDocHash` anchors the offering document on-chain — here it is the SHA-256 of the term sheet — and `maxSupply` hard-caps the programme at 1,000,000 bonds:

```ts
import { createHash } from "node:crypto";

const legalDocHash = createHash("sha256").update(termSheetText).digest("hex");

await assetAsAdmin.updateMetadata(admin, {
  name: "Acme Corp 5.00% Senior Bond 2030",
  symbol: "ACME30",
  decimals: 7,
  assetClass: "debt",
  legalDocHash,
  maxSupply: units(1_000_000),
});
```

The contract guards these fields: `decimals` can never change after deployment, and `updateMetadata` can only tighten the supply cap, never raise it (raising a cap is a governance flow, not an admin edit). On a brand-new deployment the same values are set at construction; `updateMetadata` is how you adjust them afterwards within those limits.

Optionally, register the bond so integrators can discover it:

```ts
await new RegistryClient(asAdmin).register(admin, {
  contract: RWA_ASSET_ID,
  assetClass: "debt",
  active: true,
});
```

---

## 5. Issue the First Tranche

Minting is an issuer call. This mints 250,000 bonds to the investor:

```ts
const issued = await assetAsIssuer.mint(issuer, investor, units(250_000));
console.log(`Minted in tx ${issued.hash}, ledger ${issued.ledger}`);
```

The mint succeeds only because every earlier step is in place: the issuer is authorised (`setIssuer`), the investor holds a level-2 KYC record, and the amount fits under `maxSupply`. Remove any one of them and the contract rejects the mint — that is the compliance model working as intended.

Verify the result with reads (these need no signer):

```ts
console.log(await assetAsIssuer.balance(investor));   // 2500000000000n base units
console.log(await assetAsIssuer.totalSupply());       // same, for a single-investor bond
console.log((await assetAsIssuer.metadata()).symbol); // "ACME30"
```

### Variations

- **Sponsored issuance.** To pay the fees for an investor who holds no XLM, use `buildSponsoredMintTx` and submit the sponsored transaction instead of `mint` — see the sponsored-writes section of the SDK README.
- **Coupon payments and redemption.** Coupons are off-chain cash flows (or transfers of a cash token by the issuer); at maturity, holders return bonds and the issuer calls `burn` to retire them, reducing `totalSupply` until the programme is fully redeemed.
- **Pausing.** `setPaused(admin, true)` freezes mint, burn, transfer and approve while leaving admin operations available — the emergency brake if offering documents are withdrawn.

---

## 6. Run It

```bash
cd sdk
npm install
RWA_ASSET_ID=C… COMPLIANCE_ID=C… REGISTRY_ID=C… \
ADMIN_SECRET=S… ISSUER_SECRET=S… INVESTOR_ADDRESS=G… \
npx tsx examples/issue-tokenized-bond.ts
```

Expected output ends with the investor's balance and the total supply, both `250000` ACME30 on a fresh bond. If a step fails, the error names the contract check that rejected it (not authorised, not compliant, cap exceeded) — work backwards through sections 3–4 in order.

For the read-side equivalent against an existing deployment, the SDK's opt-in smoke test (`SMOKE_RWA_ASSET_ID`, `SMOKE_COMPLIANCE_ID`, `npm run test:smoke` in `sdk/`) exercises the same clients against live testnet contracts.
