# ADR-006: Fractionalization Vault & Share Token Architecture

**Status:** Accepted
**Date:** 2026-09-18
**Authors:** StellarForge Core Team

---

## Context

Institutional Real World Assets (RWAs) often possess large unit valuations (e.g., commercial real estate, private credit facilities) that require fractionalization to facilitate broader investor participation and on-chain liquidity (PRD §5.4, FR-06).

However, tokenized vault design in Web3 suffers from critical security risks:
1. **Oracle Manipulation Threats (PRD §10.1):** Malicious actors manipulate oracle price feeds to deposit undervalued assets or redeem disproportionately large shares.
2. **ERC-4626 Inflation / Donation Attacks:** First-depositor attacks manipulate the internal exchange rate (share price) through direct asset donations to the vault contract.
3. **Regulatory Compliance and Lock-ups (PRD §11.4):** Securities laws (e.g., SEC Reg D holding periods) mandate strict transfer restrictions and accredited investor verification for all share movements.
4. **Historical Balance Queries for Yield (FR-07-2):** Fair dividend and yield distribution requires deterministic, historical balance verification at discrete snapshot ledgers.

Phase 1 contracts have no upgrade path (ADR-003). Therefore, the vault foundation and share token mechanics must be secure and immutable from deployment.

---

## Decision

### 1. The vault is its own SEP-41 share token
Each deployed vault contract wraps exactly one underlying `rwa-asset`. The vault contract implements the SEP-41 token interface (`balance`, `total_supply`, `allowance`, `approve`, `transfer`, `transfer_from`, `decimals`, `name`, `symbol`), with **one deliberate exception: there is no public `burn` or `burn_from`**. 

Redemption (#58) is the only valid mechanism by which shares leave circulation. Exposing a public `burn` would permanently strand underlying assets in the vault with no corresponding shares to redeem them, breaking the vault's core solvency invariant.

### 2. Fixed exchange rate, redeemed in kind
- Depositing `X` underlying tokens mints exactly `X × exchange_rate` shares (FR-06-2).
- Redeeming `Y` shares returns exactly `Y / exchange_rate` underlying tokens (FR-06-3).
- `exchange_rate` is a positive integer fixed at contract construction. Redemptions must be exact integer multiples of `exchange_rate`.

*Reasoning:* If deposit and redemption exchange rates are fixed and decoupled from variable market prices, oracle manipulation cannot extract value from the vault. This also inherently eliminates the ERC-4626 first-depositor inflation attack.

### 3. NAV is reported, never traded on
Net Asset Value (`nav()` and `nav_per_share()`) is reported by reading primary and secondary SEP-40 oracles (`oracle-adapter`, ADR-005) with staleness validation and divergence circuit breakers. However, `deposit` and `redeem` never query oracle feeds.

### 4. Internal accounting
`underlying_held` tracks verified contract deposits minus redemptions. Tokens sent directly to the vault contract without invoking `deposit` are treated as uncredited donations and cannot be redeemed.

### 5. Per-holder lock-up enforcement
Deposits set `locked_until = max(current, now + lockup_secs)`. While `now < locked_until(holder)`, the holder is strictly prohibited from redeeming or transferring shares (including via `transfer_from`).

### 6. Compliance screening on share movements
Every share transfer screens both the sender and recipient via `screen(subject, min_level)` (ADR-004). 
**The vault's own contract address must hold a valid KYC record** in the underlying asset's compliance contract so that deposit and redemption transfers succeed.

### 7. Balance checkpoints
Balance mutations route through a single internal write path (`write_balance`), allowing #60 to hook historical snapshots (`balance_at(holder, ledger)`) for `yield-distributor`.

### 8. Architectural conventions
- Deployment configuration via `VaultConfig` requiring `admin.require_auth()` (IR-17, ADR-002).
- Share `decimals` read immutably from the underlying token at construction.
- Emergency pause (`set_paused`) taking effect in the same ledger (NFR-R-2).
- Dual-authorization admin handover (`transfer_admin`, NFR-S-4).
- Storage TTL policies maintained via `stellarforge-common` (ADR-001).
- Frozen error discriminants (`VaultError` 1–15, ADR-003).

### Core Invariants
- `total_supply() == underlying_held() × exchange_rate` after every state transition.
- $\sum \text{balance}(\text{holder}) == \text{total\_supply}()$.
- `underlying_held() <= underlying.balance(vault)`.
- `total_supply() <= max_share_supply` when a supply cap is configured.
- No sequence of transactions permits transfers or redemptions while `now < locked_until(holder)`.
- Oracle outputs never influence deposit or redemption amounts.

---

## Alternatives Considered

1. **NAV-Priced Subscriptions and Redemptions:**
   *Rejected.* Exposing value-moving operations to oracle feeds re-introduces oracle manipulation vectors (§10.1).
2. **ERC-4626 Dynamic Pricing:**
   *Rejected.* Prone to rounding exploits, donation attacks, and requires fractional math approximations.
3. **A Separate Share-Token Contract:**
   *Rejected.* A two-contract architecture doubles cross-contract call hops, increases invocation gas overhead, and introduces split-state desynchronization risks.
4. **Lock-Up Lots Per Deposit:**
   *Rejected.* Tracking individual deposit lots requires unbounded storage vectors and variable gas costs. Rolling single-timestamp expiration (`max(current, now + lockup_secs)`) provides bounded $O(1)$ verification.
5. **Third-Party `deposit(to)`:**
   *Rejected.* Allowing arbitrary third-party deposit recipients would allow malicious actors to extend another holder's lock-up against their will.

---

## Consequences

- **Unredeemable Dust:** When `exchange_rate > 1`, share quantities that are not integer multiples of `exchange_rate` cannot be redeemed individually.
- **Stranded Direct Transfers:** Direct token transfers to the vault address do not increment `underlying_held` and cannot be recovered.
- **SEP-41 Deviation:** Deliberate omission of public `burn` and `burn_from` preserves backing solvency.
- **Vault KYC Requirement:** The vault contract address must be registered and verified in the underlying compliance contract before accepting deposits.
- **Lapsed-KYC Holders:** A holder whose KYC expires cannot transfer or redeem. Unlike `rwa-asset.burn` (which stays open for uncompliant holders), the vault cannot offer a burn without stranding underlying assets. Lapsed holders must renew compliance or undergo legal off-chain settlement.
