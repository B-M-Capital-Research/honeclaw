#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
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
MARKET_ANALYSIS="skills/market_analysis/SKILL.md"
POSITION_ADVICE="skills/position_advice/SKILL.md"
SCHEDULED_TASK="skills/scheduled_task/SKILL.md"
GOLD_ANALYSIS="skills/gold-analysis/SKILL.md"
INVESTMENT_GUARD="crates/hone-channels/src/investment_response_guard.rs"
SECURITY_IDENTIFIER="crates/hone-channels/src/security_identifier.rs"
EXECUTION="crates/hone-channels/src/execution.rs"
AGENT_TYPES="crates/hone-channels/src/agent_session/types.rs"
AGENT_CORE="crates/hone-channels/src/agent_session/core.rs"
AGENT_TESTS="crates/hone-channels/src/agent_session/tests.rs"
SCHEDULER="crates/hone-channels/src/scheduler.rs"
SOUL="soul.md"
AGENT_DISCOVERY_IMPL="$(sed -n '/pub(crate) fn build_agent_discovered_investment(/,/^fn tool_call_targets_entity(/p' "$INVESTMENT_GUARD")"
AGENT_TRUTH_IMPL="$(sed -n '/pub(crate) fn missing_agent_discovered_truth_violations(/,/pub(crate) fn missing_investment_response_sections(/p' "$INVESTMENT_GUARD")"

echo "[finance-automation-contracts] fixed sample count: 27"

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

if contains 'missing_deep_single_stock_sections' "$INVESTMENT_GUARD" && contains 'missing_agent_discovered_truth_violations' "$INVESTMENT_GUARD" && contains 'if !agent_discovered_contract' "$AGENT_CORE"; then
  record success "10.typed-deep-stock-response-contract" "typed scheduled/heartbeat work retains deep format validation while interactive Agent discovery uses truth-only validation"
else
  record fail "10.typed-deep-stock-response-contract" "typed deep validation and the interactive truth-only boundary are not both preserved"
fi

if contains 'name: "query".to_string()' "$DATA_FETCH" && contains '必须先用 search' "$DATA_FETCH" && contains '实体发现与证据加载必须在主 agent loop 内完成' "$PROMPT_FILE" && contains '不要求把千变万化的问法硬塞进闭合标签' "$PROMPT_FILE"; then
  record success "13.entity-search-contract" "DataFetch search and the open-ended main agent loop own first-stage entity discovery"
else
  record fail "13.entity-search-contract" "first-stage entity discovery is not owned by DataFetch search and the open-ended main agent loop"
fi

if contains 'extract_security_hint' "$INVESTMENT_GUARD" || contains 'fallback_symbol_mentions' "$INVESTMENT_GUARD" || contains '"REPEAT",' "$INVESTMENT_GUARD" || contains 'return Some("NBIS".to_string())' "$INVESTMENT_GUARD"; then
  record fail "14.no-ticker-guess-denylist" "legacy ticker guessing, metadata denylist, or hard-coded alias remains"
else
  record success "14.no-ticker-guess-denylist" "legacy ticker guessing, denylist, and hard-coded alias are removed"
fi

if contains 'comparison: bool' "$INVESTMENT_GUARD" && contains '多证券比较门禁' "$INVESTMENT_GUARD" && contains 'missing_investment_response_sections' "$AGENT_CORE"; then
  record success "15.multi-entity-contract" "multi-security turns retain entity and final response contracts"
else
  record fail "15.multi-entity-contract" "multi-security enforcement is incomplete"
fi

if contains 'pub enum AgentTurnOrigin' "$AGENT_TYPES" && contains 'entity_resolution_input = Some(event.task_prompt.clone())' "$SCHEDULER" && contains 'AgentTurnOrigin::Heartbeat' "$SCHEDULER"; then
  record success "16.typed-scheduler-origin" "scheduler metadata is separated from entity-resolution input"
