#!/usr/bin/env bash

# Gemini CLI non-interactive smoke test.
# Expected: gemini -p runs successfully and the JSON response contains the exact token.

set -euo pipefail

EXPECTED_TOKEN="GEMINI_CLI_OK"
PROMPT="Reply with exactly one line: ${EXPECTED_TOKEN}"

if ! command -v gemini >/dev/null 2>&1; then
  echo "[FAIL] gemini command not found in PATH" >&2
  exit 1
fi

STDOUT_FILE="$(mktemp /tmp/hone_gemini_test_stdout.XXXXXX)"
STDERR_FILE="$(mktemp /tmp/hone_gemini_test_stderr.XXXXXX)"
trap 'rm -f "$STDOUT_FILE" "$STDERR_FILE"' EXIT

if ! gemini -p "$PROMPT" -o json >"$STDOUT_FILE" 2>"$STDERR_FILE"; then
  echo "[FAIL] gemini CLI returned non-zero exit code" >&2
  echo "--- stderr ---" >&2
  cat "$STDERR_FILE" >&2 || true
  echo "--- stdout ---" >&2
  cat "$STDOUT_FILE" >&2 || true
  exit 1
fi

if ! grep -q "\"response\"[[:space:]]*:[[:space:]]*\"${EXPECTED_TOKEN}\"" "$STDOUT_FILE"; then
  echo "[FAIL] unexpected gemini output" >&2
  echo "expected response token: $EXPECTED_TOKEN" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_FILE" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_FILE" >&2 || true
  exit 1
fi

echo "[PASS] gemini exec smoke test passed: $EXPECTED_TOKEN"
