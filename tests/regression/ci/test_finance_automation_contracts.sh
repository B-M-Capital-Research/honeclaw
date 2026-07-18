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
AGENT_EMITTER="crates/hone-channels/src/agent_session/emitter.rs"
AGENT_TESTS="crates/hone-channels/src/agent_session/tests.rs"
CLI_PROBE="bins/hone-cli/src/probe.rs"
FUNCTION_AGENT="agents/function_calling/src/lib.rs"
RUN_EVENT="crates/hone-channels/src/run_event.rs"
TOOL_REASONING_RUNNER="crates/hone-channels/src/runners/tool_reasoning.rs"
OPENROUTER="crates/hone-llm/src/openrouter.rs"
OPENAI_COMPATIBLE="crates/hone-llm/src/openai_compatible.rs"
LLM_PROVIDER="crates/hone-llm/src/provider.rs"
WEB_CHAT="crates/hone-web-api/src/routes/chat.rs"
WEB_PUBLIC="crates/hone-web-api/src/routes/public.rs"
PUBLIC_CHAT_STATE="packages/app/src/lib/public-chat.ts"
PUBLIC_CHAT_TYPES="packages/app/src/lib/types.ts"
SCHEDULER="crates/hone-channels/src/scheduler.rs"
SOUL="soul.md"
AGENT_DISCOVERY_IMPL="$(sed -n '/pub(crate) fn build_agent_discovered_investment(/,/^fn tool_call_targets_entity(/p' "$INVESTMENT_GUARD")"
AGENT_DISCOVERY_CONTEXT_IMPL="$(sed -n '/fn append_agent_entity_discovery_context(/,/^fn explicit_dollar_mentions(/p' "$INVESTMENT_GUARD")"
INTERACTIVE_OBSERVATION_IMPL="$(sed -n '/Interactive entity discovery is owned/,/let Some(contract) = contract else/p' "$AGENT_CORE")"

echo "[finance-automation-contracts] fixed sample count: 32"

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

if contains 'missing_deep_single_stock_sections' "$INVESTMENT_GUARD" && contains 'missing_deep_fund_sections' "$INVESTMENT_GUARD" && contains 'missing_deep_crypto_sections' "$INVESTMENT_GUARD" && contains 'let Some(contract) = contract else' "$AGENT_CORE" && contains 'missing_investment_response_sections' "$AGENT_CORE" && contains 'enforce_server_data_time_prefix' "$AGENT_CORE" && ! contains 'missing_agent_discovered_truth_violations' "$INVESTMENT_GUARD" && ! contains 'agent_discovered_contract' "$AGENT_CORE"; then
  record success "10.typed-deep-stock-response-contract" "typed scheduled/heartbeat work retains strict asset-aware validation while Interactive observations stay outside that enforcement path"
else
  record fail "10.typed-deep-stock-response-contract" "typed deep validation is missing or Interactive discovery can still enter the strict rewrite path"
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

if contains '服务端不会在成功后追加任何用户可见内容、改写答案、重跑主 Agent 或否决这个成功答案' "$SOUL" && contains '必须由主 Agent 自己把“数据时间：北京时间' "$PROMPT_FILE" && [[ "$AGENT_DISCOVERY_CONTEXT_IMPL" == *'hone_core::beijing_now()'* ]] && [[ "$AGENT_DISCOVERY_CONTEXT_IMPL" == *'【本轮最终回答契约：由主 Agent 一次完成】'* ]] && [[ "$AGENT_DISCOVERY_CONTEXT_IMPL" == *'第一可见字符必须是“数”'* ]] && [[ "$AGENT_DISCOVERY_CONTEXT_IMPL" == *'禁止在该行之前输出 `---`、Markdown 标题'* ]] && [[ "$AGENT_DISCOVERY_CONTEXT_IMPL" == *'否则忽略本节格式，正常回答用户原问题'* ]] && contains 'After success, the service will not append any user-visible content, rewrite the answer, rerun the main Agent, or reject that successful answer' "$STOCK_RESEARCH" && contains 'Time anchor first and Interactive answer ownership' "$MARKET_ANALYSIS" && contains 'first visible line' "$MARKET_ANALYSIS" && ! contains 'server-provided' "$STOCK_RESEARCH" && ! contains 'server-owned' "$STOCK_RESEARCH" && ! contains 'server-provided' "$MARKET_ANALYSIS" && ! contains 'server-owned' "$MARKET_ANALYSIS"; then
  record success "18.agent-owned-time-first" "the main Agent authors the time-first Interactive answer and no finance skill delegates that line to a post-processor"
