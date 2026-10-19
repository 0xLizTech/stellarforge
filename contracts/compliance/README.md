# compliance

A KYC/AML status registry. Holds one `KycRecord` per address — a
jurisdiction, a verification level (0 none, 1 basic, 2 full, 3 accredited),
and an optional expiry — and screens counterparties for `rwa-asset`, or any
other contract implementing the same minimal interface.

See the root [README](../../README.md#compliance) for the full project
context, and [ADR-001](../../docs/architecture/001-storage-key-design.md)
and [ADR-004](../../docs/architecture/004-cross-contract-interaction.md) for
why `rwa-asset` binds to this contract the way it does.

## Entry points

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin)` | admin | Runs at deploy; not callable afterwards |
| `set_kyc(subject, record)` | admin | Set a KYC record; requires level 0–3 (`MAX_LEVEL`), an expiry not yet past, and a two-letter ISO 3166-1 jurisdiction |
| `set_kyc_batch(records)` | admin | Up to `MAX_BATCH_SIZE` (50) records; validates every record before writing any of them |
| `revoke_kyc(subject)` | admin | Remove a KYC record |
| `revoke_kyc_batch(subjects)` | admin | Up to `MAX_BATCH_SIZE` subjects |
| `set_paused(paused)` | admin | Emergency circuit breaker; blocks `set_kyc`/`revoke_kyc` and their batch forms |
| `propose_admin(new_admin)` / `accept_admin()` / `cancel_admin_transfer()` | admin / new_admin / admin | Two-step admin handover |
| `is_compliant(subject, min_level)` | — | Pure query: does `subject` hold a valid, unexpired record at or above `min_level`? |
| `screen(subject, min_level)` | — | Same answer as `is_compliant`, but also refreshes the record's TTL; this is what a bound contract like `rwa-asset` calls |
| `get_kyc(subject)` | — | Read the raw record |
| `admin()`, `pending_admin()`, `paused()` | — | Read-only queries |

`screen` is the entry point a compliance-screened asset must call, not
`is_compliant`: a KYC record is written once and thereafter only read, and
only this contract can extend its own entries' TTL — `screen` is where that
happens.

## Errors (`ComplianceError`)

| Code | Variant | Raised by |
|---|---|---|
| 1 | `AlreadyInitialized` | Reserved; unreachable since `initialize` was replaced by the constructor |
| 2 | `NotInitialized` | Any getter, if ever called before construction |
| 3 | `InvalidLevel` | `set_kyc`/batch, on a level above `MAX_LEVEL` |
| 4 | `AlreadyExpired` | `set_kyc`/batch, on an `expires_at` already in the past |
| 5 | `InvalidJurisdiction` | `set_kyc`/batch, on a jurisdiction that is not a two-letter ISO 3166-1 alpha-2 code |
| 6 | `ContractPaused` | `set_kyc`/`revoke_kyc`/batch forms, while paused |
| 7 | `BatchTooLarge` | `set_kyc_batch`/`revoke_kyc_batch`, over `MAX_BATCH_SIZE` |
| 8 | `NoPendingAdminTransfer` | `accept_admin`/`cancel_admin_transfer`, with nothing proposed |

Discriminants are frozen once a contract using them is deployed (ADR-003):
existing values never change meaning, and new ones are only ever appended.

## Events

| Event | Published by |
|---|---|
| `KycSet` | `set_kyc`, once per record (including each record in a batch) |
| `KycRevoked` | `revoke_kyc`, carrying the record that was removed (published only when one existed) |
| `Paused` | `set_paused` |
| `AdminTransferProposed`, `AdminTransferred`, `AdminTransferCancelled` | `propose_admin`, `accept_admin`, `cancel_admin_transfer` |
