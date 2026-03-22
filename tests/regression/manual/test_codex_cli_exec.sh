#!/usr/bin/env bash

# Codex CLI non-interactive smoke test.
# Expected: codex exec runs successfully and returns the exact token.

set -euo pipefail

EXPECTED_TOKEN="CODEX_CLI_OK"
PROMPT="Reply with exactly one line: ${EXPECTED_TOKEN}"

if ! command -v codex >/dev/null 2>&1; then
  echo "[FAIL] codex command not found in PATH" >&2
  exit 1
fi

OUT_FILE="$(mktemp /tmp/hone_codex_test_out.XXXXXX)"
STDOUT_FILE="$(mktemp /tmp/hone_codex_test_stdout.XXXXXX)"
STDERR_FILE="$(mktemp /tmp/hone_codex_test_stderr.XXXXXX)"
trap 'rm -f "$OUT_FILE" "$STDOUT_FILE" "$STDERR_FILE"' EXIT

if ! printf '%s\n' "$PROMPT" | codex exec --skip-git-repo-check -o "$OUT_FILE" - >"$STDOUT_FILE" 2>"$STDERR_FILE"; then
  echo "[FAIL] codex exec returned non-zero exit code" >&2
  echo "--- stderr ---" >&2
  cat "$STDERR_FILE" >&2 || true
  exit 1
fi

RESULT="$(tr -d '\r' <"$OUT_FILE" | sed '/^[[:space:]]*$/d' | head -n 1 | xargs)"

if [[ "$RESULT" != "$EXPECTED_TOKEN" ]]; then
  echo "[FAIL] unexpected codex output" >&2
  echo "expected: $EXPECTED_TOKEN" >&2
  echo "actual:   $RESULT" >&2
  echo "--- output file ---" >&2
  cat "$OUT_FILE" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_FILE" >&2 || true
  exit 1
fi

echo "[PASS] codex exec smoke test passed: $RESULT"
