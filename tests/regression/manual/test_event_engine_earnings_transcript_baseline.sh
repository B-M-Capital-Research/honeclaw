#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

FIXTURE="tests/fixtures/event_engine/earnings_transcript_baseline_2026-08-06.json"

# 默认只验证来源清单、四种公司形态、八份材料和正文不入夹具的合同；不访问
# 公司 IR、不调用模型、不产生费用。
cargo test -q -p hone-event-engine \
  official_transcript_fixture_covers_four_company_shapes_and_eight_calls --lib

if [ "${RUN_EVENT_ENGINE_EARNINGS_TRANSCRIPT_BASELINE:-0}" != "1" ]; then
  echo "[PASS] earnings transcript fixture and offline contract validated"
  echo "fixture=$FIXTURE"
  echo "[INFO] set RUN_EVENT_ENGINE_EARNINGS_TRANSCRIPT_BASELINE=1 for the paid IR + OpenRouter replay"
  exit 0
fi

echo "[INFO] starting paid eight-call official IR + OpenRouter transcript replay"
echo "[INFO] model=${HONE_EARNINGS_TRANSCRIPT_MODEL:-x-ai/grok-4.5}"
cargo run -q -p hone-event-engine --example earnings_transcript_models
