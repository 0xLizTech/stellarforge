# ADR-002: Auth Patterns

**Status:** Accepted
**Date:** 2026-08-23
**Authors:** Community (docs)

---

## Context

Every StellarForge contract mutates privileged state (admin keys, pause flags, KYC records, mint/burn). Soroban authorizes those mutations with `Address::require_auth` (and dual-auth where two parties must consent). We need one pattern so auditors and integrators know who must sign which call, and so NFR-S-4 (admin transfer safety) is not reinvented per contract.

## Decision

### `require_auth` is explicit at the entry point

- Privileged calls load the stored role address and call `require_auth()` on it **inside the entry point**, not buried in a helper that might be skipped.
- Shared helpers such as `require_admin(env)` are thin wrappers around `Self::admin(env).require_auth()` so the auth surface stays greppable.
- User-initiated transfers call `from.require_auth()` (or the equivalent owner/spender auth for allowance paths) before any balance mutation.

### Role model

| Role | Typical storage | Powers |
|------|-----------------|--------|
| **Admin** | instance (`ADMIN` symbol / equivalent) | Pause, metadata updates, point compliance config, transfer admin |
| **Issuer** | contract-specific (rwa-asset) | Mint/burn within policy |
| **Subject / holder** | n/a (caller address) | Transfer, approve — only for self |
| **Compliance admin** | compliance contract admin | Write/update KYC records |

Contracts do **not** share a single on-chain RBAC module. Each deployable contract owns its admin key so they remain independently deployable (see ADR-003).

### Dual-auth admin transfer (NFR-S-4)

`transfer_admin(new_admin)` must authenticate **both**:

1. The **current** admin (`require_admin` / current admin `require_auth`), and
2. The **new** admin (`new_admin.require_auth()`).

This prevents an admin from unilaterally pointing the contract at an address they do not control (typo, hostile handoff, or unclaimed key). Both signatures are required in the same invocation.

```text
transfer_admin(new_admin):
  require current admin auth
  require new_admin auth
  instance.set(ADMIN, new_admin)
```

## Alternatives Considered

- **Single-auth handoff** (only current admin signs). Rejected: silent mis-assignment is irreversible without a recovery path we deliberately do not ship.
- **Timelock / multi-sig wrapper only.** Compatible as an *outer* policy (admin can be a multisig contract) but not a substitute for dual-auth on the primitive itself.
- **Claim-based handoff** (`propose_admin` + `accept_admin`). Deferred: two-step is clearer for some ops teams but doubles the entry surface; dual-auth in one tx matches current code and keeps the audit surface small.

## Consequences

- Integrators must build transactions that include auth entries for every `require_auth` address (simulation/RPC tooling surfaces these).
- Admin keys should be contracts (multisig/DAO) when operational policy needs thresholds; the primitive still enforces dual consent on handoff.
- New privileged entry points must document Auth in rustdoc and follow the same `require_*` helpers.