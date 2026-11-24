# ADR-008: Vault Share Transfer Restrictions for Accredited Investors

**Status:** Accepted
**Date:** 2026-11-24
**Authors:** StellarForge Core Team

---

## Context

Some RWA issuers are legally required to restrict who may hold fractionalized
vault shares — for example, SEC Regulation D (506c) offerings in the United
States limit beneficial ownership to accredited investors (PRD §11.2, §11.4).
While the underlying `rwa-asset` contract can enforce compliance on its own
transfers, secondary transfers of fractional vault shares (`vault::transfer`
and `vault::transfer_from`) move shares directly inside the `vault` contract
without invoking the underlying asset. Without share-level restrictions, a
non-accredited address could acquire fractional exposure to an accredited-only
asset on secondary markets.

Phase 2's `vault` contract (`contracts/vault`, ADR-006) already integrates
with the external `compliance` contract (ADR-004), which stores per-address
`KycRecord` entries with a numeric verification level (`0 = none`,
`1 = basic`, `2 = full`, `3 = accredited`), expiry timestamp, and ISO 3166-1
alpha-2 jurisdiction deny-list. Issue #251 asks us to evaluate the options for
enforcing accredited-investor transfer restrictions on vault shares using
these compliance levels, choose whether to implement additional contract
mechanisms or defer them, and record the decision.

---

## Options Evaluated (Using Compliance Levels)

### Option 1: Reuse `VaultConfig.min_compliance_level` with `compliance` Level 3 (`Accredited`)

Configure accredited-only vaults at deployment with:

- `compliance: Some(compliance_contract_id)`
- `min_compliance_level: 3` (`accredited`)

Under the existing vault implementation (`contracts/vault/src/lib.rs`), every
`deposit` screens `from` and every `transfer` / `transfer_from` screens both
`from` and `to` against `min_compliance_level` (returning
`VaultError::NotCompliant` on failure). Because `compliance` evaluates level,
expiry, and jurisdiction deny-lists together, setting `min_compliance_level = 3`
enforces accredited-investor-only transfers and deposits out of the box, with
zero additional storage or entry points.

| Compliance Level | Meaning | Suitable Vault Offering |
|---|---|---|
| `0` | None (or `compliance: None`) | Unrestricted / public fractional shares |
| `1` | Basic KYC | Retail-eligible regulated offerings |
| `2` | Full KYC | Enhanced-due-diligence offerings |
| `3` | Accredited Investor | Reg D (506c) and institutional-only vaults |

### Option 2: Vault-Local Allowlist or Split Deposit/Transfer Compliance Levels

Add a vault-specific allowlist (or separate `deposit_min_level` and
`transfer_min_level` fields) inside `contracts/vault`.

- **Pros:** Lets a single vault apply different thresholds to primary deposits
  versus secondary transfers, or maintain an issuer-specific holder list
  independent of the shared `compliance` contract.
- **Cons:** Duplicates KYC state across every deployed vault, increases
  storage and admin key surface, and breaks ADR-004's separation of concerns
  (revoking an investor in `compliance` would no longer be the single source
  of truth).

### Option 3: Non-Transferable (Soulbound) Shares or Post-Deploy Level Tightening

Add a `transfers_enabled: bool` flag (blocking all secondary transfers) or an
admin setter to raise `min_compliance_level` after deployment.

- **Pros:** Supports strictly non-transferable private placements or mid-life
  regulatory tightening.
- **Cons:** Raising `min_compliance_level` post-deployment can strand
  previously compliant holders (who cannot burn shares without stranding
  underlying assets; see ADR-006 *Lapsed-KYC Holders*). Non-transferable
  shares and tiered investor classes overlap with Phase 3 tranching, which is
  out of scope for Phase 2.

### Option 4: No Vault-Level Screening (Rely Only on Underlying `rwa-asset`)

Leave `compliance: None` on the vault and rely on the underlying `rwa-asset`
compliance hook.

- **Rejected:** Secondary transfers of vault shares never call the underlying
  `rwa-asset`, so non-accredited buyers could freely acquire fractional shares
  from an accredited depositor.

---

## Decision

**Adopt Option 1 and defer Options 2 and 3.**

1. **Accredited-investor share transfer restrictions are enforced through the
   existing `VaultConfig.compliance` + `min_compliance_level` mechanism**,
   using level `3` (`accredited`) for accredited-only vaults. No new vault
   storage keys, error variants, or admin entry points are added in Phase 2.
2. **Both sender and recipient are screened on every share transfer**
   (`transfer` and `transfer_from`), and the depositor is screened on
   `deposit`. Combined with `lockup_secs` (ADR-006), issuers can model Reg D
   holding periods plus accredited-only secondary transfers without custom
   contract code.
3. **Deferral criteria:** Vault-local allowlists, split deposit/transfer
   levels, non-transferable share modes, and tiered investor classes are
   deferred to Phase 3 (tranching / specialized issuer policies) unless an
   auditor or pilot issuer demonstrates a regulatory requirement that
   `compliance` levels `0..=3` plus jurisdiction deny-lists cannot express.

### Implementation Note

`contracts/vault/src/compliance.rs` currently calls `is_compliant` rather than
`screen` (which refreshes KYC record TTL in `contracts/compliance`). Aligning
vault screening to call `screen` is a separate operational improvement and is
intentionally kept out of scope of this decision record so existing client
interfaces and tests remain unchanged here.

---

## Alternatives Considered

- **Option 2 (Vault-local allowlist / split levels):** Rejected for Phase 2
  because it fragments compliance state and contradicts ADR-004.
- **Option 3 (Non-transferable shares / mutable compliance level):** Deferred
  to Phase 3 to avoid stranding existing holders and to keep Phase 2 vault
  semantics minimal and immutable (ADR-003, ADR-006).
- **Option 4 (Underlying-only screening):** Rejected because it leaves
  secondary share transfers unscreened.

---

## Consequences

- **Positive:** Issuers can launch accredited-only fractional vaults today by
  setting `compliance: Some(compliance_id)` and `min_compliance_level: 3` at
  construction, with zero contract size increase and full reuse of the
  existing `compliance` registry and jurisdiction deny-list.
- **Positive:** Revoking or expiring an investor's accreditation in
  `compliance` immediately blocks further vault deposits and share transfers
  across all vaults pointing to that compliance contract.
- **Negative / Operational:** As noted in [ADR-006](006-vault.md), the vault's
  own contract address must hold a valid KYC record in the underlying asset's
  compliance contract, and holders whose accreditation lapses cannot transfer
  shares until renewed.
- **Negative / Deferred:** Issuers requiring strictly non-transferable shares
  or per-vault holder caps must wait for Phase 3 extensions or deploy a
  custom compliance contract implementing `ComplianceInterface` (ADR-004).
