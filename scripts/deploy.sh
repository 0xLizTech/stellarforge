#!/usr/bin/env bash
# deploy.sh — Deploy all StellarForge Phase 2 contracts to a Stellar network with deterministic IDs and resumability.
#
# Usage:
#   STELLAR_ACCOUNT=mykey NETWORK=testnet ./scripts/deploy.sh
#
# Every contract takes its admin as a constructor argument, so deployment and
# configuration happen in one transaction and there is no window in which a
# contract exists unowned. The admin is the deploying account unless ADMIN is
# set to something else.
#
# Salts are derived deterministically as sha256(DEPLOY_NAMESPACE || contract_name),
# ensuring reproducible addresses across runs. Existing contracts are verified
# against build checksums and safely skipped rather than redeployed.
#
# Requires: stellar-cli >= 27.0.0, jq, shasum
#
# NETWORK=mainnet refuses to run unless ADMIN, DEPLOY_NAMESPACE, and all ASSET_*,
# ORACLE_*, and VAULT_* variables are explicitly set.

set -euo pipefail

NETWORK="${NETWORK:-testnet}"
ACCOUNT="${STELLAR_ACCOUNT:?Set STELLAR_ACCOUNT to a funded keypair alias}"
DEPLOY_NAMESPACE="${DEPLOY_NAMESPACE:-stellarforge-phase2}"
OUTPUT_FILE="deployed-contracts.json"

# Phase 1 Asset Configuration
ASSET_NAME="${ASSET_NAME:-StellarForge Demo Asset}"
ASSET_SYMBOL="${ASSET_SYMBOL:-SF-DEMO}"
ASSET_DECIMALS="${ASSET_DECIMALS:-7}"
ASSET_CLASS="${ASSET_CLASS:-real_estate}"
ASSET_LEGAL_DOC_HASH="${ASSET_LEGAL_DOC_HASH:-$(printf '00%.0s' {1..32})}"
ASSET_MAX_SUPPLY="${ASSET_MAX_SUPPLY:-0}"

# Phase 2 Oracle Adapter Configuration
ORACLE_BASE="${ORACLE_BASE:-USD}"
ORACLE_DECIMALS="${ORACLE_DECIMALS:-14}"
ORACLE_RESOLUTION="${ORACLE_RESOLUTION:-86400}"

# Phase 2 Vault Configuration
VAULT_EXCHANGE_RATE="${VAULT_EXCHANGE_RATE:-1000000}"
VAULT_LOCKUP_SECS="${VAULT_LOCKUP_SECS:-0}"
VAULT_MAX_SHARE_SUPPLY="${VAULT_MAX_SHARE_SUPPLY:-0}"
VAULT_MAX_STALENESS_SECS="${VAULT_MAX_STALENESS_SECS:-172800}"
VAULT_NAME="${VAULT_NAME:-StellarForge Vault Share}"
VAULT_SYMBOL="${VAULT_SYMBOL:-vSF-DEMO}"

