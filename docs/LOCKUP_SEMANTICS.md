# Lock-up Semantics for Transferred Shares (#224, #223)

## Policy Decision
1. **Lock-up Period**: Vault positions specify a `lockup_secs` upon deployment.
2. **Transfer Semantics**:
   - Transferred shares retain the original lock-up timestamp (`locked_until`).
   - If a recipient already holds shares, the latest lock-up expiration applies to the merged balance.
3. **Redemption Gate**:
   - `redeem()` and `redeemable()` check `env.ledger().timestamp() >= locked_until(holder)`.
   - Locked shares cannot be burned or redeemed until maturity.
