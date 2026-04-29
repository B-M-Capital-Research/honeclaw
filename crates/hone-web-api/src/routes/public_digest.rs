//! Public 端用户可见的 digest 配置展示 API:
//!
//! - GET /api/public/digest-context  → 当前用户(web 邀请登录态)的蒸馏 thesis
//!   map、整体投资风格、上次蒸馏时间、跳过的 ticker 列表、其 sandbox 里现有
//!   公司画像列表(ticker + dir name + profile.md 摘要前 N 字)
//! - GET /api/public/company-profile?ticker=XXX → 单只 ticker 完整 profile.md
//!   (read-only,不暴露写入路径 —— 编辑请通过 chat agent 触发 company_portrait skill)
//! - POST /api/public/digest-context/refresh → 立即触发一次蒸馏(对当前用户)
//!
//! 与 admin 端 /api/event-engine/thesis-distill 区别:public 端 actor 限定为
//! 自己(由 session 推导),admin 端可以代任何 actor 操作。

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hone_core::ActorIdentity;
use serde::Deserialize;
use serde_json::json;

use crate::routes::json_error;
use crate::state::AppState;

/// 公开用户的 actor 推导。复制自 public.rs 的逻辑(channel="web",user_id 来自 session)。
fn require_public_actor(state: &AppState, headers: &HeaderMap) -> Result<ActorIdentity, Response> {
    let user = crate::routes::public::require_public_user(state, headers)?;
    ActorIdentity::new("web", &user.user_id, Option::<String>::None).map_err(|e| {
        json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("构造 actor 失败: {e}"),
        )
    })
}

/// GET /api/public/digest-context
pub(crate) async fn handle_get_digest_context(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let actor = match require_public_actor(&state, &headers) {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    // prefs(thesis 蒸馏结果)
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

    // 持仓(用于显示哪些 ticker 应该有 thesis 但没有)
    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let holdings: Vec<String> = match portfolio_storage.load(&actor) {
        Ok(Some(p)) => p.holdings.iter().map(|h| h.symbol.clone()).collect(),
        _ => Vec::new(),
    };

    // sandbox 里现存的画像列表
    let sandbox_base = hone_channels::sandbox_base_dir();
    let sandbox_root = hone_event_engine::global_digest::actor_sandbox_dir(&sandbox_base, &actor);
    let profile_summaries = list_profile_summaries(&sandbox_root);

    Json(json!({
        "actor": {
            "channel": "web",
            "user_id": actor.user_id,
        },
        "investment_global_style": prefs.investment_global_style,
        "investment_theses": prefs.investment_theses.clone().unwrap_or_default(),
        "last_thesis_distilled_at": prefs.last_thesis_distilled_at,
        "thesis_distill_skipped": prefs.thesis_distill_skipped,
        "holdings": holdings,
        "profile_list": profile_summaries,
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProfileQuery {
    pub ticker: String,
}

/// GET /api/public/company-profile?ticker=XXX
pub(crate) async fn handle_get_company_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ProfileQuery>,
) -> Response {
    let actor = match require_public_actor(&state, &headers) {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    let target = params.ticker.trim().to_uppercase();
    if target.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "ticker 不能为空");
    }

    let sandbox_base = hone_channels::sandbox_base_dir();
    let sandbox_root = hone_event_engine::global_digest::actor_sandbox_dir(&sandbox_base, &actor);
    let profiles = hone_event_engine::global_digest::scan_profiles(&sandbox_root, None);
    let hit = profiles.iter().find(|p| p.ticker == target);
    match hit {
        Some(p) => Json(json!({
            "ticker": p.ticker,
            "dir": p.dir_name,
            "markdown": p.markdown,
        }))
        .into_response(),
        None => json_error(
            StatusCode::NOT_FOUND,
            format!("未找到 ticker={target} 的画像;请通过 chat 触发 company_portrait skill 建档"),
        ),
    }
}

/// POST /api/public/digest-context/refresh
///
/// 用户主动触发一次蒸馏(同 admin 端,但 actor 锁死为自己)。
pub(crate) async fn handle_refresh_digest_context(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let actor = match require_public_actor(&state, &headers) {
        Ok(a) => a,
        Err(resp) => return resp,
    };

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
        return json_error(StatusCode::BAD_REQUEST, "portfolio 持仓为空");
    }

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

/// 扫 sandbox/company_profiles 列出所有画像的元信息(不回传完整 markdown,只 200 字预览)。
fn list_profile_summaries(sandbox_root: &PathBuf) -> Vec<serde_json::Value> {
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
