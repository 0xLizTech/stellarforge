# Threat Model: `rwa-asset`

**Status:** Accepted
**Date:** 2026-10-16
**Scope:** `contracts/rwa-asset` (786 lines, per the [external audit handover](../audits/external-audit-handover-phase1.md)). Phase 1 only.

---

## 1. Purpose and relationship to other security docs

This is a per-entry-point STRIDE pass over `rwa-asset`, written to close the
open follow-up named in the Phase 1 internal review: the PRD's threat model
(`docs/prd/PRODUCT_REQUIREMENTS.md` §10.1-10.2) covers the protocol at large,
and the [external audit handover](../audits/external-audit-handover-phase1.md)
gives an auditor the trust model, invariants and deliberate design choices,
but neither walks every entry point in turn.

This document does not repeat what those already say well. It cross-references:

- The **trust model** and **invariants** in the audit handover (§4 and §5),
  which this document assumes rather than restates.
- [ADR-004](../architecture/004-cross-contract-interaction.md) for the
  compliance call's threat surface, covered here only at the entry points
  that invoke it.
- [SECURITY.md](../SECURITY.md)'s resolved-issues log, cited by `SF-2026-XXX`
  ID wherever a threat below was already found and fixed.

Where this document finds a gap that is not already a recorded finding, it is
called out explicitly under that entry point's "Gaps" line rather than
silently folded into "mitigated."

## 2. STRIDE categories used below

| Letter | Category | Meaning here |
|---|---|---|
| S | Spoofing | Acting as an address without its authorization |
| T | Tampering | Writing storage the caller should not control |
| R | Repudiation | A state change with no event, or an event for a change that did not happen |
| I | Information Disclosure | Reading data that should be restricted (contracts have none; see §4) |
| D | Denial of Service | Making an entry point permanently or temporarily unusable |
| E | Elevation of Privilege | Reaching admin- or issuer-only behavior without the role |

**On Information Disclosure:** every entry point below omits `I` from its
table. All contract storage on Soroban is publicly readable by any node that
syncs the ledger; there is no private state to disclose. `I` only becomes
relevant if a future phase adds off-chain data the contract references
on-chain (e.g. an encrypted legal document pointer), which Phase 1 does not.

## 3. Entry points

Each table lists only the STRIDE categories with a non-trivial threat. A
blank table ("not applicable here") means the entry point's STRIDE surface is
fully covered by the authorization/storage model described in §4.

### `__constructor(admin, metadata)` — `lib.rs:88`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — deploying with an admin the deployer does not control | `admin.require_auth()` before any write (`lib.rs:89`) | None. Covered by `test_constructor_auth.rs` per SF-2026-019. |
| **T** — metadata that breaks supply accounting (oversized `decimals`, negative cap, empty name/symbol) | `validate_metadata` (`lib.rs:554-562`) runs before any storage write | None found. |
| **D** — a window with no admin, during which anyone can claim the role | None needed: a constructor runs atomically with deployment, so there is no window (noted in the handover §3, "no uninitialized window") | None. |

### `set_issuer(issuer, approved)` — `lib.rs:109`

| Threat | Mitigation | Gaps |
|---|---|---|
| **E** — a non-admin granting itself or another address the issuer role | `require_admin` (`lib.rs:110`) | None. |
| **R** — a grant/revoke with no trace | `IssuerSet` event (`lib.rs:117`), fixed under SF-2026-005 | None. |

