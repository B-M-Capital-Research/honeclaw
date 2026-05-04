use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use hone_core::ActorIdentity;

use crate::state::AppState;
use crate::types::{CreateWebInviteRequest, WebInviteInfo};

pub(crate) async fn handle_list_invites(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let invites = state
        .web_auth
        .list_invite_users()
        .unwrap_or_default()
        .into_iter()
        .map(|invite| {
            let user_id = invite.user_id.clone();
            to_invite_info(&state, &user_id, invite)
        })
        .collect::<Vec<_>>();

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
        Ok(Some(result)) => Json(json!({
            "invite": to_invite_info(&state, &user_id, result.invite),
            "cleared_session_count": result.cleared_session_count,
            "message": format!("已停用邀请码，并清理 {} 个登录态", result.cleared_session_count),
        }))
        .into_response(),
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
        Ok(Some(result)) => Json(json!({
            "invite": to_invite_info(&state, &user_id, result.invite),
            "cleared_session_count": result.cleared_session_count,
            "message": "已重新启用邀请码",
        }))
        .into_response(),
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
        Ok(Some(result)) => Json(json!({
            "invite": to_invite_info(&state, &user_id, result.invite),
            "cleared_session_count": result.cleared_session_count,
            "message": format!("已重置邀请码，并清理 {} 个登录态", result.cleared_session_count),
        }))
        .into_response(),
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
        Ok(Some(invite)) => Json(json!({
            "invite": to_invite_info(&state, &user_id, invite),
            "message": "已获取 API Key；明文仅显示一次，请妥善保存",
        }))
        .into_response(),
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
        Ok(Some(invite)) => Json(json!({
            "invite": to_invite_info(&state, &user_id, invite),
            "message": "已重置 API Key；旧 Key 已失效，新 Key 明文仅显示一次",
        }))
        .into_response(),
        Ok(None) => {
            crate::routes::json_error(axum::http::StatusCode::NOT_FOUND, "邀请码用户不存在")
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("重置 API Key 失败: {error}"),
        ),
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
