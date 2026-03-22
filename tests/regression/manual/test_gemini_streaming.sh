#!/usr/bin/env bash
set -euo pipefail

if ! command -v gemini >/dev/null 2>&1; then
  echo "gemini CLI not found in PATH"
  exit 1
fi

prompt="用中文回答：1+1=? 只输出数字2"
output=$(gemini --prompt "$prompt" --yolo -o stream-json)

if ! echo "$output" | grep -q '"type"'; then
  echo "gemini stream-json output missing type field"
  exit 1
fi

echo "gemini stream-json output detected"
