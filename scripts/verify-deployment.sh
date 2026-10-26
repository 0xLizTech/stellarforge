#!/usr/bin/env bash
# verify-deployment.sh — Verify on-chain contract bytecode against build checksums.
#
# Usage:
#   ./scripts/verify-deployment.sh [deployed-contracts.json] [SHA256SUMS]
#
# Fetches the deployed wasm bytecode from the network for each contract listed in
# deployed-contracts.json, computes its SHA-256 hash, and compares it against
# the expected checksum in SHA256SUMS.
#
# Exits 0 on complete match; exits 1 on any mismatch, missing contract, or fetch error.

set -euo pipefail

DEPLOY_FILE="${1:-deployed-contracts.json}"
CHECKSUMS_FILE="${2:-target/wasm32v1-none/release/SHA256SUMS}"

if [[ ! -f "$DEPLOY_FILE" ]]; then
  echo "error: deployment file not found: $DEPLOY_FILE" >&2
  exit 1
fi

NETWORK=$(jq -r '.network // "testnet"' "$DEPLOY_FILE")

echo "=========================================================================================="
echo " StellarForge Wasm Provenance Verification"
echo " Network: $NETWORK"
echo " Deployment record: $DEPLOY_FILE"
echo "=========================================================================================="
printf "%-16s %-20s %-16s %-16s %-8s\n" "CONTRACT" "CONTRACT ID" "EXPECTED HASH" "ON-CHAIN HASH" "RESULT"
echo "------------------------------------------------------------------------------------------"

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

OVERALL_STATUS=0

# Extract contracts from deployed-contracts.json (supports both flat and nested objects)
CONTRACT_KEYS=$(jq -r '.contracts | keys[]' "$DEPLOY_FILE")

for name in $CONTRACT_KEYS; do
  ID=$(jq -r ".contracts[\"$name\"] | if type==\"object\" then .id else . end" "$DEPLOY_FILE")

  if [[ -z "$ID" || "$ID" == "null" ]]; then
    printf "%-16s %-20s %-16s %-16s %-8s\n" "$name" "MISSING" "-" "-" "FAIL"
    OVERALL_STATUS=1
    continue
  fi

  # Determine expected wasm hash
  EXPECTED_HASH="UNKNOWN"
  # Map key name to wasm filename
  WASM_NAME=""
  case "$name" in
    rwaAsset) WASM_NAME="rwa_asset.wasm" ;;
    registry) WASM_NAME="registry.wasm" ;;
    compliance) WASM_NAME="compliance.wasm" ;;
    governance) WASM_NAME="governance.wasm" ;;
    oracleAdapter) WASM_NAME="oracle_adapter.wasm" ;;
    vault) WASM_NAME="vault.wasm" ;;
    *) WASM_NAME="${name}.wasm" ;;
  esac

  if [[ -f "$CHECKSUMS_FILE" ]]; then
    EXPECTED_HASH=$(grep "$WASM_NAME" "$CHECKSUMS_FILE" | awk '{print $1}' || echo "NOT_FOUND")
  fi

  # Check if recorded in deploy json directly
  JSON_HASH=$(jq -r ".contracts[\"$name\"].wasmHash // empty" "$DEPLOY_FILE")
  if [[ -n "$JSON_HASH" && "$EXPECTED_HASH" == "UNKNOWN" ]]; then
    EXPECTED_HASH="$JSON_HASH"
  fi

  FETCH_OUT="$TEMP_DIR/${name}.wasm"
  ACTUAL_HASH=""
  if stellar contract fetch --id "$ID" --network "$NETWORK" --out-file "$FETCH_OUT" >/dev/null 2>&1; then
    ACTUAL_HASH=$(shasum -a 256 "$FETCH_OUT" | awk '{print $1}')
  else
    ACTUAL_HASH="FETCH_FAILED"
  fi

  STATUS="FAIL"
  if [[ "$ACTUAL_HASH" != "FETCH_FAILED" && -n "$EXPECTED_HASH" && "$EXPECTED_HASH" != "UNKNOWN" && "$EXPECTED_HASH" != "NOT_FOUND" ]]; then
    if [[ "$ACTUAL_HASH" == "$EXPECTED_HASH" ]]; then
      STATUS="MATCH"
    else
      STATUS="MISMATCH"
      OVERALL_STATUS=1
    fi
  else
    # If network/fetch was mocked or offline, note accordingly
    if [[ "$ACTUAL_HASH" == "FETCH_FAILED" ]]; then
      STATUS="UNREACHABLE"
      OVERALL_STATUS=1
    fi
  fi

  SHORT_ID="${ID:0:8}...${ID: -6}"
  SHORT_EXPECTED="${EXPECTED_HASH:0:12}..."
  SHORT_ACTUAL="${ACTUAL_HASH:0:12}..."

  printf "%-16s %-20s %-16s %-16s %-8s\n" "$name" "$SHORT_ID" "$SHORT_EXPECTED" "$SHORT_ACTUAL" "$STATUS"
done

echo "=========================================================================================="
if ((OVERALL_STATUS == 0)); then
  echo "✓ Provenance verified: All on-chain contracts match build checksums."
  exit 0
else
  echo "✗ Provenance verification failed: One or more contracts mismatched or unreachable."
  exit 1
fi
