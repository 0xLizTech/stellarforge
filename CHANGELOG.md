# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `stellarforge-common` crate centralising the shared storage/TTL policy across all contracts.
- Event emission (`Transfer`, `Mint`, `Burn`, `Approve`, `Paused`) for every balance-changing `rwa-asset` operation, with the wire format pinned in tests.
- Compliance screening on `rwa-asset` transfers via a pluggable `ComplianceInterface` (KYC/AML transfer guard).
- `allowance` / `is_issuer` getters and read-only `RwaAssetClient` / `ComplianceClient` in the TypeScript SDK.
- Write-path methods on `RwaAssetClient` for `mint`, `burn`, `transfer`, `approve` and `transfer_from`. Each has a `build*Tx` form returning a prepared unsigned transaction for a wallet to sign, and a bare form that signs with `signerSecret` and submits. Both require the authorizing address to source the transaction; fee payment from a separate account is not supported.
- `TxResult`, the hash and ledger of a submitted write transaction.
- `RegistryClient` and `GovernanceClient`, covering every entry point on the registry and governance contracts. Reads and writes follow the same patterns as `RwaAssetClient`, including the build-or-submit split.
- `AssetEntry`, `Proposal`, `ProposalStatus` and `ProposalCreated` types.
- `.github/workflows/publish-sdk.yml`, publishing `@stellarforge/sdk` on a published release or by manual dispatch. The manual path defaults to a dry run, and a release publish refuses if `package.json` disagrees with the tag.
- `sdk/README.md`, which is what the npm package page renders.
- `docs/architecture/002-auth-patterns.md`, `003-upgrade-path.md` and `004-cross-contract-interaction.md`, the three ADRs `CONTRIBUTING.md` has called for since Phase 1 opened. They record the authorization map across all four contracts, why Phase 1 ships no upgrade entry point and what that costs, and the `ComplianceInterface` binding including the `screen` / `is_compliant` split.
- `.github/workflows/testnet-smoke.yml`, a manually dispatched job that deploys all four contracts to testnet with a throwaway friendbot-funded key, initializes them, and runs the live smoke test against them. It is the only thing in CI that exercises the wire format; `sdk-ci` stubs `simulateTransaction` and so passes identically whatever changed underneath.
- Unit tests for `RwaAssetClient` and `ComplianceClient`, stubbing Soroban RPC at `rpc.Server.prototype.simulateTransaction` so the contract call, the ScVal codecs and both failure paths are exercised for real.
- `sdk/tests/smoke.testnet.test.ts`, an opt-in live check of both clients against a deployed contract over real Soroban RPC. It asserts contract invariants rather than fixed values, and skips unless `SMOKE_RWA_ASSET_ID` or `SMOKE_COMPLIANCE_ID` names a contract, so CI never runs it.

### Changed

- **BREAKING:** all four contracts configure themselves in a `__constructor` and no longer expose `initialize`. Deployment and configuration are now one transaction, closing the window in which a deployed contract had no admin and anyone could name themselves. `stellar contract deploy` takes the arguments after `--`; `scripts/deploy.sh` does this for all four and so now initializes what it deploys, which it previously did not.
- `AlreadyInitialized` and `NotInitialized` are unreachable but retained in every error enum, marked reserved. ADR-003 freezes discriminants, so deleting a variant and letting later ones shift up would silently change what a deployed client believes went wrong.
- `docs/architecture/002-auth-patterns.md` carries an amendment recording the change, including that constructor `require_auth` is asserted by no test: `Env::register` mocks authorization, and the SDK documents that it cannot be used to test it. The same call was equally unasserted on `initialize`, so nothing regressed.
- `sdk/package.json` declares `publishConfig.access: public`. A scoped package defaults to restricted, so the first publish would otherwise have failed or gone private. A `prepack` script copies the repo LICENSE into the package, since npm ships those only from the package root.
- The four clients now share one `ContractClient` base rather than each carrying its own copy of the RPC server, contract handle, simulation helper and write path.
- `npm run typecheck` now covers `tests/` as well as `src/`, via a `tsconfig.test.json` that relaxes the `rootDir` the published build depends on. Test files were previously transpiled by vitest but never typechecked, so type errors in them reached no CI gate.
- The SDK now requires Node.js 22 or newer, and CI tests against Node 22 and 24 instead of 20 and 22. `@stellar/stellar-sdk` has required Node 22 since 16.0.0, so the previous `>=20.0.0` in `package.json` did not reflect what the package actually needed.

### Fixed

- `scripts/deploy.sh` is now executable. It was committed `100644`, so `./scripts/deploy.sh` — the invocation its own usage comment documents — failed with `Permission denied`.
- `stellar keys generate --global` in `README.md` and `CONTRIBUTING.md`. stellar-cli 27 removed the flag, so the documented setup step failed outright on a current CLI.
- SF-2026-001: a self-transfer wrote debit and credit to the same storage key and minted tokens out of nothing (critical).
- SF-2026-002: storage TTL was never extended, risking archival of live balances and instance state (high).
- `registry.register` no longer duplicates an asset in `list_assets` when an already-registered asset is re-registered.
- `ComplianceClient.isCompliant` asserted the simulation result was present and threw `Cannot read properties of undefined` when it was not. It now reports the missing result the same way `RwaAssetClient` does.

### Security

- Restored the canonical Apache-2.0 license text.
- Adopted GitHub private vulnerability reporting (see `docs/SECURITY.md`).
