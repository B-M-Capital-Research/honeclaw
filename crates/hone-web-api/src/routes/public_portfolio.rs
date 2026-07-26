//! 用户端「我的 · 自选与持仓」读写 API。
//!
//! - GET    /api/public/portfolio                     → 当前用户的自选与持仓（含折算后的仓位占比）
//! - POST   /api/public/portfolio/holdings            → 新增一条（持仓或自选）
//! - PUT    /api/public/portfolio/holdings/{symbol}   → 调整成本 / 占比 / 名称
//! - DELETE /api/public/portfolio/holdings/{symbol}   → 删除
//!
//! actor 一律由 session 推导，用户只能操作自己的组合。总条目上限
//! [`MAX_PORTFOLIO_ENTRIES`]，避免单用户无限增长拖垮下游蒸馏与推送。

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use hone_core::ActorIdentity;
use hone_memory::portfolio::{Holding, PortfolioStorage, holdings_with_weights};

use crate::routes::json_error;
use crate::state::AppState;

/// 自选 + 持仓的总条目上限。
pub(crate) const MAX_PORTFOLIO_ENTRIES: usize = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct PublicHoldingRequest {
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// 仓位占比(%)。缺省或 <= 0 视为「仅自选」。
    pub weight: Option<f64>,
    /// 成本价，可选。
    pub avg_cost: Option<f64>,
    pub notes: Option<String>,
}

fn require_actor(state: &AppState, headers: &HeaderMap) -> Result<ActorIdentity, Response> {
    let user = crate::routes::public::require_public_user(state, headers)?;
    ActorIdentity::new("web", &user.user_id, Option::<String>::None)
        .map_err(|error| json_error(StatusCode::BAD_REQUEST, error.to_string()))
}

fn storage(state: &AppState) -> PortfolioStorage {
    PortfolioStorage::new(&state.core.config.storage.portfolio_dir)
}

/// 代码规范化：去掉空白与非法字符并转大写，避免 `aapl ` / `AAPL` 存成两条。
pub(crate) fn normalize_symbol(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>()
        .to_ascii_uppercase()
}

fn holdings_payload(holdings: &[Holding]) -> Vec<serde_json::Value> {
    let weights = holdings_with_weights(holdings);
    holdings
        .iter()
        .zip(weights)
        .map(|(holding, weight)| {
            json!({
                "symbol": holding.symbol,
                "name": holding.name,
                "weight": weight,
                "avg_cost": (holding.avg_cost > 0.0).then_some(holding.avg_cost),
                "notes": holding.notes,
                "tracking_only": holding.tracking_only.unwrap_or(false) || weight.is_none(),
            })
        })
        .collect()
}

fn portfolio_response(holdings: &[Holding]) -> Response {
    Json(json!({
        "holdings": holdings_payload(holdings),
        "limit": MAX_PORTFOLIO_ENTRIES,
    }))
    .into_response()
}

/// GET /api/public/portfolio
pub(crate) async fn handle_get_portfolio(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let actor = match require_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let holdings = storage(&state)
        .load(&actor)
        .ok()
        .flatten()
        .map(|portfolio| portfolio.holdings)
        .unwrap_or_default();
    portfolio_response(&holdings)
}

