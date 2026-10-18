# Threat Model: `registry`

**Status:** Accepted
**Date:** 2026-10-16

---

## What this contract is, and is not

`registry` is a directory of `rwa-asset` contract addresses, kept so a
front end or indexer can enumerate "every asset this protocol knows about"
without crawling chain history for deploy transactions. It is not a
verification service. `register` records what the admin asserted and
checks nothing about it: not that `contract` is a deployed `rwa-asset`, not
that it even exists, not that `asset_class` matches anything the asset
itself claims. `active` is a label the admin sets and the protocol never
reads back (IR-16, ADR-003). An entry in this directory is evidence of what
the registry admin said, not proof that an asset is genuine, current, or
even real.

This document exists because that gap, once stated plainly, has concrete
failure modes for anyone building on top of the directory. It covers three:
a fake or malicious entry, directory spam, and delisting (`active = false`)
being mistaken for something the contract itself does not actually enforce.

## Threat 1: Fake or malicious entries

**Scenario.** The registry admin key is compromised, or the admin itself is
dishonest, or simply makes a typo. `register` is called with a `contract`
address that is not a real `rwa-asset`: an unrelated contract, an address
with no code at all, or (more dangerously) a contract designed to look
enough like `rwa-asset`'s read interface (`balance`, `metadata`, `decimals`)
to deceive a front end that trusts the directory blindly.

**What the contract does.** Nothing stops this. `register`'s only
check is the caller's admin authorization (`require_admin`); there is no
validation of `contract` at all. The call succeeds, an `AssetRegistered`
event publishes with `is_new: true`, and the address is now indistinguishable
in `list_assets`'s output from every genuine entry.

**Why this is accepted as-is, not fixed.** On-chain verification that
`contract` actually implements `rwa-asset`'s interface would require either
a cross-contract call into every candidate at registration time (expensive,
and callable-but-wrong-behavior would still pass it) or a WASM hash
allowlist tied to a specific `rwa-asset` build (couples the registry to one
contract version, defeating the point of a directory that can list assets
deployed at different times against different `rwa-asset` releases).
ADR-003 already frames this contract as a directory, not a registry of
truth, and that framing is the actual mitigation: nobody downstream should
be relying on registry membership as the proof.

**What actually bounds the damage.**
- A fake entry cannot do anything to a real asset. The registry has no
  write access to any `rwa-asset` contract and nothing in `rwa-asset` reads
  the registry at all; the coupling is one-directional (consumer → registry
  → address), never the reverse. A malicious entry is inert data, not an
  attack surface against the rest of the protocol.
