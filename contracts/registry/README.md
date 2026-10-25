# registry

A protocol-wide directory of deployed `rwa-asset` contracts: one `AssetEntry`
(contract address, asset class, active flag) per registration, paged in
registration order.

See the root [README](../../README.md#registry) for the full project
context, and [`docs/threat-models/registry.md`](../../docs/threat-models/registry.md)
for what this contract does and does not guarantee about an entry.

**An entry is an assertion, not a verification.** `register` does not check
that the address is a deployed `rwa-asset` or that `asset_class` matches its
metadata, and nothing in the protocol reads `active`. Treat the registry as
the admin's directory, not as proof that an asset is genuine or current.

## Entry points

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin)` | admin | Runs at deploy; not callable afterwards |
| `register(entry)` | admin | Register a new asset contract, or update a registered one |
| `set_active(contract, active)` | admin | Activate/deactivate an entry |
| `set_paused(paused)` | admin | Emergency circuit breaker; blocks `register`/`set_active` |
| `propose_admin(new_admin)` / `accept_admin()` / `cancel_admin_transfer()` | admin / new_admin / admin | Two-step admin handover |
| `get_asset(contract)` | — | Look up one entry |
| `list_assets(start, limit)` | — | Page through registered contracts in registration order, at most `MAX_PAGE_SIZE` (100) per call, each exactly once |
| `asset_count()` | — | Count registered contracts |
| `admin()`, `pending_admin()`, `paused()` | — | Read-only queries |

## Errors (`RegistryError`)

| Code | Variant | Raised by |
|---|---|---|
| 1 | `AlreadyInitialized` | Reserved; unreachable since `initialize` was replaced by the constructor |
| 2 | `NotInitialized` | Any getter, if ever called before construction |
| 3 | `AssetNotFound` | Lookups against an address that was never registered |
| 4 | `PageTooLarge` | `list_assets`, over `MAX_PAGE_SIZE` |
| 5 | `Overflow` | The directory index, in the unreachable case it would overflow `u32` |
| 6 | `ContractPaused` | `register`/`set_active`, while paused |
| — | `NoPendingAdminTransfer` | `accept_admin`/`cancel_admin_transfer`, with nothing proposed |

Discriminants are frozen once a contract using them is deployed (ADR-003):
existing values never change meaning, and new ones are only ever appended.

## Events

| Event | Published by |
|---|---|
| `AssetRegistered` | `register` |
| `ActiveSet` | `set_active` |
| `Paused` | `set_paused` |
| `AdminTransferProposed`, `AdminTransferred`, `AdminTransferCancelled` | `propose_admin`, `accept_admin`, `cancel_admin_transfer` |
