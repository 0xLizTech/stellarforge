# Security Policy

## Supported Versions

| Version | Status |
|---|---|
| `main` branch | Actively maintained |
| Tagged releases | Bug fixes only (critical severity) |

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

Use GitHub's [private vulnerability reporting](https://github.com/0xLizTech/stellarforge/security/advisories/new)
on this repository. Reports are visible only to maintainers and create a
private fork for developing the fix.

Include the following:

1. **Description:** A clear description of the vulnerability, including affected components.
2. **Reproduction steps:** Minimal steps to reproduce, including any relevant contract addresses or transaction IDs.
3. **Impact assessment:** Your estimate of the exploitability and potential impact (funds at risk, scope, etc.).
4. **Suggested fix (optional):** Any ideas on how to remediate.

### Response timeline

| Stage | Target |
|---|---|
| Acknowledgement | Within 48 hours |
| Severity assessment | Within 5 business days |
| Fix timeline communicated | Within 7 business days |
| Critical fix deployed | Within 14 days of confirmed severity |

### Disclosure policy

We follow coordinated disclosure:
- We ask reporters to allow us 90 days to resolve the issue before public disclosure.
- We will publish a full incident report after resolution.
- Responsible reporters will be credited by name (or alias) in the security advisory, unless they prefer anonymity.

## Bug Bounty

A formal bug bounty program will be announced when Phase 2 contracts are deployed on mainnet. Until then, we offer:
- Public acknowledgement for confirmed vulnerabilities
- Invitation to the `security-contributors` GitHub team
- `SFORGE` token allocation for critical findings (discretionary, at core team's determination)

## Audit Reports

All completed audit reports are published in [`docs/audits/`](audits/) as they become available.

## Resolved Issues

Findings are recorded here once fixed. Both entries below were found during
internal review while the protocol was pre-deployment, so no funds were ever
at risk, and both are covered by regression tests.

### SF-2026-001 — Self-transfer minted tokens (critical)

`transfer` wrote the debit and the credit as two independent storage writes.
When `from == to` both resolved to the same `DataKey::Balance` entry, so the
credit overwrote the debit and the holder gained `amount` from nothing.
`total_supply` was left untouched, so the sum of balances silently diverged
from the reported supply, and the call could be repeated without limit.

`transfer_from` carried the identical flaw, reachable by any spender holding
an allowance.

*Fixed in `49e3087` and `d3252f7`. Regression tests:
`test_self_transfer_does_not_create_tokens`,
`test_self_transfer_from_does_not_create_tokens`, plus
`test_supply_invariant_holds_across_operations`, which asserts
`sum(balances) == total_supply` across a full operation sequence.*

### SF-2026-002 — Unextended storage TTL (high)

No entry point extended the time-to-live of the entries it wrote. Soroban
archives entries whose TTL lapses, and an archived entry cannot be read until
restored, so a holder who did not transact for long enough would have found
their balance unreadable and transfers involving them failing. The contract
instance, holding the admin address and pause flag, was subject to the same
expiry.

*Fixed in `848feda`. Policy is centralised in `contracts/common/src/storage.rs`
and shared by every Phase 1 contract. Note that `rwa-asset`'s read-only views
still do not extend, so a holder who is idle long enough can have their
balance archived; see [ADR-001](architecture/001-storage-key-design.md).*
