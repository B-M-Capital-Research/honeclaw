use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use async_trait::async_trait;
use hone_scheduler::SchedulerEvent;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_session::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionResult, GeminiStreamOptions,
};
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService,
};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::response_finalizer::EMPTY_SUCCESS_FALLBACK_MESSAGE;
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent};
use crate::runtime::{
    is_context_overflow_error, sanitize_user_visible_output, strip_internal_reasoning_blocks,
    user_visible_error_message_or_none,
};
use crate::{AgentSession, HoneBotCore};

const HEARTBEAT_NOOP_SENTINEL: &str = "[[HEARTBEAT_NOOP]]";
const HEARTBEAT_INTERNAL_PREFIX: &str = "[[HEART";
const HEARTBEAT_MAX_ITERATIONS: u32 = 10;
const SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE: &str =
    "本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。";

static RE_HEARTBEAT_CURRENT_BEFORE_TRIGGER_PRICE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
            r"(?is)(?:当前(?:价格|价)?|最新(?:价格|价)?|现价|收盘价|跌至|跌到|降至|回落至|current(?:\s*price)?)[^\d]{0,20}\$?\s*(?P<current>\d+(?:\.\d+)?)[\s\S]{0,120}(?:触发价|触发线|配置线|trigger\s*price|trigger\s*line)[^\d]{0,20}\$?\s*(?P<threshold>\d+(?:\.\d+)?)",
        )
        .expect("valid heartbeat trigger price regex")
});

static RE_HEARTBEAT_TRIGGER_PRICE_BEFORE_CURRENT: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
            r"(?is)(?:触发价|触发线|配置线|trigger\s*price|trigger\s*line)[^\d]{0,20}\$?\s*(?P<threshold>\d+(?:\.\d+)?)[\s\S]{0,120}(?:当前(?:价格|价)?|最新(?:价格|价)?|current(?:\s*price)?)[^\d]{0,20}\$?\s*(?P<current>\d+(?:\.\d+)?)",
        )
        .expect("valid heartbeat trigger price regex")
});

static RE_HEARTBEAT_FACT_TOKEN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?ix)
        (?:\d{1,4}年)?\d{1,2}月\d{1,2}日
        |
        \d+(?:\.\d+)?\s*(?:亿|万)?\s*(?:美元|美金|港元|人民币|元)
        |
        [A-Za-z][A-Za-z0-9.-]{1,}
        |
        \d+(?:\.\d+)?%?
        ",
    )
    .expect("valid heartbeat fact token regex")
});

#[derive(Debug, PartialEq, Eq)]
pub enum HeartbeatOutcome {
    Noop,
    Deliver(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum HeartbeatParseKind {
    Empty,
    SentinelNoop,
    InternalMarker,
    JsonNoop,
    JsonEmptyStatus,
    JsonTriggered,
    JsonUnknownStatus,
    JsonMalformed,
    PlainTextSuppressed,
}

#[derive(Debug, Deserialize)]
struct HeartbeatJsonResponse {
    status: Option<String>,
    message: Option<String>,
}

fn parse_heartbeat_json_payload(content: &str) -> Option<HeartbeatJsonResponse> {
    let trimmed = content.trim();
    if let Ok(parsed) = serde_json::from_str::<HeartbeatJsonResponse>(trimmed) {
        return Some(parsed);
    }

    let mut candidates = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in trimmed.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start_idx) = start.take() {
                        candidates.push(&trimmed[start_idx..=idx]);
                    }
                }
            }
            _ => {}
        }
    }

    candidates
        .into_iter()
        .rev()
        .find_map(|candidate| serde_json::from_str::<HeartbeatJsonResponse>(candidate).ok())
}

fn heartbeat_internal_marker_prefix(text: &str) -> bool {
    let trimmed = text.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    upper.starts_with(HEARTBEAT_INTERNAL_PREFIX)
}

fn heartbeat_internal_marker_present(text: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    upper.contains(HEARTBEAT_NOOP_SENTINEL) || upper.contains(HEARTBEAT_INTERNAL_PREFIX)
}

fn unwrap_nested_json_message(text: &str) -> String {
    if !text.starts_with('{') {
        return text.to_string();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        for key in &["trigger", "message", "content", "text", "alert"] {
            if let Some(s) = v.get(key).and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    return s.to_string();
                }
            }
        }
    }
    text.to_string()
}

fn heartbeat_near_threshold_without_crossing(text: &str) -> bool {
    let compact = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if compact.is_empty() {
        return false;
    }

    let threshold_terms = [
        "阈值",
        "警戒线",
        "警戒阈值",
        "门槛",
        "触发价",
        "触发线",
        "配置线",
        "条件线",
        "threshold",
        "triggerprice",
        "triggerline",
    ];
    let proximity_terms = [
        "接近",
        "临近",
        "靠近",
        "距离",
        "仅差",
        "差约",
        "未达到",
        "未达",
        "没有达到",
        "尚未达到",
        "未触及",
        "尚未触及",
        "没有触及",
        "未触发",
        "没有触发",
        "尚未触发",
        "未命中",
        "未满足",
        "未越过",
        "未超过",
        "没有超过",
        "尚未超过",
        "未跌破",
        "未突破",
        "仍高于",
        "仍低于",
        "上方区间",
        "观察区间",
        "near",
        "approach",
        "approaching",
        "shortof",
        "notyet",
    ];

    let has_near_threshold_language = threshold_terms.iter().any(|term| compact.contains(term))
        && proximity_terms.iter().any(|term| compact.contains(term));
    has_near_threshold_language || heartbeat_lower_trigger_price_contradiction(text, &compact)
}

fn heartbeat_lower_trigger_price_contradiction(text: &str, compact: &str) -> bool {
    let claims_lower_trigger = [
        "触发价≤",
        "触发价<=",
        "触发线≤",
        "触发线<=",
        "配置线≤",
        "配置线<=",
        "触及或低于触发价",
        "触及或低于触发线",
        "触及或低于配置线",
        "触及或跌破触发价",
        "触及或跌破触发线",
        "触及或跌破配置线",
        "低于触发价",
        "跌破触发价",
        "低于触发线",
        "跌破触发线",
        "低于配置线",
        "跌破配置线",
        "belowtriggerprice",
        "undertriggerprice",
    ]
    .iter()
    .any(|term| compact.contains(term));
    if !claims_lower_trigger {
        return false;
    }

    [
        RE_HEARTBEAT_CURRENT_BEFORE_TRIGGER_PRICE.captures(text),
        RE_HEARTBEAT_TRIGGER_PRICE_BEFORE_CURRENT.captures(text),
    ]
    .into_iter()
    .flatten()
    .any(|captures| {
        let current = captures
            .name("current")
            .and_then(|m| m.as_str().parse::<f64>().ok());
        let threshold = captures
            .name("threshold")
            .and_then(|m| m.as_str().parse::<f64>().ok());
        matches!((current, threshold), (Some(current), Some(threshold)) if current > threshold)
    })
}

