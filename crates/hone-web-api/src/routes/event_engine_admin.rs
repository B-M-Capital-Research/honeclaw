//! 管理端 — 事件引擎运行时配置 HTTP API。
//!
//! * GET  /api/event-engine/global-digest               → 当前 effective config 里的
//!   `event_engine.global_digest` 节;包含已 merge 的 overlay 值。
//! * PUT  /api/event-engine/global-digest               → 整段写入(写到 overlay,
//!   不动 config.yaml 注释);响应里 `needs_restart=true` —— scheduler/RSS 子树都
//!   是启动时 spawn,不做热生效。
//!
//! * GET  /api/event-engine/rss-feeds                   → 当前生效列表
//! * POST /api/event-engine/rss-feeds                   → 新增一条 RssFeedConfig
//! * PUT  /api/event-engine/rss-feeds/{handle}          → 整条覆盖(允许换 url
//!   或 interval);path 参数 handle 必须与 body.handle 一致
//! * DELETE /api/event-engine/rss-feeds/{handle}        → 删一条
//!
//! 所有写操作:
//! - 写到 `<config>.overrides.yaml`(`apply_overlay_mutations`),保留用户手写的
//!   config.yaml 注释
//! - 校验失败 → 400 + 原因 + 现有合法清单
//! - 校验通过 → 写盘 + 返回新的整段 + `needs_restart=true`(scheduler 不会自动
//!   读新配置,需用户重启 web-api 进程才会按新值起 poller / 触发新 schedule)。

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use serde_yaml::Value as YamlValue;

use hone_core::config::{
    ConfigMutation, GlobalDigestConfig, RssFeedConfig, apply_overlay_mutations,
};

use crate::routes::json_error;
use crate::runtime::runtime_config_path;
use crate::state::AppState;

const NEEDS_RESTART_HINT: &str =
    "改动已写入 config.overrides.yaml。事件引擎需重启 web-api 进程才会按新值生效";

fn config_path_buf() -> PathBuf {
    PathBuf::from(runtime_config_path())
}

// ─────────────────────────── global digest ───────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct PutGlobalDigestBody {
    #[serde(flatten)]
    pub config: GlobalDigestConfig,
}

fn validate_global_digest(cfg: &GlobalDigestConfig) -> Result<(), Response> {
    if cfg.timezone.trim().is_empty() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "global_digest.timezone 不能为空 (例 \"Asia/Shanghai\")",
        ));
    }
    use std::str::FromStr;
    if chrono_tz::Tz::from_str(cfg.timezone.trim()).is_err() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            format!(
                "global_digest.timezone {:?} 不是合法 IANA 名;示例:Asia/Shanghai、America/New_York、Europe/London",
                cfg.timezone
            ),
        ));
    }
    for s in &cfg.schedules {
        if chrono::NaiveTime::parse_from_str(s, "%H:%M").is_err() {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                format!("global_digest.schedules 含非法时刻 {s:?},必须是 HH:MM (24h)"),
            ));
        }
    }
    if cfg.final_pick_n == 0 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "global_digest.final_pick_n 必须 > 0",
        ));
    }
    if cfg.pass2_top_n < cfg.final_pick_n {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            format!(
                "global_digest.pass2_top_n ({}) 必须 >= final_pick_n ({})",
                cfg.pass2_top_n, cfg.final_pick_n
            ),
        ));
    }
    if cfg.lookback_hours == 0 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "global_digest.lookback_hours 必须 > 0",
        ));
    }
    if cfg.pass1_model.trim().is_empty() || cfg.pass2_model.trim().is_empty() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "global_digest.pass1_model / pass2_model 不能为空",
        ));
    }
    Ok(())
}

/// GET /api/event-engine/global-digest
pub(crate) async fn handle_get_global_digest(State(state): State<Arc<AppState>>) -> Response {
    let cfg = &state.core.config.event_engine.global_digest;
    Json(json!({
        "config": cfg,
    }))
    .into_response()
}

/// PUT /api/event-engine/global-digest
pub(crate) async fn handle_put_global_digest(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<PutGlobalDigestBody>,
) -> Response {
    if let Err(resp) = validate_global_digest(&body.config) {
        return resp;
    }
    let yaml_value = match serde_yaml::to_value(&body.config) {
        Ok(v) => v,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("序列化 global_digest 失败: {e}"),
            );
        }
    };
    let result = match apply_overlay_mutations(
        &config_path_buf(),
        &[ConfigMutation::Set {
            path: "event_engine.global_digest".into(),
            value: yaml_value,
        }],
    ) {
        Ok(r) => r,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("写入 overlay 失败: {e}"),
            );
        }
    };
    Json(json!({
        "config": result.config.event_engine.global_digest,
        "needs_restart": true,
        "hint": NEEDS_RESTART_HINT,
    }))
    .into_response()
}

