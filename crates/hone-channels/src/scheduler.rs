use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use async_trait::async_trait;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike};
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
const HEARTBEAT_MAX_ITERATIONS: u32 = 18;
const HEARTBEAT_MAX_TOKENS: u16 = 4096;
const HEARTBEAT_ALLOWED_TOOLS: &[&str] = &[
    "data_fetch",
    "web_search",
    "portfolio",
    "missed_events",
    "local_list_files",
    "local_search_files",
    "local_read_file",
];

fn heartbeat_runner_selection() -> ExecutionRunnerSelection {
    ExecutionRunnerSelection::AuxiliaryFunctionCalling {
        max_iterations: HEARTBEAT_MAX_ITERATIONS,
        max_tokens_override: Some(HEARTBEAT_MAX_TOKENS),
    }
}
const SCHEDULER_INTERNAL_FAILURE_TRANSCRIPT_MESSAGE: &str =
    "本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。";
const SCHEDULER_INTERNAL_FAILURE_LEDGER_MESSAGE: &str =
    "定时任务执行环境暂时不可用，系统已记录失败并将在下一次触发时重试。";
const STALE_MARKET_DATA_FAILURE_MESSAGE: &str =
    "本轮定时任务未能完成：关键行情数据获取失败，系统已跳过旧价格版本，并将在下一次触发时重试。";

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

static RE_HEARTBEAT_PRICE_TIMESTAMP_DATE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?isx)
        (?:当前(?:价格|价)?|最新(?:价格|价)?|现价|现报|收盘价|跌至|跌到|降至|回落至|current(?:\s*price)?|last(?:\s*price)?|quote)
        [^\n。；;]{0,120}
        (?:
            (?P<year_cn>20\d{2})年(?P<month_cn>\d{1,2})月(?P<day_cn>\d{1,2})日
            |
            (?P<year_iso>20\d{2})[-/](?P<month_iso>\d{1,2})[-/](?P<day_iso>\d{1,2})
        )
        ",
    )
    .expect("valid heartbeat price timestamp regex")
});

const HEARTBEAT_PRICE_TIMESTAMP_MAX_AGE_DAYS: i64 = 3;

static RE_HEARTBEAT_BEIJING_TRIGGER_TIME: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"北京时间\s*(?P<hour>\d{1,2})(?:[:：点时](?P<minute>\d{1,2})?)?(?:分)?(?P<tail>\s*[^\n。；;]{0,24}(?:监控|检查|心跳|任务|本轮)[^\n。；;]{0,16}触发)",
    )
    .expect("valid heartbeat beijing trigger time regex")
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

static RE_HEARTBEAT_ENTITY_ANCHOR: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"[A-Za-z][A-Za-z0-9.-]{1,}").expect("valid heartbeat entity anchor regex")
});

static RE_HEARTBEAT_REVISION_FACT_TOKEN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?ix)
        \$\s*\d+(?:\.\d+)?(?:\s*(?:-|–|~|至|到)\s*\$?\s*\d+(?:\.\d+)?)?
        |
        \d+(?:\.\d+)?\s*%
        |
        \d+(?:\.\d+)?\s*(?:亿|万)?\s*(?:美元|美金|港元|人民币|元|股)
        ",
    )
    .expect("valid heartbeat revision fact regex")
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
    PlainTextNoop,
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
                    if previous_visible_char(content, idx) == Some('`') {
                        continue;
                    }
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0
                    && let Some(start_idx) = start.take()
                {
                    candidates.push(&trimmed[start_idx..=idx]);
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

fn heartbeat_status_indicates_noop(status: &str) -> bool {
    let compact = status
        .split_whitespace()
        .collect::<String>()
        .replace(['-', '_'], "")
        .to_ascii_lowercase();
    matches!(
        compact.as_str(),
        "noop"
            | "none"
            | "false"
            | "nottriggered"
            | "notrigger"
            | "conditionnotmet"
            | "conditionsnotmet"
            | "notmet"
            | "unmet"
            | "skip"
            | "skipped"
            | "nosend"
    ) || status.contains("未触发")
        || status.contains("不触发")
        || status.contains("未满足")
        || status.contains("不满足")
}

fn heartbeat_status_indicates_triggered(status: &str) -> bool {
    let compact = status
        .split_whitespace()
        .collect::<String>()
        .replace(['-', '_'], "")
        .to_ascii_lowercase();
    matches!(
        compact.as_str(),
        "triggered"
            | "trigger"
            | "alert"
            | "send"
            | "true"
            | "yes"
            | "met"
            | "hit"
            | "conditionmet"
            | "conditionsmet"
    ) || status.contains("已触发")
        || status.contains("触发")
        || status.contains("命中")
        || status.contains("满足")
}

fn previous_visible_char(content: &str, idx: usize) -> Option<char> {
    content[..idx].chars().rev().find(|ch| !ch.is_whitespace())
}

fn find_jsonish_field_value_start(content: &str, needle: &str) -> Option<usize> {
    let field_idx = content.find(needle)?;
    let colon_idx = content[field_idx + needle.len()..].find(':')? + field_idx + needle.len();
    content[colon_idx + 1..]
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(colon_idx + 1 + idx))
}

fn jsonish_quote_closer(ch: char) -> Option<char> {
    match ch {
        '"' => Some('"'),
        '\'' => Some('\''),
        '“' => Some('”'),
        '‘' => Some('’'),
        _ => None,
    }
}

fn find_jsonish_field_value_quote_start(content: &str, needle: &str) -> Option<(usize, char)> {
    let value_idx = find_jsonish_field_value_start(content, needle)?;
    let ch = content[value_idx..].chars().next()?;
    let closer = jsonish_quote_closer(ch)?;
    Some((value_idx + ch.len_utf8(), closer))
}

fn json_field_name_after_comma(rest: &str) -> Option<&str> {
    let after_comma = rest.strip_prefix(',')?.trim_start();
    let after_quote = after_comma.strip_prefix('"')?;
    let end_quote = after_quote.find('"')?;
    if !after_quote[end_quote + 1..].trim_start().starts_with(':') {
        return None;
    }
    Some(&after_quote[..end_quote])
}

fn heartbeat_message_trailing_field(field: &str) -> bool {
    matches!(
        field,
        "source"
            | "sources"
            | "confidence"
            | "reason"
            | "timestamp"
            | "time"
            | "checked_at"
            | "check_time"
            | "symbol"
            | "ticker"
            | "price"
            | "severity"
            | "risk"
            | "trigger"
            | "metadata"
    )
}

fn looks_like_json_string_field_end(content: &str, quote_idx: usize, field: &str) -> bool {
    let rest = content[quote_idx + 1..].trim_start();
    if rest.is_empty() || rest.starts_with('}') {
        return true;
    }
    let Some(next_field) = json_field_name_after_comma(rest) else {
        return false;
    };
    field != "message" || heartbeat_message_trailing_field(next_field)
}

fn recover_lossy_json_string_field(content: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let (start, closer) = find_jsonish_field_value_quote_start(content, &needle)?;
    let mut value = String::new();
    let mut chars = content[start..].char_indices().peekable();

    while let Some((rel_idx, ch)) = chars.next() {
        let abs_idx = start + rel_idx;
        match ch {
            '\\' => {
                let Some((_, escaped)) = chars.next() else {
                    value.push(ch);
                    break;
                };
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    other => {
                        value.push('\\');
                        value.push(other);
                    }
                }
            }
            '"' if closer == '"' && looks_like_json_string_field_end(content, abs_idx, field) => {
                break;
            }
            '"' if closer == '"' => value.push('"'),
            ch if ch == closer => break,
            '}' if content[abs_idx + ch.len_utf8()..].trim().is_empty() => break,
            _ => value.push(ch),
        }
    }

    let value = value.trim().trim_end_matches(',').trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn recover_lossy_json_status_field(content: &str) -> Option<String> {
    if let Some(status) = recover_lossy_json_string_field(content, "status") {
        return Some(status);
    }

    let value_idx = find_jsonish_field_value_start(content, "\"status\"")?;
    let status = content[value_idx..]
        .trim_start()
        .split(|ch: char| ch == ',' || ch == '}' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .trim_matches(|ch| matches!(ch, '"' | '\'' | '“' | '”' | '‘' | '’'))
        .trim()
        .to_string();
    if status.is_empty() {
        None
    } else {
        Some(status)
    }
}

fn json_status_candidate_starts(content: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    for (idx, _) in content.match_indices('{') {
        let candidate = &content[idx..];
        let after_brace = candidate[1..].trim_start();
        if !after_brace.starts_with("\"status\"") {
            continue;
        }
        if previous_visible_char(content, idx) == Some('`') {
            continue;
        }
        starts.push(idx);
    }
    starts
}

fn recover_malformed_triggered_heartbeat_message(content: &str) -> Option<String> {
    let trimmed = content.trim();
    let candidate_starts = if trimmed.starts_with('{') {
        vec![0]
    } else {
        json_status_candidate_starts(trimmed)
    };

    for start in candidate_starts {
        let candidate = &trimmed[start..];
        let Some(status) = recover_lossy_json_status_field(candidate) else {
            continue;
        };
        if !status.eq_ignore_ascii_case("triggered") {
            continue;
        }

        let message = recover_lossy_json_string_field(candidate, "message")?;
        let message = unwrap_nested_json_message(message.trim())
            .trim()
            .to_string();
        if message.is_empty() || message == "..." || heartbeat_internal_marker_prefix(&message) {
            continue;
        }
        return Some(message);
    }
    None
}

fn heartbeat_plain_text_indicates_noop(text: &str) -> bool {
    let compact = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
        .to_ascii_lowercase();
    [
        "条件未满足",
        "条件不满足",
        "不满足触发",
        "尚未触发",
        "未触发",
        "不触发",
        "无需触发",
        "不需要发送",
        "本轮不发送",
        "输出{\"status\":\"noop\"}",
        "输出`{\"status\":\"noop\"}`",
        "returnnoop",
        "outputnoop",
        "shouldoutputnoop",
        "notmet",
        "nottriggered",
        "notrigger",
        "conditionisnotmet",
        "conditionsarenotmet",
    ]
    .iter()
    .any(|marker| compact.contains(marker))
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
            if let Some(s) = v.get(key).and_then(|v| v.as_str())
                && !s.is_empty()
            {
                return s.to_string();
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

fn heartbeat_price_timestamp_context_date(captures: &regex::Captures<'_>) -> Option<NaiveDate> {
    let year = captures
        .name("year_cn")
        .or_else(|| captures.name("year_iso"))
        .and_then(|m| m.as_str().parse::<i32>().ok())?;
    let month = captures
        .name("month_cn")
        .or_else(|| captures.name("month_iso"))
        .and_then(|m| m.as_str().parse::<u32>().ok())?;
    let day = captures
        .name("day_cn")
        .or_else(|| captures.name("day_iso"))
        .and_then(|m| m.as_str().parse::<u32>().ok())?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn heartbeat_triggered_price_threshold_context(text: &str) -> bool {
    let compact = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    [
        "阈值",
        "触发价",
        "触发线",
        "配置线",
        "警戒线",
        "跌破",
        "低于",
        "突破",
        "高于",
        "threshold",
        "triggerprice",
        "triggerline",
        "below",
        "above",
    ]
    .iter()
    .any(|term| compact.contains(term))
}

fn heartbeat_stale_price_timestamp(text: &str, reference_date: NaiveDate) -> Option<NaiveDate> {
    if !heartbeat_triggered_price_threshold_context(text) {
        return None;
    }
    RE_HEARTBEAT_PRICE_TIMESTAMP_DATE
        .captures_iter(text)
        .filter_map(|captures| heartbeat_price_timestamp_context_date(&captures))
        .find(|date| {
            if *date > reference_date {
                return true;
            }
            reference_date.signed_duration_since(*date).num_days()
                > HEARTBEAT_PRICE_TIMESTAMP_MAX_AGE_DAYS
        })
}

fn heartbeat_reference_now_beijing() -> DateTime<FixedOffset> {
    let beijing_tz = chrono::FixedOffset::east_opt(8 * 3600).expect("valid beijing offset");
    chrono::Utc::now().with_timezone(&beijing_tz)
}

fn heartbeat_check_time_text(now: DateTime<FixedOffset>) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute()
    )
}

fn heartbeat_current_check_time_text() -> String {
    heartbeat_check_time_text(heartbeat_reference_now_beijing())
}

fn normalize_heartbeat_beijing_trigger_time(
    text: &str,
    reference_now: DateTime<FixedOffset>,
) -> (String, Option<String>) {
    let reference_hour = reference_now.hour();
    let reference_minute = reference_now.minute();
    let mut normalized_from = None;
    let normalized = RE_HEARTBEAT_BEIJING_TRIGGER_TIME
        .replace_all(text, |captures: &regex::Captures<'_>| {
            let hour = captures
                .name("hour")
                .and_then(|matched| matched.as_str().parse::<u32>().ok());
            let minute = captures
                .name("minute")
                .and_then(|matched| matched.as_str().parse::<u32>().ok())
                .unwrap_or(0);
            let Some(hour) = hour else {
                return captures[0].to_string();
            };
            if hour >= 24 || minute >= 60 || (hour == reference_hour && minute == reference_minute)
            {
                return captures[0].to_string();
            }
            normalized_from.get_or_insert_with(|| format!("{hour:02}:{minute:02}"));
            let tail = captures
                .name("tail")
                .map(|m| m.as_str())
                .unwrap_or_default();
            format!("北京时间 {reference_hour:02}:{reference_minute:02}{tail}")
        })
        .into_owned();
    (normalized, normalized_from)
}

/// 通过 cloud-aware notification prefs 后端读 actor 的 quiet_hours + timezone。
/// 第二个返回值是 actor 的 timezone（IANA 名），用于 `quiet_window_active` 解释 from/to。
fn load_actor_quiet_hours(
    core: &HoneBotCore,
    actor: &hone_core::ActorIdentity,
) -> Option<(hone_core::quiet::QuietHours, Option<String>)> {
    hone_tools::load_notification_quiet_hours(&core.config.storage.notif_prefs_dir, actor)
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

fn heartbeat_entity_anchor_stop_token(token: &str) -> bool {
    matches!(
        token,
        "ai" | "api"
            | "app"
            | "aws"
            | "bedrock"
            | "ceo"
            | "cloud"
            | "current"
            | "daily"
            | "event"
            | "events"
            | "fda"
            | "fy"
            | "ipo"
            | "market"
            | "monitor"
            | "news"
            | "openai"
            | "price"
            | "q1"
            | "q2"
            | "q3"
            | "q4"
            | "report"
            | "research"
            | "sec"
            | "stock"
            | "the"
            | "update"
            | "watchlist"
    )
}

fn heartbeat_entity_anchor_tokens(text: &str) -> std::collections::BTreeSet<String> {
    let mut tokens = std::collections::BTreeSet::new();
    for matched in RE_HEARTBEAT_ENTITY_ANCHOR.find_iter(text) {
        let token = matched.as_str().to_ascii_lowercase();
        let normalized = token.trim_matches(|ch: char| ch == '.' || ch == '-');
        if normalized.chars().count() < 2 || heartbeat_entity_anchor_stop_token(normalized) {
            continue;
        }
        tokens.insert(normalized.to_string());
    }
    tokens
}

fn heartbeat_ticker_anchor_tokens(text: &str) -> std::collections::BTreeSet<String> {
    let mut tokens = std::collections::BTreeSet::new();
    for matched in RE_HEARTBEAT_ENTITY_ANCHOR.find_iter(text) {
        let raw = matched.as_str();
        let normalized = raw.trim_matches(|ch: char| ch == '.' || ch == '-');
        let lower = normalized.to_ascii_lowercase();
        if normalized.chars().count() < 2
            || normalized.chars().count() > 6
            || heartbeat_entity_anchor_stop_token(&lower)
            || !normalized
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '.' || ch == '-')
        {
            continue;
        }
        tokens.insert(lower);
    }
    tokens
}

fn heartbeat_entity_anchors_compatible(message: &str, preview: &str) -> bool {
    let message_tickers = heartbeat_ticker_anchor_tokens(message);
    let preview_tickers = heartbeat_ticker_anchor_tokens(preview);
    if !message_tickers.is_empty()
        && !preview_tickers.is_empty()
        && message_tickers
            .intersection(&preview_tickers)
            .next()
            .is_none()
    {
        return false;
    }

    let message_entities = heartbeat_entity_anchor_tokens(message);
    let preview_entities = heartbeat_entity_anchor_tokens(preview);
    message_entities.is_empty()
        || preview_entities.is_empty()
        || message_entities
            .intersection(&preview_entities)
            .next()
            .is_some()
}

fn heartbeat_same_ticker_reworded_fact_match(message: &str, preview: &str) -> bool {
    let message_tickers = heartbeat_ticker_anchor_tokens(message);
    let preview_tickers = heartbeat_ticker_anchor_tokens(preview);
    if message_tickers.is_empty()
        || preview_tickers.is_empty()
        || message_tickers
            .intersection(&preview_tickers)
            .next()
            .is_none()
    {
        return false;
    }

    let message_entities = heartbeat_entity_anchor_tokens(message)
        .into_iter()
        .filter(|token| !message_tickers.contains(token))
        .collect::<std::collections::BTreeSet<_>>();
    let preview_entities = heartbeat_entity_anchor_tokens(preview)
        .into_iter()
        .filter(|token| !preview_tickers.contains(token))
        .collect::<std::collections::BTreeSet<_>>();
    if message_entities.intersection(&preview_entities).count() >= 2 {
        return true;
    }

    let message_tokens = normalized_similarity_tokens(message);
    let preview_tokens = normalized_similarity_tokens(preview);
    message_tokens
        .intersection(&preview_tokens)
        .filter(|token| {
            token.contains('月')
                || token.contains("美元")
                || token.contains('%')
                || token.contains('.')
        })
        .count()
        >= 1
}

fn heartbeat_revision_sensitive_terms_present(text: &str) -> bool {
    let compact = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    [
        "定价区间",
        "发行价",
        "募资",
        "发行股",
        "发行股份",
        "估值",
        "上调",
        "下调",
        "pricingrange",
        "pricerange",
        "offeringprice",
        "offering",
        "valuation",
    ]
    .iter()
    .any(|term| compact.contains(term))
}

fn heartbeat_revision_fact_tokens(text: &str) -> std::collections::BTreeSet<String> {
    RE_HEARTBEAT_REVISION_FACT_TOKEN
        .find_iter(text)
        .map(|matched| {
            matched
                .as_str()
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .collect()
}

fn heartbeat_revision_facts_diverged(message: &str, preview: &str) -> bool {
    if !(heartbeat_revision_sensitive_terms_present(message)
        || heartbeat_revision_sensitive_terms_present(preview))
    {
        return false;
    }
    let message_facts = heartbeat_revision_fact_tokens(message);
    let preview_facts = heartbeat_revision_fact_tokens(preview);
    !message_facts.is_empty() && !preview_facts.is_empty() && message_facts != preview_facts
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
        let message_tickers = heartbeat_ticker_anchor_tokens(message);
        let preview_tickers = heartbeat_ticker_anchor_tokens(preview);
        let same_explicit_ticker = !message_tickers.is_empty()
            && !preview_tickers.is_empty()
            && message_tickers
                .intersection(&preview_tickers)
                .next()
                .is_some();
        if !heartbeat_entity_anchors_compatible(message, preview) {
            continue;
        }
        let preview_tokens = normalized_similarity_tokens(preview);
        if preview_tokens.len() < 4 {
            continue;
        }
        let shared = message_tokens.intersection(&preview_tokens).count();
        let smaller = message_tokens.len().min(preview_tokens.len());
        let strong_match = shared >= 4 && shared * 100 >= smaller * 70;
        let reworded_fact_match = shared >= 5
            && (!same_explicit_ticker
                || heartbeat_same_ticker_reworded_fact_match(message, preview));
        if (strong_match || reworded_fact_match)
            && heartbeat_revision_facts_diverged(message, preview)
        {
            continue;
        }
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
        if heartbeat_plain_text_indicates_noop(content) {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::PlainTextNoop);
        }
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
        if heartbeat_status_indicates_noop(&status) {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonNoop);
        }
        if status.is_empty() {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonEmptyStatus);
        }
        if heartbeat_status_indicates_triggered(&status) {
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

    if let Some(message) = recover_malformed_triggered_heartbeat_message(trimmed) {
        return (
            HeartbeatOutcome::Deliver(message),
            HeartbeatParseKind::JsonTriggered,
        );
    }

    if trimmed.starts_with('{') {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonMalformed);
    }

    if heartbeat_plain_text_indicates_noop(trimmed) {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::PlainTextNoop);
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
        HeartbeatParseKind::JsonEmptyStatus => None,
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
    heartbeat_execution_from_content_at_beijing(
        content,
        heartbeat_model,
        heartbeat_reference_now_beijing(),
    )
}

