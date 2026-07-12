use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, header};
use axum::response::IntoResponse;
use chrono::NaiveDate;
use serde_json::json;

use hone_channels::outbound::collect_local_image_markers;
use hone_memory::{
    message_is_compact_boundary, message_is_compact_skill_snapshot, message_is_compact_summary,
    select_messages_after_compact_boundary, session_message_text,
};

use crate::routes::public_pushes::build_web_push_summary;
use crate::routes::require_actor;
use crate::state::AppState;
use crate::types::{
    HistoryAttachment, HistoryFinanceCalendar, HistoryMsg, HistoryScheduledPush, UserIdQuery,
};

const FINANCE_CALENDAR_METADATA_KEY: &str = "finance_calendar";

/// GET /api/history?user_id=...
pub(crate) async fn handle_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let session_id = if let Some(session_id) = params
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        session_id.to_string()
    } else {
        let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
            Ok(actor) => actor,
            Err(error) => return error,
        };
        actor.session_id()
    };
    let messages = state
        .core
        .session_storage
        .get_messages(&session_id, None)
        .unwrap_or_default();

    let history = history_from_messages(&messages);

    Json(json!({ "messages": history })).into_response()
}

pub(crate) fn history_from_messages(
    messages: &[hone_memory::session::SessionMessage],
) -> Vec<HistoryMsg> {
    select_messages_after_compact_boundary(messages, Some(50))
        .into_iter()
        .filter(|m| {
            matches!(m.role.as_str(), "user" | "assistant")
                || message_is_compact_boundary(m.metadata.as_ref())
                || message_is_compact_summary(m.metadata.as_ref())
                || message_is_compact_skill_snapshot(m.metadata.as_ref())
        })
        .map(|message| plain_history_message(message, false))
        .collect()
}

#[cfg(test)]
pub(crate) fn public_history_from_messages(
    messages: &[hone_memory::session::SessionMessage],
) -> Vec<HistoryMsg> {
    project_public_history(messages, false)
}

pub(crate) struct PublicHistoryPage {
    pub messages: Vec<HistoryMsg>,
    pub start: usize,
    pub next_before: Option<usize>,
}

#[cfg(test)]
pub(crate) fn public_history_page_from_messages(
    messages: &[hone_memory::session::SessionMessage],
    before: Option<usize>,
    limit: usize,
) -> PublicHistoryPage {
    public_history_page_for_client(messages, before, limit, false)
}

pub(crate) fn public_history_page_for_client(
    messages: &[hone_memory::session::SessionMessage],
    before: Option<usize>,
    limit: usize,
    prefer_mobile: bool,
) -> PublicHistoryPage {
    let projected = project_public_history(messages, prefer_mobile);
    let end = before.unwrap_or(projected.len()).min(projected.len());
    let start = end.saturating_sub(limit.clamp(1, 50));
    let messages = projected
        .into_iter()
        .skip(start)
        .take(end - start)
        .collect();
    PublicHistoryPage {
        messages,
        start,
        next_before: (start > 0).then_some(start),
    }
}