else
  record fail "18.agent-owned-time-first" "time-first ownership can regress to a server-authored prefix or disappear from a canonical prompt layer"
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

if contains '本轮公司财务数据未核验' "$SOUL" && contains '本轮公司财务数据未核验' "$STOCK_RESEARCH" && contains '不得从记忆编造收入、利润率、现金流、净债务或估值倍数' "$SOUL" && contains 'data_fetch(data_type="quote", ticker="comma-separated exact symbols")' "$MARKET_ANALYSIS" && ! contains 'data_fetch(data_type="market")' "$MARKET_ANALYSIS"; then
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

if contains 'AgentToolDiscovery' "$INVESTMENT_GUARD" && contains '【本轮证券实体发现：主 Agent 工具循环】' "$INVESTMENT_GUARD" && contains 'build_agent_discovered_investment' "$INVESTMENT_GUARD" && contains 'current_agent_discovery_calls' "$INVESTMENT_GUARD" && contains 'agent_discovery_uses_later_exact_searches_after_empty_enriched_attempts' "$INVESTMENT_GUARD" && contains 'agent_discovery_does_not_build_a_ticker_only_subset_for_unlinked_alias_search' "$INVESTMENT_GUARD" && contains 'agent_owned_no_coverage_clarification_is_not_replaced_and_is_emitted_once' "$AGENT_TESTS" && contains 'agent_owned_equal_candidate_clarification_is_not_replaced_and_is_emitted_once' "$AGENT_TESTS" && contains 'optional_agent_observation_preserves_completed_interactive_answer' "$AGENT_TESTS" && contains 'omitted_explicit_seed_is_observational_and_does_not_rerun' "$AGENT_TESTS" && contains 'single_agent_loop_accepts_later_exact_searches_after_empty_enriched_searches' "$AGENT_TESTS" && contains 'interactive_observed_crwv_nvidia_answer_is_never_repaired_or_rewritten' "$AGENT_TESTS" && contains 'crwv和英伟达什么关系，估值怎么看' "$AGENT_TESTS" && contains 'quote-stale-nbis' "$AGENT_TESTS" && contains '73.21 USD' "$AGENT_TESTS" && contains '约 73 USD' "$AGENT_TESTS" && contains 'interactive_runtime_history_drops_scheduler_and_failed_turn_groups' "$AGENT_TESTS" && contains 'main_agent_entity_discovery_input' "$AGENT_CORE" && contains 'DeferredUserOutputEmitter' "$AGENT_CORE" && contains 'mode=observational' "$AGENT_CORE" && contains 'answer_preserved=true' "$AGENT_CORE" && [[ "$INTERACTIVE_OBSERVATION_IMPL" == *'return result;'* ]] && [[ "$INTERACTIVE_OBSERVATION_IMPL" != *'response.success = false'* ]] && [[ "$INTERACTIVE_OBSERVATION_IMPL" != *'enforce_server_data_time_prefix'* ]] && [[ "$INTERACTIVE_OBSERVATION_IMPL" != *'missing_investment_response_sections'* ]] && [[ "$INTERACTIVE_OBSERVATION_IMPL" != *'runtime_input.push_str'* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'response_intent('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'is_strict_quote_only_request('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'response_requests_extended_hours_quote('* ]] && [[ "$AGENT_DISCOVERY_IMPL" != *'.enforcement_block()'* ]] && ! contains 'missing_agent_discovered_truth_violations' "$INVESTMENT_GUARD" && ! contains 'agent_truth_retry_block' "$INVESTMENT_GUARD" && ! contains 'entity_resolution.agent_loop.retry' "$AGENT_CORE" && ! contains 'agent_discovered_contract' "$AGENT_CORE" && ! contains 'first_agent_discovery_calls' "$INVESTMENT_GUARD" && ! contains 'agent_discovery_disposition' "$INVESTMENT_GUARD" && ! contains 'UNSAFE_AGENT_DISCOVERY_MESSAGE' "$AGENT_CORE" && ! contains 'AgentDiscoveryDisposition' "$AGENT_CORE" && ! contains 'request_may_need_auxiliary_entity_extraction' "$INVESTMENT_GUARD" && ! contains 'ENTITY_EXTRACTION_TIMEOUT_SECS' "$INVESTMENT_GUARD" && ! contains 'entity_extraction_unavailable_message' "$INVESTMENT_GUARD" && ! contains '.with_restore_max_messages(None)' "$WEB_CHAT" && ! contains '.with_restore_max_messages(None)' "$WEB_PUBLIC"; then
  record success "27.agent-loop-entity-discovery" "Interactive discovery is observational only: one Agent loop owns refinement and the original answer survives stale traces, formatting gaps, and attempt-local events without retry, rewrite, refusal, or unbounded Web restore"
