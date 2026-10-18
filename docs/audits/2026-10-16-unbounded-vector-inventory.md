# Unbounded Vector Inventory

**Status:** Resolved. Every item has a verdict; none require a code fix.
**Date:** 2026-10-16
**Scope:** Stage 2, Phase 1 Hardening and Audit Follow-ups (#121)

---

## 1. Why this exists

IR-01, in the [Phase 1 internal review](2026-09-14-internal-review-phase1.md),
found that `registry`'s directory was a single `Vec<Address>` in one
persistent entry: every `register` call read the whole vector, appended to
it, and wrote it back, so the entry's own size grew with every registration.
Past roughly 1,600 entries the entry crossed the mainnet 64 KiB
contract-data limit and every further `register` failed for good, with no
way to recover since no entry had an upgrade path (ADR-003). The fix
replaced it with an indexed layout: `AssetCount -> u32` plus
`AssetAt(u32) -> Address`, paginated through `list_assets(start, limit)`.

This document asks whether that same shape, an unbounded `Vec` growing
inside a single storage entry with nothing to stop it, exists anywhere else
across the five Phase 1-2 contracts or the shared `common` crate. Every
`soroban_sdk::Vec` usage in non-test code in `contracts/` was read against
every `.set(...)` call site in the same crate; nothing was sampled.

## 2. Method

A value only counts as "a Vec in storage" if a `Vec` is itself written to a
persistent or instance storage entry, either directly or as a field of a
stored `#[contracttype]`. A `Vec` built in memory and returned from a pure
read helper that pages over already-indexed storage (`registry::list_assets`,
`oracle_adapter::prices`) is not in scope: it never grows the storage entry
behind it, no matter how large the page requested is, because each caller
chooses how much to read and the underlying entries are indexed separately.
`std::vec::Vec` (plain Rust, used only in test files for assertions) is a
different type entirely and never touches ledger storage.

## 3. Inventory

| Contract | Vec-in-storage found | Verdict | Evidence |
|---|---|---|---|
| `registry` | None. IR-01 replaced the one that existed with `AssetAt(u32)` indexed storage. | **Resolved** (no open item) | `contracts/registry/src/lib.rs:28-36` (`DataKey`), `:79-114` (`register`), `:131-148` (`list_assets`, a pure paginated read helper, capped at `MAX_PAGE_SIZE = 100` per call) |
| `oracle-adapter` | `DataKey::Assets` holds a `Vec<Asset>` directly, the list of every asset the feed has ever listed. | **Bounded.** Capped at `MAX_ASSETS = 100`, checked immediately before the one and only `push_back` that can grow it. | `contracts/oracle-adapter/src/lib.rs:68` (`MAX_ASSETS`), `:307-327` (`add_asset`: the check at `:316-318`, the push at `:319`, the write at `:321`). Confirmed `add_asset` is the sole write path to `DataKey::Assets` by checking every `.set(&DataKey::Assets` and `push_back`/`push_front` call site in the crate. Already covered by `test_add_asset_stops_at_max_assets` (`contracts/oracle-adapter/tests/test_oracle_adapter.rs:186-202`), which fills the list to exactly `MAX_ASSETS` and asserts the next `add_asset` fails with `TooManyAssets`. |
| `oracle-adapter` | Price history (`prices()`) | **Not applicable.** This is the linked-list design ADR-005 describes ("each stored report links to the one before it"), not a Vec in storage: each `report`/`override_price` call writes one fixed-size `Report` entry keyed by `(asset, tick)` and updates one `Latest(asset)` pointer; nothing ever grows. `prices()` walks the links backward into an in-memory `Vec`, capped by `MAX_RECORDS = 20` regardless of how much history exists. | `contracts/oracle-adapter/src/lib.rs:104-111` (`Report`), `:125` (`DataKey::Report`), `:425-480` (`record`, the shared write path for `report` and `override_price`), `:201-227` (`prices`) |
| `governance` | None. | **Not applicable** (no open item) | `contracts/governance/src/lib.rs:50-75`: `Proposal` has no `Vec` field (`votes_for`/`votes_against` are scalar `i128` tallies incremented in place); votes are individually keyed via `DataKey::Vote(u64, Address)`, never accumulated into a list. |
| `rwa-asset` | None. | **Not applicable** (no open item) | `contracts/rwa-asset/src/lib.rs:48-82`: `Balance`, `Allowance` and `Issuer` are all keyed per address (or address pair) via `DataKey`; `AssetMetadata` and `AllowanceValue` have no `Vec` field. One entry per address that has ever held a balance, allowance or issuer flag is the standard per-key storage pattern this workspace uses everywhere else, not a single growing entry. |
| `compliance` | None. | **Not applicable** (no open item) | `contracts/compliance/src/lib.rs:26-39`: `KycStatus(Address)` is keyed per address; `KycRecord` has no `Vec` field. |
| `common` | None. | **Not applicable** (no open item) | `contracts/common/src/lib.rs` and `src/storage.rs` define no `#[contracttype]` and call no `.set(...)`; they only extend the TTL of keys the calling contract already wrote. |

## 4. Conclusion

The IR-01 pattern, an unbounded `Vec` in a single storage entry with no cap
on any write path, does not recur anywhere else in this workspace. The one
other Vec-in-storage that exists, `oracle-adapter::DataKey::Assets`, is a
different and safer shape: it is capped at a small, fixed size (100 entries,
nowhere near the 64 KiB limit that caused IR-01 at roughly 1,600
`Address`-sized entries), the cap is enforced on its only write path, and a
test already asserts the exact boundary. It required no fix to close this
item, only this record of having been checked.

## 5. Residual risk

`MAX_ASSETS` is enforced by `add_asset`'s own check, not by anything
structural that would stop a future second write path from bypassing it,
the way `registry`'s indexed layout makes "write without checking the
index" structurally impossible for its own directory. A contributor who
later added a second entry point that appends to `DataKey::Assets` directly,
such as a batch-add convenience method, would need to repeat
`add_asset`'s check itself; nothing in the type system or storage layout
would catch the omission. This is not a current defect, since no second
write path exists today, but it is worth a reviewer's attention if one is
ever proposed.

## References

- [Internal Security Review, Phase 1](2026-09-14-internal-review-phase1.md), IR-01
- [ADR-003: Upgrade Path](../architecture/003-upgrade-path.md)
- [ADR-005: Oracle Adapter](../architecture/005-oracle-adapter.md)
