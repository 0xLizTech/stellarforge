# Threat Model: `compliance`

**Status:** Accepted
**Date:** 2026-10-16

---

## Why this contract gets its own threat model

Every other Phase 1 contract's worst case is bounded to itself or to data
nobody else reads. `compliance` is different: `rwa-asset` calls into it on
every screened `mint`, `transfer`, and `transfer_from` (ADR-004), and the
boolean it returns is the thing that decides whether real value moves. A
wrong answer here does not stay contained in this contract's own storage —
it becomes a transfer that should have been blocked, or a legitimate holder
who cannot move their own tokens. This document covers three ways the
answer can be wrong even when every line of code runs exactly as written:
stale records, admin key compromise, and jurisdiction spoofing. It also
records the gaps nothing here closes.

## Threat 1: Stale records

**Scenario.** A holder's KYC record was accurate when written, but their
circumstances changed since: a visa lapsed, a sanctions designation was
added, a jurisdiction's rules changed. Does `screen`/`is_compliant` still
say they are compliant?

**What the contract does.** `evaluate` checks exactly two things against a
stored `KycRecord`: `level >= min_level` and `expires_at == 0 ||
expires_at > ledger().timestamp()`. There is no concept of "this was true
when written but might not be now" beyond the explicit `expires_at` the
admin set at write time. A record with `expires_at = 0` ("never expires")
is compliant forever unless the admin actively calls `revoke_kyc`.

**This is deliberate, not an oversight — with one real gap.** The
deliberate half: on-chain state cannot know about an off-chain life event
(a lapsed visa, a sanctions list update) unless something tells it, and
nothing in this protocol's Phase 1 scope integrates a live sanctions feed.
`expires_at` is the mechanism by which an operator who knows a record
should not last forever encodes that knowledge; a record meant to be
permanent is the exception operators opt into with `expires_at = 0`, not
the forced default.

The gap: **nothing automatically expires a record that was given a
realistic, non-zero `expires_at`, surfaces that it is about to, or notifies
anyone.** `screen`'s TTL-extension side effect (ADR-004 §"screen is a write
path") keeps the *storage entry* alive past Soroban's archival TTL; it has
no relationship to the KYC *verification* going stale per the record's own
`expires_at` field, and reading the code without that distinction clear is
an easy place to assume more freshness guarantee than the contract
actually gives. An admin who sets `expires_at` once and never revisits the
record relies entirely on their own off-chain process to revoke it before
or at expiry; the contract gives no warning, no grace-period event, and no
on-chain mechanism that notices expiry on its own. `is_compliant` simply
starts returning `false` the instant `ledger().timestamp()` passes
`expires_at` — correct, but silent, and only checked the next time
something calls it.

**Residual risk.** A record with `expires_at` far in the future, or `= 0`,
accurately reflects "this admin believed this would stay true," not "this
is still true." Operators are relying on their own re-verification
process, external to this contract, to call `set_kyc` again (or
`revoke_kyc`) when circumstances change. This threat model does not
propose fixing it in Phase 1; it records the boundary so nobody mistakes
`is_compliant() == true` for a live attestation rather than a cached one.

## Threat 2: Admin key compromise

**Scenario.** The compliance admin's private key is stolen, or the admin
itself turns malicious.

