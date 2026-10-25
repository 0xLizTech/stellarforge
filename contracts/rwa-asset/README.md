# rwa-asset

The core Real World Asset tokenization contract. A SEP-41 fungible token with
an issuer role for minting, a two-step admin handover, optional compliance
screening, versioned metadata, and a supply cap that the admin can tighten
alone but can only loosen with a second, separately configured signature.

See the root [README](../../README.md#rwaasset) for the full project context,
and [ADR-001](../../docs/architecture/001-storage-key-design.md) (storage
keys), [ADR-002](../../docs/architecture/002-auth-patterns.md) (auth
patterns) and [ADR-007](../../docs/architecture/007-governance-controlled-supply-cap.md)
(supply cap governance) for why specific entry points work the way they do.

## Entry points

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin, metadata)` | admin | Runs at deploy; not callable afterwards |
| `mint(issuer, to, amount)` | issuer | Create new tokens, subject to the supply cap and compliance screening |
| `burn(from, amount)` | from | Destroy tokens |
| `burn_from(spender, from, amount)` | spender | Destroy tokens, spending an allowance |
| `transfer(from, to, amount)` | from | Move tokens; `to` may be a muxed address (SEP-41) |
| `approve(owner, spender, amount, live_until_ledger)` | owner | Set an allowance that reads as zero after `live_until_ledger` |
| `transfer_from(spender, from, to, amount)` | spender | Spend an allowance |
| `set_issuer(issuer, approved)` | admin | Grant/revoke the issuer role |
| `set_paused(paused)` | admin | Emergency circuit breaker |
| `update_metadata(metadata)` | admin | Replace metadata; `decimals` is fixed, and a cap can only be tightened, never raised, removed or set below supply |
| `propose_admin(new_admin)` / `accept_admin()` / `cancel_admin_transfer()` | admin / new_admin / admin | Two-step admin handover |
| `set_compliance(compliance, min_level)` | admin | Point `mint`/`transfer`/`transfer_from` at a compliance contract, or `None` to disable screening |
| `set_governance(governance)` | admin | Name the address that may approve loosening a supply cap, or `None` to disable loosening |
| `propose_cap_increase(new_max_supply)` | admin | Propose raising (or removing) the supply cap |
| `approve_cap_increase()` | governance | Finalize a pending cap increase; applies it through the same path as `update_metadata` |
| `cancel_cap_increase()` | admin | Withdraw a pending cap increase before it is approved |
| `set_forced_transfer_role(account, approved)` | admin | Grant/revoke the forced-transfer role (#144) |
| `forced_transfer(operator, from, to, amount, reason_hash)` | forced-transfer role | Move tokens without `from`'s authorization, for court-ordered recovery; not blocked by `set_paused` |
| `set_holder_cap(cap)` | admin | Set an optional ceiling on the number of distinct holders (Reg D–style); `None` removes the cap (#146) |
| `holder_count()` | — | Number of addresses currently holding a strictly positive balance (#146) |
| `holder_count_cap()` | — | The configured holder ceiling, or `None` if uncapped (#146) |
| `balance(owner)`, `allowance(owner, spender)`, `total_supply()`, `metadata()`, `metadata_version()`, `decimals()`, `name()`, `symbol()`, `admin()`, `pending_admin()`, `is_issuer(address)`, `paused()`, `compliance_contract()`, `min_compliance_level()`, `governance()`, `pending_cap_increase()`, `is_forced_transfer_role(address)` | — | Read-only queries |

## Errors (`RwaError`)

| Code | Variant | Raised by |
|---|---|---|
| 1 | `AlreadyInitialized` | Reserved; unreachable since `initialize` was replaced by the constructor |
| 2 | `NotInitialized` | Any getter, if ever called before construction |
| 3 | `NotIssuer` | `mint`, when the caller is not a registered issuer |
| 4 | `InvalidAmount` | `mint`/`burn`/`burn_from`/`transfer`/`transfer_from`/`approve`, on a zero or negative amount |
| 5 | `InsufficientBalance` | `burn`/`burn_from`/`transfer`/`transfer_from` |
| 6 | `InsufficientAllowance` | `transfer_from`/`burn_from`, spending more than approved |
| 7 | `ExceedsMaxSupply` | `mint`, past the supply cap |
| 8 | `ContractPaused` | Any state-changing call while `set_paused(true)` |
| 9 | `Overflow` | Any checked arithmetic operation overflowing `i128`/`u32` |
| 10 | `InvalidMetadata` | `update_metadata`/constructor, on malformed metadata (decimals, empty name/symbol, negative cap) |
| 11 | `NotCompliant` | `mint`/`transfer`/`transfer_from`, when a party fails screening |
| 12 | `DecimalsImmutable` | `update_metadata`, attempting to change `decimals` |
| 13 | `InvalidSupplyCap` | `update_metadata`/`propose_cap_increase`, on a cap change that isn't a valid tighten/loosen |
| 14 | `InvalidExpiration` | `approve`, on a non-zero allowance whose `live_until_ledger` has already passed |
| 15 | `InvalidComplianceLevel` | `set_compliance`, on a `min_level` above `MAX_COMPLIANCE_LEVEL` |
| 16 | `InvalidLegalDocHash` | constructor/`update_metadata`, when `legal_doc_hash` is not exactly 32 bytes or is all-zero |
| 17 | `GovernanceNotConfigured` | `approve_cap_increase`, when `set_governance` was never called |
| 18 | `NoPendingCapIncrease` | `approve_cap_increase`/`cancel_cap_increase`, with nothing proposed |
| 19 | `NoPendingAdminTransfer` | `accept_admin`/`cancel_admin_transfer`, with nothing proposed |
| 20 | `NotForcedTransferRole` | `forced_transfer`, when the caller does not hold the forced-transfer role |
| 21 | `InvalidReasonHash` | `forced_transfer`, on an empty `reason_hash` |
| 22 | `InvalidForcedTransferParty` | Reserved; not currently raised (`forced_transfer` treats a same-address call as a no-op, like `transfer`) |
| 23 | `AddressFrozen` | `transfer`, `transfer_from`, when the sender's address is frozen |
| 24 | `AddressLocked` | `transfer`, `transfer_from`, when the sender's lock-up period has not yet passed |
| 25 | `ExceedsHolderCap` | `mint`, `transfer`, `transfer_from`, `forced_transfer`, when a holder cap is configured and adding a new holder would exceed it |

Discriminants are frozen once a contract using them is deployed (ADR-003):
existing values never change meaning, and new ones are only ever appended.

## Events

| Event | Published by |
|---|---|
| `Transfer`, `TransferFrom` | `transfer`, `transfer_from` |
| `Mint`, `Burn` | `mint`, `burn`/`burn_from` |
| `Approve` | `approve` |
| `Paused` | `set_paused` |
| `IssuerSet` | `set_issuer` |
| `MetadataUpdated` | `update_metadata` and `approve_cap_increase`; carries the previous and new metadata in full and the new `metadata_version` (#149) |
| `AdminTransferProposed`, `AdminTransferred`, `AdminTransferCancelled` | `propose_admin`, `accept_admin`, `cancel_admin_transfer` |
| `ComplianceSet` | `set_compliance` |
| `GovernanceSet` | `set_governance` (#148) |
| `CapIncreaseProposed` | `propose_cap_increase` (#148) |
| `CapIncreaseCancelled` | `cancel_cap_increase` (#148) |
| `ForcedTransfer` | `forced_transfer`; carries `reason_hash` so a court-ordered move is auditable against the order that justified it (#144) |
| `FrozenSet` | `set_frozen` |
| `LockupDateSet` | `set_lockup_date` |
| `HolderCapSet` | `set_holder_cap`; `cap` is `None` when the cap was removed (#146) |

## Legal document hash

`AssetMetadata::legal_doc_hash` must be the raw 32-byte SHA-256 digest (not
hex, not an IPFS CID) of the off-chain legal document the asset represents.
An all-zero or wrong-length value is rejected at both construction and
`update_metadata` (#150) — this proves the field is *present and the right
shape*, not that the document it names is genuine; verifying that remains an
off-chain, out-of-band step.
