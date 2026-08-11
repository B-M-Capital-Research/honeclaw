#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

SKILL="skills/company-thesis-ratings/SKILL.md"
CARDS="skills/company-thesis-ratings/references/company-cards.json"
INDEX="skills/company-thesis-ratings/references/company-index.json"
HARI="skills/hari-invest/SKILL.md"
PROMPT="crates/hone-channels/src/prompt.rs"
TURN_BUILDER="crates/hone-channels/src/turn_builder.rs"

contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

for file in "$SKILL" "$CARDS" "$INDEX" "$HARI" "$PROMPT" "$TURN_BUILDER"; do
  test -s "$file" || { echo "[FAIL] missing company research dialogue artifact: $file"; exit 1; }
done

contains '"symbol": "MSFT"' "$CARDS" || { echo "[FAIL] company cards lost MSFT"; exit 1; }
contains '"symbol": "SNDK"' "$CARDS" || { echo "[FAIL] company cards lost SNDK"; exit 1; }
contains '"微软"' "$INDEX" || { echo "[FAIL] company alias index lost Microsoft Chinese name"; exit 1; }
contains '"闪迪"' "$INDEX" || { echo "[FAIL] company alias index lost Sandisk Chinese name"; exit 1; }
contains '必须同时加载 `company-thesis-ratings`' "$HARI" || {
  echo "[FAIL] Hari no longer composes the covered-company research Skill"
  exit 1
}
contains 'skill_tool(skill_name=\"company-thesis-ratings\")' "$PROMPT" || {
  echo "[FAIL] function-calling runtime no longer requires the company research Skill"
  exit 1
}
contains '历史公司研究基线不是当前事实源' "$PROMPT" || {
  echo "[FAIL] historical research is no longer separated from current facts"
  exit 1
}
contains 'company_research_baseline(user_input)' "$TURN_BUILDER" || {
  echo "[FAIL] covered-company research is no longer projected into the current turn"
  exit 1
}
contains '不得向用户泄露逐字稿原文' "$PROMPT" || {
  echo "[FAIL] private transcript boundary disappeared"
  exit 1
}

echo "[PASS] company research corpus dialogue contract"
