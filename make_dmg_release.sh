#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
cd "$PROJECT_ROOT" || exit 1

TARGETS=()
if [[ $# -eq 0 ]]; then
  TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")
else
  TARGETS=("$@")
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

default_target_dir() {
  if [[ "$(uname -s)" == "Darwin" ]]; then
    echo "$HOME/Library/Caches/hone-financial/target"
  else
    echo "${XDG_CACHE_HOME:-$HOME/.cache}/hone-financial/target"
  fi
}

ensure_rust_target() {
  local target="$1"
  if rustup target list --installed | grep -qx "$target"; then
    return 0
  fi
  echo "[INFO] installing Rust target: $target"
  rustup target add "$target"
}

copy_dmg_outputs() {
  local target="$1"
  local source_dir="$CARGO_TARGET_DIR/$target/release/bundle/dmg"
  local dest_dir="$PROJECT_ROOT/dist/dmg/$target"
  mkdir -p "$dest_dir"
  if [[ ! -d "$source_dir" ]]; then
    echo "[FAIL] expected DMG output directory missing: $source_dir" >&2
    exit 1
  fi
  find "$source_dir" -maxdepth 1 -name '*.dmg' -print0 | while IFS= read -r -d '' dmg; do
    cp -f "$dmg" "$dest_dir/"
  done
  if ! find "$dest_dir" -maxdepth 1 -name '*.dmg' | grep -q .; then
    echo "[FAIL] no DMG produced for $target" >&2
    exit 1
  fi
}

if [[ -x "$HOME/.bun/bin/bun" ]]; then
  export BUN_INSTALL="$HOME/.bun"
  export PATH="$BUN_INSTALL/bin:$PATH"
fi

require_cmd cargo
require_cmd rustup
require_cmd bun
require_cmd bunx

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(default_target_dir)}"
mkdir -p "$CARGO_TARGET_DIR"

if [[ ! -d "$PROJECT_ROOT/node_modules" ]]; then
  echo "[INFO] installing Bun dependencies..."
  bun install
fi

echo "[INFO] building shared web assets..."
bun run build:web

for target in "${TARGETS[@]}"; do
  case "$target" in
    aarch64-apple-darwin|x86_64-apple-darwin) ;;
    *)
      echo "[FAIL] unsupported DMG target: $target" >&2
      exit 1
      ;;
  esac

  echo "[INFO] preparing release bundle for $target"
  ensure_rust_target "$target"
  bun scripts/prepare_tauri_sidecar.mjs release --target-triple "$target" --skip-build-command
  HONE_TAURI_TARGET_TRIPLE="$target" TAURI_ENV_TARGET_TRIPLE="$target" \
    bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json --target "$target"
  copy_dmg_outputs "$target"
done

echo "[INFO] DMG artifacts copied to $PROJECT_ROOT/dist/dmg"
