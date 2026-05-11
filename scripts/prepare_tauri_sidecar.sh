#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
args=("$@")
if [[ ${#args[@]} -eq 0 ]]; then
  args=(debug)
fi

if ! command -v bun >/dev/null 2>&1; then
  echo "[FAIL] bun not found in PATH" >&2
  echo "Install Bun or add it to PATH, then rerun: bash scripts/prepare_tauri_sidecar.sh ${args[*]}" >&2
  exit 1
fi

exec bun "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs" "${args[@]}"