/// 直接从 `notif_prefs_dir/{actor_slug}.json` 读 actor 的 quiet_hours + timezone。
/// 不依赖 hone-event-engine,只解析需要的两个字段；老 prefs JSON 缺字段返回 None。
/// 第二个返回值是 actor 的 timezone（IANA 名），用于 `quiet_window_active` 解释 from/to。
fn load_actor_quiet_hours(
    core: &HoneBotCore,
    actor: &hone_core::ActorIdentity,
) -> Option<(hone_core::quiet::QuietHours, Option<String>)> {
    #[derive(serde::Deserialize)]
    struct Probe {
        #[serde(default)]
        timezone: Option<String>,
        #[serde(default)]
        quiet_hours: Option<hone_core::quiet::QuietHours>,
    }
    let dir = std::path::Path::new(&core.config.storage.notif_prefs_dir);
    // 与 hone-event-engine::prefs::actor_slug 保持一致(scope 为空时用 "direct"
    // 占位,字符按 alnum/'-' 之外替换 '_'),否则文件路径不匹配,quiet_hours 永远
    // 读不到。这里复制实现避免引入 hone-event-engine 依赖。
    let scope = actor
        .channel_scope
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("direct");
    let sanitize = |s: &str| -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    };
    let slug = format!(
        "{}__{}__{}",
        sanitize(&actor.channel),
        sanitize(scope),
        sanitize(&actor.user_id)
    );
    let path = dir.join(format!("{slug}.json"));
    let text = std::fs::read_to_string(&path).ok()?;
    let probe: Probe = serde_json::from_str(&text).ok()?;
    Some((probe.quiet_hours?, probe.timezone))
}

fn truncate_for_log(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn heartbeat_similarity_stop_token(token: &str) -> bool {
    matches!(
        token,
        "已触发"
            | "再次"
            | "当前"
            | "大事"
            | "重大事"
            | "重大事件"
            | "公司"
            | "价格"
            | "任务"
            | "今日"
            | "已经"
            | "异动"
            | "提醒"
            | "事件提"
            | "件提"
            | "检查"
            | "检查时"
            | "查时"
            | "查时间"
            | "时间"
            | "最新"
            | "本轮"
            | "条件"
            | "监控"
            | "触发"
            | "重大"
    )
}

fn heartbeat_is_cjk(ch: char) -> bool {
    matches!(
        ch,
        '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
    )
}

fn heartbeat_insert_similarity_token(
    tokens: &mut std::collections::BTreeSet<String>,
    token: String,
) {
    let normalized = token
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if normalized.chars().count() < 2 || heartbeat_similarity_stop_token(&normalized) {
        return;
    }
    tokens.insert(normalized);
}

fn heartbeat_insert_cjk_similarity_tokens(
    tokens: &mut std::collections::BTreeSet<String>,
    segment: &str,
) {
    let chars = segment.chars().collect::<Vec<_>>();
    match chars.len() {
        0 | 1 => return,
        2..=8 => heartbeat_insert_similarity_token(tokens, chars.iter().collect()),
        _ => {}
    }

    for width in [2usize, 3usize] {
        if chars.len() < width {
            continue;
        }
        for window in chars.windows(width) {
            heartbeat_insert_similarity_token(tokens, window.iter().collect());
        }
    }
}

fn normalized_similarity_tokens(text: &str) -> std::collections::BTreeSet<String> {
    let mut tokens = std::collections::BTreeSet::new();
    for matched in RE_HEARTBEAT_FACT_TOKEN.find_iter(text) {
        heartbeat_insert_similarity_token(&mut tokens, matched.as_str().to_string());
    }

    let mut cjk_segment = String::new();
    for ch in text.chars() {
        if heartbeat_is_cjk(ch) {
            cjk_segment.push(ch);
        } else if !cjk_segment.is_empty() {
            heartbeat_insert_cjk_similarity_tokens(&mut tokens, &cjk_segment);
            cjk_segment.clear();
        }
    }
    if !cjk_segment.is_empty() {
        heartbeat_insert_cjk_similarity_tokens(&mut tokens, &cjk_segment);
    }

    tokens
}

fn heartbeat_duplicate_preview_match(
    message: &str,
    delivered_previews: &[(String, String)],
) -> Option<String> {
    let message_tokens = normalized_similarity_tokens(message);
    if message_tokens.len() < 4 {
        return None;
    }
    for (_, preview) in delivered_previews {
        let preview_tokens = normalized_similarity_tokens(preview);
        if preview_tokens.len() < 4 {
            continue;
        }
        let shared = message_tokens.intersection(&preview_tokens).count();
        let smaller = message_tokens.len().min(preview_tokens.len());
        let strong_match = shared >= 4 && shared * 100 >= smaller * 70;
        let reworded_fact_match = shared >= 5;
        if strong_match || reworded_fact_match {
            return Some(truncate_for_log(preview.trim(), 200));
        }
    }
    None
}

pub fn inspect_heartbeat_result(content: &str) -> (HeartbeatOutcome, HeartbeatParseKind) {
    // 先剥掉 runner 的 `<think>` / `<tool_code>` 等 reasoning 块，避免 balanced-brace
    // 扫描把 think 块里演示用的 JSON 片段（例如 `{}` / `{"status":"..."}`）误当成
    // 模型本轮的真实输出。与 `sanitize_user_visible_output` 共用同一条规则。
    let stripped = strip_internal_reasoning_blocks(content);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::Empty);
    }
    if trimmed == HEARTBEAT_NOOP_SENTINEL || heartbeat_internal_marker_present(trimmed) {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop);
    }
    if heartbeat_internal_marker_prefix(trimmed) {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::InternalMarker);
    }

    if let Some(parsed) = parse_heartbeat_json_payload(trimmed) {
        let status = parsed.status.unwrap_or_default();
        if status.eq_ignore_ascii_case("noop") {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonNoop);
        }
        if status.is_empty() {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonEmptyStatus);
        }
        if status.eq_ignore_ascii_case("triggered") {
            let raw_message = parsed.message.unwrap_or_default();
            let message = unwrap_nested_json_message(raw_message.trim())
                .trim()
                .to_string();
            if message.is_empty() || heartbeat_internal_marker_prefix(&message) {
                return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonTriggered);
            }
            return (
                HeartbeatOutcome::Deliver(message),
                HeartbeatParseKind::JsonTriggered,
            );
        }
        return (
            HeartbeatOutcome::Noop,
            HeartbeatParseKind::JsonUnknownStatus,
        );
    }

    if trimmed.starts_with('{') {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonMalformed);
    }

    (
        HeartbeatOutcome::Noop,
        HeartbeatParseKind::PlainTextSuppressed,
    )
}

pub struct ScheduledTaskExecution {
    pub should_deliver: bool,
    pub content: String,
    pub error: Option<String>,
    pub metadata: Value,
    pub session_id: Option<String>,
}

fn heartbeat_parse_error_message(parse_kind: &HeartbeatParseKind) -> Option<String> {
    match parse_kind {
        HeartbeatParseKind::Empty => Some("heartbeat 输出为空，任务已标记失败".to_string()),
        HeartbeatParseKind::JsonEmptyStatus => {
            Some("heartbeat 输出缺少状态字段，任务已标记失败".to_string())
        }
        HeartbeatParseKind::JsonUnknownStatus => {
            Some("heartbeat 输出包含未知状态，任务已标记失败".to_string())
        }
        HeartbeatParseKind::JsonMalformed => {
            Some("heartbeat 输出不是合法 JSON，任务已标记失败".to_string())
        }
        HeartbeatParseKind::PlainTextSuppressed => {
            Some("heartbeat 输出不是结构化 JSON，任务已标记失败".to_string())
        }
        _ => None,
    }
}

