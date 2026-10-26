# StellarForge Integrator FAQ

Frequently asked questions for developers, issuers, and integrators building on StellarForge.

---

### Keys & Authorization

#### 1. How does admin rotation work?
StellarForge contracts implement a secure two-step admin transfer pattern (`propose_admin` followed by `accept_admin`) as detailed in ADR-006. The existing admin proposes a candidate address, and only that candidate's signature can claim the admin role. The pending transfer can be cancelled by the current admin anytime before acceptance.

#### 2. What signatures are required for token transfers?
Standard token transfers require the sender's `require_auth()`. In compliance-enabled mode, transfers also query the configured compliance contract to confirm KYC status and transfer limits for both sender and receiver.

#### 3. How do sponsored transactions work?
StellarForge SDK supports sponsored writes where an operator or gas-relayer account signs the outer transaction as fee-payer, while the user signs only the Soroban authorization tree for their specific action.

#### 4. Can a lost admin key be recovered?
Lost-key recovery is governed by guardian timelocks (ADR-006). If a key is lost without a designated guardian recovery setup, admin rights cannot be bypassed because no backdoors exist.

---

### Fees & Gas

#### 5. How are Soroban transaction fees calculated?
Soroban transactions incur two fee components: a standard network inclusion fee (in stroops) and a resource fee based on CPU instructions, memory footprint, and read/write storage bytes declared in the transaction footprint.

#### 6. Who pays the transaction fee during user transfers?
By default, the transaction source account pays the fee. Using the SDK's sponsored writes (`buildSponsoredTransferTx`), an application backend can act as the fee-payer and sponsor the user's gas costs.

#### 7. How can we optimize transaction fee costs?
Minimize footprint size by bundling operations where appropriate and avoid storing unbounded collections on-chain. Read queries (`read-only` contract calls) executed via RPC simulate execution and incur zero network fees.

---

### Storage, TTL & State Archival

#### 8. What storage types are used in StellarForge?
StellarForge uses **Instance storage** for core contract configuration and admin addresses, and **Persistent storage** for user balances, KYC records, and allowances. Ephemeral data uses **Temporary storage**.

#### 9. What is TTL and how does storage rent work?
Every ledger entry in Soroban has a Time-to-Live (TTL) measured in ledgers. To remain accessible, storage entries must maintain their TTL above the network threshold. The SDK and contracts automatically extend TTL during state interactions.

#### 10. What happens if a storage entry expires?
If a Persistent entry's TTL drops to 0, it becomes archived. The entry is not permanently destroyed; any user or contract can restore the archived entry by paying a restoration fee via a restoration transaction.

---

### Contract Upgrades & Security

#### 11. Can StellarForge contracts be upgraded on-chain?
Phase 1 contracts (`rwa-asset`, `compliance`, `registry`, `governance`, `oracle-adapter`) are non-upgradeable by design to eliminate upgrade proxy risk. Subsequent phases introduce modular registries where new contract instances can be registered without modifying immutable core logic.

#### 12. Is there an emergency pause mechanism?
Yes. The `rwa-asset` contract features an admin-controlled `set_paused(bool)` switch. When paused, minting, burning, and transferring are blocked, while query methods remain operational.

#### 13. Can an admin bypass compliance or freeze arbitrary funds?
Admins cannot bypass on-chain invariants. Asset freezes and clawbacks are strictly regulated: clawbacks must be enabled in asset metadata and require authorized issuer credentials following regulatory rules.

---

### Compliance & KYC

#### 14. How does compliance screening integrate with token transfers?
When an asset contract is linked to a compliance contract, every `transfer` and `transfer_from` triggers an internal call to the compliance contract verifying that sender and receiver satisfy the required `min_compliance_level` and jurisdiction rules.

#### 15. Can an asset operate without compliance screening?
Yes. Calling `set_compliance(admin, None, 0)` switches off external screening, allowing the token to function as a standard permissionless Soroban token.

---

### Standards & Interoperability

#### 16. Are StellarForge tokens compatible with SEP-41?
Yes. StellarForge RWA assets implement the standard token interface including allowances with expiration ledgers (`live_until_ledger`), balance queries, transfers, and burns.

#### 17. How does the Oracle Adapter support SEP-40?
The `oracle-adapter` contract implements the SEP-40 consumer interface (`price`, `prices`, `lastprice`, `decimals`, `resolution`), enabling seamless integration with existing Stellar DEX and lending protocols.

#### 18. How do I test contracts locally and on testnet?
You can run contract unit tests locally with `cargo test --all --features testutils`. For testnet, fund test accounts with Stellar Friendbot and run SDK integration tests using the RPC endpoint `https://soroban-testnet.stellar.org`.
