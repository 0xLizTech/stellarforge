# StellarForge — Protocol Roadmap

This document describes the planned evolution of the StellarForge protocol across five phases. Timelines are approximate and subject to change based on community bandwidth, security audits, and ecosystem developments.

---

## Guiding Principles

- **Security before speed.** Each phase is audited before its contracts go to mainnet.
- **Composability over integration.** Modules are independent contracts; integrators may use any subset.
- **Compliance by design.** Regulatory guardrails are first-class primitives, not afterthoughts.
- **Progressive decentralization.** Start with multisig admin, migrate to on-chain governance as the protocol matures.

---

## Phase 1 — Foundation (Current)

**Theme:** Minimal viable tokenization with compliance hooks and governance scaffolding.

### Deliverables

- [x] `rwa-asset` contract: mint, burn, transfer, allowance, metadata, pause, issuer management
- [x] `registry` contract: global asset directory with active/inactive flags
- [x] `compliance` contract: KYC/AML record storage with jurisdiction and expiry
- [x] `governance` contract: proposal creation, voting, finalization
- [x] TypeScript SDK: clients for all four contracts (RwaAsset, Compliance, Registry, Governance)
- [x] CI/CD: GitHub Actions for contracts (check, test, build-wasm) and SDK
- [x] Testnet deployment scripts
- [x] Formal security review (internal): see [`docs/audits/2026-09-14-internal-review-phase1.md`](docs/audits/2026-09-14-internal-review-phase1.md)
- [x] Compliance middleware hook (integrate Compliance into RwaAsset transfer flow)
- [x] SDK: write-path transaction builder helpers
- [x] SDK: npm package publish to `@stellarforge-protocol/sdk` (0.1.0)

### Target: Q3 2026

---

## Phase 2 — Fractionalization & Yield

**Theme:** Break large assets into affordable fractional shares and distribute income to holders.

### Deliverables

- [ ] **`vault` contract:** Locks an underlying RWA asset and issues fractional ERC-20-style shares
  - Deposit / withdrawal with pro-rata share calculation
  - NAV (Net Asset Value) oracle integration
  - Time-locks and lock-up period enforcement
- [ ] **`yield-distributor` contract:** Distributes income (rent, dividends, coupons) to fractional holders
  - Snapshot-based distribution (balance at ledger N)
  - Multi-currency yield (XLM, USDC)
  - Unclaimed yield reclaim after configurable timeout
- [x] **`oracle-adapter` contract:** Standardized interface for price/NAV feeds. SEP-40, so Reflector and custom feeds read the same way; Pyth and Band wrappers when an asset needs them (ADR-005)
- [ ] SDK and deploy script: oracle adapter
- [x] SDK: write-path helpers for mint, burn, transfer, approve (0.2.0), plus sponsored writes (0.3.0)
- [ ] SDK: vault and yield-distributor clients
- [ ] Automated testnet deployment with deterministic contract IDs
- [ ] External security audit (Phase 1 + Phase 2 scope)
- [ ] Mainnet launch of Phase 1 contracts (post-audit)

### Target: Q4 2026

---

## Phase 3 — DeFi Primitives & Tranching

**Theme:** Connect RWA tokens to on-chain liquidity and introduce risk-stratified yield products.

### Deliverables

- [ ] **`tranche` contract:** Senior / mezzanine / junior tranches over a yield-bearing asset pool
  - Waterfall payment logic
  - Configurable risk parameters per tranche class
  - Compatible with external AMMs and Stellar DEX
- [ ] **`liquidity-pool` adapter:** Wrap Stellar DEX liquidity pools with RWA-specific access controls
- [ ] **Governance execution hooks:** Passed governance proposals can call contract functions directly (timelock + multisig guardian)
- [ ] **`SFORGE` governance token:** Non-transferable voting power; earned through protocol participation
- [ ] Governance-controlled protocol fee (0–50 bps; fee switch off by default)
- [ ] Protocol dashboard (Next.js reference app)
- [ ] External security audit (Phase 3 scope)
- [ ] Mainnet deployment of Phase 2 contracts

### Target: Q1 2027

---

## Phase 4 — Privacy & Cross-Chain