fn heartbeat_execution_from_content(
    content: &str,
    heartbeat_model: &str,
) -> ScheduledTaskExecution {
    let raw_preview = truncate_for_log(content.trim(), 280);
    let raw_chars = content.chars().count();
    let starts_with_json = content.trim_start().starts_with('{');
    let (outcome, parse_kind) = inspect_heartbeat_result(content);
    let metadata = json!({
        "heartbeat_model": heartbeat_model,
        "parse_kind": format!("{:?}", parse_kind),
        "raw_chars": raw_chars,
        "starts_with_json": starts_with_json,
        "raw_preview": raw_preview,
    });

    if let Some(error) = heartbeat_parse_error_message(&parse_kind) {
        return ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: Some(error),
            metadata,
            session_id: None,
        };
    }

    match outcome {
        HeartbeatOutcome::Noop => ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: None,
            metadata,
            session_id: None,
        },
        HeartbeatOutcome::Deliver(message) => {
            let sanitized_message = sanitize_scheduler_delivery_text(&message);
            if sanitized_message.trim().is_empty() {
                return ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: json!({
                        "heartbeat_model": heartbeat_model,
                        "parse_kind": format!("{:?}", parse_kind),
                        "raw_chars": raw_chars,
                        "starts_with_json": starts_with_json,
                        "raw_preview": raw_preview,
                        "deliver_preview": truncate_for_log(message.trim(), 200),
                        "sanitized_empty": true,
                    }),
                    session_id: None,
                };
            }
            let deliver_preview = truncate_for_log(message.trim(), 200);
            if heartbeat_near_threshold_without_crossing(&sanitized_message) {
                return ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: json!({
                        "heartbeat_model": heartbeat_model,
                        "parse_kind": format!("{:?}", parse_kind),
                        "raw_chars": raw_chars,
                        "starts_with_json": starts_with_json,
                        "raw_preview": raw_preview,
                        "deliver_preview": deliver_preview,
                        "near_threshold_suppressed": true,
                    }),
                    session_id: None,
                };
            }
            ScheduledTaskExecution {
                should_deliver: true,
                content: sanitized_message,
                error: None,
                metadata: json!({
                    "heartbeat_model": heartbeat_model,
                    "parse_kind": format!("{:?}", parse_kind),
                    "raw_chars": raw_chars,
                    "starts_with_json": starts_with_json,
                    "raw_preview": raw_preview,
                    "deliver_preview": deliver_preview,
                }),
                session_id: None,
            }
        }
    }
}

fn rollback_skipped_scheduler_assistant_turn(
    storage: &hone_memory::SessionStorage,
    session_id: &str,
    content: &str,
) {
    if session_id.is_empty() || content.trim().is_empty() {
        return;
    }

    match storage.remove_last_message_if_matches(session_id, "assistant", content) {
        Ok(true) => tracing::info!(
            "[SchedulerDiag] rolled back skipped assistant turn session_id={} chars={}",
            session_id,
            content.chars().count(),
        ),
        Ok(false) => tracing::warn!(
            "[SchedulerDiag] skipped assistant rollback missed tail session_id={} chars={}",
            session_id,
            content.chars().count(),
        ),
        Err(err) => tracing::warn!(
            "[SchedulerDiag] skipped assistant rollback failed session_id={} err={}",
            session_id,
            err,
        ),
    }
}

fn persist_suppressed_scheduler_failure_turn(
    storage: &hone_memory::SessionStorage,
    session_id: &str,
    failure_kind: &str,
) {
    if session_id.is_empty() {
        return;
    }

    match storage.get_messages(session_id, Some(1)) {
        Ok(messages) => {
            if messages.last().is_some_and(|message| {
                message.role == "assistant"
                    && hone_memory::session_message_text(message)
                        == SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE
            }) {
                return;
            }
        }
        Err(err) => {
            tracing::warn!(
                "[SchedulerDiag] failed to inspect session before failure transcript session_id={} err={}",
                session_id,
                err
            );
            return;
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("scheduler_failure".to_string(), Value::Bool(true));
    metadata.insert(
        "failure_kind".to_string(),
        Value::String(failure_kind.to_string()),
    );
    if let Err(err) = storage.add_message(
        session_id,
        "assistant",
        SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE,
        Some(metadata),
    ) {
        tracing::warn!(
            "[SchedulerDiag] failed to persist scheduler failure transcript session_id={} err={}",
            session_id,
            err
        );
    }
}

fn sanitize_scheduler_delivery_text(text: &str) -> String {
    let sanitized = sanitize_user_visible_output(text).content;
    let kept_lines = sanitized
        .lines()
        .filter(|line| !is_scheduler_protocol_residue(line))
        .collect::<Vec<_>>()
        .join("\n");
    kept_lines.trim().to_string()
}

fn is_empty_success_fallback(text: &str) -> bool {
    text.trim() == EMPTY_SUCCESS_FALLBACK_MESSAGE
}

fn is_scheduler_protocol_residue(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return false;
    }
    if trimmed == "{}" {
        return true;
    }

    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };

    let suspicious_keys = [
        "tool",
        "tool_call_id",
        "arguments",
        "parameters",
        "result",
        "name",
        "status",
    ];
    let user_visible_keys = ["message", "content", "text"];

    map.keys()
        .any(|key| suspicious_keys.contains(&key.as_str()))
        && !map
            .keys()
            .any(|key| user_visible_keys.contains(&key.as_str()))
}

/// 检测定时任务正文中是否包含明确的"跳过推送"信号。
/// 仅匹配直接声明"本次跳过推送"或"无需发送"的短语，避免误拦截合法内容。
pub(crate) fn has_skip_delivery_signal(text: &str) -> bool {
    let patterns = [
        "按规则应跳过正式推送",
        "按规则可跳过正式推送",
        "按规则可跳过",
        "无新增催化，跳过推送",
        "无新增催化,跳过推送",
        "可跳过正式推送",
        "按规则跳过推送",
        "跳过本次推送",
        "本轮跳过推送",
        "本次不推送",
        "本轮不推送",
        "不触发正式推送",
        "不触发本次正式推送",
        "无需正式推送",
        "无需推送",
    ];
    patterns.iter().any(|pat| text.contains(pat))
}

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
    if event.heartbeat {
        let history_section = if event.last_delivered_previews.is_empty() {
            String::new()
        } else {
            let entries = event
                .last_delivered_previews
                .iter()
                .map(|(ts, preview)| format!("  - [{}] {}", ts, preview))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "\n最近几轮已送达的提醒（供去重参考，不得重复发送相同事实）：\n{}\n\
10. 去重约束：对照上方【最近已送达】列表，若本轮检索到的事件与列表中某条内容描述的是同一个事件（相同催化 + 相同事件窗口），且没有新的独立行情时间戳、新的公告或新的状态变化，必须返回 noop，不允许重复 triggered。\n",
                entries
            )
        };
        return format!(
            "[心跳检测任务] 任务名称：{}。\n\
你正在执行一个每 30 分钟运行一次的后台条件检查。\n\
请使用可用工具检查用户设置的触发条件是否已经满足。\n\
\n\
规则：\n\
1. 如果条件尚未满足，优先只输出 `{{\"status\":\"noop\"}}`；为兼容旧行为，也允许只输出 `{{}}`。\n\
2. 如果条件已满足，只输出一段 JSON：`{{\"status\":\"triggered\",\"message\":\"...\"}}`。\n\
3. `message` 必须是一条可以直接发给用户的提醒消息，包含：满足的条件、关键数据、检查时间。\n\
4. 不要创建新的定时任务，也不要修改现有任务。\n\
5. 不要输出 Markdown 代码块，不要输出额外解释，不要暴露任何内部控制标记。\n\
6. 如果你不确定是否满足条件，或者输出格式不是严格 JSON，就必须返回 noop，不允许发送自由文本。\n\
6a. 输出契约：整条回复必须是单段 JSON，第一个可见字符必须是 `{{`。严禁使用 `<think>...</think>`、```json ... ```、`## 分析`、分步解释或任何前置/收尾的自由文本。推理过程不要对外展示；需要推理时在内部完成后，直接给出最终 JSON。\n\
7. 时间一致性约束：对于发射、财报、业绩会等有明确时间窗口的事件，必须先判断当前时间是否已越过事件预定时间，才能输出完成态结论。若当前时间早于事件计划时间，必须返回 noop，不允许把未来计划误报成已完成。\n\
8. 价格时间口径约束：引用股价时，必须核实价格的时间戳。若市场已停盘、股票停牌或价格来自上一交易日，必须在 message 中明确标注（最新可得价格为停牌前/上一交易日），不允许把旧价格包装成事件发生后的即时市场反应。\n\
9. 价格阈值口径约束：除非用户条件里明确写的是“日内最高/最低/振幅/区间波动”，否则“盘中涨跌幅超过 X%”一律按最新可得价格相对昨收的涨跌幅判断；不允许用日内高点相对昨收、日内低点相对昨收，或高低点振幅去替代当前涨跌幅。\n\
10. 若最新可得价格相对昨收尚未达到阈值，但日内高点、日内低点或盘中振幅达到阈值，且任务没有明确要求这些口径，本轮必须返回 noop，不允许触发。\n\
11. 重复事件约束：若某条件（如某只股票的某次发射或某次事件）已经在前一轮被判定为 noop 或 triggered，本轮如果没有获取到新的独立行情时间戳或新的独立事件窗口，就不允许改变结论，也不允许重复 triggered。\n\
12. 来源归因约束：引用 Reuters、WSJ、Bloomberg、官方公告等来源时，必须确认本轮工具结果明确出现该来源与对应事实；没有明确来源时，只能写“未核验/市场传闻/需继续确认”，不得把地缘政治、谈判、航运限制等叙述写成已被权威媒体共同确认的事实。\
{}\
\n以下是需要检查的用户条件：\n{}",
            event.job_name, history_section, event.task_prompt
        );
    }
    let trigger_note = format!(
        "[定时任务触发] 任务名称：{}。\n权威触发配置：repeat={}{}，北京时间 {:02}:{:02}。如果下面的用户任务正文里出现了不同的日期或时间，以这里的权威触发配置为准，不要在回复中声称本轮不是设定触发时点。\n请执行以下指令：",
        event.job_name,
        event.schedule_repeat,
        event
            .schedule_date
            .as_deref()
            .map(|date| format!(", date={date}"))
            .unwrap_or_default(),
        event.schedule_hour,
        event.schedule_minute
    );
    format!("{}\n\n{}", trigger_note, event.task_prompt)
}

