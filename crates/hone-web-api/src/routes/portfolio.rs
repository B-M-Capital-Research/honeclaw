use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use hone_core::cloud_runtime::{CloudPgRuntime, CloudPortfolioRecord};
use hone_core::{ActorIdentity, HoneResult};
use hone_memory::portfolio::{Holding, Portfolio, PortfolioStorage, normalize_holding_horizon};
use tracing::warn;

use crate::routes::{json_error, normalize_optional_string, require_actor, require_string};
use crate::state::AppState;
use crate::types::{PortfolioHoldingRequest, PortfolioSummary, UserIdQuery};

const PORTFOLIO_ACTORS_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
struct PortfolioActorsCache {
    value: Option<serde_json::Value>,
    updated_at: Option<Instant>,
    refreshing: bool,
}

static PORTFOLIO_ACTORS_CACHE: LazyLock<Mutex<PortfolioActorsCache>> =
    LazyLock::new(|| Mutex::new(PortfolioActorsCache::default()));

/// GET /api/portfolio/actors — 列出所有有持仓数据的 actor
pub(crate) async fn handle_portfolio_actors(
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    if let Some(cached) = cached_portfolio_actors(false) {
        return Json(cached).into_response();
    }
    if !mark_portfolio_actors_refreshing() {
        return Json(cached_portfolio_actors(true).unwrap_or_else(|| json!({ "actors": [] })))
            .into_response();
    }

    let result = if state
        .core
        .config
        .cloud
        .effective_mode()
        .is_cloud_authoritative()
    {
        if let Some(postgres) = CloudPgRuntime::from_cloud_config(&state.core.config.cloud) {
            list_cloud_portfolio_actor_summaries(postgres).await
        } else {
            Ok(Vec::new())
        }
    } else {
        let storage = portfolio_storage(&state);
        match tokio::time::timeout(
            Duration::from_secs(8),
            tokio::task::spawn_blocking(move || {
                storage
                    .list_all()
                    .iter()
                    .map(|(actor, portfolio)| portfolio_summary(actor, Some(portfolio)))
                    .collect::<Vec<_>>()
            }),
        )
        .await
        {
            Ok(Ok(summaries)) => Ok(summaries),
            Ok(Err(error)) => Err(hone_core::HoneError::Config(error.to_string())),
            Err(_) => Err(hone_core::HoneError::Config(
                "portfolio actors list timed out".to_string(),
            )),
        }
    };

    let summaries = match result {
        Ok(summaries) => summaries,
        Err(error) => {
            warn!(%error, "failed to list portfolio actors");
            clear_portfolio_actors_refreshing();
            return Json(cached_portfolio_actors(true).unwrap_or_else(|| json!({ "actors": [] })))
                .into_response();
        }
    };
    let value = json!({ "actors": summaries });
    update_portfolio_actors_cache(value.clone());
    Json(value).into_response()
}

