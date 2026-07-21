//! `AgentSession` struct + 所有实例方法 + per-session 运行锁。
//!
//! 这个文件是 module 里的「大脑」:把 types / helpers / emitter / guard /
//! restore / progress 这些零件组合成「一次完整对话」的 pipeline。
//! 详细的 pipeline 步骤见 [`AgentSession::run`] 顶部的分节注释。

use chrono::{DateTime, FixedOffset};
use hone_core::agent::{
    AgentContext, AgentMessage, AgentResponse, NormalizedConversationMessage,
    NormalizedConversationPart, ToolCallMade,
};
use hone_core::{ActorIdentity, SessionIdentity};
use hone_memory::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, session_message_from_normalized,
};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use crate::HoneBotCore;
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService, PreparedExecution,
};
use crate::investment_response_guard::{
    InvestmentResponseContract, build_agent_discovered_investment, contract_failure_message,
    deterministic_investment_fallback_response, enforce_server_data_time_prefix,
    forbidden_investment_tool_calls, has_main_agent_entity_discovery_seed,
    investment_contract_failure_message, investment_preflight_failure_message,
    missing_investment_response_sections, missing_required_agent_seed_symbols,
    prepare_verified_investment_turn, should_emit_investment_preflight,
    uses_main_agent_entity_discovery,
};
use crate::prompt::PromptOptions;
use crate::prompt_audit::PromptAuditMetadata;
use crate::response_finalizer::{
    EMPTY_SUCCESS_FALLBACK_MESSAGE, finalize_agent_owned_interactive_response,
    finalize_agent_response,
};
use crate::runners::{
    AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    ServiceOwnedInitialPrefix, TerminalStreamPolicy,
};
use crate::runtime::{sanitize_user_visible_output, user_visible_error_message};
use crate::session_compactor::SessionCompactor;
use crate::tool_trace::{
    PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE, PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE,
    UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE, persistent_side_effect_state_is_uncertain,
    response_has_only_known_read_only_calls, response_has_persistent_side_effect,
};
use crate::turn_builder::{PromptTurnBuilder, SlashSkillExpansion};

use super::artifacts::attach_web_generated_files;
use super::emitter::{DeferredUserOutputEmitter, SessionEventEmitter};
use super::guard::QuotaReservationGuard;
use super::helpers::{
    CONTEXT_OVERFLOW_FALLBACK_MESSAGE, CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT,
    CONTEXT_OVERFLOW_RECOVERY_LIMIT, CompactCommand, EMPTY_SUCCESS_RETRY_LIMIT,
    TRANSIENT_RUNNER_FAILURE_RETRY_LIMIT, is_context_overflow_error_text,
    is_retryable_transient_runner_failure, merge_message_metadata, non_finance_boundary_reply,
    persistable_turn_from_response, prune_historical_tool_protocol,
    prune_interactive_runtime_history, restore_limit_before_compaction,
    should_return_runner_result,
};
use super::progress::{progress_watchdog_tick, run_with_progress_ticks};
use super::restore::{restore_context_from_snapshot, restore_recent_interactive_user_references};
use super::types::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionError, AgentSessionErrorKind,
    AgentSessionEvent, AgentSessionListener, AgentSessionResult, AgentTurnOrigin,
    GeminiStreamOptions, MessageMetadata, session_error_event, session_progress_event,
};

#[derive(Clone)]
pub(super) struct PreparedInvestmentContext {
    contract: Option<InvestmentResponseContract>,
    runtime_suffix: String,
    prompt_time_beijing: DateTime<FixedOffset>,
    reexecution_policy: PreparedTurnReexecutionPolicy,
    main_agent_entity_discovery_input: Option<String>,
}

struct RestoredRuntimeContext {
    context: AgentContext,
    session_metadata: HashMap<String, Value>,
}

