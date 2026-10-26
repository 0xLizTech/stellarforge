#!/usr/bin/env bash
# doctor.sh — Verify that the local StellarForge development toolchain is
# installed and matches the versions pinned across the repository.
#
# Usage:
#   make doctor
#   ./scripts/doctor.sh
#
# Checks every required tool in one pass rather than stopping at the first
# failure, printing the expected version beside each result and exiting
# non-zero if anything is missing or mismatched.

set -uo pipefail

EXPECTED_RUSTC="1.98.1"
if [[ -f rust-toolchain.toml ]]; then
  parsed_rustc=$(sed -n 's/^channel *= *"\([^"]*\)".*/\1/p' rust-toolchain.toml)
  [[ -n "$parsed_rustc" ]] && EXPECTED_RUSTC="$parsed_rustc"
fi

EXPECTED_WASM_TARGET="wasm32v1-none"
EXPECTED_STELLAR_CLI="27.1.0"
if [[ -f .github/workflows/contracts-ci.yml ]]; then
  parsed_stellar=$(sed -n 's/^ *STELLAR_CLI_VERSION: *"\([^"]*\)".*/\1/p' .github/workflows/contracts-ci.yml)
  [[ -n "$parsed_stellar" ]] && EXPECTED_STELLAR_CLI="$parsed_stellar"
fi

EXPECTED_NODE_MIN="22.12.0"
EXPECTED_NPM_MIN="10.0.0"

failures=0

pass() {
  printf "  [OK]   %-18s %-20s (expected: %s)\n" "$1" "$2" "$3"
}

fail() {
  printf "  [FAIL] %-18s %-20s (expected: %s)\n" "$1" "$2" "$3" >&2
  failures=$((failures + 1))
}

version_ge() {
  local actual="$1"
  local required="$2"
  local IFS=.
  local i a=($actual) r=($required)
  for ((i = 0; i < ${#r[@]}; i++)); do
    local av="${a[i]:-0}"
    local rv="${r[i]:-0}"
    if ((10#$av > 10#$rv)); then
      return 0
    elif ((10#$av < 10#$rv)); then
      return 1
    fi
  done
  return 0
}

echo "Checking StellarForge toolchain..."

# 1. rustc
if command -v rustc >/dev/null 2>&1; then
  rustc_ver=$(rustc --version 2>/dev/null | awk '{print $2}')
  if [[ "$rustc_ver" == "$EXPECTED_RUSTC" ]]; then
    pass "rustc" "$rustc_ver" "$EXPECTED_RUSTC"
  else
    fail "rustc" "${rustc_ver:-unknown}" "$EXPECTED_RUSTC"
  fi
else
  fail "rustc" "missing" "$EXPECTED_RUSTC"
fi

# 2. wasm32v1-none target
if command -v rustup >/dev/null 2>&1; then
  if rustup target list --installed 2>/dev/null | grep -qx "$EXPECTED_WASM_TARGET"; then
    pass "wasm target" "$EXPECTED_WASM_TARGET" "$EXPECTED_WASM_TARGET installed"
  else
    fail "wasm target" "not installed" "$EXPECTED_WASM_TARGET installed"
  fi
else
  fail "wasm target" "rustup missing" "$EXPECTED_WASM_TARGET installed"
fi

# 3. stellar-cli
if command -v stellar >/dev/null 2>&1; then
  stellar_ver=$(stellar --version 2>/dev/null | head -n 1 | awk '{print $2}')
  if [[ "$stellar_ver" == "$EXPECTED_STELLAR_CLI" ]]; then
    pass "stellar-cli" "$stellar_ver" "$EXPECTED_STELLAR_CLI"
  else
    fail "stellar-cli" "${stellar_ver:-unknown}" "$EXPECTED_STELLAR_CLI"
  fi
else
  fail "stellar-cli" "missing" "$EXPECTED_STELLAR_CLI"
fi

# 4. node
if command -v node >/dev/null 2>&1; then
  node_ver=$(node --version 2>/dev/null | sed 's/^v//')
  if [[ -n "$node_ver" ]] && version_ge "$node_ver" "$EXPECTED_NODE_MIN"; then
    pass "node" "$node_ver" ">= $EXPECTED_NODE_MIN"
  else
    fail "node" "${node_ver:-unknown}" ">= $EXPECTED_NODE_MIN"
  fi
else
  fail "node" "missing" ">= $EXPECTED_NODE_MIN"
fi

# 5. npm
if command -v npm >/dev/null 2>&1; then
  npm_ver=$(npm --version 2>/dev/null | tr -d '[:space:]')
  if [[ -n "$npm_ver" ]] && version_ge "$npm_ver" "$EXPECTED_NPM_MIN"; then
    pass "npm" "$npm_ver" ">= $EXPECTED_NPM_MIN"
  else
    fail "npm" "${npm_ver:-unknown}" ">= $EXPECTED_NPM_MIN"
  fi
else
  fail "npm" "missing" ">= $EXPECTED_NPM_MIN"
fi

if ((failures > 0)); then
  echo "" >&2
  echo "doctor: $failures check(s) failed. See CONTRIBUTING.md for installation instructions." >&2
  exit 1
fi

echo ""
echo "All toolchain checks passed."
