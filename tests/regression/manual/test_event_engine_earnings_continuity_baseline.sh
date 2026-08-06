#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

FIXTURE="tests/fixtures/event_engine/earnings_continuity_baseline_2026-08-06.json"

# 默认路径不访问网络、不调用付费模型；Rust 契约测试会验证 6 类公司、每类连续
# 4 季、24 个唯一 SEC 一手来源以及日期顺序。
cargo test -q -p hone-event-engine \
  institutional_continuity_fixture_covers_six_archetypes_and_four_quarters --lib

if [ "${RUN_EVENT_ENGINE_EARNINGS_CONTINUITY_BASELINE:-0}" != "1" ]; then
  echo "[PASS] earnings continuity fixture and offline contract validated"
  echo "fixture=$FIXTURE"
  echo "[INFO] set RUN_EVENT_ENGINE_EARNINGS_CONTINUITY_BASELINE=1 for the paid SEC + OpenRouter replay"
  exit 0
fi

echo "[INFO] starting paid 24-event SEC + OpenRouter continuity replay"
echo "[INFO] model=${HONE_EARNINGS_CONTINUITY_MODEL:-x-ai/grok-4.5}"
cargo run -q -p hone-event-engine --example earnings_continuity_models
