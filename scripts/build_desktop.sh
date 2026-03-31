#!/usr/bin/env bash
set -euo pipefail

# Desktop packaging script (separated from launch flow)
#
# Usage:
#   bash scripts/build_desktop.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT" || exit 1

default_target_dir() {
  if [[ "$(uname -s)" == "Darwin" ]]; then
    echo "$HOME/Library/Caches/hone-financial/target"
  else
    echo "${XDG_CACHE_HOME:-$HOME/.cache}/hone-financial/target"
  fi
}

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(default_target_dir)}"
mkdir -p "$CARGO_TARGET_DIR"

if [[ -x "$HOME/.bun/bin/bun" ]]; then
  export BUN_INSTALL="$HOME/.bun"
  export PATH="$BUN_INSTALL/bin:$PATH"
fi

if ! command -v bun >/dev/null 2>&1; then
  echo "[FAIL] bun not found in PATH"
  exit 1
fi

echo "[INFO] installing dependencies..."
bun install

echo "[INFO] preparing desktop sidecar binaries..."
bun scripts/prepare_tauri_sidecar.mjs release

echo "[INFO] building desktop package..."
bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json

echo "[INFO] desktop package build completed."
