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

Findings are recorded here once fixed. Every entry below was found during
internal review while the protocol was pre-deployment, so no funds were ever
at risk, and each is covered by regression tests. SF-2026-003 onward come from
the [Phase 1 internal security review](audits/2026-09-14-internal-review-phase1.md),
whose IDs are given in brackets.

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
and shared by every Phase 1 contract: write paths extend to the network maximum
TTL, read paths stay pure. Since protocol 23 an archived entry is restored
automatically by the transaction that touches it, so the residual exposure is a
restore fee for a holder idle longer than the maximum TTL, not a failed
transfer. State rent means that exposure cannot be removed, only reassigned;
[ADR-001](architecture/001-storage-key-design.md) records who pays and why.*

### SF-2026-003 — Registry directory stopped accepting assets (medium) [IR-01]

The directory index was a single `Vec<Address>` held in one persistent entry,
and every `register` rewrote the whole vector. Each address adds 40 bytes, so
at about 1,600 assets the entry crossed the 64 KiB contract-data limit. From
then on every `register` failed, and without an upgrade path (ADR-003) nothing
could recover the contract in place. NFR-P-3 requires 10,000 assets.

*Fixed in `70b761a`. Each asset now has its own `AssetAt(index)` entry, and
`list_assets(start, limit)` pages at most 100 addresses per call. Regression
tests: `test_directory_scales_past_the_former_entry_size_ceiling` (2,000
assets, under mainnet limits), and `test_directory_scales_to_the_nfr_p_3_size`
(10,000 assets, run with `--ignored`).*

### SF-2026-004 — `update_metadata` could re-denominate or uncap an asset (medium) [IR-02]

`update_metadata` checked the new metadata only in isolation. The admin could
change `decimals` after issuance, which resizes every holder's position by a
power of ten, lift a cap back to uncapped, or set a cap below circulating
supply. None of these changes emitted an event.

*Fixed in `a1c73c2`. `decimals` is now immutable. A cap can move, but never
below `total_supply` and never back to 0. Regression tests:
`test_update_metadata_cannot_change_decimals`,
`test_update_metadata_cannot_lift_a_cap`,
`test_update_metadata_cannot_cap_below_circulating_supply`.*

### SF-2026-005 — Privileged state changes emitted no events (medium) [IR-03]

Only holder operations and `set_paused` published events. Admin handover,
issuer grants, metadata changes, disabling compliance, KYC decisions, registry
changes and every governance action left no trace for monitors, which
violated NFR-A-1 and NFR-A-3.

*Fixed in `8147f6d`. Every state-changing entry point now publishes an
event, and each wire format is pinned by a test in its contract's suite.*

### SF-2026-006 — Three contracts could not rotate their admin (medium) [IR-04]

Only `rwa-asset` exposed `transfer_admin`. The admin of `compliance`,
`registry` and `governance` was fixed at deployment for good. A compromised
compliance key could mark any address permanently compliant on every
dependent asset, and could never be rotated out.

*Fixed in `2021ed5`. All four contracts now have dual-authorization
`transfer_admin`. Regression tests in each:
`test_transfer_admin_without_the_incoming_signature_is_rejected`, plus a
test showing that a rotated-out admin loses its powers.*

### SF-2026-007 — Two write paths skipped the TTL policy (low) [IR-07]

`update_metadata` and `transfer_admin` wrote storage without extending its
TTL. That is the SF-2026-002 policy, missed in two places. Since protocol 23
the consequence is a restore fee rather than a failure.

*Fixed in `a1c73c2`. Regression test:
`test_admin_write_paths_extend_what_they_write`.*

### SF-2026-008 — SDK could report a write as failed while it could still land (medium) [IR-05]

Writes are built with a 180-second validity window, but the SDK stopped polling
after about 30 seconds and reported a still-pending transaction as failed. A
caller that retried built a new transaction while the original could still be
included, so a mint or transfer could execute twice. `TRY_AGAIN_LATER` and
`DUPLICATE` submissions were also treated as pending.

*Fixed in `db1677e`. Polling now covers the validity window plus 30 seconds. An
unsettled transaction ends in `TransactionExpiredError`, which is safe to
retry, or `TransactionOutcomeUnknownError`, which carries the hash to check
first. Regression tests are in `sdk/tests/client.test.ts`, under "RwaAssetClient
write submission".*

### SF-2026-009 — SDK helpers silently misread amounts and hashes (medium) [IR-06]

`toStroops` read `"1.2.3"` as 1.2 and `"0x10"` as hexadecimal, truncated
precision beyond the asset's decimals, and accepted numbers that had already
lost precision. `fromStroops` rendered 100 whole tokens at zero decimals as
`"0.1"`. `hexToBytes32` turned non-hex digits into zero bytes. Any of these
could put a wrong amount into a transfer, or a wrong document hash on chain.

*Fixed in `c0c0543`. The helpers now reject malformed input instead of guessing at
it. Regression tests are in `sdk/tests/utils.test.ts`.*

### SF-2026-010 to SF-2026-019 — Low and Info findings [IR-08 to IR-17]

None of these gave a path to funds. Each is fixed, and the
[internal review](audits/2026-09-14-internal-review-phase1.md) records the
detail and the regression tests.

| ID | Review | Finding | Fixed in |
|---|---|---|---|
| SF-2026-010 | IR-08 | `set_kyc` accepted out-of-range levels, past expiries and malformed jurisdictions (low) | `e01ff66` |
| SF-2026-011 | IR-09 | The token surface diverged from SEP-41, and allowances never expired (low) | `0e7b905` |
| SF-2026-012 | IR-10 | The Rust toolchain was unpinned, so builds were not reproducible (low) | `d1f0ad7` |
| SF-2026-013 | IR-11 | Dependency review never ran, and the SDK's npm dependencies were never audited (low) | `d1f0ad7` |
| SF-2026-014 | IR-12 | CI trusted a mutable action tag and an unverified CLI download (low) | `d1f0ad7` |
| SF-2026-015 | IR-13 | `deploy.sh` would deploy placeholder metadata to mainnet (low) | `0c91166` |
| SF-2026-016 | IR-14 | `propose` accepted a zero voting period and unbounded titles (low) | `2f736c0` |
| SF-2026-017 | IR-15 | A self-`transfer_from` emitted a phantom `Transfer` event (info) | `0e7b905` |
| SF-2026-018 | IR-16 | Registry entries were not documented as unverified (info) | `4f04c60` |
| SF-2026-019 | IR-17 | Constructor `require_auth` was asserted by no test (info) | `d9fc624` |
