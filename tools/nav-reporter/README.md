# stellarforge-nav-reporter

Institutional appraisal publication CLI for the StellarForge SEP-40 `oracle-adapter` contract.

## Overview

The `stellarforge-nav-reporter` CLI enables appraisers and back-office operators to validate and publish audited net asset values (NAV) to the on-chain oracle feed safely, idempotently, and in sponsored transaction mode.

## Appraisal File Format

Appraisal files are formatted as JSON and signed offline with an Ed25519 private key:

```json
{
  "payload": {
    "asset": {
      "type": "other",
      "symbol": "REALESTATE"
    },
    "price": "123456.78",
    "observedAt": "2026-09-30T00:00:00Z",
    "appraiser": "Institutional Real Estate Appraisers Inc.",
    "documentSha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  },
  "signature": "3045022100...",
  "publicKey": "GBBM7ALZ74ZLV4QGVFDT6B47X7345654..."
}
```

## Signing Appraisals Offline

Signatures must be computed over RFC 8785 canonical JSON (keys sorted alphabetically).

Example using Node.js:

```typescript
import { canonicalJsonStringify } from "@stellarforge-protocol/nav-reporter";
import { Keypair } from "@stellar/stellar-sdk";

const canonicalBytes = Buffer.from(canonicalJsonStringify(payload), "utf8");
const signature = keypair.sign(canonicalBytes).toString("hex");
```

## CLI Usage

```bash
export REPORTER_SECRET_KEY="S..."
export RELAYER_SECRET_KEY="S..."
export APPRAISER_PUBLIC_KEY="G..."

stellarforge-nav-reporter \
  --appraisal path/to/appraisal.json \
  --oracle CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --network testnet
```

### Dry Run

Perform all signature checks and pre-flight simulations without submitting on-chain:

```bash
stellarforge-nav-reporter \
  --appraisal path/to/appraisal.json \
  --oracle CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --dry-run
```

### Structured Output for Automation

```bash
stellarforge-nav-reporter --appraisal path/to/appraisal.json --oracle ... --json
```

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success (or dry-run passed, or duplicate idempotent report) |
| `2` | Authentication failure (invalid signature or unknown appraiser key) |
| `3` | Pre-flight validation failure (stale tick, future date, lossy conversion, or deviation exceeded) |
| `4` | Submission failure (network or host error) |
| `5` | Unknown outcome (timeout where on-chain state could not be verified) |
