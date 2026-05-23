#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LAUNCH_SCRIPT="$ROOT_DIR/launch.sh"
CLI_START="$ROOT_DIR/bins/hone-cli/src/start.rs"
RESTART_SCRIPT="$ROOT_DIR/scripts/restart_hone.sh"

contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

if ! contains 'launch.sh has been retired.' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh should be an explicit retired-entry shim" >&2
  exit 1
fi

if ! contains 'cargo run -p hone-cli -- start --build' "$LAUNCH_SCRIPT"; then
  echo "[FAIL] launch.sh retirement message does not point to the source CLI start path" >&2
  exit 1
fi

if ! contains 'pub(crate) struct StartArgs' "$CLI_START"; then
  echo "[FAIL] hone-cli start should expose structured start arguments" >&2
  exit 1
fi

if ! contains 'build_source_runtime_binaries' "$CLI_START"; then
  echo "[FAIL] hone-cli start no longer owns source runtime builds" >&2
  exit 1
fi

if ! contains 'current.pid' "$CLI_START"; then
  echo "[FAIL] hone-cli start no longer records the supervisor pid" >&2
  exit 1
fi

if ! contains 'hone-cli start --build' "$RESTART_SCRIPT"; then
  echo "[FAIL] restart_hone should restart through the CLI source start path" >&2
  exit 1
fi

restart_usage="$(bash "$RESTART_SCRIPT" 2>&1 || true)"
if [[ "$restart_usage" != *'scripts/restart_hone.sh <project_root> [old_pid]'* ]]; then
  echo "[FAIL] restart_hone missing-argument usage should show old_pid as optional" >&2
  exit 1
fi

invalid_root_output="$(bash "$RESTART_SCRIPT" "$ROOT_DIR/does-not-exist" 2>&1 || true)"
if [[ "$invalid_root_output" != *'[restart_hone] error: project root is not a directory:'* ]]; then
  echo "[FAIL] restart_hone should reject invalid project roots before log setup" >&2
  exit 1
fi

invalid_pid_output="$(bash "$RESTART_SCRIPT" "$ROOT_DIR" -1 2>&1 || true)"
if [[ "$invalid_pid_output" != *'[restart_hone] error: old_pid must be a positive integer: -1'* ]]; then
  echo "[FAIL] restart_hone should reject invalid old_pid before sending signals" >&2
  exit 1
fi

echo "[PASS] source CLI start contract regression passed"
