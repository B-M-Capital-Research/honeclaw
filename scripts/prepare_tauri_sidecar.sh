#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-debug}"
case "$PROFILE" in
  debug|release) ;;
  *)
    echo "usage: $0 [debug|release]" >&2
    exit 1
    ;;
esac

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_TRIPLE="${TAURI_ENV_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}"
TARGET_DIR="$ROOT_DIR/target/${TARGET_TRIPLE}/${PROFILE}"
DEST_DIR="$ROOT_DIR/bins/hone-desktop/binaries"
BINS=(
  "hone-imessage"
  "hone-discord"
  "hone-feishu"
  "hone-telegram"
)

mkdir -p "$DEST_DIR"

if [[ "$PROFILE" == "release" ]]; then
  for bin in "${BINS[@]}"; do
    cargo build --bin "$bin" --release --target "$TARGET_TRIPLE"
  done
else
  for bin in "${BINS[@]}"; do
    cargo build --bin "$bin" --target "$TARGET_TRIPLE"
  done
fi

for bin in "${BINS[@]}"; do
  SRC_BIN="$TARGET_DIR/$bin"
  DEST_BIN="$DEST_DIR/${bin}-${TARGET_TRIPLE}"

  if [[ "$TARGET_TRIPLE" == *windows* ]]; then
    SRC_BIN="${SRC_BIN}.exe"
    DEST_BIN="${DEST_BIN}.exe"
  fi

  cp "$SRC_BIN" "$DEST_BIN"
  chmod +x "$DEST_BIN" 2>/dev/null || true
done
