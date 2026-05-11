#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
WRAPPER_SCRIPT="$ROOT_DIR/scripts/prepare_tauri_sidecar.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone-tauri-wrapper.XXXXXX")"
TOOLS_DIR="$TMP_ROOT/tools"
ARGS_LOG="$TMP_ROOT/bun-args.log"

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

echo "[PASS] prepare_tauri_sidecar wrapper forwards all arguments"