else
  record fail "27.agent-loop-entity-discovery" "Interactive discovery can regress into a second runner, post-hoc validation/rewrite, fixed refusal, attempt-event flash, or unbounded polluted history"
fi

if contains 'finish_research' "$FUNCTION_AGENT" && contains 'chat_terminal_streaming' "$FUNCTION_AGENT" && contains 'on_final_content_delta' "$FUNCTION_AGENT" && contains 'with_finish_research_terminal_synthesis' "$TOOL_REASONING_RUNNER" && contains 'TerminalStreamPolicy::CanonicalInvestmentHeader' "$TOOL_REASONING_RUNNER" && contains 'CommittedStreamDelta' "$RUN_EVENT" && contains 'CommittedStreamDelta' "$AGENT_EMITTER" && contains 'committed_visible_prefix.is_some()' "$AGENT_CORE" && contains 'committed_visible_prefix.is_none()' "$AGENT_CORE" && contains 'committed_terminal_prefix_makes_runner_attempt_irreversible_and_suppresses_retry' "$AGENT_TESTS" && contains 'the early committed header plus the terminal tail must exactly equal the persisted answer' "$AGENT_TESTS" && contains 'remove_tool_fields_without_tools' "$OPENROUTER" && contains 'remove_tool_fields_without_tools' "$OPENAI_COMPATIBLE"; then
  record success "28.agent-signaled-terminal-stream" "tool-capable drafts remain deferred while an isolated Agent finish decision enables one empty-tools terminal stream, one canonical committed line, exact persisted suffix, and no outer retry/replay after commit"
else
  record fail "28.agent-signaled-terminal-stream" "terminal streaming can regress into speculative draft exposure, tool-bearing final synthesis, prefix rewrite, or retry after visible output"
fi