else
  record fail "16.typed-scheduler-origin" "scheduler provenance still depends on prompt text"
fi

if contains 'DeepAnalysisKind::Fund' "$INVESTMENT_GUARD" && contains 'DeepAnalysisKind::Crypto' "$INVESTMENT_GUARD" && contains 'isEtf' "$INVESTMENT_GUARD" && contains '"etf_holdings"' "$INVESTMENT_GUARD" && contains 'missing_deep_fund_sections' "$INVESTMENT_GUARD" && contains 'missing_deep_crypto_sections' "$INVESTMENT_GUARD" && contains 'numbered_section_has_substance' "$INVESTMENT_GUARD" && contains 'forbidden_investment_tool_calls' "$INVESTMENT_GUARD" && contains 'entity_verified_price_appears' "$INVESTMENT_GUARD" && contains 'has_matching_financial_data' "$INVESTMENT_GUARD" && contains 'should_cache_fmp_value' "$DATA_FETCH"; then
  record success "17.asset-aware-fund-preflight" "company, fund, and crypto routing plus substantive output, tool-call, price, evidence, and cache guards are code-gated"
else
  record fail "17.asset-aware-fund-preflight" "ETF/fund requests can regress into the corporate financials route"
fi

if contains 'quote_has_positive_matching_price' "$INVESTMENT_GUARD" && contains 'financials' "$INVESTMENT_GUARD" && contains 'earnings_calendar' "$INVESTMENT_GUARD"; then
  record success "11.deep-stock-data-preflight" "entity, same-symbol quote, financials, and outlook evidence are code-gated"
else
  record fail "11.deep-stock-data-preflight" "deep single-stock data preflight is incomplete"
fi

if contains 'B. 单股深度分析' "$SOUL" && contains 'create_strict_actor_runner' "$EXECUTION"; then
  record success "12.full-prompt-and-safe-runner" "full response contract and actor-bound fallback remain in the repository"
else
  record fail "12.full-prompt-and-safe-runner" "full prompt or strict actor runner regressed"
fi

if contains '所有证券、市场和板块回答的第一条可见内容必须是服务端提供的' "$SOUL" && contains 'first visible line is the server-provided' "$STOCK_RESEARCH" && contains 'first visible line is always the server-owned' "$MARKET_ANALYSIS"; then
  record success "18.server-owned-time-first" "canonical prompt and finance skills keep the server data-time line first"
else
  record fail "18.server-owned-time-first" "time-first response ownership is missing from a canonical prompt layer"
fi

if contains '证券实体发现是不可跳过的证据阶段' "$SOUL" && contains '用户直接输入 `NBIS`、`INTL`、`RMBS` 这类股票代码是正常用法' "$SOUL" && contains 'A plain ticker such as `NBIS`, `INTL`, or `RMBS` is normal user input' "$STOCK_RESEARCH" && contains 'require an exact-symbol result' "$STOCK_RESEARCH" && contains 'agent_discovery_query_is_explicit_symbol' "$INVESTMENT_GUARD" && contains 'missing_required_agent_seed_symbols' "$INVESTMENT_GUARD" && contains 'provider_lookup_variants' "$INVESTMENT_GUARD"; then
  record success "19.plain-ticker-agent-discovery" "plain tickers enter the open Agent loop, preserve exact-symbol lookup, and cannot be silently omitted from the observed search trace"
else
  record fail "19.plain-ticker-agent-discovery" "the prompt or runtime can regress into rejecting, rewriting, guessing, or silently omitting ordinary ticker requests"
fi

if contains '每个公司或证券问题先调用 DataFetch `search`' "$SOUL" && contains 'DataFetch 本轮同代码 quote' "$SOUL" && contains '禁止声称“没有实时行情”' "$SOUL" && contains 'never claim that real-time/current market data was not requested' "$STOCK_RESEARCH"; then
  record success "20.current-data-capability" "DataFetch/search/quote usage and false capability denial are explicitly constrained"
