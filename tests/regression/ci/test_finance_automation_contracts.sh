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
  # Keep fixed-string semantics consistent with rg --fixed-strings.
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

DATA_FETCH="crates/hone-tools/src/data_fetch.rs"
PROMPT_FILE="crates/hone-channels/src/prompt.rs"
STOCK_RESEARCH="skills/stock_research/SKILL.md"
POSITION_ADVICE="skills/position_advice/SKILL.md"
SCHEDULED_TASK="skills/scheduled_task/SKILL.md"
GOLD_ANALYSIS="skills/gold-analysis/SKILL.md"

echo "[finance-automation-contracts] fixed sample count: 8"

if contains '"snapshot".into()' "$DATA_FETCH" && contains 'data_fetch(data_type="snapshot"' "$STOCK_RESEARCH"; then
  record success "1.stock_research->snapshot" "tool enum and skill contract are aligned"
else
  record fail "1.stock_research->snapshot" "skill references snapshot but tool contract is incomplete"
fi

if contains '"financials".into()' "$DATA_FETCH" && contains 'data_fetch(data_type="financials"' "$STOCK_RESEARCH" && contains 'OWGZ' "$STOCK_RESEARCH"; then
  record success "2.stock_research->valuation-mode" "canonical stock research skill covers valuation mode"
else
  record fail "2.stock_research->valuation-mode" "valuation mode is missing from canonical stock research"
fi

if contains '"gainers_losers".into()' "$DATA_FETCH" && contains 'data_fetch(data_type="gainers_losers")' "$STOCK_RESEARCH" && contains 'OWXG' "$STOCK_RESEARCH"; then
  record success "3.stock_research->screening-mode" "canonical stock research skill covers screener mode"
else
  record fail "3.stock_research->screening-mode" "screening mode is missing from canonical stock research"
fi

if contains 'earnings_calendar' "$SCHEDULED_TASK" && contains 'from=2024-01-01&to=2024-12-31' "$DATA_FETCH"; then
  record fail "4.scheduled_task->earnings_calendar-window" "scheduled-task linkage still depends on the legacy 2024 window"
else
  record success "4.scheduled_task->earnings_calendar-window" "scheduled-task linkage is not pinned to the legacy 2024 window"
fi

if contains 'TODO:' "$GOLD_ANALYSIS" || contains '[TODO' "$GOLD_ANALYSIS"; then
  record fail "5.gold-analysis-template" "skill still contains template placeholders"
else
  record success "5.gold-analysis-template" "skill has been filled out"
fi

if contains 'trim, add, or hold' "$POSITION_ADVICE" || contains 'Give actionable, explicit advice' "$POSITION_ADVICE"; then
  record fail "6.position_advice-policy" "skill still encourages direct action recommendations"
else
  record success "6.position_advice-policy" "skill stays within the global finance policy"
fi

if contains 'recommendation list' "$STOCK_RESEARCH" && ! contains 'Do not output a blunt recommendation list' "$STOCK_RESEARCH"; then
  record fail "7.stock_research-policy" "canonical stock research still encourages direct recommendation lists"
else
  record success "7.stock_research-policy" "canonical stock research avoids direct stock-picking language"
fi

if contains 'overvalued, fair, or undervalued' "$STOCK_RESEARCH"; then
  record review "8.stock_research-conditionality" "valuation mode still uses categorical end states and should be reviewed in a later round"
else
  record success "8.stock_research-conditionality" "valuation mode wording is conditional instead of categorical"
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
