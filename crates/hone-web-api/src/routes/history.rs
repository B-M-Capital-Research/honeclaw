use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde_json::json;

use hone_channels::outbound::collect_local_image_markers;
use hone_memory::{
    message_is_compact_boundary, message_is_compact_skill_snapshot, message_is_compact_summary,
    select_messages_after_compact_boundary, session_message_text,
};

use crate::routes::require_actor;
use crate::state::AppState;
use crate::types::{HistoryAttachment, HistoryMsg, UserIdQuery};

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
        .map(|m| HistoryMsg {
            attachments: extract_history_attachments(&session_message_text(m)),
            role: if message_is_compact_boundary(m.metadata.as_ref()) {
                "system".to_string()
            } else {
                m.role.clone()
            },
            content: session_message_text(m),
            subtype: if message_is_compact_boundary(m.metadata.as_ref()) {
                Some("compact_boundary".to_string())
            } else if message_is_compact_summary(m.metadata.as_ref()) {
                Some("compact_summary".to_string())
            } else if message_is_compact_skill_snapshot(m.metadata.as_ref()) {
                Some("compact_skill_snapshot".to_string())
            } else {
                None
            },
            synthetic: message_is_compact_boundary(m.metadata.as_ref())
                || message_is_compact_summary(m.metadata.as_ref())
                || message_is_compact_skill_snapshot(m.metadata.as_ref()),
            transcript_only: message_is_compact_summary(m.metadata.as_ref())
                || message_is_compact_skill_snapshot(m.metadata.as_ref()),
        })
        .collect()
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
    use super::extract_history_attachments;

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
}