if contains 'investment_research_started' "$FUNCTION_AGENT" && contains 'starts_investment_research_protocol' "$FUNCTION_AGENT" && contains 'eq_ignore_ascii_case("data_fetch")' "$FUNCTION_AGENT" && contains 'non_finance_web_search_does_not_activate_the_investment_terminal_protocol' "$FUNCTION_AGENT" && contains 'ToolChoiceMode::Required' "$FUNCTION_AGENT" && contains 'tool_choice_mode == ToolChoiceMode::Required' "$OPENROUTER" && contains 'tool_choice_mode == ToolChoiceMode::Required' "$OPENAI_COMPATIBLE" && contains 'CONTINUE_RESEARCH_TOOL_NAME' "$FUNCTION_AGENT" && contains 'chat_research_control_decision' "$FUNCTION_AGENT" && contains 'content_bypass' "$FUNCTION_AGENT" && contains 'ACTIVE_BUSINESS_TIMEOUT' "$FUNCTION_AGENT" && contains 'requested_tool_choice = tool_choice_mode_name(telemetry.requested)' "$FUNCTION_AGENT" && contains 'session_id = %context.session_id' "$FUNCTION_AGENT" && contains 'active_business_timeout' "$FUNCTION_AGENT" && contains 'active_business_provider_error_degrades_to_terminal_from_existing_evidence' "$FUNCTION_AGENT" && contains 'data_fetch_in_an_eligible_turn_activates_the_control_protocol' "$FUNCTION_AGENT" && contains '必要证据已明确不可得且可在答案中如实披露' "$FUNCTION_AGENT" && contains '数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：' "$FUNCTION_AGENT" && contains '未明确标注 forward 时不得称为 Forward PE' "$FUNCTION_AGENT" && contains '关系、事件与估值证据纪律' "$PROMPT_FILE" && contains '搜索摘要明确陈述的有限事实只能按原范围使用' "$SOUL" && contains '未取得资产负债表中的现金、债务或可直接使用的企业价值' "$SOUL" && contains 'this metadata does not establish a market session' "$DATA_FETCH" && contains '只有 `extended_hours` 的规范化 bar 可以核验美股扩展时段' "$DATA_FETCH" && contains '年度数据不得写成 TTM' "$FUNCTION_AGENT" && contains '输入不完整时使用一种可严谨计算的方法并明确披露缺项，禁止补数' "$INVESTMENT_GUARD" && contains 'disclosed_valuation_gap' "$INVESTMENT_GUARD"; then
  record success "29.required-terminal-and-evidence-ledger" "eligible investment tool use alternates isolated control and bounded business rounds, provider bypasses converge on terminal synthesis, and relationship/valuation/time claims stay evidence-grounded"
else
  record fail "29.required-terminal-and-evidence-ledger" "the Agent can bypass terminal streaming or fabricate relationship, valuation, or quote-time claims from incomplete evidence"
fi

if contains 'ToolChoiceMetadata {' "$LLM_PROVIDER" && contains 'Finish(ChatStreamFinishReason)' "$LLM_PROVIDER" && contains 'Done,' "$LLM_PROVIDER" && contains 'top_level_stream_error' "$LLM_PROVIDER" && contains 'null_top_level_error_is_not_treated_as_a_provider_failure' "$LLM_PROVIDER" && contains 'stream_eof_without_done_remains_detectable' "$OPENROUTER" && contains 'required_stream_retries_same_client_once_in_auto_on_capability_rejection' "$OPENROUTER" && contains 'required_stream_retries_same_client_once_without_required_when_unsupported' "$OPENAI_COMPATIBLE" && contains 'control_stream_missing_done_degrades_with_control_error_reason' "$FUNCTION_AGENT" && contains 'active_stream_missing_done_degrades_with_active_error_reason' "$FUNCTION_AGENT" && contains 'terminal_stream_requires_stop_and_done' "$FUNCTION_AGENT" && contains 'non_success_stream_finish_reasons_are_errors' "$FUNCTION_AGENT" && contains 'effective_tool_choice' "$FUNCTION_AGENT" && contains 'tool_choice_fallback' "$FUNCTION_AGENT"; then
  record success "30.native-stream-lifecycle" "provider tool-choice fallback, finish reason, top-level errors, and DONE are explicit protocol events consumed by every Agent phase"
else
  record fail "30.native-stream-lifecycle" "an incomplete, downgraded, or provider-error stream can again be accepted as a completed Agent round"