/// POST /api/public/portfolio/holdings
pub(crate) async fn handle_create_holding(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicHoldingRequest>,
) -> Response {
    let actor = match require_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let symbol = normalize_symbol(request.symbol.as_deref().unwrap_or_default());
    if symbol.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "请填写股票代码");
    }

    let store = storage(&state);
    let existing = store
        .load(&actor)
        .ok()
        .flatten()
        .map(|portfolio| portfolio.holdings)
        .unwrap_or_default();
    let already_present = existing.iter().any(|item| item.symbol == symbol);
    if !already_present && existing.len() >= MAX_PORTFOLIO_ENTRIES {
        return json_error(
            StatusCode::BAD_REQUEST,
            format!("自选与持仓最多 {MAX_PORTFOLIO_ENTRIES} 条，请先删除一些再添加"),
        );
    }

    let holding = match build_holding(&symbol, &request, existing.iter().find(|h| h.symbol == symbol)) {
        Ok(holding) => holding,
        Err(response) => return response,
    };
    match store.upsert_holding(&actor, holding) {
        Ok(portfolio) => (StatusCode::CREATED, portfolio_response(&portfolio.holdings)).into_response(),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// PUT /api/public/portfolio/holdings/{symbol}
pub(crate) async fn handle_update_holding(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(symbol): Path<String>,
    Json(request): Json<PublicHoldingRequest>,
) -> Response {
    let actor = match require_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let symbol = normalize_symbol(&symbol);
    if symbol.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "请填写股票代码");
    }

    let store = storage(&state);
    let existing = store
        .load(&actor)
        .ok()
        .flatten()
        .map(|portfolio| portfolio.holdings)
        .unwrap_or_default();
    let Some(current) = existing.iter().find(|item| item.symbol == symbol) else {
        return json_error(StatusCode::NOT_FOUND, "没有找到这条自选或持仓");
    };

    let holding = match build_holding(&symbol, &request, Some(current)) {
        Ok(holding) => holding,
        Err(response) => return response,
    };
    match store.upsert_holding(&actor, holding) {
        Ok(portfolio) => portfolio_response(&portfolio.holdings),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// DELETE /api/public/portfolio/holdings/{symbol}
pub(crate) async fn handle_delete_holding(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(symbol): Path<String>,
) -> Response {
    let actor = match require_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let symbol = normalize_symbol(&symbol);
    let store = storage(&state);
    match store.remove_holding(&actor, &symbol) {
        // 已经不存在时返回当前列表即可 —— 删除是幂等的。
        Ok(Some(portfolio)) => portfolio_response(&portfolio.holdings),
        Ok(None) => {
            let holdings = store
                .load(&actor)
                .ok()
                .flatten()
                .map(|portfolio| portfolio.holdings)
                .unwrap_or_default();
            portfolio_response(&holdings)
        }
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// 组装写入用的 Holding：保留旧记录里前端不管理的字段（期权、期限、主线备注…），
/// 只覆盖用户在「我的」里能编辑的部分。
fn build_holding(
    symbol: &str,
    request: &PublicHoldingRequest,
    current: Option<&Holding>,
) -> Result<Holding, Response> {
    let weight = request
        .weight
        .filter(|value| value.is_finite() && *value > 0.0);
    if let Some(weight) = weight
        && weight > 100.0
    {
        return Err(json_error(StatusCode::BAD_REQUEST, "仓位占比不能超过 100%"));
    }
    let avg_cost = request
        .avg_cost
        .filter(|value| value.is_finite() && *value > 0.0);
    let name = request
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| current.and_then(|item| item.name.clone()));
    let notes = request
        .notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| current.and_then(|item| item.notes.clone()));

    Ok(Holding {
        symbol: symbol.to_string(),
        asset_type: current
            .map(|item| item.asset_type.clone())
            .unwrap_or_else(|| "stock".to_string()),
        // 用户端按比例管理，不再维护股数；旧记录的股数保留以免影响其它渠道读数。
        shares: current.map(|item| item.shares).unwrap_or(0.0),
        avg_cost: avg_cost.or_else(|| current.map(|item| item.avg_cost)).unwrap_or(0.0),
        underlying: current.and_then(|item| item.underlying.clone()),
        option_type: current.and_then(|item| item.option_type.clone()),
        strike_price: current.and_then(|item| item.strike_price),
        expiration_date: current.and_then(|item| item.expiration_date.clone()),
        contract_multiplier: current.and_then(|item| item.contract_multiplier),
        holding_horizon: current.and_then(|item| item.holding_horizon.clone()),
        strategy_notes: current.and_then(|item| item.strategy_notes.clone()),
        notes,
        weight,
        name,
        // 没给占比 = 仅自选。
        tracking_only: weight.is_none().then_some(true),
    })
}

#[cfg(test)]
mod tests {
    use super::{MAX_PORTFOLIO_ENTRIES, build_holding, normalize_symbol};
    use hone_memory::portfolio::{Holding, holdings_with_weights};

    fn request(weight: Option<f64>, avg_cost: Option<f64>) -> super::PublicHoldingRequest {
        super::PublicHoldingRequest {
            symbol: Some("aapl".to_string()),
            name: Some("Apple".to_string()),
            weight,
            avg_cost,
            notes: None,
        }
    }

    fn position(symbol: &str, shares: f64, avg_cost: f64, weight: Option<f64>) -> Holding {
        Holding {
            symbol: symbol.to_string(),
            asset_type: "stock".to_string(),
            shares,
            avg_cost,
            underlying: None,
            option_type: None,
            strike_price: None,
            expiration_date: None,
            contract_multiplier: None,
            holding_horizon: None,
            strategy_notes: None,
            notes: None,
            weight,
            name: None,
            tracking_only: None,
        }
    }

    #[test]
    fn symbols_are_normalized_to_uppercase() {
        assert_eq!(normalize_symbol("  aapl "), "AAPL");
        assert_eq!(normalize_symbol("brk.b"), "BRK.B");
        assert_eq!(normalize_symbol("!!"), "");
    }

    #[test]
    fn missing_weight_marks_the_entry_as_watchlist_only() {
        let holding = build_holding("AAPL", &request(None, None), None).expect("holding");
        assert_eq!(holding.tracking_only, Some(true));
        assert_eq!(holding.weight, None);
        assert_eq!(holding.name.as_deref(), Some("Apple"));
    }

    #[test]
    fn weight_over_one_hundred_is_rejected() {
        assert!(build_holding("AAPL", &request(Some(120.0), None), None).is_err());
    }

    #[test]
    fn editing_keeps_fields_the_web_form_does_not_manage() {
        let mut current = position("AAPL", 10.0, 100.0, None);
        current.holding_horizon = Some("long_term".to_string());
        current.strategy_notes = Some("核心仓".to_string());

        let holding = build_holding("AAPL", &request(Some(25.0), Some(180.0)), Some(&current))
            .expect("holding");

        assert_eq!(holding.weight, Some(25.0));
        assert_eq!(holding.avg_cost, 180.0);
        assert_eq!(holding.holding_horizon.as_deref(), Some("long_term"));
        assert_eq!(holding.strategy_notes.as_deref(), Some("核心仓"));
        assert_eq!(holding.tracking_only, None);
    }

    #[test]
    fn legacy_share_based_holdings_are_converted_to_percentages() {
        let holdings = vec![
            position("AAPL", 10.0, 100.0, None), // 成本 1000
            position("MSFT", 10.0, 300.0, None), // 成本 3000
        ];

        let weights = holdings_with_weights(&holdings);
        assert_eq!(weights[0], Some(25.0));
        assert_eq!(weights[1], Some(75.0));
    }

    #[test]
    fn explicit_weights_cap_the_derived_share_and_watchlist_stays_empty() {
        let mut watch = position("TSLA", 0.0, 0.0, None);
        watch.tracking_only = Some(true);
        let holdings = vec![
            position("AAPL", 0.0, 0.0, Some(60.0)),
            position("MSFT", 10.0, 100.0, None),
            watch,
        ];

        let weights = holdings_with_weights(&holdings);
        assert_eq!(weights[0], Some(60.0));
        // 剩余 40% 全部给唯一一条待折算的持仓。
        assert_eq!(weights[1], Some(40.0));
        assert_eq!(weights[2], None);
    }

    #[test]
    fn entry_limit_is_fifty() {
        assert_eq!(MAX_PORTFOLIO_ENTRIES, 50);
    }
}
