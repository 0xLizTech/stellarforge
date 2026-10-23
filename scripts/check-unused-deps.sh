#!/usr/bin/env bash
# Fails if any crate in the workspace declares a dependency it never uses.
#
#   scripts/check-unused-deps.sh              scan the workspace
#   scripts/check-unused-deps.sh --self-test  prove the scan can fail (and pass)
#
# Uses cargo-machete: it reads Cargo.toml and greps the sources, so it needs no
# nightly toolchain and no build. The version is pinned in
# .github/workflows/unused-deps.yml; keep MACHETE_VERSION below in step with it.
set -euo pipefail

MACHETE_VERSION="0.7.0"

if ! command -v cargo-machete >/dev/null 2>&1; then
  echo "cargo-machete is not installed. Install it with:" >&2
  echo "  cargo install cargo-machete --locked --version ${MACHETE_VERSION}" >&2
  exit 2
fi

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT

  mkdir -p "$tmp/clean/src" "$tmp/dirty/src"
  for crate in clean dirty; do
    cat > "$tmp/$crate/Cargo.toml" <<TOML
[package]
name = "machete-fixture-$crate"
version = "0.0.0"
edition = "2021"

[dependencies]
serde = "1"
TOML
  done
  printf 'use serde::Serialize;\n' > "$tmp/clean/src/lib.rs"
  printf '// serde is declared but never used\n' > "$tmp/dirty/src/lib.rs"

  cargo machete "$tmp/clean" >/dev/null
  echo "self-test: a crate that uses its dependency passes"

  if cargo machete "$tmp/dirty" >/dev/null 2>&1; then
    echo "self-test FAILED: an unused dependency was not detected" >&2
    exit 1
  fi
  echo "self-test: a crate with an unused dependency fails"
  exit 0
fi

cd "$(git rev-parse --show-toplevel)"
exec cargo machete
