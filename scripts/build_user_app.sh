#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/Library/Caches/honeclaw/user-app-target}"
TARGET="${HONE_USER_APP_TARGET:-universal-apple-darwin}"

export CARGO_TARGET_DIR
export PATH="$HOME/.bun/bin:/opt/homebrew/bin:$PATH"

if ! command -v bun >/dev/null 2>&1; then
  echo "Bun is required to run the Tauri build." >&2
  exit 1
fi

echo "[INFO] building standalone Hone user app"
echo "[INFO] target dir: $CARGO_TARGET_DIR"
echo "[INFO] target: $TARGET"

if [[ "$TARGET" == "universal-apple-darwin" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
fi

cd "$ROOT_DIR/bins/hone-user-app"
bunx tauri build \
  --target "$TARGET" \
  --bundles app,dmg

BUNDLE_DIR="$CARGO_TARGET_DIR/$TARGET/release/bundle"
APP_PATH="$BUNDLE_DIR/macos/Hone.app"
DMG_PATH="$(find "$BUNDLE_DIR/dmg" -maxdepth 1 -type f -name 'Hone_*.dmg' -print -quit)"

test -d "$APP_PATH"
test -n "$DMG_PATH"
test -f "$DMG_PATH"

echo "[OK] $APP_PATH"
echo "[OK] $DMG_PATH"