**Blast radius.** This is explicitly the widest in the protocol (per this
contract's own `transfer_admin` doc comment): a compromised compliance
admin can call `set_kyc` to mark *any* address compliant at the maximum
level, on *every* asset that screens against this specific compliance
contract instance. Unlike a compromised `rwa-asset` admin (confined to one
asset) or `registry` admin (confined to inert directory data, see
[`registry.md`](registry.md)), one compliance contract can back many
assets, so one compromised key compromises every transfer-restriction
guarantee for all of them simultaneously.

**What bounds it.**
- `transfer_admin` requires both the outgoing and incoming admin's
  signature in the same transaction (NFR-S-4, ADR-002). A thief who has
  only exfiltrated the private key cannot additionally forge the *new*
  admin's signature to hand the role to an address they also control
  unless they also control that second key — the dual-auth requirement
  does not stop the original compromise, but it does stop the thief from
  using stolen access to entrench a replacement admin the legitimate
  operator cannot contest.
- `set_paused`/`paused` (#126) let anyone who notices the compromise (an
  indexer alerting on `KycSet` events with an unrecognized admin, a
  monitoring system) call `set_paused(true)` — wait, **no**: `set_paused`
  is itself admin-gated (`require_admin`), so a compromised admin key can
  just as easily keep writing KYC records *through* a self-imposed pause
  gate it controls. Pausing this contract protects against the admin
  being unable to act, not against the admin being the attacker. This is
  worth stating explicitly: `#126`'s circuit breaker helps when the key is
  *lost* or an operator wants to freeze things during an incident
  response, not when the key is actively *held by an adversary*, since
  the adversary holds the same pause toggle.
- The actual backstop against an actively malicious admin is off-chain:
  `transfer_admin`'s dual-auth requirement means the legitimate operator,
  if they still hold their own key, can race to rotate the admin to a new
  address before the attacker does anything further — but this requires
  the legitimate operator to still control a signing key the attacker
  does not also have, which is a key-management property outside what
  this contract can enforce.
- `screen`'s TTL extension is explicitly the "weakest possible write"
  (ADR-004): even a fully malicious caller invoking it in a loop cannot
  create, alter, or revoke a record. The compromise surface is entirely
  `set_kyc`/`revoke_kyc`/`transfer_admin`, all of which are already
  admin-gated and dual-signed where it matters (`transfer_admin`).

**What this threat model recommends, beyond what #126 already does:**
monitoring infrastructure (outside this contract's scope to build) that
alerts on `KycSet`/`KycRevoked` event volume or rate anomalies from the
current admin address, since the contract itself has no rate limiting and
cannot distinguish "an operator onboarding many legitimate holders" from
"an attacker marking many addresses compliant" by pattern alone. Issue
#140's `set_kyc_batch` makes a large, fast burst of compliant records
*more* efficient to produce, legitimately (bulk onboarding) or otherwise,
which is a direct trade-off worth naming: the batch entry point does not
change what a compromised admin could already do (any volume of `set_kyc`
calls, just more transactions and gas for the same outcome), but it does
lower the cost of doing it, and any alerting built against this threat
should account for that.

## Threat 3: Jurisdiction spoofing

**Scenario.** An admin (or a key that compromised the admin) sets a
`KycRecord.jurisdiction` that does not reflect the holder's actual
jurisdiction — either an honest data-entry error or a deliberate attempt
to make a holder appear compliant in a jurisdiction where their actual
one would fail a downstream policy.

**What the contract checks, and what it does not.** `validate_record`
enforces *shape*: exactly two bytes, both ASCII uppercase
(`jurisdiction.len() == 2` and `is_ascii_uppercase` on each byte). This
guarantees a value is a syntactically valid ISO 3166-1 alpha-2 code shape.
It does not check that the two letters name a jurisdiction that actually
exists (`"ZZ"`, `"QQ"` and similar unassigned-but-well-formed codes pass),
and it has no way whatsoever to verify that the string matches the
specific real-world holder's actual jurisdiction. There is no
jurisdiction-to-identity binding anywhere in this contract or protocol;
`jurisdiction` is exactly as trustworthy as the admin who wrote it, with
zero additional verification.

**Why this is the correct scope for Phase 1, not a gap to close here.**
This contract's entire design is "an admin asserts a verification result";
it is not itself a KYC/AML provider, and nothing in the PRD's Phase 1 scope
calls for on-chain identity verification (which would require an oracle or
attestation scheme well beyond this contract's stated purpose). The
`validate_record` shape check exists only to catch the *unambiguous* input
errors IR-08 found (a `u32::MAX` level, an already-past expiry, "US " with
a trailing space) — errors that are wrong regardless of what the real-world
truth is. It was never meant to, and does not, defend against an admin who
asserts something false on purpose or by a mistake that happens to still
be shape-valid.

**Residual risk, named explicitly:** any downstream consumer that branches
on `jurisdiction` for a jurisdiction-specific rule (e.g. a tax-reporting
threshold, a jurisdiction-restricted asset class) is trusting the
compliance admin's data entry and honesty completely, with the same blast
radius as Threat 2: one compromised or dishonest admin can misrepresent
any holder's jurisdiction, for any reason, with no on-chain signal
distinguishing a spoofed value from a correct one. The mitigation is
entirely organizational (who holds the admin key, what off-chain process
verifies before `set_kyc` is ever called), not something a future version
of this contract alone can fix without changing what it fundamentally is.

## Related documents

- [ADR-004: Cross-Contract Interaction](../architecture/004-cross-contract-interaction.md)
  — why `rwa-asset` depends on `compliance` through exactly one function,
  the `screen`/`is_compliant` split, and the `min_level = 0` surprise
  (still requires every party to hold *some* record).
- [ADR-002: Authorization Patterns](../architecture/002-auth-patterns.md) — the
  dual-authorization requirement on `transfer_admin` this document's
  Threat 2 relies on.
- [Internal review, IR-04 and IR-08](../audits/2026-09-14-internal-review-phase1.md)
  — admin rotation and the input-validation fixes this document assumes
  are already in place.
- [External audit handover, §4 Trust model](../audits/external-audit-handover-phase1.md)
  — "A compromised compliance admin can make any address compliant on
  every asset that screens against it," stated there and expanded into
  Threat 2 here.
