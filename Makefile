.PHONY: all build test fmt lint clean sdk-install sdk-build sdk-test sdk-lint sdk-typecheck deploy-testnet help

# ─── Variables ────────────────────────────────────────────────────────────────
NETWORK        ?= testnet
STELLAR_CLI    := stellar
CARGO          := cargo

# ─── Default ──────────────────────────────────────────────────────────────────
all: fmt lint test build sdk-lint sdk-typecheck sdk-test sdk-build ## Run formatters, linters, tests, and builds across contracts and SDK

##@ Contracts
build: ## Build all Soroban WASM contracts
	@echo "Building all Soroban contracts..."
	$(STELLAR_CLI) contract build --manifest-path contracts/rwa-asset/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/registry/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/compliance/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/governance/Cargo.toml
	$(STELLAR_CLI) contract build --manifest-path contracts/oracle-adapter/Cargo.toml
	@echo "Build complete. Artifacts in target/wasm32v1-none/release/"

test: ## Run all Rust contract tests
	@echo "Running contract tests..."
	$(CARGO) test --all --features testutils

fmt: ## Format Rust source code
	@echo "Formatting Rust code..."
	$(CARGO) fmt --all

lint: ## Run Clippy linter
	@echo "Running Clippy..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf sdk/dist sdk/node_modules

##@ SDK
sdk-install: ## Install SDK dependencies
	cd sdk && npm ci

sdk-build: sdk-install ## Build the TypeScript SDK
	cd sdk && npm run build

sdk-test: sdk-install ## Run SDK unit tests
	cd sdk && npm test

sdk-lint: sdk-install ## Run ESLint on the SDK
	cd sdk && npm run lint

sdk-typecheck: sdk-install ## Run TypeScript typecheck on the SDK
	cd sdk && npm run typecheck

##@ Release
deploy-testnet: ## Deploy all four contracts to Stellar testnet
	@echo "Deploying all four contracts to Stellar testnet..."
	@echo "Ensure STELLAR_ACCOUNT env var is set to a funded testnet keypair."
	NETWORK=testnet ./scripts/deploy.sh

##@ Help
help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"} \
		/^##@/ { printf "\n%s:\n", substr($$0, 5) } \
		/^[a-zA-Z_-]+:.*?##/ { printf "  %-18s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
