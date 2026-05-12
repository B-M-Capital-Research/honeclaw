#!/usr/bin/env bash
# Manual-only POC for llm.providers + llm.profiles with real OpenRouter calls.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

CONFIG_PATH="${HONE_LLM_PROFILE_POC_CONFIG:-tests/fixtures/llm/profile_poc.yaml}"
PROFILE="${HONE_LLM_PROFILE_POC_PROFILE:-poc_reasoning_json}"

if [ "${RUN_LLM_PROFILE_POC:-}" != "1" ]; then
  cargo run -p hone-llm --example llm_profile_poc -- "$CONFIG_PATH" "$PROFILE"
  exit 0
fi

RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc -- "$CONFIG_PATH" "$PROFILE"
