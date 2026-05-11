.PHONY: all build test fmt lint clean deploy-testnet help

# ─── Variables ────────────────────────────────────────────────────────────────
NETWORK        ?= testnet
STELLAR_CLI    := stellar
CARGO          := cargo

# ─── Default ──────────────────────────────────────────────────────────────────
all: fmt lint test build

# ─── Rust Contracts ───────────────────────────────────────────────────────────
build:
	@echo "Building all Soroban contracts..."
	$(STELLAR_CLI) contract build --manifest-path contracts/rwa-asset/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/registry/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/compliance/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/governance/Cargo.toml
	@echo "Build complete. Artifacts in target/wasm32-unknown-unknown/release/"

test:
	@echo "Running contract tests..."
	$(CARGO) test --all --features testutils

fmt:
	@echo "Formatting Rust code..."
	$(CARGO) fmt --all

lint:
	@echo "Running Clippy..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

clean:
	$(CARGO) clean
	rm -rf sdk/dist sdk/node_modules

# ─── SDK ──────────────────────────────────────────────────────────────────────
sdk-install:
	cd sdk && npm ci

sdk-build: sdk-install
	cd sdk && npm run build

sdk-test: sdk-install
	cd sdk && npm test

# ─── Deployment ───────────────────────────────────────────────────────────────
deploy-testnet: build
	@echo "Deploying to Stellar testnet..."
	@echo "Ensure STELLAR_ACCOUNT env var is set to a funded testnet keypair."
	$(STELLAR_CLI) contract deploy \
		--wasm target/wasm32-unknown-unknown/release/rwa_asset.wasm \
		--network $(NETWORK) \
		--source $(STELLAR_ACCOUNT)

# ─── Help ─────────────────────────────────────────────────────────────────────
help:
	@echo ""
	@echo "StellarForge — Makefile targets"
	@echo "─────────────────────────────────────────"
	@echo "  make build          Build all Soroban WASM contracts"
	@echo "  make test           Run all Rust contract tests"
	@echo "  make fmt            Format Rust source code"
	@echo "  make lint           Run Clippy linter"
	@echo "  make clean          Remove build artifacts"
	@echo "  make sdk-build      Build the TypeScript SDK"
	@echo "  make sdk-test       Run SDK unit tests"
	@echo "  make deploy-testnet Deploy rwa-asset to Stellar testnet"
	@echo ""
