#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
args=("$@")
if [[ ${#args[@]} -eq 0 ]]; then
  args=(debug)
fi

exec bun "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs" "${args[@]}"