fi

if contains 'visible_control_preamble_followed_by_continue_is_silently_accepted' "$FUNCTION_AGENT" && contains 'visible_control_preamble_followed_by_finish_is_silently_accepted' "$FUNCTION_AGENT" && contains 'terminal reasoning must not persist into cross-turn context' "$FUNCTION_AGENT" && contains 'turn_message_start' "$FUNCTION_AGENT" && contains 'stale prior-turn ticker/price evidence reached terminal synthesis' "$FUNCTION_AGENT" && contains 'terminal_scrubs_tool_round_drafts_that_precede_data_fetch_activation' "$FUNCTION_AGENT" && contains 'terminal_content_rejects_header_only_and_duplicate_committed_prefix' "$FUNCTION_AGENT" && contains 'committed_terminal_prefix_recovers_once_without_restreaming_or_rerunning_tools' "$FUNCTION_AGENT" && contains 'committed_terminal_prefix_recovery_mismatch_fails_after_exactly_one_attempt' "$FUNCTION_AGENT" && contains 'chat_terminal_recovery_without_tools' "$FUNCTION_AGENT" && contains 'committed_visible_prefix(&self) -> Option<String>' "$FUNCTION_AGENT" && contains 'committed_visible_prefix(&self) -> Option<String>' "$TOOL_REASONING_RUNNER" && contains 'explicit_provider_error_text' "$LLM_PROVIDER" && contains 'committed_terminal_header_recovers_in_place_and_session_emits_only_the_tail' "$AGENT_TESTS" && contains 'committed_terminal_header_double_failure_emits_honest_partial_and_persists_visible_prefix' "$AGENT_TESTS" && contains 'PartialDone {' "$AGENT_TYPES" && contains 'response: AgentResponse' "$AGENT_TYPES" && contains 'AgentSessionEvent::PartialDone' "$AGENT_CORE" && contains 'terminal_stream_incomplete' "$AGENT_CORE" && contains 'partial_done_preserves_streamed_content_without_claiming_success_or_flashing_error' "$WEB_CHAT" && contains 'publicChatTerminalEventPatch' "$PUBLIC_CHAT_STATE" && contains 'public_api_failure_message' "$WEB_PUBLIC" && contains 'public_api_finish_reason' "$WEB_PUBLIC" && contains 'public_openai_partial_done_does_not_become_success_or_a_second_content_chunk' "$WEB_PUBLIC" && contains 'committed_visible_prefix.is_none()' "$AGENT_CORE"; then
  record success "31.isolated-control-and-terminal-recovery" "control/tool drafts stay outside terminal evidence, provider request echoes cannot widen Required fallback, and committed-prefix failures recover once or end as an honest exact-byte partial without UI flash or API success"
else
  record fail "31.isolated-control-and-terminal-recovery" "research control, provider fallback, terminal evidence, or committed-prefix recovery can regress into draft pollution, tool reruns, duplicate headers, raw errors, refresh mismatch, or error flicker"
fi

if contains 'AgentSessionEvent::PartialDone { response }' "$CLI_PROBE" && contains '[partial_done] success=false' "$CLI_PROBE" && contains 'partial?: boolean' "$PUBLIC_CHAT_TYPES"; then
  record success "32.partial-terminal-consumer-contract" "CLI diagnostics and Browser event types consume a partial terminal without presenting it as success"
else
  record fail "32.partial-terminal-consumer-contract" "a typed PartialDone can again break a workspace consumer or be mistaken for a successful Browser completion"
fi

echo
echo "summary: success=$success review=$review fail=$fail total=$((success + review + fail))"

if [ "$success" -lt 32 ]; then
  echo "[ERROR] acceptance failed: expected all 32 successes"
  exit 1
fi

if [ "$fail" -gt 0 ]; then
  echo "[ERROR] acceptance failed: expected no failures"
  exit 1
fi