**Theme:** Enable confidential compliance verification and bridge RWA liquidity to other ecosystems.

### Deliverables

- [ ] **ZK compliance proofs:** Prove KYC/AML status without revealing identity — using ZK-SNARKs (Groth16 or PLONK)
  - Off-chain proof generation by compliant identity providers
  - On-chain verifier contract (Soroban WASM verifier)
  - Compatible with Semaphore / zkPassport identity primitives
- [ ] **`bridge` contract (Stellar → Ethereum):** Lock-and-mint pattern via a decentralized relayer network
  - IBC or LayerZero transport layer
  - ERC-20 wrapped RWA tokens on Ethereum (EVM side open-source, separate repo)
  - Canonical bridge with dispute window
- [ ] **`bridge` contract (Stellar → Cosmos):** IBC channel for Cosmos SDK chains
- [ ] Privacy-preserving transfers: Pedersen commitment balance model (research track — may become Phase 5)
- [ ] Multi-sig guardian retirement (replace with on-chain governance for all admin ops)
- [ ] External security audit (Phase 4 scope)
- [ ] Mainnet deployment of Phase 3 contracts

### Target: Q3 2027

---

## Phase 5 — Full DAO & Insurance

**Theme:** Complete decentralization. The protocol governs itself. Risk is socialized on-chain.

### Deliverables

- [ ] **`insurance-pool` contract:** Protocol-native coverage for smart contract failure and oracle manipulation
  - Permissionless policy purchases (pay premium, receive coverage)
  - Claims adjudication via DAO vote
  - Solvency ratio enforcement (minimum coverage ratio)
- [ ] **Full DAO migration:** All admin keys relinquished; all protocol parameters governed by `SFORGE` holders
  - Optimistic governance (executable after 48h timelock if no veto)
  - Emergency veto council (3-of-7 multisig of elected community members)
- [ ] **Protocol treasury:** On-chain treasury managed by DAO
  - Fee accumulation from all protocol operations
  - Grant program for ecosystem projects
  - Buyback-and-burn mechanism for `SFORGE`
- [ ] **Institutional custody integrations:** Fireblocks, Copper, BitGo adapters for enterprise issuers
- [ ] **Regulatory reporting module:** On-chain issuance ledger compatible with MiCA, SEC Reg D/A+, and MAS reporting requirements
- [ ] External security audit (Phase 5 scope)
- [ ] Mainnet deployment of Phase 4 contracts

### Target: Q1–Q2 2028

---

## Beyond Phase 5 — Research Tracks

These are longer-horizon ideas the team is actively researching but has not committed to:

- **Real-time NAV settlement:** Sub-second NAV updates via Stellar's native DEX + oracle aggregation
- **Tokenized debt issuance:** On-chain private credit with covenant monitoring
- **Carbon credit integration:** RWA primitives extended to verified carbon offset tokenization (Verra, Gold Standard)
- **AI-assisted due diligence:** On-chain attestation of off-chain AI audit reports from asset underwriters
- **Recursive ZK rollup:** Batch multiple RWA operations into a single Stellar transaction using validity proofs

---

## Milestones Summary

| Milestone | Phase | Target |
|---|---|---|
| Phase 1 testnet live | 1 | Q3 2026 |
| Phase 1 external audit | 1 | Q3 2026 |
| Phase 1 mainnet | 2 | Q4 2026 |
| Fractionalization live on testnet | 2 | Q4 2026 |
| Phase 2 external audit | 2 | Q4 2026 |
| Phase 2 mainnet | 3 | Q1 2027 |
| Governance token launch | 3 | Q1 2027 |
| Cross-chain bridge (Ethereum) | 4 | Q3 2027 |
| ZK compliance proofs | 4 | Q3 2027 |
| Full DAO | 5 | Q1 2028 |
| Insurance pool | 5 | Q2 2028 |

---

## How to Influence the Roadmap

1. Open a [GitHub Discussion](https://github.com/0xLizTech/stellarforge/discussions) for major feature proposals.
2. Comment on existing roadmap issues.
3. Vote using 👍 reactions on GitHub issues to surface demand.
4. Once `SFORGE` governance is live, all roadmap changes will be proposed on-chain.