pub(super) fn prompt_time_for_attempt(
    prepared_prompt_time: Option<DateTime<FixedOffset>>,
    current_prompt_time: DateTime<FixedOffset>,
) -> DateTime<FixedOffset> {
    prepared_prompt_time.unwrap_or(current_prompt_time)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedTurnReexecutionPolicy {
    Allowed,
    ExecuteOnce,
}

pub(super) const SERVICE_OWNED_PREFIX_FAILURE_SUFFIX: &str =
    "\n\n本轮研究未能完成，暂未形成可供参考的标的结论。";

/// Persistent mutations are deliberately classified conservatively before the
/// runner starts. Observed tool traces are a second defense, but an ACP
/// disconnect can lose the trace after a write; a false positive here only
/// disables an automatic retry for the current turn.
pub(super) fn prepared_turn_reexecution_policy(input: &str) -> PreparedTurnReexecutionPolicy {
    let normalized = input.to_ascii_lowercase();
    let compact = normalized
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let research_follow_up = [
        "怎么看",
        "为什么",
        "原因",
        "分析",
        "研究",
        "怎么",
        "如何",
        "是否",
        "适合",
        "应该",
        "该不该",
        "能不能",
        "能买吗",
        "值不值得",
        "analyze",
        "analysis",
        "research",
        "why",
        "how",
        "what",
        "review",
        "should",
        "whether",
    ]
    .iter()
    .any(|marker| compact.contains(marker));

    let explicit_mutation_phrase = [
        "加入自选",
        "加到自选",
        "放进自选",
        "添加自选",
        "移出自选",
        "删除自选",
        "帮我关注",
        "加入关注",
        "添加关注",
        "取消关注",
        "不再关注",
        "移出关注",
        "放进持仓",
        "加到持仓",
        "记录持仓",
        "加入持仓",
        "添加持仓",
        "更新持仓",
        "修改持仓",
        "删除持仓",
        "移出持仓",
        "清空持仓",
        "清空自选",
        "清空提醒",
        "创建提醒",
        "设置提醒",
        "新增提醒",
        "删除提醒",
        "取消提醒",
        "提醒我",
        "创建定时",
        "新增定时",
        "修改定时",
        "删除定时",
        "取消定时",
        "开启推送",
        "关闭推送",
        "设置推送",
        "修改推送",
        "设置勿扰",
        "关闭勿扰",
        "重启hone",
        "重启服务",
        "restarthone",
        "restart_hone",
    ]
    .iter()
    .any(|marker| compact.contains(marker));
    let english_object_mutation = [
        "add", "remove", "delete", "update", "watch", "unwatch", "create", "set", "cancel",
        "clear", "put", "move", "increase", "reduce",
    ]
    .iter()
    .any(|marker| {
        Regex::new(&format!(r"(?i)\b{}\b", regex::escape(marker)))
            .expect("english persistent mutation regex")
            .is_match(&normalized)
    }) && [
        "watchlist",
        "portfolio",
        "holding",
        "holdings",
        "position",
        "positions",
        "alert",
        "schedule",
        "notification",
        "do not disturb",
    ]
    .iter()
    .any(|object| normalized.contains(object));
    let mutation_then_research = [
        "并分析",
        "然后分析",
        "再分析",
        "并研究",
        "然后研究",
        "再研究",
        "后分析",
        "后再分析",
        "后研究",
        "and analyze",
        "then analyze",
        "and review",
        "then review",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    let direct_operation_request = ["帮我", "请", "给我", "please", "could you", "can you"]
        .iter()
        .any(|marker| normalized.contains(marker));
    let completed_chinese_operation_statement = [
        "我买了",
        "已经买了",
        "我刚买入",
        "我买入",
        "已经买入",
        "我卖出",
        "已经卖出",
        "我加仓",
        "已经加仓",
        "我减仓",
        "已经减仓",
        "我清仓",
        "已经清仓",
    ]
    .iter()
    .any(|marker| compact.contains(marker));
    let completed_english_operation_statement = [
        "i bought",
        "i just bought",
        "i sold",
        "i just sold",
        "i increased my position",
        "i reduced my position",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    let completed_operation_statement =
        completed_chinese_operation_statement || completed_english_operation_statement;
    let has_trade_or_position_verb = ["买入", "卖出", "加仓", "减仓", "清仓"]
        .iter()
        .any(|marker| compact.contains(marker))
        || Regex::new(r"(?i)\b(?:buy|sell|increase|reduce)\b")
            .expect("trade or position verb regex")
            .is_match(&normalized);
    let explicit_trade_recommendation_question = has_trade_or_position_verb
        && ([
            "是否",
            "适合买",
            "适合卖",
            "应该",
            "该不该",
            "能不能",
            "能买吗",
            "卖吗",
            "买吗",
            "值不值得",
            "should",
            "whether",
            "would you",
            "do you think",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
            || Regex::new(r"(?i)^\s*is\b[^?\r\n]{0,80}\ba\s+(?:buy|sell)\b")
                .expect("buy or sell recommendation question regex")
                .is_match(&normalized)
            || (normalized.trim_end().ends_with('?')
                && !Regex::new(r"(?i)^\s*(?:(?:please|can\s+you|could\s+you)\s+)?(?:buy|sell)\b")
                    .expect("direct English trade request regex")
                    .is_match(&normalized)));
    let direct_chinese_trade_request = [
        "帮我买入",
        "帮我卖出",
        "帮我加仓",
        "帮我减仓",
        "帮我清仓",
        "请买入",
        "请卖出",
        "请加仓",
        "请减仓",
        "请清仓",
        "给我买入",
        "给我卖出",
        "给我加仓",
        "给我减仓",
        "给我清仓",
    ]
    .iter()
    .any(|marker| compact.contains(marker));
    let imperative_chinese_trade = has_trade_or_position_verb
        && !explicit_trade_recommendation_question
        && (direct_chinese_trade_request
            || ["买入", "卖出", "加仓", "减仓", "清仓"]
                .iter()
                .any(|marker| compact.starts_with(marker))
            || compact.starts_with('把'));
    let imperative_english_trade = !explicit_trade_recommendation_question
        && Regex::new(r"(?i)^\s*(?:(?:please|can\s+you|could\s+you)\s+)?(?:buy|sell)\b")
            .expect("imperative English trade regex")
            .is_match(&normalized);
    let explicit_mutation = (explicit_mutation_phrase || english_object_mutation)
        && !explicit_trade_recommendation_question
        && (!research_follow_up
            || direct_operation_request
            || completed_operation_statement
            || mutation_then_research);
    let generalized_persistent_mutation = !research_follow_up
        && !explicit_trade_recommendation_question
        && [
            "添加", "加入", "放进", "加到", "记录", "更新", "修改", "删除", "移出", "移除", "取消",
            "清空", "买入", "卖出", "加仓", "减仓", "清仓",
        ]
        .iter()
        .any(|marker| compact.contains(marker))
        && ["持仓", "自选", "关注", "提醒", "定时任务", "推送", "勿扰"]
            .iter()
            .any(|marker| compact.contains(marker));
    let completed_holding_removal =
        compact.contains("不持有") && compact.contains('了') && !research_follow_up;
    let directed_scheduled_delivery = ["每天", "每周", "每个交易日", "今晚", "明早", "明天"]
        .iter()
        .any(|marker| compact.contains(marker))
        && compact.contains("给我")
        && ["提醒", "播报", "发送", "推送", "发一", "看"]
            .iter()
            .any(|marker| compact.contains(marker));
    let has_deep_research_request =
        compact.contains("深度研究") || normalized.contains("deep research");
    let deep_research_question = has_deep_research_request
        && [
            "是什么",
            "什么是",
            "如何",
            "怎么",
            "为什么",
            "what",
            "how",
            "why",
        ]
        .iter()
        .any(|marker| normalized.contains(marker));
    let explicit_deep_research_start = !deep_research_question
        && (has_deep_research_request
            && (["启动", "创建", "发起", "开始", "帮我", "请", "做深度研究"]
                .iter()
                .any(|marker| compact.contains(marker))
                || compact.starts_with("深度研究"))
            || Regex::new(
                r"(?i)^\s*(?:(?:please|can\s+you|could\s+you)\s+)?(?:(?:start|create|run|launch|do)\s+(?:a\s+)?)?deep\s+research\b",
            )
            .expect("explicit deep research start regex")
            .is_match(&normalized));

    if explicit_mutation
        || completed_operation_statement
        || imperative_chinese_trade
        || imperative_english_trade
        || generalized_persistent_mutation
        || completed_holding_removal
        || directed_scheduled_delivery
        || explicit_deep_research_start
    {
        PreparedTurnReexecutionPolicy::ExecuteOnce
    } else {
        PreparedTurnReexecutionPolicy::Allowed
    }
}

fn mark_investment_attempt_output_deferred(result: &mut AgentRunnerResult) {
    // Runner flags describe what the attempt emitted. Investment attempts use a
    // deferred emitter. A typed committed prefix is the sole exception: it was
    // already ACKed at an irreversible Web boundary; only known-read-only
    // finance work may continue afterward.
    result.streamed_output = result.committed_visible_prefix.is_some();
    result.terminal_error_emitted = false;
}

/// Keep the finalized response byte-aligned with an already ACKed prefix.
/// Providers may leave whitespace behind after hidden reasoning is stripped;
/// only that leading whitespace is removable. Any non-whitespace content or
/// a genuinely different prefix must still fail closed.
pub(super) fn align_response_to_committed_prefix(
    response: &mut AgentResponse,
    prefix: &str,
) -> bool {
    if response.content.starts_with(prefix) {
        return true;
    }
    let leading_whitespace_bytes = response.content.len() - response.content.trim_start().len();
    if leading_whitespace_bytes == 0
        || !response.content[leading_whitespace_bytes..].starts_with(prefix)
    {
        return false;
    }
    response.content.drain(..leading_whitespace_bytes);
    true
}

fn normalize_execute_once_failure(
    result: &mut AgentRunnerResult,
    reexecution_policy: PreparedTurnReexecutionPolicy,
) {
    if reexecution_policy != PreparedTurnReexecutionPolicy::ExecuteOnce || result.response.success {
        return;
    }
    result.response.content = PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string();
    result.response.error = Some(PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string());
    result.streamed_output = result.committed_visible_prefix.is_some();
    result.terminal_error_emitted = false;
}

pub(super) fn normalize_persistent_trace_failure(result: &mut AgentRunnerResult) {
    if result.response.success
        || !response_has_persistent_side_effect(&result.response.tool_calls_made)
    {
        return;
    }
    result.response.content = PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string();
    result.response.error = Some(PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string());
    result.streamed_output = result.committed_visible_prefix.is_some();
    result.terminal_error_emitted = false;
}

fn preserve_verified_investment_failure(
    contract: &InvestmentResponseContract,
    result: &mut AgentRunnerResult,
) {
    if result.response.success {
        return;
    }
    let message = result
        .response
        .error
        .as_deref()
        .filter(|message| !message.trim().is_empty())
        .or_else(|| {
            (!result.response.content.trim().is_empty()).then_some(result.response.content.as_str())
        })
        .unwrap_or(contract_failure_message());
    let failure = investment_contract_failure_message(contract, message);
    result.response.content = failure.clone();
    result.response.error = Some(failure);
    result.streamed_output = result.committed_visible_prefix.is_some();
    result.terminal_error_emitted = false;
}

fn unsafe_investment_repair_trace_message(tool_calls: &[ToolCallMade]) -> Option<&'static str> {
    let trace_is_known_read_only = response_has_only_known_read_only_calls(tool_calls);
    let has_persistent_side_effect = response_has_persistent_side_effect(tool_calls);
    if trace_is_known_read_only && !has_persistent_side_effect {
        return None;
    }
    if !trace_is_known_read_only && !has_persistent_side_effect {
        return Some(UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE);
    }
    if persistent_side_effect_state_is_uncertain(tool_calls) {
        Some(PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE)
    } else {
        Some(PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE)
    }
}

fn prepend_investment_attempt_tool_trace(
    result: &mut AgentRunnerResult,
    mut previous_attempt_tool_calls: Vec<ToolCallMade>,
) {
    previous_attempt_tool_calls.append(&mut result.response.tool_calls_made);
    result.response.tool_calls_made = previous_attempt_tool_calls;
}

pub(super) fn apply_deterministic_investment_fallback(
    contract: &InvestmentResponseContract,
    result: &mut AgentRunnerResult,
) -> bool {
    if result.committed_visible_prefix.is_some() {
        return false;
    }
    if !response_has_only_known_read_only_calls(&result.response.tool_calls_made) {
        return false;
    }
    let Some(fallback) = deterministic_investment_fallback_response(contract) else {
        return false;
    };
    let sanitized = sanitize_user_visible_output(&fallback);
    if sanitized.content.trim().is_empty()
        || sanitized.only_internal
        || sanitized.removed_internal
        || !missing_investment_response_sections(contract, &sanitized.content).is_empty()
    {
        return false;
    }

    result.response.content = sanitized.content;
    result.response.success = true;
    result.response.error = None;
    // The rejected model draft is not part of the accepted turn. Its read-only
    // tool trace and normalized transcript must not be persisted as if they
    // supported the deterministic answer.
    result.response.tool_calls_made.clear();
    result.context_messages = None;
    result.streamed_output = false;
    result.terminal_error_emitted = false;
    true
}

pub struct AgentSession {
    pub(super) core: Arc<HoneBotCore>,
    pub(super) actor: ActorIdentity,
    pub(super) session_identity: SessionIdentity,
    pub(super) session_id: String,
    pub(super) channel_target: String,
    pub(super) message_id: Option<String>,
    pub(super) restore_max_messages: Option<usize>,
    pub(super) prompt_options: PromptOptions,
    pub(super) session_metadata: Option<HashMap<String, Value>>,
    pub(super) message_metadata: MessageMetadata,
    pub(super) listeners: Vec<Arc<dyn AgentSessionListener>>,
    pub(super) recv_extra: Option<String>,
    pub(super) allow_cron: bool,
}

/// 统一串行化同一 session 的整次 run，避免多个入口同时 restore_context + run 时共享旧快照。
///
/// 使用 `Weak` 引用存储锁，当没有任何调用方持有该 session 的锁时，Map 中的条目
/// 会在下次访问时被自然替换，避免长期运行后 HashMap 无限增长。
static SESSION_RUN_LOCKS: OnceLock<
    Mutex<HashMap<String, std::sync::Weak<tokio::sync::Mutex<()>>>>,
> = OnceLock::new();

fn get_session_run_lock(session_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    let map = SESSION_RUN_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("session run lock poisoned");
    guard.retain(|_, weak| weak.upgrade().is_some());
    // 尝试从已有的 Weak 引用升级；若失败（已无持有者）则创建新锁并覆盖旧条目
    if let Some(existing) = guard.get(session_id).and_then(|w| w.upgrade()) {
        return existing;
    }
    let lock = Arc::new(tokio::sync::Mutex::new(()));
    guard.insert(session_id.to_string(), Arc::downgrade(&lock));
    lock
}

impl AgentSession {
    pub(super) async fn run_runner_with_empty_success_retry(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
        reexecution_policy: PreparedTurnReexecutionPolicy,
    ) -> AgentRunnerResult {
        let mut last_result = self
            .run_runner_with_progress_watchdog(
                runner,
                runner_name,
                session_id,
                0,
                request.clone(),
                emitter.clone(),
            )
            .await;
        let mut transient_retry_count = 0usize;
        let mut empty_success_retry_count = 0usize;

        loop {
            let retry_blocked = reexecution_policy == PreparedTurnReexecutionPolicy::ExecuteOnce
                || !response_has_only_known_read_only_calls(&last_result.response.tool_calls_made)
                || response_has_persistent_side_effect(&last_result.response.tool_calls_made)
                || last_result.committed_visible_prefix.is_some();
            if is_retryable_transient_runner_failure(&last_result)
                && !retry_blocked
                && transient_retry_count < TRANSIENT_RUNNER_FAILURE_RETRY_LIMIT
            {
                transient_retry_count += 1;
                tracing::warn!(
                    "[AgentSession] transient runner failure, retrying runner={} session_id={} attempt={}/{} error={}",
                    runner_name,
                    session_id,
                    transient_retry_count,
                    TRANSIENT_RUNNER_FAILURE_RETRY_LIMIT,
                    last_result.response.error.as_deref().unwrap_or_default()
                );
                self.core.log_message_step(
                    &self.actor.channel,
                    &self.actor.user_id,
                    session_id,
                    "agent.run.retry",
                    &format!(
                        "transient_runner_failure attempt={transient_retry_count}/{TRANSIENT_RUNNER_FAILURE_RETRY_LIMIT}"
                    ),
                    self.message_id.as_deref(),
                    None,
                );
                self.emit(session_progress_event(
                    "agent.run.retry",
                    Some(format!(
                        "{runner_name} transient_runner_failure attempt={transient_retry_count}/{TRANSIENT_RUNNER_FAILURE_RETRY_LIMIT}"
                    )),
                ))
                .await;

                last_result = self
                    .run_runner_with_progress_watchdog(
                        runner,
                        runner_name,
                        session_id,
                        transient_retry_count,
                        request.clone(),
                        emitter.clone(),
                    )
                    .await;
                continue;
            }

            // 如果运行失败，或者已经拿到了正文/工具调用，则不重试。
            // 对“支持流式但本次没有任何输出”的 runner，继续走空回复兜底逻辑。
            if should_return_runner_result(&last_result) {
                return last_result;
            }

            if retry_blocked {
                tracing::warn!(
                    runner = runner_name,
                    session_id,
                    "[AgentSession] automatic rerun suppressed for execute-once turn"
                );
                break;
            }

            if empty_success_retry_count >= EMPTY_SUCCESS_RETRY_LIMIT {
                break;
            }

            empty_success_retry_count += 1;
            let attempt = transient_retry_count + empty_success_retry_count;
            tracing::warn!(
                "[AgentSession] empty successful response, retrying runner={} session_id={} attempt={}/{}",
                runner_name,
                session_id,
                empty_success_retry_count,
                EMPTY_SUCCESS_RETRY_LIMIT
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "agent.run.retry",
                &format!(
                    "empty_success attempt={empty_success_retry_count}/{EMPTY_SUCCESS_RETRY_LIMIT}"
                ),
                self.message_id.as_deref(),
                None,
            );
            self.emit(session_progress_event(
                "agent.run.retry",
                Some(format!(
                    "{runner_name} empty_success attempt={empty_success_retry_count}/{EMPTY_SUCCESS_RETRY_LIMIT}"
                )),
            ))
            .await;

            last_result = self
                .run_runner_with_progress_watchdog(
                    runner,
                    runner_name,
                    session_id,
                    attempt,
                    request.clone(),
                    emitter.clone(),
                )
                .await;
        }

        if last_result.response.success && last_result.response.content.trim().is_empty() {
            tracing::warn!(
                "[AgentSession] empty successful response exhausted retries runner={} session_id={}",
                runner_name,
                session_id
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "agent.run.fallback",
                "empty_success_exhausted",
                self.message_id.as_deref(),
                None,
            );
            last_result.response.success = false;
            last_result.response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
            last_result.response.error = Some(EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string());
        }

        last_result
    }

    pub(super) async fn run_runner_with_investment_contract_retry(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
        contract: Option<&InvestmentResponseContract>,
        reexecution_policy: PreparedTurnReexecutionPolicy,
        agent_discovery_input: Option<&str>,
    ) -> AgentRunnerResult {
        let deferred_emitter: Arc<dyn AgentRunnerEmitter> =
            Arc::new(DeferredUserOutputEmitter::new(emitter.clone()));
        let defer_output = contract.is_some()
            || agent_discovery_input.is_some()
            || reexecution_policy == PreparedTurnReexecutionPolicy::ExecuteOnce;
        let attempt_emitter = if defer_output {
            deferred_emitter.clone()
        } else {
            emitter
        };
        let mut result = self
            .run_runner_with_empty_success_retry(
                runner,
                runner_name,
                session_id,
                request.clone(),
                attempt_emitter,
                reexecution_policy,
            )
            .await;
        if defer_output {
            mark_investment_attempt_output_deferred(&mut result);
        }
        normalize_execute_once_failure(&mut result, reexecution_policy);
        normalize_persistent_trace_failure(&mut result);
        if contract.is_some_and(|contract| contract.origin == AgentTurnOrigin::Interactive) {
            // Defense in depth: Interactive answers are Agent-owned even if an
            // upstream regression accidentally constructs a typed contract.
            // Never let that contract authorize repair, deterministic fallback,
            // normalization, or a fixed publication refusal.
            tracing::warn!(
                session_id,
                runner = runner_name,
                "ignored strict Interactive investment contract at publication boundary"
            );
            return result;
        }
        // Interactive entity discovery is owned by the configured Agent. The
        // service may reconstruct a provider-backed scope from the completed
        // read-only trace for diagnostics, but that observation must never
        // authorize, rewrite, retry, or reject the Agent's successful answer.
        // Typed Scheduled/Heartbeat contracts continue through the strict
        // validation path below.
        if contract.is_none()
            && let Some(input) = agent_discovery_input
        {
            let missing_seed_symbols = result.response.success.then(|| {
                missing_required_agent_seed_symbols(input, &result.response.tool_calls_made)
            });
            let discovered = if result.response.success {
                build_agent_discovered_investment(
                    input,
                    crate::agent_session::AgentTurnOrigin::Interactive,
                    &result.response.tool_calls_made,
                )
            } else {
                None
            };
            let detail = if let Some(discovered) = discovered.as_ref() {
                format!(
                    "contract_built=true entities={} answer_preserved=true mode=observational{}",
                    discovered
                        .contract
                        .entities
                        .iter()
                        .map(|entity| entity.symbol.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    missing_seed_symbols
                        .as_ref()
                        .filter(|symbols| !symbols.is_empty())
                        .map(|symbols| format!(" missing_explicit_seeds={}", symbols.join(",")))
                        .unwrap_or_default()
                )
            } else {
                format!(
                    "contract_built=false answer_preserved=true mode=observational{}",
                    missing_seed_symbols
                        .as_ref()
                        .filter(|symbols| !symbols.is_empty())
                        .map(|symbols| format!(" missing_explicit_seeds={}", symbols.join(",")))
                        .unwrap_or_default()
                )
            };
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "entity_resolution.agent_loop",
                &detail,
                self.message_id.as_deref(),
                None,
            );
            return result;
        }
        let Some(contract) = contract else {
            return result;
        };
        if !result.response.success {
            preserve_verified_investment_failure(contract, &mut result);
            return result;
        }
        let initial_forbidden_calls =
            forbidden_investment_tool_calls(contract, &result.response.tool_calls_made);
        let visible_content = sanitize_user_visible_output(&result.response.content).content;
        let normalized_visible_content =
            enforce_server_data_time_prefix(contract, &visible_content);
        let mut missing =
            missing_investment_response_sections(contract, &normalized_visible_content);
        for violation in initial_forbidden_calls.iter().copied() {
            if !missing.contains(&violation) {
                missing.push(violation);
            }
        }
        if missing.is_empty() {
            result.response.content = normalized_visible_content;
            return result;
        }

        let repair_trace_is_known_read_only =
            response_has_only_known_read_only_calls(&result.response.tool_calls_made);
        if reexecution_policy == PreparedTurnReexecutionPolicy::ExecuteOnce
            || !repair_trace_is_known_read_only
            || response_has_persistent_side_effect(&result.response.tool_calls_made)
        {
            let message = if !repair_trace_is_known_read_only
                && !response_has_persistent_side_effect(&result.response.tool_calls_made)
            {
                UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE
            } else if persistent_side_effect_state_is_uncertain(&result.response.tool_calls_made)
                || reexecution_policy == PreparedTurnReexecutionPolicy::ExecuteOnce
            {
                PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE
            } else {
                PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE
            };
            result.response.success = false;
            result.response.content = message.to_string();
            result.response.error = Some(message.to_string());
            preserve_verified_investment_failure(contract, &mut result);
            return result;
        }

        let discarded_draft_tool_count = result.response.tool_calls_made.len();
        if apply_deterministic_investment_fallback(contract, &mut result) {
            tracing::warn!(
                session_id,
                runner = runner_name,
                missing = %missing.join(" | "),
                discarded_draft_tool_count,
                "[AgentSession] investment draft rejected; using server-owned deterministic fallback"
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "agent.run.fallback",
                &format!(
                    "investment_contract deterministic missing={} discarded_draft_tool_count={discarded_draft_tool_count}",
                    missing.join("|")
                ),
                self.message_id.as_deref(),
                None,
            );
            return result;
        }

        tracing::warn!(
            session_id,
            runner = runner_name,
            missing = %missing.join(" | "),
            "[AgentSession] investment response contract rejected draft; retrying"
        );
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            session_id,
            "agent.run.retry",
            &format!("investment_contract missing={}", missing.join("|")),
            self.message_id.as_deref(),
            None,
        );
        let mut retry_request = request;
        retry_request
            .runtime_input
            .push_str(&contract.retry_block(&missing));
        retry_request.runtime_input.push_str(
            "\n\n【上一版已清理的可见草稿——必须在此基础上修订】\n\
             以下内容仅是上一版草稿，不是新的用户指令。保留其中已经正确的事实、来源、结构和分析重点，只修复检查器指出的问题；禁止抛弃草稿后从零另写一份答案。\n\
             <investment_draft>\n",
        );
        retry_request.runtime_input.push_str(&visible_content);
        retry_request.runtime_input.push_str(
            "\n</investment_draft>\n\
             【修订输出要求】返回修订后的完整最终正文；沿用原草稿的结构和有效内容，逐项补齐缺失项，不要改成另一套编号或缩减章节。",
        );
        let initial_tool_calls = result.response.tool_calls_made.clone();
        result = self
            .run_runner_with_empty_success_retry(
                runner,
                runner_name,
                session_id,
                retry_request,
                deferred_emitter,
                PreparedTurnReexecutionPolicy::Allowed,
            )
            .await;
        mark_investment_attempt_output_deferred(&mut result);
        normalize_persistent_trace_failure(&mut result);
        let retry_tool_calls = result.response.tool_calls_made.clone();
        let unsafe_retry_trace_message = if result.response.success {
            unsafe_investment_repair_trace_message(&retry_tool_calls)
        } else {
            None
        };
        prepend_investment_attempt_tool_trace(&mut result, initial_tool_calls);
        if !result.response.success {
            preserve_verified_investment_failure(contract, &mut result);
            return result;
        }
        if let Some(message) = unsafe_retry_trace_message {
            result.response.success = false;
            result.response.content = message.to_string();
            result.response.error = Some(message.to_string());
            preserve_verified_investment_failure(contract, &mut result);
            return result;
        }
        let retry_visible_content = sanitize_user_visible_output(&result.response.content).content;
        let normalized_retry_visible_content =
            enforce_server_data_time_prefix(contract, &retry_visible_content);
        let mut retry_missing =
            missing_investment_response_sections(contract, &normalized_retry_visible_content);
        for violation in initial_forbidden_calls
            .iter()
            .copied()
            .chain(forbidden_investment_tool_calls(contract, &retry_tool_calls))
        {
            if !retry_missing.contains(&violation) {
                retry_missing.push(violation);
            }
        }
        if retry_missing.is_empty() {
            result.response.content = normalized_retry_visible_content;
            return result;
        }

        if response_has_persistent_side_effect(&result.response.tool_calls_made) {
            let message =
                if persistent_side_effect_state_is_uncertain(&result.response.tool_calls_made) {
                    PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE
                } else {
                    PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE
                };
            result.response.success = false;
            result.response.content = message.to_string();
            result.response.error = Some(message.to_string());
            preserve_verified_investment_failure(contract, &mut result);
            return result;
        }

        tracing::error!(
            session_id,
            runner = runner_name,
            missing = %retry_missing.join(" | "),
            "[AgentSession] investment response contract retry still incomplete"
        );
        let failure = investment_contract_failure_message(contract, contract_failure_message());
        result.response.success = false;
        result.response.content = failure.clone();
        result.response.error = Some(failure);
        result
    }

    /// Run the underlying runner while emitting a periodic "still running" heartbeat.
    ///
    /// 背景：`runner.run(...)` 内部会一直驻留到 ACP 会话结束；一旦进入长工具链或上游静默，
    /// 外层除了 `agent.run start` 之外没有任何痕迹，直到整个 run 结束或超时才会再次落日志
    /// （参见 `docs/bugs/feishu_scheduler_run_stuck_without_cron_job_run.md`）。这里用一个
    /// `tokio::select!` ticker 在 run_fut 未完成时定期打 `agent.run.progress`，保证：
    /// - 结构化运行日志在卡死期间仍有心跳，运维能立刻判定「执行中 vs 卡死」；
    /// - session 可见进度事件 (`session_progress_event`) 同步到 UI/下游，避免客户端以为 run 已失联。
    async fn run_runner_with_progress_watchdog(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        attempt: usize,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let tick = progress_watchdog_tick();
        let runner_name = runner_name.to_string();
        let session_id_s = session_id.to_string();
        let run_fut = runner.run(request, emitter);
        run_with_progress_ticks(run_fut, tick, |ticks, elapsed| {
            let runner_name = runner_name.clone();
            let session_id = session_id_s.clone();
            let elapsed_s = elapsed.as_secs();
            let detail = if attempt == 0 {
                format!("elapsed_s={elapsed_s} tick={ticks}")
            } else {
                format!("elapsed_s={elapsed_s} tick={ticks} retry_attempt={attempt}")
            };
            tracing::warn!(
                state = "agent_iterating",
                "[AgentSession] agent.run still running runner={} session_id={} {}",
                runner_name,
                session_id,
                detail
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.progress",
                &format!("{runner_name} {detail}"),
                self.message_id.as_deref(),
                Some("agent_iterating"),
            );
            async move {
                self.emit(session_progress_event(
                    "agent.run.progress",
                    Some(format!("{runner_name} {detail}")),
                ))
                .await;
            }
        })
        .await
    }

    fn build_skill_runtime(&self) -> hone_tools::SkillRuntime {
        hone_tools::SkillRuntime::new(
            self.core.configured_system_skills_dir(),
            self.core.configured_custom_skills_dir(),
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        )
        .with_registry_path(self.core.configured_skill_registry_path())
    }

    fn restore_runtime_context(
        &self,
        session_id: &str,
        persisted_user_input: &str,
        restore_max_override: Option<usize>,
        turn_origin: AgentTurnOrigin,
        user_references_only: bool,
    ) -> RestoredRuntimeContext {
        let restore_limit = restore_max_override.or(self.restore_max_messages);
        let session_snapshot = self
            .core
            .session_storage
            .load_session(session_id)
            .ok()
            .flatten();
        let skill_runtime = self.build_skill_runtime();
        let mut context = if user_references_only {
            session_snapshot
                .as_ref()
                .map(|session| {
                    restore_recent_interactive_user_references(
                        session,
                        persisted_user_input,
                        Some(&skill_runtime),
                    )
                })
                .unwrap_or_else(|| AgentContext::new(session_id.to_string()))
        } else {
            session_snapshot
                .as_ref()
                .map(|session| {
                    restore_context_from_snapshot(session, restore_limit, Some(&skill_runtime))
                })
                .unwrap_or_else(|| AgentContext::new(session_id.to_string()))
        };
        context.set_actor_identity(&self.actor);

        if turn_origin == AgentTurnOrigin::Interactive && !user_references_only {
            let removed =
                prune_interactive_runtime_history(&mut context.messages, persisted_user_input);
            if removed > 0 {
                tracing::info!(
                    session_id,
                    channel = %self.actor.channel,
                    user_id = %self.actor.user_id,
                    removed_messages = removed,
                    "pruned automation and failed turns from interactive runner context"
                );
            }
        }
        if !user_references_only {
            if let Some(last) = context.messages.last() {
                if last.role == "user" && last.content.as_deref() == Some(persisted_user_input) {
                    context.messages.pop();
                }
            }
        }

        RestoredRuntimeContext {
            context,
            session_metadata: session_snapshot
                .map(|session| session.metadata)
                .unwrap_or_default(),
        }
    }

    fn uses_initial_strict_interactive_research_context(
        &self,
        runtime_user_input: &str,
        options: &AgentRunOptions,
        restore_max_override: Option<usize>,
        prepared_investment: Option<&PreparedInvestmentContext>,
    ) -> bool {
        if restore_max_override.is_some()
            || prepared_investment.is_some()
            || options.turn_origin != AgentTurnOrigin::Interactive
            || !self.core.actor_uses_strict_runner_fallback(&self.actor)
            || prepared_turn_reexecution_policy(runtime_user_input)
                != PreparedTurnReexecutionPolicy::Allowed
        {
            return false;
        }
        let entity_resolution_input = options
            .entity_resolution_input
            .as_deref()
            .unwrap_or(runtime_user_input);
        has_main_agent_entity_discovery_seed(entity_resolution_input, options.turn_origin)
    }

    pub(super) async fn prepare_execution_for_turn(
        &self,
        session_id: &str,
        persisted_user_input: &str,
        runtime_user_input: &str,
        options: &AgentRunOptions,
        restore_max_override: Option<usize>,
        prepared_investment: Option<&PreparedInvestmentContext>,
    ) -> Result<(PreparedExecution, PreparedInvestmentContext), (AgentSessionErrorKind, String)>
    {
        let use_fast_interactive_context = self.uses_initial_strict_interactive_research_context(
            runtime_user_input,
            options,
            restore_max_override,
            prepared_investment,
        );
        let restored = self.restore_runtime_context(
            session_id,
            persisted_user_input,
            restore_max_override,
            options.turn_origin,
            use_fast_interactive_context,
        );
        let mut context = restored.context;
        if options.turn_origin == AgentTurnOrigin::Interactive
            && restore_max_override == Some(CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT)
        {
            let removed = prune_historical_tool_protocol(&mut context.messages);
            if removed > 0 {
                tracing::info!(
                    session_id,
                    channel = %self.actor.channel,
                    user_id = %self.actor.user_id,
                    removed_messages = removed,
                    "pruned historical tool protocol from context-overflow recovery prompt"
                );
            }
        }
        let prompt_time_beijing = prompt_time_for_attempt(
            prepared_investment.map(|prepared| prepared.prompt_time_beijing),
            hone_core::beijing_now(),
        );
        let (system_prompt, mut runtime_input, answer_time_beijing) = self.resolve_prompt_input_at(
            session_id,
            runtime_user_input,
            prompt_time_beijing,
            !use_fast_interactive_context,
        );
        let investment_context = if let Some(prepared) = prepared_investment {
            runtime_input.push_str(&prepared.runtime_suffix);
            prepared.clone()
        } else {
            let suffix_start = runtime_input.len();
            let entity_resolution_input = options
                .entity_resolution_input
                .as_deref()
                .unwrap_or(runtime_user_input);
            let main_agent_entity_discovery =
                uses_main_agent_entity_discovery(entity_resolution_input, options.turn_origin);
            let emit_market_data_progress =
                should_emit_investment_preflight(entity_resolution_input, options.turn_origin);
            // Typed scheduled/heartbeat entity and market-data preparation can
            // execute before a runner exists, so those direct tool calls cannot
            // emit runner ToolStatus events. Interactive discovery stays in the
            // main Agent loop and therefore does not emit this preflight stage.
            if emit_market_data_progress {
                self.emit(session_progress_event("entity_resolution.preflight", None))
                    .await;
            }
            let contract_result = prepare_verified_investment_turn(
                &self.core,
                &self.actor,
                &self.channel_target,
                self.allow_cron,
                entity_resolution_input,
                options.turn_origin,
                &answer_time_beijing,
                &mut runtime_input,
            )
            .await;
            if emit_market_data_progress {
                let completed_stage = match &contract_result {
                    Ok(Some(_)) => "market_data.preflight.done",
                    Ok(None) => "entity_resolution.preflight.done",
                    Err(_) => "entity_resolution.preflight.failed",
                };
                self.emit(session_progress_event(completed_stage, None))
                    .await;
            }
            let contract = contract_result.map_err(|err| {
                (
                    AgentSessionErrorKind::AgentFailed,
                    investment_preflight_failure_message(&err),
                )
            })?;
            PreparedInvestmentContext {
                contract,
                runtime_suffix: runtime_input[suffix_start..].to_string(),
                prompt_time_beijing,
                reexecution_policy: prepared_turn_reexecution_policy(runtime_user_input),
                main_agent_entity_discovery_input: main_agent_entity_discovery
                    .then(|| entity_resolution_input.to_string()),
            }
        };
        if prepared_investment.is_none()
            && let Some(contract) = investment_context.contract.as_ref()
        {
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "market_data.preflight",
                &format!(
                    "entities={} deep_analysis={:?} deep_comparison={} requires_verified_price={} comparison={} outlook={} origin={:?}",
                    contract
                        .entities
                        .iter()
                        .map(|entity| entity.symbol.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    contract.deep_analysis,
                    contract.deep_comparison,
                    contract.requires_verified_price,
                    contract.comparison,
                    contract.needs_outlook_evidence,
                    contract.origin,
                ),
                self.message_id.as_deref(),
                None,
            );
        }
        let mut execution = ExecutionService::new(self.core.clone())
            .prepare(ExecutionRequest {
                mode: ExecutionMode::PersistentConversation,
                session_id: session_id.to_string(),
                actor: self.actor.clone(),
                channel_target: self.channel_target.clone(),
                allow_cron: self.allow_cron,
                system_prompt,
                runtime_input,
                context,
                timeout: options.timeout,
                gemini_stream: self.default_gemini_stream_options(options.timeout),
                session_metadata: restored.session_metadata,
                model_override: options.model_override.clone(),
                runner_selection: ExecutionRunnerSelection::Configured,
                allowed_tools: None,
                max_tool_calls: None,
                tool_call_limits: None,
                prompt_audit: Some(PromptAuditMetadata {
                    session_identity: self.session_identity.clone(),
                    message_id: self.message_id.clone(),
                }),
            })
            .map_err(|err| {
                tracing::error!(
                    session_id = %session_id,
                    channel = %self.actor.channel,
                    user_id = %self.actor.user_id,
                    channel_target = %self.channel_target,
                    "[AgentSession] execution prepare failed: {}",
                    err
                );
                let kind = if err.contains("sandbox") {
                    AgentSessionErrorKind::Io
                } else {
                    AgentSessionErrorKind::AgentFailed
                };
                (kind, err)
            })?;
        execution.runner_request.agent_owned_finance_loop = options.turn_origin
            == AgentTurnOrigin::Interactive
            && execution.runner_name == "function_calling"
            && investment_context.contract.is_none()
            && investment_context
                .main_agent_entity_discovery_input
                .is_some()
            && investment_context.reexecution_policy == PreparedTurnReexecutionPolicy::Allowed;
        let bounded_finance_turn = execution.runner_request.agent_owned_finance_loop
            && investment_context
                .main_agent_entity_discovery_input
                .as_deref()
                .is_some_and(|input| {
                    has_main_agent_entity_discovery_seed(input, options.turn_origin)
                });
        if bounded_finance_turn {
            // A shortlist question must not turn one user turn into an
            // unbounded market crawl. These are request-local safety limits;
            // reaching one moves the same Agent to a tools-disabled natural
            // final rather than feeding budget errors through many rounds.
            execution.runner_request.max_tool_calls = Some(24);
            execution.runner_request.tool_call_limits = Some(HashMap::from([
                ("data_fetch".to_string(), 20),
                ("web_search".to_string(), 6),
            ]));
        }
        if execution.runner_request.agent_owned_finance_loop && self.actor.channel == "web" {
            execution.runner_request.terminal_stream_policy =
                TerminalStreamPolicy::CanonicalInvestmentHeader;
            execution.runner_request.service_owned_initial_prefix = Some(
                ServiceOwnedInitialPrefix {
                    content: format!(
                        "数据时间：北京时间 {answer_time_beijing}；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露"
                    ),
                    commit_before_model: bounded_finance_turn,
                },
            );
        }
        Ok((execution, investment_context))
    }

    pub(super) fn persist_successful_assistant_turn(
        &self,
        session_id: &str,
        response: &AgentResponse,
        context_messages: Option<&[AgentMessage]>,
    ) {
        let mut metadata = self.message_metadata.assistant.clone();
        if let Some(source) = context_messages.and_then(|messages| {
            messages
                .iter()
                .rfind(|message| message.role == "assistant")
                .and_then(|message| message.metadata.clone())
        }) {
            metadata = merge_message_metadata(metadata, source);
        }

        let Some(message) = persistable_turn_from_response(response, metadata) else {
            return;
        };

        let _ = self.core.session_storage.append_session_messages(
            session_id,
            vec![session_message_from_normalized(
                &message,
                hone_core::beijing_now_rfc3339(),
            )],
        );
    }

    fn persist_assistant_text_turn(
        &self,
        session_id: &str,
        content: &str,
        metadata_extra: HashMap<String, Value>,
    ) {
        if content.trim().is_empty() {
            return;
        }
        let metadata =
            merge_message_metadata(self.message_metadata.assistant.clone(), metadata_extra);
        // Failure-facing text has already crossed a user-visible boundary or
        // been sanitized by the caller. Preserve its exact bytes, including a
        // committed terminal prefix's trailing newline, so refresh/history can
        // never disagree with the live stream.
        let message = NormalizedConversationMessage {
            role: "assistant".to_string(),
            content: vec![NormalizedConversationPart {
                part_type: "final".to_string(),
                text: Some(content.to_string()),
                id: None,
                name: None,
                args: None,
                result: None,
                metadata: None,
            }],
            status: Some("completed".to_string()),
            metadata,
        };
        let _ = self.core.session_storage.append_session_messages(
            session_id,
            vec![session_message_from_normalized(
                &message,
                hone_core::beijing_now_rfc3339(),
            )],
        );
    }

    fn persist_failed_assistant_turn_if_needed(
        &self,
        session_id: &str,
        kind: AgentSessionErrorKind,
        message: &str,
    ) {
        let should_persist = self
            .core
            .session_storage
            .get_messages(session_id, None)
            .ok()
            .and_then(|messages| messages.last().cloned())
            .is_some_and(|message| message.role == "user");
        if !should_persist {
            return;
        }

        let mut metadata = HashMap::new();
        metadata.insert("run_failed".to_string(), Value::Bool(true));
        metadata.insert("error_kind".to_string(), Value::String(format!("{kind:?}")));
        self.persist_assistant_text_turn(session_id, message, metadata);
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            session_id,
            "session.persist_assistant",
            "failed",
            self.message_id.as_deref(),
            None,
        );
    }

    async fn force_compact_for_context_overflow(&self, session_id: &str) -> Result<bool, String> {
        let outcome = SessionCompactor::new(&self.core)
            .compact_session(
                session_id,
                "context_overflow_recovery",
                true,
                Some("优先保留最近用户问题、最近结论、未完成事项，以及继续当前回答所必需的最小上下文。"),
                true,
            )
            .await
            .map_err(|err| err.to_string())?;
        Ok(outcome.compacted)
    }

    pub fn new(
        core: Arc<HoneBotCore>,
        actor: ActorIdentity,
        channel_target: impl Into<String>,
    ) -> Self {
        let session_identity = SessionIdentity::from_actor(&actor).unwrap_or_else(|_| {
            SessionIdentity::direct(&actor.channel, &actor.user_id)
                .expect("actor should always map to a direct session")
        });
        let restore_max_messages = restore_limit_before_compaction(&core.config, &session_identity);
        Self {
            core,
            actor,
            session_id: session_identity.session_id(),
            session_identity,
            channel_target: channel_target.into(),
            message_id: None,
            restore_max_messages,
            prompt_options: PromptOptions::default(),
            session_metadata: None,
            message_metadata: MessageMetadata::default(),
            listeners: Vec::new(),
            recv_extra: None,
            allow_cron: true,
        }
    }

    pub fn with_message_id(mut self, message_id: Option<String>) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn with_restore_max_messages(mut self, limit: Option<usize>) -> Self {
        self.restore_max_messages = limit;
        self
    }

    pub fn with_session_identity(mut self, session_identity: SessionIdentity) -> Self {
        self.session_id = session_identity.session_id();
        self.restore_max_messages =
            restore_limit_before_compaction(&self.core.config, &session_identity);
        self.session_identity = session_identity;
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    pub fn with_prompt_options(mut self, options: PromptOptions) -> Self {
        self.prompt_options = options;
        self
    }

    pub fn with_session_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.session_metadata = Some(metadata);
        self
    }

    pub fn with_message_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.message_metadata = metadata;
        self
    }

    pub fn with_recv_extra(mut self, extra: Option<String>) -> Self {
        self.recv_extra = extra;
        self
    }

    pub fn with_cron_allowed(mut self, allowed: bool) -> Self {
        self.allow_cron = allowed;
        self
    }

    pub fn add_listener(&mut self, listener: Arc<dyn AgentSessionListener>) {
        self.listeners.push(listener);
    }

    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }

    async fn emit(&self, event: AgentSessionEvent) {
        for listener in &self.listeners {
            listener.on_event(event.clone()).await;
        }
    }

    fn ensure_session_exists(&self) -> hone_core::HoneResult<()> {
        let session_id = self.session_id();
        if self
            .core
            .session_storage
            .load_session(&session_id)
            .ok()
            .flatten()
            .is_none()
        {
            self.core
                .session_storage
                .create_session_for_identity(&self.session_identity, Some(&self.actor))?;
        }
        Ok(())
    }

    fn update_session_metadata(&self) {
        let Some(metadata) = self.session_metadata.clone() else {
            return;
        };
        let _ = self
            .core
            .session_storage
            .update_metadata(&self.session_id, metadata);
    }

    #[cfg(test)]
    pub(super) fn resolve_prompt_input(
        &self,
        session_id: &str,
        user_input: &str,
    ) -> (String, String, String) {
        self.resolve_prompt_input_at(session_id, user_input, hone_core::beijing_now(), true)
    }

    fn resolve_prompt_input_at(
        &self,
        session_id: &str,
        user_input: &str,
        prompt_time_beijing: DateTime<FixedOffset>,
        include_conversation_context: bool,
    ) -> (String, String, String) {
        let turn = PromptTurnBuilder::new(
            &self.core,
            &self.actor,
            session_id,
            self.prompt_options.clone(),
            self.allow_cron,
            self.recv_extra.as_deref(),
        )
        .resolve_prompt_input_at(
            user_input,
            prompt_time_beijing,
            include_conversation_context,
        );
        (
            turn.system_prompt,
            turn.runtime_input,
            turn.answer_time_beijing,
        )
    }

    fn expand_slash_skill_input(
        &self,
        session_id: &str,
        user_input: &str,
    ) -> hone_core::HoneResult<Option<SlashSkillExpansion>> {
        PromptTurnBuilder::new(
            &self.core,
            &self.actor,
            session_id,
            self.prompt_options.clone(),
            self.allow_cron,
            self.recv_extra.as_deref(),
        )
        .expand_slash_skill_input(user_input)
    }

    fn parse_compact_command(&self, user_input: &str) -> Option<CompactCommand> {
        let trimmed = user_input.trim();
        let compact = trimmed.strip_prefix("/compact")?;
        if !compact.is_empty() && !compact.starts_with(char::is_whitespace) {
            return None;
        }
        let instructions = compact.trim();
        Some(CompactCommand {
            instructions: (!instructions.is_empty()).then(|| instructions.to_string()),
        })
    }

    fn persist_invoked_skill_prompt(
        &self,
        session_id: &str,
        skill_id: &str,
        prompt: &str,
    ) -> hone_core::HoneResult<()> {
        let existing = self
            .core
            .session_storage
            .load_session(session_id)?
            .map(|session| session.metadata)
            .unwrap_or_default();
        let mut invoked = hone_memory::invoked_skills_from_metadata(&existing)
            .into_iter()
            .filter(|skill| skill.skill_name != skill_id)
            .collect::<Vec<_>>();
        invoked.push(hone_memory::InvokedSkillRecord {
            skill_name: skill_id.to_string(),
            display_name: skill_id.to_string(),
            path: format!("slash:{skill_id}"),
            prompt: prompt.to_string(),
            execution_context: "inline".to_string(),
            allowed_tools: Vec::new(),
            model: None,
            effort: None,
            agent: None,
            loaded_from: "slash".to_string(),
            updated_at: hone_core::beijing_now_rfc3339(),
        });
        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::to_value(invoked)
                .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))?,
        );
        let _ = self
            .core
            .session_storage
            .update_metadata(session_id, metadata)?;
        Ok(())
    }

    async fn run_manual_compact(
        &self,
        session_id: String,
        raw_input: &str,
        command: CompactCommand,
    ) -> AgentSessionResult {
        self.core.log_message_received(
            &self.actor.channel,
            &self.actor.user_id,
            &self.channel_target,
            &session_id,
            raw_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        self.emit(session_progress_event(
            "session.compress",
            Some("start".to_string()),
        ))
        .await;
        let started = Instant::now();
        let outcome = self
            .core
            .compact_session(&session_id, "manual", true, command.instructions.as_deref())
            .await;

        let response = match outcome {
            Ok(outcome) => {
                self.emit(session_progress_event(
                    "session.compress",
                    Some("done".to_string()),
                ))
                .await;
                let content = if outcome.compacted {
                    "Conversation compacted.".to_string()
                } else {
                    "未执行 compact：当前没有可压缩的会话内容，或压缩器暂不可用。".to_string()
                };
                AgentResponse {
                    content,
                    tool_calls_made: Vec::new(),
                    iterations: 0,
                    success: true,
                    error: None,
                }
            }
            Err(err) => {
                tracing::error!(
                    session_id = %session_id,
                    channel = %self.actor.channel,
                    user_id = %self.actor.user_id,
                    channel_target = %self.channel_target,
                    "[AgentSession] manual compact failed: {}",
                    err
                );
                self.emit(session_progress_event(
                    "session.compress",
                    Some("failed".to_string()),
                ))
                .await;
                AgentResponse {
                    content: String::new(),
                    tool_calls_made: Vec::new(),
                    iterations: 0,
                    success: false,
                    error: Some(err.to_string()),
                }
            }
        };
        let elapsed_ms = started.elapsed().as_millis();

        if response.success {
            self.core.log_message_finished(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &response,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        } else {
            let err = response
                .error
                .clone()
                .unwrap_or_else(|| "manual compact failed".to_string());
            self.core.log_message_failed(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &err,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(session_error_event(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: err,
            }))
            .await;
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        }

        AgentSessionResult {
            response,
            elapsed_ms,
            session_id,
        }
    }

    async fn run_domain_boundary_short_circuit(
        &self,
        session_id: String,
        raw_input: &str,
        reply: &str,
    ) -> AgentSessionResult {
        let started = Instant::now();
        let _ = self.core.session_storage.add_message(
            &session_id,
            "user",
            raw_input,
            self.message_metadata.user.clone(),
        );
        self.emit(AgentSessionEvent::UserMessage {
            content: raw_input.to_string(),
        })
        .await;
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "session.persist_user",
            "domain_boundary",
            self.message_id.as_deref(),
            None,
        );
        self.core.log_message_received(
            &self.actor.channel,
            &self.actor.user_id,
            &self.channel_target,
            &session_id,
            raw_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        let response = AgentResponse {
            content: reply.to_string(),
            tool_calls_made: Vec::new(),
            iterations: 0,
            success: true,
            error: None,
        };
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "agent.domain_boundary",
            "short_circuit_non_finance",
            self.message_id.as_deref(),
            None,
        );
        self.persist_successful_assistant_turn(&session_id, &response, None);
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "session.persist_assistant",
            "domain_boundary",
            self.message_id.as_deref(),
            None,
        );
        let elapsed_ms = started.elapsed().as_millis();
        self.core.log_message_finished(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            &response,
            elapsed_ms,
            self.message_id.as_deref(),
        );
        self.emit(AgentSessionEvent::Done {
            response: response.clone(),
        })
        .await;

        AgentSessionResult {
            response,
            elapsed_ms,
            session_id,
        }
    }

    fn default_gemini_stream_options(&self, timeout: Option<Duration>) -> GeminiStreamOptions {
        GeminiStreamOptions {
            max_iterations: 18,
            overall_timeout: timeout.unwrap_or_else(|| self.core.config.agent.overall_timeout()),
            per_line_timeout: self.core.config.agent.step_timeout(),
        }
    }

    fn runner_emitter(&self, working_directory: String) -> Arc<dyn AgentRunnerEmitter> {
        Arc::new(SessionEventEmitter {
            listeners: self.listeners.clone(),
            channel: self.actor.channel.clone(),
            user_id: self.actor.user_id.clone(),
            session_id: self.session_id.clone(),
            message_id: self.message_id.clone(),
            working_directory,
        })
    }

    async fn fail_run(
        &self,
        session_id: String,
        kind: AgentSessionErrorKind,
        message: String,
    ) -> AgentSessionResult {
        let persisted_message = user_visible_error_message(Some(message.as_str()));
        self.persist_failed_assistant_turn_if_needed(&session_id, kind, &persisted_message);
        let error = AgentSessionError {
            kind,
            // Session error events cross channel/Web boundaries. Preserve the
            // raw diagnostic in the returned response and logs, but never put
            // provider/Agent protocol details on a user-visible event.
            message: persisted_message,
        };
        self.emit(session_error_event(error.clone())).await;
        let response = AgentResponse {
            content: String::new(),
            tool_calls_made: Vec::new(),
            iterations: 0,
            success: false,
            error: Some(message),
        };
        self.emit(AgentSessionEvent::Done {
            response: response.clone(),
        })
        .await;
        AgentSessionResult {
            response,
            elapsed_ms: 0,
            session_id,
        }
    }

    fn reserve_conversation_quota(
        &self,
        quota_mode: AgentRunQuotaMode,
    ) -> hone_core::HoneResult<Option<ConversationQuotaReservation>> {
        if quota_mode == AgentRunQuotaMode::ScheduledTask {
            return Ok(None);
        }

        let daily_limit = self.core.config.agent.daily_conversation_limit;
        if daily_limit == 0 {
            return Ok(None);
        }

        let is_admin = self.core.is_admin_actor(&self.actor);
        match self
            .core
            .conversation_quota_storage
            .try_reserve_daily_conversation(&self.actor, daily_limit, is_admin)?
        {
            ConversationQuotaReserveResult::Reserved(reservation) => Ok(Some(reservation)),
            ConversationQuotaReserveResult::Bypassed => Ok(None),
            ConversationQuotaReserveResult::Rejected(snapshot) => {
                Err(hone_core::HoneError::Other(format!(
                    "已达到今日对话上限（{}/{}，北京时间 {}），请明天再试",
                    snapshot.success_count + snapshot.in_flight,
                    snapshot.limit,
                    snapshot.quota_date
                )))
            }
        }
    }

    /// 一次完整的 agent run,职责按顺序:
    ///
    /// 1. 拿下 per-session 串行化锁,防止两次 run 互相读到对方半成品;
    /// 2. 保证 session 存在 + 覆写 session metadata;
    /// 3. 若用户输入是 `/compact` 走 `run_manual_compact` 直接返回;
    /// 4. 预留 daily 配额(由 `QuotaReservationGuard` 负责失败时 release);
    /// 5. 展开可能的 slash skill 并把用户消息落盘(Fast Persist);
    /// 6. 做一次自动 compact 检查(non-fatal);
    /// 7. 组装 execution 并把 runner 跑起来;
    /// 8. 若 runner 报 context overflow,强制 compact 后按更小的 restore limit 再跑一轮;
    /// 9. 成功时:commit 配额、若非流式则按 segmenter 切片发给 listener、
    ///    把 assistant turn 落盘、打 finished 日志;失败时:drop guard 让 release 生效,
    ///    按错误类型翻译 ErrorKind,再 emit Done。
    pub async fn run(&self, user_input: &str, options: AgentRunOptions) -> AgentSessionResult {
        let session_id = self.session_id();
        let _run_guard = {
            let lock = get_session_run_lock(&session_id);
            lock.lock_owned().await
        };
        if let Err(err) = self.ensure_session_exists() {
            return self
                .fail_run(
                    session_id,
                    AgentSessionErrorKind::AgentFailed,
                    err.to_string(),
                )
                .await;
        }

        self.update_session_metadata();

        if let Some(command) = self.parse_compact_command(user_input) {
            return self
                .run_manual_compact(session_id, user_input, command)
                .await;
        }

        if options.quota_mode != AgentRunQuotaMode::ScheduledTask
            && !self.core.is_admin_actor(&self.actor)
        {
            if let Some(reply) = non_finance_boundary_reply(user_input) {
                return self
                    .run_domain_boundary_short_circuit(session_id, user_input, reply)
                    .await;
            }
        }

        // 配额预留；后续任何失败分支都靠 guard 在 drop 时自动把预留释放掉,
        // 不再需要每处都手写 release_daily_conversation。
        let quota_guard = match self.reserve_conversation_quota(options.quota_mode) {
            Ok(reservation) => QuotaReservationGuard::new(self.core.clone(), reservation),
            Err(err) => {
                let raw_error = err.to_string();
                let quota_message = user_visible_error_message(Some(raw_error.as_str()));
                let _ = self.core.session_storage.add_message(
                    &session_id,
                    "user",
                    user_input,
                    self.message_metadata.user.clone(),
                );
                self.emit(AgentSessionEvent::UserMessage {
                    content: user_input.to_string(),
                })
                .await;
                self.core.log_message_step(
                    &self.actor.channel,
                    &self.actor.user_id,
                    &session_id,
                    "session.persist_user",
                    "quota_rejected",
                    self.message_id.as_deref(),
                    None,
                );
                self.core.log_message_received(
                    &self.actor.channel,
                    &self.actor.user_id,
                    &self.channel_target,
                    &session_id,
                    user_input,
                    self.recv_extra.as_deref(),
                    self.message_id.as_deref(),
                );
                let mut metadata = HashMap::new();
                metadata.insert("quota_rejected".to_string(), Value::Bool(true));
                self.persist_assistant_text_turn(&session_id, &quota_message, metadata);
                self.core.log_message_step(
                    &self.actor.channel,
                    &self.actor.user_id,
                    &session_id,
                    "session.persist_assistant",
                    "quota_rejected",
                    self.message_id.as_deref(),
                    None,
                );
                return self
                    .fail_run(
                        session_id,
                        AgentSessionErrorKind::AgentFailed,
                        quota_message,
                    )
                    .await;
            }
        };

        let slash_skill = match self.expand_slash_skill_input(&session_id, user_input) {
            Ok(value) => value,
            Err(err) => {
                // quota_guard 在 return 的 drop 中自动 release
                drop(quota_guard);
                return self
                    .fail_run(
                        session_id,
                        AgentSessionErrorKind::AgentFailed,
                        err.to_string(),
                    )
                    .await;
            }
        };
        let persisted_user_input = slash_skill
            .as_ref()
            .map(|skill| skill.raw_input.as_str())
            .unwrap_or(user_input);
        let runtime_user_input = slash_skill
            .as_ref()
            .map(|skill| skill.runtime_input.as_str())
            .unwrap_or(user_input);
        let user_metadata = if let Some(skill) = &slash_skill {
            let mut extra = HashMap::new();
            extra.insert(
                hone_memory::SLASH_SKILL_METADATA_KEY.to_string(),
                Value::String(skill.skill_id.clone()),
            );
            merge_message_metadata(self.message_metadata.user.clone(), extra)
        } else {
            self.message_metadata.user.clone()
        };

        // ── Fast Persist: 立即写入用户消息 ──
        // 确保 ensureHistory 轮询时 DB 里已有此消息，避免前端因为竞态丢失消息显示
        let _ = self.core.session_storage.add_message(
            &session_id,
            "user",
            persisted_user_input,
            user_metadata,
        );
        if let Some(skill) = &slash_skill {
            let _ = self.persist_invoked_skill_prompt(
                &session_id,
                &skill.skill_id,
                &skill.invoked_prompt,
            );
        }
        self.emit(AgentSessionEvent::UserMessage {
            content: persisted_user_input.to_string(),
        })
        .await;
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "session.persist_user",
            "done",
            self.message_id.as_deref(),
            None,
        );

        self.core.log_message_received(
            &self.actor.channel,
            &self.actor.user_id,
            &self.channel_target,
            &session_id,
            persisted_user_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        let skip_initial_compaction = self.uses_initial_strict_interactive_research_context(
            runtime_user_input,
            &options,
            None,
            None,
        );
        if skip_initial_compaction {
            tracing::debug!(
                session_id = %session_id,
                channel = %self.actor.channel,
                user_id = %self.actor.user_id,
                "skipping synchronous pre-run compaction for initial strict Interactive research"
            );
        } else {
            self.emit(session_progress_event(
                "session.compress",
                Some("start".to_string()),
            ))
            .await;

            if let Err(err) = self
                .core
                .maybe_compress_session(&session_id, &self.actor)
                .await
            {
                tracing::error!(
                    session_id = %session_id,
                    channel = %self.actor.channel,
                    user_id = %self.actor.user_id,
                    channel_target = %self.channel_target,
                    "[AgentSession] compress failed: {}",
                    err
                );
                self.emit(session_progress_event(
                    "session.compress",
                    Some("failed".to_string()),
                ))
                .await;
            } else {
                self.emit(session_progress_event(
                    "session.compress",
                    Some("done".to_string()),
                ))
                .await;
            }
        }

        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "agent.prepare",
            "restore_context + build_prompt + create_runner",
            self.message_id.as_deref(),
            None,
        );

        let (mut execution, investment_context) = match self
            .prepare_execution_for_turn(
                &session_id,
                persisted_user_input,
                runtime_user_input,
                &options,
                None,
                None,
            )
            .await
        {
            Ok(prepared) => prepared,
            Err((kind, err)) => {
                drop(quota_guard);
                return self.fail_run(session_id, kind, err).await;
            }
        };
        let agent_owned_interactive_output = options.turn_origin == AgentTurnOrigin::Interactive;

        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "agent.run",
            "start",
            self.message_id.as_deref(),
            None,
        );
        self.emit(session_progress_event(
            "agent.run",
            Some(execution.runner_name.to_string()),
        ))
        .await;
        let started = Instant::now();
        let run_started_at = SystemTime::now();
        let defer_validated_output = agent_owned_interactive_output
            || investment_context.contract.is_some()
            || investment_context
                .main_agent_entity_discovery_input
                .is_some()
            || investment_context.reexecution_policy == PreparedTurnReexecutionPolicy::ExecuteOnce;
        let mut streamed_output = false;
        let mut terminal_error_emitted = false;
        let mut committed_visible_prefix: Option<String> = None;
        let mut context_messages: Option<Vec<AgentMessage>> = None;
        let mut response = AgentResponse {
            content: String::new(),
            tool_calls_made: Vec::new(),
            iterations: 0,
            success: false,
            error: None,
        };
        for recovery_idx in 0..=CONTEXT_OVERFLOW_RECOVERY_LIMIT {
            let runner_emitter =
                self.runner_emitter(execution.runner_request.working_directory.clone());
            let runner_result = self
                .run_runner_with_investment_contract_retry(
                    execution.runner.as_ref(),
                    execution.runner_name,
                    &session_id,
                    execution.runner_request.clone(),
                    runner_emitter.clone(),
                    investment_context.contract.as_ref(),
                    investment_context.reexecution_policy,
                    investment_context
                        .main_agent_entity_discovery_input
                        .as_deref(),
                )
                .await;
            streamed_output = runner_result.streamed_output;
            terminal_error_emitted = runner_result.terminal_error_emitted;
            committed_visible_prefix = runner_result.committed_visible_prefix;
            if !runner_result.session_metadata_updates.is_empty() {
                let _ = self
                    .core
                    .session_storage
                    .update_metadata(&session_id, runner_result.session_metadata_updates.clone());
            }
            context_messages = runner_result.context_messages;
            response = runner_result.response;

            let should_try_recovery = !response.success
                && response
                    .error
                    .as_deref()
                    .is_some_and(is_context_overflow_error_text)
                && investment_context.reexecution_policy == PreparedTurnReexecutionPolicy::Allowed
                && response_has_only_known_read_only_calls(&response.tool_calls_made)
                && !response_has_persistent_side_effect(&response.tool_calls_made)
                && committed_visible_prefix.is_none()
                && recovery_idx < CONTEXT_OVERFLOW_RECOVERY_LIMIT;
            if !should_try_recovery {
                break;
            }

            tracing::warn!(
                session_id = %session_id,
                channel = %self.actor.channel,
                user_id = %self.actor.user_id,
                channel_target = %self.channel_target,
                runner = %execution.runner_name,
                attempt = recovery_idx + 1,
                max_attempts = CONTEXT_OVERFLOW_RECOVERY_LIMIT,
                "[AgentSession] context overflow detected, compacting and retrying"
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.retry",
                &format!(
                    "context_overflow attempt={}/{}",
                    recovery_idx + 1,
                    CONTEXT_OVERFLOW_RECOVERY_LIMIT
                ),
                self.message_id.as_deref(),
                None,
            );
            self.emit(session_progress_event(
                "agent.run.retry",
                Some(format!(
                    "{} context_overflow attempt={}/{}",
                    execution.runner_name,
                    recovery_idx + 1,
                    CONTEXT_OVERFLOW_RECOVERY_LIMIT
                )),
            ))
            .await;

            match self.force_compact_for_context_overflow(&session_id).await {
                Ok(compacted) => {
                    tracing::info!(
                        session_id = %session_id,
                        channel = %self.actor.channel,
                        user_id = %self.actor.user_id,
                        channel_target = %self.channel_target,
                        compacted,
                        "[AgentSession] context overflow recovery compacted"
                    );
                }
                Err(err) => {
                    tracing::error!(
                        session_id = %session_id,
                        channel = %self.actor.channel,
                        user_id = %self.actor.user_id,
                        channel_target = %self.channel_target,
                        "[AgentSession] context overflow recovery compact failed: {}",
                        err
                    );
                    response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
                    break;
                }
            }

            let recovered = match self
                .prepare_execution_for_turn(
                    &session_id,
                    persisted_user_input,
                    runtime_user_input,
                    &options,
                    Some(CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT),
                    Some(&investment_context),
                )
                .await
            {
                Ok(prepared) => prepared,
                Err((_kind, err)) => {
                    tracing::error!(
                        session_id = %session_id,
                        channel = %self.actor.channel,
                        user_id = %self.actor.user_id,
                        channel_target = %self.channel_target,
                        "[AgentSession] context overflow recovery prepare failed: {}",
                        err
                    );
                    response.success = false;
                    response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
                    break;
                }
            };
            execution = recovered.0;
        }

        if !response.success
            && response
                .error
                .as_deref()
                .is_some_and(is_context_overflow_error_text)
        {
            response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
        }
        let finalize_outcome = if agent_owned_interactive_output {
            finalize_agent_owned_interactive_response(
                &self.core,
                &session_id,
                &execution.runner_name,
                &mut response,
            )
        } else {
            finalize_agent_response(
                &self.core,
                &session_id,
                &execution.runner_name,
                &mut response,
            )
        };
        if let Some(reason) = finalize_outcome.fallback_reason {
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.fallback",
                reason,
                self.message_id.as_deref(),
                None,
            );
        }
        if self.actor.channel == "web"
            && response.success
            && !self
                .recv_extra
                .as_deref()
                .is_some_and(|extra| extra.contains("openai_compatible_api=true"))
        {
            let attached = attach_web_generated_files(
                &mut response,
                &execution.runner_request.working_directory,
                run_started_at,
            );
            if attached > 0 {
                self.core.log_message_step(
                    &self.actor.channel,
                    &self.actor.user_id,
                    &session_id,
                    "agent.run.attachments",
                    &format!("generated_files={attached}"),
                    self.message_id.as_deref(),
                    None,
                );
            }
        }
        if response.success
            && let Some(prefix) = committed_visible_prefix.as_deref()
            && !align_response_to_committed_prefix(&mut response, prefix)
        {
            tracing::error!(
                session_id,
                runner = execution.runner_name,
                "committed terminal prefix no longer matches the finalized response"
            );
            response.success = false;
            response.content.clear();
            response.error = Some("committed terminal prefix mismatch".to_string());
            terminal_error_emitted = false;
        }
        let elapsed_ms = started.elapsed().as_millis();

        if response.success {
            // 成功路径：主动 commit 把预留转成当日计数,并消耗 guard 阻止
            // 后续 drop 再执行 release。
            quota_guard.commit();
            if defer_validated_output {
                if let Some(prefix) = committed_visible_prefix.as_deref() {
                    // The Agent already committed this exact canonical prefix
                    // through the unique Web sink. Only publish the finalized
                    // tail so UI concatenation and persistence remain
                    // byte-for-byte identical without a reset/replay.
                    let tail = response
                        .content
                        .strip_prefix(prefix)
                        .expect("committed prefix was verified above");
                    if !tail.is_empty() {
                        self.emit(AgentSessionEvent::Segment {
                            text: tail.to_string(),
                        })
                        .await;
                    }
                } else {
                    // Attempt-local draft/reset/error events were intentionally
                    // hidden. Publish the completed Agent answer exactly once.
                    if agent_owned_interactive_output {
                        // Security cleanup has already run exactly once. Do not
                        // send the completed Agent body back through the runner
                        // event sanitizer, whose legacy market-copy rewriting
                        // could diverge from the body persisted below.
                        self.emit(AgentSessionEvent::Segment {
                            text: response.content.clone(),
                        })
                        .await;
                    } else {
                        self.runner_emitter(execution.runner_request.working_directory.clone())
                            .emit(AgentRunnerEvent::StreamDelta {
                                content: response.content.clone(),
                            })
                            .await;
                    }
                }
                streamed_output = true;
            }
            if !streamed_output {
                if let Some(segmenter) = options.segmenter.as_ref() {
                    let segments = segmenter(&response.content);
                    for seg in segments {
                        self.emit(AgentSessionEvent::Segment { text: seg }).await;
                    }
                }
            }
            self.persist_successful_assistant_turn(
                &session_id,
                &response,
                context_messages.as_deref(),
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "session.persist_assistant",
                "done",
                self.message_id.as_deref(),
                None,
            );
            self.core.log_message_finished(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &response,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        } else {
            // 失败路径：显式 drop 触发 release,让配额回到预留前的状态。
            drop(quota_guard);
            let err = response
                .error
                .clone()
                .unwrap_or_else(|| "未知错误".to_string());
            let kind = if err.contains("agent_timeout") {
                AgentSessionErrorKind::AgentTimeout
            } else if err == CONTEXT_OVERFLOW_FALLBACK_MESSAGE {
                AgentSessionErrorKind::ContextWindowOverflow
            } else {
                AgentSessionErrorKind::AgentFailed
            };
            self.core.log_message_failed(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &err,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            if let Some(prefix) = committed_visible_prefix.as_deref() {
                // The runner still returns/logs the real failure, but the Web
                // stream has crossed an irreversible user-visible boundary.
                // Close that public stream normally and persist exactly the
                // bytes already shown; a failed Done would make the UI flash an
                // error card, while persisting a synthetic error would make a
                // refresh disagree with the visible transcript.
                tracing::warn!(
                    session_id,
                    "closing user-visible stream normally after committed terminal prefix failure"
                );
                let service_owned_prefix = execution
                    .runner_request
                    .service_owned_initial_prefix
                    .as_ref()
                    .map(|configured| configured.content.as_str())
                    == Some(prefix);
                let visible_partial = if service_owned_prefix {
                    self.emit(AgentSessionEvent::Segment {
                        text: SERVICE_OWNED_PREFIX_FAILURE_SUFFIX.to_string(),
                    })
                    .await;
                    format!("{prefix}{SERVICE_OWNED_PREFIX_FAILURE_SUFFIX}")
                } else {
                    prefix.to_string()
                };
                let mut metadata = HashMap::new();
                metadata.insert("run_failed".to_string(), Value::Bool(true));
                metadata.insert("error_kind".to_string(), Value::String(format!("{kind:?}")));
                metadata.insert("terminal_stream_incomplete".to_string(), Value::Bool(true));
                metadata.insert(
                    "service_owned_initial_prefix".to_string(),
                    Value::Bool(service_owned_prefix),
                );
                self.persist_assistant_text_turn(&session_id, &visible_partial, metadata);
                self.core.log_message_step(
                    &self.actor.channel,
                    &self.actor.user_id,
                    &session_id,
                    "session.persist_assistant",
                    "committed_prefix_after_terminal_failure",
                    self.message_id.as_deref(),
                    None,
                );
                let partial_response = AgentResponse {
                    content: visible_partial,
                    tool_calls_made: response.tool_calls_made.clone(),
                    iterations: response.iterations,
                    success: false,
                    error: None,
                };
                self.emit(AgentSessionEvent::PartialDone {
                    response: partial_response,
                })
                .await;
            } else if !terminal_error_emitted {
                let public_error = user_visible_error_message(Some(err.as_str()));
                self.emit(session_error_event(AgentSessionError {
                    kind,
                    message: public_error,
                }))
                .await;
                let persisted_message = user_visible_error_message(response.error.as_deref());
                self.persist_failed_assistant_turn_if_needed(&session_id, kind, &persisted_message);
                self.emit(AgentSessionEvent::Done {
                    response: response.clone(),
                })
                .await;
            } else {
                let persisted_message = user_visible_error_message(response.error.as_deref());
                self.persist_failed_assistant_turn_if_needed(&session_id, kind, &persisted_message);
                self.emit(AgentSessionEvent::Done {
                    response: response.clone(),
                })
                .await;
            }
        }

        AgentSessionResult {
            response,
            elapsed_ms,
            session_id,
        }
    }
}
