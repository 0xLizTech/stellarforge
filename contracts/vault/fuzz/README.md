# Vault Fuzzing Harness

This directory contains `cargo-fuzz` targets for exploring arbitrary call sequences on the Phase 2 Vault contract.

## Prerequisites

- Nightly Rust compiler
- `cargo install cargo-fuzz`

## Running the fuzzer

```bash
cd contracts/vault/fuzz
cargo +nightly fuzz run vault_fuzz
```

## Seed Corpus

Initial seeds in `corpus/vault_fuzz` provide valid sequences of deposit, transfer, redeem, and allowance operations.

## Invariants Tested

- `total_supply == underlying_held * exchange_rate`
- sum of user balances == total supply
- Non-negative balances and supplies
