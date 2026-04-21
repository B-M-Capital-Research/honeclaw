//! 管理端 — 通知偏好 HTTP API。
//!
//! * GET  /api/notification-prefs?channel=&user_id=&channel_scope=
//!   → 指定 actor 的 NotificationPrefs JSON;文件缺失返默认,不 404。
//! * PUT  /api/notification-prefs  body: { actor, prefs }
//!   → 写盘。非法 kind tag 返 400 并附合法清单,下一条事件即可感知(router
//!     每次 dispatch 重读)。
//!
//! 给管理员代改任意 actor 的设置用;终端用户自己在渠道里通过 Tool+Skill 自然
//! 语言改(那条路径在构造 Tool 时硬绑定 actor,不会被这个 API 的"代改任何人"
//! 能力暴露)。

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use hone_event_engine::prefs::{
    first_invalid_kind_tag, FilePrefsStorage, NotificationPrefs, PrefsProvider, ALL_KIND_TAGS,
};

use crate::routes::{json_error, require_actor};
use crate::state::AppState;
use crate::types::UserIdQuery;

#[derive(Deserialize)]
pub(crate) struct PutPrefsBody {
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub channel_scope: Option<String>,
    pub prefs: NotificationPrefs,
}

fn prefs_dir(state: &AppState) -> PathBuf {
    PathBuf::from(&state.core.config.storage.notif_prefs_dir)
}

fn validate_prefs(prefs: &NotificationPrefs) -> Result<(), Response> {
    if let Some(bad) = first_invalid_kind_tag(prefs.blocked_kinds.iter().map(|s| s.as_str())) {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            format!(
                "blocked_kinds 含未知 tag '{bad}';合法清单:{}",
                ALL_KIND_TAGS.join(", ")
            ),
        ));
    }
    if let Some(allow) = &prefs.allow_kinds {
        if let Some(bad) = first_invalid_kind_tag(allow.iter().map(|s| s.as_str())) {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                format!(
                    "allow_kinds 含未知 tag '{bad}';合法清单:{}",
                    ALL_KIND_TAGS.join(", ")
                ),
            ));
        }
    }
    Ok(())
}

/// GET /api/notification-prefs
pub(crate) async fn handle_get_prefs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> Response {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let storage = match FilePrefsStorage::new(prefs_dir(&state)) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("打开 prefs 目录失败: {e}"),
            );
        }
    };
    Json(json!({
        "prefs": storage.load(&actor),
        "kind_tags": ALL_KIND_TAGS,
    }))
    .into_response()
}

/// PUT /api/notification-prefs
pub(crate) async fn handle_put_prefs(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PutPrefsBody>,
) -> Response {
    let actor = match require_actor(body.channel, body.user_id, body.channel_scope) {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    if let Err(resp) = validate_prefs(&body.prefs) {
        return resp;
    }
    let storage = match FilePrefsStorage::new(prefs_dir(&state)) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("打开 prefs 目录失败: {e}"),
            );
        }
    };
    if let Err(e) = storage.save(&actor, &body.prefs) {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("保存 prefs 失败: {e}"),
        );
    }
    Json(json!({ "prefs": body.prefs })).into_response()
}
