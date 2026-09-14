# Internal Security Review — Phase 1

**Status:** Open. Findings are not yet fixed.
**Date:** 2026-09-14
**Commit reviewed:** `1f43935` (`main`)
**Roadmap item:** Phase 1, *Formal security review (internal)*

> **Handling.** [SECURITY.md](../SECURITY.md) says findings are recorded publicly
> *once fixed*. This report lists open findings. Keep it off the public `main`
> branch until the Medium findings are fixed, or move it into a private
> security advisory. None of the contracts are deployed on mainnet, so no funds
> are at risk today.

---

## 1. Scope

| Component | Files | Lines |
|---|---|---|
| `rwa-asset` | `contracts/rwa-asset/src/*` | 554 |
| `compliance` | `contracts/compliance/src/*` | 147 |
| `registry` | `contracts/registry/src/*` | 147 |
| `governance` | `contracts/governance/src/*` | 279 |
| `common` (TTL policy) | `contracts/common/src/*` | 83 |
| TypeScript SDK | `sdk/src/*` | 873 |
| Deployment and CI | `scripts/deploy.sh`, `Makefile`, `.github/workflows/*`, `rust-toolchain.toml` | — |

The review used the PRD's non-functional requirements (NFR-S-*, NFR-A-*,
NFR-P-3) and ADRs 001–004 as the specification. Where an ADR records a
behaviour as a deliberate choice, this report does not count it as a finding.
Examples are the unscreened `burn`, the permissionless `finalize`, advisory
governance weights and fail-open compliance when none is configured.

### Method

- Manual line-by-line review of every contract entry point against the ADR-002
  authorization map and ADR-004 screening table.
- Proof-of-concept tests for IR-01 and IR-02. They ran under
  `InvocationResourceLimits::mainnet()`, which is the `soroban-sdk` 27.0.6
  test default, and were deleted afterwards.
- Direct checks of the SDK utilities with malformed input.
- `cargo test --workspace`: 92 tests pass. `cargo clippy --all-targets`: clean.
- `npm audit --omit=dev`: 0 vulnerabilities. `cargo audit` did not run
  locally, because the tool is not installed. CI runs it weekly.
- A scan of the full git history for Stellar secret keys and credential
  assignments found nothing.

### Severity scale

| Severity | Meaning |
|---|---|
| Critical | Unauthorized minting, theft, or loss of funds |
| High | Loss of control of an asset, or a core asset function permanently broken |
| Medium | Breaks a guarantee holders or operators rely on, permanently disables a component, or violates a MUST requirement with security impact |
| Low | Limited impact, defence in depth, or hygiene |
| Info | Observation. No change strictly required |

**Why severity is higher here than it would be elsewhere.** ADR-003 ships no
upgrade entry point, and `rwa-asset` cannot migrate once it has holders. A
contract-level finding that survives until deployment therefore stays
permanently. Every contract finding at Medium or above must be fixed before
any deployment that will hold real positions.

---

## 2. Summary

| ID | Title | Component | Severity |
|---|---|---|---|
| IR-01 | Registry stops accepting assets after ~1,600 entries | registry | Medium |
| IR-02 | `update_metadata` can break the supply cap and re-denominate balances | rwa-asset | Medium |
| IR-03 | Privileged and governance state changes emit no events | all contracts | Medium |
| IR-04 | `compliance`, `registry` and `governance` admins cannot be rotated | compliance, registry, governance | Medium |
| IR-05 | SDK can report a failed transaction that later succeeds | sdk | Medium |
| IR-06 | SDK amount helpers silently accept malformed input | sdk | Medium |
| IR-07 | `update_metadata` and `transfer_admin` skip the TTL policy | rwa-asset | Low |
| IR-08 | `set_kyc` accepts out-of-range levels and past expiries | compliance | Low |
| IR-09 | Token surface diverges from SEP-41; allowances never expire | rwa-asset | Low |
| IR-10 | Build is not reproducible (NFR-S-5) | toolchain, CI | Low |
| IR-11 | Dependency review never runs, and no SDK dependency audit exists | CI | Low |
| IR-12 | CI trusts mutable action tags and an unverified CLI download | CI | Low |
| IR-13 | Deployment tooling lacks mainnet guards and has stale paths | scripts, Makefile | Low |
| IR-14 | `propose` accepts a zero voting period and unbounded input | governance | Low |
| IR-15 | Self-`transfer_from` emits a phantom `Transfer` event | rwa-asset | Info |
| IR-16 | Registry entries are unverified, and `active` is unenforced | registry | Info |
| IR-17 | Constructor `require_auth` is untested (carried from ADR-002) | all contracts | Info |

