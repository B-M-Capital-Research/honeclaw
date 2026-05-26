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

run_live_engine_e2e() {
  require_env HONE_FMP_API_KEY
  echo "==> hone-event-engine::tests::live_engine_e2e"
  cargo test -p hone-event-engine --lib tests::live_engine_e2e -- --ignored --nocapture
}

require_telegram_env() {
  require_env HONE_TG_BOT_TOKEN
  require_env HONE_TG_CHAT_ID
}

require_telegram_llm_env() {
  require_telegram_env
  require_env HONE_OPENROUTER_KEY
}

require_portfolio_env() {
  require_telegram_env
  require_env HONE_FMP_API_KEY
}

run_live_telegram_push_demo() {
  require_telegram_env
  echo "==> hone-event-engine::tests::live_telegram_push_demo"
  cargo test -p hone-event-engine --lib tests::live_telegram_push_demo -- --ignored --nocapture
}

run_live_telegram_push_llm_polished_demo() {
  require_telegram_llm_env
  echo "==> hone-event-engine::tests::live_telegram_push_llm_polished_demo"
  cargo test -p hone-event-engine --lib tests::live_telegram_push_llm_polished_demo -- --ignored --nocapture
}

run_live_portfolio_backtest_push() {
  require_portfolio_env
  echo "==> hone-event-engine::tests::live_portfolio_backtest_push"
  cargo test -p hone-event-engine --lib tests::live_portfolio_backtest_push -- --ignored --nocapture
}

run_live_social_engine_e2e() {
  echo "==> hone-event-engine::tests::live_social_engine_e2e"
  cargo test -p hone-event-engine --lib tests::live_social_engine_e2e -- --ignored --nocapture
}

if [[ "${RUN_EVENT_ENGINE_LIVE_SMOKES:-0}" != "1" ]]; then
  echo "[INFO] skip event-engine live integration smokes; set RUN_EVENT_ENGINE_LIVE_SMOKES=1 to run"
  exit 0
fi

require_cmd cargo

case "${EVENT_ENGINE_LIVE_SCOPE:-all}" in
  fmp)
    run_live_engine_e2e
    ;;
  telegram)
    run_live_telegram_push_demo
    ;;
  telegram-llm)
    run_live_telegram_push_llm_polished_demo
    ;;
  portfolio)
    run_live_portfolio_backtest_push
    ;;
  social)
    run_live_social_engine_e2e
    ;;
  all)
    require_env HONE_FMP_API_KEY
    require_telegram_env
    require_telegram_llm_env
    require_portfolio_env
    run_live_engine_e2e
    run_live_telegram_push_demo
    run_live_telegram_push_llm_polished_demo
    run_live_portfolio_backtest_push
    run_live_social_engine_e2e
    ;;
  *)
    echo "[FAIL] EVENT_ENGINE_LIVE_SCOPE must be one of: all, fmp, telegram, telegram-llm, portfolio, social" >&2
    exit 1
    ;;
esac
