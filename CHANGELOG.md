# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `set_paused`/`paused` on `compliance`, `registry` and `oracle-adapter`, matching `rwa-asset`'s existing emergency pause. Each blocks only its own state-changing writes (`set_kyc`/`revoke_kyc`; `register`/`set_active`; `report`/`override_price`/`add_asset`/`set_max_deviation`/`set_reporter`); reads and `transfer_admin` stay open while paused, so a compromised admin key can always be rotated out and consumers never lose access to data already on record (#126). SDK bindings (`isPaused`/`setPaused`) added to `RegistryClient` and `OracleAdapterClient`.
- `tools/benchmark`, a CPU/memory/ledger-read-write resource baseline for every entry point across all five Phase 1/2 contracts, regenerated with `cargo run -p stellarforge-benchmark`. Committed at `docs/benchmarks/resource-baseline.md`; see `docs/benchmarks/README.md` for what it does and does not capture (#123).
- `docs/threat-models/compliance.md` (#129) and `docs/threat-models/registry.md` (#130), each expanding one risk the PRD or internal review named only briefly: stale KYC records, compliance admin compromise, and jurisdiction spoofing for `compliance`; fake/malicious entries, directory spam, delisting, and indexer guidance for `registry`. Linked from the external audit handover's prior-work section.
- `governance`: `cancel_proposal(caller, proposal_id)`, letting the proposer or the admin withdraw a mistaken proposal before it has received a single vote. Publishes `ProposalCancelled`. Restricted to pre-first-vote so a cancellation can never erase a vote a voter already committed (#137).
- `governance`: `list_proposals(start, limit)`, paging proposals in creation order, capped at `MAX_PAGE_SIZE` (100) per call, matching `registry::list_assets`'s pagination pattern (#138).
- `compliance`: `set_kyc_batch(records)` and `revoke_kyc_batch(subjects)`, each capped at `MAX_BATCH_SIZE` (50) per call. `set_kyc_batch` validates every record before writing any of them, so a batch lands in full or not at all; one `KycSet`/`KycRevoked` is published per record, same as the single-address entry points (#140).
- `oracle-adapter` contract, the first Phase 2 contract: a SEP-40 price feed for net asset values (ADR-005). It implements SEP-40's consumer interface (`base`, `assets`, `decimals`, `resolution`, `price`, `prices`, `lastprice`), so a consumer reads it the same way as any SEP-40 oracle.
  - The admin lists assets and authorizes reporters per asset.
  - History is append-only, one price per resolution tick.
  - A report may move the price at most the asset's `max_deviation_bps`. A larger move needs the admin's `override_price`, which publishes its own event.
  - Deployed in `scripts/deploy.sh` and exposed in the SDK as `OracleAdapterClient`.
- `VaultClient` in `@stellarforge-protocol/sdk` for interacting with the Phase 2 Vault contract (#63):
  - Typed read and write methods matching the vault contract interface (`config`, `underlyingHeld`, `balance`, `totalSupply`, `allowance`, `decimals`, `name`, `symbol`, `lockedUntil`, `redeemable`, `paused`, `admin`, `oracles`, `nav`, `navPerShare`, `balanceAt`, `totalSupplyAt`; `deposit`, `redeem`, `transfer`, `approve`, `transferFrom`, `setPaused`, `setOracles`).
  - Sponsored write methods (`buildSponsoredDepositTx`, `buildSponsoredRedeemTx`, `buildSponsoredTransferTx`, `buildSponsoredTransferFromTx`) with recursive multi-contract auth-tree verification (`checkInvocationTree`), detecting and preventing any tampering of root invocations, nested child invocations (such as underlying token transfers), contracts, function names, or arguments.
  - `describeAuthTree`: human-readable call tree string formatter for user consent and logging.
  - Pure rounding and lockup helpers (`sharesForDeposit`, `underlyingForRedeem`, `maxRedeemable`) verified against a 50+ test-case suite.
  - Typed event decoders (`decodeVaultEvent`, `decodeVaultEvents`) parsing raw RPC events and transaction results into strongly typed event union types (`deposit`, `redeem`, `nav_update`, `lock_period_set`, `emergency_paused`, `oracle_changed`, `admin_transfer`).
  - Structured error types (`VaultError`, `VaultErrorCode`, `NavRefusalError`).
- `OracleAdapterClient` in `@stellarforge-protocol/sdk` for the SEP-40 oracle-adapter contract (#62 SDK half):
  - Reads: `base`, `assets`, `decimals`, `resolution`, `price`, `prices`, `lastprice`, `admin`, `isReporter`, `assetConfig`.
  - Writes: `report`, `overridePrice`, `addAsset`, `setMaxDeviation`, `setReporter`, each as `build*Tx` plus a submitting form.
  - Sponsored: `buildSponsoredReportTx` and `buildSponsoredOverridePriceTx`.
  - Pure helpers: `tickOf`, `checkDeviation` (mirrors contract `InvalidDeviation` / `Overflow`), `isFresh`, `toDecimalString`.
  - `OracleAsset` ScVal codecs and `OracleErrorCode` discriminants 1–13.
- `docs/audits/external-audit-handover-phase1.md`, the handover for the Phase 1 external audit. It covers scope, how to rebuild the audited wasm, the trust model, the invariants to test, the deliberate design choices, and how the PRD relates to Phase 1.
- `docs/audits/2026-10-16-unbounded-vector-inventory.md` (#121), a Stage 2 Phase 1 hardening follow-up. Audits every `#[contracttype]` and `.set(...)` call site across all five contracts and `common` for the IR-01 pattern (an unbounded `Vec` growing inside one storage entry). Finds it does not recur: `oracle-adapter::DataKey::Assets` is the only other Vec-in-storage in the workspace, and it is capped at `MAX_ASSETS` on its one write path, already covered by a boundary test. No code change required to close this item.
- A "Trust Assumptions and Privileged Roles" section in `docs/SECURITY.md` (#127): a role matrix covering every privileged role across all five contracts (admin, issuer, holder, asset admin, proposer, voter, reporter), each row citing the actual entry points, the limits enforced in code, and the regression test proving each limit holds. Introduces no new behavior; gathers what was previously scattered across each contract's own doc comments and the ADRs into one place.
- Storage TTL regression tests for every persistent key family IR-07 did not already cover (#120). `rwa-asset` previously had TTL coverage only for `Metadata` (`update_metadata`) and the instance (`transfer_admin`), exactly IR-07's two fixes; `TotalSupply`, `Balance`, `Allowance` and `Issuer` had none, now covered by `test_mint_extends_balance_and_total_supply`, `test_transfer_extends_both_balances`, `test_burn_extends_balance_and_total_supply`, `test_approve_extends_the_allowance`, `test_transfer_from_extends_the_spent_allowance` and `test_set_issuer_extends_what_it_writes`. `oracle-adapter`'s `DataKey::Assets` had no TTL test at all, and `add_asset`/`set_reporter`/`set_max_deviation`'s own extensions of `Config`/`Reporter` were masked by `report`'s redundant re-extension of the same keys; both close with `test_add_asset_set_reporter_and_set_max_deviation_extend_what_they_write`. Each of the five contracts' `transfer_admin` also gains its own isolated instance-TTL test, closing the one call site every contract's existing coverage happened to exercise through a different function instead.
- `docs/threat-models/governance.md` and `docs/threat-models/oracle-adapter.md`, Stage 2 Phase 1 hardening follow-ups expanding the PRD's threat model into entry-point-level coverage for each contract: voting-weight manipulation, quorum, and admin-key risk for governance (#131); reporter-key compromise, stale prices, and the admin override's blast radius for the oracle adapter (#132).
- `contracts/{governance,oracle-adapter}/tests/test_resource_limits.rs` (#122): resource-limit regression tests for `governance.{propose,vote,finalize}` and `oracle-adapter.report`, run under `NetworkInvocationResourceLimits::mainnet()`. Each prints its CPU instruction count to the CI log and fails if it exceeds a committed baseline by more than 20%, so a flow that quietly grows past what mainnet allows is caught even before it crosses the limit outright (the internal review's own mainnet-budget proof-of-concept for IR-01/IR-02 was deleted after use and left no such tracking in place).
- `governance.set_proposal_deposit(token, amount)`, an optional admin-configured deposit `propose` collects from the proposer and `finalize` refunds in full, regardless of outcome (#134). Mitigates IR-14's remaining follow-up: proposal spam cost only fees and rent before this. Disabled by default (`token = None`); the token and amount a proposal paid travel with it, so refunds stay correct even if the configuration changes before that proposal closes. Publishes `DepositConfigured`, and `ProposalCreated`/`ProposalFinalized`'s sibling `ProposalDepositRefunded` carries the refunded amount.
- `docs/architecture/006-admin-key-recovery.md` (ADR-006): decides a guardian + timelock recovery path for a lost (not merely wrong) admin key, separate from the two-step handover below. Implementation tracked in #583.
- `governance.set_quorum(quorum)` and `governance.set_pass_threshold_bps(bps)`, admin-only (#135). `finalize` now rejects a proposal whose total votes (`votes_for + votes_against`) fall below the configured quorum (default 0, disabled) before evaluating whether `votes_for` clears the configured pass threshold in basis points (default 5,001, i.e. >50%, matching the prior unconditional majority rule). A proposal with zero votes is rejected unconditionally regardless of quorum.
- `governance.propose` rejects a second active proposal whose title and description hash both match one already active, with `DuplicateActiveProposal` (#136). The check clears once the earlier proposal is finalized, so an identical resubmission after a decision is a new, distinct proposal.
- `GovernanceClient` in the SDK gains `quorum`, `passThresholdBps` reads and `setQuorum`, `setPassThresholdBps` admin writes (#135).

### Changed

- **BREAKING:** `governance.propose`'s `ProposalCreated` event gains a `deposit_amount: i128` field (#134), and the `Proposal` type gains `deposit_token: Option<Address>` and `deposit_amount: i128`. A consumer decoding either by its prior shape must update. Both are `0`/`None` for every proposal created before a deposit is ever configured on a given deployment.
- **BREAKING:** `rwa-asset.update_metadata` refuses raising a supply cap, with `InvalidSupplyCap`. The admin could already neither remove a cap nor set it below supply. Now a cap can only be tightened, which makes PRD §10.2's "not bypassable by admin" true. An uncapped asset can still be given a cap at or above its supply.
- `docs/prd/PRODUCT_REQUIREMENTS.md` scopes three security claims to the phases that implement them. The over-minting mitigation now describes the cap rule (#54). Quorum, timelock and veto defences apply once governance can execute (Phase 3+). The only Phase 1 emergency control is `set_paused` on `rwa-asset`, since Phase 1 contracts cannot be upgraded.
- **BREAKING:** `transfer_admin(new_admin)` is replaced by a two-step `propose_admin(new_admin)` / `accept_admin()` handover on all five contracts (`rwa-asset`, `compliance`, `registry`, `governance`, `oracle-adapter`), plus `cancel_admin_transfer()` and a `pending_admin()` query (#125, see ADR-006). ADR-002 originally rejected a two-step handover because the SDK could not build a transaction with two distinct signers (#31); #31 is closed, and this both makes handover reachable from the SDK (previously `stellar-cli`-only) and removes the need for sponsorship on this path entirely, since each step now needs only one signature. The SDK's `buildTransferAdminTx`/`transferAdmin` are replaced by `buildProposeAdminTx`/`proposeAdmin`, `buildAcceptAdminTx`/`acceptAdmin`, `buildCancelAdminTransferTx`/`cancelAdminTransfer`, and a `pendingAdmin` read, shared across every client.
- `governance`'s `Proposal` struct gains a `content_hash` field (the sha256 of its title and description hash), used by the duplicate-proposal check above.

### Fixed

- **BREAKING:** `rwa-asset.set_compliance` refuses a `min_level` above `MAX_COMPLIANCE_LEVEL` (3), with `InvalidComplianceLevel` (#139). A compliance record's own `level` is already capped at the same scale in `ComplianceContract::set_kyc`, so a higher `min_level` could never be satisfied by any verified party: every transfer would silently and permanently fail `NotCompliant` until an admin reconfigured it. A caller already passing 0-3 is unaffected.

## [0.3.0] - 2026-09-14

Adds operator methods and sponsored writes to the SDK. Nothing is removed or changed incompatibly from 0.2.0, and the contract interfaces are unchanged.

### Added

- Operator methods in the SDK. `RwaAssetClient` gains `admin`, `complianceContract` and `minComplianceLevel` reads, and `setIssuer`, `setPaused`, `updateMetadata` and `setCompliance` admin writes (`setCompliance(admin, null, level)` switches screening off). `ComplianceClient` gains `getKyc` and `admin` reads, and `setKyc` and `revokeKyc` admin writes. Each write has a `build*Tx` and a submitting form.
- Sponsored writes (#31): one account sources and pays for a write that another address authorizes.
  - `RwaAssetClient` gains `buildSponsoredMintTx`, `buildSponsoredBurnTx`, `buildSponsoredBurnFromTx`, `buildSponsoredTransferTx`, `buildSponsoredApproveTx` and `buildSponsoredTransferFromTx`.
  - Every client gains `finalizeSponsoredTx`, `submitSponsoredTx` and `buildTransferAdminTx`, which makes `transfer_admin` reachable from the SDK.
  - `authorizeEntries` signs an authorizer's entries and refuses a key they don't name. `authEntriesToXdr` and `authEntriesFromXdr` move entries between machines.
  - Finalizing checks each signed entry against the one built, and requires its expiry to fall between `MIN_AUTH_REMAINING_LEDGERS` (40) and `MAX_AUTH_VALIDITY_LEDGERS` (17,280) ledgers ahead.
  - The write smoke test runs a sponsored transfer, and with `SMOKE_WRITE_TRANSFER_ADMIN`, a sponsored admin handover.
- The write smoke test can configure its own deployment through the operator methods (`SMOKE_WRITE_SETUP`). It runs `setIssuer`, `setKyc`, `revokeKyc`, `setCompliance`, `setPaused` and `updateMetadata` on testnet, checking each through its read. The Testnet Smoke workflow now sets up this way instead of with `stellar-cli`.

## [0.2.0] - 2026-09-14

Breaking for SDK users on Node.js older than 22.12, for anyone relying on `MAINNET_CONFIG` to supply an RPC URL, and for code that handles the `@stellar/stellar-sdk` objects the SDK returns. The contract interfaces are unchanged from 0.1.0.

### Added

- `sdk/tests/smoke.write.testnet.test.ts`, a live write-path smoke test. It runs `mint`, `transfer` (including to a muxed address, checking the event's `to_muxed_id`), `approve`, `transferFrom`, `burnFrom` and `burn` against a deployed `rwa-asset`, asserting before-and-after deltas, and checks that screening refuses an unverified recipient. It has its own opt-in variables and refuses any passphrase but testnet's. The Testnet Smoke workflow now runs it against a fresh deployment with compliance switched on.

### Changed

- **BREAKING:** the SDK depends on `@stellar/stellar-sdk` 17 (Protocol 28), up from 16, and so requires Node.js 22.12 or newer. Transactions returned by `build*Tx` are version 17 objects. `toXdr()` is their current spelling; `toXDR()` remains as a deprecated alias. The SDK's own API is unchanged. The read and write testnet smoke tests pass against this version.

### Fixed

- **BREAKING:** `MAINNET_CONFIG` no longer includes an `rpcUrl`. In 0.1.0 it named `https://soroban-rpc.stellar.org`, which does not resolve, so every mainnet call made with the preset failed. SDF runs no public mainnet RPC, so there is no URL to default to. Callers now supply `rpcUrl` from an [RPC provider](https://developers.stellar.org/docs/data/apis/rpc/providers). TypeScript rejects a config without one, and the clients throw a message naming where to find one. `TESTNET_CONFIG` is unchanged.

### Security

- The SDK's build tooling now uses esbuild 0.28 through an npm override, which clears [GHSA-g7r4-m6w7-qqqr](https://github.com/advisories/GHSA-g7r4-m6w7-qqqr). tsup 8.5.1, the newest release, still requires esbuild `^0.27`. esbuild is a development dependency only, and the built `dist` output is byte-for-byte identical under both versions, so the published package is unaffected.

## [0.1.0] - 2026-09-14

First release. The SDK is published to npm as `@stellarforge-protocol/sdk@0.1.0` and tagged `v0.1.0`. The contracts are not deployed to mainnet.

### Added

- `stellarforge-common` crate centralising the shared storage/TTL policy across all contracts.
- Event emission (`Transfer`, `Mint`, `Burn`, `Approve`, `Paused`) for every balance-changing `rwa-asset` operation, with the wire format pinned in tests.
- Compliance screening on `rwa-asset` transfers via a pluggable `ComplianceInterface` (KYC/AML transfer guard).
- `allowance` / `is_issuer` getters and read-only `RwaAssetClient` / `ComplianceClient` in the TypeScript SDK.
- Write-path methods on `RwaAssetClient` for `mint`, `burn`, `transfer`, `approve` and `transfer_from`. Each has a `build*Tx` form returning a prepared unsigned transaction for a wallet to sign, and a bare form that signs with `signerSecret` and submits. Both require the authorizing address to source the transaction; fee payment from a separate account is not supported.
- `TxResult`, the hash and ledger of a submitted write transaction.
- `RegistryClient` and `GovernanceClient`, covering every entry point on the registry and governance contracts. Reads and writes follow the same patterns as `RwaAssetClient`, including the build-or-submit split.
- `AssetEntry`, `Proposal`, `ProposalStatus` and `ProposalCreated` types.
- `.github/workflows/publish-sdk.yml`, publishing `@stellarforge-protocol/sdk` on a published release or by manual dispatch. The manual path defaults to a dry run, and a release publish refuses if `package.json` disagrees with the tag.
- `sdk/README.md`, which is what the npm package page renders.
- `docs/architecture/002-auth-patterns.md`, `003-upgrade-path.md` and `004-cross-contract-interaction.md`, the three ADRs `CONTRIBUTING.md` has called for since Phase 1 opened. They record the authorization map across all four contracts, why Phase 1 ships no upgrade entry point and what that costs, and the `ComplianceInterface` binding including the `screen` / `is_compliant` split.
- `.github/workflows/testnet-smoke.yml`, a manually dispatched job that deploys all four contracts to testnet with a throwaway friendbot-funded key, initializes them, and runs the live smoke test against them. It is the only thing in CI that exercises the wire format; `sdk-ci` stubs `simulateTransaction` and so passes identically whatever changed underneath.
- Unit tests for `RwaAssetClient` and `ComplianceClient`, stubbing Soroban RPC at `rpc.Server.prototype.simulateTransaction` so the contract call, the ScVal codecs and both failure paths are exercised for real.
- `docs/audits/2026-09-14-internal-review-phase1.md`, the Phase 1 internal security review of all four contracts, the SDK and the deployment/CI tooling. All 17 findings are fixed.
- `transfer_admin` on `compliance`, `registry` and `governance`. As on `rwa-asset`, the current and the incoming admin must both authorize in one transaction (NFR-S-4). Previously the admin named at deployment could never be rotated out.
- Events for every privileged and governance state change. `rwa-asset`: `IssuerSet`, `MetadataUpdated`, `AdminTransferred`, `ComplianceSet`. `compliance`: `KycSet`, `KycRevoked`, `AdminTransferred`. `registry`: `AssetRegistered`, `ActiveSet`, `AdminTransferred`. `governance`: `ProposalCreated`, `VoteCast`, `ProposalFinalized`, `AdminTransferred`. Each topic is the entry point name, and each wire format is pinned by a test.
- `registry.asset_count()`, plus `RegistryClient.assetCount()` and `RegistryClient.listAllAssets()`, which walks every page.
- `rwa-asset.burn_from` and the SEP-41 metadata getters `decimals`, `name` and `symbol`, plus `RwaAssetClient.burnFrom` and `buildBurnFromTx`.
- `tests/test_constructor_auth.rs` in each contract, asserting that the constructor demands the admin's authorization.
- `contracts-ci` publishes `SHA256SUMS` with the wasm artifacts, and `security.yml` audits the SDK's runtime npm dependencies.
- `TransactionExpiredError` and `TransactionOutcomeUnknownError`, which a bare write method throws when its transaction was accepted but not seen to settle. The first means a retry is safe. The second means the transaction may still land, and carries the hash to check before retrying.
- `sdk/tests/smoke.testnet.test.ts`, an opt-in live check of both clients against a deployed contract over real Soroban RPC. It asserts contract invariants rather than fixed values, and skips unless `SMOKE_RWA_ASSET_ID` or `SMOKE_COMPLIANCE_ID` names a contract, so CI never runs it.

### Changed

- The SDK package is `@stellarforge-protocol/sdk`. The `stellarforge` npm organization belongs to someone else, and nothing was ever published under the old name.
- `publish-sdk.yml` authenticates with npm trusted publishing (OIDC) instead of an `NPM_TOKEN` secret. npm only accepts a publish from this workflow, in this repository, in the `npm-publish` environment, and attaches provenance automatically. `docs/RELEASING.md` covers the one-time setup and the release steps.
- **BREAKING:** `registry.list_assets` takes `(start, limit)` and returns at most 100 addresses per call. A larger `limit` fails with `PageTooLarge` rather than being silently truncated. `RegistryClient.listAssets(start, limit)` defaults to the first full page. `RegistryError` appends `PageTooLarge = 4` and `Overflow = 5`.
- `rwa-asset.update_metadata` refuses to change `decimals` (`DecimalsImmutable = 12`), and refuses to set a cap below circulating supply or lift it to uncapped (`InvalidSupplyCap = 13`). Raising a cap is still allowed.
- **BREAKING:** `governance.propose` requires a voting period of `MIN_VOTING_PERIOD_LEDGERS` to `MAX_VOTING_PERIOD_LEDGERS` (about 1 to 90 days) and a title of at most `MAX_TITLE_BYTES` (256). `GovernanceError` appends `InvalidVotingPeriod = 10` and `TitleTooLong = 11`.
- **BREAKING:** `rwa-asset.approve` takes `live_until_ledger`, and an allowance reads as zero after it (`InvalidExpiration = 14`). `transfer` takes `to` as a `MuxedAddress`, and its `Transfer` event data is `{amount, to_muxed_id}`. `transfer_from` publishes `TransferFrom`, whose data is the amount, and publishes nothing on a self-transfer. `Approve` event data is `[amount, live_until_ledger]`. `RwaAssetClient.approve` and `buildApproveTx` take `liveUntilLedger`.
- **BREAKING:** `compliance.set_kyc` rejects a level above 3, an expiry already past, and a jurisdiction that is not two uppercase ASCII letters. `ComplianceError` appends `InvalidLevel = 3`, `AlreadyExpired = 4` and `InvalidJurisdiction = 5`.
- The Rust toolchain is pinned to 1.98.1, and CI installs exactly that through rustup. `Swatinem/rust-cache` is pinned to a commit, the `stellar-cli` download is checked against a pinned sha256, `security.yml` also runs on pull requests, and `publish-sdk` runs in the `npm-publish` environment.
- `make deploy-testnet` runs `scripts/deploy.sh`, which deploys all four contracts. `deploy.sh` refuses `NETWORK=mainnet` unless `ADMIN`, every `ASSET_*` variable and a real document hash are set explicitly.
- Bare write methods poll for the transaction's whole validity window plus 30 seconds, instead of the stellar-sdk default of about 30 seconds. `TRY_AGAIN_LATER` now fails immediately without polling, and `DUPLICATE` waits for the queued transaction.
- **BREAKING:** `toStroops` throws on anything but plain decimal notation, on digits beyond `decimals` (except zeros), and on a `number` that is not a safe integer. It used to truncate or guess. `fromStroops` and `toStroops` require `decimals` from 0 to 18, and `hexToBytes32` requires exactly 64 hex digits.
- **BREAKING:** all four contracts configure themselves in a `__constructor` and no longer expose `initialize`. Deployment and configuration are now one transaction, closing the window in which a deployed contract had no admin and anyone could name themselves. `stellar contract deploy` takes the arguments after `--`; `scripts/deploy.sh` does this for all four and so now initializes what it deploys, which it previously did not.
- `AlreadyInitialized` and `NotInitialized` are unreachable but retained in every error enum, marked reserved. ADR-003 freezes discriminants, so deleting a variant and letting later ones shift up would silently change what a deployed client believes went wrong.
- `docs/architecture/002-auth-patterns.md` carries amendments recording the change. The first noted that constructor `require_auth` was asserted by no test, because `Env::register` authorizes the constructor itself. A later one records the `tests/test_constructor_auth.rs` coverage that closed that gap.
- `sdk/package.json` declares `publishConfig.access: public`. A scoped package defaults to restricted, so the first publish would otherwise have failed or gone private. A `prepack` script copies the repo LICENSE into the package, since npm ships those only from the package root.
- The four clients now share one `ContractClient` base rather than each carrying its own copy of the RPC server, contract handle, simulation helper and write path.
- `npm run typecheck` now covers `tests/` as well as `src/`, via a `tsconfig.test.json` that relaxes the `rootDir` the published build depends on. Test files were previously transpiled by vitest but never typechecked, so type errors in them reached no CI gate.
- The SDK now requires Node.js 22 or newer, and CI tests against Node 22 and 24 instead of 20 and 22. `@stellar/stellar-sdk` has required Node 22 since 16.0.0, so the previous `>=20.0.0` in `package.json` did not reflect what the package actually needed.

### Fixed

- `scripts/deploy.sh` is now executable. It was committed `100644`, so `./scripts/deploy.sh` — the invocation its own usage comment documents — failed with `Permission denied`.
- `stellar keys generate --global` in `README.md` and `CONTRIBUTING.md`. stellar-cli 27 removed the flag, so the documented setup step failed outright on a current CLI.
- SF-2026-001: a self-transfer wrote debit and credit to the same storage key and minted tokens out of nothing (critical).
- SF-2026-002: storage TTL was never extended, risking archival of live balances and instance state (high).
- SF-2026-003: the registry directory stopped accepting assets at about 1,600 entries (medium).
- SF-2026-004: `update_metadata` could re-denominate every balance or lift the supply cap (medium).
- SF-2026-005: privileged and governance state changes emitted no events (medium).
- SF-2026-006: `compliance`, `registry` and `governance` could not rotate their admin (medium).
- SF-2026-007: `update_metadata` and `transfer_admin` did not extend the storage they wrote (low).
- SF-2026-008: a bare SDK write could be reported as failed while its transaction was still able to land, inviting a double-executing retry (medium).
- SF-2026-009: SDK amount and hash helpers silently misread malformed input, and `fromStroops` misrendered zero-decimal and negative amounts (medium).
- SF-2026-010 to SF-2026-019: the Low and Info findings of the internal review, IR-08 to IR-17. `docs/SECURITY.md` lists each with its fix.
- `registry.register` no longer duplicates an asset in `list_assets` when an already-registered asset is re-registered.
- The SDK's `package.json` exports map listed `types` after `import` and `require`, so no resolver used it. Each condition now names its own declaration file first, and `repository.url` is in the form npm expects.
- `ComplianceClient.isCompliant` asserted the simulation result was present and threw `Cannot read properties of undefined` when it was not. It now reports the missing result the same way `RwaAssetClient` does.

### Security

- Restored the canonical Apache-2.0 license text.
- Adopted GitHub private vulnerability reporting (see `docs/SECURITY.md`).