pub async fn run_scheduled_task(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    mut run_options: AgentRunOptions,
) -> AgentSessionResult {
    let full_prompt = build_scheduled_prompt(event);
    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    let session = AgentSession::new(core, event.actor.clone(), event.channel_target.clone())
        .with_prompt_options(prompt_options);
    session.run(&full_prompt, run_options).await
}

pub async fn execute_scheduler_event(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    mut run_options: AgentRunOptions,
) -> ScheduledTaskExecution {
    // quiet_hours 拦截:除非任务显式 bypass,否则在用户的勿扰区间内全部跳过执行,
    // 避免 cron 任务在半夜把模型唤醒推送。落 metadata.skipped='quiet_hours' 供巡检。
    if !event.bypass_quiet_hours {
        if let Some((qh, tz_name)) = load_actor_quiet_hours(&core, &event.actor) {
            if hone_core::quiet::quiet_window_active(
                tz_name.as_deref(),
                8,
                &qh.from,
                &qh.to,
                chrono::Utc::now(),
            ) {
                tracing::info!(
                    job_id = %event.job_id,
                    job = %event.job_name,
                    quiet_from = %qh.from,
                    quiet_to = %qh.to,
                    "[SchedulerDiag] cron skipped by quiet_hours"
                );
                return ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: json!({
                        "skipped": "quiet_hours",
                        "quiet_from": qh.from,
                        "quiet_to": qh.to,
                    }),
                    session_id: None,
                };
            }
        }
    }
    if !event.heartbeat {
        let result = run_scheduled_task(core.clone(), event, prompt_options, run_options).await;
        let response = result.response;
        let session_id = result.session_id;
        return if response.success {
            let sanitized = sanitize_scheduler_delivery_text(&response.content);
            if is_empty_success_fallback(&sanitized) {
                tracing::warn!(
                    "[SchedulerDiag] empty_success_fallback job_id={} job={} chars={}",
                    event.job_id,
                    event.job_name,
                    sanitized.chars().count(),
                );
                ScheduledTaskExecution {
                    should_deliver: true,
                    content: String::new(),
                    error: Some(sanitized),
                    metadata: json!({
                        "failure_kind": "empty_success_fallback",
                    }),
                    session_id: Some(session_id),
                }
            } else if has_skip_delivery_signal(&sanitized) {
                tracing::info!(
                    "[SchedulerDiag] skip_signal job_id={} job={} chars={}",
                    event.job_id,
                    event.job_name,
                    sanitized.chars().count(),
                );
                rollback_skipped_scheduler_assistant_turn(
                    &core.session_storage,
                    &session_id,
                    &sanitized,
                );
                ScheduledTaskExecution {
                    should_deliver: false,
                    content: sanitized,
                    error: None,
                    metadata: Value::Null,
                    session_id: Some(session_id),
                }
            } else {
                ScheduledTaskExecution {
                    should_deliver: true,
                    content: sanitized,
                    error: None,
                    metadata: Value::Null,
                    session_id: Some(session_id),
                }
            }
        } else {
            let sanitized_error = user_visible_error_message_or_none(response.error.as_deref());
            if sanitized_error.is_none() {
                tracing::warn!(
                    "[SchedulerDiag] suppressed internal failure fallback job_id={} job={} error=\"{}\"",
                    event.job_id,
                    event.job_name,
                    response.error.as_deref().unwrap_or("").replace('\n', "\\n"),
                );
                persist_suppressed_scheduler_failure_turn(
                    &core.session_storage,
                    &session_id,
                    "internal_error_suppressed",
                );
            }
            ScheduledTaskExecution {
                should_deliver: sanitized_error.is_some(),
                content: String::new(),
                error: sanitized_error,
                metadata: json!({
                    "failure_kind": "internal_error_suppressed",
                }),
                session_id: Some(session_id),
            }
        };
    }

    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    run_options.model_override = Some(core.auxiliary_model_name());
    let heartbeat_model = run_options.model_override.clone().unwrap_or_default();

    match run_heartbeat_task(core, event, prompt_options, run_options).await {
        Ok(content) => {
            let raw_preview = truncate_for_log(content.trim(), 280);
            let raw_chars = content.chars().count();
            let starts_with_json = content.trim_start().starts_with('{');
            let (outcome, parse_kind) = inspect_heartbeat_result(&content);
            tracing::info!(
                "[HeartbeatDiag] job_id={} job={} target={} model={} raw_chars={} starts_with_json={} parse_kind={:?} raw_preview=\"{}\"",
                event.job_id,
                event.job_name,
                event.channel_target,
                heartbeat_model,
                raw_chars,
                starts_with_json,
                parse_kind,
                raw_preview.replace('\n', "\\n"),
            );
            if parse_kind == HeartbeatParseKind::JsonMalformed {
                tracing::warn!(
                    "[HeartbeatDiag] malformed heartbeat json suppressed job_id={} job={} target={} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    raw_preview.replace('\n', "\\n"),
                );
            }
            if matches!(
                parse_kind,
                HeartbeatParseKind::JsonUnknownStatus | HeartbeatParseKind::JsonMalformed
            ) {
                tracing::warn!(
                    "[HeartbeatDiag] parse failure escalated job_id={} job={} target={} parse_kind={:?} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    parse_kind,
                    raw_preview.replace('\n', "\\n"),
                );
            }
            if let HeartbeatOutcome::Deliver(message) = &outcome {
                let deliver_preview = truncate_for_log(message.trim(), 200);
                tracing::info!(
                    "[HeartbeatDiag] deliver job_id={} job={} target={} parse_kind={:?} deliver_chars={} deliver_preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    parse_kind,
                    message.chars().count(),
                    deliver_preview.replace('\n', "\\n"),
                );
            }
            let mut execution = heartbeat_execution_from_content(&content, &heartbeat_model);
            if execution.should_deliver
                && let Some(matched_preview) = heartbeat_duplicate_preview_match(
                    &execution.content,
                    &event.last_delivered_previews,
                )
            {
                tracing::info!(
                    "[HeartbeatDiag] duplicate_suppressed job_id={} job={} target={} matched_preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    matched_preview.replace('\n', "\\n"),
                );
                execution.should_deliver = false;
                execution.content.clear();
                execution.error = None;
                execution.metadata = json!({
                    "heartbeat_model": heartbeat_model,
                    "parse_kind": format!("{:?}", parse_kind),
                    "duplicate_suppressed": true,
                    "matched_preview": matched_preview,
                });
            }
            execution
        }
        Err(error) => {
            let (parse_kind_label, treat_as_noop) = if is_context_overflow_error(&error) {
                ("ContextOverflowNoop", true)
            } else {
                ("", false)
            };
            if treat_as_noop {
                tracing::warn!(
                    "[HeartbeatDiag] transient_noop parse_kind={} job_id={} job={} target={} model={} error=\"{}\"",
                    parse_kind_label,
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    heartbeat_model,
                    truncate_for_log(&error, 280).replace('\n', "\\n"),
                );
                ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: json!({
                        "heartbeat_model": heartbeat_model,
                        "parse_kind": parse_kind_label,
                    }),
                    session_id: None,
                }
            } else {
                tracing::warn!(
                    "[HeartbeatDiag] runner_error job_id={} job={} target={} model={} error=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    heartbeat_model,
                    truncate_for_log(&error, 280).replace('\n', "\\n"),
                );
                ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: Some(error),
                    metadata: json!({
                        "heartbeat_model": heartbeat_model,
                    }),
                    session_id: None,
                }
            }
        }
    }
}

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

