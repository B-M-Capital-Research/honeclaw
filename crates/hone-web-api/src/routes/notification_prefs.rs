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
use serde::{Deserialize, Serialize};
use serde_json::json;

use hone_event_engine::prefs::{ALL_KIND_TAGS, FilePrefsStorage, NotificationPrefs, PrefsProvider};

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

#[derive(Deserialize)]
pub(crate) struct BatchPrefsBody {
    pub actors: Vec<BatchPrefsActor>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct BatchPrefsActor {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
}

#[derive(Serialize)]
struct BatchPrefsEntry {
    actor: BatchPrefsActor,
    prefs: NotificationPrefs,
}

fn prefs_dir(state: &AppState) -> PathBuf {
    PathBuf::from(&state.core.config.storage.notif_prefs_dir)
}

fn validate_prefs(prefs: &NotificationPrefs) -> Result<(), String> {
    prefs.validate().map_err(|error| error.to_string())
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
        "prefs": storage.load(&actor).await,
        "kind_tags": ALL_KIND_TAGS,
    }))
    .into_response()
}

/// POST /api/notification-prefs/batch
pub(crate) async fn handle_batch_get_prefs(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchPrefsBody>,
) -> Response {
    if body.actors.len() > 500 {
        return json_error(
            StatusCode::BAD_REQUEST,
            format!("actors 过多，最多支持 500 个，收到 {}", body.actors.len()),
        );
    }

    let mut actor_refs = Vec::with_capacity(body.actors.len());
    let mut actors = Vec::with_capacity(body.actors.len());
    for actor in body.actors {
        let identity = match require_actor(
            Some(actor.channel.clone()),
            Some(actor.user_id.clone()),
            actor.channel_scope.clone(),
        ) {
            Ok(actor) => actor,
            Err(resp) => return resp,
        };
        actor_refs.push(actor);
        actors.push(identity);
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
    let prefs = storage.load_many(&actors).await;
    let entries = actor_refs
        .into_iter()
        .zip(prefs)
        .map(|(actor, prefs)| BatchPrefsEntry { actor, prefs })
        .collect::<Vec<_>>();
    Json(json!({
        "entries": entries,
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
    if let Err(error) = validate_prefs(&body.prefs) {
        return json_error(StatusCode::BAD_REQUEST, error);
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
    if let Err(e) = storage.save(&actor, &body.prefs).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("保存 prefs 失败: {e}"),
        );
    }
    Json(json!({ "prefs": body.prefs })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_event_engine::prefs::QuietHours;
    use hone_event_engine::unified_digest::DigestSlot;

    #[test]
    fn shared_validation_accepts_structured_time_and_numeric_controls() {
        let prefs = NotificationPrefs {
            timezone: Some("Asia/Shanghai".into()),
            digest_slots: Some(vec![
                DigestSlot {
                    id: "postmarket".into(),
                    time: "07:30".into(),
                    label: Some("盘后要闻".into()),
                    floor_macro: Some(2),
                },
                DigestSlot {
                    id: "premarket".into(),
                    time: "21:00".into(),
                    label: Some("盘前要闻".into()),
                    floor_macro: None,
                },
            ]),
            price_high_pct_override: Some(6.0),
            price_high_pct_up_override: Some(7.0),
            price_high_pct_down_override: Some(5.0),
            price_realert_step_pct_override: Some(4.0),
            large_position_weight_pct: Some(20.0),
            quiet_hours: Some(QuietHours {
                from: "23:00".into(),
                to: "07:30".into(),
                exempt_kinds: vec!["earnings_released".into()],
            }),
            ..Default::default()
        };
        validate_prefs(&prefs).unwrap();
    }

    #[test]
    fn shared_validation_rejects_duplicate_slots_and_out_of_range_numbers() {
        let duplicate_slots = NotificationPrefs {
            digest_slots: Some(vec![
                DigestSlot {
                    id: "one".into(),
                    time: "08:30".into(),
                    label: None,
                    floor_macro: None,
                },
                DigestSlot {
                    id: "two".into(),
                    time: "08:30".into(),
                    label: None,
                    floor_macro: None,
                },
            ]),
            ..Default::default()
        };
        assert!(
            validate_prefs(&duplicate_slots)
                .unwrap_err()
                .contains("重复时刻")
        );

        let invalid_number = NotificationPrefs {
            large_position_weight_pct: Some(101.0),
            ..Default::default()
        };
        assert!(
            validate_prefs(&invalid_number)
                .unwrap_err()
                .contains("large_position_weight_pct")
        );
    }

    #[test]
    fn shared_validation_rejects_quiet_overlap_but_allows_end_boundary() {
        let quiet_hours = QuietHours {
            from: "23:00".into(),
            to: "07:30".into(),
            exempt_kinds: Vec::new(),
        };
        let overlapping = NotificationPrefs {
            digest_slots: Some(vec![DigestSlot {
                id: "night".into(),
                time: "02:00".into(),
                label: None,
                floor_macro: None,
            }]),
            quiet_hours: Some(quiet_hours.clone()),
            ..Default::default()
        };
        assert!(validate_prefs(&overlapping).unwrap_err().contains("吞掉"));

        let boundary = NotificationPrefs {
            digest_slots: Some(vec![DigestSlot {
                id: "postmarket".into(),
                time: "07:30".into(),
                label: None,
                floor_macro: None,
            }]),
            quiet_hours: Some(quiet_hours),
            ..Default::default()
        };
        validate_prefs(&boundary).unwrap();
    }
}