else
  record fail "20.current-data-capability" "the prompt no longer guarantees current-turn market-data usage"
fi

if contains 'B. 单股深度分析' "$SOUL" && contains '7. Bull / Bear / Base Case' "$SOUL" && contains '9. 动作建议：买、等、减、卖、观察，并给触发条件' "$SOUL" && contains '三、估值纪律' "$SOUL" && contains '四、辩证框架' "$SOUL" && contains '六、输出纪律' "$SOUL"; then
  record success "21.large-prompt-single-stock-template" "the pre-71a4498e single-stock, valuation, scenario, and output contracts remain complete"
else
  record fail "21.large-prompt-single-stock-template" "the large prompt was compacted or lost its single-stock contract again"
fi

if contains 'C.1 大盘 / 区域市场 / 跨市场分析' "$SOUL" && contains '2. 已核验行情事实：每个代表标的独立写出同代码现价、涨跌幅与报价时间口径' "$SOUL" && contains '5. 动作建议、触发条件与证伪条件' "$SOUL" && contains '### Broad / Regional Market Output Contract' "$MARKET_ANALYSIS"; then
  record success "22.full-market-template" "broad and mixed-market answers keep their five-section current-evidence template"
else
  record fail "22.full-market-template" "the broad-market response template is incomplete"
fi

if contains 'C. 板块 / 技术 / 产业链分析' "$SOUL" && contains '精确核验至少三个相关代表证券' "$SOUL" && contains '6. 主要上市公司对比：每个代表证券独立写出本轮同代码现价与数据时间口径' "$SOUL" && contains '### Sector / Industry Output Contract' "$MARKET_ANALYSIS"; then
  record success "23.full-sector-template" "sector research keeps representative discovery, exact quotes, and the nine-section template"
else
  record fail "23.full-sector-template" "the sector template or representative-security evidence contract is incomplete"
fi

if contains '本轮公司财务数据未核验' "$SOUL" && contains '本轮公司财务数据未核验' "$STOCK_RESEARCH" && contains '不得从记忆编造收入、利润率、现金流或估值倍数' "$SOUL" && contains 'data_fetch(data_type="quote", ticker="comma-separated exact symbols")' "$MARKET_ANALYSIS" && ! contains 'data_fetch(data_type="market")' "$MARKET_ANALYSIS"; then
  record success "24.layered-missing-data-disclosure" "financial gaps are disclosed without fabrication or a nonexistent market endpoint"
else
  record fail "24.layered-missing-data-disclosure" "missing financials can still be fabricated or widened into a false market-data outage"
fi

if contains 'SecurityIdentifierKind' "$SECURITY_IDENTIFIER" && contains 'provider_lookup_variants' "$SECURITY_IDENTIFIER" && contains 'provider_symbols_equivalent' "$SECURITY_IDENTIFIER" && contains 'digit_leading_composite_is_consumed_without_suffix_rescan' "$SECURITY_IDENTIFIER" && contains 'encode_fmp_symbols' "$DATA_FETCH" && contains 'digit_leading_symbol_never_degrades_to_its_exchange_suffix' "$INVESTMENT_GUARD"; then
  record success "25.cross-market-symbol-canonicalization" "one parser, bounded provider aliases, suffix-rescan prevention, and encoded provider URLs are regression-gated"
else
  record fail "25.cross-market-symbol-canonicalization" "cross-market identifiers can regress into suffix truncation, fuzzy aliases, or unsafe provider URLs"
fi

if contains 'resolve_tentative_named_match' "$INVESTMENT_GUARD" && contains 'candidate_is_embedded_ticker_reference' "$INVESTMENT_GUARD" && contains 'entity_words_start_with' "$INVESTMENT_GUARD" && contains 'GraniteShares YieldBOOST CRWV ETF' "$INVESTMENT_GUARD" && contains 'Appleseed Fund' "$INVESTMENT_GUARD" && contains 'a derivative-only semantic result must not replace a missing exact ticker' "$INVESTMENT_GUARD"; then
  record success "26.exact-ticker-beats-embedded-product-reference" "exact ticker identity cannot be displaced by an embedded-code product, and natural-name fallback uses word boundaries"
