#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

shopt -s nullglob
scripts=(tests/regression/manual/test_*.sh)

if [ ${#scripts[@]} -eq 0 ]; then
  echo "[INFO] no manual regression scripts found under tests/regression/manual"
  exit 0
fi

for t in "${scripts[@]}"; do
  echo "==> $t"
  bash "$t"
done

