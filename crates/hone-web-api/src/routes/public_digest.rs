//! Public 端用户可见的投资上下文读取与刷新 API:
//!
//! - GET /api/public/digest-context  → 当前用户(web session 登录态)的蒸馏投资主线
//!   map、整体投资风格、上次蒸馏时间、跳过的 ticker 列表、其 sandbox 里现有
//!   公司画像列表(ticker + dir name + profile.md 摘要前 N 字)
//! - GET /api/public/company-profile?ticker=XXX → 单只 ticker 完整 profile.md
//!   (read-only,不暴露写入路径 —— 编辑请通过 chat agent 触发 company_portrait skill)
//! - POST /api/public/digest-context/refresh → 立即触发一次蒸馏(对当前用户)
//!
//! 与 admin 端 mainline-context / mainline-distill 端点的区别:public 端 actor
//! 限定为自己(由 session 推导),admin 端可以代任何 actor 操作。

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

/// 公开用户的 actor 推导。复用 public.rs 的 session 鉴权逻辑(channel="web",user_id 来自 session)。
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

    // prefs(投资主线蒸馏结果)
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

    // 持仓(用于显示哪些 ticker 应该有投资主线但没有)
    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let holdings: Vec<String> = match portfolio_storage.load(&actor) {
        Ok(Some(p)) => p.holdings.iter().map(|h| h.symbol.clone()).collect(),
        _ => Vec::new(),
    };

    // sandbox 里现存的画像列表
    let sandbox_base = hone_channels::sandbox_base_dir();
    let profiles =
        hone_event_engine::global_digest::scan_profiles_for_actor(&sandbox_base, &actor, None);
    let profile_summaries = profile_summaries_from_sources(&profiles);

    Json(json!({
        "actor": {
            "channel": "web",
            "user_id": actor.user_id,
        },
        "mainline_style": prefs.mainline_style,
        "mainline_by_ticker": prefs.mainline_by_ticker.clone().unwrap_or_default(),
        "last_mainline_distilled_at": prefs.last_mainline_distilled_at,
        "mainline_distill_skipped": prefs.mainline_distill_skipped,
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
    let profiles =
        hone_event_engine::global_digest::scan_profiles_for_actor(&sandbox_base, &actor, None);
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
                "actor 没有 portfolio,无法蒸馏投资主线(请先建仓)",
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

    let gd = &state.core.config.event_engine.global_digest;
    let profile_ref = if gd.mainline_distill_llm.trim().is_empty() {
        &gd.event_dedupe_llm
    } else {
        &gd.mainline_distill_llm
    };
    let created = match hone_llm::LlmResolver::new(&state.core.config)
        .provider_for_profile_or_openrouter_model(
            Some(profile_ref),
            &gd.event_dedupe_model,
            &gd.event_dedupe_model,
            Some(1200),
        ) {
        Ok(created) => created,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM provider 不可用: {e}"),
            );
        }
    };
    let distiller = hone_event_engine::global_digest::LlmMainlineDistiller::new(
        created.provider,
        created.model,
    );

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
        "mainline_count": updated.mainline_by_ticker.as_ref().map(|m| m.len()).unwrap_or(0),
        "mainline_style_set": updated.mainline_style.is_some(),
        "skipped_tickers": updated.mainline_distill_skipped,
        "last_distilled_at": updated.last_mainline_distilled_at,
    }))
    .into_response()
}

fn profile_summaries_from_sources(
    profiles: &[hone_event_engine::global_digest::ProfileSource],
) -> Vec<serde_json::Value> {
    profiles
        .iter()
        .map(|profile| {
            json!({
                "ticker": profile.ticker,
                "dir": profile.dir_name,
                "preview": profile.markdown.chars().take(200).collect::<String>(),
            })
        })
        .collect()
}
