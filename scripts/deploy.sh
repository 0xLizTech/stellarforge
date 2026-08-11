#!/usr/bin/env bash
# deploy.sh — Deploy all StellarForge Phase 1 contracts to a Stellar network.
#
# Usage:
#   STELLAR_ACCOUNT=mykey NETWORK=testnet ./scripts/deploy.sh
#
# Requires: stellar-cli >= 22.0.0, jq

set -euo pipefail

NETWORK="${NETWORK:-testnet}"
ACCOUNT="${STELLAR_ACCOUNT:?Set STELLAR_ACCOUNT to a funded keypair alias}"
OUTPUT_FILE="deployed-contracts.json"

echo "==> Building contracts..."
make build

echo "==> Deploying to $NETWORK as $ACCOUNT"

deploy_contract() {
  local name="$1"
  local wasm="$2"
  echo -n "  Deploying $name... "
  local id
  id=$(stellar contract deploy \
    --wasm "$wasm" \
    --network "$NETWORK" \
    --source "$ACCOUNT" 2>&1 | tail -1)
  echo "$id"
  echo "$id"
}

RWA_ASSET_ID=$(deploy_contract "rwa-asset" \
  "target/wasm32v1-none/release/rwa_asset.wasm")

REGISTRY_ID=$(deploy_contract "registry" \
  "target/wasm32v1-none/release/registry.wasm")

COMPLIANCE_ID=$(deploy_contract "compliance" \
  "target/wasm32v1-none/release/compliance.wasm")

GOVERNANCE_ID=$(deploy_contract "governance" \
  "target/wasm32v1-none/release/governance.wasm")

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
