# ADR-005: Oracle Adapter

**Status:** Accepted
**Date:** 2026-09-14
**Authors:** StellarForge Core Team

---

## Context

The Phase 2 vault prices fractional shares from the underlying asset's net asset value. FR-06-4 requires the NAV to be "updateable via an oracle", and the vault must "reject oracle prices older than a configurable max staleness". The roadmap asks for a "standardized interface for price/NAV feeds (Pyth, Band, custom)".

The PRD's threat model names an oracle manipulator who "attempts to fake NAV to trigger unjust redemptions". Its mitigations (§10.2) are a staleness check, multi-source aggregation and a circuit breaker on NAV deviation. §17 adds a manual override for when the feed locks up.

The main fact behind this decision: **no public oracle publishes the NAV of a real-world asset.** Reflector, Band and Pyth publish market prices for liquid assets, such as XLM/USD, BTC or EUR. A tokenized apartment block or loan book has a value set by an appraiser or fund administrator, on a schedule measured in weeks or quarters. Its NAV has to be reported by the party responsible for it. No public feed carries it.

## Decision

### SEP-40 is the interface

The adapter implements SEP-40's consumer interface exactly: `base`, `assets`, `decimals`, `resolution`, `price`, `prices` and `lastprice`, with SEP-40's `Asset` and `PriceData` types.

This makes SEP-40 the "standardized interface". The vault will bind to SEP-40 through a `#[contractclient]` trait, as `rwa-asset` binds to `screen` in ADR-004, and not to this crate. A vault can then read:

- this adapter, for appraised NAVs;
- Reflector, which is natively SEP-40, for liquid assets;
- any other SEP-40 contract, including a future aggregator or a wrapper around a non-SEP-40 oracle.

None of those choices changes the vault.

### A push feed, with reporters authorized per asset

The admin lists assets (`add_asset`, at most `MAX_ASSETS` = 100) and grants reporters (`set_reporter`). A grant covers one asset, so an appraiser for one property cannot move another's NAV. The reporter signs each `report`.

### History is append-only, one price per tick

A report's timestamp is rounded down to the resolution, as SEP-40 specifies, and must fall in a later tick than the latest price. A recorded price can never be changed afterwards, by a reporter or the admin. An auditor reconstructing what a vault saw at a given ledger can rely on it.

Each stored report links to the one before it. `prices` therefore walks back through reports rather than ticks. A NAV reported quarterly at a one-hour resolution has thousands of empty ticks between reports, and counting those would make `prices` useless. `price(asset, timestamp)` is an exact tick lookup and returns `None` for a tick nobody reported in, as SEP-40 requires.

### A per-report deviation limit, and an admin override

Each listed asset has a `max_deviation_bps`. A report further than that from the latest price is refused with `DeviationTooLarge`. This is the §10.2 circuit breaker, and it bounds what a single compromised reporter key can do in one report.

A genuine revaluation can exceed any limit that is still useful. Without an escape hatch the feed would be stuck at a price everyone knows is wrong, which is the §17 lockout risk. `override_price` lets the admin record such a price. It skips only the deviation check. Positivity, the future-timestamp check and append-only history still apply. It publishes `override_price`, a separate event from `report`, so a monitor can alert on every override with one topic filter.

A limit of 0 is refused rather than read as "unlimited". Disabling the breaker has to be written out as a very large value.

### Freshness is the consumer's decision

The adapter never refuses to return an old price. SEP-40 leaves staleness to the consumer, and it is a property of the consumer's use. A quarterly NAV is fresh enough for a vault whose redemptions settle quarterly and stale for one that redeems daily. FR-06-4 puts the configurable max staleness on the vault, and the vault is where it will live.

### Storage

Configuration (admin, base, decimals, resolution) is instance storage, set once by the constructor. `decimals` and `resolution` cannot change, since changing either would reinterpret every recorded price. Assets, configs, reporter grants, the latest price and history are persistent. Write paths extend what they touch to the network maximum, per `stellarforge-common`. A price written once and read for months must not archive between reports.

The error enum is `OracleError` 1–13, frozen per ADR-003.

## Alternatives Considered

**A custom NAV interface.** It could return staleness and the reporter's identity in one call. Rejected because the vault could then read only this contract, and a Reflector-priced asset would need a second code path. SEP-40 is a small interface, and the missing context is available from events.

**Wrap Pyth and Band now.** Reflector is already SEP-40, so it needs no wrapper. Band's Soroban contract and Pyth have their own interfaces, and neither publishes the values this protocol needs first. A wrapper is a small SEP-40 contract that can be written when an asset needs one, with no change to the vault. Building them speculatively would add audit scope for no current user.

**Aggregate several reporters (median or quorum).** Deferred. An appraised NAV normally has one valuer of record. Two appraisals that disagree need a human decision, and a median across two parties does not make one. Where aggregation fits, such as several market feeds for a liquid asset, an aggregator can itself be a SEP-40 contract over other feeds. Consumers still see one SEP-40 address. The per-asset reporter model already allows more than one reporter, with the latest valid report winning.

**Temporary storage for history, as Reflector uses.** Rejected. Reflector's feeds update every few minutes, so expiring history costs little. A NAV feed may go months between reports, and its latest price must still be readable. Temporary entries cannot be restored once expired, while persistent entries can.

**A time-weighted breaker**, limiting movement per day rather than per report. Deferred for simplicity. See the first consequence below.

## Consequences

- **The breaker bounds each report, not each day.** One report per tick can move the price by the limit. A compromised reporter can compound this: at a 10% limit and a one-hour resolution, the price can double in under eight hours. The resolution is therefore also a rate limit, and NAV feeds should use one matched to how often the value really changes, typically a day. Consumers such as the vault should still bound how far they act on a single price change.
- **The admin can record any price** through `override_price`. The admin of a NAV feed has the same power over a vault's share price as the issuer has over supply, so it should be a multisig and, from Phase 3, governance. `transfer_admin` requires both parties, as in every other contract.
- **Every report pays rent for a persistent entry** at the network maximum TTL. At NAV update frequencies this is negligible. A high-frequency market feed should use Reflector instead.
- **Rotating a reporter takes one admin call per asset**, because grants are per asset. A deliberate cost of least privilege.
- **Not yet wired in.** `scripts/deploy.sh`, the testnet smoke test and the SDK do not cover the adapter yet. The vault, which consumes it, is the next Phase 2 contract.
