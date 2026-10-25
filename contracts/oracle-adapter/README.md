# oracle-adapter

A SEP-40 price feed for net asset values. Public Stellar oracles publish
market prices for liquid assets; nobody publishes the appraised value of a
building or a loan book, so a StellarForge asset needs a feed its own
appraisers can report to. The read side is SEP-40's consumer interface byte
for byte, so anything that reads a SEP-40 oracle reads this one.

See the root [README](../../README.md#oracle-adapter-phase-2) for the full
project context, [ADR-005](../../docs/architecture/005-oracle-adapter.md)
for why this shape was chosen, and
[`docs/threat-models/oracle-adapter.md`](../../docs/threat-models/oracle-adapter.md)
for the risks a deployer should weigh (reporter-key compromise, stale
prices, the blast radius of `override_price`).

Prices are scaled by `10^decimals()` and timestamped in Unix seconds,
rounded down to `resolution()`. History is append-only: a report must land
in a later resolution tick than the latest one. A report may move the price
at most `max_deviation_bps` from the latest; a larger move needs the admin's
`override_price`.

## Entry points

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin, base, decimals, resolution)` | admin | Runs at deploy; not callable afterwards |
| `report(reporter, asset, price, timestamp)` | reporter | Record a price; reporter must be authorized for the asset, the tick must be later than the latest, and the move must be within the asset's deviation limit |
| `override_price(asset, price, timestamp)` | admin | Record a price beyond the deviation limit; every other rule still applies |
| `add_asset(asset, max_deviation_bps)` | admin | List an asset, up to `MAX_ASSETS` (100) |
| `set_max_deviation(asset, max_deviation_bps)` | admin | Change an asset's deviation limit |
| `set_reporter(asset, reporter, allowed)` | admin | Grant or revoke a reporter for one asset |
| `set_paused(paused)` | admin | Emergency circuit breaker; blocks `report`/`override_price`/`add_asset`/`set_max_deviation`/`set_reporter` |
| `propose_admin(new_admin)` / `accept_admin()` / `cancel_admin_transfer()` | admin / new_admin / admin | Two-step admin handover |
| `lastprice(asset)`, `price(asset, timestamp)`, `prices(asset, records)` | — | SEP-40 price reads; `prices` returns up to `MAX_RECORDS` (20), newest first |
| `base()`, `assets()`, `decimals()`, `resolution()` | — | SEP-40 feed configuration |
| `is_reporter(asset, reporter)`, `asset_config(asset)`, `admin()`, `pending_admin()`, `paused()` | — | Read-only queries |

## Errors (`OracleError`)

| Code | Variant | Raised by |
|---|---|---|
| 1 | `NotInitialized` | Any getter, if ever called before construction |
| 2 | `InvalidDecimals` | Constructor, over `MAX_DECIMALS` |
| 3 | `InvalidResolution` | Constructor, on a resolution of zero seconds |
| 4 | `UnknownAsset` | Any call naming an asset `add_asset` has not listed |
| 5 | `AssetAlreadyExists` | `add_asset`, on an asset already listed |
| 6 | `TooManyAssets` | `add_asset`, past `MAX_ASSETS` |
| 7 | `NotReporter` | `report`, when the caller is not authorized for the asset |
| 8 | `InvalidPrice` | `report`/`override_price`, on a zero or negative price |
| 9 | `TimestampInFuture` | `report`/`override_price`, past the ledger's close time |
| 10 | `StaleReport` | `report`/`override_price`, not later than the latest recorded tick |
| 11 | `DeviationTooLarge` | `report`, moving further than `max_deviation_bps` allows |
| 12 | `InvalidDeviation` | `add_asset`/`set_max_deviation`, on a deviation of zero basis points |
| 13 | `Overflow` | Any checked arithmetic operation overflowing |
| 14 | `ContractPaused` | Any state-changing call while paused |
| — | `NoPendingAdminTransfer` | `accept_admin`/`cancel_admin_transfer`, with nothing proposed |

Discriminants are frozen once a contract using them is deployed (ADR-003).
`NoPendingAdminTransfer`'s code is omitted above because, as of this
writing, `contracts/oracle-adapter/src/error.rs` assigns it the same
discriminant as `ContractPaused` (a pre-existing defect tracked separately
from #151, outside this document's scope to fix).

## Events

| Event | Published by |
|---|---|
| `AssetAdded` | `add_asset` |
| `MaxDeviationSet` | `set_max_deviation` |
| `ReporterSet` | `set_reporter` |
| `PriceReported` | `report` |
| `PriceOverridden` | `override_price` |
| `Paused` | `set_paused` |
| `AdminTransferProposed`, `AdminTransferred`, `AdminTransferCancelled` | `propose_admin`, `accept_admin`, `cancel_admin_transfer` |