fn project_public_history(
    messages: &[hone_memory::session::SessionMessage],
    prefer_mobile: bool,
) -> Vec<HistoryMsg> {
    let mut history = Vec::new();
    let mut legacy_job_name: Option<String> = None;
    for message in select_messages_after_compact_boundary(messages, None) {
        let content = session_message_text(message);
        let scheduler_source = message
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("source"))
            .and_then(serde_json::Value::as_str)
            == Some("scheduler");
        if message.role == "user" {
            if scheduler_source {
                legacy_job_name = message
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get("job_name"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string);
                continue;
            }
            if let Some(job_name) = legacy_scheduler_job_name(&content) {
                legacy_job_name = Some(job_name);
                continue;
            }
            legacy_job_name = None;
        }

        if message.role == "assistant" && (scheduler_source || legacy_job_name.is_some()) {
            let job_name = message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("job_name"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .or_else(|| legacy_job_name.take())
                .unwrap_or_else(|| "定时推送".to_string());
            let push_id = message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("web_push_id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            let summary = build_web_push_summary(&job_name, &content);
            history.push(HistoryMsg {
                role: "assistant".to_string(),
                content: String::new(),
                subtype: Some("scheduled_push".to_string()),
                synthetic: false,
                transcript_only: false,
                attachments: Vec::new(),
                scheduled_push: Some(HistoryScheduledPush {
                    fallback_content: push_id.is_none().then_some(content),
                    push_id,
                    title: job_name,
                    summary,
                }),
                finance_calendar: None,
            });
            continue;
        }

        legacy_job_name = None;
        history.push(plain_history_message(message, prefer_mobile));
    }
    history
}

fn plain_history_message(
    message: &hone_memory::session::SessionMessage,
    prefer_mobile: bool,
) -> HistoryMsg {
    let compact_boundary = message_is_compact_boundary(message.metadata.as_ref());
    let compact_summary = message_is_compact_summary(message.metadata.as_ref());
    let compact_skill_snapshot = message_is_compact_skill_snapshot(message.metadata.as_ref());
    HistoryMsg {
        attachments: extract_history_attachments(&session_message_text(message)),
        role: if compact_boundary {
            "system".to_string()
        } else {
            message.role.clone()
        },
        content: session_message_text(message),
        subtype: if compact_boundary {
            Some("compact_boundary".to_string())
        } else if compact_summary {
            Some("compact_summary".to_string())
        } else if compact_skill_snapshot {
            Some("compact_skill_snapshot".to_string())
        } else {
            None
        },
        synthetic: compact_boundary || compact_summary || compact_skill_snapshot,
        transcript_only: compact_summary || compact_skill_snapshot,
        scheduled_push: None,
        finance_calendar: history_finance_calendar(message, prefer_mobile),
    }
}

pub(crate) fn public_client_prefers_mobile(headers: &HeaderMap) -> bool {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    ["iphone", "ipad", "ipod", "android", "mobile"]
        .iter()
        .any(|needle| user_agent.contains(needle))
}

fn history_finance_calendar(
    message: &hone_memory::session::SessionMessage,
    prefer_mobile: bool,
) -> Option<HistoryFinanceCalendar> {
    if let Some(value) = message
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(FINANCE_CALENDAR_METADATA_KEY))
    {
        let desktop_path = value.get("desktop_path")?.as_str()?.trim();
        let mobile_path = value
            .get("mobile_path")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty());
        let month = value.get("month")?.as_str()?.trim();
        if desktop_path.is_empty() || month.is_empty() {
            return None;
        }
        let (image_path, variant) = if prefer_mobile && mobile_path.is_some() {
            (mobile_path.unwrap_or(desktop_path), "mobile")
        } else {
            (desktop_path, "desktop")
        };
        return Some(HistoryFinanceCalendar {
            month: month.to_string(),
            image_path: image_path.to_string(),
            variant: variant.to_string(),
        });
    }

    let content = session_message_text(message);
    let month = legacy_finance_calendar_month(&content)?;
    let markers = collect_local_image_markers(&content);
    let desktop_path = markers.first()?.path.as_str();
    let mobile_path = markers.get(1).map(|marker| marker.path.as_str());
    let (image_path, variant) = if prefer_mobile && mobile_path.is_some() {
        (mobile_path.unwrap_or(desktop_path), "mobile")
    } else {
        (desktop_path, "desktop")
    };
    Some(HistoryFinanceCalendar {
        month,
        image_path: image_path.to_string(),
        variant: variant.to_string(),
    })
}

fn legacy_finance_calendar_month(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    if !content.contains("财经日历") && !lower.contains("finance calendar") {
        return None;
    }
    content
        .split(|character: char| !(character.is_ascii_digit() || character == '-'))
        .find(|candidate| {
            candidate.len() == 7
                && NaiveDate::parse_from_str(&format!("{candidate}-01"), "%Y-%m-%d").is_ok()
        })
        .map(str::to_string)
}

pub(crate) fn legacy_scheduler_job_name(content: &str) -> Option<String> {
    let first_line = content.lines().next()?.trim();
    let value = first_line
        .strip_prefix("[定时任务触发] 任务名称：")
        .or_else(|| first_line.strip_prefix("[定时任务触发] 任务名称:"))?;
    let job_name = value.trim().trim_end_matches('。').trim();
    (!job_name.is_empty()).then(|| job_name.to_string())
}