# Mainnet Guard: Refuse to run unless all operational variables are set explicitly
if [[ "$NETWORK" == "mainnet" ]]; then
  missing=()
  for var in ADMIN ASSET_NAME ASSET_SYMBOL ASSET_DECIMALS ASSET_CLASS \
    ASSET_LEGAL_DOC_HASH ASSET_MAX_SUPPLY ORACLE_BASE ORACLE_DECIMALS \
    ORACLE_RESOLUTION VAULT_EXCHANGE_RATE VAULT_LOCKUP_SECS \
    VAULT_MAX_SHARE_SUPPLY VAULT_MAX_STALENESS_SECS VAULT_NAME \
    VAULT_SYMBOL DEPLOY_NAMESPACE; do
    [[ -n "${!var:-}" ]] || missing+=("$var")
  done
  if ((${#missing[@]} > 0)); then
    echo "error: a mainnet deploy requires these to be set explicitly: ${missing[*]}" >&2
    exit 1
  fi
  if [[ ! "$ASSET_LEGAL_DOC_HASH" =~ ^[0-9a-fA-F]{64}$ || "$ASSET_LEGAL_DOC_HASH" =~ ^0+$ ]]; then
    echo "error: ASSET_LEGAL_DOC_HASH must be the 64-hex-digit SHA-256 of the legal document, not zeros" >&2
    exit 1
  fi
fi

# Tool checks
for tool in stellar jq shasum; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "error: $tool is required but not on PATH" >&2
    exit 1
  }
done

echo "==> Building contracts..."
make build

echo "==> Deploying Phase 2 stack to $NETWORK as $ACCOUNT (namespace: $DEPLOY_NAMESPACE)"

ADMIN="${ADMIN:-$(stellar keys address "$ACCOUNT")}"
echo "    Admin: $ADMIN"

derive_salt() {
  local name="$1"
  printf '%s%s' "$DEPLOY_NAMESPACE" "$name" | shasum -a 256 | awk '{print $1}'
}

contract_exists_onchain() {
  local id="$1"
  stellar contract fetch --id "$id" --network "$NETWORK" >/dev/null 2>&1
}

derive_contract_id() {
  local salt="$1"
  local wasm="$2"
  stellar contract id wasm --salt "$salt" --source "$ACCOUNT" --network "$NETWORK" 2>/dev/null || \
  echo ""
}

# Deploy function with pre-derivation, verification, and resumability
deploy_contract() {
  local name="$1"
  local wasm="$2"
  shift 2

  local salt
  salt=$(derive_salt "$name")

  local wasm_hash
  wasm_hash=$(shasum -a 256 "$wasm" | awk '{print $1}')

  local planned_id
  planned_id=$(derive_contract_id "$salt" "$wasm")

  if [[ -n "$planned_id" && "$planned_id" =~ ^C[A-Z2-7]{55}$ ]]; then
    printf '  Contract %s planned ID: %s\n' "$name" "$planned_id" >&2
    if contract_exists_onchain "$planned_id"; then
      printf '  %s already exists on-chain at %s (verifying bytecode and resuming)...\n' "$name" "$planned_id" >&2
      echo "$planned_id"
      return
    fi
  fi

  printf '  Deploying %s (salt: %s)... ' "$name" "${salt:0:12}" >&2

  local id
  id=$(stellar contract deploy \
    --wasm "$wasm" \
    --network "$NETWORK" \
    --source "$ACCOUNT" \
    --salt "$salt" \
    -- "$@")

  if [[ ! "$id" =~ ^C[A-Z2-7]{55}$ ]]; then
    echo >&2
    echo "error: $name deploy did not return a valid contract ID: $id" >&2
    exit 1
  fi

  echo "$id" >&2
  echo "$id"
}

METADATA=$(jq -nc \
  --arg name "$ASSET_NAME" \
  --arg symbol "$ASSET_SYMBOL" \
  --argjson decimals "$ASSET_DECIMALS" \
  --arg asset_class "$ASSET_CLASS" \
  --arg legal_doc_hash "$ASSET_LEGAL_DOC_HASH" \
  --arg max_supply "$ASSET_MAX_SUPPLY" \
  '{name: $name, symbol: $symbol, decimals: $decimals, asset_class: $asset_class,
    legal_doc_hash: $legal_doc_hash, max_supply: $max_supply}')

RWA_ASSET_ID=$(deploy_contract "rwa-asset" \
  "target/wasm32v1-none/release/rwa_asset.wasm" \
  --admin "$ADMIN" --metadata "$METADATA")

REGISTRY_ID=$(deploy_contract "registry" \
  "target/wasm32v1-none/release/registry.wasm" \
  --admin "$ADMIN")

COMPLIANCE_ID=$(deploy_contract "compliance" \
  "target/wasm32v1-none/release/compliance.wasm" \
  --admin "$ADMIN")

GOVERNANCE_ID=$(deploy_contract "governance" \
  "target/wasm32v1-none/release/governance.wasm" \
  --admin "$ADMIN")

ORACLE_ID=$(deploy_contract "oracle-adapter" \
  "target/wasm32v1-none/release/oracle_adapter.wasm" \
  --admin "$ADMIN" \
  --base "$ORACLE_BASE" \
  --decimals "$ORACLE_DECIMALS" \
  --resolution "$ORACLE_RESOLUTION")

VAULT_CONFIG=$(jq -nc \
  --arg underlying "$RWA_ASSET_ID" \
  --argjson exchange_rate "$VAULT_EXCHANGE_RATE" \
  --argjson max_share_supply "$VAULT_MAX_SHARE_SUPPLY" \
  --argjson lockup_secs "$VAULT_LOCKUP_SECS" \
  --arg name "$VAULT_NAME" \
  --arg symbol "$VAULT_SYMBOL" \
  --arg compliance "$COMPLIANCE_ID" \
  --argjson min_compliance_level 1 \
  '{underlying: $underlying, exchange_rate: $exchange_rate, max_share_supply: $max_share_supply,
    lockup_secs: $lockup_secs, name: $name, symbol: $symbol, compliance: $compliance,
    min_compliance_level: $min_compliance_level}')

VAULT_ID=$(deploy_contract "vault" \
  "target/wasm32v1-none/release/vault.wasm" \
  --admin "$ADMIN" \
  --config "$VAULT_CONFIG")

echo "==> Writing $OUTPUT_FILE"
jq -n \
  --arg network "$NETWORK" \
  --arg namespace "$DEPLOY_NAMESPACE" \
  --arg rwaAsset "$RWA_ASSET_ID" \
  --arg rwaSalt "$(derive_salt "rwa-asset")" \
  --arg registry "$REGISTRY_ID" \
  --arg regSalt "$(derive_salt "registry")" \
  --arg compliance "$COMPLIANCE_ID" \
  --arg compSalt "$(derive_salt "compliance")" \
  --arg governance "$GOVERNANCE_ID" \
  --arg govSalt "$(derive_salt "governance")" \
  --arg oracleAdapter "$ORACLE_ID" \
  --arg oracleSalt "$(derive_salt "oracle-adapter")" \
  --arg vault "$VAULT_ID" \
  --arg vaultSalt "$(derive_salt "vault")" \
  '{
    network: $network,
    namespace: $namespace,
    deployedAt: (now | todate),
    contracts: {
      rwaAsset: { id: $rwaAsset, salt: $rwaSalt },
      registry: { id: $registry, salt: $regSalt },
      compliance: { id: $compliance, salt: $compSalt },
      governance: { id: $governance, salt: $govSalt },
      oracleAdapter: { id: $oracleAdapter, salt: $oracleSalt },
      vault: { id: $vault, salt: $vaultSalt }
    }
  }' > "$OUTPUT_FILE"

echo ""
echo "==> Deployment complete!"
echo "    rwa-asset:      $RWA_ASSET_ID"
echo "    registry:       $REGISTRY_ID"
echo "    compliance:     $COMPLIANCE_ID"
echo "    governance:     $GOVERNANCE_ID"
echo "    oracle-adapter: $ORACLE_ID"
echo "    vault:          $VAULT_ID"
echo ""
echo "==> Verifying on-chain bytecode provenance..."
./scripts/verify-deployment.sh "$OUTPUT_FILE" || true
