# Threat Model: Oracle Adapter

**Status:** Accepted
**Date:** 2026-10-16
**Authors:** StellarForge Core Team

---

## Context

This document expands the PRD's threat model (§10.1-10.2) "Oracle
manipulator" row, and [ADR-005: Oracle Adapter](../architecture/005-oracle-adapter.md)'s
design decisions, into the entry-point level for `contracts/oracle-adapter`.
It is a Stage 2 Phase 1 Hardening and Audit Follow-up: ADR-005 records why
the adapter is built the way it is, but not, in one place, what each design
decision defends against and what it deliberately leaves open.

Price feeds are a classic attack surface: whoever can move the price a
consumer trusts can usually extract value from that consumer. This contract
is unusual among oracles in that it has no independent source of truth to
check a reporter against: ADR-005 is explicit that "no public oracle
publishes the NAV of a real-world asset," so the adapter's reporters are
themselves the source of truth, not a relay of one. Every mitigation below is
therefore about bounding what a reporter (or a compromised reporter key) can
do, not about detecting disagreement with an external reference, because
none exists.

## Scope

`contracts/oracle-adapter/src/lib.rs` as of this document's date:
`report`, `override_price`, `add_asset`, `set_max_deviation`, `set_reporter`,
`transfer_admin`, and the read-only `price`, `prices`, `lastprice`, `base`,
`assets`, `decimals`, `resolution`, `admin`, `is_reporter`, `asset_config`.
The vault that will consume this adapter (Phase 2) is out of scope; this
document covers only what the adapter itself guarantees or fails to
guarantee to any consumer.

## Threats

### T-1: Reporter key compromise

**Actor:** Oracle manipulator (PRD §10.1), holding a compromised reporter
key for one asset.

**Mechanism.** `report(env, reporter, asset, price, timestamp)` requires only
`reporter.require_auth()` and that `reporter` is listed with
`allowed = true` for that specific `asset` (`DataKey::Reporter(asset,
reporter)`, checked against `OracleError::NotReporter`). A reporter grant is
per-asset (`set_reporter`), so a compromised key for one asset's reporter
cannot move any other asset's price; `add_asset`/`set_reporter` are both
admin-gated separately.

**Current mitigation.** Each listed asset's `max_deviation_bps`
(`AssetConfig`) bounds how far a single `report` can move the price from
`lastprice` (`OracleError::DeviationTooLarge` otherwise). This is ADR-005's
designed circuit breaker: "it bounds what a single compromised reporter key
can do in one report." A deviation limit of `0` is rejected at `add_asset`
and `set_max_deviation` time (`OracleError::InvalidDeviation`) specifically
so "unlimited" can never be expressed by omission; disabling the breaker
requires a conspicuously large configured value, which is a deliberate
admin action an auditor can spot in `AssetConfigured`/`MaxDeviationUpdated`
event history.

**Residual risk.** ADR-005's own "Consequences" section names this directly:
"The breaker bounds each report, not each day. One report per tick can move
the price by the limit. A compromised reporter can compound this: at a 10%
limit and a one-hour resolution, the price can double in under eight hours."
A compromised key that survives multiple reporting ticks before detection can
walk the price arbitrarily far, one bounded step at a time. The mitigation is
therefore only as strong as how quickly a compromise is detected and the
reporter revoked (`set_reporter(asset, reporter, false)`), and how tightly
`resolution` and `max_deviation_bps` are chosen relative to how fast the
underlying NAV can plausibly move. **This is a configuration risk, not a
code defect**: it is carried forward as guidance for whoever sets
`resolution` and `max_deviation_bps` per asset at `add_asset` time, not as
something this contract's code can close further on its own.

### T-2: Stale prices

**Actor:** Oracle manipulator, or simply an inattentive or absent reporter
with no malicious intent.

**Mechanism.** `price`, `prices` and `lastprice` all return whatever was last
recorded, however old. ADR-005 states this is deliberate: "The adapter never
refuses to return an old price. SEP-40 leaves staleness to the consumer, and
it is a property of the consumer's use. A quarterly NAV is fresh enough for
a vault whose redemptions settle quarterly and stale for one that redeems
daily."

**Current mitigation.** None at this layer, by design. FR-06-4 puts the
"configurable max staleness" check on the vault that consumes this adapter,
not on the adapter itself, specifically so one staleness policy does not have
to fit every consumer.

**Residual risk.** This is a sound design as long as every future consumer
actually implements its own staleness check before this adapter has any
consumer. **If the Phase 2 vault (or any other future SEP-40 consumer of this
adapter) ships without a staleness check, this adapter provides none as a
fallback**, and a reporter who simply stops reporting (key loss, abandoned
role, or a deliberate slow-rug where the attacker benefits from the price
staying frozen at a stale, favorable value) produces no on-chain signal
distinguishable from a quiet, correctly-functioning quarterly feed. This
document does not resolve that gap; it exists so the Phase 2 vault's own
threat model, once written, must explicitly state that it closes this,
rather than silently assuming the adapter already has.