#[cfg(test)]
fn heartbeat_execution_from_content_at(
    content: &str,
    heartbeat_model: &str,
    reference_date: NaiveDate,
) -> ScheduledTaskExecution {
    heartbeat_execution_from_content_internal(content, heartbeat_model, reference_date, None)
}

fn heartbeat_execution_from_content_at_beijing(
    content: &str,
    heartbeat_model: &str,
    reference_now: DateTime<FixedOffset>,
) -> ScheduledTaskExecution {
    heartbeat_execution_from_content_internal(
        content,
        heartbeat_model,
        reference_now.date_naive(),
        Some(reference_now),
    )
}

fn heartbeat_execution_from_content_internal(
    content: &str,
    heartbeat_model: &str,
    reference_date: NaiveDate,
    reference_now: Option<DateTime<FixedOffset>>,
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
            let mut sanitized_message = sanitize_scheduler_delivery_text(&message);
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
            let normalized_beijing_trigger_time = reference_now.and_then(|reference_now| {
                let (normalized, normalized_from) =
                    normalize_heartbeat_beijing_trigger_time(&sanitized_message, reference_now);
                sanitized_message = normalized;
                normalized_from
            });
            let deliver_preview = truncate_for_log(sanitized_message.trim(), 200);
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
            if let Some(stale_price_timestamp) =
                heartbeat_stale_price_timestamp(&sanitized_message, reference_date)
            {
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
                        "failure_kind": "stale_price_timestamp",
                        "stale_price_timestamp": stale_price_timestamp.to_string(),
                        "stale_price_timestamp_suppressed": true,
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
                    "beijing_trigger_time_normalized": normalized_beijing_trigger_time.is_some(),
                    "original_beijing_trigger_time": normalized_beijing_trigger_time,
                }),
                session_id: None,
            }
        }
    }
}

fn scheduler_event_is_commodity_related(event: &SchedulerEvent) -> bool {
    let job_name = event.job_name.to_ascii_lowercase();
    let prompt = event.task_prompt.to_ascii_lowercase();
    let job_is_commodity_focused = event.job_name.contains("原油")
        || event.job_name.contains("油价")
        || event.job_name.contains("布伦特")
        || event.job_name.contains("大宗商品")
        || job_name.contains("crude")
        || job_name.contains("wti")
        || job_name.contains("brent")
        || job_name.contains("oil_price")
        || job_name.contains("oil price");
    if job_is_commodity_focused {
        return true;
    }

    let prompt_is_commodity_focused = event.task_prompt.contains("原油价格")
        || event.task_prompt.contains("油价播报")
        || event.task_prompt.contains("大宗商品播报")
        || event.task_prompt.contains("播报 WTI")
        || event.task_prompt.contains("播报WTI")
        || event.task_prompt.contains("汇总 WTI")
        || event.task_prompt.contains("汇总WTI")
        || prompt.contains("crude oil")
        || prompt.contains("oil price monitor")
        || prompt.contains("wti/brent")
        || prompt.contains("wti / brent");
    prompt_is_commodity_focused && !scheduler_event_is_broad_market_review(event)
}

fn scheduler_event_is_broad_market_review(event: &SchedulerEvent) -> bool {
    let haystack = compact_lowercase_text(&format!("{} {}", event.job_name, event.task_prompt));
    [
        "复盘",
        "简报",
        "风控",
        "温度",
        "收盘",
        "盘后",
        "盘前",
        "大盘",
        "市场",
        "指数",
        "情绪",
        "早报",
        "降息",
        "概率",
        "宏观",
        "briefing",
        "morningbriefing",
        "marketreview",
        "postmarket",
        "premarket",
        "riskbrief",
    ]
    .iter()
    .any(|term| haystack.contains(term))
}

fn compact_lowercase_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn count_distinct_keyword_hits(compact: &str, keywords: &[&str]) -> usize {
    keywords
        .iter()
        .filter(|term| compact.contains(**term))
        .count()
}

fn text_has_commodity_causality_claim(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    if compact.is_empty() {
        return false;
    }

    let causality_terms = [
        "主因",
        "原因",
        "归因",
        "主要受",
        "受",
        "驱动",
        "导致",
        "支撑",
        "承压",
        "影响",
        "背景",
        "因素",
        "解释",
        "推高",
        "推升",
        "拉动",
        "压制",
        "压力",
        "缓和",
        "修复",
        "担忧",
        "风险",
        "风险溢价",
        "中断",
        "关停",
        "升级",
        "because",
        "dueto",
        "drivenby",
        "causedby",
        "pushhigher",
        "riskpremium",
    ];
    let high_risk_terms = [
        "地缘",
        "宏观",
        "供应",
        "需求",
        "库存",
        "航运",
        "能源",
        "通胀",
        "油价",
        "谈判",
        "军事",
        "战争",
        "冲突",
        "紧张",
        "关税",
        "风险溢价",
        "供应中断",
        "关停风险",
        "战略储备",
        "石油储备",
        "沙特",
        "阿美",
        "警告",
        "中东",
        "伊朗",
        "霍尔木兹",
        "美伊",
        "opec",
        "geopolitical",
        "supply",
        "demand",
        "inventory",
        "shipping",
        "sanction",
        "tariff",
        "riskpremium",
        "supplydisruption",
    ];

    causality_terms.iter().any(|term| compact.contains(term))
        && high_risk_terms.iter().any(|term| compact.contains(term))
}

fn split_commodity_message_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '\n' | '。' | '；' | ';' | '!' | '！' | '?' | '？') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                segments.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }
    segments
}

fn text_has_speculative_commodity_price(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    let approximate_commodity_quote = compact.contains("约")
        && text_looks_commodity_related(text)
        && text.chars().any(|ch| ch.is_ascii_digit())
        && ["$", "美元", "桶"].iter().any(|term| text.contains(term));
    approximate_commodity_quote
        || [
            "估算",
            "约$",
            "约为$",
            "约每桶",
            "约人民币",
            "推算",
            "预测",
            "预测区间",
            "通常较",
            "贴水",
            "未独立校验",
            "未完成同窗",
            "精确收盘价未",
            "无法证明",
            "未核验",
            "assume",
            "estimated",
            "forecast",
        ]
        .iter()
        .any(|term| compact.contains(term))
}

fn chinese_weekday_number(label: &str) -> Option<u32> {
    match label {
        "一" | "1" => Some(1),
        "二" | "2" => Some(2),
        "三" | "3" => Some(3),
        "四" | "4" => Some(4),
        "五" | "5" => Some(5),
        "六" | "6" => Some(6),
        "日" | "天" | "7" | "0" => Some(7),
        _ => None,
    }
}

fn text_has_date_weekday_mismatch(text: &str) -> bool {
    static RE_DATE_WEEKDAY: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r"(?x)
            (?P<year>20\d{2})[-年/](?P<month>\d{1,2})[-月/](?P<day>\d{1,2})日?
            [^\n。；;，,]{0,12}
            (?:周|星期)(?P<weekday>[一二三四五六日天0-7])
            ",
        )
        .expect("valid date weekday regex")
    });

    for captures in RE_DATE_WEEKDAY.captures_iter(text) {
        let year = captures
            .name("year")
            .and_then(|m| m.as_str().parse::<i32>().ok());
        let month = captures
            .name("month")
            .and_then(|m| m.as_str().parse::<u32>().ok());
        let day = captures
            .name("day")
            .and_then(|m| m.as_str().parse::<u32>().ok());
        let weekday = captures
            .name("weekday")
            .and_then(|m| chinese_weekday_number(m.as_str()));
        let (Some(year), Some(month), Some(day), Some(weekday)) = (year, month, day, weekday)
        else {
            continue;
        };
        let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) else {
            continue;
        };
        if date.weekday().number_from_monday() != weekday {
            return true;
        }
    }
    false
}