### `mint(issuer, to, amount)` — `lib.rs:130`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — minting as an issuer without that issuer's signature | `issuer.require_auth()` (`lib.rs:131`) | None. |
| **E** — a non-issuer minting | `is_issuer` check (`lib.rs:133-135`) after auth, so an unregistered caller cannot even reach the compliance check | None. |
| **T** — minting past the supply cap, or a negative/zero amount | `require_positive` (`lib.rs:136`) and the `max_supply` check (`lib.rs:143-145`), both before any write | None. |
| **T** — minting to a party that fails compliance screening | `require_compliant(&env, &to)` (`lib.rs:137`), see ADR-004 for the cross-contract threat surface this delegates to. `compliance::evaluate` now also rejects a subject whose jurisdiction is on the admin-maintained deny list (`set_jurisdiction_denied`, issue #141), closing the gap this review originally found at this entry point: a sufficient-level record in a denied jurisdiction no longer passes. | None remaining; regression-tested in `contracts/compliance/tests/test_jurisdiction_deny_list.rs`. |
| **D** — integer overflow in the running total | `checked_add` (`lib.rs:592-597`), panics rather than wrapping | None. |

### `burn(from, amount)` / `burn_from(spender, from, amount)` — `lib.rs:165`, `lib.rs:196`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — burning from a balance without its owner's (or an authorized spender's) signature | `from.require_auth()` on `burn`; `spender.require_auth()` plus `spend_allowance` on `burn_from` | None. |
| **T** — burning more than the balance holds | Balance check before the write (`lib.rs:171-173`, `lib.rs:203-205`) | None. |
| **T** — screening bypass via burn | Deliberately not mitigated: burn screens nobody by design (ADR-004, "burn screening nobody is deliberate"). This is a documented non-finding, not a gap: burning destroys value, so there is no counterparty for a jurisdiction or level rule to protect. | None beyond what ADR-004 already records. |

### `transfer(from, to, amount)` — `lib.rs:231`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — moving a balance without its owner's signature | `from.require_auth()` (`lib.rs:234`) | None. |
| **T** — self-transfer minting tokens from nothing | Explicit `if from == to { return; }` no-op (`lib.rs:248-250`), fixed under **SF-2026-001 (critical)** | None remaining; regression-tested (`test_self_transfer_does_not_create_tokens`). |
| **T** — transferring to/from a party that fails screening | `require_compliant` on both legs (`lib.rs:237-238`), now including the jurisdiction deny-list check (#141) | None remaining. |
| **D** — transferring while paused | `require_not_paused` (`lib.rs:235`) makes this fail safely rather than silently succeed | None. |
| **R** — a self-transfer emitting a `Transfer` event for a change that did not happen | The no-op path returns before the event publish | None; this is `transfer`'s half of the pattern SF-2026-017 fixed for `transfer_from`. |

### `approve(owner, spender, amount, live_until_ledger)` — `lib.rs:286`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — setting an allowance without the owner's signature | `owner.require_auth()` (`lib.rs:293`) | None. |
| **T** — an allowance that outlives its intended window | `live_until_ledger` must not be in the past when `amount > 0` (`lib.rs:298-300`); `read_allowance` treats a lapsed allowance as zero (`lib.rs:496-503`) | **Documented, not a gap**: a new `approve` call replaces the old amount outright (SEP-41 approve-overwrite race). This is deliberate per the function's own doc comment and ADR-004's consequences list; a spender racing the change can still spend the old allowance once. Mitigation available to callers: set to 0, confirm, then set the new amount. |

### `transfer_from(spender, from, to, amount)` — `lib.rs:323`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S** — spending an allowance without the spender's signature | `spender.require_auth()` (`lib.rs:324`) | None. |
| **E** — spending beyond the granted allowance | `spend_allowance` checks `current.amount < amount` before any balance write (`lib.rs:511-513`) | None. |
| **T** — self-transfer-from minting tokens from nothing | Same no-op guard as `transfer` (`lib.rs:344-347`), fixed under **SF-2026-001** | None. |
| **R** — a self-transfer-from emitting a phantom `Transfer`/allowance-spend event | No-op path still extends storage and returns before any event (`lib.rs:345-346`); the phantom-event half of this was **SF-2026-017 (info)** | None remaining. |
| **T** — screening bypass on the owner or recipient | `require_compliant` on both `from` and `to`, not the spender (`lib.rs:327-328`); ADR-004 explains why the spender itself is not screened. Jurisdiction is screened here the same way (#141). | None remaining. |

### `set_paused(paused)` — `lib.rs:421`

| Threat | Mitigation | Gaps |
|---|---|---|
| **E** — a non-admin pausing or unpausing the asset | `require_admin` (`lib.rs:422`) | None. |
| **D** — pause used to permanently freeze holder funds with no recourse | Admin operations (`set_issuer`, `update_metadata`, `transfer_admin`, `set_compliance`) remain callable while paused, so a stuck pause is recoverable by rotating the admin key; this is the trust model's accepted risk (admins are trusted, per handover §4) rather than a contract-level mitigation | None beyond the trust-model boundary already documented. |

### `update_metadata(metadata)` — `lib.rs:428`

| Threat | Mitigation | Gaps |
|---|---|---|
| **E** — a non-admin changing metadata | `require_admin` (`lib.rs:429`) | None. |
| **T** — re-denominating balances via `decimals`, or uncapping/lowering a cap below circulating supply | `validate_metadata_change` (`lib.rs:571-590`), fixed under **SF-2026-004** | None remaining; regression-tested. |
| **R** — a metadata change with no trace | `MetadataUpdated` event carrying both old and new values (`lib.rs:440`) | None. |

### `transfer_admin(new_admin)` — `lib.rs:443`

| Threat | Mitigation | Gaps |
|---|---|---|
| **S/E** — handing the admin role to an address that never agreed to hold it, or a compromised current admin unilaterally installing a successor | Dual authorization: `require_admin` (current) **and** `new_admin.require_auth()` (`lib.rs:444-445`), fixed under **SF-2026-006**, satisfying NFR-S-4 | None. |
| **R** — a handover with no trace | `AdminTransferred` event with both addresses (`lib.rs:450-454`) | None. |

### `set_compliance(compliance, min_level)` — `lib.rs:465`

| Threat | Mitigation | Gaps |
|---|---|---|
| **E** — a non-admin disabling or redirecting screening | `require_admin` (`lib.rs:466`) | None. |
| **T** — pointing at a contract that does not implement `screen`, or panics | Not mitigated at this entry point; detected at the first `require_compliant` call as a failed invocation (ADR-004), which fails the transfer rather than silently passing it. Issue #143 added regression tests (`test_compliance_failure_modes.rs`) proving this fail-closed behaviour against a panicking contract, a contract missing `screen`, and an address with nothing deployed behind it, so this is now demonstrated rather than only asserted in prose. | None remaining beyond the accepted residual noted in §5: this failure mode exists at all only because screening is a runtime, not compile-time, dependency (ADR-004's own tradeoff). |
| **R** — a compliance change with no trace | `ComplianceSet` event (`lib.rs:474-478`) | None. |
| **D** — an asset permanently stuck unscreened because no admin ever calls this | Not a contract-level threat: an unconfigured asset fails *open* by design (ADR-004), so this is an availability property, not a denial of it. The exposure is operational visibility, which the handover already flags as "the check an operator ... should be making." | None beyond what ADR-004 already records. |

### Read-only views (`balance`, `allowance`, `total_supply`, `metadata`, `decimals`, `name`, `symbol`, `admin`, `paused`, `is_issuer`, `compliance_contract`, `min_compliance_level`)

| Threat | Mitigation | Gaps |
|---|---|---|
| **T** — any write path reachable through a read | None of these entry points touch `env.storage().*.set` or `.remove`; confirmed by inspection of each body (`lib.rs:368-487`, `lib.rs:120-125`) | None. |
| **D** — a read whose cost grows with protocol state | Every read here is a single keyed lookup (`O(1)` storage get), not a scan; `rwa-asset` holds no list-shaped state (unlike `registry`, which SF-2026-003 fixed for exactly this reason) | None. |

## 4. Cross-cutting threats not tied to one entry point

**Authorization completeness.** Every state-changing entry point above requires
the authorization the [ADR-002 map](../architecture/002-auth-patterns.md)
specifies, including `transfer_from`/`burn_from` requiring the spender's
(not the owner's) signature, and `screen` (in `compliance`, called from here)
requiring none, by design. No entry point in this contract was found to be
missing a `require_auth`/`require_admin` call its role implies.

**Error-discriminant stability.** `RwaError`'s doc comment states discriminants
are append-only. A future PR that renumbers or removes a variant would not be
a STRIDE threat against a holder, but would silently break any off-chain
system (an SDK, an indexer, a wallet) that matches on error codes, which this
document records as a process threat rather than a code one: enforced by
code review, not by a runtime check.

**Reentrancy.** Out of scope for every entry point above: Soroban's execution
model does not allow a called contract to call back into the caller before
the caller's frame completes, so no entry point here needs a reentrancy guard.
Recorded once here rather than repeated under each entry point that calls out
to `compliance`.

## 5. Residual risk accepted as-is (not a gap)

These are deliberate, already documented elsewhere, and this document takes
no further position on them:

- An asset with no compliance contract configured transfers freely (ADR-004).
- `burn`, `burn_from` and `approve` screen nobody (ADR-004).
- `min_level = 0` still requires a KYC record to exist; it is not "screening off" (ADR-004 Consequences).
- A compromised admin key can disable screening, grant itself issuer, or pause
  the asset indefinitely; admins are a trusted role (handover §4).
- The approve-overwrite race (`approve`'s own doc comment).

## 6. Gaps this review surfaced, and their fixes

This review surfaced two gaps, filed and closed as part of the same
hardening stage this document belongs to rather than left open:

1. **Jurisdiction was not screened**, only verification level (`mint`,
   `transfer`, `transfer_from`). Closed by issue #141: `compliance` now
   carries an admin-maintained jurisdiction deny list
   (`set_jurisdiction_denied`/`is_jurisdiction_denied`), checked inside
   `evaluate` alongside level and expiry. It lives in `compliance`, not
   `rwa-asset`, per ADR-004's reasoning for keeping jurisdictional rules out
   of the unupgradeable asset contract.
2. **A broken or malicious compliance contract had a defined failure mode
   (fails closed) with no regression test proving it.** Closed by issue
   #143: `contracts/rwa-asset/tests/test_compliance_failure_modes.rs` now
   demonstrates the fail-closed behaviour against a panicking stand-in
   contract, a contract that does not implement `screen`, and a plain
   address with no deployed contract at all, and confirms `burn`/`approve`
   still work and `set_compliance` is the recovery path in every case.

## 7. Review

This document should be revisited whenever `rwa-asset`'s entry-point surface
changes, and at minimum alongside every future external audit engagement
(handover §2, "Fixes land as later commits"). It has no expiry of its own;
the audit handover is the controlling scope document for any given audit
engagement.
