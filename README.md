# StellarForge

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Contracts CI](https://github.com/0xLizTech/stellarforge/actions/workflows/contracts-ci.yml/badge.svg)](https://github.com/0xLizTech/stellarforge/actions/workflows/contracts-ci.yml)
[![SDK CI](https://github.com/0xLizTech/stellarforge/actions/workflows/sdk-ci.yml/badge.svg)](https://github.com/0xLizTech/stellarforge/actions/workflows/sdk-ci.yml)
[![Discord](https://img.shields.io/discord/placeholder?label=Discord&logo=discord)](https://discord.gg/stellarforge)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

> **Institutional-grade Real World Asset tokenization on Stellar/Soroban.**
> Mint, transfer, fraction, and govern tokenized real-world assets — with built-in compliance, yield tranching, and cross-chain interoperability.

---

## What is StellarForge?

StellarForge is an open-source, permissionless protocol for **Real World Asset (RWA) tokenization** built on the [Stellar](https://stellar.org) blockchain and [Soroban](https://soroban.stellar.org) smart contract platform. It provides the primitive contracts, TypeScript SDK, and composable modules that institutions, fintech builders, and individual issuers need to bring real-world value on-chain — legally, securely, and at scale.

### Why Stellar?

| Concern | Stellar's answer |
|---|---|
| Transaction cost | ~$0.00001 per operation |
| Settlement speed | 3–5 second finality |
| Compliance tooling | Native DEX, federation, memo fields |
| Environmental footprint | Proof-of-Agreement; minimal energy use |
| Ecosystem | 10+ year track record, regulated corridors |

---

## Protocol Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                         StellarForge Protocol                         │
├───────────────┬──────────────┬───────────────┬───────────────────────┤
│  RWA Asset    │  Registry    │  Compliance   │  Governance           │
│  Contract     │  Contract    │  Engine       │  Contract             │
│               │              │               │                       │
│  • Mint/Burn  │  • Asset     │  • KYC/AML    │  • Proposals          │
│  • Transfer   │    Registry  │    records    │  • Voting             │
│  • Allowances │  • Lookup    │  • Jurisdiction│  • Execution hooks   │
│  • Metadata   │  • Active    │  • Expiry     │  (Phase 3+)           │
│  • Pause/     │    flags     │    management │                       │
│    Unpause    │              │               │                       │
└───────────────┴──────────────┴───────────────┴───────────────────────┘
         │                                               │
         │              TypeScript SDK                   │
         └───────────────────────────────────────────────┘
                  @stellarforge/sdk  ·  npm package
```

**Planned modules** (see [ROADMAP.md](ROADMAP.md)):
- Fractionalization vault
- Yield tranching engine
- Decentralized insurance pool
- Privacy layer (ZK proofs)
- Cross-chain bridge (IBC / LayerZero)
- Governance token + DAO

---

## Repository Structure

```
stellarforge/
├── contracts/
│   ├── common/             # Shared storage/TTL policy (library, not deployed)
│   ├── rwa-asset/          # Core tokenization contract (Soroban/Rust)
│   ├── registry/           # Global asset registry
│   ├── compliance/         # KYC/AML compliance engine
│   └── governance/         # On-chain governance
├── sdk/
│   ├── src/
│   │   ├── client.ts       # Contract interaction clients
│   │   ├── types.ts        # Shared TypeScript types
│   │   └── utils.ts        # Address, amount, hashing utilities
│   └── tests/
├── docs/
│   ├── prd/                # Product Requirements Document
│   └── architecture/       # Architecture decision records (ADRs)
├── scripts/                # Deployment & maintenance scripts
├── .github/workflows/      # CI pipelines
├── Cargo.toml              # Workspace manifest
├── Makefile                # Developer shortcuts
├── ROADMAP.md
└── CONTRIBUTING.md
```

---

## Quick Start

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | ≥ 1.81 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| wasm32 target | any | `rustup target add wasm32v1-none` |
| stellar-cli | ≥ 27.0 | `cargo install --locked stellar-cli` |
| Node.js | ≥ 22 | [nodejs.org](https://nodejs.org) |
| npm | ≥ 10 | bundled with Node |

### 1. Clone the repository

```bash
git clone https://github.com/0xLizTech/stellarforge.git
cd stellarforge
```

### 2. Build all contracts

```bash
make build
```

Compiled `.wasm` files appear in `target/wasm32v1-none/release/`.

### 3. Run contract tests

```bash
make test
```

### 4. Build & test the TypeScript SDK

```bash
make sdk-build
make sdk-test
```

### 5. Deploy to testnet

```bash
# Fund a test account first
stellar keys generate --global mykey --network testnet
stellar keys fund mykey --network testnet

# Deploy
STELLAR_ACCOUNT=mykey make deploy-testnet
```

### 6. Use the SDK

```typescript
import { RwaAssetClient, toStroops, TESTNET_CONFIG } from "@stellarforge/sdk";

const client = new RwaAssetClient({
  ...TESTNET_CONFIG,
  contracts: {
    rwaAsset: "C...", // your deployed contract ID
  },
});

const supply = await client.totalSupply();
const meta   = await client.metadata();
console.log(`${meta.symbol} — total supply: ${supply}`);
```

Writes come in two forms. `build*Tx` returns a prepared but unsigned transaction
for a wallet to sign, so no secret ever reaches the SDK:

```typescript
const tx = await client.buildTransferTx(from, to, toStroops("100", meta.decimals));
const signedXdr = await freighter.signTransaction(tx.toXDR(), { networkPassphrase });
```

The bare method signs with `signerSecret` and submits, for server-side use:

```typescript
const client = new RwaAssetClient({ ...TESTNET_CONFIG, contracts, signerSecret });
const { hash, ledger } = await client.transfer(from, to, toStroops("100", meta.decimals));
```

Both assume the authorizing address also sources the transaction, so its
signature satisfies the contract's `require_auth`. Paying fees from a separate
account is not supported yet.

---

## Contract Reference (Phase 1)

### RwaAsset

| Function | Auth | Description |
|---|---|---|
| `initialize(admin, metadata)` | admin | One-time setup |
| `mint(issuer, to, amount)` | issuer | Create new tokens |
| `burn(from, amount)` | from | Destroy tokens |
| `transfer(from, to, amount)` | from | Move tokens |
| `approve(owner, spender, amount)` | owner | Set allowance |
| `transfer_from(spender, from, to, amount)` | spender | Spend allowance |
| `set_issuer(issuer, approved)` | admin | Grant/revoke issuer role |
| `set_paused(paused)` | admin | Emergency circuit breaker |
| `update_metadata(metadata)` | admin | Update asset metadata |
| `transfer_admin(new_admin)` | admin + new_admin | Transfer admin role |
| `set_compliance(compliance, min_level)` | admin | Point at a compliance contract, or `None` to disable screening |
| `balance(owner)` | — | Query balance |
| `allowance(owner, spender)` | — | Query allowance |
| `total_supply()` | — | Query supply |
| `metadata()` | — | Query metadata |
| `admin()` | — | Query admin address |
| `is_issuer(address)` | — | Query issuer status |
| `paused()` | — | Query pause state |
| `compliance_contract()` | — | Query the configured compliance contract |
| `min_compliance_level()` | — | Query the required verification level |

**Transfer screening.** When `set_compliance` names a contract, `mint`,
`transfer` and `transfer_from` require every counterparty to hold a valid
verification record at or above `min_level` (0 none, 1 basic, 2 full,
3 accredited). `burn` is deliberately exempt, so a holder whose verification
has lapsed can still exit their position. Screening is skipped entirely while
no compliance contract is configured.

### Compliance

| Function | Auth | Description |
|---|---|---|
| `set_kyc(subject, record)` | admin | Set KYC record |
| `revoke_kyc(subject)` | admin | Remove KYC record |
| `is_compliant(subject, min_level)` | — | Check compliance; pure query, no ledger write |
| `screen(subject, min_level)` | — | As above, but refreshes the record's TTL; bound by `RwaAsset` |
| `get_kyc(subject)` | — | Read KYC record |
| `admin()` | — | Query admin address |

A compliance contract plugged into `RwaAsset` must implement `screen`, not just
`is_compliant`. The two return identical answers and differ only in that
`screen` extends the lifetime of the record it consults — a KYC record is
written once and thereafter only read, and only the compliance contract can
extend its own entries. See [ADR-001](docs/architecture/001-storage-key-design.md).

### Registry

| Function | Auth | Description |
|---|---|---|
| `register(entry)` | admin | Register a new asset contract, or update a registered one |
| `set_active(contract, active)` | admin | Activate/deactivate asset |
| `get_asset(contract)` | — | Look up asset entry |
| `list_assets()` | — | List all registered contracts, each exactly once |
| `admin()` | — | Query admin address |

### Governance

| Function | Auth | Description |
|---|---|---|
| `propose(proposer, title, hash, period)` | proposer | Create proposal |
| `vote(voter, id, support, weight)` | voter | Cast vote |
| `finalize(id)` | — | Tally and finalize; a tie is rejected |
| `get_proposal(id)` | — | Read proposal |
| `has_voted(id, voter)` | — | Whether an address has voted on a proposal |
| `proposal_count()` | — | Count proposals |
| `admin()` | — | Query admin address |

> **Phase 1 caveat.** `vote` accepts a caller-supplied `weight` and does not
> check it against any token balance or voting-power source, and `finalize`
> applies no quorum. Governance is a skeleton until the `SFORGE` token and
> execution hooks land in Phase 3 — do not treat a passed proposal as a
> trustworthy signal before then.

**Error codes.** Every contract returns a typed `contracterror` enum, so
clients match on a stable numeric code. Discriminants are part of the public
interface: variants keep their values and new ones are appended.

---

## Security

StellarForge is pre-audit software. **Do not use on mainnet with real funds until a formal audit is complete.**

- All sensitive operations require Soroban `require_auth()` — no permission can be spoofed at the contract level.
- The `set_paused` circuit breaker allows emergency halts without upgrading contracts.
- Admin transfer requires both current and new admin to authenticate.
- See [SECURITY.md](docs/SECURITY.md) for the responsible disclosure policy.

---

## Contributing

We warmly welcome contributors of all skill levels. Whether you're fixing a typo, writing tests, building a new module, or reviewing a PR — you belong here.

Read [CONTRIBUTING.md](CONTRIBUTING.md) to get started. Areas where we especially need help are listed there.

---

## Community

| Channel | Purpose |
|---|---|
| [GitHub Discussions](https://github.com/0xLizTech/stellarforge/discussions) | Architecture, proposals, Q&A |
| [Discord](https://discord.gg/stellarforge) | Real-time chat, dev support |
| [Twitter/X](https://twitter.com/stellarforge_io) | Announcements |

---

## Roadmap Summary

| Phase | Theme | Status |
|---|---|---|
| 1 — Foundation | Core asset contract, compliance, registry, governance skeleton | **In Progress** |
| 2 — Fractionalization | Vault contract, yield distribution, fractional shares | Planned |
| 3 — DeFi Primitives | Liquidity pools, orderbook integration, yield tranching | Planned |
| 4 — Privacy & Cross-chain | ZK-compliant transfers, bridge to Ethereum/Cosmos | Planned |
| 5 — Full DAO | Decentralized governance, insurance pool, protocol treasury | Planned |

Full details in [ROADMAP.md](ROADMAP.md).

---

## License

Apache 2.0 — see [LICENSE](LICENSE).

Built with love on [Stellar](https://stellar.org) and [Soroban](https://soroban.stellar.org).
