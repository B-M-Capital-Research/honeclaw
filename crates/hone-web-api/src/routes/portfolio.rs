use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use hone_core::ActorIdentity;
use hone_memory::portfolio::{Holding, Portfolio, PortfolioStorage, normalize_holding_horizon};

use crate::routes::{json_error, normalize_optional_string, require_actor, require_string};
use crate::state::AppState;
use crate::types::{PortfolioHoldingRequest, PortfolioSummary, UserIdQuery};

/// GET /api/portfolio/actors — 列出所有有持仓数据的 actor
pub(crate) async fn handle_portfolio_actors(
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    let storage = portfolio_storage(&state);
    let all = storage.list_all();
    let summaries: Vec<PortfolioSummary> = all
        .iter()
        .map(|(actor, portfolio)| portfolio_summary(actor, Some(portfolio)))
        .collect();
    Json(json!({ "actors": summaries })).into_response()
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
        },
    ) {
        Ok(portfolio) => (
            StatusCode::CREATED,
            Json(json!({
                "portfolio": portfolio,
                "summary": portfolio_summary(&actor, Some(&portfolio))
            })),
        )
            .into_response(),
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
    let holding_horizon = req
        .holding_horizon
        .or(existing_holding.holding_horizon);
    let strategy_notes = req
        .strategy_notes
        .or(existing_holding.strategy_notes);

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
        },
    ) {
        Ok(portfolio) => Json(json!({
            "portfolio": portfolio,
            "summary": portfolio_summary(&actor, Some(&portfolio))
        }))
        .into_response(),
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
        Ok(Some(portfolio)) => Json(json!({
            "portfolio": portfolio,
            "summary": portfolio_summary(&actor, Some(&portfolio))
        }))
        .into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, format!("未找到持仓 {symbol}")),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

fn portfolio_storage(state: &AppState) -> PortfolioStorage {
    PortfolioStorage::new(&state.core.config.storage.portfolio_dir)
}

fn portfolio_summary(actor: &ActorIdentity, portfolio: Option<&Portfolio>) -> PortfolioSummary {
    match portfolio {
        Some(portfolio) => PortfolioSummary {
            channel: actor.channel.clone(),
            user_id: actor.user_id.clone(),
            channel_scope: actor.channel_scope.clone(),
            holdings_count: portfolio.holdings.len(),
            total_shares: portfolio.holdings.iter().map(|h| h.shares).sum(),
            updated_at: Some(portfolio.updated_at.clone()),
        },
        None => PortfolioSummary {
            channel: actor.channel.clone(),
            user_id: actor.user_id.clone(),
            channel_scope: actor.channel_scope.clone(),
            holdings_count: 0,
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
    value
        .as_deref()
        .and_then(normalize_holding_horizon)
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
        };

        let req = PortfolioHoldingRequest {
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
        };

        let shares = req.quantity.or(req.shares).unwrap_or(existing.shares);
        let avg_cost = req.cost_basis.or(req.avg_cost).unwrap_or(existing.avg_cost);
        let holding_horizon = req
            .holding_horizon
            .or(existing.holding_horizon.clone());
        let strategy_notes = req
            .strategy_notes
            .or(existing.strategy_notes.clone());

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
}
