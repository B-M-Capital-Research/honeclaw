use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use tracing::warn;

use hone_core::ActorIdentity;
use hone_core::cloud_runtime::CloudPgRuntime;

use crate::state::AppState;
use crate::types::{CreateWebInviteRequest, WebInviteInfo};

const WEB_INVITES_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
struct WebInvitesCache {
    invites: Vec<WebInviteInfo>,
    updated_at: Option<Instant>,
    refreshing: bool,
}

static WEB_INVITES_CACHE: LazyLock<Mutex<WebInvitesCache>> =
    LazyLock::new(|| Mutex::new(WebInvitesCache::default()));

pub(crate) async fn handle_list_invites(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = cached_web_invites(false) {
        return Json(json!({ "invites": cached }));
    }
    if !mark_web_invites_refreshing() {
        return Json(json!({ "invites": cached_web_invites(true).unwrap_or_default() }));
    }

    if state
        .core
        .config
        .cloud
        .effective_mode()
        .is_cloud_authoritative()
        && let Some(postgres) = CloudPgRuntime::from_cloud_config(&state.core.config.cloud)
    {
        let invites_result = tokio::time::timeout(
            Duration::from_secs(8),
            postgres.list_web_invite_user_records_cached(),
        )
        .await;
        let invites = match invites_result {
            Ok(Ok(records)) => records
                .into_iter()
                .filter_map(|value| {
                    serde_json::from_value::<hone_memory::WebInviteUser>(value).ok()
                })
                .map(|invite| to_invite_info_summary(&state, invite))
                .collect::<Vec<_>>(),
            Ok(Err(error)) => {
                warn!(%error, "failed to list cloud web invite users");
                Vec::new()
            }
            Err(_) => {
                warn!("cloud web invite list timed out");
                Vec::new()
            }
        };
        update_web_invites_cache(invites.clone());
        return Json(json!({ "invites": invites }));
    }

    let state_for_worker = state.clone();
    let invites_result = tokio::time::timeout(
        Duration::from_secs(8),
        tokio::task::spawn_blocking(move || {
            state_for_worker
                .web_auth
                .list_invite_users()
                .map(|invites| {
                    invites
                        .into_iter()
                        .map(|invite| to_invite_info_summary(&state_for_worker, invite))
                        .collect::<Vec<_>>()
                })
        }),
    )
    .await;
    let invites = match invites_result {
        Ok(Ok(Ok(invites))) => invites,
        Ok(Ok(Err(error))) => {
            warn!(%error, "failed to list web invite users");
            Vec::new()
        }
        Ok(Err(error)) => {
            warn!(%error, "web invite list worker failed");
            Vec::new()
        }
        Err(_) => {
            warn!("web invite list timed out");
            Vec::new()
        }
    };

    update_web_invites_cache(invites.clone());
    Json(json!({ "invites": invites }))
}

