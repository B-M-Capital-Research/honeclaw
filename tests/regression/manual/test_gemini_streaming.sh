#!/usr/bin/env bash
set -euo pipefail

if ! command -v gemini >/dev/null 2>&1; then
  echo "[FAIL] gemini command not found in PATH" >&2
  echo "Install Gemini CLI or add it to PATH, then rerun: bash tests/regression/manual/test_gemini_streaming.sh" >&2
  exit 1
fi

prompt="用中文回答：1+1=? 只输出数字2"
if ! output=$(gemini --prompt "$prompt" --yolo -o stream-json); then
  echo "[FAIL] gemini stream-json command returned non-zero exit code" >&2
  exit 1
fi

if ! echo "$output" | grep -q '"type"'; then
  echo "[FAIL] gemini stream-json output missing type field" >&2
  printf '%s\n' "$output" >&2
  exit 1
fi

echo "[PASS] gemini stream-json output detected"
