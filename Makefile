.PHONY: all build test fmt lint clean deploy-testnet help unused-deps unused-deps-selftest sdk-lint commitlint hooks

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
	$(STELLAR_CLI) contract build --manifest-path contracts/oracle-adapter/Cargo.toml
	@echo "Build complete. Artifacts in target/wasm32v1-none/release/"

test:
	@echo "Running contract tests..."
	$(CARGO) test --all --features testutils

fmt:
	@echo "Formatting Rust code..."
	$(CARGO) fmt --all

lint:
	@echo "Running Clippy..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Needs cargo-machete: cargo install cargo-machete --locked --version 0.7.0
unused-deps:
	@echo "Checking for unused Rust dependencies..."
	bash scripts/check-unused-deps.sh

unused-deps-selftest:
	bash scripts/check-unused-deps.sh --self-test

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

sdk-lint: sdk-install
	cd sdk && npm run lint

# ─── Git hooks and commit style ───────────────────────────────────────────────
# Installs the pre-commit hooks in .pre-commit-config.yaml (cargo fmt, eslint).
# Needs the pre-commit tool: pipx install pre-commit
hooks: sdk-install
	pre-commit install

# Lints the commits on this branch that are not on origin/main, as CI does.
commitlint: sdk-install
	cd sdk && npx --no-install commitlint --config ../commitlint.config.cjs --from origin/main --to HEAD --verbose

# ─── Deployment ───────────────────────────────────────────────────────────────
# scripts/deploy.sh builds, deploys all four contracts with their constructor
# arguments, and writes deployed-contracts.json. Deploying one wasm here with no
# constructor arguments could only ever fail.
deploy-testnet:
	@echo "Deploying all four contracts to Stellar testnet..."
	@echo "Ensure STELLAR_ACCOUNT env var is set to a funded testnet keypair."
	NETWORK=testnet ./scripts/deploy.sh

# ─── Help ─────────────────────────────────────────────────────────────────────
help:
	@echo ""
	@echo "StellarForge — Makefile targets"
	@echo "─────────────────────────────────────────"
	@echo "  make build          Build all Soroban WASM contracts"
	@echo "  make test           Run all Rust contract tests"
	@echo "  make fmt            Format Rust source code"
	@echo "  make lint           Run Clippy linter"
	@echo "  make unused-deps    Fail on unused Rust dependencies (cargo-machete)"
	@echo "  make clean          Remove build artifacts"
	@echo "  make sdk-build      Build the TypeScript SDK"
	@echo "  make sdk-test       Run SDK unit tests"
	@echo "  make sdk-lint       Run eslint on the SDK"
	@echo "  make hooks          Install the pre-commit hooks (cargo fmt, eslint)"
	@echo "  make commitlint     Lint this branch's commit messages against origin/main"
	@echo "  make deploy-testnet Deploy all four contracts to Stellar testnet"
	@echo ""
