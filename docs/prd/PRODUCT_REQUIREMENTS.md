# StellarForge Protocol — Product Requirements Document

**Version:** 1.0.0
**Date:** May 2026
**Status:** Active
**Authors:** StellarForge Core Team

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Vision & Mission](#2-vision--mission)
3. [Market Context & Problem Statement](#3-market-context--problem-statement)
4. [Target Users & Personas](#4-target-users--personas)
5. [Protocol Scope & Feature Set](#5-protocol-scope--feature-set)
6. [Phased Roadmap Requirements](#6-phased-roadmap-requirements)
7. [Functional Requirements](#7-functional-requirements)
8. [Non-Functional Requirements](#8-non-functional-requirements)
9. [Architecture Principles](#9-architecture-principles)
10. [Security Considerations](#10-security-considerations)
11. [Compliance & Regulatory Framework](#11-compliance--regulatory-framework)
12. [Privacy Requirements](#12-privacy-requirements)
13. [Governance Model](#13-governance-model)
14. [Cross-Chain Strategy](#14-cross-chain-strategy)
15. [Economic Model](#15-economic-model)
16. [Success Metrics](#16-success-metrics)
17. [Open Questions & Risks](#17-open-questions--risks)
18. [Glossary](#18-glossary)

---

## 1. Executive Summary

StellarForge is an open-source, institutional-grade protocol for **Real World Asset (RWA) tokenization** built on the Stellar blockchain and Soroban smart contract platform. It provides a composable stack of on-chain primitives — tokenization, fractionalization, yield tranching, compliance, governance, privacy, and cross-chain interoperability — that together enable any real-world asset to be issued, traded, fractionalized, and governed on-chain with full regulatory compliance.

The protocol is designed to serve both institutional issuers (funds, banks, custodians) and individual builders (fintech startups, DAOs) through a permissionless, auditable, and progressively decentralized architecture.

**Core value proposition:**
- Issue compliant RWA tokens in minutes, not months.
- Fractional ownership from $1, opening institutional-grade assets to retail investors.
- Programmable compliance that adapts to multiple jurisdictions without re-architecting.
- Decentralized governance ensuring no single entity controls the protocol.

---

## 2. Vision & Mission

### Vision

A world where every real-world asset — from a Manhattan skyscraper to a Nairobi micro-loan — is liquid, fractionally owned, globally accessible, and settled in seconds.

### Mission

StellarForge's mission is to build the most secure, composable, and compliant RWA infrastructure on Stellar, enabling the next generation of financial products that bridge on-chain efficiency with off-chain legal reality.

### Core Values

| Value | Meaning |
|---|---|
| **Security First** | Every contract is audited before mainnet. Emergency circuit breakers are non-negotiable. |
| **Compliance by Design** | KYC/AML, jurisdiction controls, and legal document anchoring are protocol primitives, not optional plugins. |
| **Progressive Decentralization** | We launch with pragmatic centralization and systematically transfer power to token holders. |
| **Composability** | Every module is a standalone contract. Builders use what they need and ignore the rest. |
| **Radical Openness** | Apache 2.0. All code, all design decisions, all audit reports — public. |

---

## 3. Market Context & Problem Statement

### The RWA Opportunity

The global market for tokenizable real-world assets is estimated at **$16–$30 trillion** by 2030 (Boston Consulting Group, 2023; BlackRock, 2024). By early 2026, on-chain RWA TVL has crossed $10 billion across all chains, led by tokenized US Treasuries and private credit.

Yet the infrastructure is fragmented:
- Ethereum-based protocols (Centrifuge, Maple, Ondo) serve institutional DeFi but carry high gas costs and EVM complexity.
- Stellar's low cost and native compliance features are underutilized for RWA.
- No open-source, modular RWA stack exists on Soroban.

### Core Problems

**P1: Cost and speed of traditional asset issuance**
Issuing a private placement security requires months of legal work, expensive intermediaries, and settlement that takes T+2 or longer. Tokenization compresses this to days and settles in seconds.

**P2: Liquidity fragmentation**
Real estate, private credit, and art are illiquid by nature. Tokenization enables 24/7 secondary markets, but only if the asset's token is natively tradeable.

**P3: Fractional ownership barrier**
A $50M real estate fund has a $100,000 minimum investment. Tokenization enables $10 minimums, democratizing access.

**P4: Compliance complexity**
Each jurisdiction has different KYC requirements, investor accreditation thresholds, and transfer restrictions. Hardcoding these into a smart contract creates inflexible, brittle systems.

**P5: Privacy paradox**
On-chain transparency conflicts with financial privacy. Investors shouldn't need to expose their identity or wealth to participate in a compliant market.

**P6: Cross-chain isolation**
RWA tokens issued on Stellar cannot natively reach Ethereum DeFi liquidity or Cosmos app-chains without complex, centralized bridges.

---

## 4. Target Users & Personas

### Primary Personas

#### Persona A: Institutional Issuer — "The Fund Manager"
- **Who:** Private equity fund, REIT manager, or debt issuer looking to tokenize an existing portfolio asset.
- **Goals:** Reduce administrative overhead, expand investor base globally, automate compliance, offer 24/7 secondary liquidity.
- **Pain points:** Regulatory uncertainty, technical complexity of smart contracts, need for custodian integrations.
- **StellarForge's answer:** Pre-built compliant token contract, SDK for rapid integration, legal doc anchoring on-chain.

#### Persona B: Fintech Builder — "The Protocol Integrator"
- **Who:** A startup building a tokenized real estate platform, or a DeFi developer adding RWA collateral.
- **Goals:** Ship fast, use audited primitives, avoid rebuilding compliance from scratch.
- **Pain points:** No standard RWA interface; reinventing the compliance wheel every time.
- **StellarForge's answer:** Composable contracts, TypeScript SDK, open-source reference implementations.

#### Persona C: Retail Investor — "The Fractional Holder"
- **Who:** An individual investor who wants exposure to real estate or private credit with $500, not $100,000.
- **Goals:** Earn yield on real assets, diversify beyond stocks and crypto, exit position when needed.
- **Pain points:** Accreditation barriers, illiquidity, opaque fee structures.
- **StellarForge's answer:** Fractionalization vault, on-chain yield distribution, secondary market via Stellar DEX.

#### Persona D: DAO Participant — "The Governance Token Holder"
- **Who:** A crypto-native who holds `SFORGE` tokens and wants to govern protocol parameters.
- **Goals:** Vote on fee changes, new module approvals, treasury grants; earn protocol revenue.
- **Pain points:** Governance apathy, plutocratic voting, low signal-to-noise in proposals.
- **StellarForge's answer:** Transparent on-chain proposals, weighted voting, optimistic governance with veto rights.

#### Persona E: Compliance Officer — "The Regulator Interface"
- **Who:** An in-house compliance officer at an issuer or investor institution.
- **Goals:** Verify that all token holders are KYC'd, generate regulatory reports, enforce transfer restrictions.
- **Pain points:** On-chain data is pseudonymous; off-chain records don't match on-chain activity.
- **StellarForge's answer:** On-chain KYC record anchoring, jurisdiction flags, compliance-queryable by issuers.

---

## 5. Protocol Scope & Feature Set

### 5.1 Core Tokenization

The foundation of StellarForge is a programmable RWA asset token. Each token represents a legal claim on an underlying real-world asset, anchored by an off-chain legal document whose hash is stored on-chain.

**Features:**
- Mint / burn with issuer authentication
- Transfer with optional compliance gate
- Allowance/delegated transfer (ERC-20-compatible semantics)
- Rich metadata (name, symbol, decimals, asset class, legal doc hash, supply cap)
- Emergency circuit breaker (pause/unpause)
- Admin role management (grant, transfer, revoke)
- Upgrade path via Stellar's contract upgrade mechanism

### 5.2 Asset Registry

A protocol-wide registry of all deployed RWA asset contracts. Provides discoverability and active/inactive lifecycle management.

**Features:**
- Register new asset contracts with class and metadata
- Active/inactive flag (delist without destroying contract)
- Enumerable asset list for frontends and indexers

### 5.3 Compliance Engine

An on-chain KYC/AML record store that issuers query before allowing transfers. Designed to be maintained by licensed identity providers or issuers themselves.

**Features:**
- Per-address KYC records with jurisdiction (ISO-3166) and verification level (0–3)
- Expiry dates on all records (no perpetual approvals)
- Revocation without on-chain data destruction
- Queried by asset contracts at transfer time (Phase 2+ via hook)
- Multiple compliance admins per jurisdiction (Phase 2)

### 5.4 Fractionalization Vault (Phase 2)

A vault that holds an underlying asset and issues pro-rata fractional shares.

**Features:**
- Deposit underlying RWA token → receive fractional shares
- Redeem fractional shares → receive underlying (subject to lock-ups)
- NAV calculation via oracle
- Configurable lock-up period per vault
- Maximum fractional share supply

### 5.5 Yield Distribution (Phase 2)

Distributes off-chain income (rental income, dividends, loan repayments) to on-chain fractional token holders.

**Features:**
- Snapshot-based distribution (snapshot balance at announcement ledger)
- Multi-currency payout (XLM, USDC, custom asset)
- Unclaimed yield after configurable timeout reverts to pool
- Transparent distribution history on-chain

### 5.6 Yield Tranching (Phase 3)

Risk-stratified pools where investors choose between senior (lower yield, protected principal) and junior (higher yield, first-loss) positions.

**Features:**
- Configurable tranche classes (Senior, Mezzanine, Junior)
- Waterfall payment logic: senior tranche paid first from yield
- Junior tranche earns excess yield as reward for first-loss position
- Loss absorption: junior tranche principal is consumed first on defaults
- Tranche tokens are transferable on Stellar DEX

### 5.7 On-Chain Governance (Phase 1–5 progressive)

**Phase 1:** Proposal creation, binary voting, finalization
**Phase 2:** Off-chain discussion integration (Snapshot-style metadata hash)
**Phase 3:** Execution hooks — passed proposals can invoke contract functions
**Phase 4:** `SFORGE` token weighted voting
**Phase 5:** Full DAO with optimistic execution and guardian veto

### 5.8 Privacy Layer — ZK Compliance (Phase 4)

Enable investors to prove compliance without revealing identity.

**Features:**
- Zero-knowledge proofs of KYC status (Groth16 or PLONK circuit)
- Off-chain proof generation by approved identity providers
- On-chain verifier contract (WASM-based)
- Proof validity tied to ledger sequence (replay protection)
- Compatible with zkPassport and Semaphore primitives

### 5.9 Cross-Chain Bridge (Phase 4)

**Stellar → Ethereum:**
- Lock RWA tokens on Stellar; mint ERC-20 wrapped tokens on Ethereum
- Decentralized relayer network with economic bonding
- 7-day dispute window for fraud proofs

**Stellar → Cosmos:**
- IBC channel via Penumbra-compatible light client
- Interchain accounts for multi-step settlement

### 5.10 Decentralized Insurance Pool (Phase 5)

**Features:**
- Permissionless coverage purchase (pay premium, specify coverage amount and duration)
- Policy backed by pooled collateral staked by insurance providers
- Claims adjudication via DAO vote
- Solvency enforcement (minimum 150% coverage ratio)
- Coverage scope: smart contract bugs, oracle manipulation, bridge exploits

---

## 6. Phased Roadmap Requirements

### Phase 1 — Foundation (Q3 2026)

**In scope:**
- `rwa-asset`, `registry`, `compliance`, `governance` contracts
- TypeScript SDK (read + write paths)
- Full CI pipeline
- Internal security review
- Testnet deployment tooling
- `@stellarforge/sdk` npm package v0.1.0

**Out of scope:**
- Fractionalization, yield, tranching
- Cross-chain
- ZK proofs
- Governance execution hooks
- `SFORGE` token

**Exit criteria:**
- All contracts pass Soroban integration tests on testnet
- External audit completed with no critical findings
- SDK published and functional against testnet contracts

### Phase 2 — Fractionalization (Q4 2026)

**In scope:**
- `vault` contract (fractionalization)
- `yield-distributor` contract
- `oracle-adapter` interface
- Compliance hook integrated into `rwa-asset` transfer
- SDK: vault and yield clients
- Phase 1 mainnet deployment

**Exit criteria:**
- Fractionalization vault tested with real estate asset on testnet
- Yield distribution executed to 1,000+ simulated holders
- Phase 1 mainnet deployment with $0 TVL launch (protocol not yet accepting user funds)

### Phase 3 — DeFi Primitives (Q1 2027)

**In scope:**
- `tranche` contract
- `liquidity-pool` adapter (Stellar DEX integration)
- Governance execution hooks + timelock
- `SFORGE` token genesis
- Protocol fee switch (off by default)
- Reference frontend (Next.js)

**Exit criteria:**
- Tranche contract tested with simulated defaults
- Governance proposal successfully executes a contract call on testnet
- `SFORGE` distributed to early contributors and testnet participants

### Phase 4 — Privacy & Cross-Chain (Q3 2027)

**In scope:**
- ZK compliance verifier contract
- Ethereum bridge (lock-and-mint)
- Cosmos IBC bridge
- Multi-sig guardian retirement (replaced by governance)

**Exit criteria:**
- ZK proof verified on-chain in < 100ms simulated latency
- Bridge tested with 100+ round-trip transfers on testnets
- No admin multisig keys for Phase 1–3 contracts (governance controls all)

### Phase 5 — Full DAO & Insurance (Q1–Q2 2028)

**In scope:**
- `insurance-pool` contract
- Full DAO migration
- Protocol treasury management
- Institutional custody integrations (Fireblocks, Copper)
- Regulatory reporting module

**Exit criteria:**
- Insurance pool solvency ratio maintained above 200% for 90 days
- All protocol admin keys relinquished
- DAO successfully approves and executes a fee change proposal

---

## 7. Functional Requirements

### FR-01: Asset Issuance

| ID | Requirement |
|---|---|
| FR-01-1 | Any address designated as an issuer by the admin MAY mint tokens up to the configured max supply. |
| FR-01-2 | Minting MUST require `require_auth()` on the issuer address. |
| FR-01-3 | Minting MUST fail if `amount <= 0`. |
| FR-01-4 | Minting MUST fail if `total_supply + amount > max_supply` when max_supply > 0. |
| FR-01-5 | Minting MUST fail if the contract is paused. |
| FR-01-6 | The admin MAY grant or revoke the issuer role at any time. |

### FR-02: Transfers

| ID | Requirement |
|---|---|
| FR-02-1 | Token holders MAY transfer tokens to any address. |
| FR-02-2 | Transfers MUST require `require_auth()` on the sender. |
| FR-02-3 | Transfers MUST fail if `amount > balance(sender)`. |
| FR-02-4 | Transfers MUST fail if the contract is paused. |
| FR-02-5 | Delegated transfer via `transfer_from` MUST deduct the consumed amount from the spender's allowance. |
| FR-02-6 | (Phase 2) Transfers MAY be blocked by a compliance gate if the recipient fails `is_compliant` check. |

### FR-03: Compliance

| ID | Requirement |
|---|---|
| FR-03-1 | The compliance admin MAY set a KYC record for any address. |
| FR-03-2 | KYC records MUST include: jurisdiction (ISO-3166), level (0–3), and expiry timestamp. |
| FR-03-3 | `is_compliant(address, min_level)` MUST return false if the record is expired. |
| FR-03-4 | `is_compliant` MUST return false if no record exists. |
| FR-03-5 | Records MUST be revocable by the compliance admin at any time. |

### FR-04: Registry

| ID | Requirement |
|---|---|
| FR-04-1 | The registry admin MAY register any deployed contract address. |
| FR-04-2 | Registered entries MUST be enumerable (list all, get by address). |
| FR-04-3 | The admin MAY set an entry inactive without removing it. |

### FR-05: Governance

| ID | Requirement |
|---|---|
| FR-05-1 | Any address MAY create a proposal by providing a title and description hash. |
| FR-05-2 | Proposals MUST have a configurable voting deadline (expressed in ledger sequence count). |
| FR-05-3 | An address MAY vote at most once per proposal. |
| FR-05-4 | Voting MUST fail after the deadline ledger is passed. |
| FR-05-5 | `finalize` MAY be called by anyone after the deadline; it MUST mark the proposal Passed or Rejected. |
| FR-05-6 | (Phase 3) Passed proposals with an attached execution payload MUST be executable after a timelock period. |

### FR-06: Fractionalization (Phase 2)

| ID | Requirement |
|---|---|
| FR-06-1 | A vault MUST accept a configured RWA asset as its underlying. |
| FR-06-2 | Depositing X underlying tokens MUST mint exactly X * exchange_rate fractional shares. |
| FR-06-3 | Redeeming Y fractional shares MUST return Y / exchange_rate underlying tokens, subject to lock-up. |
| FR-06-4 | The NAV MUST be updateable via an oracle; the vault MUST reject oracle prices older than a configurable max staleness. |

### FR-07: Yield Distribution (Phase 2)

| ID | Requirement |
|---|---|
| FR-07-1 | The yield admin MAY deposit income in any supported asset (XLM, USDC). |
| FR-07-2 | A snapshot of fractional token balances MUST be taken at the distribution announcement ledger. |
| FR-07-3 | Each holder's claim MUST be proportional to their balance at snapshot time. |
| FR-07-4 | Unclaimed yield MUST be reclaimable by the admin after a configurable expiry (minimum 90 days). |

---

## 8. Non-Functional Requirements

### Performance

| ID | Requirement |
|---|---|
| NFR-P-1 | All read-only contract calls (balance, metadata, is_compliant) MUST complete simulation in < 500ms on Stellar testnet. |
| NFR-P-2 | Write operations (mint, transfer) MUST be confirmed on-chain within 2 Stellar ledgers (≈10 seconds). |
| NFR-P-3 | The registry's `list_assets` function MUST support at least 10,000 registered contracts without ledger key size overflow. |

### Reliability

| ID | Requirement |
|---|---|
| NFR-R-1 | Contracts MUST NOT be upgradeable without explicit admin authorization and a governance vote (Phase 3+). |
| NFR-R-2 | The circuit breaker (`set_paused`) MUST take effect in the same ledger in which it is called. |
| NFR-R-3 | Storage entries MUST use persistent storage (not temporary) to survive ledger archival. |

### Security

| ID | Requirement |
|---|---|
| NFR-S-1 | Every privileged function MUST call `require_auth()` on the relevant authority address. |
| NFR-S-2 | No `unsafe` Rust code is permitted in any contract. |
| NFR-S-3 | All integer arithmetic MUST use checked arithmetic or validated ranges to prevent overflow. |
| NFR-S-4 | Admin transfer MUST require authentication from BOTH current and new admin. |
| NFR-S-5 | All contracts MUST be deployed from a reproducible, deterministic build (pinned Rust toolchain + Soroban SDK version). |
| NFR-S-6 | All Phase 1–4 contracts MUST receive an external security audit before mainnet deployment. |

### Auditability

| ID | Requirement |
|---|---|
| NFR-A-1 | All state changes MUST emit Soroban events readable by Horizon indexers. |
| NFR-A-2 | Legal document hashes stored on-chain MUST be the SHA-256 of the document bytes, verifiable off-chain. |
| NFR-A-3 | The complete governance history (all proposals, votes, outcomes) MUST be queryable via events without proprietary indexing. |

### Developer Experience

| ID | Requirement |
|---|---|
| NFR-D-1 | The TypeScript SDK MUST be usable without any Rust knowledge. |
| NFR-D-2 | Contract deployment MUST be achievable with a single `make deploy-testnet` command. |
| NFR-D-3 | All SDK public functions MUST have JSDoc documentation. |
| NFR-D-4 | The SDK MUST publish TypeScript declarations alongside the CommonJS and ESM builds. |

---

## 9. Architecture Principles

### AP-1: Separation of Concerns

Each contract addresses one concern. `rwa-asset` handles tokens. `compliance` handles identity records. `governance` handles voting. Contracts communicate via inter-contract calls (not shared storage), keeping each independently auditable and upgradeable.

### AP-2: Minimal On-Chain State

Store on-chain only what must be there for trustless operation. Legal documents, KYC identity details, and governance discussion live off-chain; their integrity is assured by hashes stored on-chain.

### AP-3: Auth-First Design

No privileged function may execute without an authenticated authorization from the relevant party. The Soroban `require_auth()` mechanism is the foundation; we add no custom auth logic that bypasses it.

### AP-4: Storage Tier Discipline

| Tier | Use case |
|---|---|
| `instance` | Contract-level config (admin, pause flag) — small, always loaded |
| `persistent` | Per-entity data (balances, KYC records, proposals) — archived, cheap |
| `temporary` | Nonces, short-lived flags — auto-expired, cheapest |

### AP-5: Fail-Fast, Fail-Loud

Contracts panic with descriptive messages on invariant violations. There is no silent failure. Every error is observable and debuggable without access to private state.

### AP-6: Deterministic Builds

All contracts use a pinned Rust toolchain (via `rust-toolchain.toml`) and a pinned `soroban-sdk` version. The WASM artifact hash is reproducible from source, enabling trustless verification of on-chain bytecode.

### AP-7: No Circular Dependencies

Contracts may call other contracts, but there are no circular call graphs. Dependency direction is: `rwa-asset` → `compliance`; `governance` controls parameters of both. `registry` depends on nothing.

### AP-8: Event-Driven Indexing

All state transitions emit Soroban events. Off-chain indexers (hosted by the protocol and by third parties) reconstruct full state from events. This enables:
- Reorg-resistant balance indexing
- Auditability without on-chain list iteration
- Real-time notifications for frontends

---

## 10. Security Considerations

### 10.1 Threat Model

**Adversarial actors:**
- Malicious issuer: attempts to mint beyond cap or to unapproved addresses
- Rogue compliance admin: attempts to approve non-compliant investors
- Governance attacker: attempts to pass a malicious proposal by purchasing votes
- Relay attacker (Phase 4): attempts to double-spend on the cross-chain bridge
- Oracle manipulator (Phase 2): attempts to fake NAV to trigger unjust redemptions

### 10.2 Mitigations

| Threat | Mitigation |
|---|---|
| Over-minting | Hard supply cap enforced in contract; not bypassable by admin |
| Replay attacks | Soroban's built-in nonce management; sequence-bound ZK proofs (Phase 4) |
| Admin key compromise | Admin transfer requires dual auth; Phase 3+ replaces multisig with governance |
| Governance takeover | Minimum quorum requirements; optimistic timelock; guardian veto (Phase 5) |
| Oracle manipulation | Staleness check; multi-source oracle aggregation; circuit breaker on NAV deviation |
| Bridge replay | Unique bridge message IDs; 7-day dispute window; economic bonds for relayers |
| Reentrancy | Soroban's execution model prevents reentrancy by design (no external contract can interrupt execution) |
| Integer overflow | All arithmetic uses checked operations or Rust's overflow-checks = true profile flag |

### 10.3 Audit Strategy

| Phase | Scope | Approach |
|---|---|---|
| Phase 1 | rwa-asset, registry, compliance, governance | External audit (1 firm) + internal review |
| Phase 2 | vault, yield-distributor, oracle-adapter | External audit (2 firms, independent) |
| Phase 3 | tranche, liquidity-pool, governance hooks | External audit + formal verification of tranche math |
| Phase 4 | ZK verifier, bridge | Specialized ZK audit firm + bridge security specialists |
| Phase 5 | insurance-pool, treasury | External audit + economic security review |

All audit reports will be published in full in the repository.

### 10.4 Emergency Response

The protocol maintains a Security Council (initially a 3-of-5 multisig) empowered to:
- Call `set_paused(true)` on any contract
- Initiate emergency contract upgrades
- Freeze bridge relayers

The Security Council's powers are bounded by governance (Phase 5: council is elected by DAO vote, powers are defined by on-chain parameters).

---

## 11. Compliance & Regulatory Framework

### 11.1 Jurisdiction Strategy

StellarForge is jurisdiction-agnostic at the protocol level. Each asset issuer configures the jurisdictions they accept investors from and the minimum KYC level required. The protocol stores these restrictions on-chain; enforcement at the transfer level is optional (configurable per asset).

### 11.2 Supported Regulatory Frameworks (Target)

| Framework | Jurisdiction | Key requirement | Protocol support |
|---|---|---|---|
| Reg D (506c) | USA | Accredited investor verification | KYC level 3 + jurisdiction = "US" |
| Reg A+ | USA | Registered offering, up to $75M | On-chain offering document hash + cap |
| MiCA | EU | CASP licensing, whitepaper disclosure | Legal doc hash + issuer identity |
| MAS PS Act | Singapore | Payment service provider compliance | Jurisdiction + compliance level checks |
| ADGM / DIFC | UAE | Recognized investment platform | Configurable per issuer |

### 11.3 Legal Document Anchoring

Every deployed RWA asset contract stores the SHA-256 hash of its primary legal offering document. The issuer publishes the document off-chain (IPFS or their own storage). Investors can independently verify the document matches the on-chain hash before purchasing. This:
- Creates an immutable, timestamped link between on-chain tokens and legal rights
- Prevents silent document amendment
- Is verifiable without trusting the issuer's server

### 11.4 Investor Transfer Restrictions

Issuers may configure:
- **Allowlist only:** Only addresses with a valid KYC record of level ≥ N may receive tokens.
- **Jurisdictional block:** Block transfers to/from specific countries.
- **Holding limits:** Maximum tokens per address (prevents concentration risk violations).
- **Lock-up periods:** Minimum holding time before transfer (Reg D 6-month restriction, etc.).

### 11.5 Reporting Hooks (Phase 5)

The regulatory reporting module will generate:
- Cap table exports (all token holders at a given ledger)
- Transfer history reports (for AML audit trails)
- Yield distribution reports (for tax reporting)

These are generated from on-chain events by an indexer; no private data is stored on-chain.

---

## 12. Privacy Requirements

### 12.1 Privacy Design Goals

1. **KYC privacy:** Investors should not need to broadcast their identity to all on-chain observers.
2. **Amount privacy:** Sensitive institutional investors may not want holdings sizes to be public.
3. **Compliance preservation:** Privacy must not impede regulators' ability to investigate when legally compelled.

### 12.2 Phase 4 ZK Compliance

The ZK compliance system works as follows:

1. An investor completes KYC with a licensed identity provider.
2. The identity provider issues a cryptographic credential (e.g., a Semaphore group membership or a zkPassport proof).
3. The investor generates a ZK-SNARK proof proving: "I hold a valid credential from provider P for jurisdiction J at level L, valid at ledger N."
4. The proof is submitted alongside a transfer transaction. The on-chain verifier checks the proof without learning the investor's identity.
5. The identity provider can, under legal compulsion, reveal the investor's identity by linking their credential to their real identity.

This design satisfies:
- Compliance: identity is discoverable under legal process
- Privacy: on-chain observers see only a valid proof, not identity
- Censorship resistance: identity providers cannot arbitrarily block proofs already issued

### 12.3 Amount Privacy (Research Track)

Using Pedersen commitments, token balances could be represented as encrypted amounts verifiable via range proofs. This is a significant engineering undertaking and is designated a research track beyond Phase 5.

---

## 13. Governance Model

### 13.1 Phase 1–2: Pragmatic Multisig

Admin keys are held by the StellarForge core team (3-of-5 multisig). Changes require off-chain discussion and on-chain execution by keyholders.

Rationale: pre-audit protocol should not be governed by anonymous token holders who may not understand the security tradeoffs.

### 13.2 Phase 3: Hybrid Governance

The `SFORGE` governance token is introduced. Certain non-critical parameters (fee rates, oracle sources, new asset class additions to registry) are governed by `SFORGE` holders. Critical parameters (contract upgrades, emergency actions) remain with the Security Council.

### 13.3 Phase 5: Full DAO

**Governance lifecycle:**
1. **Discussion (off-chain):** 5-day discussion period on the forum.
2. **Proposal (on-chain):** Proposer stakes 1,000 `SFORGE` (returned if proposal passes quorum).
3. **Voting (on-chain):** 7-day voting window. Quorum: 4% of circulating `SFORGE`. Passing threshold: simple majority.
4. **Timelock:** 48-hour delay before execution.
5. **Guardian veto (optional):** The Security Council may veto within the 48-hour window (requires 4-of-7 guardians).
6. **Execution:** Anyone may execute a passed, timelocked proposal.

**Proposal types:**

| Type | Example | Execution |
|---|---|---|
| Parameter change | Adjust protocol fee from 10bps to 5bps | Automatic (timelock) |
| Contract upgrade | Deploy new vault v2 | Automatic (timelock + audit requirement) |
| Treasury grant | Fund contributor for 3 months | Automatic (timelock) |
| Emergency action | Pause a contract due to bug | Immediate (Security Council) |

---

## 14. Cross-Chain Strategy

### 14.1 Design Principles

- **No canonical bridge.** The protocol defines a bridge interface standard. Multiple competing bridges may implement it.
- **Proof-first, not oracle-first.** Bridge validity should rely on cryptographic proofs (ZK or IBC light clients), not trusted oracle feeds.
- **7-day dispute window** for lock-and-mint bridges. Fraud proofs can be submitted to cancel fraudulent mints.

### 14.2 Stellar → Ethereum Bridge

**Architecture:**
- **Stellar side:** `bridge` Soroban contract. Locks RWA tokens, emits a `BridgeInitiated` event.
- **Relayer network:** Decentralized relayers watch Stellar and submit mint requests to Ethereum. Relayers post bonds to participate.
- **Ethereum side:** ERC-20 `WrappedRWA` contract (open-source, separate repo). Mints upon receiving a valid relayer attestation.
- **Dispute resolution:** Any party may submit a fraud proof (Stellar Horizon proof of non-event) within 7 days to reverse a fraudulent mint.

### 14.3 Stellar → Cosmos Bridge (IBC)

- Requires a Stellar light client on the Cosmos side.
- Stellar's consensus (SCP) proofs are compatible with IBC's general light client interface.
- StellarForge will contribute the Stellar IBC light client implementation to the Cosmos ecosystem.

### 14.4 Destination Chain Compliance

Wrapped RWA tokens on Ethereum preserve compliance metadata via ERC-1400 (security token standard) extension. Transfers on Ethereum are restricted to addresses whose Ethereum address is linked to a valid Stellar compliance record via a cross-chain identity attestation.

---

## 15. Economic Model

### 15.1 Protocol Fee

StellarForge charges a protocol fee on:
- Minting (0–50 bps on mint amount)
- Yield distributions (0–100 bps on distributed amount)
- Bridge transfers (fixed fee in XLM + 0–25 bps on transfer value)

**Default configuration:** All fees start at 0 bps. The fee switch requires a governance vote to enable.

### 15.2 `SFORGE` Token

| Parameter | Value |
|---|---|
| Total supply | 100,000,000 `SFORGE` |
| Inflation | None (fixed supply) |
| Distribution | 40% community (contributors, testnet participants, grants); 25% team (4-year vest, 1-year cliff); 20% treasury; 15% early supporters |
| Utility | Protocol governance voting; staking for insurance pool; required stake to submit governance proposals |

### 15.3 Treasury

Protocol fee revenue accrues to the on-chain treasury (Phase 5). Treasury funds may be deployed by DAO vote for:
- Security audits
- Developer grants
- Marketing and ecosystem development
- Bug bounties
- `SFORGE` buyback-and-burn

### 15.4 Insurance Pool Economics

- **Premiums:** Paid by asset issuers or investors purchasing coverage.
- **Yields to stakers:** Premiums distributed to insurance capital providers proportional to their stake.
- **Capital requirement:** Minimum 150% solvency ratio (coverage_available / premium_liability).
- **Claims:** Paid from pooled capital upon successful DAO claim vote.

---

## 16. Success Metrics

### Phase 1 (Q3 2026)

| Metric | Target |
|---|---|
| GitHub stars | ≥ 500 |
| Contributors (all-time) | ≥ 25 |
| Contracts deployed on testnet | 4 (rwa-asset, registry, compliance, governance) |
| SDK npm weekly downloads | ≥ 200 |
| Audit findings (critical) | 0 |
| Audit findings (high) | ≤ 2, all resolved |
| CI build time | ≤ 5 minutes |

### Phase 2 (Q4 2026)

| Metric | Target |
|---|---|
| Total Value Locked (testnet) | ≥ $1M simulated |
| Fractional token holders (testnet) | ≥ 500 |
| Yield distributions executed | ≥ 10 |
| Ecosystem integrations (3rd party) | ≥ 3 |

### Phase 3 (Q1 2027)

| Metric | Target |
|---|---|
| Total Value Locked (mainnet) | ≥ $5M |
| Unique wallet holders | ≥ 1,000 |
| `SFORGE` holders | ≥ 5,000 |
| Governance proposals passed | ≥ 5 |
| Protocol fee revenue (annualized) | ≥ $50K |

### Phase 5 (Q2 2028)

| Metric | Target |
|---|---|
| Total Value Locked (mainnet) | ≥ $500M |
| Unique wallet holders | ≥ 50,000 |
| Asset classes supported | ≥ 10 |
| Cross-chain bridge volume (annualized) | ≥ $100M |
| Insurance pool TVL | ≥ $20M |
| Protocol fee revenue (annualized) | ≥ $5M |
| `SFORGE` DAO participation rate | ≥ 10% (of holders voting per proposal) |
| Zero critical security incidents | Required |

---

## 17. Open Questions & Risks

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Soroban VM breaking changes in future SDK versions | Medium | High | Pin SDK version; maintain upgrade test suite |
| ZK verifier WASM size exceeds Soroban contract size limit | Medium | High | Explore recursive proofs; split verifier across multiple contracts |
| Stellar IBC light client not available by Phase 4 target | Medium | Medium | Fall back to trusted relayer bridge; pursue IBC development in parallel |
| Oracle unavailability causing NAV staleness lockout | Low | High | Multiple oracle sources; manual override by Security Council |

### Regulatory Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Key jurisdiction (USA, EU) bans on-chain RWA | Low | Critical | Compliance-first design makes adaptation easier; legal counsel on retainer |
| SEC classifies `SFORGE` as a security | Medium | High | Structure distribution carefully; avoid promises of profit; utility-only marketing |
| FATF travel rule requiring identity disclosure on transfers | High | Medium | ZK compliance proofs in Phase 4 are designed to satisfy travel rule |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Insufficient developer community adoption | Medium | High | Invest in documentation, examples, and contributor incentives from day one |
| Competitor protocol launches on Stellar with VC backing | Medium | Medium | Open-source advantage; composability means StellarForge can integrate, not compete |
| Core team bandwidth insufficient for 5-phase roadmap | High | Medium | Phased approach; community contributors; grant-funded development |

---

## 18. Glossary

| Term | Definition |
|---|---|
| **RWA** | Real World Asset — a financial claim on a physical or off-chain asset (real estate, commodities, private credit, etc.) |
| **Soroban** | The smart contract platform on the Stellar blockchain |
| **Fractionalization** | Dividing an asset token into smaller units (fractional shares) to lower investment minimums |
| **Tranche** | A structured risk class of a pooled asset (senior = lower risk/return; junior = higher risk/return) |
| **NAV** | Net Asset Value — the current fair market value of the underlying asset, used to price fractional shares |
| **KYC** | Know Your Customer — identity verification process required for financial compliance |
| **AML** | Anti-Money Laundering — regulatory requirements to detect and prevent money laundering |
| **ZK-SNARK** | Zero-Knowledge Succinct Non-Interactive Argument of Knowledge — a cryptographic proof that proves knowledge without revealing the knowledge itself |
| **IBC** | Inter-Blockchain Communication — a Cosmos SDK protocol for trustless cross-chain messaging |
| **Oracle** | An off-chain data source (e.g., asset price feed) that supplies data to smart contracts |
| **Waterfall** | The payment priority order in tranched structures (senior paid first, junior last) |
| **SFORGE** | The StellarForge governance token |
| **DAO** | Decentralized Autonomous Organization — a governance structure where token holders vote on protocol decisions |
| **Timelock** | A delay between proposal approval and execution, allowing time for review or veto |
| **Circuit breaker** | An emergency pause mechanism that halts contract operations |
| **Solvency ratio** | Insurance pool: (available capital) / (maximum liability); must exceed a minimum threshold |