/// GET /api/portfolio?user_id=... — 查看用户持仓
pub(crate) async fn handle_portfolio(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> axum::response::Response {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let storage = portfolio_storage(&state);
    match storage.load(&actor) {
        Ok(portfolio) => Json(json!({
            "portfolio": portfolio,
            "summary": portfolio_summary(&actor, portfolio.as_ref())
        }))
        .into_response(),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// POST /api/portfolio/holdings — 新增或覆盖单个持仓
pub(crate) async fn handle_create_holding(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PortfolioHoldingRequest>,
) -> axum::response::Response {
    let actor = match require_actor(req.channel, req.user_id, req.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let symbol = match require_string(req.symbol, "symbol") {
        Ok(symbol) => normalize_symbol(&symbol),
        Err(error) => return error,
    };
    let asset_type = normalize_asset_type(req.asset_type.as_deref());
    let shares = req.quantity.or(req.shares).unwrap_or(0.0);
    let avg_cost = req.cost_basis.or(req.avg_cost).unwrap_or(0.0);
    let storage = portfolio_storage(&state);

    match storage.upsert_holding(
        &actor,
        Holding {
            symbol,
            asset_type,
            shares,
            avg_cost,
            underlying: normalize_optional_string(req.underlying).map(|value| value.to_uppercase()),
            option_type: normalize_optional_option_type(req.option_type),
            strike_price: req.strike_price,
            expiration_date: normalize_optional_string(req.expiration_date),
            contract_multiplier: req.contract_multiplier,
            holding_horizon: normalize_optional_holding_horizon(req.holding_horizon),
            strategy_notes: normalize_optional_string(req.strategy_notes),
            notes: normalize_optional_string(req.notes),
            tracking_only: if req.tracking_only.unwrap_or(false) {
                Some(true)
            } else {
                None
            },
        },
    ) {
        Ok(portfolio) => {
            clear_portfolio_actors_cache();
            (
                StatusCode::CREATED,
                Json(json!({
                    "portfolio": portfolio,
                    "summary": portfolio_summary(&actor, Some(&portfolio))
                })),
            )
                .into_response()
        }
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// PUT /api/portfolio/holdings/{symbol} — 更新单个持仓
pub(crate) async fn handle_update_holding(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Json(req): Json<PortfolioHoldingRequest>,
) -> axum::response::Response {
    let actor = match require_actor(req.channel, req.user_id, req.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let symbol = normalize_symbol(&symbol);
    let asset_type = normalize_asset_type(req.asset_type.as_deref());
    let storage = portfolio_storage(&state);
    let existing_portfolio = match storage.load(&actor) {
        Ok(portfolio) => portfolio,
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };
    let Some(existing_portfolio) = existing_portfolio else {
        return json_error(
            StatusCode::NOT_FOUND,
            format!("用户 {} 暂无持仓", actor.user_id),
        );
    };
    let Some(existing_holding) = existing_portfolio
        .holdings
        .iter()
        .find(|holding| holding.symbol == symbol && holding.asset_type == asset_type)
        .cloned()
    else {
        return json_error(StatusCode::NOT_FOUND, format!("未找到持仓 {symbol}"));
    };

    let shares = req
        .quantity
        .or(req.shares)
        .unwrap_or(existing_holding.shares);
    let avg_cost = req
        .cost_basis
        .or(req.avg_cost)
        .unwrap_or(existing_holding.avg_cost);
    let notes = req.notes.or(existing_holding.notes);
    let underlying = req.underlying.or(existing_holding.underlying);
    let option_type = req.option_type.or(existing_holding.option_type);
    let strike_price = req.strike_price.or(existing_holding.strike_price);
    let expiration_date = req.expiration_date.or(existing_holding.expiration_date);
    let contract_multiplier = req
        .contract_multiplier
        .or(existing_holding.contract_multiplier);
    let holding_horizon = req.holding_horizon.or(existing_holding.holding_horizon);
    let strategy_notes = req.strategy_notes.or(existing_holding.strategy_notes);
    let tracking_only = match req.tracking_only {
        Some(true) => Some(true),
        Some(false) => None,
        None => existing_holding.tracking_only,
    };

    match storage.upsert_holding(
        &actor,
        Holding {
            symbol,
            asset_type,
            shares,
            avg_cost,
            underlying: normalize_optional_string(underlying).map(|value| value.to_uppercase()),
            option_type: normalize_optional_option_type(option_type),
            strike_price,
            expiration_date: normalize_optional_string(expiration_date),
            contract_multiplier,
            holding_horizon: normalize_optional_holding_horizon(holding_horizon),
            strategy_notes: normalize_optional_string(strategy_notes),
            notes: normalize_optional_string(notes),
            tracking_only,
        },
    ) {
        Ok(portfolio) => {
            clear_portfolio_actors_cache();
            Json(json!({
                "portfolio": portfolio,
                "summary": portfolio_summary(&actor, Some(&portfolio))
            }))
            .into_response()
        }
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// DELETE /api/portfolio/holdings/{symbol}?user_id=... — 删除持仓
pub(crate) async fn handle_delete_holding(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> axum::response::Response {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let storage = portfolio_storage(&state);
    let symbol = normalize_symbol(&symbol);

    match storage.remove_holding(&actor, &symbol) {
        Ok(Some(portfolio)) => {
            clear_portfolio_actors_cache();
            Json(json!({
                "portfolio": portfolio,
                "summary": portfolio_summary(&actor, Some(&portfolio))
            }))
            .into_response()
        }
        Ok(None) => json_error(StatusCode::NOT_FOUND, format!("未找到持仓 {symbol}")),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

fn portfolio_storage(state: &AppState) -> PortfolioStorage {
    PortfolioStorage::new(&state.core.config.storage.portfolio_dir)
}

async fn list_cloud_portfolio_actor_summaries(
    postgres: CloudPgRuntime,
) -> HoneResult<Vec<PortfolioSummary>> {
    let records = tokio::time::timeout(Duration::from_secs(8), postgres.list_portfolios_cached())
        .await
        .map_err(|_| {
            hone_core::HoneError::Config("portfolio actors list timed out".to_string())
        })??;
    Ok(records
        .into_iter()
        .filter_map(portfolio_summary_from_cloud_record)
        .collect())
}

fn portfolio_summary_from_cloud_record(record: CloudPortfolioRecord) -> Option<PortfolioSummary> {
    let mut portfolio = serde_json::from_value::<Portfolio>(record.portfolio).ok()?;
    let actor = serde_json::from_value::<ActorIdentity>(record.actor).ok()?;
    portfolio.actor = Some(actor.clone());
    if portfolio.user_id.is_empty() {
        portfolio.user_id = actor.user_id.clone();
    }
    Some(portfolio_summary(&actor, Some(&portfolio)))
}

fn cached_portfolio_actors(allow_stale: bool) -> Option<serde_json::Value> {
    let guard = PORTFOLIO_ACTORS_CACHE.lock().ok()?;
    let updated_at = guard.updated_at?;
    if allow_stale || updated_at.elapsed() < PORTFOLIO_ACTORS_CACHE_TTL {
        return guard.value.clone();
    }
    None
}

fn mark_portfolio_actors_refreshing() -> bool {
    let Ok(mut guard) = PORTFOLIO_ACTORS_CACHE.lock() else {
        return true;
    };
    if guard.refreshing {
        return false;
    }
    guard.refreshing = true;
    true
}

fn clear_portfolio_actors_refreshing() {
    if let Ok(mut guard) = PORTFOLIO_ACTORS_CACHE.lock() {
        guard.refreshing = false;
    }
}

fn update_portfolio_actors_cache(value: serde_json::Value) {
    if let Ok(mut guard) = PORTFOLIO_ACTORS_CACHE.lock() {
        guard.value = Some(value);
        guard.updated_at = Some(Instant::now());
        guard.refreshing = false;
    }
}

fn clear_portfolio_actors_cache() {
    if let Ok(mut guard) = PORTFOLIO_ACTORS_CACHE.lock() {
        guard.value = None;
        guard.updated_at = None;
        guard.refreshing = false;
    }
}

fn is_tracking_only(holding: &Holding) -> bool {
    holding.tracking_only.unwrap_or(false)
}

fn portfolio_summary(actor: &ActorIdentity, portfolio: Option<&Portfolio>) -> PortfolioSummary {
    match portfolio {
        Some(portfolio) => {
            let holdings_count = portfolio
                .holdings
                .iter()
                .filter(|h| !is_tracking_only(h))
                .count();
            let watchlist_count = portfolio
                .holdings
                .iter()
                .filter(|h| is_tracking_only(h))
                .count();
            let total_shares = portfolio
                .holdings
                .iter()
                .filter(|h| !is_tracking_only(h))
                .map(|h| h.shares)
                .sum();
            PortfolioSummary {
                channel: actor.channel.clone(),
                user_id: actor.user_id.clone(),
                channel_scope: actor.channel_scope.clone(),
                holdings_count,
                watchlist_count,
                total_shares,
                updated_at: Some(portfolio.updated_at.clone()),
            }
        }
        None => PortfolioSummary {
            channel: actor.channel.clone(),
            user_id: actor.user_id.clone(),
            channel_scope: actor.channel_scope.clone(),
            holdings_count: 0,
            watchlist_count: 0,
            total_shares: 0.0,
            updated_at: None,
        },
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_uppercase()
}

fn normalize_asset_type(asset_type: Option<&str>) -> String {
    match asset_type.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value == "option" => "option".to_string(),
        _ => "stock".to_string(),
    }
}

fn normalize_optional_option_type(option_type: Option<String>) -> Option<String> {
    option_type
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .map(|value| match value.as_str() {
            "c" => "call".to_string(),
            "p" => "put".to_string(),
            _ => value,
        })
}

fn normalize_optional_holding_horizon(value: Option<String>) -> Option<String> {
    value.as_deref().and_then(normalize_holding_horizon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_memory::portfolio::{HOLDING_HORIZON_LONG_TERM, HOLDING_HORIZON_SHORT_TERM};

    #[test]
    fn normalize_optional_holding_horizon_accepts_aliases() {
        assert_eq!(
            normalize_optional_holding_horizon(Some("长持".to_string())).as_deref(),
            Some(HOLDING_HORIZON_LONG_TERM)
        );
        assert_eq!(
            normalize_optional_holding_horizon(Some("short".to_string())).as_deref(),
            Some(HOLDING_HORIZON_SHORT_TERM)
        );
        assert_eq!(
            normalize_optional_holding_horizon(Some("event-driven".to_string())),
            None
        );
        assert_eq!(normalize_optional_holding_horizon(None), None);
    }

    #[test]
    fn update_path_can_preserve_negative_avg_cost_and_strategy_fields() {
        let existing = Holding {
            symbol: "AAPL".to_string(),
            asset_type: "stock".to_string(),
            shares: 10.0,
            avg_cost: -1.25,
            underlying: None,
            option_type: None,
            strike_price: None,
            expiration_date: None,
            contract_multiplier: None,
            holding_horizon: Some(HOLDING_HORIZON_SHORT_TERM.to_string()),
            strategy_notes: Some("期权指派后遗留成本".to_string()),
            notes: Some("existing".to_string()),
            tracking_only: None,
        };

        let request = PortfolioHoldingRequest {
            channel: Some("discord".to_string()),
            user_id: Some("alice".to_string()),
            channel_scope: None,
            symbol: None,
            asset_type: Some("stock".to_string()),
            shares: None,
            avg_cost: None,
            quantity: Some(12.0),
            cost_basis: None,
            underlying: None,
            option_type: None,
            strike_price: None,
            expiration_date: None,
            contract_multiplier: None,
            holding_horizon: None,
            strategy_notes: None,
            notes: None,
            tracking_only: None,
        };

        let shares = request
            .quantity
            .or(request.shares)
            .unwrap_or(existing.shares);
        let avg_cost = request
            .cost_basis
            .or(request.avg_cost)
            .unwrap_or(existing.avg_cost);
        let holding_horizon = request.holding_horizon.or(existing.holding_horizon.clone());
        let strategy_notes = request.strategy_notes.or(existing.strategy_notes.clone());

        assert_eq!(shares, 12.0);
        assert_eq!(avg_cost, -1.25);
        assert_eq!(
            normalize_optional_holding_horizon(holding_horizon).as_deref(),
            Some(HOLDING_HORIZON_SHORT_TERM)
        );
        assert_eq!(
            normalize_optional_string(strategy_notes).as_deref(),
            Some("期权指派后遗留成本")
        );
    }

    #[test]
    fn tracking_only_excluded_from_total_shares() {
        let actor = ActorIdentity {
            channel: "telegram".to_string(),
            user_id: "bob".to_string(),
            channel_scope: None,
        };
        let portfolio = Portfolio {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            updated_at: "2026-04-22T00:00:00Z".to_string(),
            holdings: vec![
                Holding {
                    symbol: "AAPL".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 100.0,
                    avg_cost: 150.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                },
                Holding {
                    symbol: "NVDA".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 0.0,
                    avg_cost: 0.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: Some(true),
                },
                Holding {
                    symbol: "TSLA".to_string(),
                    asset_type: "stock".to_string(),
                    shares: 50.0,
                    avg_cost: 250.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                },
            ],
        };

        let summary = portfolio_summary(&actor, Some(&portfolio));
        assert_eq!(summary.total_shares, 150.0);
        assert_eq!(summary.holdings_count, 2);
        assert_eq!(summary.watchlist_count, 1);
    }
}
