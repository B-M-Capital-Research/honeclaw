#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LAUNCH_SCRIPT="$ROOT_DIR/launch.sh"
BUILD_SCRIPT="$ROOT_DIR/scripts/build_desktop.sh"

# Fixed-string matcher used by this regression check, with rg->grep fallback.
contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

if ! contains '$HOME/Library/Caches/honeclaw/target' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh no longer defaults release builds to the honeclaw cache target" >&2
  exit 1
fi

if ! contains '$HOME/Library/Caches/honeclaw/target' "$BUILD_SCRIPT"; then
  echo "[FAIL] scripts/build_desktop.sh no longer defaults to the honeclaw cache target" >&2
  exit 1
fi

if ! contains 'Hone Financial.app/Contents/MacOS/hone-desktop' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh --release no longer targets the packaged .app executable" >&2
  exit 1
fi

if contains 'RELEASE_DESKTOP_BIN="$TARGET_DIR/release/hone-desktop"' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh --release regressed to the naked target/release binary" >&2
  exit 1
fi

if ! contains 'write_current_pid()' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh is missing the supervisor current.pid helper" >&2
  exit 1
fi

if ! contains 'echo "$$" > "$(pid_file current)"' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh no longer records its supervisor pid into data/runtime/current.pid" >&2
  exit 1
fi

echo "[PASS] release launch runtime contract regression passed"
