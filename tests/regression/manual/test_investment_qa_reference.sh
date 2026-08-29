#!/usr/bin/env bash

# Generate independent GPT-5.6 Sol reference answers for the reusable HONE
# investment QA benchmark. This is a manual external-account regression and is
# intentionally excluded from CI.

set -euo pipefail

if [[ "${HONE_RUN_INVESTMENT_QA_REFERENCE:-0}" != "1" ]]; then
  echo "[SKIP] set HONE_RUN_INVESTMENT_QA_REFERENCE=1 to run the paid GPT-5.6 Sol benchmark"
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
FIXTURE="$ROOT_DIR/tests/regression/manual/fixtures/investment_qa_benchmark_v1.json"
source "$ROOT_DIR/tests/regression/manual/codex_probe_home.sh"

if ! command -v codex >/dev/null 2>&1; then
  echo "[FAIL] codex command not found" >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "[FAIL] jq command not found" >&2
  exit 1
fi

CASE_FILTER="${1:-}"
CASE_TIMEOUT_SECONDS="${HONE_INVESTMENT_QA_TIMEOUT_SECONDS:-240}"
RUN_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone_investment_qa.XXXXXX")"
OUTPUT_ROOT="${HONE_INVESTMENT_QA_OUTPUT_DIR:-$ROOT_DIR/data/evals/investment-qa-v1}"
mkdir -p "$OUTPUT_ROOT"
trap 'rm -rf "$RUN_ROOT"' EXIT
hone_prepare_isolated_codex_home "$RUN_ROOT"

while IFS= read -r case_id; do
  if [[ -n "$CASE_FILTER" && "$CASE_FILTER" != "$case_id" ]]; then
    continue
  fi

  question="$(jq -r --arg id "$case_id" '.cases[] | select(.id == $id) | .prompt' "$FIXTURE")"
  must_check="$(jq -r --arg id "$case_id" '.cases[] | select(.id == $id) | .must_check | join("、")' "$FIXTURE")"
  output_file="$OUTPUT_ROOT/${case_id}.gpt-5.6-sol.md"
  prompt_file="$RUN_ROOT/${case_id}.prompt.txt"

  printf '%s\n' \
    '你是 HONE 投资问答的独立参考评审，使用 GPT-5.6 Sol。' \
    '当前时间以北京时间 2026-08-12 为准。请主动使用你当前可用的联网能力核验最新事实；优先公司 IR、SEC、监管机构和交易所等一手来源。' \
    '不要读取或假装拥有 HONE 的私有公司研究 Skill。你可以使用通用价值投资方法，但必须把事实、推断和缺口分开。' \
    '总检索预算最多 6 次，优先取得最相关的一手资料；预算内取不到的项目直接披露缺口，不要继续反复搜索。最终答案控制在 1200 个汉字以内。' \
    '先给明确行动结论，再给日期、来源、核心数据、最强反方和证伪条件。没有真实输入时不要补数。' \
    "本题必查项：$must_check" \
    "问题：$question" >"$prompt_file"

  echo "[INFO] generating GPT-5.6 Sol reference: $case_id"
  if ! perl -e 'alarm shift; exec @ARGV' "$CASE_TIMEOUT_SECONDS" codex exec \
    --ephemeral \
    --skip-git-repo-check \
    --ignore-rules \
    -s read-only \
    -m gpt-5.6-sol \
    -c 'model_reasoning_effort="high"' \
    -C "$RUN_ROOT" \
    -o "$output_file" \
    - <"$prompt_file"; then
    echo "[FAIL] GPT-5.6 Sol reference timed out or failed: $case_id" >&2
    exit 1
  fi

  if [[ ! -s "$output_file" ]]; then
    echo "[FAIL] empty reference answer: $case_id" >&2
    exit 1
  fi
done < <(jq -r '.cases[].id' "$FIXTURE")

echo "[PASS] reference answers written to $OUTPUT_ROOT"
