use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;

use hone_memory::{
    WEB_ADMIN_DAILY_INVITE_LIMIT, WebAdminInviteCreateOutcome, WebAdminInviteDisableOutcome,
    WebInviteUser,
};

use crate::state::AppState;
use crate::types::{
    PublicAdminCreateInviteRequest, PublicAdminInviteInfo, PublicAdminInviteList,
    PublicAdminInviteMutation,
};

const ADMIN_ACTION_HEADER: &str = "x-hone-admin-action";
const ADMIN_ACTION_HEADER_VALUE: &str = "whitelist";

pub(crate) async fn handle_list_invites(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let admin = match require_public_admin(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let invites = state_for_worker.web_auth.list_invite_users()?;
        let created_today = state_for_worker
            .web_auth
            .web_admin_create_count_today(&admin_user_id)?;
        Ok::<_, hone_core::HoneError>((invites, created_today))
    })
    .await;
    let (invites, created_today) = match result {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取会员白名单失败: {error}"),
            );
        }
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取会员白名单任务失败: {error}"),
            );
        }
    };
    let response = PublicAdminInviteList {
        invites: invites
            .into_iter()
            .filter(|invite| !invite.phone_number.trim().is_empty())
            .map(|invite| to_public_admin_invite(&admin.user_id, invite))
            .collect(),
        daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
        created_today,
        remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
    };
    Json(response).into_response()
}

pub(crate) async fn handle_create_invite(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicAdminCreateInviteRequest>,
) -> Response {
    let admin = match require_public_admin_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(phone_number) => phone_number,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        state_for_worker
            .web_auth
            .create_invite_user_by_admin(&admin_user_id, &phone_number)
    })
    .await;
    match result {
        Ok(Ok(WebAdminInviteCreateOutcome::Created { invite, used_today })) => {
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_invite(&admin.user_id, invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today: used_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(used_today),
                cleared_session_count: 0,
                message: "已加入会员白名单".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteCreateOutcome::NotAdmin)) => {
            crate::routes::json_error(StatusCode::FORBIDDEN, "当前账号没有管理权限")
        }
        Ok(Ok(WebAdminInviteCreateOutcome::LimitReached { used_today })) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "今日新增白名单已达到 5 人上限",
                "daily_create_limit": WEB_ADMIN_DAILY_INVITE_LIMIT,
                "created_today": used_today,
                "remaining_today": 0,
            })),
        )
            .into_response(),
        Ok(Ok(WebAdminInviteCreateOutcome::DuplicatePhone)) => {
            crate::routes::json_error(StatusCode::CONFLICT, "该手机号已在会员白名单中")
        }
        Ok(Err(error)) if error.to_string().contains("手机号格式不合法") => {
            crate::routes::json_error(StatusCode::BAD_REQUEST, "手机号格式不合法")
        }
        Ok(Err(error)) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("新增会员白名单失败: {error}"),
        ),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("新增会员白名单任务失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_disable_invite(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(target_user_id): Path<String>,
) -> Response {
    let admin = match require_public_admin_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let admin_user_id = admin.user_id.clone();
    let state_for_worker = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        state_for_worker
            .web_auth
            .disable_invite_user_by_admin(&admin_user_id, &target_user_id)
    })
    .await;
    match result {
        Ok(Ok(WebAdminInviteDisableOutcome::Disabled(result))) => {
            let created_today = state
                .web_auth
                .web_admin_create_count_today(&admin.user_id)
                .unwrap_or(WEB_ADMIN_DAILY_INVITE_LIMIT);
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_invite(&admin.user_id, result.invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
                cleared_session_count: result.cleared_session_count,
                message: "已禁用会员白名单，并清理该用户登录态".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteDisableOutcome::AlreadyDisabled(invite))) => {
            let created_today = state
                .web_auth
                .web_admin_create_count_today(&admin.user_id)
                .unwrap_or(WEB_ADMIN_DAILY_INVITE_LIMIT);
            Json(PublicAdminInviteMutation {
                invite: to_public_admin_invite(&admin.user_id, invite),
                daily_create_limit: WEB_ADMIN_DAILY_INVITE_LIMIT,
                created_today,
                remaining_today: WEB_ADMIN_DAILY_INVITE_LIMIT.saturating_sub(created_today),
                cleared_session_count: 0,
                message: "该用户已处于禁用状态".to_string(),
            })
            .into_response()
        }
        Ok(Ok(WebAdminInviteDisableOutcome::NotAdmin)) => {
            crate::routes::json_error(StatusCode::FORBIDDEN, "当前账号没有管理权限")
        }
        Ok(Ok(WebAdminInviteDisableOutcome::NotFound)) => {
            crate::routes::json_error(StatusCode::NOT_FOUND, "会员白名单用户不存在")
        }
        Ok(Ok(WebAdminInviteDisableOutcome::ProtectedAdmin)) => {
            crate::routes::json_error(StatusCode::CONFLICT, "不能禁用当前管理员或其他管理员")
        }
        Ok(Err(error)) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("禁用会员白名单失败: {error}"),
        ),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("禁用会员白名单任务失败: {error}"),
        ),
    }
}