**Counts:** 0 Critical, 0 High, 6 Medium, 8 Low, 3 Info.

### Confirmed sound

The review looked for these problems specifically and found none:

- **Authorization.** Every privileged entry point matches the ADR-002 map. The
  only exceptions are the three documented ones: `transfer_from`, `screen` and
  `finalize`. NFR-S-4 dual authorization on `transfer_admin` is present.
- **Arithmetic.** Additions to balances, supply and tallies are checked.
  Subtractions are preceded by a bound check. The release profile sets
  `overflow-checks = true`. There is no `unsafe`, `unwrap()` or `expect()` in
  contract code (NFR-S-2, NFR-S-3).
- **SF-2026-001 fix.** Both `transfer` and `transfer_from` handle `from == to`
  without creating tokens, and balances cannot go negative.
- **Re-entrancy.** The Soroban host rejects contract re-entry, so a configured
  compliance contract cannot call back into the asset mid-transfer.
- **Compliance failure mode.** Once configured, a missing, wrong or panicking
  compliance contract makes `mint`, `transfer` and `transfer_from` fail
  closed.
- **SDK transport.** The RPC client sets `allowHttp: false`. The SDK checks
  that the signer matches the source before signing. No secrets are logged.

---

## 3. Findings

### IR-01 — Registry stops accepting assets after ~1,600 entries

**Severity:** Medium  **Location:** `contracts/registry/src/lib.rs:56-95`

`DataKey::AssetList` is a single persistent `Vec<Address>`. `register` reads
the whole vector, appends to it and writes it back. Each contract address
takes 40 bytes of XDR, and a contract-data entry is limited to 65,536 bytes on
mainnet. After roughly 1,600 registrations every further `register` exceeds
the limit and fails. No entry point removes addresses from the list, and no
upgrade path exists (ADR-003), so a registry in this state can never register
another asset.

**Proof of concept.** In a loop that registered fresh contract addresses under
mainnet limits, the first 1,000 succeeded. A later call then failed with:

```
HostError: Error(Budget, ExceededLimit)
contract data entry with key '...AssetList...' size: 65540 > 65536
```

The failure happens well before that point in practice too. Each
registration rewrites the whole list, so its write cost grows linearly, with
the per-transaction `write_bytes` limit at 132,096. `list_assets` also returns
the entire list in one read. NFR-P-3 requires 10,000 assets, which is more
than six times the actual ceiling.

**Recommendation.** Replace the vector with an indexed layout:
`AssetCount -> u32` plus `AssetAt(u32) -> Address`. Paginate reads with
`list_assets(start: u32, limit: u32)`. Add a test that registers more than
2,000 assets under the default mainnet limits.

---

### IR-02 — `update_metadata` can break the supply cap and re-denominate balances

**Severity:** Medium  **Location:** `contracts/rwa-asset/src/lib.rs:338-344`

`update_metadata` runs only `validate_metadata`, which checks the shape of the
value and never compares it with the asset's current state. The admin can,
at any time and without emitting an event:

1. Set `max_supply` below the circulating `total_supply`.
2. Set `max_supply` to `0`, which removes the cap entirely.
3. Change `decimals` after issuance. Every holder's position is then silently
   re-denominated by a power of ten, although raw balances are unchanged.
4. Replace `legal_doc_hash`, which changes the legal terms the token claims to
   represent.

**Proof of concept.** With the asset capped at 1,000 and 1,000 already minted:

```
update_metadata(max_supply = 10)            -> accepted; total_supply=1000 > max_supply=10
update_metadata(decimals = 0, max_supply=0) -> accepted; asset now uncapped
mint(1_000_000)                             -> accepted; total_supply=1001000
```

The admin is a trusted role. Still, the supply cap and the decimals are the
on-chain guarantees an RWA holder relies on, and ADR-003 argues against
exactly this power when it explains why an admin upgrade was rejected: "the
admin can rewrite the rules at any time". This entry point hands the admin a
narrower version of that same power.

**Recommendation.**
- Make `decimals` immutable after construction.
- Once `max_supply > 0`, accept only a new value `>= total_supply` and `> 0`,
  so a cap can be tightened down to circulating supply but never removed.
- Emit a `MetadataUpdated` event carrying the old and new `legal_doc_hash`
  (see IR-03).
- Consider moving `name`, `symbol` and `asset_class` into a separate
  entry point from the economic fields.

---

### IR-03 — Privileged and governance state changes emit no events

**Severity:** Medium  **Requirement:** NFR-A-1, NFR-A-3