else
  record fail "26.exact-ticker-beats-embedded-product-reference" "weak product-name or partial-word matches can again override provider-verified identity"
fi

if contains 'AgentToolDiscovery' "$INVESTMENT_GUARD" && contains '【本轮证券实体发现：主 Agent 工具循环】' "$INVESTMENT_GUARD" && contains 'build_agent_discovered_investment' "$INVESTMENT_GUARD" && contains 'current_agent_discovery_calls' "$INVESTMENT_GUARD" && contains 'missing_agent_discovered_truth_violations' "$INVESTMENT_GUARD" && contains 'agent_discovery_uses_later_exact_searches_after_empty_enriched_attempts' "$INVESTMENT_GUARD" && contains 'agent_discovery_does_not_build_a_ticker_only_subset_for_unlinked_alias_search' "$INVESTMENT_GUARD" && contains 'agent_owned_no_coverage_clarification_is_not_replaced_and_is_emitted_once' "$AGENT_TESTS" && contains 'agent_owned_equal_candidate_clarification_is_not_replaced_and_is_emitted_once' "$AGENT_TESTS" && contains 'optional_agent_contract_failure_preserves_completed_interactive_answer' "$AGENT_TESTS" && contains 'retry_loop_accepts_later_exact_searches_after_empty_enriched_searches' "$AGENT_TESTS" && contains '分析下crwv和nbis的估值' "$INVESTMENT_GUARD" && contains 'main_agent_entity_discovery_input' "$AGENT_CORE" && contains 'DeferredUserOutputEmitter' "$AGENT_CORE" && contains 'contract_built=false answer_preserved=true' "$AGENT_CORE" && contains '!agent_discovered_contract' "$AGENT_CORE" && [[ "$AGENT_DISCOVERY_IMPL" != *'response_intent('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'is_strict_quote_only_request('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'response_requests_extended_hours_quote('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'.enforcement_block()'* ]] && [[ "$AGENT_TRUTH_IMPL" != *'unsupported_financial_fact_claims('* ]] && [[ "$AGENT_TRUTH_IMPL" != *'unsupported_fund_fact_claims('* ]] && [[ "$AGENT_TRUTH_IMPL" != *'unsupported_recent_event_fact('* ]] && ! contains 'first_agent_discovery_calls' "$INVESTMENT_GUARD" && ! contains 'agent_discovery_disposition' "$INVESTMENT_GUARD" && ! contains 'UNSAFE_AGENT_DISCOVERY_MESSAGE' "$AGENT_CORE" && ! contains 'AgentDiscoveryDisposition' "$AGENT_CORE" && ! contains 'request_may_need_auxiliary_entity_extraction' "$INVESTMENT_GUARD" && ! contains 'ENTITY_EXTRACTION_TIMEOUT_SECS' "$INVESTMENT_GUARD" && ! contains 'entity_extraction_unavailable_message' "$INVESTMENT_GUARD"; then
  record success "27.agent-loop-entity-discovery" "the main Agent owns open-ended scope and evidence depth; iterative search refinements are retained and an optional contract can never become a fixed refusal"
else
  record fail "27.agent-loop-entity-discovery" "interactive discovery can regress into first-round freezing, a fixed refusal, wording classification, format takeover, or replayed drafts"
fi

echo
echo "summary: success=$success review=$review fail=$fail total=$((success + review + fail))"

if [ "$success" -lt 27 ]; then
  echo "[ERROR] acceptance failed: expected all 27 successes"
  exit 1
fi

if [ "$fail" -gt 0 ]; then
  echo "[ERROR] acceptance failed: expected no failures"
  exit 1
fi