fn text_has_unverified_commodity_market_claim(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    text_has_speculative_commodity_price(text)
        || text_has_date_weekday_mismatch(text)
        || [
            "近一个月",
            "累计上涨",
            "累计下跌",
            "bloomberg",
            "reuters",
            "wsj",
            "eia",
            "截至",
            "5月8日",
            "5月9日",
        ]
        .iter()
        .any(|term| compact.contains(term))
}

fn text_looks_like_commodity_price_observation(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    let mentions_commodity = ["原油", "油价", "布伦特", "wti", "brent", "crude", "oil"]
        .iter()
        .any(|term| compact.contains(term));
    let has_numeric_quote = text.chars().any(|ch| ch.is_ascii_digit())
        && ["$", "美元", "桶"].iter().any(|term| text.contains(term));

    mentions_commodity
        && has_numeric_quote
        && !text_has_commodity_causality_claim(text)
        && !text_has_speculative_commodity_price(text)
        && !text_has_date_weekday_mismatch(text)
        && ![
            "未完成同窗",
            "未核验",
            "未校验",
            "未验证",
            "无法证明",
            "bloomberg",
            "reuters",
            "wsj",
            "eia",
            "近一个月",
            "累计上涨",
            "累计下跌",
        ]
        .iter()
        .any(|term| compact.contains(term))
}

fn text_has_explicit_grounded_commodity_source(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    [
        "本轮工具",
        "本轮检索",
        "本轮来源",
        "同窗来源",
        "同窗核验",
        "已核验",
        "已校验",
        "交易所",
        "官方报价",
        "official",
        "exchange",
        "verified",
    ]
    .iter()
    .any(|term| compact.contains(term))
}

fn text_has_broad_market_review_context(text: &str) -> bool {
    broad_market_review_anchor_hits(text) >= 2
}

fn broad_market_review_anchor_hits(text: &str) -> usize {
    let compact = compact_lowercase_text(text);
    count_distinct_keyword_hits(
        &compact,
        &[
            "a股",
            "港股",
            "美股",
            "大盘",
            "纳指",
            "nasdaq",
            "qqq",
            "标普",
            "s&p",
            "sp500",
            "道指",
            "dow",
            "vix",
            "fear&greed",
            "feargreed",
            "恒生",
            "hsi",
            "上证",
            "深成指",
            "创业板",
            "科技股",
            "半导体",
            "ai",
            "硬件",
            "etf",
            "xme",
            "加密",
            "降息",
            "fomc",
            "fedwatch",
            "pce",
            "利率",
            "宏观",
            "国债收益率",
            "10年期美债",
            "风险偏好",
            "风控",
            "温度",
            "休市",
            "交易日",
            "情绪",
            "贪婪",
            "greed",
            "追涨",
            "赔率",
            "高位",
            "低波动",
            "偏热",
            "盈利兑现",
        ],
    )
}

fn rewrite_commodity_causality_message(text: &str) -> String {
    let mut retained_segments = Vec::new();
    for segment in split_commodity_message_segments(text) {
        if text_looks_like_commodity_price_observation(&segment)
            && text_has_explicit_grounded_commodity_source(&segment)
            && !retained_segments
                .iter()
                .any(|existing| existing == &segment)
        {
            retained_segments.push(segment);
        }
        if retained_segments.len() >= 4 {
            break;
        }
    }

    let mut rewritten = "【归因口径】本轮原油/大宗商品播报包含未完成同窗来源核验的原因归因，已移除原正文中的宏观、地缘、供需、库存等主因叙述；不能视为已确认油价主因。"
        .to_string();
    if retained_segments.is_empty() {
        rewritten.push_str(
            "\n本轮未保留原正文中的价格或归因句；请等待下一轮核验或手动查询交易所/官方数据。",
        );
    } else {
        rewritten.push_str("\n【已保留的价格口径】");
        for segment in retained_segments {
            rewritten.push_str("\n- ");
            rewritten.push_str(segment.trim());
        }
    }
    rewritten
}

fn guard_commodity_causality_for_event(text: &str, event: &SchedulerEvent) -> Option<String> {
    let has_unsafe_commodity_claim = text_has_commodity_causality_claim(text)
        || text_has_unverified_commodity_market_claim(text);
    if !has_unsafe_commodity_claim {
        return None;
    }
    let event_is_commodity_related = scheduler_event_is_commodity_related(event);
    let text_is_predominantly_commodity_related = text_is_predominantly_commodity_related(text);
    if !event_is_commodity_related && !text_is_predominantly_commodity_related {
        return None;
    }
    let rewritten = rewrite_commodity_causality_message(text);
    if rewritten.trim() == text.trim() {
        None
    } else {
        Some(rewritten)
    }
}

fn text_looks_commodity_related(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    ["原油", "油价", "布伦特", "wti", "brent", "crude", "oil"]
        .iter()
        .any(|term| compact.contains(term))
}

fn commodity_keyword_hits(text: &str) -> usize {
    let compact = compact_lowercase_text(text);
    count_distinct_keyword_hits(
        &compact,
        &[
            "原油",
            "油价",
            "布伦特",
            "wti",
            "brent",
            "crude",
            "oil",
            "uso",
        ],
    )
}

fn text_is_predominantly_commodity_related(text: &str) -> bool {
    if !text_looks_commodity_related(text) {
        return false;
    }

    let commodity_hits = commodity_keyword_hits(text);
    let broad_market_hits = broad_market_review_anchor_hits(text);
    if broad_market_hits >= 4 && broad_market_hits >= commodity_hits + 2 {
        return false;
    }

    let segments = split_commodity_message_segments(text);
    let meaningful_segments: Vec<&str> = segments
        .iter()
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .collect();
    if meaningful_segments.is_empty() {
        return true;
    }
    if meaningful_segments.len() <= 2 {
        if broad_market_hits >= 3 {
            return commodity_hits >= 4 && commodity_hits > broad_market_hits;
        }
        return commodity_hits >= 3 || !text_has_broad_market_review_context(text);
    }

    let total_chars: usize = meaningful_segments
        .iter()
        .map(|segment| segment.chars().count())
        .sum();
    if total_chars == 0 {
        return false;
    }

    let commodity_segments = meaningful_segments
        .iter()
        .filter(|segment| text_looks_commodity_related(segment))
        .count();
    let commodity_chars: usize = meaningful_segments
        .iter()
        .filter(|segment| text_looks_commodity_related(segment))
        .map(|segment| segment.chars().count())
        .sum();

    if text_has_broad_market_review_context(text) {
        return commodity_segments * 3 >= meaningful_segments.len() * 2
            && commodity_chars * 3 >= total_chars * 2;
    }

    commodity_segments * 2 >= meaningful_segments.len() || commodity_chars * 2 >= total_chars
}

fn scheduler_metadata_with_commodity_guard(original: &str, guarded: &str) -> Value {
    json!({
        "commodity_causality_guarded": true,
        "raw_preview": truncate_for_log(original.trim(), 280),
        "guarded_preview": truncate_for_log(guarded.trim(), 200),
        "deliver_preview": truncate_for_log(guarded.trim(), 200),
    })
}

fn text_has_direct_trade_instruction(text: &str) -> bool {
    let compact = compact_lowercase_text(text);
    let has_direct_phrase = [
        "无条件止损",
        "必须止损",
        "必须卖出",
        "必须清仓",
        "立即止损",
        "立即卖出",
        "立即清仓",
        "马上止损",
        "马上卖出",
        "马上清仓",
        "立即买入",
        "马上买入",
        "全仓买入",
        "无条件买入",
        "无条件卖出",
        "mustsell",
        "sellimmediately",
        "liquidateimmediately",
        "buyimmediately",
        "stoplossimmediately",
    ]
    .iter()
    .any(|term| compact.contains(term));
    if has_direct_phrase {
        return true;
    }

    (compact.contains("建议动作") || compact.contains("操作建议"))
        && ["止损", "清仓", "卖出", "买入", "抄底", "持有等待反弹"]
            .iter()
            .any(|term| compact.contains(term))
}

fn update_heartbeat_delivery_preview_metadata(metadata: &mut Value, content: &str) {
    if let Value::Object(map) = metadata {
        map.insert(
            "deliver_preview".to_string(),
            Value::String(truncate_for_log(content.trim(), 200)),
        );
    }
}

fn rewrite_direct_trade_instruction_message(text: &str) -> String {
    let retained_segments = split_commodity_message_segments(text)
        .into_iter()
        .filter(|segment| !text_has_direct_trade_instruction(segment))
        .take(6)
        .collect::<Vec<_>>();

    let mut rewritten = "【风险提示】本轮自动预警只确认触发条件与风险事实，不构成买卖、止损、加仓或清仓指令。若你原本以该阈值作为风控线，请结合仓位、成本、流动性和风险承受能力复核；需要动作时应按你预先设定的条件执行。"
        .to_string();
    if !retained_segments.is_empty() {
        rewritten.push_str("\n【触发事实】");
        for segment in retained_segments {
            rewritten.push_str("\n- ");
            rewritten.push_str(segment.trim());
        }
    }
    rewritten
}

fn guard_direct_trade_instruction_for_event(text: &str, event: &SchedulerEvent) -> Option<String> {
    if !event.heartbeat || !text_has_direct_trade_instruction(text) {
        return None;
    }

    let rewritten = rewrite_direct_trade_instruction_message(text);
    if rewritten.trim() == text.trim() {
        None
    } else {
        Some(rewritten)
    }
}

fn heartbeat_runner_failure_kind(error: &str) -> &'static str {
    if is_context_overflow_error(error) {
        return "context_window_overflow";
    }
    let lower = error.to_ascii_lowercase();
    if lower.contains("upstream http 402")
        || lower.contains("upstream http 429")
        || lower.contains("http 402")
        || lower.contains("http 429")
        || lower.contains("code: 402")
        || lower.contains("code: 429")
        || lower.contains("requires more credits")
        || lower.contains("insufficient credit")
        || lower.contains("insufficient balance")
        || lower.contains("quota exceeded")
        || lower.contains("rate limit exceeded")
        || lower.contains("too many requests")
        || lower.contains("resource exhausted")
    {
        return "provider_quota_exhausted";
    }
    if lower.contains("upstream http ")
        || lower.contains("http 4")
        || lower.contains("http 5")
        || lower.contains("status: 4")
        || lower.contains("status: 5")
    {
        return "provider_http_error";
    }
    "runner_error"
}

fn heartbeat_execution_from_runner_error(
    error: String,
    heartbeat_model: &str,
) -> ScheduledTaskExecution {
    let failure_kind = heartbeat_runner_failure_kind(&error);
    let mut metadata = json!({
        "heartbeat_model": heartbeat_model,
        "failure_kind": failure_kind,
    });
    if is_context_overflow_error(&error)
        && let Value::Object(map) = &mut metadata
    {
        map.insert(
            "parse_kind".to_string(),
            Value::String("ContextOverflowError".to_string()),
        );
    }
    ScheduledTaskExecution {
        should_deliver: false,
        content: String::new(),
        error: Some(error),
        metadata,
        session_id: None,
    }
}

fn scheduler_suppressed_failure_kind(raw_error: Option<&str>) -> &'static str {
    let Some(error) = raw_error else {
        return "internal_error_suppressed";
    };
    let lower = error.to_ascii_lowercase();
    if lower.contains("stream disconnected before completion")
        || lower.contains("stream closed before response")
        || lower.contains("acp stream disconnected")
        || lower.contains("transport disconnected")
    {
        return "acp_transport_disconnect";
    }
    if lower.contains("timeout") || lower.contains("timed out") {
        return "scheduler_runner_timeout";
    }
    "internal_error_suppressed"
}

