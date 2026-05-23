#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
WRAPPER_SCRIPT="$ROOT_DIR/scripts/prepare_tauri_sidecar.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone-tauri-wrapper.XXXXXX")"
TOOLS_DIR="$TMP_ROOT/tools"
ARGS_LOG="$TMP_ROOT/bun-args.log"
REAL_BUN="$(command -v bun || true)"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

mkdir -p "$TOOLS_DIR"

cat > "$TOOLS_DIR/bun" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$HONE_TEST_BUN_ARGS_LOG"
EOF
chmod +x "$TOOLS_DIR/bun"

run_wrapper() {
  : > "$ARGS_LOG"
  env \
    PATH="$TOOLS_DIR:/usr/bin:/bin" \
    HONE_TEST_BUN_ARGS_LOG="$ARGS_LOG" \
    bash "$WRAPPER_SCRIPT" "$@"
}

run_wrapper_with_home_bun() {
  local home_dir="$TMP_ROOT/home-with-bun"
  mkdir -p "$home_dir/.bun/bin"
  cp "$TOOLS_DIR/bun" "$home_dir/.bun/bin/bun"

  : > "$ARGS_LOG"
  env \
    HOME="$home_dir" \
    PATH="/usr/bin:/bin" \
    HONE_TEST_BUN_ARGS_LOG="$ARGS_LOG" \
    bash "$WRAPPER_SCRIPT" "$@"
}

assert_args() {
  local expected="$1"
  local actual
  actual="$(tr '\n' ' ' < "$ARGS_LOG" | sed 's/ $//')"
  if [[ "$actual" != "$expected" ]]; then
    echo "[FAIL] wrapper forwarded unexpected args" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  fi
}

run_wrapper
assert_args "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs debug"

run_wrapper release --target-triple aarch64-apple-darwin --skip-build --json
assert_args "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs release --target-triple aarch64-apple-darwin --skip-build --json"

run_wrapper_with_home_bun --shell-only
assert_args "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs --shell-only"

if [[ -z "$REAL_BUN" ]]; then
  echo "[FAIL] bun is required to validate prepare_tauri_sidecar argument errors" >&2
  exit 1
fi

if output="$("$REAL_BUN" "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs" --target-triple --json 2>&1)"; then
  echo "[FAIL] prepare_tauri_sidecar accepted --target-triple without a value" >&2
  exit 1
fi
if [[ "$output" != *"missing value for --target-triple"* ]]; then
  echo "[FAIL] missing --target-triple value should be explicit" >&2
  echo "$output" >&2
  exit 1
fi

if output="$("$REAL_BUN" "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs" debug release --shell-only 2>&1)"; then
  echo "[FAIL] prepare_tauri_sidecar accepted multiple profiles" >&2
  exit 1
fi
if [[ "$output" != *"profile can be specified only once: release"* ]]; then
  echo "[FAIL] repeated profile should be explicit" >&2
  echo "$output" >&2
  exit 1
fi

echo "[PASS] prepare_tauri_sidecar wrapper forwards all arguments"
