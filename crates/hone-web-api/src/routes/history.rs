use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde_json::json;

use hone_memory::{
    message_is_compact_boundary, message_is_compact_skill_snapshot, message_is_compact_summary,
    select_messages_after_compact_boundary,
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

    let history: Vec<HistoryMsg> = select_messages_after_compact_boundary(&messages, Some(50))
        .into_iter()
        .filter(|m| {
            matches!(m.role.as_str(), "user" | "assistant")
                || message_is_compact_boundary(m.metadata.as_ref())
                || message_is_compact_summary(m.metadata.as_ref())
                || message_is_compact_skill_snapshot(m.metadata.as_ref())
        })
        .map(|m| HistoryMsg {
            attachments: extract_history_attachments(&m.content),
            role: if message_is_compact_boundary(m.metadata.as_ref()) {
                "system".to_string()
            } else {
                m.role.clone()
            },
            content: m.content.clone(),
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
        .collect();

    Json(json!({ "messages": history })).into_response()
}

fn extract_history_attachments(content: &str) -> Vec<HistoryAttachment> {
    let mut attachments = Vec::new();
    for line in content.lines() {
        let Some(path) = line.strip_prefix("[附件: ") else {
            continue;
        };
        let Some(path) = path.strip_suffix(']') else {
            continue;
        };
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "attachment".to_string());
        let kind = if filename.ends_with(".pdf") {
            "pdf"
        } else if filename.ends_with(".png")
            || filename.ends_with(".jpg")
            || filename.ends_with(".jpeg")
        {
            "image"
        } else {
            "file"
        };
        attachments.push(HistoryAttachment {
            path: path.to_string(),
            name: filename,
            kind: kind.to_string(),
        });
    }
    attachments
}
