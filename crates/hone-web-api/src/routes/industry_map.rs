//! GET /api/public/industry-map — AI 数据中心行业树。
//!
//! 树本身是静态研究资产（`skills/industry-map/references/industry-map.json`），与
//! `company-thesis-ratings` 的公司卡同一处理方式：`include_str!` 编进二进制，改数据要重新构建。
//! 唯一的动态部分是成员公司的市值——它决定每一行的公司排序，所以必须取实时行情，
//! 不能把市值写进静态文件（写进去当天就过期，而排序会跟着一起错）。
//!
//! 非美股上市的成员（韩交所的海力士、深交所的中际旭创等）拿不到 FMP 行情，它们排在
//! 有市值的公司之后并显式标注，而不是被悄悄丢掉：它们在树里的作用是解释这一行的供给格局。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::warn;

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

const INDUSTRY_MAP_JSON: &str =
    include_str!("../../../../skills/industry-map/references/industry-map.json");
/// 市值只用来排序，分钟级新鲜度足够；这条缓存同时挡住研究台反复打开时的重复外呼。
const MARKET_CAP_CACHE_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct IndustryRoot {
    pub id: String,
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub(crate) struct AiValuationLogic {
    #[serde(default)]
    pub driver_chain: String,
    #[serde(default)]
    pub key_variables: Vec<Value>,
    #[serde(default)]
    pub multiple_anchor: String,
    #[serde(default)]
    pub anti_pattern: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct IndustryMember {
    pub symbol: String,
    pub name: String,
    pub role: String,
    /// 缺省视为美股上市：树里绝大多数成员是美股，非美股的那几家显式写 false。
    #[serde(default = "default_true")]
    pub listed: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Industry {
    pub id: String,
    pub name: String,
    pub parent: String,
    #[serde(default)]
    pub one_liner: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub ai_valuation_logic: AiValuationLogic,
    #[serde(default)]
    pub core_watch: Vec<Value>,
    #[serde(default)]
    pub members: Vec<IndustryMember>,
    #[serde(default)]
    pub sources: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct IndustryMapFile {
    pub schema_version: u32,
    pub generated_at: String,
    pub root: IndustryRoot,
    pub industries: Vec<Industry>,
}

/// 成员 + 本轮市值，`market_cap` 缺失表示这一家本轮没有可用行情（非美股或上游未覆盖）。
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RankedMember {
    #[serde(flatten)]
    pub member: IndustryMember,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,
}

pub(crate) fn industry_map() -> &'static IndustryMapFile {
    static MAP: OnceLock<IndustryMapFile> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(INDUSTRY_MAP_JSON).expect("industry-map.json must parse at build time")
    })
}

/// 树里每一家美股成员的代码，去重后保持稳定顺序，用于一次批量取行情。
fn listed_symbols() -> Vec<String> {
    let mut seen = Vec::new();
    for industry in &industry_map().industries {
        for member in &industry.members {
            if member.listed && !seen.iter().any(|s: &String| s == &member.symbol) {
                seen.push(member.symbol.clone());
            }
        }
    }
    seen
}

fn cache() -> &'static Mutex<Option<(Instant, HashMap<String, MarketFact>)>> {
    static CACHE: OnceLock<Mutex<Option<(Instant, HashMap<String, MarketFact>)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MarketFact {
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
}

/// GET /api/public/industry-map
pub(crate) async fn handle_get_industry_map(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers).await {
        return response;
    }

    let facts = market_facts(&state).await;
    let map = industry_map();
    let industries = map
        .industries
        .iter()
        .map(|industry| {
            json!({
                "id": industry.id,
                "name": industry.name,
                "parent": industry.parent,
                "one_liner": industry.one_liner,
                "ai_valuation_logic": industry.ai_valuation_logic,
                "core_watch": industry.core_watch,
                "sources": industry.sources,
                "members": rank_members(&industry.members, &facts),
            })
        })
        .collect::<Vec<_>>();

    Json(json!({
        "available": true,
        "schema_version": map.schema_version,
        "generated_at": map.generated_at,
        "market_data_available": !facts.is_empty(),
        "root": map.root,
        "industries": industries,
    }))
    .into_response()
}

