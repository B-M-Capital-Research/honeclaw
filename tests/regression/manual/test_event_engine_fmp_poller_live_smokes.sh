#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_env() {
  if [[ -z "${!1:-}" ]]; then
    echo "[FAIL] missing env: $1" >&2
    exit 1
  fi
}

if [[ "${RUN_EVENT_ENGINE_FMP_LIVE_SMOKES:-0}" != "1" ]]; then
  echo "[INFO] skip event-engine FMP live poller smokes; set RUN_EVENT_ENGINE_FMP_LIVE_SMOKES=1 to run"
  exit 0
fi

require_cmd cargo
require_env HONE_FMP_API_KEY

tests=(
  live_fmp_news_smoke
  live_fmp_price_smoke
  live_fmp_earnings_smoke
  live_fmp_analyst_grade_smoke
  live_fmp_corp_action_smoke
  live_fmp_earnings_surprise_smoke
  live_fmp_macro_smoke
)

for test_name in "${tests[@]}"; do
  echo "==> hone-event-engine::$test_name"
  cargo test -p hone-event-engine --lib "$test_name" -- --ignored --nocapture
done
