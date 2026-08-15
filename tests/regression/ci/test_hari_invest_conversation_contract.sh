#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

SKILL="skills/hari-invest/SKILL.md"
RUBRIC="skills/hari-invest/references/decision-rubric.md"
CONVERSATION="skills/hari-invest/references/conversation-contract.md"
PROMPT="crates/hone-channels/src/prompt.rs"

contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

for file in "$SKILL" "$RUBRIC" "$CONVERSATION"; do
  test -s "$file" || { echo "[FAIL] missing Hari conversation artifact: $file"; exit 1; }
done

for required in \
  "机会区" \
  "持有区" \
  "风险区" \
  "数据不足" \
  "短期、中期和长期" \
  "最强反方" \
  "不冒充老王"; do
  contains "$required" "$SKILL" || {
    echo "[FAIL] Hari Skill lost conversation requirement: $required"
    exit 1
  }
done

contains '首行之后的第一段必须直接给出“结论：”' "$PROMPT" || {
  echo "[FAIL] system fallback no longer guarantees a conclusion-first answer"
  exit 1
}
contains "次要数据缺失不能成为" "$PROMPT" || {
  echo "[FAIL] system fallback no longer distinguishes critical and secondary gaps"
  exit 1
}
contains "allow_implicit_invocation: true" "skills/hari-invest/agents/openai.yaml" || {
  echo "[FAIL] Hari Skill is no longer implicitly invocable"
  exit 1
}

for internal_only in "团队内部说明" "可发送草稿" "laowang-investment-internal"; do
  if contains "$internal_only" "$SKILL"; then
    echo "[FAIL] internal-team workflow leaked into the public conversation Skill: $internal_only"
    exit 1
  fi
done

echo "[PASS] Hari Invest conversation contract"