/// 按市值降序；没有市值的排在最后并保持文件里的原顺序，便于人工维护时对照。
pub(crate) fn rank_members(
    members: &[IndustryMember],
    facts: &HashMap<String, MarketFact>,
) -> Vec<RankedMember> {
    let mut ranked = members
        .iter()
        .map(|member| {
            let fact = facts.get(&member.symbol);
            RankedMember {
                member: member.clone(),
                market_cap: fact.and_then(|f| f.market_cap),
                price: fact.and_then(|f| f.price),
                change_percent: fact.and_then(|f| f.change_percent),
            }
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| match (a.market_cap, b.market_cap) {
        (Some(left), Some(right)) => right
            .partial_cmp(&left)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    ranked
}

async fn market_facts(state: &AppState) -> HashMap<String, MarketFact> {
    if let Some(cached) = cached_facts() {
        return cached;
    }
    let pool = state.core.config.fmp.effective_key_pool();
    let symbols = listed_symbols();
    if symbols.is_empty() || pool.keys().is_empty() {
        return HashMap::new();
    }
    match fetch_market_facts(state, pool.keys(), &symbols).await {
        Ok(facts) if !facts.is_empty() => {
            store_facts(&facts);
            facts
        }
        Ok(_) => HashMap::new(),
        Err(error) => {
            // 行情拿不到时仍然返回整棵树，只是失去市值排序——树的研究内容本身不依赖行情。
            warn!("industry map market caps unavailable: {error}");
            HashMap::new()
        }
    }
}

fn cached_facts() -> Option<HashMap<String, MarketFact>> {
    let cache = cache().lock().ok()?;
    let (at, facts) = cache.as_ref()?;
    (at.elapsed() < MARKET_CAP_CACHE_TTL).then(|| facts.clone())
}

fn store_facts(facts: &HashMap<String, MarketFact>) {
    if let Ok(mut cache) = cache().lock() {
        *cache = Some((Instant::now(), facts.clone()));
    }
}

async fn fetch_market_facts(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
) -> Result<HashMap<String, MarketFact>, String> {
    let joined = symbols.join(",");
    let encoded_symbols = utf8_percent_encode(&joined, NON_ALPHANUMERIC).to_string();
    let base = quote_base_url(&state.core.config.fmp.base_url);
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("{base}/v3/quote/{encoded_symbols}?apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await {
            Ok(value) => return Ok(facts_from_value(&value)),
            Err(error) => last_error = error,
        }
    }
    Err(if last_error.is_empty() {
        "FMP 请求失败".to_string()
    } else {
        last_error
    })
}

pub(crate) fn facts_from_value(value: &Value) -> HashMap<String, MarketFact> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.trim().to_string();
            if symbol.is_empty() {
                return None;
            }
            let finite = |key: &str| {
                item.get(key)
                    .and_then(Value::as_f64)
                    .filter(|value| value.is_finite())
            };
            Some((
                symbol,
                MarketFact {
                    market_cap: finite("marketCap"),
                    price: finite("price"),
                    change_percent: finite("changesPercentage"),
                },
            ))
        })
        .collect()
}

/// FMP 配置的 base_url 可能是 `.../api` 或 `.../api/v3`，统一成不带 `/v3` 的形态。
fn quote_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix("/v3").unwrap_or(base).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_industry_map_parses_and_hangs_every_industry_off_the_root() {
        let map = industry_map();
        assert_eq!(map.schema_version, 1);
        assert!(!map.industries.is_empty());
        for industry in &map.industries {
            assert_eq!(
                industry.parent, map.root.id,
                "{} 的 parent 必须是根节点",
                industry.id
            );
            assert!(!industry.members.is_empty(), "{} 没有成员", industry.id);
            assert!(!industry.aliases.is_empty(), "{} 没有别名", industry.id);
        }
    }

    #[test]
    fn industry_ids_and_member_symbols_are_unique_within_their_scope() {
        let map = industry_map();
        let mut ids = map.industries.iter().map(|i| &i.id).collect::<Vec<_>>();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "行业 id 重复");
        for industry in &map.industries {
            let mut symbols = industry
                .members
                .iter()
                .map(|m| &m.symbol)
                .collect::<Vec<_>>();
            symbols.sort();
            let before = symbols.len();
            symbols.dedup();
            assert_eq!(before, symbols.len(), "{} 内成员代码重复", industry.id);
        }
    }

    #[test]
    fn members_sort_by_market_cap_and_unpriced_members_keep_file_order_at_the_end() {
        // 非美股成员没有行情，它们必须留在表里（解释供给格局），只是排在有市值的后面。
        let members = vec![
            IndustryMember {
                symbol: "000660.KS".into(),
                name: "SK 海力士".into(),
                role: "HBM 份额领先".into(),
                listed: false,
            },
            IndustryMember {
                symbol: "MU".into(),
                name: "美光".into(),
                role: "一体化".into(),
                listed: true,
            },
            IndustryMember {
                symbol: "SNDK".into(),
                name: "闪迪".into(),
                role: "企业级 NAND".into(),
                listed: true,
            },
        ];
        let facts = HashMap::from([
            (
                "MU".to_string(),
                MarketFact {
                    market_cap: Some(1.0e12),
                    price: Some(932.86),
                    change_percent: Some(-0.27),
                },
            ),
            (
                "SNDK".to_string(),
                MarketFact {
                    market_cap: Some(8.5e10),
                    price: Some(106.23),
                    change_percent: Some(-6.22),
                },
            ),
        ]);
        let ranked = rank_members(&members, &facts);
        assert_eq!(
            ranked
                .iter()
                .map(|m| m.member.symbol.as_str())
                .collect::<Vec<_>>(),
            ["MU", "SNDK", "000660.KS"]
        );
        assert!(ranked[2].market_cap.is_none());
    }

    #[test]
    fn market_facts_ignore_rows_without_a_symbol_or_with_non_finite_numbers() {
        let value = json!([
            {"symbol": "MU", "marketCap": 1.0e12, "price": 932.86, "changesPercentage": -0.27},
            {"symbol": "", "marketCap": 1.0},
            {"marketCap": 2.0},
            {"symbol": "SNDK", "marketCap": null, "price": 106.23}
        ]);
        let facts = facts_from_value(&value);
        assert_eq!(facts.len(), 2);
        assert_eq!(facts["MU"].market_cap, Some(1.0e12));
        assert!(facts["SNDK"].market_cap.is_none());
        assert_eq!(facts["SNDK"].price, Some(106.23));
    }
}