Only `rwa-asset`'s holder operations and `set_paused` publish events. The
following state changes are invisible to indexers:

| Contract | Silent entry points |
|---|---|
| `rwa-asset` | `set_issuer`, `update_metadata`, `transfer_admin`, `set_compliance` |
| `compliance` | `set_kyc`, `revoke_kyc` |
| `registry` | `register`, `set_active` |
| `governance` | `propose`, `vote`, `finalize` |

**Security impact.** The changes a monitor most needs to see are exactly the
ones that go unseen: an admin handover, a new issuer grant, compliance being
switched off with `set_compliance(None, _)`, or an address being KYC-approved.
ADR-004 names `compliance_contract()` returning `None` as "the check an
operator, an auditor or a monitoring system should be making". Without an
event, a monitor can only catch that by polling every asset. The ADR-003
migration table also says governance "history is preserved in events", but no
governance events exist.

**Recommendation.** Add `#[contractevent]` types for every entry point listed
above, following the existing `events.rs` pattern. Extend `test_events.rs` so
that every state-changing entry point is asserted to publish.

---

### IR-04 — `compliance`, `registry` and `governance` admins cannot be rotated

**Severity:** Medium  **Location:** `contracts/compliance/src/lib.rs`, `contracts/registry/src/lib.rs`, `contracts/governance/src/lib.rs`

Only `rwa-asset` exposes `transfer_admin`. In the other three contracts,
the admin set in the constructor holds the role for the contract's whole
lifetime.

The compliance contract carries the most risk:

- **Compromised key.** An attacker can `set_kyc` any address at level 3 with
  `expires_at = 0`, which makes it permanently compliant on every asset that
  points at this engine. The compromised admin cannot be removed. The only
  remedy is to deploy a new engine, re-issue every record, and have **each
  asset's admin** call `set_compliance`.
- **Lost key.** KYC records can no longer be issued or renewed. As records
  expire, every transfer on every dependent asset starts to fail.

The PRD threat model (§ Admin key compromise) cites dual-authorization admin
transfer as the mitigation. That mitigation exists in only one of the four
contracts.

**Recommendation.** Add `transfer_admin` with dual `require_auth` to
`compliance`, `registry` and `governance`, matching `rwa-asset` and NFR-S-4,
and have each one emit an event. Record in the deployment runbook that the
default deploy uses one key as admin of all four contracts (ADR-002 notes
this) and that production deployments should not.

---

### IR-05 — SDK can report a failed transaction that later succeeds

**Severity:** Medium  **Location:** `sdk/src/base.ts:128-162`

Write transactions are built with a 180-second validity window
(`WRITE_TX_TIMEOUT_SECONDS`). `signAndSubmit` then calls
`server.pollTransaction(hash)` with its defaults. In `@stellar/stellar-sdk`
16.2.0 those defaults are 30 attempts at 1-second intervals, about 30 seconds.
If the transaction is still pending when polling ends, the method throws
`did not succeed: NOT_FOUND`. The transaction can still be included in a
ledger for roughly another 150 seconds.

`sendTransaction` statuses other than `ERROR` also fall through to polling.
These include `TRY_AGAIN_LATER`, where the transaction was never accepted, and
`DUPLICATE`.

**Impact.** The signing path exists for server-side automation: treasury
services and issuance desks (ADR-002). A caller that retries after this error
builds a new transaction with a new sequence number. If the first one then
lands, a `mint` or `transfer` executes twice.

**Recommendation.**
- Poll until the transaction's `timeBounds.maxTime` has passed, not for a
  fixed attempt count.
- Throw a distinct error type, such as `TransactionOutcomeUnknown`, carrying
  the hash, so callers know to check that hash before retrying.
- Treat `TRY_AGAIN_LATER` as "not submitted" explicitly, and `DUPLICATE` as
  "already pending".

---

### IR-06 — SDK amount helpers silently accept malformed input

**Severity:** Medium  **Location:** `sdk/src/utils.ts:30-63`

These helpers convert user-entered amounts into the integers passed to
`mint`, `transfer` and `approve`. Confirmed behaviour:

| Call | Result | Problem |
|---|---|---|
| `toStroops("1.2.3")` | `12000000n` | Malformed input accepted as 1.2 |
| `toStroops("1.99999999")` | `19999999n` | Excess precision silently truncated |
| `toStroops("0x10")` | `4294967296n` | Parsed as hexadecimal |
| `toStroops(12345678901234567890)` | `…67000…n` | `number` above 2^53 loses precision silently |
| `fromStroops(100n, 0)` | `"0.1"` | 100 whole tokens displayed as 0.1. The contract allows `decimals` 0–18 |
| `fromStroops(-5n)` | `"0.00000-5"` | Negative values garbled |
| `hexToBytes32("zz…")` | all zero bytes | Non-hex silently becomes zeros, which weakens NFR-A-2 document-hash integrity |

