# Vault Operations Guide

This guide covers operational procedures for deploying, configuring, monitoring, and operating the StellarForge Phase 2 fractionalization vault (`contracts/vault`) and its companion SEP-40 `oracle-adapter` contract on Stellar/Soroban.

---

## 1. Parameter Sizing & Arithmetic

### Choosing `exchange_rate`

Each vault wraps exactly one underlying `rwa-asset`. The `exchange_rate` parameter determines how many fractional shares are issued per 1 unit of underlying token:

$$\text{shares} = \text{underlying\_amount} \times \text{exchange\_rate}$$

For a commercial real-estate property tokenized with 7 decimals (where 1 token = 1 whole building or unit):
- **Underlying decimals:** 7 (`1.0000000` = `10_000_000` stroops / base units).
- **Target share sizing:** If 1 whole building represents \$10,000,000 and the goal is \$1.00 fractional retail shares:
  - Total underlying units = 1.
  - Shares to mint = 10,000,000.
  - Setting `exchange_rate = 1_000_000`: depositing `0.0000001` underlying yields 1 share (`1_000_000` base shares at 7 decimals).
- **Integer division on redeem:** Redemptions require that redeemed shares are a whole multiple of `exchange_rate`:
  $$\text{shares} \pmod{\text{exchange\_rate}} = 0$$
  This prevents rounding dust or fractional losses during redemption.

### Lock-Up Periods (`lockup_secs`)

- Lockups apply to newly deposited shares to satisfy regulatory seasoning periods (e.g., SEC Rule 144 restricted securities).
- Stored per holder as `locked_until(holder) = now + lockup_secs`.
- Subsequent deposits in the same position take `max(existing_lock, now + lockup_secs)`.
- Transfers and redemptions are blocked until `ledger.timestamp() >= locked_until(holder)`.

### Oracle Staleness (`max_staleness_secs`)

- Set `max_staleness_secs` relative to the external appraisal frequency.
- For quarterly commercial appraisals with daily automated adjustments: `max_staleness_secs = 172_800` (48 hours) provides a grace period for network delays while preventing stale NAV pricing.
- If primary and secondary feeds diverge beyond `max_divergence_bps` (e.g., 200 bps = 2%), NAV reads fail closed to protect shareholders.

---

## 2. KYC Requirements for Vault Address

In a regulated compliance architecture:
1. **Shareholders** must be KYC-screened before minting or transferring shares.
2. **The Vault Contract Address itself** holds underlying `rwa-asset` tokens in escrow.
3. Therefore, **the vault's contract address must be registered in the `compliance` contract** at or above the minimum required compliance level before any deposit can succeed.
4. Failure to KYC the vault contract address causes underlying token transfers to fail with `NotCompliant`.

---

## 3. Deterministic Deployment and Resumability

Deployments use deterministic salts:

$$\text{salt} = \text{sha256}(\text{DEPLOY\_NAMESPACE} \mathbin{\Vert} \text{contract\_name})$$

Pre-derivation allows planning all addresses before executing transactions:
```bash
STELLAR_ACCOUNT=deployer NETWORK=testnet DEPLOY_NAMESPACE=prod-2026 ./scripts/deploy.sh
```

If a deployment process terminates unexpectedly, re-running the command derives identical contract IDs, checks on-chain bytecode, verifies provenance, and resumes without duplicate deployments.

---

## 4. Bytecode Provenance Verification

Run the verification tool to confirm on-chain contract code against CI build checksums:

```bash
./scripts/verify-deployment.sh deployed-contracts.json target/wasm32v1-none/release/SHA256SUMS
```

The script prints a provenance report:
```
==========================================================================================
 CONTRACT         CONTRACT ID          EXPECTED HASH    ON-CHAIN HASH    RESULT  
------------------------------------------------------------------------------------------
 rwaAsset         CA3D...7B2A          6a8f12...        6a8f12...        MATCH   
 registry         CB91...1C8F          9b124e...        9b124e...        MATCH   
 compliance       CD84...44EF          e2810a...        e2810a...        MATCH   
 governance       CE52...01DA          71a4f0...        71a4f0...        MATCH   
 oracleAdapter    CF18...92BC          c4501d...        c4501d...        MATCH   
 vault            CG33...67DE          8821bc...        8821bc...        MATCH   
==========================================================================================
✓ Provenance verified: All on-chain contracts match build checksums.
```

---

## 5. Troubleshooting Reference

### VaultError Codes (`contracts/vault`)