pub fn scheduled_task_failure_kind(execution: &ScheduledTaskExecution) -> Option<&str> {
    execution
        .metadata
        .get("failure_kind")
        .and_then(|value| value.as_str())
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

fn is_stale_market_data_success_fallback(text: &str) -> bool {
    let normalized = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    if [
        "不复用旧价格",
        "不使用旧价格",
        "不沿用旧价格",
        "已跳过旧价格",
        "跳过旧价格版本",
    ]
    .iter()
    .any(|term| normalized.contains(term))
    {
        return false;
    }

    let market_data_failed = [
        "底层行情数据链路暂时阻断",
        "行情数据链路暂时阻断",
        "行情链路暂时阻断",
        "数据链路暂时阻断",
        "报价接口触及限额",
        "行情数据获取失败",
        "实时行情获取失败",
        "拉取持仓实时行情时",
        "data_fetch失败",
        "data_fetchfailed",
    ]
    .iter()
    .any(|term| normalized.contains(&term.to_ascii_lowercase()));

    let stale_price_reused = [
        "使用本会话此前已核验",
        "采用同一会话",
        "采用此前",
        "沿用此前",
        "先前已核验",
        "此前已核验",
        "旧价格",
        "旧价",
        "上一交易日收盘",
        "收盘口径",
    ]
    .iter()
    .any(|term| normalized.contains(&term.to_ascii_lowercase()));

    market_data_failed && stale_price_reused
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
    let normalized = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let patterns = [
        "按规则应跳过正式推送",
        "按规则可跳过正式推送",
        "按规则可跳过",
        "无新增催化，跳过推送",
        "无新增催化,跳过推送",
        "不触发重大催化或风险证伪推送",
        "不触发新增重大催化或风险证伪推送",
        "不触发新增重大推送",
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
    patterns
        .iter()
        .any(|pat| text.contains(pat) || normalized.contains(pat))
}

fn scheduled_prompt_needs_stable_local_context(event: &SchedulerEvent) -> bool {
    let haystack = format!("{} {}", event.job_name, event.task_prompt).to_ascii_lowercase();
    let has_hit_zone = event.job_name.contains("击球区")
        || event.task_prompt.contains("击球区")
        || haystack.contains("hit zone")
        || haystack.contains("hit-zone");
    let has_watch_pool = event.job_name.contains("观察池")
        || event.job_name.contains("观察股池")
        || event.task_prompt.contains("观察池")
        || event.task_prompt.contains("观察股池")
        || haystack.contains("watchlist")
        || haystack.contains("watch pool");
    has_hit_zone && has_watch_pool
}

fn extract_watchlist_tickers(task_prompt: &str) -> Vec<String> {
    let mut tickers = Vec::new();
    for matched in regex::Regex::new(r"\b[A-Z]{2,5}\b")
        .expect("valid watchlist ticker regex")
        .find_iter(task_prompt)
    {
        let ticker = matched.as_str();
        if !tickers.iter().any(|existing| existing == ticker) {
            tickers.push(ticker.to_string());
        }
    }
    tickers
}

fn normalize_recovered_hit_zone(zone: &str) -> Option<String> {
    let zone = zone
        .trim()
        .trim_matches(|ch: char| matches!(ch, '|' | ',' | '，' | '。' | ';' | '；'))
        .trim_start_matches("击球区：")
        .trim_start_matches("击球区:")
        .trim();
    if zone.is_empty() || zone.contains("待确认") || !zone.contains('$') {
        return None;
    }
    let looks_like_zone = zone.contains('-')
        || zone.contains('–')
        || zone.contains('/')
        || zone.contains("保守")
        || zone.contains("合理")
        || zone.contains("激进");
    if !looks_like_zone || zone.chars().count() > 120 {
        return None;
    }
    Some(zone.to_string())
}

fn extract_ticker_hit_zone_from_source(source: &str, ticker: &str) -> Option<String> {
    let table_pattern = format!(
        r"(?m)^\|\s*{}\s*\|\s*[^|\n]*\|\s*[^|\n]*\|\s*(?P<zone>[^|\n]+?)\s*\|",
        regex::escape(ticker)
    );
    if let Some(zone) = regex::Regex::new(&table_pattern)
        .ok()
        .and_then(|re| re.captures(source))
        .and_then(|caps| caps.name("zone"))
        .and_then(|zone| normalize_recovered_hit_zone(zone.as_str()))
    {
        return Some(zone);
    }

    let inline_pattern = format!(
        r"(?m)\b{}\b[^\n]{{0,80}}?击球区[:：]?\s*(?P<zone>[^\n]+)",
        regex::escape(ticker)
    );
    regex::Regex::new(&inline_pattern)
        .ok()
        .and_then(|re| re.captures(source))
        .and_then(|caps| caps.name("zone"))
        .and_then(|zone| normalize_recovered_hit_zone(zone.as_str()))
        .or_else(|| {
            source.lines().find_map(|line| {
                extract_compact_line_ticker_hit_zone(line, Some(ticker)).map(|(_, zone)| zone)
            })
        })
}

fn push_recovered_hit_zone(recovered: &mut Vec<(String, String)>, ticker: &str, zone: &str) {
    if !recovered
        .iter()
        .any(|(existing_ticker, _)| existing_ticker == ticker)
    {
        recovered.push((ticker.to_string(), zone.to_string()));
    }
}

fn extract_compact_line_ticker_hit_zone(
    line: &str,
    expected_ticker: Option<&str>,
) -> Option<(String, String)> {
    let ticker_re = regex::Regex::new(r"\b[A-Z]{2,5}\b").expect("valid watchlist ticker regex");
    let ticker = match expected_ticker {
        Some(ticker) if ticker_re.is_match(ticker) && line.contains(ticker) => ticker.to_string(),
        Some(_) => return None,
        None => ticker_re.find(line)?.as_str().to_string(),
    };
    let ticker_pos = line.find(&ticker)?;
    let tail = &line[ticker_pos + ticker.len()..];
    let compact_zone_re = regex::Regex::new(
        r"(?P<zone>(?:(?:保守|合理|激进观察|激进|观察)\s*)?\$[\d,.]+\s*[-–—~至]\s*\$?[\d,.]+(?:\s*/\s*(?:(?:保守|合理|激进观察|激进|观察)\s*)?\$[\d,.]+\s*[-–—~至]\s*\$?[\d,.]+)*)",
    )
    .expect("valid compact watchlist hit-zone regex");
    compact_zone_re
        .captures(tail)
        .and_then(|caps| caps.name("zone"))
        .and_then(|zone| normalize_recovered_hit_zone(zone.as_str()))
        .map(|zone| (ticker, zone))
}

fn extract_all_ticker_hit_zones_from_source(source: &str) -> Vec<(String, String)> {
    let mut recovered = Vec::new();

    let table_re = regex::Regex::new(
        r"(?m)^\|\s*(?P<ticker>[A-Z]{2,5})\s*\|\s*[^|\n]*\|\s*[^|\n]*\|\s*(?P<zone>[^|\n]+?)\s*\|",
    )
    .expect("valid watchlist hit-zone table regex");
    for caps in table_re.captures_iter(source) {
        let Some(ticker) = caps.name("ticker").map(|matched| matched.as_str()) else {
            continue;
        };
        let Some(zone) = caps
            .name("zone")
            .and_then(|matched| normalize_recovered_hit_zone(matched.as_str()))
        else {
            continue;
        };
        push_recovered_hit_zone(&mut recovered, ticker, &zone);
    }

    let inline_re = regex::Regex::new(
        r"(?m)\b(?P<ticker>[A-Z]{2,5})\b[^\n]{0,80}?击球区[:：]?\s*(?P<zone>[^\n]+)",
    )
    .expect("valid watchlist inline hit-zone regex");
    for caps in inline_re.captures_iter(source) {
        let Some(ticker) = caps.name("ticker").map(|matched| matched.as_str()) else {
            continue;
        };
        let Some(zone) = caps
            .name("zone")
            .and_then(|matched| normalize_recovered_hit_zone(matched.as_str()))
        else {
            continue;
        };
        push_recovered_hit_zone(&mut recovered, ticker, &zone);
    }

    let ticker_re = regex::Regex::new(r"\b[A-Z]{2,5}\b").expect("valid watchlist ticker regex");
    for line in source.lines() {
        for matched in ticker_re.find_iter(line) {
            let ticker = matched.as_str();
            if let Some((ticker, zone)) = extract_compact_line_ticker_hit_zone(line, Some(ticker)) {
                push_recovered_hit_zone(&mut recovered, &ticker, &zone);
            }
        }
    }

    recovered
}

fn recover_watchlist_hit_zone_context(core: &HoneBotCore, event: &SchedulerEvent) -> Vec<String> {
    if !scheduled_prompt_needs_stable_local_context(event) {
        return Vec::new();
    }
    let tickers = extract_watchlist_tickers(&event.task_prompt);
    let session_id = event.actor.session_id();
    let Some(session) = core
        .session_storage
        .load_session(&session_id)
        .ok()
        .flatten()
    else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    if let Some(message) = hone_memory::latest_compact_summary(&session.messages) {
        let text = hone_memory::session_message_text(message);
        if !text.trim().is_empty() {
            sources.push(text);
        }
    }
    if let Some(summary) = session.summary
        && !summary.content.trim().is_empty()
    {
        sources.push(summary.content);
    }

    let mut recovered = Vec::new();
    if tickers.is_empty() {
        for source in &sources {
            for (ticker, zone) in extract_all_ticker_hit_zones_from_source(source) {
                push_recovered_hit_zone(&mut recovered, &ticker, &zone);
            }
        }
    } else {
        for ticker in tickers {
            if let Some(zone) = sources
                .iter()
                .find_map(|source| extract_ticker_hit_zone_from_source(source, &ticker))
            {
                push_recovered_hit_zone(&mut recovered, &ticker, &zone);
            }
        }
    }
    recovered
        .into_iter()
        .map(|(ticker, zone)| format!("- {ticker}: {zone}"))
        .collect()
}

fn build_scheduled_prompt_with_recovered_local_context(
    core: &HoneBotCore,
    event: &SchedulerEvent,
) -> String {
    let prompt = build_scheduled_prompt(event);
    let recovered = recover_watchlist_hit_zone_context(core, event);
    if recovered.is_empty() {
        return prompt;
    }
    format!(
        "{prompt}\n\n【已恢复的本地击球区参考】\n以下区间来自当前会话已保存的 compact summary / 本地观察池上下文；除非本轮任务正文显式改动，否则沿用这些区间，不要因为 `data_fetch` 未返回击球区字段而删除或改写为“待确认”。\n{}",
        recovered.join("\n")
    )
}

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
    if event.heartbeat {
        let check_time = heartbeat_current_check_time_text();
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
本轮权威检查时间（北京时间）：{}。\n\
请使用可用工具检查用户设置的触发条件是否已经满足。\n\
\n\
规则：\n\
1. 如果条件尚未满足，优先只输出 `{{\"status\":\"noop\"}}`；为兼容旧行为，也允许只输出 `{{}}`。\n\
2. 如果条件已满足，只输出一段 JSON：`{{\"status\":\"triggered\",\"message\":\"...\"}}`。\n\
3. `message` 必须是一条可以直接发给用户的提醒消息，包含：满足的条件、关键数据、检查时间；检查时间必须使用上方“本轮权威检查时间（北京时间）”，不得自行换算或推断另一个北京时间。\n\
4. 不要创建新的定时任务，也不要修改现有任务。\n\
5. 不要输出 Markdown 代码块，不要输出额外解释，不要暴露任何内部控制标记。\n\
6. 如果你不确定是否满足条件，或者输出格式不是严格 JSON，就必须返回 noop，不允许发送自由文本。\n\
6a. 输出契约：整条回复必须是单段 JSON，第一个可见字符必须是 `{{`。严禁使用 `<think>...</think>`、```json ... ```、`## 分析`、分步解释或任何前置/收尾的自由文本。推理过程不要对外展示；需要推理时在内部完成后，直接给出最终 JSON。\n\
6b. 如果你发现用户条件、交易动作边界、来源归因或输出契约之间存在冲突，不要解释冲突、不要复述规则、不要输出空文本；必须返回 `{{\"status\":\"noop\"}}`，除非你能用合规的 `triggered` JSON 只报告触发事实和条件化风险提示。\n\
6c. 严禁输出工具配置、任务配置、画像建档说明、`set_immediate_kinds`、`cron_job` 或任何“已配置/将创建监控”的说明；如果本轮误入配置/建档/任务治理路径，必须返回 `{{\"status\":\"noop\"}}`。\n\
7. 时间一致性约束：对于发射、财报、业绩会等有明确时间窗口的事件，必须先判断当前时间是否已越过事件预定时间，才能输出完成态结论。若当前时间早于事件计划时间，必须返回 noop，不允许把未来计划误报成已完成。\n\
7a. 时间口径命名约束：`message` 中写“北京时间 HH:MM 触发/监控触发/检查触发”时，只能使用上方权威检查时间；市场时段、数据时间或美东盘前/盘后只能标注为“数据时间”“美东时间”“交易时段”，不能写成另一个“北京时间触发”。\n\
8. 价格时间口径约束：引用股价、金价、汇率或商品价格时，必须核实价格的时间戳。价格阈值 / 跌破 / 突破类 heartbeat 只有在最新可得价格属于当前检查窗口或最近可解释交易窗口时才能 triggered；若工具只返回明显旧日期、缺少价格时间戳，或无法证明该价格仍是最新可得价格，必须返回 noop，不允许把旧价格包装成当前触发依据。\n\
9. 价格阈值口径约束：除非用户条件里明确写的是“日内最高/最低/振幅/区间波动”，否则“盘中涨跌幅超过 X%”一律按最新可得价格相对昨收的涨跌幅判断；不允许用日内高点相对昨收、日内低点相对昨收，或高低点振幅去替代当前涨跌幅。\n\
10. 若最新可得价格相对昨收尚未达到阈值，但日内高点、日内低点或盘中振幅达到阈值，且任务没有明确要求这些口径，本轮必须返回 noop，不允许触发。\n\
11. 重复事件约束：若某条件（如某只股票的某次发射或某次事件）已经在前一轮被判定为 noop 或 triggered，本轮如果没有获取到新的独立行情时间戳或新的独立事件窗口，就不允许改变结论，也不允许重复 triggered。\n\
12. 来源归因约束：引用 Reuters、WSJ、Bloomberg、官方公告等来源时，必须确认本轮工具结果明确出现该来源与对应事实；没有明确来源时，只能写“未核验/市场传闻/需继续确认”，不得把地缘政治、谈判、航运限制等叙述写成已被权威媒体共同确认的事实。\n\
13. 交易动作边界：预警只能报告触发事实、价格/成交量/时间口径和条件化风险管理框架，不得输出“无条件止损”“必须卖出”“立即清仓”“马上买入”等直接交易指令；涉及买卖、止损、加仓、减仓时必须明确这是分析参考，并要求用户结合仓位、成本、流动性和风险承受能力复核。\n\
14. 工具预算约束：必须以最少工具调用收口。优先复用本轮已经拿到的价格、新闻、组合和文件信息；若需要逐标的穷举或反复重复同一查询才能确认，本轮只检查最可能触发的少数候选并尽快返回 noop 或 triggered，禁止为了展示分析过程反复调用相同工具。\n\
{}\
\n以下是需要检查的用户条件：\n{}",
            event.job_name, check_time, history_section, event.task_prompt
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
    let stable_context_note = if scheduled_prompt_needs_stable_local_context(event) {
        "\n\n稳定本地字段约束：本任务里的观察池、击球区、策略纪律等固定配置属于用户本地状态，不属于 `data_fetch` 行情结果。涉及击球区时，先使用任务正文、已恢复会话上下文、portfolio/local state 或本地文件中的既有区间；`data_fetch` 只校验最新价格和财报日期。不要因为行情工具没有返回击球区字段，就把已经存在于上下文或本地状态里的区间统一降级为“待确认”。只有本轮任务正文和已恢复上下文都没有给出某个标的的区间时，才可标注该标的击球区待确认。"
    } else {
        ""
    };
    format!(
        "{}\n\n{}{}",
        trigger_note, event.task_prompt, stable_context_note
    )
}

pub async fn run_scheduled_task(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    mut run_options: AgentRunOptions,
) -> AgentSessionResult {
    let full_prompt = build_scheduled_prompt_with_recovered_local_context(&core, event);
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
    if !event.bypass_quiet_hours
        && let Some((qh, tz_name)) = load_actor_quiet_hours(&core, &event.actor)
        && hone_core::quiet::quiet_window_active(
            tz_name.as_deref(),
            8,
            &qh.from,
            &qh.to,
            chrono::Utc::now(),
        )
    {
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
            } else if is_stale_market_data_success_fallback(&sanitized) {
                let suppressed_preview = truncate_for_log(sanitized.trim(), 200);
                tracing::warn!(
                    "[SchedulerDiag] stale_market_data_fallback job_id={} job={} chars={} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    sanitized.chars().count(),
                    suppressed_preview.replace('\n', "\\n"),
                );
                rollback_skipped_scheduler_assistant_turn(
                    &core.session_storage,
                    &session_id,
                    &sanitized,
                );
                ScheduledTaskExecution {
                    should_deliver: true,
                    content: String::new(),
                    error: Some(STALE_MARKET_DATA_FAILURE_MESSAGE.to_string()),
                    metadata: json!({
                        "failure_kind": "stale_market_data_fallback",
                        "suppressed_preview": suppressed_preview,
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
                if let Some(guarded_content) =
                    guard_commodity_causality_for_event(&sanitized, event)
                {
                    tracing::info!(
                        "[SchedulerDiag] commodity_causality_guarded job_id={} job={} target={}",
                        event.job_id,
                        event.job_name,
                        event.channel_target,
                    );
                    return ScheduledTaskExecution {
                        should_deliver: true,
                        content: guarded_content.clone(),
                        error: None,
                        metadata: scheduler_metadata_with_commodity_guard(
                            &sanitized,
                            &guarded_content,
                        ),
                        session_id: Some(session_id),
                    };
                }
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
            let suppressed_failure_kind =
                scheduler_suppressed_failure_kind(response.error.as_deref());
            if sanitized_error.is_none() {
                tracing::warn!(
                    "[SchedulerDiag] suppressed internal failure fallback job_id={} job={} failure_kind={} error=\"{}\"",
                    event.job_id,
                    event.job_name,
                    suppressed_failure_kind,
                    response.error.as_deref().unwrap_or("").replace('\n', "\\n"),
                );
                persist_suppressed_scheduler_failure_turn(
                    &core.session_storage,
                    &session_id,
                    suppressed_failure_kind,
                );
            }
            let should_deliver = sanitized_error.is_some();
            ScheduledTaskExecution {
                should_deliver,
                content: String::new(),
                error: sanitized_error.or_else(|| {
                    Some(SCHEDULER_INTERNAL_FAILURE_LEDGER_MESSAGE.to_string())
                }),
                metadata: json!({
                    "failure_kind": suppressed_failure_kind,
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
                && let Some(guarded_content) =
                    guard_direct_trade_instruction_for_event(&execution.content, event)
            {
                tracing::info!(
                    "[HeartbeatDiag] direct_trade_instruction_guarded job_id={} job={} target={}",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                );
                execution.content = guarded_content;
                update_heartbeat_delivery_preview_metadata(
                    &mut execution.metadata,
                    &execution.content,
                );
                if let Value::Object(map) = &mut execution.metadata {
                    map.insert(
                        "direct_trade_instruction_guarded".to_string(),
                        Value::Bool(true),
                    );
                    map.insert(
                        "guarded_preview".to_string(),
                        Value::String(truncate_for_log(execution.content.trim(), 200)),
                    );
                }
            }
            if execution.should_deliver
                && let Some(guarded_content) =
                    guard_commodity_causality_for_event(&execution.content, event)
            {
                tracing::info!(
                    "[HeartbeatDiag] commodity_causality_guarded job_id={} job={} target={}",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                );
                execution.content = guarded_content;
                update_heartbeat_delivery_preview_metadata(
                    &mut execution.metadata,
                    &execution.content,
                );
                if let Value::Object(map) = &mut execution.metadata {
                    map.insert("commodity_causality_guarded".to_string(), Value::Bool(true));
                    map.insert(
                        "guarded_preview".to_string(),
                        Value::String(truncate_for_log(execution.content.trim(), 200)),
                    );
                }
            }
            if execution.should_deliver
                && let Some(matched_preview) = heartbeat_duplicate_preview_match(
                    &execution.content,
                    &event.last_delivered_previews,
                )
            {
                let suppressed_preview = truncate_for_log(execution.content.trim(), 200);
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
                    "suppressed_preview": suppressed_preview,
                });
            }
            execution
        }
        Err(error) => {
            tracing::warn!(
                "[HeartbeatDiag] runner_error job_id={} job={} target={} model={} failure_kind={} error=\"{}\"",
                event.job_id,
                event.job_name,
                event.channel_target,
                heartbeat_model,
                heartbeat_runner_failure_kind(&error),
                truncate_for_log(&error, 280).replace('\n', "\\n"),
            );
            heartbeat_execution_from_runner_error(error, &heartbeat_model)
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
    // 与 turn_builder::PromptTurnBuilder 保持一致：self-managed-context runner
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
        runtime_input: bundle.compose_user_input(
            &build_scheduled_prompt_with_recovered_local_context(&core, event),
        ),
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
        runner_selection: heartbeat_runner_selection(),
        allowed_tools: Some(
            HEARTBEAT_ALLOWED_TOOLS
                .iter()
                .map(|tool| (*tool).to_string())
                .collect(),
        ),
        max_tool_calls: None,
        prompt_audit: None,
    })?;
    tracing::info!(
        "[HeartbeatDiag] run_start job_id={} job={} target={} runner={} model_override={} max_tokens={} timeout_secs={}",
        event.job_id,
        event.job_name,
        event.channel_target,
        execution.runner_name,
        run_options.model_override.as_deref().unwrap_or(""),
        HEARTBEAT_MAX_TOKENS,
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
        ScheduledTaskExecution, build_scheduled_prompt,
        build_scheduled_prompt_with_recovered_local_context, execute_scheduler_event,
        guard_commodity_causality_for_event, guard_direct_trade_instruction_for_event,
        has_skip_delivery_signal, heartbeat_duplicate_preview_match,
        heartbeat_execution_from_content, heartbeat_execution_from_content_at,
        heartbeat_execution_from_content_at_beijing, heartbeat_execution_from_runner_error,
        heartbeat_runner_selection, inspect_heartbeat_result, is_empty_success_fallback,
        is_stale_market_data_success_fallback, load_actor_quiet_hours,
        persist_suppressed_scheduler_failure_turn, rollback_skipped_scheduler_assistant_turn,
        sanitize_scheduler_delivery_text, scheduler_suppressed_failure_kind,
    };
    use crate::HoneBotCore;
    use crate::agent_session::{AgentRunOptions, AgentRunQuotaMode};
    use crate::execution::ExecutionRunnerSelection;
    use crate::prompt::PromptOptions;
    use crate::response_finalizer::EMPTY_SUCCESS_FALLBACK_MESSAGE;
    use hone_core::config::HoneConfig;
    use hone_core::{ActorIdentity, quiet::QuietHours};
    use hone_memory::{
        SessionStorage, build_compact_summary_metadata, session_message_from_text,
        session_message_text,
    };
    use hone_scheduler::SchedulerEvent;
    use serde_json::Value;
    use std::sync::Arc;

    fn assert_near_threshold_suppressed(execution: &ScheduledTaskExecution) {
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(
            execution.metadata["near_threshold_suppressed"].as_bool(),
            Some(true)
        );
    }

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
    fn heartbeat_malformed_triggered_json_recovers_unescaped_message_quotes() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"【Cerebras IPO 心跳监控】IPO 认购需求强劲，"市场报道/未核验" 仍需关注。"}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【Cerebras IPO 心跳监控】IPO 认购需求强劲，\"市场报道/未核验\" 仍需关注。"
                        .to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_triggered_json_recovers_message_before_extra_fields() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"【Cerebras IPO 认购超热 · 2026-05-09 15:00 北京时间】Bloomberg 报道称 IPO 认购需求超过 20 倍，CEO 称需求"超级健康"，触发业务进展提醒。","source":"Bloomberg","confidence":"medium"}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【Cerebras IPO 认购超热 · 2026-05-09 15:00 北京时间】Bloomberg 报道称 IPO 认购需求超过 20 倍，CEO 称需求\"超级健康\"，触发业务进展提醒。"
                        .to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_triggered_json_keeps_quoted_colon_text_inside_message() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"【RKLB 异动提醒 | 检查时间：2026-05-10 11:30 北京时间】管理层称"公司史上最强一季度","订单需求":持续强劲，单日涨跌幅超过 8%，触发提醒。","source":"earnings call"}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【RKLB 异动提醒 | 检查时间：2026-05-10 11:30 北京时间】管理层称\"公司史上最强一季度\",\"订单需求\":持续强劲，单日涨跌幅超过 8%，触发提醒。"
                        .to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_prefixed_malformed_triggered_json_is_recovered() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"最终输出如下：
{"status":"triggered","message":"【RKLB 异动提醒】管理层称"公司史上最强一季度"，触发提醒。"}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【RKLB 异动提醒】管理层称\"公司史上最强一季度\"，触发提醒。".to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_triggered_json_recovers_truncated_message() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"【持仓重大事件】ASTS 大股东减持、BlueBird 7 发射异常，触发条件已满足"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【持仓重大事件】ASTS 大股东减持、BlueBird 7 发射异常，触发条件已满足"
                        .to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_triggered_json_recovers_unquoted_status() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status": triggered, "message": "【DRAM 心跳监控】盘中触及上市以来新高，触发提醒。"}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【DRAM 心跳监控】盘中触及上市以来新高，触发提醒。".to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_triggered_json_recovers_smart_quoted_message() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":“【持仓心跳检测】ASTS Q1 财报与盘中走势触发提醒。”}"#
            ),
            (
                HeartbeatOutcome::Deliver(
                    "【持仓心跳检测】ASTS Q1 财报与盘中走势触发提醒。".to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_near_threshold_trigger_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"ASTS 最新价格 $71.88，相对昨收 $77.20 跌幅 -6.89%，触发原因：单日涨跌幅（跌）接近 8% 警戒阈值，且距离 8% 仅差约 1.1 个百分点。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_explicit_below_threshold_denial_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"触发条件：单日涨跌幅超过 8%。ASTS 当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛，本轮仅建议观察。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_explicit_not_triggered_threshold_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"RKLB异动提醒：最新价$77.02，较前收$78.59下跌-2.00%，未触发涨跌幅8%阈值，仅记录重大事件观察。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_explicit_not_exceeding_threshold_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"RKLB触发重大订单提醒：当前股价$77.02，涨跌幅未超过8%阈值，合同事件仅作观察。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_watchlist_above_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"ASTS 当前 71.88，触发价≤69.83，仍高于触发价但已进入触发价上方区间，建议关注。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_watchlist_contradictory_lower_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"【价格提醒】ASTS触发买入条件。当前价格$71.88，已低于触发价$69.83。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_watchlist_touch_or_below_above_trigger_price_is_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"【触发条件】ASTS 跌至 69.85，已触及或低于触发价 69.83。"}"#,
            "model-x",
        );
        assert_near_threshold_suppressed(&execution);
    }

    #[test]
    fn heartbeat_record_high_trigger_is_not_near_threshold_suppressed() {
        let execution = heartbeat_execution_from_content(
            r#"{"status":"triggered","message":"【DRAM 心跳监控】触发条件：DRAM 盘中创历史新高（满足条件2）。盘中最高 $56.38 = 上市以来历史最高价，本轮应发送提醒。"}"#,
            "model-x",
        );
        assert!(execution.should_deliver);
        assert!(execution.error.is_none());
        assert_ne!(
            execution.metadata["near_threshold_suppressed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn heartbeat_stale_price_timestamp_trigger_is_suppressed() {
        let execution = heartbeat_execution_from_content_at(
            r#"{"status":"triggered","message":"XAU/USD 现货黄金当前价格已跌破 $4,500 阈值，现报 $4,483.12（2026年4月4日），较昨收下跌约 0.54%。"}"#,
            "MiniMax-M2.7-highspeed",
            chrono::NaiveDate::from_ymd_opt(2026, 5, 27).expect("date"),
        );

        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(
            execution.metadata["failure_kind"].as_str(),
            Some("stale_price_timestamp")
        );
        assert_eq!(
            execution.metadata["stale_price_timestamp"].as_str(),
            Some("2026-04-04")
        );
        assert_eq!(
            execution.metadata["stale_price_timestamp_suppressed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn heartbeat_recent_price_timestamp_trigger_is_allowed() {
        let execution = heartbeat_execution_from_content_at(
            r#"{"status":"triggered","message":"XAU/USD 现货黄金当前价格已跌破 $4,500 阈值，现报 $4,483.12（2026年5月26日），检查时间 2026年5月27日。"}"#,
            "MiniMax-M2.7-highspeed",
            chrono::NaiveDate::from_ymd_opt(2026, 5, 27).expect("date"),
        );

        assert!(execution.should_deliver);
        assert!(execution.error.is_none());
        assert_ne!(
            execution.metadata["stale_price_timestamp_suppressed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn heartbeat_future_price_timestamp_trigger_is_suppressed() {
        let execution = heartbeat_execution_from_content_at(
            r#"{"status":"triggered","message":"【黄金急跌预警】XAU/USD 现货黄金当前价 4161.56 美元/盎司（数据时间：2026年6月18日 北京时间 13:10，盘中日低 4130.62 美元/盎司），已跌破 4500 美元/盎司阈值。"}"#,
            "MiniMax-M2.7-highspeed",
            chrono::NaiveDate::from_ymd_opt(2026, 6, 10).expect("date"),
        );

        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(
            execution.metadata["failure_kind"].as_str(),
            Some("stale_price_timestamp")
        );
        assert_eq!(
            execution.metadata["stale_price_timestamp"].as_str(),
            Some("2026-06-18")
        );
        assert_eq!(
            execution.metadata["stale_price_timestamp_suppressed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn heartbeat_normalizes_conflicting_beijing_trigger_time() {
        let reference_now =
            chrono::DateTime::parse_from_rfc3339("2026-05-29T11:31:32+08:00").expect("time");
        let execution = heartbeat_execution_from_content_at_beijing(
            r#"{"status":"triggered","message":"2026年5月29日 北京时间 04:00 盘后监控触发。已核验事实：AI 产业链出现关键事件。"}"#,
            "MiniMax-M2.7-highspeed",
            reference_now,
        );

        assert!(execution.should_deliver);
        assert!(execution.error.is_none());
        assert!(execution.content.contains("北京时间 11:31 盘后监控触发"));
        assert!(!execution.content.contains("北京时间 04:00 盘后监控触发"));
        assert_eq!(
            execution.metadata["beijing_trigger_time_normalized"].as_bool(),
            Some(true)
        );
        assert_eq!(
            execution.metadata["original_beijing_trigger_time"].as_str(),
            Some("04:00")
        );
    }

    #[test]
    fn heartbeat_prompt_rejects_direct_trade_instructions() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_cai", None::<String>).expect("actor"),
            job_id: "job-cai".to_string(),
            job_name: "CAI破位预警".to_string(),
            task_prompt: "CAI 跌破 52 周低点时提醒".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_cai".to_string(),
            delivery_key: "delivery-cai".to_string(),
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
        assert!(prompt.contains("交易动作边界"));
        assert!(prompt.contains("不得输出“无条件止损”"));
        assert!(prompt.contains("结合仓位、成本、流动性和风险承受能力复核"));
    }

    #[test]
    fn heartbeat_prompt_requires_noop_json_for_contract_conflicts() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_oil", None::<String>).expect("actor"),
            job_id: "job-oil".to_string(),
            job_name: "全天原油价格3小时播报".to_string(),
            task_prompt: "如果当前小时符合条件，播报原油价格。".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_oil".to_string(),
            delivery_key: "delivery-oil".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
            schedule_hour: 15,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("不要解释冲突"));
        assert!(prompt.contains("不要复述规则"));
        assert!(prompt.contains("不要输出空文本"));
        assert!(prompt.contains(r#"必须返回 `{"status":"noop"}`"#));
        assert!(prompt.contains("必须以最少工具调用收口"));
        assert!(prompt.contains("严禁输出工具配置"));
        assert!(prompt.contains("set_immediate_kinds"));
        assert!(prompt.contains("误入配置/建档/任务治理路径"));
    }

    #[test]
    fn heartbeat_direct_trade_instruction_gets_risk_guard() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_cai", None::<String>).expect("actor"),
            job_id: "job-cai".to_string(),
            job_name: "CAI破位预警".to_string(),
            task_prompt: "CAI 跌破 52 周低点时提醒".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_cai".to_string(),
            delivery_key: "delivery-cai".to_string(),
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

        let guarded = guard_direct_trade_instruction_for_event(
            "【CAI破位预警】CAI 跌破 52 周低点，当前价 $12.30，成交量放大。建议动作：无条件止损，不建议抄底或持有等待反弹。",
            &event,
        )
        .expect("direct trade instruction should be guarded");

        assert!(guarded.contains("不构成买卖、止损、加仓或清仓指令"));
        assert!(guarded.contains("CAI 跌破 52 周低点"));
        assert!(guarded.contains("当前价 $12.30"));
        assert!(!guarded.contains("无条件止损"));
        assert!(!guarded.contains("不建议抄底或持有等待反弹"));
    }

    #[test]
    fn heartbeat_direct_trade_instruction_detects_action_heading() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_cai", None::<String>).expect("actor"),
            job_id: "job-cai".to_string(),
            job_name: "CAI破位预警".to_string(),
            task_prompt: "CAI 跌破 52 周低点时提醒".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_cai".to_string(),
            delivery_key: "delivery-cai".to_string(),
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

        let guarded = guard_direct_trade_instruction_for_event(
            "【CAI破位预警】当前价跌破阈值。建议动作：止损，不建议抄底或持有等待反弹。",
            &event,
        )
        .expect("action heading should be guarded");

        assert!(guarded.contains("不构成买卖、止损、加仓或清仓指令"));
        assert!(!guarded.contains("建议动作：止损"));
        assert!(!guarded.contains("不建议抄底或持有等待反弹"));
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
    fn heartbeat_backticked_triggered_json_example_is_not_recovered() {
        let content = "如果条件满足，应输出 `{\"status\":\"triggered\",\"message\":\"小米跌破 30 港元\"}`；当前条件未满足。";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::PlainTextNoop)
        );
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
    fn scheduler_delivery_text_strips_skill_load_degradation_prelude() {
        let raw = "定时任务技能在当前运行器里没有成功加载，我改用行情和新闻工具直接完成这次复盘。\n\n组合今日核心变化：ORCL 与 AMD 对组合贡献最大，QCOM 和 IBM 权重漂移较小，后续重点看云业务订单和 AI 服务器出货节奏。";
        let sanitized = sanitize_scheduler_delivery_text(raw);
        assert_eq!(
            sanitized,
            "组合今日核心变化：ORCL 与 AMD 对组合贡献最大，QCOM 和 IBM 权重漂移较小，后续重点看云业务订单和 AI 服务器出货节奏。"
        );
        assert!(!sanitized.contains("当前运行器"));
        assert!(!sanitized.contains("技能"));
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
    fn scheduler_detects_stale_market_data_success_fallback() {
        assert!(is_stale_market_data_success_fallback(
            "说明：本轮重新拉取持仓实时行情时，底层行情数据链路暂时阻断。\n以下价格使用本会话此前已核验的美股5月1日收盘口径；新闻、评级与产业动态使用本轮搜索核验。"
        ));
        assert!(is_stale_market_data_success_fallback(
            "本轮报价接口触及限额，以下持仓价格采用同一会话04:30已校验的美股4月29日收盘口径。"
        ));
        assert!(!is_stale_market_data_success_fallback(
            "本轮新闻检索正常，以下价格使用同窗 data_fetch 返回的最新行情。"
        ));
        assert!(!is_stale_market_data_success_fallback(
            "行情数据获取失败，已跳过报价表，不复用旧价格。"
        ));
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
    fn suppressed_scheduler_failure_kind_classifies_acp_disconnect() {
        assert_eq!(
            scheduler_suppressed_failure_kind(Some(
                "codex acp error: stream disconnected before completion"
            )),
            "acp_transport_disconnect"
        );
        assert_eq!(
            scheduler_suppressed_failure_kind(Some("codex acp session/prompt idle timeout (180s)")),
            "scheduler_runner_timeout"
        );
        assert_eq!(
            scheduler_suppressed_failure_kind(Some("codex acp prompt ended before tool completion")),
            "internal_error_suppressed"
        );
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
    fn heartbeat_not_triggered_json_status_is_compatible_noop() {
        let content = r#"{"status":"not_triggered","message":"条件未触发"}"#;
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonNoop)
        );
        let execution = heartbeat_execution_from_content(content, "model-x");
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(execution.metadata["parse_kind"], "JsonNoop");
    }

    #[test]
    fn heartbeat_trigger_alias_json_status_delivers_message() {
        let content = r#"{"status":"condition_met","message":"触发事实"}"#;
        assert_eq!(
            inspect_heartbeat_result(content),
            (
                HeartbeatOutcome::Deliver("触发事实".to_string()),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_empty_json_is_compatible_noop() {
        let (outcome, parse_kind) = inspect_heartbeat_result("{}");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonEmptyStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
        let execution = heartbeat_execution_from_content("{}", "model-x");
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(execution.metadata["parse_kind"], "JsonEmptyStatus");
    }

    #[test]
    fn heartbeat_think_plus_empty_json_is_compatible_noop() {
        let (outcome, parse_kind) = inspect_heartbeat_result("<think>reasoning</think>\n\n{}");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonEmptyStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
        let execution =
            heartbeat_execution_from_content("<think>reasoning</think>\n\n{}", "model-x");
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(execution.metadata["parse_kind"], "JsonEmptyStatus");
    }

    #[test]
    fn heartbeat_plain_text_noop_is_compatible_noop() {
        let content = "<think>\n当前价格高于触发线，条件未满足，所以本轮应该返回 noop。\n";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::PlainTextNoop)
        );
        let execution = heartbeat_execution_from_content(content, "MiniMax-M2.7-highspeed");
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(execution.metadata["parse_kind"], "PlainTextNoop");
    }

    #[test]
    fn heartbeat_closed_think_only_noop_is_compatible_noop() {
        let content = "<think>当前没有触发条件，本轮不发送。</think>";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::PlainTextNoop)
        );
        let execution = heartbeat_execution_from_content(content, "MiniMax-M2.7-highspeed");
        assert!(!execution.should_deliver);
        assert!(execution.error.is_none());
        assert_eq!(execution.metadata["parse_kind"], "PlainTextNoop");
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
    fn heartbeat_provider_quota_error_is_classified() {
        let execution = heartbeat_execution_from_runner_error(
            "LLM 错误: upstream HTTP 402: This request requires more credits, or fewer max_tokens (code: 402)"
                .to_string(),
            "moonshotai/kimi-k2.5",
        );
        assert!(!execution.should_deliver);
        let error = execution
            .error
            .as_deref()
            .expect("provider quota error should be recorded");
        assert!(error.contains("HTTP 402"), "unexpected error: {error}");
        assert_eq!(
            execution.metadata["failure_kind"],
            "provider_quota_exhausted"
        );
        assert_eq!(
            execution.metadata["heartbeat_model"],
            "moonshotai/kimi-k2.5"
        );
    }

    #[test]
    fn heartbeat_provider_429_quota_error_is_classified() {
        let execution = heartbeat_execution_from_runner_error(
            "LLM 错误: 所有 OpenAI-compatible API Key 均失败（共 1 个）。最后错误：LLM 错误: upstream HTTP 429: rate limit exceeded (code: 429)"
                .to_string(),
            "mimo-v2.5-pro",
        );
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.metadata["failure_kind"],
            "provider_quota_exhausted"
        );
        assert_eq!(execution.metadata["heartbeat_model"], "mimo-v2.5-pro");
    }

    #[test]
    fn heartbeat_provider_http_error_is_classified_without_noop() {
        let execution = heartbeat_execution_from_runner_error(
            "LLM 错误: upstream HTTP 500: provider unavailable".to_string(),
            "model-x",
        );
        assert!(!execution.should_deliver);
        assert!(execution.error.is_some());
        assert_eq!(execution.metadata["failure_kind"], "provider_http_error");
    }

    #[test]
    fn heartbeat_context_overflow_error_is_not_classified_as_noop() {
        let execution = heartbeat_execution_from_runner_error(
            "LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"
                .to_string(),
            "mimo-v2.5-pro",
        );
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some(
                "LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"
            )
        );
        assert_eq!(
            execution.metadata["failure_kind"],
            "context_window_overflow"
        );
        assert_eq!(execution.metadata["parse_kind"], "ContextOverflowError");
        assert_eq!(execution.metadata["heartbeat_model"], "mimo-v2.5-pro");
    }

    #[test]
    fn heartbeat_runner_uses_capped_completion_budget() {
        match heartbeat_runner_selection() {
            ExecutionRunnerSelection::AuxiliaryFunctionCalling {
                max_iterations,
                max_tokens_override,
            } => {
                assert_eq!(max_iterations, 18);
                assert_eq!(max_tokens_override, Some(4096));
            }
            ExecutionRunnerSelection::Configured => {
                panic!("heartbeat must use auxiliary function-calling runner")
            }
        }
    }

    #[test]
    fn heartbeat_tool_allowlist_stays_narrow() {
        assert_eq!(
            super::HEARTBEAT_ALLOWED_TOOLS,
            &[
                "data_fetch",
                "web_search",
                "portfolio",
                "missed_events",
                "local_list_files",
                "local_search_files",
                "local_read_file",
            ]
        );
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
        assert!(prompt.contains("本轮权威检查时间（北京时间）"));
        assert!(prompt.contains("检查时间必须使用上方"));
        assert!(prompt.contains("时间口径命名约束"));
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
    fn heartbeat_duplicate_preview_match_allows_asts_after_rklb_move() {
        let message =
            "【ASTS 单日涨跌幅超阈值】ASTS 单日上涨 14.8%，Rakuten 退出完成，Q1 财报临近。";
        let previews = vec![(
            "2026-05-09T18:00:31+08:00".to_string(),
            "【RKLB 单日暴涨34% · 2026-05-09 18:00 北京时间】RKLB 因新合同与财报预期出现单日大幅上涨。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_tem_after_rklb_move() {
        let message =
            "【TEM Q1财报超预期 + 可转债发行 + 新合作】TEM 披露 Q1 收入增长，并宣布新的战略合作。";
        let previews = vec![(
            "2026-05-09T18:00:31+08:00".to_string(),
            "【RKLB 单日暴涨34% · 2026-05-09 18:00 北京时间】RKLB 因新合同与 Q1 财报预期出现单日大幅上涨。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_portfolio_asts_after_rklb_move() {
        let message = "【ASTS 单日暴涨近15%】持仓重大事件：ASTS 单日涨幅接近 15%，Rakuten 退出完成，Q1 财报临近。";
        let previews = vec![(
            "2026-05-09T18:00:31+08:00".to_string(),
            "【RKLB 单日暴涨34% · 2026-05-09 18:00 北京时间】RKLB 因新合同与财报预期出现单日大幅上涨。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_cross_job_different_entities() {
        let message = "【ORCL 大事件监控 | 检查时间: 2026-05-04 23:00 北京时间】ORCL 最新价 171.83 美元，OpenAI 合作叙事仍在发酵。";
        let previews = vec![(
            "2026-05-04T22:31:43+08:00".to_string(),
            "【Cerebras IPO重大进展 | 检查时间: 2026-05-04 22:30 北京时间】Cerebras IPO 定价区间 22-25 美元，AWS Bedrock 与 OpenAI 协议兼容继续推进。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_portfolio_alert_after_unrelated_ipo() {
        let message = "【持仓重大事件心跳检测 | 检查时间: 2026-05-04 23:00 北京时间】TEM 财报窗口临近，ORCL 价格异动继续触发持仓重大事件观察。";
        let previews = vec![(
            "2026-05-04T22:31:43+08:00".to_string(),
            "【Cerebras IPO重大进展 | 检查时间: 2026-05-04 22:30 北京时间】Cerebras IPO 定价区间 22-25 美元，AWS Bedrock 与 OpenAI 协议兼容继续推进。"
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
    fn heartbeat_duplicate_preview_match_allows_tsla_distinct_same_ticker_events() {
        let message =
            "【TSLA 负向触发】TSLA 因 FSD 诉讼风险扩大和车辆召回事件触发提醒，检查时间 19:02。";
        let previews = vec![(
            "2026-05-10T15:00:00+08:00".to_string(),
            "【TSLA 重大事件】Tesla Semi 新订单披露，SEC 与 Musk 和解进展继续发酵。".to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_cerebras_after_portfolio_summary() {
        let message =
            "【Cerebras IPO与业务进展心跳监控】Cerebras IPO 定价区间上调，上市时间线出现新进展。";
        let previews = vec![(
            "2026-05-10T18:30:00+08:00".to_string(),
            "【持仓重大事件】RKLB 与 TEM 本轮均有重要更新，持仓摘要已提醒。".to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_dram_record_high_after_cerebras_ipo() {
        let message = "【DRAM 心跳监控】触发条件：DRAM 盘中创历史新高（满足条件2）。盘中最高 $56.38 = 上市以来历史最高价。";
        let previews = vec![(
            "2026-05-12T08:30:00+08:00".to_string(),
            "【Cerebras IPO 重大更新 | 2026-05-12 08:30 北京时间】Cerebras IPO 定价区间上修，上市时间线出现新进展。"
                .to_string(),
        )];

        assert!(heartbeat_duplicate_preview_match(message, &previews).is_none());
    }

    #[test]
    fn heartbeat_duplicate_preview_match_allows_cerebras_ipo_pricing_range_revision() {
        let message = "【Cerebras IPO与业务进展心跳监控】Cerebras IPO 定价区间从 $115-$125 上调至 $150-$160，发行股数与募资额同步提高，预计今日定价。";
        let previews = vec![(
            "2026-05-13T13:00:00+08:00".to_string(),
            "【Cerebras IPO 临门】Cerebras IPO 定价区间仍为 $115-$125，最终定价预计当晚或次日早间确定。"
                .to_string(),
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
        assert!(has_skip_delivery_signal(
            "RKLB 今日不触发重大催化或风险证伪推送。"
        ));
        assert!(has_skip_delivery_signal(
            "TEM 今日不触发新增重大催化或风险证伪推送。"
        ));
        assert!(has_skip_delivery_signal("AAOI 今日不触发新增重大推送。"));
        assert!(has_skip_delivery_signal(
            "今日不触发新增重大\n推送，保留观察即可。"
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
        assert!(prompt.contains("明显旧日期、缺少价格时间戳"));
        assert!(prompt.contains("旧价格包装成当前触发依据"));
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

    #[test]
    fn commodity_heartbeat_causality_claim_gets_uncertainty_guard() {
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

        let guarded = guard_commodity_causality_for_event(
            "近期变动背景：油价承压主要受 OPEC+ 供应政策不确定性及全球经济增速担忧影响。",
            &event,
        )
        .expect("commodity causal claim should be guarded");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(guarded.contains("不能视为已确认油价主因"));
        assert!(!guarded.contains("OPEC+"));
        assert!(!guarded.contains("全球经济增速担忧"));
    }

    #[test]
    fn commodity_heartbeat_geopolitical_risk_premium_gets_guard() {
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

        let guarded = guard_commodity_causality_for_event(
            "WTI 原油：$95.79/桶。地缘政治升级：美伊在霍尔木兹海峡发生交火事件，推高风险溢价。供应中断担忧：中东约 670 万桶/日产能存在关停风险。",
            &event,
        )
        .expect("geopolitical risk-premium claim should be guarded");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(!guarded.contains("【已保留的价格口径】"));
        assert!(!guarded.contains("WTI 原油：$95.79/桶"));
        assert!(!guarded.contains("美伊在霍尔木兹海峡发生交火事件"));
        assert!(!guarded.contains("670 万桶"));
    }

    #[test]
    fn commodity_heartbeat_generic_market_disclaimer_does_not_bypass_causality_guard() {
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

        let guarded = guard_commodity_causality_for_event(
            "注：价格为市场参考数据，仅供参考。近期变动主因：中东地缘风险溢价持续消退，OPEC+ 延续增产节奏。",
            &event,
        )
        .expect("generic market disclaimer should not qualify causal claims");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(!guarded.contains("中东地缘风险溢价"));
    }

    #[test]
    fn commodity_heartbeat_keeps_already_qualified_causality() {
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

        assert!(
            guard_commodity_causality_for_event(
                "原因暂不归因；仅报告 WTI 原油当前 $95.79/桶。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_heartbeat_guard_rewrites_prefixed_bad_body() {
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

        let guarded = guard_commodity_causality_for_event(
            "【归因口径】原因归因未完成同窗来源核验，以下宏观、地缘、供需表述仅作待确认线索，不能视为已确认油价主因。\nWTI 6月合约估算收盘约 $95.9/桶（精确收盘价未独立校验）。中东霍尔木兹海峡近封锁状态持续推高地缘风险溢价。5月5日中东直接军事冲突消息曾令油价单日飙升超6%。2026年以来布伦特累计涨幅约 59%-80%。",
            &event,
        )
        .expect("prefixed but unsafe commodity claim should be rewritten");

        assert!(guarded.contains("已移除原正文中的宏观、地缘、供需、库存等主因叙述"));
        assert!(!guarded.contains("霍尔木兹海峡近封锁"));
        assert!(!guarded.contains("中东直接军事冲突"));
        assert!(!guarded.contains("59%-80%"));
        assert!(!guarded.contains("估算收盘约 $95.9"));
    }

    #[test]
    fn commodity_heartbeat_guard_rewrites_wrong_weekday_and_unverified_prices() {
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

        let guarded = guard_commodity_causality_for_event(
            "【2026-05-10 周六 18:00 北京时间】WTI 原油（近月合约）：约 $95.42/桶（5月8日 Bloomberg 数据）。布伦特原油（近月合约）：约 $100.49-$101.29/桶；近一个月布伦特累计上涨约4.76%。",
            &event,
        )
        .expect("wrong weekday and unverified commodity market claims should be guarded");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(!guarded.contains("2026-05-10 周六"));
        assert!(!guarded.contains("Bloomberg 数据"));
        assert!(!guarded.contains("累计上涨约4.76%"));
    }

    #[test]
    fn commodity_heartbeat_guard_rewrites_unverified_price_and_war_claims() {
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
            schedule_hour: 15,
            schedule_minute: 0,
            schedule_repeat: "heartbeat".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let guarded = guard_commodity_causality_for_event(
            "WTI原油约$99.02/桶，布伦特原油约$105.27/桶。价格上涨原因包括战争紧张、霍尔木兹海峡、沙特阿美 CEO 警告、美国战略储备贷款等。",
            &event,
        )
        .expect("unverified price and war causality should be guarded");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(guarded.contains("本轮未保留原正文中的价格或归因句"));
        assert!(!guarded.contains("$99.02"));
        assert!(!guarded.contains("$105.27"));
        assert!(!guarded.contains("战争紧张"));
        assert!(!guarded.contains("战略储备"));
    }

    #[test]
    fn commodity_guard_covers_non_heartbeat_oil_scheduler() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_oil", None::<String>).expect("actor"),
            job_id: "job-oil-close".to_string(),
            job_name: "Oil_Price_Monitor_Closing".to_string(),
            task_prompt: "收盘后汇总 WTI / Brent 价格与科技股影响".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_oil".to_string(),
            delivery_key: "delivery-oil-close".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 4,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let guarded = guard_commodity_causality_for_event(
            "WTI 约 101.02 美元，Brent 约 105.63 美元，WSJ 今日结算口径。油价回落对今晚科技股不是压制项，反而是边际缓和。",
            &event,
        )
        .expect("ordinary oil scheduler should share the commodity guard");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(!guarded.contains("101.02"));
        assert!(!guarded.contains("105.63"));
        assert!(!guarded.contains("WSJ"));
        assert!(!guarded.contains("科技股不是压制项"));
    }

    #[test]
    fn commodity_guard_covers_oil_scheduler_contract_months_and_tail_risk_claim() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_oil", None::<String>).expect("actor"),
            job_id: "job-oil-close".to_string(),
            job_name: "Oil_Price_Monitor_Closing".to_string(),
            task_prompt: "收盘后汇总 WTI / Brent 价格与科技股影响".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_oil".to_string(),
            delivery_key: "delivery-oil-close".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 4,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let guarded = guard_commodity_causality_for_event(
            "Brent Jul 2026 约 106.30 美元，WTI Jun 2026 约 101.80 美元。这对今晚高估值科技股不是尾盘强防守信号；油价仍是通胀与利率风险项，但从盘面看，QQQ、RKLB、COHR 没有被油价持续压住。",
            &event,
        )
        .expect("ordinary oil scheduler contract-month claims should be guarded");

        assert!(guarded.contains("未完成同窗来源核验"));
        assert!(guarded.contains("本轮未保留原正文中的价格或归因句"));
        assert!(!guarded.contains("106.30"));
        assert!(!guarded.contains("101.80"));
        assert!(!guarded.contains("尾盘强防守信号"));
        assert!(!guarded.contains("通胀与利率风险项"));
    }

    #[test]
    fn commodity_guard_covers_non_heartbeat_market_scheduler_output() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_oil", None::<String>).expect("actor"),
            job_id: "job-postmarket".to_string(),
            job_name: "OWALERT_PostMarket".to_string(),
            task_prompt: "盘后扫描影响科技成长股的宏观与行业变量".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_oil".to_string(),
            delivery_key: "delivery-postmarket".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 4,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let guarded = guard_commodity_causality_for_event(
            "USO 收盘附近 142.07；WTI 跌 1.1% 至 101.02 美元，Brent 跌 2.0% 至 105.63 美元，能源通胀压力边际缓和，是 AI 风险偏好修复的核心解释之一。",
            &event,
        )
        .expect("commodity claims in broader scheduler output should be guarded");

        assert!(guarded.contains("不能视为已确认油价主因"));
        assert!(!guarded.contains("101.02"));
        assert!(!guarded.contains("105.63"));
        assert!(!guarded.contains("风险偏好修复"));
    }

    #[test]
    fn commodity_guard_skips_broad_market_review_with_secondary_oil_clause() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-market-review".to_string(),
            job_name: "每日美股大盘风险简报".to_string(),
            task_prompt: "生成包含 Nasdaq、S&P 500、VIX 与风险偏好的市场复盘".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-market-review".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let original = "美股因 Memorial Day 休市，纳指期货与标普期货波动有限，VIX 回落到 13 附近，Fear & Greed 维持中性。科技股整体等待英伟达链条与长端利率信号，能源板块则受油价回落与库存预期压制。";

        assert_eq!(
            guard_commodity_causality_for_event(original, &event),
            None,
            "broad market reviews should not be fully replaced by the commodity guard"
        );
    }

    #[test]
    fn commodity_guard_skips_cross_market_review_with_oil_sector_mention() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-ah-review".to_string(),
            job_name: "A股港股收盘后跨市场复盘".to_string(),
            task_prompt: "总结 A 股、港股与美股休市背景下的跨市场结构变化".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-ah-review".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 17,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let original = "A股今日由算力和机器人链领涨，港股因佛诞翌日休市，美股则因 Memorial Day 休市。上证与恒生科技相关映射资产仍偏强，油气板块因油价回落承压，但这只是结构分化的一部分。";

        assert_eq!(
            guard_commodity_causality_for_event(original, &event),
            None,
            "cross-market reviews should keep their main content when oil is only one sector clause"
        );
    }

    #[test]
    fn commodity_guard_skips_low_segmentation_ah_market_review_with_oil_risk_note() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-ah-close-review".to_string(),
            job_name: "A股港股收盘后跨市场复盘".to_string(),
            task_prompt: "复盘 A 股、港股、美股映射、AI 硬件链和跨市场风险提示。".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-ah-close-review".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 17,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let original = "北京时间 2026年5月29日 17:30，A股、港股今天均实际开市；结论是：A股从昨天硬科技反攻切到高位兑现，港股则靠联想、百度、内房、航空托住指数，AI 硬件、港股科技和美股映射仍是正文主体，风险提示里只把 WTI、Brent 与油价波动作为通胀和航空成本的边际变量，不能把它当成本轮 A/H 收盘复盘的主因。";

        assert_eq!(
            guard_commodity_causality_for_event(original, &event),
            None,
            "low-segmentation A/H market reviews should not be treated as commodity-first text"
        );
    }

    #[test]
    fn commodity_guard_does_not_rewrite_broad_us_market_risk_brief() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-us-risk".to_string(),
            job_name: "每日美股大盘风控简报".to_string(),
            task_prompt: "复盘 Nasdaq、S&P 500、VIX、长端利率和主要风险因子。".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-us-risk".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【每日美股大盘风控简报】\n美股因 Memorial Day 休市，Nasdaq、S&P 500 和 QQQ 缺少新的收盘确认。\nVIX 与 Fear & Greed 仍指向风险偏好偏谨慎，长端利率是今晚估值压力的主线。\n油价与能源需求担忧只是观察项，不足以解释整个科技股风险温度。\n操作上继续关注 AI 算力、半导体和高 beta 成长股的开盘确认。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_broad_market_prompt_with_oil_watch_item() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-us-premarket".to_string(),
            job_name: "美股盘前宏观与财报日历梳理".to_string(),
            task_prompt: "梳理美股盘前宏观、财报日历、AI 芯片链与油价观察项。".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-us-premarket".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【美股盘前宏观与财报日历梳理】\n今晚美股盘前的主线仍是 Nasdaq、S&P 500 与 QQQ 的风险偏好修复，长端利率和消费者信心数据决定估值压力。\nAI 芯片、半导体和云资本开支是财报日历的重点。\n油价回落受中东谈判预期和能源需求担忧影响，但这只是宏观观察项，不应替代大盘、财报和科技股主线。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_owalert_premarket_when_market_context_dominates() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-owalert-premarket".to_string(),
            job_name: "OWALERT_PreMarket".to_string(),
            task_prompt: "盘前扫描美股期货、QQQ、AI 二阶链、油价与宏观风险，形成市场行动简报。"
                .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-owalert-premarket".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 21,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【OWALERT_PreMarket】\n美股期货修复，QQQ 盘前走强，Nasdaq 与 S&P 500 的风险偏好改善。\nAI 二阶链继续跟踪电力、光模块和半导体设备，开盘后看成交确认。\n油价低于 100 美元，主要受中东谈判预期和需求担忧影响，对今晚市场只是风险变量之一，不是本轮盘前结论的主体。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_ai_morning_briefing_with_secondary_oil_clause() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-ai-morning".to_string(),
            job_name: "Hone_AI_Morning_Briefing".to_string(),
            task_prompt: "生成 AI 科技前沿、宏观风险、持仓标的和油价观察项的早间 briefing。"
                .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-ai-morning".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 8,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【Hone AI Morning Briefing】AI 基建和半导体仍是今日主线，QQQ 与 Nasdaq 的风险偏好需要看长端利率确认。宏观侧关注 PCE、FOMC 纪要和美元指数。油价回落主要受中东谈判预期影响，但这只是组合风险变量，不能替代 AI 科技和持仓标的早报主体。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_rate_cut_probability_digest() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("discord", "rate-cut", None::<String>).expect("actor"),
            job_id: "job-rate-cut".to_string(),
            job_name: "每日美股降息概率推送".to_string(),
            task_prompt: "汇总 FedWatch、FOMC、PCE 风险和美股降息概率。".to_string(),
            channel: "discord".to_string(),
            channel_scope: None,
            channel_target: "rate-cut".to_string(),
            delivery_key: "delivery-rate-cut".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 9,
            schedule_minute: 30,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【每日美股降息概率推送】FedWatch 显示市场继续押注年内降息，FOMC 纪要和 PCE 是本周利率路径的核心变量。美股方面，Nasdaq 与 S&P 500 对长端利率更敏感。油价上行会影响通胀预期，但不是本轮降息概率分析的主体。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_us_market_risk_brief_with_repeated_oil_risk_clauses() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-us-evening-risk".to_string(),
            job_name: "美股大盘晚间风控简报".to_string(),
            task_prompt: "生成美股盘前、PCE、GDP、伊朗局势、油价和利率扰动的广义风控简报。"
                .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-us-evening-risk".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【美股大盘晚间风控简报】Nasdaq、S&P 500、QQQ 盘前维持强势，VIX 与 Fear & Greed 显示风险偏好仍偏热，AI 半导体和电力链是今晚主线。\nPCE 与 GDP 修正值会影响长端利率，科技股仓位需要看开盘后成交和涨跌家数确认。\n油价受伊朗局势、供应担忧和需求预期影响回落，但这只是通胀路径的边际变量。\n若油价继续下行，能源通胀压力会缓和，不过不能把它作为今晚 AI 成长股修复的核心解释。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_ai_chain_digest_with_secondary_oil_mentions() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-ai-chain".to_string(),
            job_name: "美股盘后AI及高景气产业链推演".to_string(),
            task_prompt: "盘后推演 AI 硬件、CPO、PCB、服务器、液冷电源与美股盘前映射。".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-ai-chain".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 45,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【美股盘后AI及高景气产业链推演】AI 硬件、CPO、PCB、服务器和液冷电源仍是盘后映射的主体，重点看 NVDA、AVGO、ANET、VRT 与光模块链条。\nNasdaq 与 QQQ 的风险偏好主要取决于长端利率、财报指引和半导体成交强度。\n油价受中东谈判预期和需求担忧影响回落，会降低部分能源通胀压力。\n但油价变化只是宏观噪音，不应覆盖 AI 产业链、半导体和高景气方向的推演正文。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_weekend_us_market_temperature_review() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-us-weekend-temperature".to_string(),
            job_name: "每日美股大盘温度检查".to_string(),
            task_prompt: "周末按最近完整交易日收盘口径检查 Nasdaq、S&P 500、Greed 情绪与追涨赔率。"
                .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-us-weekend-temperature".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【每日美股大盘温度检查】当前北京时间 2026年5月30日20:00，美东时间周六08:00，美股现货与期货均处于周末休市阶段，只能按最近完整交易日收盘口径复盘。Nasdaq 与 S&P 500 仍在高位，低波动、Greed 情绪和追涨赔率显示风险偏好偏强但偏热。\nAI 硬件盈利兑现后仍是主线，利率和油价压制边际缓和，但这只是大盘温度的风险变量，不是原油或大宗商品播报。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn commodity_guard_skips_weekend_us_market_risk_brief_with_oil_risk_variable() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_market", None::<String>).expect("actor"),
            job_id: "job-us-weekend-risk".to_string(),
            job_name: "每日美股大盘风险简报".to_string(),
            task_prompt: "周末按最近完整交易日收盘口径复盘 AI 硬件、利率、油价压制和高位偏热风险。"
                .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_market".to_string(),
            delivery_key: "delivery-us-weekend-risk".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        assert!(
            guard_commodity_causality_for_event(
                "【每日美股大盘风险简报】当前北京时间 2026年5月30日20:00，美股周末休市，本轮按 2026-05-29 最近完整交易日收盘口径评估。结论：Nasdaq、S&P 500 和 QQQ 的风险偏好仍偏强，AI 硬件盈利兑现、半导体高位震荡和追涨赔率是正文主体。\n风险提示：利率与油价压制有所缓和，但高位偏热和低波动更需要警惕；油价只是宏观风险变量，不能把本轮大盘风险简报改写成原油/大宗商品归因。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn non_commodity_heartbeat_does_not_get_causality_guard() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_rklb", None::<String>).expect("actor"),
            job_id: "job-rklb".to_string(),
            job_name: "RKLB异动监控".to_string(),
            task_prompt: "监控 RKLB 订单与价格异动".to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_rklb".to_string(),
            delivery_key: "delivery-rklb".to_string(),
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

        assert!(
            guard_commodity_causality_for_event(
                "近期变动背景：股价承压主要受供应链不确定性影响。",
                &event,
            )
            .is_none()
        );
    }

    #[test]
    fn scheduled_watchlist_hit_zone_prompt_keeps_stable_local_fields() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("feishu", "ou_watch", None::<String>).expect("actor"),
            job_id: "job-watch".to_string(),
            job_name: "核心观察股池晚间快报".to_string(),
            task_prompt:
                "按当前25支观察池发送日报，每个标的列出当前价格、击球区区间值、下一次财报时间。涉及价格和财报日期必须调用 data_fetch 校验。"
                    .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_watch".to_string(),
            delivery_key: "delivery-watch".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 23,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("稳定本地字段约束"));
        assert!(prompt.contains("data_fetch` 只校验最新价格和财报日期"));
        assert!(prompt.contains("不要因为行情工具没有返回击球区字段"));
        assert!(prompt.contains("统一降级为“待确认”"));
    }

    #[test]
    fn scheduled_watchlist_prompt_recovers_hit_zones_from_compact_summary() {
        let root = std::env::temp_dir().join(format!(
            "scheduler_hit_zone_prompt_{}_{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        let prefs_dir = root.join("prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("feishu", "ou_watch", None::<String>).expect("actor");
        let session_id = actor.session_id();
        core.session_storage
            .create_session_for_actor(&actor)
            .expect("create session");
        core.session_storage
            .append_session_messages(
                &session_id,
                vec![session_message_from_text(
                    "system",
                    "【Compact Summary】\n| 股票代码 | 公司名 | 当前价 | 击球区 | 财报时间 |\n| --- | --- | --- | --- | --- |\n| MSFT | 微软 | $416.97 | $335–$350 | 2026-04-29 |\n| TSM | 台积电 | $367.09 | 保守$290–$310 / 合理$320–$340 / 激进$345–$355 | 2026-07-16 |\n| LITE | Lumentum | $881.64 | 保守$520–$580 / 合理$600–$650 / 激进观察$680–$720 | 2026-05-05 |",
                    hone_core::beijing_now_rfc3339(),
                    Some(build_compact_summary_metadata("test")),
                )],
            )
            .expect("append summary");

        let event = SchedulerEvent {
            actor,
            job_id: "job-watch".to_string(),
            job_name: "核心观察股池晚间快报".to_string(),
            task_prompt:
                "按当前25支观察池发送日报，每个标的列出当前价格、击球区区间值、下一次财报时间。核心股包含 MSFT；拓展股包含 TSM、LITE。涉及价格和财报日期必须调用 data_fetch 校验。"
                    .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_watch".to_string(),
            delivery_key: "delivery-watch".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 23,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt_with_recovered_local_context(&core, &event);
        assert!(prompt.contains("【已恢复的本地击球区参考】"));
        assert!(prompt.contains("- MSFT: $335–$350"));
        assert!(prompt.contains("- TSM: 保守$290–$310 / 合理$320–$340 / 激进$345–$355"));
        assert!(prompt.contains("- LITE: 保守$520–$580 / 合理$600–$650 / 激进观察$680–$720"));
    }

    #[test]
    fn scheduled_watchlist_prompt_recovers_all_hit_zones_when_task_omits_tickers() {
        let root = std::env::temp_dir().join(format!(
            "scheduler_hit_zone_prompt_all_{}_{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        let prefs_dir = root.join("prefs");
        let core = make_test_core(&prefs_dir);
        let actor = ActorIdentity::new("feishu", "ou_watch_all", None::<String>).expect("actor");
        let session_id = actor.session_id();
        core.session_storage
            .create_session_for_actor(&actor)
            .expect("create session");
        core.session_storage
            .append_session_messages(
                &session_id,
                vec![session_message_from_text(
                    "system",
                    "【Compact Summary】\n| 股票代码 | 公司名 | 当前价 | 击球区 | 财报时间 |\n| --- | --- | --- | --- | --- |\n| MSFT | 微软 | $416.97 | $335-$350 | 2026-04-29 |\n| NVDA | 英伟达 | $183.40 | $150-$165 | 2026-05-28 |\n| GOOGL | Alphabet | $285.14 | $255-$275 | 2026-07-24 |\n| LITE | Lumentum | $881.64 | 保守$520-$580 / 合理$600-$650 / 激进观察$680-$720 | 2026-05-05 |",
                    hone_core::beijing_now_rfc3339(),
                    Some(build_compact_summary_metadata("test")),
                )],
            )
            .expect("append summary");

        let event = SchedulerEvent {
            actor,
            job_id: "job-watch".to_string(),
            job_name: "核心观察股池晚间快报".to_string(),
            task_prompt:
                "按当前25支观察池发送日报，每个标的列出当前价格、击球区区间值、下一次财报时间。涉及价格和财报日期必须调用 data_fetch 校验。"
                    .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_watch_all".to_string(),
            delivery_key: "delivery-watch".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 23,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt_with_recovered_local_context(&core, &event);
        assert!(prompt.contains("【已恢复的本地击球区参考】"));
        assert!(prompt.contains("- MSFT: $335-$350"));
        assert!(prompt.contains("- NVDA: $150-$165"));
        assert!(prompt.contains("- GOOGL: $255-$275"));
        assert!(prompt.contains("- LITE: 保守$520-$580 / 合理$600-$650 / 激进观察$680-$720"));
    }

    #[test]
    fn scheduled_watchlist_prompt_recovers_compact_inline_hit_zones() {
        let root = std::env::temp_dir().join(format!(
            "scheduler_hit_zone_prompt_compact_{}_{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        let prefs_dir = root.join("prefs");
        let core = make_test_core(&prefs_dir);
        let actor =
            ActorIdentity::new("feishu", "ou_watch_compact", None::<String>).expect("actor");
        let session_id = actor.session_id();
        core.session_storage
            .create_session_for_actor(&actor)
            .expect("create session");
        core.session_storage
            .append_session_messages(
                &session_id,
                vec![session_message_from_text(
                    "system",
                    "【Compact Summary】\n观察池击球区：MSFT $335-$350；NVDA $150-$165；GOOGL $255-$275。\nTSM 当前价 $367.09，击球区 保守$290-$310 / 合理$320-$340 / 激进$345-$355。\nLITE 击球区：待确认。",
                    hone_core::beijing_now_rfc3339(),
                    Some(build_compact_summary_metadata("test")),
                )],
            )
            .expect("append summary");

        let event = SchedulerEvent {
            actor,
            job_id: "job-watch".to_string(),
            job_name: "核心观察股池晚间快报".to_string(),
            task_prompt:
                "按当前25支观察池发送日报，每个标的列出当前价格、击球区区间值、下一次财报时间。涉及价格和财报日期必须调用 data_fetch 校验。"
                    .to_string(),
            channel: "feishu".to_string(),
            channel_scope: None,
            channel_target: "ou_watch_compact".to_string(),
            delivery_key: "delivery-watch".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 23,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        };

        let prompt = build_scheduled_prompt_with_recovered_local_context(&core, &event);
        assert!(prompt.contains("【已恢复的本地击球区参考】"));
        assert!(prompt.contains("- MSFT: $335-$350"));
        assert!(prompt.contains("- NVDA: $150-$165"));
        assert!(prompt.contains("- GOOGL: $255-$275"));
        assert!(prompt.contains("- TSM: 保守$290-$310 / 合理$320-$340 / 激进$345-$355"));
        assert!(!prompt.contains("- LITE: 待确认"));
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
