# External Audit Handover — Phase 1

**Audience:** the firm engaged for the Phase 1 external audit (NFR-S-6).
**Status:** Preparation. The commit to audit is frozen when the engagement starts; see [Commit and build](#2-commit-and-build).
**Deployments:** testnet only. No contract holds real funds, and none is deployed to mainnet.

This document gives an auditor what the code alone does not: what is in scope, how to rebuild exactly what was reviewed, who is trusted with what, the invariants the contracts are meant to keep, the design choices that are deliberate rather than defects, and where the product requirements describe more than Phase 1 implements.

---

## 1. Scope

### In scope

The four Phase 1 Soroban contracts and the crate they share. Counts are Rust source lines in `src/`, excluding tests.

| Crate | Path | Lines | Role |
|---|---|---|---|
| `rwa-asset` | `contracts/rwa-asset` | 786 | The token: SEP-41 surface, issuance, pause, metadata, compliance screening |
| `compliance` | `contracts/compliance` | 261 | KYC records that `rwa-asset` screens transfers against |
| `registry` | `contracts/registry` | 279 | Admin-maintained directory of asset contracts |
| `governance` | `contracts/governance` | 416 | Proposals and voting; advisory in Phase 1 |
| `stellarforge-common` | `contracts/common` | 83 | Shared storage TTL policy; a compile-time library, never deployed |

### Optional, if the engagement allows

The TypeScript SDK in `sdk/` (`@stellarforge-protocol/sdk`). It holds no funds, but it builds and signs transactions, and two parts carry real risk if wrong:

- **Sponsored writes** (`sdk/src/sponsored.ts`, `sdk/src/base.ts`): checking signed authorization entries before a fee payer submits them.
- **Settlement handling** (`awaitSettlement` in `base.ts`): classifying an unsettled transaction so that callers do not retry a write that may still land.

### Out of scope

- Deployment and CI tooling (`scripts/`, `.github/workflows/`). The internal review covered it (IR-10 to IR-13).
- Phase 2+ contracts, which do not exist yet.
- Stellar core, Soroban host behaviour and `soroban-sdk` itself, except where our code depends on a specific behaviour (noted below).

---

## 2. Commit and build

**Commit.** At engagement start the maintainers tag the audited commit as `audit-phase1-<YYYY-MM-DD>` on `main` and send the tag and full hash. Findings should reference that commit. Fixes land as later commits, so the tag stays a fixed reference.

**Toolchain.** Everything that affects the wasm bytes is pinned (NFR-S-5):

| Component | Version | Where |
|---|---|---|
| Rust | 1.98.1 | `rust-toolchain.toml` |
| `soroban-sdk` | 27.0.5 (see `Cargo.lock` for the resolved version) | `Cargo.toml` |
| `stellar-cli` | 27.1.0, checksum-verified in CI | `.github/workflows/contracts-ci.yml` |
| Target | `wasm32v1-none` | `rust-toolchain.toml` |

**Rebuilding.**

```bash
rustup toolchain install          # installs the pinned toolchain from rust-toolchain.toml
make build                        # stellar contract build, for all four contracts
sha256sum target/wasm32v1-none/release/*.wasm
```

The **Contracts CI** workflow publishes `SHA256SUMS` next to the wasm artifacts for every commit it builds, so the hashes of the tagged commit can be compared with a local build. Nobody has yet reproduced the bytes on a second machine, so a mismatch is worth reporting rather than assuming.

**Tests.**

```bash
cargo test --workspace                       # 145 contract tests pass; 1 is ignored
cargo test -p registry -- --ignored          # the 10,000-asset registry test (slow)
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Per crate, counting the ignored test: `rwa-asset` 60, `compliance` 26, `registry` 26, `governance` 34, for 146 in all. Live testnet coverage runs in the **Testnet Smoke** workflow (read and write paths, described in `CONTRIBUTING.md`).

---

## 3. System overview

- **`rwa-asset`** holds balances, allowances and total supply. `mint` is restricted to addresses holding the issuer role. When a compliance contract is configured, `mint`, `transfer` and `transfer_from` call `screen` on it for each counterparty (ADR-004). `set_paused` halts holder operations.
- **`compliance`** stores one KYC record per address (level 0–3, ISO 3166-1 alpha-2 jurisdiction, expiry). `is_compliant` is a pure query; `screen` is identical except that it extends the record's TTL, and is what the asset calls.
- **`registry`** records asset contracts in an indexed directory, with pages of at most 100 addresses.
- **`governance`** records proposals and declared-weight votes. Outcomes execute nothing in Phase 1.

Every contract is configured in a `__constructor` at deploy, so there is no uninitialized window. None has an upgrade entry point (ADR-003). Storage writes extend entries to the network maximum TTL, and reads extend nothing (ADR-001).

---

## 4. Trust model

| Role | Held by | Can | Cannot |
|---|---|---|---|
| Asset admin | One address per `rwa-asset` | Grant or revoke issuers; pause; update metadata within the rules in §5; set or remove the compliance contract; hand the role over with the new admin's signature | Mint, move or burn tokens; change `decimals`; remove a supply cap or set it below supply |
| Issuer | Any number of addresses | Mint up to the cap to addresses that pass screening | Anything else |
| Compliance admin | One address per `compliance` | Set and revoke KYC records; hand the role over | Affect an asset that does not point at this contract |
| Registry admin | One address | Register assets and set their `active` flag; hand the role over | Affect any asset contract |
| Governance admin | One address | Hand the role over. It has no other power in Phase 1 | — |
| Anyone | — | Call `screen` (only extends a record's TTL) and `finalize` (settles a proposal after its deadline) | — |

Admins are trusted. A default deploy (`scripts/deploy.sh`) makes one key the admin of all four contracts, and the deployment docs warn against doing that in production. A compromised compliance admin can make any address compliant on every asset that screens against it. A compromised asset admin can switch screening off or grant itself the issuer role.

---

## 5. Invariants the contracts are meant to keep

Please treat a way to break any of these as a finding.

**`rwa-asset`**
1. The sum of all balances equals `total_supply`.
2. While `max_supply > 0`, `total_supply <= max_supply`. The admin can never set a nonzero cap back to 0 or below the current supply.
3. `decimals` never changes after construction.
4. Only an address with the issuer role, authorizing the call, can mint.
5. No balance changes without the authorization of the address losing tokens, or of a spender within its unexpired allowance.
6. `transfer_from` and `burn_from` never spend more than the allowance, and an allowance reads as zero after its `live_until_ledger`.
7. A transfer to oneself changes no balance and publishes no event.
8. While paused, `mint`, `burn`, `burn_from`, `transfer`, `transfer_from` and `approve` all fail. Admin operations still work.
9. When a compliance contract is set, each counterparty in the ADR-004 table is screened, and a failing or missing compliance contract makes the screened operation fail, never pass.
10. Every state change publishes an event, whose wire format the tests pin.

**`compliance`**
11. `is_compliant` and `screen` return the same answer for the same state.
12. A record with a level above 3, a non-zero expiry at or before the ledger time, or a malformed jurisdiction is never stored.

**`registry`**
13. Each asset address appears in the directory at most once, and no single call's footprint grows with the directory's size.

**`governance`**
14. Each address votes at most once per proposal, and only until the deadline ledger.
15. A proposal is finalized exactly once, and only after its deadline.

**All contracts**
16. Every privileged entry point requires the authorization the ADR-002 map lists, and `transfer_admin` requires both the current and the incoming admin.
17. Error discriminants never change meaning (ADR-003).

---

## 6. Deliberate design choices (not findings)

Each is recorded in an ADR with its reasoning. We are glad to discuss any of them, but they are intentional.

| Choice | Where recorded |
|---|---|
| No upgrade entry point on any contract. A deployed `rwa-asset` cannot be fixed, only migrated, and its balances cannot be exported. | ADR-003 |
| An asset with no compliance contract transfers between anyone (fails open). Screening starts once `set_compliance` names a contract. | ADR-004 |
| `min_level = 0` still requires every party to hold a record. Only removing the compliance contract turns screening off. | ADR-004 |
| `burn`, `burn_from` and `approve` screen nobody, so a holder whose verification lapsed can still exit. | ADR-004 |
| `screen` writes (a TTL extension) without authorization. | ADR-002, ADR-004 |
| `finalize` requires no authorization. | ADR-002 |
| Governance weights are declared by the voter and verified against nothing, and there is no quorum. Outcomes are advisory and gate nothing in Phase 1. | `contracts/governance/src/lib.rs` module docs |
| The admin may *raise* a supply cap, which is dilutive, and the change is visible through `MetadataUpdated`. | Internal review IR-02 |
| Registry entries are admin assertions: nothing verifies that an address is a deployed `rwa-asset`, and nothing reads `active`. | Internal review IR-16 |
| `approve` replaces the old amount outright, so the SEP-41 approve-overwrite race applies. | `rwa-asset` `approve` docs |
| Reads never extend TTL. An entry idle beyond the maximum TTL costs its next user a restore fee. | ADR-001 |
| `AlreadyInitialized` and `NotInitialized` are unreachable, and kept so discriminants stay stable. | ADR-002, ADR-003 |

---

## 7. Where the PRD describes more than Phase 1 implements

`docs/prd/PRODUCT_REQUIREMENTS.md` covers all five phases. These statements do not hold for the Phase 1 code, and should not be read as requirements it meets:

| PRD statement | Phase 1 reality |
|---|---|
| §10.2: the hard supply cap is "not bypassable by admin" | The admin cannot remove a cap or set it below supply, but can raise it (§6) |
| §10.2: governance takeover is mitigated by "minimum quorum requirements; optimistic timelock; guardian veto" | No quorum, timelock or veto exists; governance is advisory (§6) |
| §10.4: a Security Council can "initiate emergency contract upgrades" | No contract can be upgraded (ADR-003). The only emergency control is `set_paused` on `rwa-asset` |
| NFR-R-1: upgrades gated by admin authorization and a governance vote (Phase 3+) | Not applicable: there is no upgrade path to gate |

---

## 8. Known limitations and platform dependencies

- **Constructor authorization.** Tests prove each constructor *demands* the admin's authorization (`tests/test_constructor_auth.rs`). That the network then refuses a deploy without that signature is host behaviour; no test of ours shows it.
- **TTL and archival.** Correctness assumes protocol 23+ automatic restore of archived persistent entries. Testnet runs protocol 28.
- **Cross-contract trust.** `rwa-asset` calls whatever address `set_compliance` names, via a one-function interface. Soroban prevents re-entry, but the configured contract's answers are trusted completely.
- **Expired allowances stay in storage**, reading as zero. Nothing removes them.
- **Governance proposal state** is bounded by a 1–90 day voting period and a 256-byte title. There is no deposit, so spam costs only fees and rent.

---

## 9. Prior work

- **Internal security review:** [`2026-09-14-internal-review-phase1.md`](2026-09-14-internal-review-phase1.md). It found 17 issues (6 Medium, 8 Low, 3 Info); all are fixed, and each is recorded in [`SECURITY.md`](../SECURITY.md) as SF-2026-003 to SF-2026-019. SF-2026-001 (critical) and SF-2026-002 (high) predate it.
- **Architecture decisions:** [`docs/architecture/`](../architecture/) — ADR-001 storage keys and TTL, ADR-002 authorization (with amendments), ADR-003 upgrade path, ADR-004 cross-contract interaction.
- **Requirements:** [`docs/prd/PRODUCT_REQUIREMENTS.md`](../prd/PRODUCT_REQUIREMENTS.md), NFR-S-1 to NFR-S-6 in particular, read alongside §7 above.

---

## 10. Where scrutiny is most wanted

1. **Authorization completeness** against the ADR-002 map, including the documented exceptions (`transfer_from`, `burn_from`, `screen`, `finalize`).
2. **Allowance semantics**: expiry, spending, and the self-`transfer_from` path, which spends the allowance without moving tokens.
3. **Compliance screening**: every path that moves value to or from a holder, and the fail-closed behaviour when the compliance contract misbehaves.
4. **Supply accounting** across `mint`, `burn`, `burn_from` and the cap rules in `update_metadata`.
5. **Storage lifetime**: whether any write path fails to extend what it writes, and whether archival and restore can break an invariant in §5.
6. **Registry growth**: any path by which directory size affects a single call's cost.
7. **Event completeness** for indexers, and whether any event can be emitted for a change that did not happen.

---

## 11. Logistics

- **Reporting findings:** privately, through GitHub's [private vulnerability reporting](https://github.com/0xLizTech/stellarforge/security/advisories/new) on this repository, or a channel agreed at engagement start. Please do not open public issues for findings.
- **Report publication:** the final report is published in full in `docs/audits/`, as the PRD commits to (§10.3), after the findings are fixed.
- **Fix verification:** each fix references its finding and adds a regression test, as the internal review's fixes did.
