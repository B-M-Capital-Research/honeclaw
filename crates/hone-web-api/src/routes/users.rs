use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use hone_core::cloud_runtime::{CloudPgRuntime, CloudSessionListEntry};
use hone_core::{ActorIdentity, SessionIdentity};
use hone_memory::session::SessionMessage;
use hone_memory::session_message_text;
use tracing::warn;

use crate::state::AppState;
use crate::types::UserInfo;

#[derive(Default)]
struct UsersRouteCache {
    users: Vec<UserInfo>,
    updated_at: Option<Instant>,
    refreshing: bool,
}

static USERS_ROUTE_CACHE: LazyLock<Mutex<UsersRouteCache>> =
    LazyLock::new(|| Mutex::new(UsersRouteCache::default()));
const USERS_ROUTE_CACHE_TTL: Duration = Duration::from_secs(30);

/// GET /api/users — 列出所有有会话记录的 session，按最后活跃时间降序
pub(crate) async fn handle_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state
        .core
        .config
        .cloud
        .effective_mode()
        .is_cloud_authoritative()
        && let Some(postgres) = CloudPgRuntime::from_cloud_config(&state.core.config.cloud)
    {
        if let Some(cached) = cached_cloud_users(false) {
            return Json(serde_json::to_value(&cached).unwrap_or(serde_json::json!([])));
        }
        if !mark_cloud_users_refreshing() {
            let cached = cached_cloud_users(true).unwrap_or_default();
            return Json(serde_json::to_value(&cached).unwrap_or(serde_json::json!([])));
        }
        let summaries_result =
            tokio::time::timeout(Duration::from_secs(8), postgres.list_session_summaries()).await;
        let summaries = match summaries_result {
            Ok(Ok(summaries)) => summaries,
            Ok(Err(error)) => {
                warn!(%error, "failed to list cloud session summaries for users route");
                clear_cloud_users_refreshing();
                return Json(serde_json::json!([]));
            }
            Err(_) => {
                warn!("cloud session summary list timed out for users route");
                clear_cloud_users_refreshing();
                return Json(serde_json::json!([]));
            }
        };
        let users = summaries
            .into_iter()
            .filter_map(user_info_from_cloud_summary)
            .collect::<Vec<_>>();
        update_cloud_users_cache(users.clone());
        return Json(serde_json::to_value(&users).unwrap_or(serde_json::json!([])));
    }

    let mut users: Vec<UserInfo> = Vec::new();
    let core = state.core.clone();
    let sessions_result = tokio::time::timeout(
        Duration::from_secs(8),
        tokio::task::spawn_blocking(move || core.session_storage.list_sessions()),
    )
    .await;
    let sessions = match sessions_result {
        Ok(Ok(Ok(sessions))) => sessions,
        Ok(Ok(Err(error))) => {
            warn!(%error, "failed to list sessions for users route");
            return Json(serde_json::json!([]));
        }
        Ok(Err(error)) => {
            warn!(%error, "users route session list worker failed");
            return Json(serde_json::json!([]));
        }
        Err(_) => {
            warn!("users route session list timed out");
            return Json(serde_json::json!([]));
        }
    };

    for session in sessions {
        let session_id = session.id.clone();
        let actor = session
            .actor
            .clone()
            .or_else(|| hone_core::ActorIdentity::from_session_id(&session_id));
        let session_identity = session
            .session_identity
            .clone()
            .or_else(|| {
                actor
                    .as_ref()
                    .and_then(|value| hone_core::SessionIdentity::from_actor(value).ok())
            })
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session_id));
        let Some(identity) = session_identity else {
            continue;
        };

        // 取最后一条 user 或 assistant 消息作为预览
        let last_msg = session
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role.as_str(), "user" | "assistant"));

        let (last_message, last_role, last_time) = match last_msg {
            Some(m) => {
                let content = session_message_text(m);
                let preview: String = content.chars().take(60).collect();
                let preview = if content.chars().count() > 60 {
                    format!("{}…", preview)
                } else {
                    preview
                };
                (preview, m.role.clone(), m.timestamp.clone())
            }
            None => (
                "暂无消息".to_string(),
                "".to_string(),
                session.updated_at.clone(),
            ),
        };

        let message_count = session
            .messages
            .iter()
            .filter(|m| matches!(m.role.as_str(), "user" | "assistant"))
            .count();

        users.push(UserInfo {
            channel: identity.channel.clone(),
            user_id: actor
                .as_ref()
                .map(|value| value.user_id.clone())
                .or_else(|| identity.user_id.clone())
                .unwrap_or_else(|| "group".to_string()),
            channel_scope: identity.channel_scope.clone(),
            session_id,
            session_kind: if identity.is_group() {
                "group".to_string()
            } else {
                "direct".to_string()
            },
            session_label: if identity.is_group() {
                identity
                    .channel_scope
                    .clone()
                    .unwrap_or_else(|| "群聊".to_string())
            } else {
                actor
                    .as_ref()
                    .map(|value| value.user_id.clone())
                    .or_else(|| identity.user_id.clone())
                    .unwrap_or_else(|| "direct".to_string())
            },
            actor_user_id: actor.as_ref().map(|value| value.user_id.clone()),
            last_message,
            last_role,
            last_time,
            message_count,
        });
    }

    // 按最后时间降序
    users.sort_by(|a, b| b.last_time.cmp(&a.last_time));

    Json(serde_json::to_value(&users).unwrap_or(serde_json::json!([])))
}

