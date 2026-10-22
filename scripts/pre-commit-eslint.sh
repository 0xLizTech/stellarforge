#!/usr/bin/env bash
# Runs the SDK's eslint on the staged files pre-commit hands us.
#
# pre-commit passes repo-root-relative paths (sdk/src/x.ts); eslint must run
# from sdk/ so it finds sdk/eslint.config.mjs, so the prefix is stripped.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root/sdk"

eslint_bin="node_modules/.bin/eslint"
if [ ! -e "$eslint_bin" ]; then
  echo "eslint is not installed. Run 'make sdk-install' (npm ci in sdk/) first." >&2
  exit 2
fi

files=()
for f in "$@"; do
  files+=("${f#sdk/}")
done

if [ "${#files[@]}" -eq 0 ]; then
  exit 0
fi

exec "$eslint_bin" "${files[@]}"