// ─────────────────────────── rss feeds ───────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct UpsertRssFeedBody {
    pub handle: String,
    pub url: String,
    #[serde(default = "default_rss_interval_api")]
    pub interval_secs: u64,
}

fn default_rss_interval_api() -> u64 {
    30 * 60
}

fn validate_rss_handle(handle: &str) -> Result<String, Response> {
    let h = handle.trim();
    if h.is_empty() {
        return Err(json_error(StatusCode::BAD_REQUEST, "rss handle 不能为空"));
    }
    if !h
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            format!(
                "rss handle {h:?} 含非法字符;只允许字母/数字/_/-(影响 source 标签 \"rss:{h}\")"
            ),
        ));
    }
    Ok(h.to_string())
}

fn validate_rss_url(url: &str) -> Result<(), Response> {
    let u = url.trim();
    if u.is_empty() {
        return Err(json_error(StatusCode::BAD_REQUEST, "rss url 不能为空"));
    }
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            format!("rss url {u:?} 必须以 http:// 或 https:// 开头"),
        ));
    }
    Ok(())
}

fn build_rss_feed(body: UpsertRssFeedBody) -> Result<RssFeedConfig, Response> {
    let handle = validate_rss_handle(&body.handle)?;
    validate_rss_url(&body.url)?;
    if body.interval_secs == 0 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "rss interval_secs 必须 > 0",
        ));
    }
    Ok(RssFeedConfig {
        handle,
        url: body.url.trim().to_string(),
        interval_secs: body.interval_secs,
    })
}

fn write_rss_feeds(feeds: Vec<RssFeedConfig>) -> Result<Vec<RssFeedConfig>, Response> {
    let yaml_value = match serde_yaml::to_value(&feeds) {
        Ok(v) => v,
        Err(e) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("序列化 rss_feeds 失败: {e}"),
            ));
        }
    };
    let mutation = if feeds.is_empty() {
        // 空列表用 Set [] 显式覆盖,而不是 Unset —— 用户明确想清空,
        // 而非"删掉 overlay 让 base 重新生效"。
        ConfigMutation::Set {
            path: "event_engine.sources.rss_feeds".into(),
            value: YamlValue::Sequence(Vec::new()),
        }
    } else {
        ConfigMutation::Set {
            path: "event_engine.sources.rss_feeds".into(),
            value: yaml_value,
        }
    };
    match apply_overlay_mutations(&config_path_buf(), &[mutation]) {
        Ok(result) => Ok(result.config.event_engine.sources.rss_feeds),
        Err(e) => Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("写入 overlay 失败: {e}"),
        )),
    }
}

/// GET /api/event-engine/rss-feeds
pub(crate) async fn handle_list_rss_feeds(State(state): State<Arc<AppState>>) -> Response {
    let feeds = &state.core.config.event_engine.sources.rss_feeds;
    Json(json!({ "feeds": feeds })).into_response()
}