fn cached_cloud_users(allow_stale: bool) -> Option<Vec<UserInfo>> {
    let cache = USERS_ROUTE_CACHE.lock().ok()?;
    if cache.users.is_empty() {
        return None;
    }
    if allow_stale
        || cache
            .updated_at
            .map(|updated_at| updated_at.elapsed() < USERS_ROUTE_CACHE_TTL)
            .unwrap_or(false)
    {
        return Some(cache.users.clone());
    }
    None
}

fn mark_cloud_users_refreshing() -> bool {
    let Ok(mut cache) = USERS_ROUTE_CACHE.lock() else {
        return true;
    };
    if cache.refreshing {
        return false;
    }
    cache.refreshing = true;
    true
}

fn clear_cloud_users_refreshing() {
    if let Ok(mut cache) = USERS_ROUTE_CACHE.lock() {
        cache.refreshing = false;
    }
}

fn update_cloud_users_cache(users: Vec<UserInfo>) {
    if let Ok(mut cache) = USERS_ROUTE_CACHE.lock() {
        cache.users = users;
        cache.updated_at = Some(Instant::now());
        cache.refreshing = false;
    }
}

fn user_info_from_cloud_summary(summary: CloudSessionListEntry) -> Option<UserInfo> {
    let actor = summary
        .actor
        .and_then(|value| serde_json::from_value::<ActorIdentity>(value).ok())
        .or_else(|| ActorIdentity::from_session_id(&summary.session_id));
    let session_identity = summary
        .session_identity
        .and_then(|value| serde_json::from_value::<SessionIdentity>(value).ok())
        .or_else(|| {
            actor
                .as_ref()
                .and_then(|value| SessionIdentity::from_actor(value).ok())
        })
        .or_else(|| SessionIdentity::from_session_id(&summary.session_id))?;

    let last_message = summary
        .last_message
        .and_then(|value| serde_json::from_value::<SessionMessage>(value).ok());
    let (last_message, last_role, last_time) = match last_message {
        Some(message) => {
            let content = session_message_text(&message);
            let preview: String = content.chars().take(60).collect();
            let preview = if content.chars().count() > 60 {
                format!("{}…", preview)
            } else {
                preview
            };
            (preview, message.role, message.timestamp)
        }
        None => (
            "暂无消息".to_string(),
            "".to_string(),
            summary.updated_at.clone(),
        ),
    };

    Some(UserInfo {
        channel: session_identity.channel.clone(),
        user_id: actor
            .as_ref()
            .map(|value| value.user_id.clone())
            .or_else(|| session_identity.user_id.clone())
            .unwrap_or_else(|| "group".to_string()),
        channel_scope: session_identity.channel_scope.clone(),
        session_id: summary.session_id,
        session_kind: if session_identity.is_group() {
            "group".to_string()
        } else {
            "direct".to_string()
        },
        session_label: if session_identity.is_group() {
            session_identity
                .channel_scope
                .clone()
                .unwrap_or_else(|| "群聊".to_string())
        } else {
            actor
                .as_ref()
                .map(|value| value.user_id.clone())
                .or_else(|| session_identity.user_id.clone())
                .unwrap_or_else(|| "direct".to_string())
        },
        actor_user_id: actor.as_ref().map(|value| value.user_id.clone()),
        last_message,
        last_role,
        last_time,
        message_count: summary.message_count,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hone_memory::session::Session;

    fn empty_session(id: &str) -> Session {
        Session {
            version: 4,
            id: id.to_string(),
            actor: None,
            session_identity: None,
            created_at: "2026-04-16T09:00:00+08:00".to_string(),
            updated_at: "2026-04-16T09:05:00+08:00".to_string(),
            messages: Vec::new(),
            metadata: HashMap::new(),
            runtime: Default::default(),
            summary: None,
        }
    }

    #[test]
    fn actor_session_id_is_enough_for_listing_identity() {
        let session = empty_session("Actor_feishu__direct__ou_5f123");
        let actor = session
            .actor
            .clone()
            .or_else(|| hone_core::ActorIdentity::from_session_id(&session.id))
            .expect("actor");
        let identity = session
            .session_identity
            .clone()
            .or_else(|| hone_core::SessionIdentity::from_actor(&actor).ok())
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session.id))
            .expect("identity");

        assert_eq!(actor.channel, "feishu");
        assert_eq!(actor.user_id, "ou_123");
        assert_eq!(identity.channel, "feishu");
        assert_eq!(identity.user_id.as_deref(), Some("ou_123"));
        assert_eq!(identity.channel_scope, None);
    }

    #[test]
    fn shared_group_session_id_is_enough_for_listing_identity() {
        let session = empty_session("Session_discord__group__g_3a1_3ac_3a2");
        let identity = session
            .session_identity
            .clone()
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session.id))
            .expect("identity");

        assert_eq!(identity.channel, "discord");
        assert_eq!(identity.channel_scope.as_deref(), Some("g:1:c:2"));
        assert!(identity.is_group());
    }
}
