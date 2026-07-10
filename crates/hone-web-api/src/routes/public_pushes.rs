use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hone_core::ActorIdentity;
use hone_memory::cron_job::{WebPushMessage, WebPushMessageInput};
use hone_scheduler::SchedulerEvent;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::routes::history::legacy_scheduler_job_name;
use crate::routes::public::require_public_user;
use crate::state::AppState;

const DEFAULT_PUSH_PAGE_SIZE: usize = 30;
const MAX_PUSH_PAGE_SIZE: usize = 100;
const SUMMARY_MAX_CHARS: usize = 180;

#[derive(Debug, Deserialize)]
pub(crate) struct PublicPushListQuery {
    before: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct PublicPushListItem {
    push_id: String,
    job_id: String,
    title: String,
    summary: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct PublicPushDetail {
    push_id: String,
    job_id: String,
    title: String,
    summary: String,
    content: String,
    created_at: String,
}

pub(crate) struct StoredWebPush {
    pub message: WebPushMessage,
    pub unread_count: usize,
}

pub(crate) fn store_web_scheduler_push(
    state: &AppState,
    event: &SchedulerEvent,
    content: &str,
) -> hone_core::HoneResult<StoredWebPush> {
    let created_at = hone_core::beijing_now_rfc3339();
    let storage = state.core.cron_job_storage();
    let message = storage.upsert_web_push_message(
        &event.actor,
        WebPushMessageInput {
            push_id: event.delivery_key.clone(),
            job_id: event.job_id.clone(),
            job_name: event.job_name.clone(),
            summary: build_web_push_summary(&event.job_name, content),
            content: content.trim().to_string(),
            created_at,
        },
    )?;
    let unread_count = storage.count_unread_web_push_messages(&event.actor)?;
    Ok(StoredWebPush {
        message,
        unread_count,
    })
}

pub(crate) async fn handle_list_pushes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<PublicPushListQuery>,
) -> Response {
    let actor = match public_web_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PUSH_PAGE_SIZE)
        .clamp(1, MAX_PUSH_PAGE_SIZE);
    let storage = state.core.cron_job_storage();
    if let Err(error) = backfill_legacy_web_pushes(&state, &actor) {
        return crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("整理历史推送失败: {error}"),
        );
    }
    let mut messages = match storage.list_web_push_messages(
        &actor,
        query.before.as_deref(),
        limit.saturating_add(1),
    ) {
        Ok(messages) => messages,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取推送列表失败: {error}"),
            );
        }
    };
    let has_more = messages.len() > limit;
    messages.truncate(limit);
    let next_before = has_more
        .then(|| messages.last().map(|message| message.push_id.clone()))
        .flatten();
    let unread_count = match storage.count_unread_web_push_messages(&actor) {
        Ok(count) => count,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取推送未读数失败: {error}"),
            );
        }
    };

    Json(json!({
        "items": messages.into_iter().map(public_push_list_item).collect::<Vec<_>>(),
        "unread_count": unread_count,
        "next_before": next_before,
    }))
    .into_response()
}

pub(crate) async fn handle_open_push(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(push_id): Path<String>,
) -> Response {
    let actor = match public_web_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let storage = state.core.cron_job_storage();
    let message = match storage.get_web_push_message(&actor, &push_id) {
        Ok(Some(message)) => message,
        Ok(None) => return crate::routes::json_error(StatusCode::NOT_FOUND, "推送不存在"),
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取推送详情失败: {error}"),
            );
        }
    };
    if let Err(error) = storage.mark_web_push_messages_read_through(&actor, &push_id) {
        return crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("更新推送已读状态失败: {error}"),
        );
    }
    let unread_count = match storage.count_unread_web_push_messages(&actor) {
        Ok(count) => count,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取推送未读数失败: {error}"),
            );
        }
    };

    Json(json!({
        "push": public_push_detail(message),
        "unread_count": unread_count,
    }))
    .into_response()
}

fn public_web_actor(state: &AppState, headers: &HeaderMap) -> Result<ActorIdentity, Response> {
    let user = require_public_user(state, headers)?;
    ActorIdentity::new("web", user.user_id, None::<String>)
        .map_err(|error| crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()))
}

fn backfill_legacy_web_pushes(
    state: &AppState,
    actor: &ActorIdentity,
) -> hone_core::HoneResult<usize> {
    let storage = state.core.cron_job_storage();
    if storage.has_legacy_web_push_messages(actor)? {
        return Ok(0);
    }
    let messages = state
        .core
        .session_storage
        .get_messages(&actor.session_id(), None)?;
    storage.upsert_web_push_messages(actor, legacy_web_push_inputs(&messages))
}

