# StellarForge Glossary

This glossary defines key terms used throughout StellarForge's contracts, SDK, and documentation, spanning Real World Assets (RWA), Soroban smart contracts, and the Stellar network.

---

### A

- **Address**: A Stellar public key (`G...`) or Soroban contract identifier (`C...`) representing an account, signer, or contract instance.
- **Allowance**: The pre-authorized amount of an asset that an owner permits a designated spender to transfer or burn on their behalf, typically scoped with an expiration ledger.
- **Asset Code**: An alphanumeric identifier (1–12 characters) naming a Stellar token (e.g., `USD`, `TBILL`).
- **Asset Issuer**: The administrative identity authorized to mint, burn, or manage compliance for a tokenized asset.
- **Attestation**: Cryptographic or regulatory proof confirming off-chain facts, such as audit results, reserve backing, or KYC status.
- **Auth Tree (Authorization Tree)**: The hierarchical structure of authorizations signed by accounts for Soroban contract invocations, including nested sub-invocations.

### B

- **Basis Points (bps)**: A unit equal to 1/100th of 1% (0.0001). Commonly used for percentage parameters, fee calculations, and deviation boundaries (e.g., 5,000 bps = 50%).
- **Burn**: The permanent destruction and reduction of circulating token supply by the owner, authorized spender, or protocol admin.

### C

- **Clawback**: An administrative regulatory capability allowing an authorized issuer to revoke and reclaim tokens from an account due to legal, fraud, or sanction actions.
- **Compliance Contract**: A Soroban contract implementing identity verification, sanction checks, transfer rules, and regulatory permissions for RWA assets.
- **Compliance Level**: A numerical integer tier (e.g., Tier 1, Tier 2) representing the verification status of an account (e.g., accredited, qualified institutional, retail).
- **Contract Data**: State stored on-chain by a Soroban contract, categorized into Persistent, Temporary, or Instance storage.

### D

- **Decimals**: The number of digits of precision following the decimal point for asset units (commonly 7 on Stellar, or standard 18).
- **Deposit**: Locking underlying collateral or base currency into a contract (such as a Vault) in exchange for newly minted derivative shares.
- **Deviation Guard**: A safeguard preventing an oracle price report from altering net asset value by more than a configured percentage threshold (`max_deviation_bps`) in a single tick.

### E

- **Escrow**: A smart contract or custody arrangement where funds are held until predefined programmatic conditions are met.
- **Event**: An append-only log record emitted by a contract invocation containing topic symbols and data payloads for indexers and subscribers.
- **Expiration Ledger**: The ledger sequence number after which an allowance, proposal, or lockup period ceases to be valid.

### F

- **Footprint**: The set of ledger keys that a Soroban transaction reads or writes, declared prior to execution to enable parallel transaction validation.
- **Freeze / Pause**: An administrative intervention halting token transfers, minting, or contract interactions during security incidents or regulatory holds.
- **Friendbot**: A testnet faucet service providing free test lumens (XLM) to fund developer accounts.

### H

- **Horizon**: The REST API client server for indexing and querying Stellar ledger transactions, accounts, and payments.

### I

- **Instance Storage**: Contract storage tied directly to the contract instance executable and metadata, sharing its lifecycle.
- **Invariant**: A fundamental system property that must remain strictly true before and after every valid transaction (e.g., total supply equals the sum of account balances).
- **Issuer**: The administrative address responsible for minting, token configuration, and asset governance.

### J

- **Jurisdiction**: A geographic or regulatory region identifier (e.g., ISO country code) used by compliance modules to filter allowed or restricted counterparties.

### K

- **Keypair**: A cryptographic public-private key pair (Ed25519) used to authenticate actions and sign transactions.
- **KYC (Know Your Customer)**: The regulatory due diligence process verifying the identity and legal status of token holders and transaction counterparties.

### L

- **Ledger**: The authoritative state of the Stellar blockchain, updated periodically (typically every 5 seconds) through the Stellar Consensus Protocol.
- **Ledger Sequence**: The monotonically increasing integer identifier assigned to each closed ledger block.
- **Live Until Ledger**: The ledger sequence up to which an allowance or authorization is active.

### M

- **Maximum Supply (Supply Cap)**: The strictly enforced ceiling on the total quantity of tokens that can ever exist for an asset.
- **Memo**: An optional 28-byte data field, hash, or integer attached to a Stellar transaction for invoice tracking or routing.
- **Mint**: The creation of new tokens by an authorized issuer or vault, increasing total supply.

### N

- **Net Asset Value (NAV)**: The evaluated per-unit monetary valuation of the collateral, fund, or real-world assets backing a token.
- **Nonce**: An incremental counter ensuring each transaction or approval can only be executed once, preventing replay attacks.

### O

- **Oracle**: An off-chain data reporting system providing real-time pricing, benchmark rates, or state updates to smart contracts.
- **Oracle Adapter**: A contract adapting external market feeds into Soroban standard formats (e.g., SEP-40).

### P

- **Persistent Storage**: On-chain Soroban storage retained indefinitely as long as its rent / TTL is periodically refreshed before expiration.
- **Proposal**: A formal governance action submitted for vote, containing targets, parameters, and executable actions.
- **Provenance**: Cryptographic proof certifying the origin, build environment, and source code commit of published packages and binaries.

### Q

- **Quorum**: The minimum total vote participation required for a governance proposal to be considered valid and eligible for finalization.

### R

- **Real World Asset (RWA)**: Tangible or financial assets existing off-chain (e.g., real estate, treasury bills, private credit, carbon credits) tokenized on-chain.
- **Registry**: The master on-chain directory mapping asset addresses, symbols, compliance modules, and issuer metadata.
- **Resolution**: The granularity time interval (in seconds) between sequential oracle updates.
- **Root Invocation**: The top-level function call in a transaction call tree from which sub-contract calls descend.

### S

- **ScVal (Soroban Value)**: The unified binary serialization format representing values (integers, symbols, addresses, maps, vectors) in Soroban.
- **SEP (Stellar Ecosystem Proposal)**: Public architectural standards guiding integrations across the Stellar network.
- **SEP-40**: The standard interface specification for Soroban oracle and price feed contracts.
- **SEP-41**: The standard interface specification for fungible token smart contracts on Soroban.
- **Soroban**: The Rust-based, WASM smart contract execution environment integrated into the Stellar blockchain.
- **Sponsored Transaction**: A transaction pattern where an operator account covers fees and network reserves on behalf of an end-user.
- **Stroop**: The smallest atomic divisible unit of Stellar Lumens (XLM), equal to 0.0000001 XLM (10^-7).

### T

- **Temporary Storage**: Low-cost Soroban storage intended for ephemeral data that automatically clears upon reaching its expiration ledger.
- **Time-to-Live (TTL)**: The remaining number of ledgers before a contract instance or persistent storage key becomes inactive.
- **Tokenization**: The legal and technical process of representing real-world asset rights and claims as cryptographic tokens.
- **Tranche**: A segregated risk or yield division within a structured asset or debt portfolio.

### V

- **Vault**: A smart contract managing pooled investor capital, collateral custody, and yield distribution according to calculated NAV.
- **Veto**: The emergency governance authority allowing an assigned guardian or admin to reject an unsafe proposal before execution.

### W

- **WASM (WebAssembly)**: The portable binary-code format compiled from Rust that Soroban executes deterministically in its runtime environment.
