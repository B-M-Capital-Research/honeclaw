//! 管理端"我的推送日程" — `GET /api/admin/schedule?actor=channel::scope::user_id`。
//!
//! 把散落在 3 个地方的推送时间拍平成一张表，供前端 `/schedule` 页面与 NL 工具
//! `notification_prefs.get_overview` 共享同一份事实源。
//!
//! 实现是 thin wrapper：所有聚合逻辑都在 `hone_tools::schedule_view::build_overview`
//! 里。本文件只负责:解析查询参数、绑定 admin 配置(unified digest 默认槽位时刻),
//! 把结果渲染成 JSON 返回。

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::Deserialize;

use hone_core::ActorIdentity;
use hone_tools::schedule_view::{
    DigestDefaultSlot, DigestDefaults, ScheduleOverview, build_overview,
};

use crate::state::AppState;

#[derive(Deserialize)]
pub(crate) struct ScheduleQuery {
    /// `channel::scope::user_id`。scope 为空表示 direct session。
    actor: String,
}

pub(crate) async fn handle_schedule(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ScheduleQuery>,
) -> Result<Json<ScheduleOverview>, (StatusCode, String)> {
    let actor = parse_actor_key(&q.actor).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid actor key: {}", q.actor),
        )
    })?;

    let cfg = &state.core.config;
    let digest_defaults = DigestDefaults {
        slots: cfg
            .event_engine
            .digest
            .default_slots
            .iter()
            .map(|s| DigestDefaultSlot {
                time: s.time.clone(),
                label: s.label.clone(),
            })
            .collect(),
    };

    let prefs_dir = std::path::Path::new(&cfg.storage.notif_prefs_dir);
    let cron_dir = std::path::Path::new(&cfg.storage.cron_jobs_dir);

    let overview = build_overview(prefs_dir, cron_dir, &actor, &digest_defaults, Utc::now())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("build_overview failed: {e}"),
            )
        })?;
    Ok(Json(overview))
}

/// `channel::scope::user_id` → `ActorIdentity`。空 scope 段视为 direct session。
fn parse_actor_key(key: &str) -> Option<ActorIdentity> {
    let parts: Vec<&str> = key.splitn(3, "::").collect();
    if parts.len() != 3 {
        return None;
    }
    let channel = parts[0];
    let scope = parts[1];
    let user_id = parts[2];
    if channel.is_empty() || user_id.is_empty() {
        return None;
    }
    let scope_opt: Option<String> = if scope.is_empty() {
        None
    } else {
        Some(scope.to_string())
    };
    ActorIdentity::new(channel, user_id, scope_opt).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_actor_key_handles_direct() {
        let a = parse_actor_key("imessage::::u1").unwrap();
        assert_eq!(a.channel, "imessage");
        assert_eq!(a.user_id, "u1");
        assert!(a.channel_scope.is_none());
    }

    #[test]
    fn parse_actor_key_handles_group() {
        let a = parse_actor_key("discord::guild_123::u1").unwrap();
        assert_eq!(a.channel, "discord");
        assert_eq!(a.channel_scope.as_deref(), Some("guild_123"));
        assert_eq!(a.user_id, "u1");
    }

    #[test]
    fn parse_actor_key_rejects_malformed() {
        assert!(parse_actor_key("only-one-part").is_none());
        assert!(parse_actor_key("").is_none());
        assert!(parse_actor_key("imessage::scope::").is_none());
    }
}