- `transfer_admin` requires dual authorization (ADR-002), so a compromised
  admin key, once noticed, can be rotated out, and `set_paused` (added in
  #126) lets anyone watching stop further `register`/`set_active` calls
  immediately without needing the dual-auth handover to complete first.
- `AssetRegistered`'s `is_new` field and topic-filterable `contract` address
  mean every registration is independently auditable by any indexer that
  chooses to watch, which is the actual backstop this threat model
  recommends relying on (see **Indexer guidance** below), not anything the
  contract itself enforces.

## Threat 2: Directory spam

**Scenario.** The admin key (compromised or not) calls `register`
repeatedly with junk or duplicate-intent entries, trying to make the
directory large enough to be expensive or slow to page through, or to bury
genuine entries among noise.

**What the contract does.** `register` is admin-gated
(`require_admin`), so this threat only materializes if the admin key
itself is the attacker or is compromised — there is no public spam vector.
Within that constraint:
- Re-registering the *same* `contract` address updates its existing entry
  in place rather than appending a duplicate (the `is_new` check against
  `DataKey::Asset(contract)`), so an admin cannot inflate the directory's
  length by re-registering one address repeatedly.
- Registering *new, distinct* addresses has no cap: `MAX_ASSETS`-style
  bounds exist on `oracle-adapter` (`MAX_ASSETS = 100`) but not here, since
  `list_assets` pages rather than returning the whole directory in one call
  (IR-01's original motivation: a single-list design hit the 64 KiB entry
  size limit around 1,600 assets and failed permanently after that; the
  per-asset-entry, paged design removed that ceiling entirely).
- Because `list_assets` pages at `MAX_PAGE_SIZE` (100) regardless of
  directory size, and `register`'s own footprint is O(1) regardless of how
  many entries already exist, an arbitrarily large directory degrades
  nobody's single call. The cost of spam is paid once, by whoever submits
  the spamming transactions (their own fees), not by future callers.

**Residual risk.** An admin determined to make the directory large can do
so; this is the known, deliberate cost of a directory with no
registration cap. A directory of thousands of mostly-junk entries is
inconvenient for `listAllAssets()`-style "fetch everything" callers (more
pages to walk) but not broken: each page is still bounded and cheap.
Flagged here as a known characteristic of the design, not a fix proposed
for Phase 1.

## Threat 3: Delisting (`active = false`) is not enforcement

**Scenario.** The registry admin, trying to respond to a problem with a
listed asset (a bug discovered in that `rwa-asset` deployment, a legal
issue, a compromised asset admin key), calls `set_active(contract, false)`,
expecting this to stop the asset from being usable.

**What the contract does.** `active` is a flag stored in the asset's
`AssetEntry`, surfaced through `get_asset`/`list_assets`, and nothing else.
No code path in `rwa-asset` reads the registry at all, so setting `active`
to `false` has zero on-chain effect on that asset's `mint`, `transfer`, or
any other entry point. The asset keeps working exactly as before. This is
IR-16's second half, carried forward here rather than fixed, because fixing
it would mean `rwa-asset` calling back into `registry` on every state
change to check a flag the asset contract has no other reason to know
about — a cross-contract dependency in the wrong direction (ADR-004's whole
point is that `rwa-asset` depends on `compliance` for this kind of check,
deliberately, through a narrow interface; it does not generally depend on
`registry`).

**Why this matters more than Threat 1.** A fake entry is obviously inert to
anyone who checks; `active = false` looks like it should do something,
which is exactly the condition under which someone acts on a false
assumption. An operator who delists an asset and believes transfers have
stopped, when they have not, is a worse outcome than an operator who never
trusted the flag to begin with.

**What actually stops a problem asset.** `rwa-asset`'s own `set_paused`
(the admin's pause on that specific asset contract) or `transfer_admin`
away from a compromised key are the only on-chain ways to actually stop an
asset's transfers. `set_active(false)` on the registry is, and should be
treated as, a signal to off-chain consumers (front ends, indexers) to stop
*surfacing* the asset — not a kill switch on the asset itself.

## Indexer guidance

Anyone building on top of this directory should treat it as follows:

1. **Registry membership is not verification.** Before trusting an address
   from `list_assets`/`get_asset` as a genuine `rwa-asset`, independently
   confirm it behaves like one (e.g. that `metadata()`, `decimals()`,
   `total_supply()` return sane, internally consistent values) rather than
   assuming the registry already checked this. It did not.
2. **`active = false` means "the registry admin says don't show this,"**
   not "this asset stopped working." An indexer that hides a delisted
   asset from its own UI is doing the right thing with the signal; an
   indexer that tells users the asset itself is now inert is overstating
   what the flag means.
3. **Watch `AssetRegistered` for `is_new: true` on addresses you have not
   seen before**, and treat a sudden high rate of new registrations from
   the current admin as worth a manual look — it is the closest on-chain
   signal to the spam/fake-entry threats above, even though the contract
   itself does not rate-limit or flag it.
4. **Watch `AdminTransferred`.** A rotation to an address you do not
   recognize, especially one that happens outside a publicly announced
   maintenance window, is the leading indicator of a key compromise this
   document's Threat 1 depends on; it is dual-authorized (ADR-002) so it
   cannot happen silently with only one party's cooperation, but it can
   still happen with both of a stolen pair's signatures.
5. **A paused registry (`paused() == true`, #126) still has whatever
   entries it already had.** Reads keep working while paused; pausing
   stops new registrations and `active` changes, not the ability to list
   what is already recorded. Do not treat a paused registry as "empty" or
   "unavailable" in indexing logic.

## Related documents

- [ADR-003: Upgrade Path](../architecture/003-upgrade-path.md) — frames the
  registry as a directory, the deliberate choice this whole document
  builds on.
- [ADR-004: Cross-Contract Interaction](../architecture/004-cross-contract-interaction.md)
  — why `rwa-asset` depends on `compliance` but not on `registry`.
- [Internal review, IR-16](../audits/2026-09-14-internal-review-phase1.md)
  — the original finding this document expands into a full threat model.
- [External audit handover, §4 Trust model](../audits/external-audit-handover-phase1.md)
  — the registry admin's powers and limits, in the context of every other
  contract's trust boundaries.