**Recommendation.** Validate `toStroops` input against
`/^-?\d+(\.\d+)?$/`. Reject a fraction longer than `decimals` rather than
truncating it. Accept `number` only when `Number.isSafeInteger` holds for
the scaled value, or accept only `string`. Handle `decimals === 0` and
negative values in `fromStroops`. Validate `/^[0-9a-fA-F]{64}$/` in
`hexToBytes32`. Add tests for each row above.

---

### IR-07 — `update_metadata` and `transfer_admin` skip the TTL policy

**Severity:** Low  **Location:** `contracts/rwa-asset/src/lib.rs:338-350`

`contracts/common/src/storage.rs` says write paths must extend the entries
they write, which is the SF-2026-002 fix. `update_metadata` writes
`DataKey::Metadata`, and `transfer_admin` writes instance storage, but neither
calls `extend_persistent` or `extend_instance`. Since protocol 23 an archived
entry is restored automatically when next accessed, so the consequence is a
restore fee rather than a failure. The gap is still a regression of a policy
this repository adopted after an actual incident.

**Recommendation.** Add the extensions. A test that walks every entry point
and asserts the TTL of each key it wrote would catch any future omission.

---

### IR-08 — `set_kyc` accepts out-of-range levels and past expiries

**Severity:** Low  **Location:** `contracts/compliance/src/lib.rs:55-62`

`KycRecord.level` is documented as 0–3, but `set_kyc` accepts any `u32`. A
record with `level = u32::MAX` satisfies every `min_level` an asset could ever
configure, including tiers added later. `expires_at` values already in the
past are accepted, and `jurisdiction` is not checked against the ISO-3166
alpha-2 shape. All three are admin input errors that produce wrong screening
results without any error.

**Recommendation.** Reject `level > 3`, reject a non-zero
`expires_at <= ledger().timestamp()`, and require `jurisdiction` to be two
ASCII uppercase letters.

---

### IR-09 — Token surface diverges from SEP-41; allowances never expire

**Severity:** Low  **Location:** `contracts/rwa-asset/src/lib.rs:219-286`

ADR-002 describes the token interface as following SEP-41, but it differs in
several ways:

- `approve` has no `expiration_ledger`. Allowances last indefinitely: they
  are extended to the maximum TTL on write and restored automatically if
  archived.
- `name()`, `symbol()` and `decimals()` getters are missing (only
  `metadata()` exists).
- `burn_from` is missing.

Wallets and indexers written for SEP-41 will fail to call this contract, or
misread it. Because approvals never expire, a forgotten approval remains
exploitable until the owner revokes it. The usual approve-overwrite race also
applies: a spender who watches for a pending `approve(N → M)` can spend N
before it lands, then spend M afterwards.

