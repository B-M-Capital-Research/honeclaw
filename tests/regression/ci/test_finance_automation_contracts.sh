#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

success=0
review=0
fail=0

record() {
  local status="$1"
  local sample="$2"
  local detail="$3"

  case "$status" in
    success) success=$((success + 1)) ;;
    review) review=$((review + 1)) ;;
    fail) fail=$((fail + 1)) ;;
    *)
      echo "[ERROR] unknown status: $status"
      exit 1
      ;;
  esac

  echo "[$status] $sample - $detail"
}

contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

contains_regex() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q "$pattern" "$file"
  else
    grep -E -q -- "$pattern" "$file"
  fi
}

DATA_FETCH="crates/hone-tools/src/data_fetch.rs"
PROMPT_FILE="crates/hone-channels/src/prompt.rs"
STOCK_RESEARCH="skills/stock_research/SKILL.md"
STOCK_SELECTION="skills/stock_selection/SKILL.md"
POSITION_ADVICE="skills/position_advice/SKILL.md"
VALUATION="skills/valuation/SKILL.md"
MAJOR_ALERT="skills/major_alert/SKILL.md"
SCHEDULED_TASK="skills/scheduled_task/SKILL.md"
GOLD_ANALYSIS="skills/gold-analysis/SKILL.md"

echo "[finance-automation-contracts] fixed sample count: 9"

if contains '"snapshot".into()' "$DATA_FETCH" && contains 'data_fetch(data_type="snapshot"' "$STOCK_RESEARCH"; then
  record success "1.stock_research->snapshot" "tool enum and skill contract are aligned"
else
  record fail "1.stock_research->snapshot" "skill references snapshot but tool contract is incomplete"
fi

if contains '"snapshot".into()' "$DATA_FETCH" && contains 'data_fetch(data_type="snapshot"' "$STOCK_SELECTION"; then
  record success "2.stock_selection->snapshot" "tool enum and skill contract are aligned"
else
  record fail "2.stock_selection->snapshot" "skill references snapshot but tool contract is incomplete"
fi

if contains 'data_fetch(data_type="earnings_calendar")' "$MAJOR_ALERT" && contains 'from=2024-01-01&to=2024-12-31' "$DATA_FETCH"; then
  record fail "3.major_alert->earnings_calendar-window" "earnings calendar is still pinned to 2024"
else
  record success "3.major_alert->earnings_calendar-window" "earnings calendar is not pinned to the legacy 2024 window"
fi

if contains 'earnings_calendar' "$SCHEDULED_TASK" && contains 'from=2024-01-01&to=2024-12-31' "$DATA_FETCH"; then
  record fail "4.scheduled_task->earnings_calendar-window" "scheduled-task linkage still depends on the legacy 2024 window"
else
  record success "4.scheduled_task->earnings_calendar-window" "scheduled-task linkage is not pinned to the legacy 2024 window"
fi

if contains_regex 'TODO:|\[TODO' "$GOLD_ANALYSIS"; then
  record fail "5.gold-analysis-template" "skill still contains template placeholders"
else
  record success "5.gold-analysis-template" "skill has been filled out"
fi

if contains 'trim, add, or hold' "$POSITION_ADVICE" || contains 'Give actionable, explicit advice' "$POSITION_ADVICE"; then
  record fail "6.position_advice-policy" "skill still encourages direct action recommendations"
else
  record success "6.position_advice-policy" "skill stays within the global finance policy"
fi

if contains 'Return a recommendation list' "$STOCK_SELECTION"; then
  record fail "7.stock_selection-policy" "skill still asks for direct recommendation lists"
else
  record success "7.stock_selection-policy" "skill avoids direct stock-picking language"
fi

if contains 'overvalued, fair, or undervalued' "$VALUATION"; then
  record review "8.valuation-conditionality" "valuation still uses categorical end states and should be reviewed in a later round"
else
  record success "8.valuation-conditionality" "valuation wording is conditional instead of categorical"
fi

if contains 'DEFAULT_FINANCE_DOMAIN_POLICY' "$PROMPT_FILE" && contains 'static_system.push_str(DEFAULT_FINANCE_DOMAIN_POLICY);' "$PROMPT_FILE"; then
  record success "9.runtime-finance-prompt" "global finance prompt is injected at runtime"
else
  record fail "9.runtime-finance-prompt" "global finance prompt injection is missing"
fi

echo
echo "summary: success=$success review=$review fail=$fail total=$((success + review + fail))"

if [ "$success" -lt 3 ]; then
  echo "[ERROR] Round 1 acceptance failed: expected at least 3 successes"
  exit 1
fi

if [ "$fail" -gt 5 ]; then
  echo "[ERROR] Round 1 acceptance failed: expected at most 5 failures"
  exit 1
fi