async fn run_heartbeat_task(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    run_options: AgentRunOptions,
) -> Result<String, String> {
    let transient_session_id = format!("heartbeat_probe::{}", event.job_id);
    let mut bundle = build_prompt_bundle(
        &core.config,
        &core.session_storage,
        &event.actor.channel,
        &transient_session_id,
        &Default::default(),
        &prompt_options,
    );
    // 与 agent_session.rs::resolve_prompt_input 一致：self-managed-context runner
    // 不需要 honeclaw 灌注 conversation_context，runner 自带 ACP session 管理。
    if core.config.agent.runner_kind().manages_own_context() {
        bundle.conversation_context = None;
    }
    let timeout = run_options.timeout;
    let execution = ExecutionService::new(core.clone()).prepare(ExecutionRequest {
        mode: ExecutionMode::TransientTask,
        session_id: transient_session_id.clone(),
        actor: event.actor.clone(),
        channel_target: event.channel_target.clone(),
        allow_cron: false,
        system_prompt: bundle.system_prompt(),
        runtime_input: bundle.compose_user_input(&build_scheduled_prompt(event)),
        context: hone_core::agent::AgentContext::new(transient_session_id),
        timeout,
        gemini_stream: timeout
            .map(|duration| GeminiStreamOptions {
                overall_timeout: duration,
                per_line_timeout: core.config.agent.step_timeout(),
                ..GeminiStreamOptions::default()
            })
            .unwrap_or_default(),
        session_metadata: std::collections::HashMap::new(),
        model_override: run_options.model_override.clone(),
        runner_selection: ExecutionRunnerSelection::AuxiliaryFunctionCalling {
            max_iterations: HEARTBEAT_MAX_ITERATIONS,
        },
        allowed_tools: None,
        max_tool_calls: None,
        prompt_audit: None,
    })?;
    tracing::info!(
        "[HeartbeatDiag] run_start job_id={} job={} target={} runner={} model_override={} timeout_secs={}",
        event.job_id,
        event.job_name,
        event.channel_target,
        execution.runner_name,
        run_options.model_override.as_deref().unwrap_or(""),
        timeout.map(|duration| duration.as_secs()).unwrap_or(0),
    );
    let result = execution
        .runner
        .run(execution.runner_request, Arc::new(NoopEmitter))
        .await;
    if result.response.success {
        tracing::info!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=true content_chars={}",
            event.job_id,
            event.job_name,
            event.channel_target,
            execution.runner_name,
            result.response.content.chars().count(),
        );
        Ok(result.response.content)
    } else {
        tracing::warn!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=false error=\"{}\"",
            event.job_id,
            event.job_name,
            event.channel_target,
            execution.runner_name,
            truncate_for_log(
                result
                    .response
                    .error
                    .as_deref()
                    .unwrap_or("心跳检测执行失败"),
                280
            )
            .replace('\n', "\\n"),
        );
        Err(result
            .response
            .error
            .unwrap_or_else(|| "心跳检测执行失败".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HeartbeatOutcome, HeartbeatParseKind, SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE,
        build_scheduled_prompt, execute_scheduler_event, has_skip_delivery_signal,
        heartbeat_duplicate_preview_match, heartbeat_execution_from_content,
        inspect_heartbeat_result, is_empty_success_fallback, load_actor_quiet_hours,
        persist_suppressed_scheduler_failure_turn, rollback_skipped_scheduler_assistant_turn,
        sanitize_scheduler_delivery_text,
    };
    use crate::HoneBotCore;
    use crate::agent_session::{AgentRunOptions, AgentRunQuotaMode};
    use crate::prompt::PromptOptions;
    use crate::response_finalizer::EMPTY_SUCCESS_FALLBACK_MESSAGE;
    use hone_core::config::HoneConfig;
    use hone_core::{ActorIdentity, quiet::QuietHours};
    use hone_memory::{SessionStorage, session_message_text};
    use hone_scheduler::SchedulerEvent;
    use serde_json::Value;
    use std::sync::Arc;

    #[test]
    fn heartbeat_exact_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("[[HEARTBEAT_NOOP]]").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_partial_internal_marker_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("[[HEART").0,
            HeartbeatOutcome::Noop
        );
        assert_eq!(
            inspect_heartbeat_result("  [[HEARTBEAT").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_json_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result(r#"{"status":"noop"}"#).0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_json_triggered_delivers_message_only() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"闪迪股价已低于 520，当前 519.7（检查时间：09:30）"}"#
            )
            .0,
            HeartbeatOutcome::Deliver(
                "闪迪股价已低于 520，当前 519.7（检查时间：09:30）".to_string()
            )
        );
    }

    #[test]
    fn heartbeat_near_threshold_trigger_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"ASTS 最新价格 $71.88，相对昨收 $77.20 跌幅 -6.89%，触发原因：单日涨跌幅（跌）接近 8% 警戒阈值，且距离 8% 仅差约 1.1 个百分点。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_explicit_below_threshold_denial_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"触发条件：单日涨跌幅超过 8%。ASTS 当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛，本轮仅建议观察。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_explicit_not_triggered_threshold_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"RKLB异动提醒：最新价$77.02，较前收$78.59下跌-2.00%，未触发涨跌幅8%阈值，仅记录重大事件观察。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_explicit_not_exceeding_threshold_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"RKLB触发重大订单提醒：当前股价$77.02，涨跌幅未超过8%阈值，合同事件仅作观察。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_watchlist_above_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"ASTS 当前 71.88，触发价≤69.83，仍高于触发价但已进入触发价上方区间，建议关注。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_watchlist_contradictory_lower_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"【价格提醒】ASTS触发买入条件。当前价格$71.88，已低于触发价$69.83。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_watchlist_touch_or_below_above_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"【触发条件】ASTS 跌至 69.85，已触及或低于触发价 69.83。"}"#,
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(execution.error, None);
        assert_eq!(execution.metadata["near_threshold_suppressed"], true);
    }

    #[test]
    fn heartbeat_prefixed_json_triggered_delivers_message_only() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"当前时间：09:00:58，小时数为9，分钟数0 < 30，条件满足。正在查询原油价格...
{"status":"triggered","message":"【原油价格播报 - 09:00】"}"#
            )
            .0,
            HeartbeatOutcome::Deliver("【原油价格播报 - 09:00】".to_string())
        );
    }

    #[test]
    fn heartbeat_prefixed_json_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("先检查一下...\n{\"status\":\"noop\"}").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_plain_text_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("闪迪股价已低于 520，当前 519.7（检查时间：09:30）"),
            (
                HeartbeatOutcome::Noop,
                HeartbeatParseKind::PlainTextSuppressed
            )
        );
    }

    #[test]
    fn heartbeat_plain_text_marks_execution_failed() {
        let execution = heartbeat_execution_from_content(
            "闪迪股价已低于 520，当前 519.7（检查时间：09:30）",
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出不是结构化 JSON，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "PlainTextSuppressed");
        assert_eq!(execution.metadata["heartbeat_model"], "model-x");
    }

    #[test]
    fn heartbeat_think_wrapped_json_noop_is_suppressed() {
        let content = "<think> 当前小米股价为30.88港元，高于30港元的触发线，所以条件未满足。根据规则，我应该输出 `{\"status\":\"noop\"}` 或 `[[HEARTBEAT_NOOP]]`。 </think>\n{\"status\":\"noop\"}";
        assert_eq!(inspect_heartbeat_result(content).0, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_think_wrapped_noop_marker_is_suppressed() {
        let content = "<think>\n让我检查一下这个心跳检测任务的条件。\n\n当前北京时间：2026-04-05 08:30:00\n当前小时数：8\n当前分钟数：30\n\n用户条件：\n如果当前小时数是 0、3、6、9、12、15、18、21 其中之一\n并且当前分钟数小于 30 分钟\n当前小时数 8 不在 [0, 3, 6, 9, 12, 15, 18, 21] 这个列表中，所以条件不满足。\n\n按照规则，我应该保持静默，不输出任何内容。\n</think>\n\n[[HEARTBEAT_NOOP]]";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop)
        );
    }

    #[test]
    fn heartbeat_english_think_wrapped_noop_marker_is_suppressed() {
        let content = "<think>\nLet me analyze this request carefully.\n\nThe user is asking me to check if a heartbeat condition has been met. Let me parse the condition:\nCheck if current hour (Beijing time) is one of: 0, 3, 6, 9, 12, 15, 18, 21\nAND current minute is less than 30\nCurrent time: 2026-04-05 07:30:00 (Beijing time)\nHour: 07 (7)\nMinute: 30\nIs 7 in [0, 3, 6, 9, 12, 15, 18, 21]? No.\nTherefore, the condition is NOT met.\n\n</think>\n\n[[HEARTBEAT_NOOP]]";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop)
        );
    }

    // 2026-04-24 真实 heartbeat 样本：think 块里演示 `{}` 作为 noop 示例，随后
    // LLM 只输出裸 `{}`。strip_internal_reasoning_blocks 让 balanced-brace 扫描不再
    // 误把 think 里演示的 `{"status":"triggered",...}` 当成真实输出。
    #[test]
    fn heartbeat_think_wrapped_empty_json_is_suppressed() {
        let content = "<think>\n示例：条件满足时应输出 `{\"status\":\"triggered\",\"message\":\"...\"}`，否则输出 `{}`。\n当前条件未满足。\n</think>\n{}";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonEmptyStatus)
        );
    }

    // think 块内部若出现 `{"status":"triggered",...}` 作为「如何输出」的示例，
    // 而 think 块外没有独立 JSON，整体应视为 noop，不能把示例 JSON 误当成真实触发。
    #[test]
    fn heartbeat_think_demo_triggered_without_outer_json_is_suppressed() {
        let content = "<think>\n如果条件满足，我应该输出 `{\"status\":\"triggered\",\"message\":\"小米跌破 30 港元\"}`。\n当前小米股价 31.2 港元，未跌破 30，所以不触发。\n</think>";
        let (outcome, parse_kind) = inspect_heartbeat_result(content);
        assert_eq!(outcome, HeartbeatOutcome::Noop, "parse_kind={parse_kind:?}");
    }

    #[test]
    fn heartbeat_think_wrapped_triggered_json_delivers_message_only() {
        let content = "<think> 先整理结果。最终应该输出 JSON。 </think>\n{\"status\":\"triggered\",\"message\":\"小米已跌破 30 港元，当前 29.88 港元（检查时间：22:33）\"}";
        assert_eq!(
            inspect_heartbeat_result(content),
            (
                HeartbeatOutcome::Deliver(
                    "小米已跌破 30 港元，当前 29.88 港元（检查时间：22:33）".to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_json_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result(r#"{"status":"noop"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn scheduler_delivery_text_strips_internal_blocks_and_tool_protocol() {
        let raw =
            "<think>先判断一下</think>\n最终答案\n\n<tool_call>{\"tool\":\"cron_job\"}</tool_call>";
        let sanitized = sanitize_scheduler_delivery_text(raw);
        assert_eq!(sanitized, "最终答案");
    }

    #[test]
    fn scheduler_delivery_text_keeps_user_visible_json_message() {
        let raw = r#"{"status":"triggered","message":"今晚 20:30 继续复盘"}"#;
        let sanitized = sanitize_scheduler_delivery_text(raw);
        assert_eq!(sanitized, raw);
    }

    #[test]
    fn scheduler_detects_empty_success_fallback_as_failure_content() {
        assert!(is_empty_success_fallback(EMPTY_SUCCESS_FALLBACK_MESSAGE));
        assert!(is_empty_success_fallback(&format!(
            "\n{}\n",
            EMPTY_SUCCESS_FALLBACK_MESSAGE
        )));
        assert!(!is_empty_success_fallback("这是正常的定时任务输出"));
    }

    #[test]
    fn skip_signal_rolls_back_persisted_assistant_turn() {
        let root = std::env::temp_dir().join(format!(
            "hone_scheduler_skip_rollback_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("create root");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "ou_skip", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");
        let skipped_content =
            "TEM 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送";

        storage
            .add_message(&session_id, "user", "[定时任务触发] TEM", None)
            .expect("add user");
        storage
            .add_message(&session_id, "assistant", skipped_content, None)
            .expect("add assistant");

        rollback_skipped_scheduler_assistant_turn(&storage, &session_id, skipped_content);

        let messages = storage
            .get_messages(&session_id, None)
            .expect("get messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(session_message_text(&messages[0]), "[定时任务触发] TEM");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn suppressed_scheduler_failure_persists_single_transcript_marker() {
        let root = std::env::temp_dir().join(format!(
            "hone_scheduler_failure_marker_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("create root");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "ou_failure", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        storage
            .add_message(&session_id, "user", "[定时任务触发] 盘前复盘", None)
            .expect("add user");
        persist_suppressed_scheduler_failure_turn(
            &storage,
            &session_id,
            "internal_error_suppressed",
        );
        persist_suppressed_scheduler_failure_turn(
            &storage,
            &session_id,
            "internal_error_suppressed",
        );

        let messages = storage
            .get_messages(&session_id, None)
            .expect("get messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(
            session_message_text(&messages[1]),
            SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE
        );
        let metadata = messages[1].metadata.as_ref().expect("metadata");
        assert_eq!(
            metadata
                .get("scheduler_failure")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            metadata
                .get("failure_kind")
                .and_then(|value| value.as_str()),
            Some("internal_error_suppressed")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn heartbeat_truncated_json_prefix_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result(r#"{"status"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_single_brace_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result("{");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_unknown_json_status_is_suppressed() {
        let (outcome, parse_kind) =
            inspect_heartbeat_result(r#"{"status":"maybe","message":"foo"}"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonUnknownStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_unknown_json_status_marks_execution_failed() {
        let execution =
            heartbeat_execution_from_content(r#"{"status":"maybe","message":"foo"}"#, "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出包含未知状态，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "JsonUnknownStatus");
        assert_eq!(execution.metadata["heartbeat_model"], "model-x");
        assert!(
            execution.metadata["raw_preview"]
                .as_str()
                .expect("raw_preview")
                .contains("\"status\":\"maybe\"")
        );
    }

    #[test]
    fn heartbeat_empty_json_marks_execution_failed() {
        let (outcome, parse_kind) = inspect_heartbeat_result("{}");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonEmptyStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
        let execution = heartbeat_execution_from_content("{}", "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出缺少状态字段，任务已标记失败")
        );
    }

    #[test]
    fn heartbeat_think_plus_empty_json_marks_execution_failed() {
        let (outcome, parse_kind) = inspect_heartbeat_result("<think>reasoning</think>\n\n{}");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonEmptyStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
        let execution =
            heartbeat_execution_from_content("<think>reasoning</think>\n\n{}", "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出缺少状态字段，任务已标记失败")
        );
    }

    #[test]
    fn heartbeat_empty_output_marks_execution_failed() {
        let execution = heartbeat_execution_from_content("", "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出为空，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "Empty");
    }

    #[test]
    fn heartbeat_nested_json_message_is_unwrapped() {
        let raw =
            r#"{"status":"triggered","message":"{\"trigger\":\"标的: TEM\\n事件: 大事件\"}"}"#;
        let (outcome, parse_kind) = inspect_heartbeat_result(raw);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonTriggered);
        assert_eq!(
            outcome,
            HeartbeatOutcome::Deliver("标的: TEM\n事件: 大事件".to_string())
        );
    }

    #[test]
    fn heartbeat_malformed_json_marks_execution_failed() {
        let execution = heartbeat_execution_from_content(r#"{"status":"noop"#, "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出不是合法 JSON，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "JsonMalformed");
    }

    #[test]
    fn heartbeat_prompt_keeps_legacy_empty_json_example_literal() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("discord", "alice", Some("dm")).expect("actor"),
            job_id: "job-1".to_string(),
            job_name: "heartbeat".to_string(),
            task_prompt: "检查条件".to_string(),
            channel: "discord".to_string(),
            channel_scope: Some("dm".to_string()),
            channel_target: "alice".to_string(),
            delivery_key: "delivery-1".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 0,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("也允许只输出 `{}`。"));
        assert!(!prompt.contains("[[HEARTBEAT_NOOP]]"));
    }

    #[test]
    fn heartbeat_prompt_includes_delivery_history_when_present() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_abc", None::<String>).expect("actor"),
            job_id: "job-2".to_string(),
            job_name: "ASTS 重大异动心跳监控".to_string(),
            task_prompt: "ASTS 异动监控".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_abc".to_string(),
            delivery_key: "delivery-2".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 0,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![
                (
                    "2026-04-20T05:01:00+08:00".to_string(),
                    "BlueBird 7 低轨事件".to_string(),
                ),
                (
                    "2026-04-20T04:31:00+08:00".to_string(),
                    "BlueBird 7 发射".to_string(),
                ),
            ],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("最近几轮已送达的提醒"));
        assert!(prompt.contains("BlueBird 7 低轨事件"));
        assert!(prompt.contains("去重约束"));
        assert!(prompt.contains("不允许重复 triggered"));
    }

    #[test]
    fn heartbeat_duplicate_preview_match_suppresses_same_event() {
        let message = "【RKLB 重大事件提醒】Blue Origin Blue Ring 与 Rocket Lab 相关合作再次被报道，检查时间 02:01";
        let previews = vec![(
            "2026-04-25T23:01:00+08:00".to_string(),
            "【RKLB 重大事件提醒】Blue Origin Blue Ring 与 Rocket Lab 相关合作已触发提醒，检查时间 23:01"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_some());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_suppresses_reworded_cjk_event() {
        let message = "【TEM大事件触发提醒】TEM 当前上涨 +10.92%，5月5日财报、TIME 2026 健康与生命科学公司十强、USC 战略合作继续发酵。";
        let previews = vec![(
            "2026-05-01T17:31:01+08:00".to_string(),
            "【TEM 价格异动触发】4月28日 TIME 榜单、USC 合作、5月5日财报已提醒，检查时间 17:31。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_some());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_suppresses_reworded_contract_event() {
        let message =
            "【RKLB重大订单】Rocket Lab 于4月29日获批 1.9 亿美元国防合同，本轮价格接近阈值。";
        let previews = vec![(
            "2026-04-30T13:00:31+08:00".to_string(),
            "RKLB异动提醒：Rocket Lab 4月29日宣布赢得1.9亿美元国防合同，已发送。".to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_some());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_new_event() {
        let message = "【ASTS 重大事件提醒】公司宣布新的卫星发射窗口，检查时间 02:01";
        let previews = vec![(
            "2026-04-25T23:01:00+08:00".to_string(),
            "【RKLB 重大事件提醒】Blue Origin Blue Ring 与 Rocket Lab 相关合作已触发提醒"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_new_same_ticker_event() {
        let message = "【TEM大事件提醒】TEM 宣布新的 FDA 批准结果，检查时间 02:01。";
        let previews = vec![(
            "2026-05-01T17:31:01+08:00".to_string(),
            "【TEM 价格异动触发】4月28日 TIME 榜单、USC 合作、5月5日财报已提醒。".to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_prompt_no_history_section_when_empty() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_abc", None::<String>).expect("actor"),
            job_id: "job-3".to_string(),
            job_name: "新任务".to_string(),
            task_prompt: "条件检查".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_abc".to_string(),
            delivery_key: "delivery-3".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 0,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(!prompt.contains("最近几轮已送达的提醒"));
        assert!(!prompt.contains("去重约束"));
    }

    #[test]
    fn skip_delivery_signal_detected() {
        assert!(has_skip_delivery_signal(
            "AAOI 今日没有出现新的实质性催化或风险证伪信号，按规则应跳过正式推送，以下是背景分析..."
        ));
        assert!(has_skip_delivery_signal(
            "RKLB 今日未发现新的实质性催化或风险证伪信号，按规则可跳过正式推送。"
        ));
        assert!(has_skip_delivery_signal(
            "TEM 今日无新增公司级催化，不触发正式推送。"
        ));
        assert!(has_skip_delivery_signal("当前行情平稳，跳过本次推送。"));
        assert!(!has_skip_delivery_signal("AAOI 今日出现重大利好，建议关注"));
        assert!(!has_skip_delivery_signal(""));
    }

    #[test]
    fn heartbeat_prompt_clarifies_price_threshold_semantics() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_threshold", None::<String>).expect("actor"),
            job_id: "job-4".to_string(),
            job_name: "ORCL 大事件监控".to_string(),
            task_prompt: "若 ORCL 盘中涨跌幅超过 5%，提醒我".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_threshold".to_string(),
            delivery_key: "delivery-4".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 0,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("盘中涨跌幅超过 X%"));
        assert!(prompt.contains("不允许用日内高点相对昨收"));
        assert!(prompt.contains("本轮必须返回 noop"));
    }

    #[test]
    fn heartbeat_prompt_requires_source_grounding_for_geopolitics() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_oil", None::<String>).expect("actor"),
            job_id: "job-oil".to_string(),
            job_name: "全天原油价格3小时播报".to_string(),
            task_prompt: "播报 WTI/Brent，并说明地缘政治影响".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_oil".to_string(),
            delivery_key: "delivery-oil".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 0,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("来源归因约束"));
        assert!(prompt.contains("必须确认本轮工具结果明确出现该来源"));
        assert!(prompt.contains("未核验/市场传闻/需继续确认"));
    }

    fn make_test_core(prefs_dir: &std::path::Path) -> Arc<HoneBotCore> {
        let mut config = HoneConfig::default();
        let root = prefs_dir.parent().unwrap();
        config.storage.notif_prefs_dir = prefs_dir.to_string_lossy().to_string();
        config.storage.sessions_dir = root.join("sessions").to_string_lossy().to_string();
        config.storage.session_sqlite_db_path =
            root.join("sessions.sqlite3").to_string_lossy().to_string();
        config.storage.llm_audit_db_path =
            root.join("llm_audit.sqlite3").to_string_lossy().to_string();
        config.storage.portfolio_dir = root.join("portfolio").to_string_lossy().to_string();
        config.storage.cron_jobs_dir = root.join("cron_jobs").to_string_lossy().to_string();
        config.storage.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        Arc::new(HoneBotCore::new(config))
    }

    fn write_prefs_with_quiet(prefs_dir: &std::path::Path, actor: &ActorIdentity, qh: QuietHours) {
        std::fs::create_dir_all(prefs_dir).unwrap();
        let scope = actor
            .channel_scope
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("direct");
        let slug = format!("{}__{}__{}", actor.channel, scope, actor.user_id);
        let body = serde_json::json!({
            "timezone": "UTC",
            "quiet_hours": { "from": qh.from, "to": qh.to, "exempt_kinds": qh.exempt_kinds },
        });
        std::fs::write(
            prefs_dir.join(format!("{slug}.json")),
            serde_json::to_string(&body).unwrap(),
        )
        .unwrap();
    }

    fn quiet_hours_around_now() -> QuietHours {
        use chrono::Timelike;
        let now = chrono::Utc::now();
        let now_min = now.hour() as i32 * 60 + now.minute() as i32;
        let from_m = ((now_min - 30).rem_euclid(24 * 60)) as u32;
        let to_m = ((now_min + 30).rem_euclid(24 * 60)) as u32;
        QuietHours {
            from: format!("{:02}:{:02}", from_m / 60, from_m % 60),
            to: format!("{:02}:{:02}", to_m / 60, to_m % 60),
            exempt_kinds: Vec::new(),
        }
    }

    fn make_event(actor: ActorIdentity, bypass: bool) -> SchedulerEvent {
        SchedulerEvent {
            actor,
            job_id: "j_quiet_test".into(),
            job_name: "quiet test".into(),
            task_prompt: "noop".into(),
            channel: "imessage".into(),
            channel_scope: None,
            channel_target: "test".into(),
            delivery_key: "k1".into(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 9,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: bypass,
        }
    }

    #[test]
    fn load_actor_quiet_hours_returns_none_when_file_absent() {
        let dir = tempfile::tempdir().unwrap();
        let prefs_dir = dir.path().join("notif_prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("imessage", "ghost", None::<String>).unwrap();
        assert!(load_actor_quiet_hours(&core, &actor).is_none());
    }

    #[test]
    fn load_actor_quiet_hours_reads_field_correctly() {
        let dir = tempfile::tempdir().unwrap();
        let prefs_dir = dir.path().join("notif_prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).unwrap();
        write_prefs_with_quiet(
            &prefs_dir,
            &actor,
            QuietHours {
                from: "23:00".into(),
                to: "07:00".into(),
                exempt_kinds: vec!["earnings_released".into()],
            },
        );
        let (qh, tz) = load_actor_quiet_hours(&core, &actor).expect("present");
        assert_eq!(qh.from, "23:00");
        assert_eq!(qh.to, "07:00");
        assert_eq!(qh.exempt_kinds, vec!["earnings_released".to_string()]);
        assert_eq!(tz.as_deref(), Some("UTC"));
    }

    #[tokio::test]
    async fn execute_scheduler_event_skips_during_quiet_hours() {
        let dir = tempfile::tempdir().unwrap();
        let prefs_dir = dir.path().join("notif_prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).unwrap();
        write_prefs_with_quiet(&prefs_dir, &actor, quiet_hours_around_now());

        let event = make_event(actor, /* bypass */ false);
        let mut run_options = AgentRunOptions::default();
        run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
        let result =
            execute_scheduler_event(core, &event, PromptOptions::default(), run_options).await;

        assert!(!result.should_deliver, "quiet 内不应送达");
        assert!(result.session_id.is_none(), "skipped 不应携带 session_id");
        assert_eq!(
            result.metadata.get("skipped").and_then(|v| v.as_str()),
            Some("quiet_hours")
        );
    }

    #[tokio::test]
    async fn execute_scheduler_event_with_bypass_does_not_short_circuit_on_quiet() {
        // bypass=true → 不应在 quiet_hours 这一步早退;后续会走真实调度逻辑(没 LLM 配置会失败,
        // 但不会落 metadata.skipped='quiet_hours'),足以证明 quiet 闸门没拦下来。
        let dir = tempfile::tempdir().unwrap();
        let prefs_dir = dir.path().join("notif_prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).unwrap();
        write_prefs_with_quiet(&prefs_dir, &actor, quiet_hours_around_now());

        let event = make_event(actor, /* bypass */ true);
        let mut run_options = AgentRunOptions::default();
        run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
        let result =
            execute_scheduler_event(core, &event, PromptOptions::default(), run_options).await;
        assert_ne!(
            result.metadata.get("skipped").and_then(|v| v.as_str()),
            Some("quiet_hours"),
            "bypass=true 应避开 quiet_hours 早退分支"
        );
    }

    #[tokio::test]
    async fn execute_scheduler_event_no_quiet_set_does_not_skip() {
        let dir = tempfile::tempdir().unwrap();
        let prefs_dir = dir.path().join("notif_prefs");
        std::fs::create_dir_all(&prefs_dir).unwrap();
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("imessage", "u1", None::<String>).unwrap();
        // 不写 prefs 文件 → quiet_hours None → 不拦截
        let event = make_event(actor, /* bypass */ false);
        let mut run_options = AgentRunOptions::default();
        run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
        let result =
            execute_scheduler_event(core, &event, PromptOptions::default(), run_options).await;
        assert_ne!(
            result.metadata.get("skipped").and_then(|v| v.as_str()),
            Some("quiet_hours"),
            "无 quiet_hours 不应 skip"
        );
    }
}
