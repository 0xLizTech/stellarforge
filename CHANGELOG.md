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
- Unit tests for `RwaAssetClient` and `ComplianceClient`, stubbing Soroban RPC at `rpc.Server.prototype.simulateTransaction` so the contract call, the ScVal codecs and both failure paths are exercised for real.

### Changed

- `npm run typecheck` now covers `tests/` as well as `src/`, via a `tsconfig.test.json` that relaxes the `rootDir` the published build depends on. Test files were previously transpiled by vitest but never typechecked, so type errors in them reached no CI gate.
- The SDK now requires Node.js 22 or newer, and CI tests against Node 22 and 24 instead of 20 and 22. `@stellar/stellar-sdk` has required Node 22 since 16.0.0, so the previous `>=20.0.0` in `package.json` did not reflect what the package actually needed.

### Fixed

- SF-2026-001: a self-transfer wrote debit and credit to the same storage key and minted tokens out of nothing (critical).
- SF-2026-002: storage TTL was never extended, risking archival of live balances and instance state (high).
- `registry.register` no longer duplicates an asset in `list_assets` when an already-registered asset is re-registered.
- `ComplianceClient.isCompliant` asserted the simulation result was present and threw `Cannot read properties of undefined` when it was not. It now reports the missing result the same way `RwaAssetClient` does.

### Security

- Restored the canonical Apache-2.0 license text.
- Adopted GitHub private vulnerability reporting (see `docs/SECURITY.md`).
