#!/usr/bin/env bash
# deploy.sh — Deploy all StellarForge Phase 1 contracts to a Stellar network.
#
# Usage:
#   STELLAR_ACCOUNT=mykey NETWORK=testnet ./scripts/deploy.sh
#
# Every contract takes its admin as a constructor argument, so deployment and
# configuration happen in one transaction and there is no window in which a
# contract exists unowned. The admin is the deploying account unless ADMIN is
# set to something else.
#
# rwa-asset additionally needs its metadata at deploy time. The ASSET_* vars
# below override the defaults, which describe a placeholder asset suitable for
# testnet and nothing else — in particular ASSET_LEGAL_DOC_HASH defaults to
# zeroes, which is not a real document.
#
#   ASSET_NAME, ASSET_SYMBOL, ASSET_DECIMALS, ASSET_CLASS,
#   ASSET_LEGAL_DOC_HASH, ASSET_MAX_SUPPLY (0 = uncapped)
#
# Requires: stellar-cli >= 27.0.0, jq

set -euo pipefail

NETWORK="${NETWORK:-testnet}"
ACCOUNT="${STELLAR_ACCOUNT:?Set STELLAR_ACCOUNT to a funded keypair alias}"
OUTPUT_FILE="deployed-contracts.json"

ASSET_NAME="${ASSET_NAME:-StellarForge Demo Asset}"
ASSET_SYMBOL="${ASSET_SYMBOL:-SF-DEMO}"
ASSET_DECIMALS="${ASSET_DECIMALS:-7}"
ASSET_CLASS="${ASSET_CLASS:-real_estate}"
ASSET_LEGAL_DOC_HASH="${ASSET_LEGAL_DOC_HASH:-$(printf '00%.0s' {1..32})}"
ASSET_MAX_SUPPLY="${ASSET_MAX_SUPPLY:-0}"

# Fail before spending any deployments if a tool we need at the end is absent.
for tool in stellar jq; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "error: $tool is required but not on PATH" >&2
    exit 1
  }
done

echo "==> Building contracts..."
make build

echo "==> Deploying to $NETWORK as $ACCOUNT"

# Prints the deployed contract ID on stdout, and nothing else — the caller
# captures it via command substitution. Progress goes to stderr so it stays
# visible without being captured.
# Arguments after the wasm path are passed to the contract's __constructor.
deploy_contract() {
  local name="$1"
  local wasm="$2"
  shift 2
  printf '  Deploying %s... ' "$name" >&2

  local id
  id=$(stellar contract deploy \
    --wasm "$wasm" \
    --network "$NETWORK" \
    --source "$ACCOUNT" \
    -- "$@")

  # A malformed capture must not reach the output file, where it would be
  # indistinguishable from a real address until someone tried to use it.
  if [[ ! "$id" =~ ^C[A-Z2-7]{55}$ ]]; then
    echo >&2
    echo "error: $name deploy did not return a contract ID: $id" >&2
    exit 1
  fi

  echo "$id" >&2
  echo "$id"
}

ADMIN="${ADMIN:-$(stellar keys address "$ACCOUNT")}"
echo "    Admin: $ADMIN"

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

echo "==> Writing $OUTPUT_FILE"
jq -n \
  --arg network "$NETWORK" \
  --arg rwaAsset "$RWA_ASSET_ID" \
  --arg registry "$REGISTRY_ID" \
  --arg compliance "$COMPLIANCE_ID" \
  --arg governance "$GOVERNANCE_ID" \
  '{
    network: $network,
    deployedAt: (now | todate),
    contracts: {
      rwaAsset: $rwaAsset,
      registry: $registry,
      compliance: $compliance,
      governance: $governance
    }
  }' > "$OUTPUT_FILE"

echo ""
echo "==> Deployment complete!"
echo "    rwa-asset:   $RWA_ASSET_ID"
echo "    registry:    $REGISTRY_ID"
echo "    compliance:  $COMPLIANCE_ID"
echo "    governance:  $GOVERNANCE_ID"
echo ""
echo "    Written to: $OUTPUT_FILE"
echo "    (This file is in .gitignore — do not commit contract IDs to the repo)"