fn require_public_admin(state: &AppState, headers: &HeaderMap) -> Result<WebInviteUser, Response> {
    let user = crate::routes::public::require_public_session_user(state, headers)?;
    match state.web_auth.is_web_admin(&user.user_id) {
        Ok(true) => Ok(user),
        Ok(false) => Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "当前账号没有管理权限",
        )),
        Err(error) => Err(crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取管理员权限失败: {error}"),
        )),
    }
}

fn require_public_admin_mutation(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<WebInviteUser, Response> {
    let user = require_public_admin(state, headers)?;
    let marker = headers
        .get(ADMIN_ACTION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim);
    if marker != Some(ADMIN_ACTION_HEADER_VALUE) {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "管理操作校验失败，请刷新后重试",
        ));
    }
    Ok(user)
}

fn to_public_admin_invite(admin_user_id: &str, invite: WebInviteUser) -> PublicAdminInviteInfo {
    let enabled = invite.revoked_at.is_none();
    PublicAdminInviteInfo {
        can_disable: enabled && invite.user_id != admin_user_id,
        user_id: invite.user_id,
        phone_number: invite.phone_number,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::{ADMIN_ACTION_HEADER, ADMIN_ACTION_HEADER_VALUE, to_public_admin_invite};
    use axum::http::{HeaderMap, HeaderValue};
    use hone_memory::WebInviteUser;

    fn invite(user_id: &str, revoked: bool) -> WebInviteUser {
        WebInviteUser {
            user_id: user_id.to_string(),
            invite_code: "HONE-TEST".to_string(),
            phone_number: "13800138000".to_string(),
            created_at: "2026-07-31T00:00:00+08:00".to_string(),
            last_login_at: None,
            revoked_at: revoked.then(|| "2026-07-31T01:00:00+08:00".to_string()),
            password_hash: None,
            password_set_at: None,
            tos_accepted_at: None,
            tos_version: None,
            api_key_prefix: None,
            api_key_created_at: None,
            api_key_last_used_at: None,
            api_key_plaintext: None,
        }
    }

    #[test]
    fn public_projection_never_exposes_invite_or_api_credentials() {
        let value = serde_json::to_value(to_public_admin_invite("admin", invite("member", false)))
            .expect("serialize");
        assert!(value.get("invite_code").is_none());
        assert!(value.get("api_key").is_none());
        assert!(value.get("password_hash").is_none());
        assert_eq!(value["can_disable"], true);
    }

    #[test]
    fn public_projection_protects_self_and_disabled_rows() {
        assert!(!to_public_admin_invite("admin", invite("admin", false)).can_disable);
        assert!(!to_public_admin_invite("admin", invite("member", true)).can_disable);
    }

    #[test]
    fn mutation_marker_requires_exact_custom_header() {
        let mut headers = HeaderMap::new();
        assert!(headers.get(ADMIN_ACTION_HEADER).is_none());
        headers.insert(
            ADMIN_ACTION_HEADER,
            HeaderValue::from_static(ADMIN_ACTION_HEADER_VALUE),
        );
        assert_eq!(
            headers
                .get(ADMIN_ACTION_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some(ADMIN_ACTION_HEADER_VALUE)
        );
    }
}