**Recommendation.** Adopt SEP-41's `approve(from, spender, amount,
expiration_ledger)` and store allowances with the expiry, following the
reference token. Add the metadata getters and `burn_from`, or document
explicitly that the contract is not SEP-41 compatible.

---

### IR-10 — Build is not reproducible (NFR-S-5)

**Severity:** Low  **Location:** `rust-toolchain.toml:2`, `.github/workflows/*.yml`

`rust-toolchain.toml` pins `channel = "stable"`, and CI installs
`dtolnay/rust-toolchain@stable`. Wasm output depends on the `rustc` version,
so a given commit produces different bytes over time. As a result, nobody can
verify that a deployed wasm hash matches the audited source. That check
matters most for contracts that cannot be upgraded. NFR-S-5 requires a pinned
toolchain.

**Recommendation.** Pin an exact version, for example `channel = "1.xx.y"`.
Use it in CI through `rust-toolchain.toml` instead of `@stable`, and publish
the sha256 of each release wasm alongside the tag.

---

### IR-11 — Dependency review never runs, and no SDK dependency audit exists

**Severity:** Low  **Location:** `.github/workflows/security.yml:1-32`

- The `dependency-review` job is conditioned on
  `github.event_name == 'pull_request'`, but the workflow has no
  `pull_request` trigger. The job therefore never runs.
- `cargo-audit` runs only after a merge to `main` and weekly, so an
  advisory-affected dependency can merge without being flagged.
- No workflow runs `npm audit` for the SDK, although `publish-sdk.yml`
  publishes it to npm.

**Recommendation.** Add `pull_request` to the workflow's triggers, run
`cargo audit` on pull requests, and add an `npm audit --omit=dev` step to
`sdk-ci.yml`.

---

### IR-12 — CI trusts mutable action tags and an unverified CLI download

**Severity:** Low  **Location:** `.github/workflows/contracts-ci.yml`, `testnet-smoke.yml`, `publish-sdk.yml`

- Third-party actions are referenced by mutable tags:
  `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2`.
- `stellar-cli` is fetched with `curl | tar` and never checked against a
  checksum, then used to build the wasm artifacts that get deployed.
- `publish-sdk.yml` runs with `id-token: write` and `NPM_TOKEN` in scope.

**Recommendation.** Pin third-party actions to full commit SHAs, verify the
`stellar-cli` tarball against a checksum committed in the workflow, and
restrict publishing to a protected environment that requires reviewer
approval.

---

### IR-13 — Deployment tooling lacks mainnet guards and has stale paths

**Severity:** Low  **Location:** `scripts/deploy.sh:24-33`, `Makefile:47-53`, `.env.example:21`

- `deploy.sh` accepts `NETWORK=mainnet` with the placeholder metadata, which
  includes an all-zero `legal_doc_hash`. It also makes the deploying hot key
  the admin of all four contracts.
- `make deploy-testnet` deploys `rwa-asset` without constructor arguments, so
  it now fails. `.env.example` claims it writes `deployed-contracts.json`, but
  only `deploy.sh` does.

**Recommendation.** When `NETWORK=mainnet`, refuse to run unless every
`ASSET_*` variable and `ADMIN` are set explicitly and the legal document hash
is non-zero. Point `make deploy-testnet` at `scripts/deploy.sh` and correct
`.env.example`.

---

### IR-14 — `propose` accepts a zero voting period and unbounded input

**Severity:** Low  **Location:** `contracts/governance/src/lib.rs:81-131`

- With `voting_period_ledgers = 0`, voting is open only during the ledger the
  proposal is created in, and the proposer can finalize in the next ledger. No
  one else has a practical chance to see the proposal or vote on it.
- `title` has no length bound beyond the ledger entry limit.
- Proposing requires no deposit, so spam costs only fees and rent.

Governance is advisory in Phase 1, so the impact is low. These properties
become important once Phase 3 attaches execution to outcomes.

**Recommendation.** Enforce minimum and maximum voting periods and a
`title` length cap now, so that Phase 3 does not inherit proposals created
under looser rules.

---

### IR-15 — Self-`transfer_from` emits a phantom `Transfer` event

**Severity:** Info  **Location:** `contracts/rwa-asset/src/lib.rs:259-285`

A self-`transfer` returns early and publishes nothing
(`test_self_transfer_no_op_emits_nothing`). A self-`transfer_from` consumes
the allowance and publishes `Transfer { from, to: from, amount }`, although no
balance changed. Indexers that sum events will record a movement that did not
happen.

**Recommendation.** Either suppress the event when `from == to`, or document
that indexers must ignore self-transfers. Also emit an allowance-change event,
so consumption of the allowance is still visible.

---

### IR-16 — Registry entries are unverified, and `active` is unenforced

**Severity:** Info  **Location:** `contracts/registry/src/lib.rs:56-109`

`register` does not check that the address is a deployed `rwa-asset`, or that
`asset_class` matches that asset's metadata. `active` is a label only: nothing
in the protocol reads it. ADR-003 already frames the registry as a directory.
Integrators should be told explicitly that a registry entry is not proof that
an asset is genuine or current.

---

### IR-17 — Constructor `require_auth` is untested

**Severity:** Info  **Location:** every `__constructor`

The ADR-002 amendment already records this. It is carried forward here so it
is tracked to closure: `Env::register` mocks constructor authorization, so no
test asserts that deployment fails without the admin's signature. The fix is
a test that uploads the wasm and deploys through `env.deployer()` without
mocked authorization.

---

## 4. Recommended order of work

1. **Before any contract is deployed with holders.** Fix IR-01, IR-02, IR-03,
   IR-04 and IR-07. These are permanent once deployed (ADR-003).
2. **Before `@stellarforge/sdk` is published to npm.** Fix IR-05 and IR-06.
3. **Before the external audit.** Fix IR-10, IR-11 and IR-12, so auditors can
   tie a wasm hash to a commit. Then fix IR-08, IR-09, IR-13 and IR-14.
4. **Track.** IR-15, IR-16 and IR-17.

When a finding is fixed, move it into the *Resolved Issues* section of
[SECURITY.md](../SECURITY.md) with its commit and regression test, following
the SF-2026-00x format.
