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

use crate::routes::{json_error, require_actor};
use crate::runtime::runtime_config_path;
use crate::state::AppState;
use crate::types::UserIdQuery;

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
    if cfg.event_dedupe_enabled && cfg.event_dedupe_model.trim().is_empty() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "global_digest.event_dedupe_enabled=true 时 event_dedupe_model 不能为空",
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

// ─────────────────────────── thesis context (admin view) ──────────

/// GET /api/event-engine/thesis-context?channel=&user_id=&channel_scope=
///
/// 管理端查看任意 actor 的蒸馏 thesis 与画像 inventory。和 public 端
/// `/api/public/digest-context` 内容一致,但 actor 由 query 指定而非 session。
pub(crate) async fn handle_get_thesis_context(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<UserIdQuery>,
) -> Response {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    let prefs_storage = match hone_event_engine::prefs::FilePrefsStorage::new(
        &state.core.config.storage.notif_prefs_dir,
    ) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("打开 prefs 失败: {e}"),
            );
        }
    };
    use hone_event_engine::prefs::PrefsProvider;
    let prefs = prefs_storage.load(&actor);

    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let holdings: Vec<String> = match portfolio_storage.load(&actor) {
        Ok(Some(p)) => p.holdings.iter().map(|h| h.symbol.clone()).collect(),
        _ => Vec::new(),
    };

    let sandbox_base = hone_channels::sandbox_base_dir();
    let sandbox_root = hone_event_engine::global_digest::actor_sandbox_dir(&sandbox_base, &actor);
    let profile_summaries = list_profile_summaries_admin(&sandbox_root);

    Json(json!({
        "actor": {
            "channel": actor.channel,
            "user_id": actor.user_id,
            "channel_scope": actor.channel_scope,
        },
        "investment_global_style": prefs.investment_global_style,
        "investment_theses": prefs.investment_theses.clone().unwrap_or_default(),
        "global_digest_enabled": prefs.global_digest_enabled,
        "global_digest_floor_macro_picks": prefs.global_digest_floor_macro_picks,
        "last_thesis_distilled_at": prefs.last_thesis_distilled_at,
        "thesis_distill_skipped": prefs.thesis_distill_skipped,
        "holdings": holdings,
        "profile_list": profile_summaries,
    }))
    .into_response()
}

/// GET /api/event-engine/company-profile?channel=&user_id=&channel_scope=&ticker=
///
/// 管理端查看任意 actor 任意 ticker 的完整画像 markdown。read-only。
pub(crate) async fn handle_get_actor_company_profile(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<AdminProfileQuery>,
) -> Response {
    let actor = match require_actor(
        params.channel.clone(),
        params.user_id.clone(),
        params.channel_scope.clone(),
    ) {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let target = params.ticker.trim().to_uppercase();
    if target.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "ticker 不能为空");
    }
    let _ = &state; // silence unused
    let sandbox_base = hone_channels::sandbox_base_dir();
    let sandbox_root = hone_event_engine::global_digest::actor_sandbox_dir(&sandbox_base, &actor);
    let profiles = hone_event_engine::global_digest::scan_profiles(&sandbox_root, None);
    match profiles.iter().find(|p| p.ticker == target) {
        Some(p) => Json(json!({
            "ticker": p.ticker,
            "dir": p.dir_name,
            "markdown": p.markdown,
        }))
        .into_response(),
        None => json_error(
            StatusCode::NOT_FOUND,
            format!(
                "actor={}/{} 没有 ticker={target} 的画像",
                actor.channel, actor.user_id
            ),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AdminProfileQuery {
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub channel_scope: Option<String>,
    pub ticker: String,
}

/// 扫 sandbox/company_profiles 列出所有画像的元信息(摘要)。
fn list_profile_summaries_admin(sandbox_root: &PathBuf) -> Vec<serde_json::Value> {
    let cp = sandbox_root.join("company_profiles");
    if !cp.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&cp) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let profile_md = path.join("profile.md");
        if !profile_md.is_file() {
            continue;
        }
        let md = match std::fs::read_to_string(&profile_md) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let tickers = hone_event_engine::global_digest::extract_tickers(&md);
        let title = md
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches('#')
            .trim();
        let preview: String = md.chars().take(200).collect();
        let bytes = md.len();
        out.push(json!({
            "dir": dir_name,
            "tickers": tickers,
            "title": title,
            "preview": preview,
            "bytes": bytes,
        }));
    }
    out
}

// ─────────────────────────── thesis distill manual ─────────────────

/// POST /api/event-engine/thesis-distill?channel=&user_id=&channel_scope=
///
/// 立即对指定 actor 跑一次 thesis 蒸馏 —— admin 调试 / 用户主动刷新用。
/// 平时由 web-api 启动的 cron 每 7 天自动跑,不需要走这条。
pub(crate) async fn handle_distill_thesis_now(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<UserIdQuery>,
) -> Response {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    // portfolio + holdings
    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let portfolio = match portfolio_storage.load(&actor) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return json_error(
                StatusCode::NOT_FOUND,
                "actor 没有 portfolio,无法蒸馏 thesis(请先建仓)",
            );
        }
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读 portfolio 失败: {e}"),
            );
        }
    };
    let holdings: Vec<String> = portfolio
        .holdings
        .iter()
        .map(|h| h.symbol.clone())
        .collect();
    if holdings.is_empty() {
        return json_error(
            StatusCode::BAD_REQUEST,
            "portfolio 持仓为空,无法蒸馏 thesis",
        );
    }

    // LLM provider
    let provider: Arc<dyn hone_llm::LlmProvider> =
        match hone_llm::OpenRouterProvider::from_config(&state.core.config) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("OpenRouter provider 不可用: {e}"),
                );
            }
        };
    let model = state
        .core
        .config
        .event_engine
        .global_digest
        .event_dedupe_model
        .clone();
    let distiller = hone_event_engine::global_digest::LlmThesisDistiller::new(provider, model);

    let prefs_storage = match hone_event_engine::prefs::FilePrefsStorage::new(
        &state.core.config.storage.notif_prefs_dir,
    ) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("打开 prefs 目录失败: {e}"),
            );
        }
    };

    let sandbox_base = hone_channels::sandbox_base_dir();
    let updated = match hone_event_engine::global_digest::distill_and_persist_one(
        &distiller,
        &prefs_storage,
        &sandbox_base,
        &actor,
        &holdings,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, format!("蒸馏失败: {e}"));
        }
    };

    Json(json!({
        "ok": true,
        "theses_count": updated.investment_theses.as_ref().map(|m| m.len()).unwrap_or(0),
        "global_style_set": updated.investment_global_style.is_some(),
        "skipped_tickers": updated.thesis_distill_skipped,
        "last_distilled_at": updated.last_thesis_distilled_at,
    }))
    .into_response()
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
            event_dedupe_enabled: true,
            event_dedupe_model: "x-ai/grok-4.1-fast".into(),
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
