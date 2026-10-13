#!/usr/bin/env bash
# Real 公司完整分析 canary for prompt iterations: runs the staged V4 report for
# one company outside the web service (no task row, no object storage) and
# writes the Markdown, every node output, token usage and the rendered PDF.
#
# This spends model and data-provider credits. It only runs when
# HONE_CANARY_CONFIG points at a config with OpenRouter, FMP and search keys.
#
#   HONE_CANARY_CONFIG=/path/config.yaml HONE_CANARY_COMPANY=MU \
#     bash tests/regression/manual/test_company_analysis_canary.sh
#
# Optional: HONE_CANARY_OUT (default target/company-analysis-canary/<company>),
# HONE_CANARY_MODEL (default agent.company_analysis_model), HONE_SKILLS_DIR.
# Rerunning with the same HONE_CANARY_OUT resumes from checkpoint.json.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

if [[ -z "${HONE_CANARY_CONFIG:-}" ]]; then
  echo "[SKIP] HONE_CANARY_CONFIG is not set; the company analysis canary spends real credits"
  exit 0
fi
if [[ ! -f "$HONE_CANARY_CONFIG" ]]; then
  echo "[FAIL] HONE_CANARY_CONFIG does not exist: $HONE_CANARY_CONFIG" >&2
  exit 1
fi

export HONE_CANARY_COMPANY="${HONE_CANARY_COMPANY:-MU}"
safe_company="$(printf '%s' "$HONE_CANARY_COMPANY" | tr -c 'A-Za-z0-9._-' '-')"
export HONE_CANARY_OUT="${HONE_CANARY_OUT:-$ROOT_DIR/target/company-analysis-canary/$safe_company}"
mkdir -p "$HONE_CANARY_OUT"

echo "[INFO] company=$HONE_CANARY_COMPANY out=$HONE_CANARY_OUT"
started=$(date +%s)
cargo test -p hone-web-api --lib company_analysis_canary -- --ignored --nocapture
echo "[INFO] wall time $(( $(date +%s) - started ))s"

for file in report.md usage.json; do
  if [[ ! -s "$HONE_CANARY_OUT/$file" ]]; then
    echo "[FAIL] missing $HONE_CANARY_OUT/$file" >&2
    exit 1
  fi
done
echo "[PASS] company analysis canary wrote report.md, nodes/, usage.json$( [[ -s "$HONE_CANARY_OUT/report.pdf" ]] && echo ' and report.pdf')"
echo "[INFO] review the content by hand; this script does not judge report quality"