| Code | Variant | Root Cause | Operator Action |
|---|---|---|---|
| 1 | `NotInitialized` | Contract constructor has not been called. | Re-run deployment via `deploy.sh`. |
| 2 | `InvalidExchangeRate` | `exchange_rate < 1`. | Provide `exchange_rate >= 1` in configuration. |
| 3 | `InvalidMaxSupply` | `max_share_supply < 0`. | Set to 0 (uncapped) or positive integer. |
| 4 | `InvalidLockup` | `lockup_secs > MAX_LOCKUP_SECS` (10 years). | Reduce lockup duration. |
| 5 | `InvalidMetadata` | Missing or empty name/symbol. | Provide non-empty string identifiers. |
| 6 | `InvalidUnderlying` | Target underlying contract failed decimals call. | Ensure underlying contract ID is deployed and implements SEP-41. |
| 7 | `ContractPaused` | Emergency pause active. | Call `set_paused(false)` after investigating condition. |
| 8 | `InvalidAmount` | Non-positive amount in operation. | Ensure amount > 0. |
| 9 | `InsufficientBalance` | Holder balance or vault underlying balance too low. | Check balances before submitting transfer or redemption. |
| 10 | `InsufficientAllowance` | Spender allowance insufficient. | Owner must call `approve` with sufficient allowance. |
| 11 | `Overflow` | Arithmetic operation exceeded `i128`. | Reduce transaction amount. |
| 12 | `InvalidExpiration` | Allowance expiration ledger has already passed. | Set `live_until_ledger` in future. |
| 13 | `NotCompliant` | Sender, recipient, or vault address lacks KYC. | KYC the address in the compliance contract. |
| 14 | `SharesLocked` | Current time is before `locked_until(holder)`. | Wait until lockup period elapses. |
| 15 | `ExceedsMaxSupply` | Deposit would breach configured `max_share_supply`. | Wait for redemptions or adjust cap. |
| 16 | `InvalidComplianceLevel` | Configured compliance level > 3. | Set compliance level between 0 and 3. |
| 17 | `NotMultipleOfExchangeRate` | Shares to redeem not a multiple of `exchange_rate`. | Redeem in multiples of exchange rate. |
| 18 | `OracleBaseMismatch` | Primary and secondary oracles report in different base currencies. | Align oracle base currencies (e.g. both USD). |
| 19 | `PriceUnavailable` | Oracle failed to return valid price. | Verify reporter has submitted price to feed. |
| 20 | `InvalidPrice` | Oracle reported price <= 0. | Correct appraisal feed. |
| 21 | `PriceInFuture` | Oracle report timestamp is ahead of ledger time. | Synchronize reporting clock. |
| 22 | `StalePrice` | Price age exceeds `max_staleness_secs`. | Publish updated appraisal report. |
| 23 | `OracleDivergence` | Feeds diverged beyond `max_divergence_bps`. | Investigate discrepancy between appraisal sources. |
| 24 | `IdenticalOracleSources` | Same oracle address configured for primary and secondary. | Configure distinct oracle contracts. |
| 25 | `OracleNotConfigured` | Oracle query called before `set_oracles`. | Admin must configure oracles. |
| 26 | `LedgerNotClosed` | Historical read requested for current or future ledger. | Query ledger sequence `< current_ledger`. |

### OracleError Codes (`contracts/oracle-adapter`)

| Code | Variant | Root Cause | Operator Action |
|---|---|---|---|
| 1 | `NotInitialized` | Contract not initialized. | Initialize with admin, base, decimals, resolution. |
| 2 | `InvalidDecimals` | Decimals <= 0 or > 18. | Provide valid decimals parameter. |
| 3 | `InvalidResolution` | Resolution <= 0. | Set resolution in seconds (e.g. 86400). |
| 4 | `UnknownAsset` | Asset not registered in feed. | Admin must call `add_asset`. |
| 5 | `AssetAlreadyExists` | Duplicate asset registration. | Use existing asset symbol or address. |
| 6 | `TooManyAssets` | Asset limit reached. | Remove obsolete assets or use separate adapter. |
| 7 | `NotReporter` | Sender is not authorized reporter. | Admin must call `set_reporter`. |
| 8 | `InvalidPrice` | Reported price <= 0. | Submit positive price value. |
| 9 | `TimestampInFuture` | Report timestamp in future. | Synchronize system clock. |
| 10 | `StaleReport` | Report tick is not later than latest recorded tick. | Wait for next resolution period. |
| 11 | `DeviationTooLarge` | Price change exceeds `max_deviation_bps`. | Use admin `override_price` for major revaluations. |
| 12 | `InvalidDeviation` | `max_deviation_bps` <= 0 or > 10,000. | Set deviation between 1 and 10,000 bps. |
| 13 | `Overflow` | Integer overflow during calculation. | Validate integer bounds. |