/// POST /api/event-engine/rss-feeds
pub(crate) async fn handle_create_rss_feed(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpsertRssFeedBody>,
) -> Response {
    let new_feed = match build_rss_feed(body) {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    let mut feeds = state.core.config.event_engine.sources.rss_feeds.clone();
    if feeds.iter().any(|f| f.handle == new_feed.handle) {
        return json_error(
            StatusCode::CONFLICT,
            format!(
                "rss handle {:?} 已存在;用 PUT /api/event-engine/rss-feeds/{} 修改",
                new_feed.handle, new_feed.handle
            ),
        );
    }
    feeds.push(new_feed);
    match write_rss_feeds(feeds) {
        Ok(updated) => Json(json!({
            "feeds": updated,
            "needs_restart": true,
            "hint": NEEDS_RESTART_HINT,
        }))
        .into_response(),
        Err(resp) => resp,
    }
}

/// PUT /api/event-engine/rss-feeds/{handle}
pub(crate) async fn handle_update_rss_feed(
    State(state): State<Arc<AppState>>,
    Path(handle): Path<String>,
    Json(body): Json<UpsertRssFeedBody>,
) -> Response {
    let new_feed = match build_rss_feed(body) {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    if new_feed.handle != handle.trim() {
        return json_error(
            StatusCode::BAD_REQUEST,
            format!(
                "URL handle {handle:?} 与 body.handle {:?} 不一致",
                new_feed.handle
            ),
        );
    }
    let mut feeds = state.core.config.event_engine.sources.rss_feeds.clone();
    let pos = match feeds.iter().position(|f| f.handle == new_feed.handle) {
        Some(p) => p,
        None => {
            return json_error(
                StatusCode::NOT_FOUND,
                format!("找不到 handle={:?} 的 rss feed", new_feed.handle),
            );
        }
    };
    feeds[pos] = new_feed;
    match write_rss_feeds(feeds) {
        Ok(updated) => Json(json!({
            "feeds": updated,
            "needs_restart": true,
            "hint": NEEDS_RESTART_HINT,
        }))
        .into_response(),
        Err(resp) => resp,
    }
}

/// DELETE /api/event-engine/rss-feeds/{handle}
pub(crate) async fn handle_delete_rss_feed(
    State(state): State<Arc<AppState>>,
    Path(handle): Path<String>,
) -> Response {
    let target = handle.trim().to_string();
    let mut feeds = state.core.config.event_engine.sources.rss_feeds.clone();
    let original_len = feeds.len();
    feeds.retain(|f| f.handle != target);
    if feeds.len() == original_len {
        return json_error(
            StatusCode::NOT_FOUND,
            format!("找不到 handle={target:?} 的 rss feed"),
        );
    }
    match write_rss_feeds(feeds) {
        Ok(updated) => Json(json!({
            "feeds": updated,
            "needs_restart": true,
            "hint": NEEDS_RESTART_HINT,
        }))
        .into_response(),
        Err(resp) => resp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(schedules: Vec<&str>, top_n: u32, pick_n: u32) -> GlobalDigestConfig {
        GlobalDigestConfig {
            enabled: true,
            timezone: "Asia/Shanghai".into(),
            schedules: schedules.into_iter().map(String::from).collect(),
            lookback_hours: 24,
            pass1_model: "amazon/nova-lite-v1".into(),
            pass2_model: "x-ai/grok-4.1-fast".into(),
            pass2_top_n: top_n,
            final_pick_n: pick_n,
            fetch_full_text: true,
        }
    }

    #[test]
    fn validate_global_digest_passes_on_canonical_config() {
        assert!(validate_global_digest(&cfg(vec!["09:00", "21:00"], 15, 8)).is_ok());
    }

    #[test]
    fn validate_global_digest_rejects_unknown_timezone() {
        let mut c = cfg(vec!["09:00"], 15, 8);
        c.timezone = "Mars/Olympus".into();
        let err = validate_global_digest(&c).unwrap_err();
        let body = format!("{:?}", err);
        assert!(body.contains("400") || body.contains("BAD_REQUEST"));
    }

    #[test]
    fn validate_global_digest_rejects_bad_schedule_format() {
        let c = cfg(vec!["25:99"], 15, 8);
        assert!(validate_global_digest(&c).is_err());
    }

    #[test]
    fn validate_global_digest_rejects_zero_final_pick_n() {
        let c = cfg(vec!["09:00"], 5, 0);
        assert!(validate_global_digest(&c).is_err());
    }

    #[test]
    fn validate_global_digest_rejects_top_n_below_pick_n() {
        let c = cfg(vec!["09:00"], 3, 8);
        assert!(validate_global_digest(&c).is_err());
    }

    #[test]
    fn validate_rss_handle_accepts_safe_chars() {
        assert!(validate_rss_handle("bloomberg_markets").is_ok());
        assert!(validate_rss_handle("space-news").is_ok());
        assert!(validate_rss_handle("stat2").is_ok());
    }

    #[test]
    fn validate_rss_handle_rejects_unsafe_chars() {
        assert!(validate_rss_handle("blo:omberg").is_err());
        assert!(validate_rss_handle("foo bar").is_err());
        assert!(validate_rss_handle("").is_err());
    }

    #[test]
    fn validate_rss_url_requires_http_scheme() {
        assert!(validate_rss_url("https://feeds.bloomberg.com/markets/news.rss").is_ok());
        assert!(validate_rss_url("http://example.com/feed").is_ok());
        assert!(validate_rss_url("ftp://example.com/feed").is_err());
        assert!(validate_rss_url("").is_err());
    }
}