pub(crate) async fn handle_create_invite(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateWebInviteRequest>,
) -> impl IntoResponse {
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(phone_number) => phone_number,
        Err(response) => return response,
    };

    match state.web_auth.create_invite_user(&phone_number) {
        Ok(invite) => {
            clear_web_invites_cache();
            let user_id = invite.user_id.clone();
            Json(json!({ "invite": to_invite_info(&state, &user_id, invite) })).into_response()
        }
        Err(error) if error.to_string().contains("手机号格式不合法") => {
            crate::routes::json_error(StatusCode::BAD_REQUEST, "手机号格式不合法")
        }
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("生成邀请码失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_disable_invite(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.web_auth.set_invite_revoked(&user_id, true) {
        Ok(Some(result)) => {
            clear_web_invites_cache();
            Json(json!({
                "invite": to_invite_info(&state, &user_id, result.invite),
                "cleared_session_count": result.cleared_session_count,
                "message": format!("已停用邀请码，并清理 {} 个登录态", result.cleared_session_count),
            }))
            .into_response()
        }
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("停用邀请码失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_enable_invite(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.web_auth.set_invite_revoked(&user_id, false) {
        Ok(Some(result)) => {
            clear_web_invites_cache();
            Json(json!({
                "invite": to_invite_info(&state, &user_id, result.invite),
                "cleared_session_count": result.cleared_session_count,
                "message": "已重新启用邀请码",
            }))
            .into_response()
        }
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("启用邀请码失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_reset_invite(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.web_auth.reset_invite_code(&user_id) {
        Ok(Some(result)) => {
            clear_web_invites_cache();
            Json(json!({
                "invite": to_invite_info(&state, &user_id, result.invite),
                "cleared_session_count": result.cleared_session_count,
                "message": format!("已重置邀请码，并清理 {} 个登录态", result.cleared_session_count),
            }))
            .into_response()
        }
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("重置邀请码失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_get_api_key(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.web_auth.ensure_api_key_for_user(&user_id) {
        Ok(Some(invite)) => {
            clear_web_invites_cache();
            Json(json!({
                "invite": to_invite_info(&state, &user_id, invite),
                "message": "已获取 API Key；明文仅显示一次，请妥善保存",
            }))
            .into_response()
        }
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("获取 API Key 失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_reset_api_key(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.web_auth.reset_api_key_for_user(&user_id) {
        Ok(Some(invite)) => {
            clear_web_invites_cache();
            Json(json!({
                "invite": to_invite_info(&state, &user_id, invite),
                "message": "已重置 API Key；旧 Key 已失效，新 Key 明文仅显示一次",
            }))
            .into_response()
        }
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("重置 API Key 失败: {error}"),
        ),
    }
}

fn cached_web_invites(allow_stale: bool) -> Option<Vec<WebInviteInfo>> {
    let guard = WEB_INVITES_CACHE.lock().ok()?;
    let updated_at = guard.updated_at?;
    if allow_stale || updated_at.elapsed() < WEB_INVITES_CACHE_TTL {
        return Some(guard.invites.clone());
    }
    None
}

fn mark_web_invites_refreshing() -> bool {
    let Ok(mut guard) = WEB_INVITES_CACHE.lock() else {
        return true;
    };
    if guard.refreshing {
        return false;
    }
    guard.refreshing = true;
    true
}

fn update_web_invites_cache(invites: Vec<WebInviteInfo>) {
    if let Ok(mut guard) = WEB_INVITES_CACHE.lock() {
        guard.invites = invites;
        guard.updated_at = Some(Instant::now());
        guard.refreshing = false;
    }
}

fn clear_web_invites_cache() {
    if let Ok(mut guard) = WEB_INVITES_CACHE.lock() {
        guard.invites.clear();
        guard.updated_at = None;
        guard.refreshing = false;
    }
}

fn to_invite_info(
    state: &AppState,
    user_id: &str,
    invite: hone_memory::WebInviteUser,
) -> WebInviteInfo {
    let actor = ActorIdentity::new("web", user_id, Option::<String>::None).ok();
    let daily_limit = state.core.config.agent.daily_conversation_limit;
    let quota_date = hone_core::beijing_now().format("%F").to_string();
    let snapshot = actor.as_ref().and_then(|actor| {
        state
            .core
            .conversation_quota_storage
            .snapshot_for_date(actor, &quota_date)
            .ok()
            .flatten()
    });
    let success_count = snapshot
        .as_ref()
        .map(|value| value.success_count)
        .unwrap_or(0);
    let in_flight = snapshot.as_ref().map(|value| value.in_flight).unwrap_or(0);
    let remaining_today = if daily_limit == 0 {
        0
    } else {
        daily_limit.saturating_sub(success_count.saturating_add(in_flight))
    };
    let active_session_count = state
        .web_auth
        .count_active_sessions_for_user(user_id)
        .unwrap_or(0);
    let enabled = invite.revoked_at.is_none();

    WebInviteInfo {
        user_id: invite.user_id,
        invite_code: invite.invite_code,
        phone_number: invite.phone_number,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        revoked_at: invite.revoked_at,
        api_key_prefix: invite.api_key_prefix,
        api_key_created_at: invite.api_key_created_at,
        api_key_last_used_at: invite.api_key_last_used_at,
        api_key: invite.api_key_plaintext,
        enabled,
        active_session_count,
        daily_limit,
        success_count,
        in_flight,
        remaining_today,
    }
}

fn to_invite_info_summary(state: &AppState, invite: hone_memory::WebInviteUser) -> WebInviteInfo {
    let daily_limit = state.core.config.agent.daily_conversation_limit;
    let remaining_today = if daily_limit == 0 { 0 } else { daily_limit };
    let enabled = invite.revoked_at.is_none();

    WebInviteInfo {
        user_id: invite.user_id,
        invite_code: invite.invite_code,
        phone_number: invite.phone_number,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        revoked_at: invite.revoked_at,
        api_key_prefix: invite.api_key_prefix,
        api_key_created_at: invite.api_key_created_at,
        api_key_last_used_at: invite.api_key_last_used_at,
        api_key: invite.api_key_plaintext,
        enabled,
        active_session_count: 0,
        daily_limit,
        success_count: 0,
        in_flight: 0,
        remaining_today,
    }
}
