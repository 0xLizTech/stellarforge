# Recipe: Issue a Tokenized Bond (#209)

## Overview
This recipe walks through issuing a tokenized fixed-income bond using StellarForge.

## Workflow
1. **Deploy Asset**:
   Use `RwaAssetClient` to deploy a fixed-supply bond token with metadata (ISIN, maturity date, coupon rate).
2. **Apply Compliance**:
   Set accreditation check rules via `ComplianceClient` requiring verified KYC before transfers.
3. **Configure Vault**:
   Initialize fractionalization `VaultClient` with designated bond custody and deposit lock-up period.
4. **Distribute Shares**:
   Accept institutional investor USDC deposits in exchange for fractional bond shares.
