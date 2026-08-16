//! GET /api/public/quotes — 当前用户持仓的批量行情快照。
//!
//! 数据源为 FMP `/v3/quote/{SYMBOLS}`（key pool 轮换），带 60 秒进程内缓存，
//! 用于投资页展示现价与日涨跌。未配置 FMP key、持仓为空或上游全部失败时
//! 返回 `available: false`，前端据此隐藏行情区，不打断页面其余内容。

use std::sync::Arc;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Serialize;
use serde_json::{Value, json};
use tracing::warn;

use hone_core::ActorIdentity;

use crate::routes::public_finance_calendar::{fetch_fmp_json_once, portfolio_calendar_symbols};
use crate::state::AppState;

const QUOTE_CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct PublicQuote {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,
}

fn quote_cache() -> &'static Mutex<Option<(String, Instant, Vec<PublicQuote>)>> {
    static CACHE: OnceLock<Mutex<Option<(String, Instant, Vec<PublicQuote>)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

/// GET /api/public/quotes
pub(crate) async fn handle_get_quotes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => {
            return crate::routes::json_error(
                axum::http::StatusCode::BAD_REQUEST,
                error.to_string(),
            );
        }
    };

    let symbols = portfolio_calendar_symbols(&state, &actor).await;
    let pool = state.core.config.fmp.effective_key_pool();
    if symbols.is_empty() || pool.keys().is_empty() {
        return Json(json!({ "available": false, "quotes": [] })).into_response();
    }

    let cache_key = format!("{}:{}", actor.user_id, symbols.join(","));
    if let Some(quotes) = cached_quotes(&cache_key) {
        return Json(json!({ "available": true, "quotes": quotes })).into_response();
    }

    match fetch_quotes(&state, pool.keys(), &symbols).await {
        Ok(quotes) if !quotes.is_empty() => {
            store_quotes(&cache_key, &quotes);
            Json(json!({ "available": true, "quotes": quotes })).into_response()
        }
        Ok(_) => Json(json!({ "available": false, "quotes": [] })).into_response(),
        Err(error) => {
            warn!("public quotes FMP fetch failed: {error}");
            Json(json!({ "available": false, "quotes": [] })).into_response()
        }
    }
}

fn cached_quotes(cache_key: &str) -> Option<Vec<PublicQuote>> {
    let cache = quote_cache().lock().ok()?;
    let (key, at, quotes) = cache.as_ref()?;
    (key == cache_key && at.elapsed() < QUOTE_CACHE_TTL).then(|| quotes.clone())
}

fn store_quotes(cache_key: &str, quotes: &[PublicQuote]) {
    if let Ok(mut cache) = quote_cache().lock() {
        *cache = Some((cache_key.to_string(), Instant::now(), quotes.to_vec()));
    }
}

async fn fetch_quotes(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
) -> Result<Vec<PublicQuote>, String> {
    let joined = symbols.join(",");
    let encoded_symbols = utf8_percent_encode(&joined, NON_ALPHANUMERIC).to_string();
    let base = quote_base_url(&state.core.config.fmp.base_url);
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("{base}/v3/quote/{encoded_symbols}?apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await {
            Ok(value) => return Ok(quotes_from_value(&value)),
            Err(error) => last_error = error,
        }
    }
    Err(if last_error.is_empty() {
        "FMP 请求失败".to_string()
    } else {
        last_error
    })
}

/// FMP 配置的 base_url 可能是 `.../api` 或 `.../api/v3`，统一成不带 `/v3` 的形态。
fn quote_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix("/v3").unwrap_or(base).to_string()
}

fn quotes_from_value(value: &Value) -> Vec<PublicQuote> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.trim();
            let price = item.get("price")?.as_f64()?;
            if symbol.is_empty() || !price.is_finite() {
                return None;
            }
            Some(PublicQuote {
                symbol: symbol.to_string(),
                name: item
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_string),
                price,
                change: item.get("change").and_then(Value::as_f64),
                change_percent: item.get("changesPercentage").and_then(Value::as_f64),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{quote_base_url, quotes_from_value};
    use serde_json::json;

    #[test]
    fn quote_base_url_normalizes_v3_suffix() {
        assert_eq!(
            quote_base_url("https://financialmodelingprep.com/api"),
            "https://financialmodelingprep.com/api"
        );
        assert_eq!(
            quote_base_url("https://financialmodelingprep.com/api/v3/"),
            "https://financialmodelingprep.com/api"
        );
    }

    #[test]
    fn quotes_parse_and_skip_incomplete_rows() {
        let value = json!([
            {
                "symbol": "AAPL",
                "name": "Apple Inc.",
                "price": 231.5,
                "change": -1.2,
                "changesPercentage": -0.52
            },
            { "symbol": "", "price": 10.0 },
            { "symbol": "NOPRICE" },
        ]);

        let quotes = quotes_from_value(&value);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].symbol, "AAPL");
        assert_eq!(quotes[0].name.as_deref(), Some("Apple Inc."));
        assert_eq!(quotes[0].change_percent, Some(-0.52));
    }

    #[test]
    fn quotes_reject_non_array_payload() {
        assert!(quotes_from_value(&json!({ "Error Message": "x" })).is_empty());
    }
}