fn legacy_web_push_inputs(
    messages: &[hone_memory::session::SessionMessage],
) -> Vec<WebPushMessageInput> {
    let mut inputs = Vec::new();
    let mut pending_job: Option<(String, String)> = None;

    for (index, message) in messages.iter().enumerate() {
        let content = hone_memory::session_message_text(message);
        let scheduler_source = metadata_string(message, "source").as_deref() == Some("scheduler");
        if message.role == "user" {
            let job_name = metadata_string(message, "job_name")
                .or_else(|| legacy_scheduler_job_name(&content));
            pending_job = if scheduler_source || job_name.is_some() {
                Some((
                    job_name.unwrap_or_else(|| "定时推送".to_string()),
                    metadata_string(message, "job_id").unwrap_or_else(|| "legacy".to_string()),
                ))
            } else {
                None
            };
            continue;
        }

        if message.role != "assistant" || (!scheduler_source && pending_job.is_none()) {
            continue;
        }
        let (pending_name, pending_id) = pending_job
            .take()
            .unwrap_or_else(|| ("定时推送".to_string(), "legacy".to_string()));
        if metadata_string(message, "web_push_id").is_some() || content.trim().is_empty() {
            continue;
        }
        let job_name = metadata_string(message, "job_name").unwrap_or(pending_name);
        let job_id = metadata_string(message, "job_id").unwrap_or(pending_id);
        inputs.push(WebPushMessageInput {
            push_id: legacy_push_id(index, &message.timestamp, &job_name, &content),
            job_id,
            summary: build_web_push_summary(&job_name, &content),
            job_name,
            content,
            created_at: message.timestamp.clone(),
        });
    }
    inputs
}

fn metadata_string(message: &hone_memory::session::SessionMessage, key: &str) -> Option<String> {
    message
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn legacy_push_id(index: usize, timestamp: &str, job_name: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(index.to_be_bytes());
    hasher.update(timestamp.as_bytes());
    hasher.update(job_name.as_bytes());
    hasher.update(content.as_bytes());
    let digest = hasher.finalize();
    let suffix = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("legacy:{suffix}")
}

fn public_push_list_item(message: WebPushMessage) -> PublicPushListItem {
    PublicPushListItem {
        push_id: message.push_id,
        job_id: message.job_id,
        title: message.job_name,
        summary: message.summary,
        created_at: message.created_at,
    }
}

fn public_push_detail(message: WebPushMessage) -> PublicPushDetail {
    PublicPushDetail {
        push_id: message.push_id,
        job_id: message.job_id,
        title: message.job_name,
        summary: message.summary,
        content: message.content,
        created_at: message.created_at,
    }
}

pub(crate) fn build_web_push_summary(job_name: &str, content: &str) -> String {
    let normalized_title = normalize_summary_line(job_name);
    let mut pieces = Vec::new();
    for raw_line in content.lines() {
        let line = normalize_summary_line(raw_line);
        if line.is_empty()
            || line == normalized_title
            || line.contains("定时任务触发")
            || line.starts_with("file://")
            || line.starts_with("附件:")
        {
            continue;
        }
        if pieces.last() == Some(&line) {
            continue;
        }
        pieces.push(line);
        if pieces.len() >= 3 || pieces.join(" · ").chars().count() >= SUMMARY_MAX_CHARS {
            break;
        }
    }
    let summary = pieces.join(" · ");
    if summary.is_empty() {
        return "任务已完成，点击查看完整内容。".to_string();
    }
    truncate_summary(&summary, SUMMARY_MAX_CHARS)
}

fn normalize_summary_line(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("```") {
        return String::new();
    }
    let trimmed = trimmed
        .trim_start_matches(|ch: char| matches!(ch, '#' | '>' | '*' | '-' | '+' | '•'))
        .trim();
    let trimmed = trimmed
        .strip_prefix("【")
        .and_then(|value| value.strip_suffix('】'))
        .unwrap_or(trimmed);
    let trimmed = trimmed.trim_matches(|ch: char| matches!(ch, '*' | '_' | '`'));
    trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_summary(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut output = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    output.push('…');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn summary_skips_repeated_title_and_markdown_noise() {
        let summary = build_web_push_summary(
            "美股收盘复盘",
            "# 美股收盘复盘\n\n**核心结论**\n- 纳指收高，半导体领涨。\n- 风险偏好回升。",
        );

        assert_eq!(
            summary,
            "核心结论 · 纳指收高，半导体领涨。 · 风险偏好回升。"
        );
        assert!(!summary.contains("美股收盘复盘"));
    }

    #[test]
    fn summary_is_bounded_and_has_empty_fallback() {
        assert_eq!(
            build_web_push_summary("任务", "\n```json\n```\n"),
            "任务已完成，点击查看完整内容。"
        );
        let summary = build_web_push_summary("任务", &"长内容".repeat(100));
        assert!(summary.chars().count() <= SUMMARY_MAX_CHARS);
        assert!(summary.ends_with('…'));
    }

    #[test]
    fn legacy_push_inputs_are_deterministic_and_skip_current_pushes() {
        let trigger = hone_memory::session_message_from_text(
            "user",
            "[定时任务触发] 任务名称：每日复盘。",
            "2026-07-09T20:00:00+08:00",
            None,
        );
        let answer = hone_memory::session_message_from_text(
            "assistant",
            "# 每日复盘\n市场风险偏好回升。",
            "2026-07-09T20:01:00+08:00",
            None,
        );
        let current_metadata = HashMap::from([
            ("source".to_string(), json!("scheduler")),
            ("job_name".to_string(), json!("新任务")),
            ("web_push_id".to_string(), json!("current:1")),
        ]);
        let current = hone_memory::session_message_from_text(
            "assistant",
            "当前任务内容",
            "2026-07-10T20:01:00+08:00",
            Some(current_metadata),
        );
        let messages = vec![trigger, answer, current];

        let first = legacy_web_push_inputs(&messages);
        let second = legacy_web_push_inputs(&messages);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].push_id, second[0].push_id);
        assert!(first[0].push_id.starts_with("legacy:"));
        assert_eq!(first[0].job_name, "每日复盘");
        assert_eq!(first[0].created_at, "2026-07-09T20:01:00+08:00");
    }
}