fn extract_history_attachments(content: &str) -> Vec<HistoryAttachment> {
    let mut attachments = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in content.lines() {
        let Some(path) = line.strip_prefix("[附件: ") else {
            continue;
        };
        let Some(path) = path.strip_suffix(']') else {
            continue;
        };
        if seen.insert(path.to_string()) {
            attachments.push(build_history_attachment(path));
        }
    }

    for marker in collect_local_image_markers(content) {
        if seen.insert(marker.path.clone()) {
            attachments.push(build_history_attachment(&marker.path));
        }
    }

    attachments
}

fn build_history_attachment(path: &str) -> HistoryAttachment {
    let clean_path = path.strip_prefix("file://").unwrap_or(path);
    let filename = std::path::Path::new(clean_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "attachment".to_string());
    let lower = filename.to_ascii_lowercase();
    let kind = if lower.ends_with(".pdf") {
        "pdf"
    } else if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
    {
        "image"
    } else {
        "file"
    };

    HistoryAttachment {
        path: clean_path.to_string(),
        name: filename,
        kind: kind.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::http::{HeaderMap, HeaderValue, header};

    use super::{
        extract_history_attachments, public_client_prefers_mobile, public_history_from_messages,
        public_history_page_for_client, public_history_page_from_messages,
    };

    #[test]
    fn history_attachments_include_inline_local_images() {
        let attachments = extract_history_attachments("结论如下\nfile:///tmp/chart.png\n后续说明");

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].path, "/tmp/chart.png");
        assert_eq!(attachments[0].kind, "image");
    }

    #[test]
    fn history_attachments_deduplicate_between_attachment_lines_and_inline_images() {
        let attachments =
            extract_history_attachments("[附件: /tmp/chart.png]\nfile:///tmp/chart.png");

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].name, "chart.png");
    }

    #[test]
    fn history_attachments_include_html_anchor_local_images() {
        let attachments = extract_history_attachments(
            "图如下\n<a href=\"file:///tmp/chart.png\">file:///tmp/chart.png</a>",
        );

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].path, "/tmp/chart.png");
        assert_eq!(attachments[0].kind, "image");
    }

    #[test]
    fn public_history_selects_persisted_calendar_variant_for_device() {
        let metadata = HashMap::from([(
            "finance_calendar".to_string(),
            serde_json::json!({
                "month": "2026-07",
                "desktop_path": "/tmp/calendar.png",
                "mobile_path": "/tmp/calendar-mobile-v4.png",
            }),
        )]);
        let messages = vec![hone_memory::session_message_from_text(
            "assistant",
            "这是你的 2026-07 财经日历：\n\nfile:///tmp/calendar.png\n\nfile:///tmp/calendar-mobile-v4.png",
            "2026-07-12T10:00:00+08:00",
            Some(metadata),
        )];

        let desktop = public_history_page_for_client(&messages, None, 20, false);
        let mobile = public_history_page_for_client(&messages, None, 20, true);
        let desktop_calendar = desktop.messages[0]
            .finance_calendar
            .as_ref()
            .expect("desktop calendar");
        let mobile_calendar = mobile.messages[0]
            .finance_calendar
            .as_ref()
            .expect("mobile calendar");

        assert_eq!(desktop_calendar.image_path, "/tmp/calendar.png");
        assert_eq!(desktop_calendar.variant, "desktop");
        assert_eq!(mobile_calendar.image_path, "/tmp/calendar-mobile-v4.png");
        assert_eq!(mobile_calendar.variant, "mobile");
    }

    #[test]
    fn public_history_selects_legacy_calendar_markers_without_client_rendering() {
        let messages = vec![hone_memory::session_message_from_text(
            "assistant",
            "这是你的 2026-07 财经日历：\n\nfile:///tmp/calendar.png\n\nfile:///tmp/calendar-mobile-v3.png",
            "2026-07-12T10:00:00+08:00",
            None,
        )];

        let mobile = public_history_page_for_client(&messages, None, 20, true);
        let calendar = mobile.messages[0]
            .finance_calendar
            .as_ref()
            .expect("legacy calendar");

        assert_eq!(calendar.month, "2026-07");
        assert_eq!(calendar.image_path, "/tmp/calendar-mobile-v3.png");
        assert_eq!(calendar.variant, "mobile");
    }

    #[test]
    fn public_client_device_detection_uses_user_agent() {
        let mut mobile = HeaderMap::new();
        mobile.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 18_5) Mobile"),
        );
        let mut desktop = HeaderMap::new();
        desktop.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"),
        );

        assert!(public_client_prefers_mobile(&mobile));
        assert!(!public_client_prefers_mobile(&desktop));
    }

    #[test]
    fn public_history_projects_scheduler_turn_to_card_and_hides_trigger() {
        let metadata = HashMap::from([
            ("source".to_string(), serde_json::json!("scheduler")),
            ("job_name".to_string(), serde_json::json!("收盘复盘")),
            (
                "web_push_id".to_string(),
                serde_json::json!("job-1:2026-07-10:20:00"),
            ),
        ]);
        let messages = vec![
            hone_memory::session_message_from_text(
                "user",
                "[定时任务触发] 任务名称：收盘复盘",
                "2026-07-10T20:00:00+08:00",
                Some(metadata.clone()),
            ),
            hone_memory::session_message_from_text(
                "assistant",
                "# 收盘复盘\n核心结论\n市场风险偏好回升。",
                "2026-07-10T20:01:00+08:00",
                Some(metadata),
            ),
        ];

        let history = public_history_from_messages(&messages);
        assert_eq!(history.len(), 1);
        let push = history[0].scheduled_push.as_ref().expect("push card");
        assert_eq!(push.title, "收盘复盘");
        assert_eq!(push.summary, "核心结论 · 市场风险偏好回升。");
        assert_eq!(push.push_id.as_deref(), Some("job-1:2026-07-10:20:00"));
        assert!(push.fallback_content.is_none());
        assert!(history[0].content.is_empty());
    }

    #[test]
    fn public_history_projects_legacy_scheduler_pair_with_local_fallback() {
        let messages = vec![
            hone_memory::session_message_from_text(
                "user",
                "[定时任务触发] 任务名称：盘前快报\n权威触发配置：daily",
                "2026-07-10T08:00:00+08:00",
                None,
            ),
            hone_memory::session_message_from_text(
                "assistant",
                "盘前重点：留意 CPI。",
                "2026-07-10T08:01:00+08:00",
                None,
            ),
        ];

        let history = public_history_from_messages(&messages);
        assert_eq!(history.len(), 1);
        let push = history[0].scheduled_push.as_ref().expect("push card");
        assert_eq!(push.title, "盘前快报");
        assert!(push.push_id.is_none());
        assert_eq!(
            push.fallback_content.as_deref(),
            Some("盘前重点：留意 CPI。")
        );
    }

    #[test]
    fn public_history_pages_from_the_newest_projected_messages() {
        let messages = (0..45)
            .map(|index| {
                hone_memory::session_message_from_text(
                    if index % 2 == 0 { "user" } else { "assistant" },
                    &format!("message-{index}"),
                    "2026-07-12T10:00:00+08:00",
                    None,
                )
            })
            .collect::<Vec<_>>();

        let latest = public_history_page_from_messages(&messages, None, 20);
        assert_eq!(latest.start, 25);
        assert_eq!(latest.next_before, Some(25));
        assert_eq!(latest.messages.len(), 20);
        assert_eq!(latest.messages[0].content, "message-25");
        assert_eq!(latest.messages[19].content, "message-44");

        let older = public_history_page_from_messages(&messages, latest.next_before, 20);
        assert_eq!(older.start, 5);
        assert_eq!(older.next_before, Some(5));
        assert_eq!(older.messages[0].content, "message-5");
        assert_eq!(older.messages[19].content, "message-24");

        let oldest = public_history_page_from_messages(&messages, older.next_before, 20);
        assert_eq!(oldest.start, 0);
        assert_eq!(oldest.next_before, None);
        assert_eq!(oldest.messages.len(), 5);
    }
}
