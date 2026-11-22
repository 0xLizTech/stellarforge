# Recipe: Tokenized Real Estate with KYC

This recipe walks through tokenizing a commercial real estate property as a StellarForge `rwa-asset` on **testnet**, using the TypeScript SDK (`@stellarforge-protocol/sdk`): configure KYC records for the property sponsor and investor in the `compliance` contract, bind compliance screening into the real estate asset token, configure asset metadata including property title deeds and offering term sheets, register the asset in the `registry`, and mint the initial tokenized equity tranche.

The full runnable script lives in [`sdk/examples/tokenized-real-estate.ts`](../../sdk/examples/tokenized-real-estate.ts); this guide explains each step. The property in the example is *Grandview Plaza Commercial REIT* (`REIT-BLD1`), representing fractional ownership in a commercial building with 7 decimals of precision, capped at 10,000,000 tokens.

---

## 1. Prerequisites

- **Deployed contracts.** A deployment of `rwa-asset` (representing the real estate token), `compliance`, and optionally `registry` on testnet. Deploy using `./scripts/deploy.sh` (see the [Vault Operations Guide](vault-operations.md) for deployment options) and note the deployed contract addresses.
- **Two funded testnet accounts.** An *admin* account (protocol administrator) and a *sponsor/issuer* account (property issuer). Create and fund accounts via `stellar keys generate --fund` or through [friendbot](https://friendbot.stellar.org). You also need the public key of the first *investor*.
- **The SDK.**

```bash
npm install @stellarforge-protocol/sdk
```

- **Environment.**

| Variable | What it is |
|---|---|
| `RWA_ASSET_ID` | Contract ID of the real estate property's `rwa-asset` instance |
| `COMPLIANCE_ID` | Contract ID of the `compliance` contract |
| `REGISTRY_ID` | Contract ID of the `registry` contract (optional) |
| `ADMIN_SECRET` | Secret key of the protocol admin |
| `ISSUER_SECRET` | Secret key of the property sponsor / issuer account |
| `INVESTOR_ADDRESS` | Public key of the KYC-cleared investor |
| `ISSUE_AMOUNT` | Whole tokens to issue in the initial offering (default `500000`) |

> Note: For automated/scripted flows, the SDK accepts `signerSecret`. In production or frontend browser integrations, always use `build*Tx` methods paired with hardware wallets or browser extensions (such as Freighter) to avoid exposing private keys.

Real estate token amounts use 7 decimal places (`1 token = 10^7 base units`):

```ts
const units = (whole: number): bigint => BigInt(whole) * 10n ** 7n;
```

---

## 2. Configure the Clients

Admin operations (such as KYC provisioning, setting compliance requirements, and registry entries) are authorized by the protocol admin, whereas `mint` calls must be authorized by the designated property issuer:

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

## 3. Compliance and KYC Screening

Real estate securities regulations (such as SEC Regulation D / Regulation S) require strict identity and accreditation verification:
- **Property Issuer/Sponsor:** Accredited institutional entity (`level: 3`).
- **Investor:** Verified identity and compliance check (`level: 2`).
- Setting `expiresAt: 0` specifies a non-expiring record, or set an epoch timestamp for annual recertification.

```ts
// Provision KYC records
await compliance.setKyc(admin, issuer, { jurisdiction: "US", level: 3, expiresAt: 0 });
await compliance.setKyc(admin, investor, { jurisdiction: "US", level: 2, expiresAt: 0 });

// Verify compliance before transacting
const isInvestorCompliant = await compliance.isCompliant(investor, 2);
if (!isInvestorCompliant) {
  throw new Error("Investor failed compliance verification at level 2.");
}
```

Next, link the `rwa-asset` contract to the compliance registry, enforcing a minimum verification level of 2 on all transfers and mint operations:

```ts
await assetAsAdmin.setCompliance(admin, COMPLIANCE_ID, 2);
```

Authorize the sponsor account to act as the token issuer:

```ts
await assetAsAdmin.setIssuer(admin, issuer, true);
```

---

## 4. Real Estate Property Terms (Metadata) & Registry Listing

The property's legal provenance (appraisal reports, deed of trust, title register, offering memorandum) is anchored on-chain using `legalDocHash` (a 32-byte SHA-256 hash digest):

```ts
import { createHash } from "node:crypto";

const deedSummary = "Grandview Plaza — 100 Main St, Commercial REIT Equity Units (Offering Memorandum v1)";
const legalDocHash = createHash("sha256").update(deedSummary).digest("hex");

await assetAsAdmin.updateMetadata(admin, {
  name: "Grandview Plaza REIT Units",
  symbol: "REIT-BLD1",
  decimals: 7,
  assetClass: "real_estate",
  legalDocHash,
  maxSupply: units(10_000_000),
});
```

Register the tokenized real estate asset in the protocol registry for ecosystem visibility:

```ts
if (REGISTRY_ID) {
  const registry = new RegistryClient(asAdmin);
  await registry.register(admin, {
    contract: RWA_ASSET_ID,
    assetClass: "real_estate",
    active: true,
  });
}
```

---

## 5. Minting Initial Equity Units

With compliance screening active and the issuer role authorized, mint the initial allocation to the verified investor:

```ts
const tx = await assetAsIssuer.mint(issuer, investor, units(500_000));
console.log(`Minted 500,000 REIT-BLD1 units in transaction ${tx.hash}`);

// Verify balances and supply
const balance = await assetAsIssuer.balance(investor);
const total = await assetAsIssuer.totalSupply();
console.log(`Investor balance: ${balance}`);
console.log(`Total supply: ${total}`);
```

If an unverified address attempts to receive tokens, or if a transfer would exceed the supply cap, the contract rejects the transaction on-chain.

---

## 6. Execution

Run the recipe script against testnet:

```bash
cd sdk
RWA_ASSET_ID=C... COMPLIANCE_ID=C... REGISTRY_ID=C... \
ADMIN_SECRET=S... ISSUER_SECRET=S... INVESTOR_ADDRESS=G... \
npx tsx examples/tokenized-real-estate.ts
```