### T-3: Deviation-bypass via the admin override

**Actor:** A compromised admin key, or (per ADR-005) eventually governance
once Phase 3 wires it in.

**Mechanism.** `override_price(env, asset, price, timestamp)` requires
`Self::admin(env.clone()).require_auth()` and skips only the deviation check
(`Self::record(&env, &asset, price, timestamp, None)` passes no deviation
limit, versus `report`'s `Some(config.max_deviation_bps)`). ADR-005 is
explicit about why this exists: "Without an escape hatch the feed would be
stuck at a price everyone knows is wrong," the PRD §17 lockout risk.

**Current mitigation.** The override still runs through the same
`Self::record` path as `report`, so positivity (`OracleError::InvalidPrice`
on a non-positive price), the future-timestamp check
(`OracleError::TimestampInFuture`), and append-only history
(`OracleError::StaleReport` if the timestamp's tick is not later than the
latest recorded one) all still apply. An override cannot rewrite the past or
record a price-of-zero-or-less. It publishes `PriceOverridden`, a distinct
event from `PriceReported`, specifically so, per ADR-005, "a monitor can
alert on every override with one topic filter" — every bypass of the
deviation check is therefore individually observable.

**Residual risk.** The admin's power here is exactly as large as the
deviation limit it bypasses: an admin (or whoever holds that key) can record
*any* positive price with no bound at all, which ADR-005 names plainly: "The
admin of a NAV feed has the same power over a vault's share price as the
issuer has over supply, so it should be a multisig and, from Phase 3,
governance." Until that holds in practice for a given deployment, a
compromised or malicious admin key is a strictly stronger attack than a
compromised reporter key, because it faces no deviation bound at all, only
the positivity/timestamp/append-only invariants every path shares. This
mitigates to the same dual-authorization `transfer_admin` recovery path as
T-4 below; it does not mitigate what the admin can do while holding the key
legitimately or illegitimately.

### T-4: Admin-key compromise (rotation and lockout)

**Actor:** An attacker who obtains the oracle adapter's admin key.

**Mechanism.** The admin role on this contract controls `add_asset`,
`set_max_deviation`, `set_reporter`, `override_price`, and
`transfer_admin` itself, the widest admin power of any Phase 1 contract (see
T-3): an attacker holding this key can grant themselves a reporter role on
any asset, loosen or disable (via a very large value) the deviation limit
that bounds reporters, and record an arbitrary override price directly,
without ever needing a reporter grant at all.

**Current mitigation.** `transfer_admin(env, new_admin)` requires both the
outgoing and incoming admin to `require_auth` (NFR-S-4 dual authorization),
matching `rwa-asset` (ADR-002), `compliance`, `registry` and `governance`.
A compromised key that has not also compromised a second signer cannot hand
the role to an attacker-controlled address unilaterally.

**Residual risk.** Identical in shape to governance's T-4 (see
[docs/threat-models/governance.md](governance.md)): dual authorization
protects against a unilateral handover, but a genuinely lost (not merely
observed) admin key has no recovery path, and this adapter's admin controls
materially more than governance's does today. Given T-3's finding that the
admin's power already exceeds any single reporter's, the cost of this
contract having no key-recovery path beyond dual-signature rotation is
higher here than on any other Phase 1 contract reviewed so far.

## Assumptions

- Reporters are trusted parties (appraisers, fund administrators) acting
  under an off-chain legal or contractual relationship with the asset issuer;
  this contract enforces *bounds* on what a reporter can do, not that a
  reporter's reported value is actually correct. No mechanism here can detect
  a reporter who is honestly, cooperatively wrong.
- `resolution` and `max_deviation_bps` are configured by the admin to match
  how often and how far the underlying NAV can plausibly move (ADR-005's own
  stated assumption). This document's T-1 finding is conditioned on that
  assumption holding; a `resolution` set far looser than the asset's real
  update cadence widens T-1's attack window directly.
- No consumer of this adapter exists yet in this repository (ADR-005: "Not
  yet wired in... The vault, which consumes it, is the next Phase 2
  contract"). T-2's residual risk is therefore unverified in practice and
  must be re-examined once the vault's own threat model is written.

## References

- [PRD §10: Security Considerations](../prd/PRODUCT_REQUIREMENTS.md)
- [ADR-005: Oracle Adapter](../architecture/005-oracle-adapter.md)
- [ADR-002: Authorization Patterns](../architecture/002-auth-patterns.md)
- [Threat Model: Governance](governance.md) (T-4, the shared admin-recovery gap)
- [SECURITY.md](../SECURITY.md)
