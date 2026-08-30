//! GET /api/public/industry-map — AI 数据中心行业树。
//!
//! 树的模型、编译进二进制的底稿、以及管理员在对话里改出来的那份改动日志，都在
//! `hone_core::industry_map`：channels 进程做每轮注入时读的是同一份，两边不会各看各的。
//! 本模块只负责这一层加不上去的东西——成员公司的市值。
//!
//! 市值决定每一行的公司排序，所以必须取实时行情，不能写进静态文件（写进去当天就过期，
//! 而排序会跟着一起错）。取不到时整棵树照常返回，只是失去排序并在页面上说明。
//!
//! 树里只收美股（含 ADR）。非美股的同行——海力士、三星、中际旭创、鸿海——在产业上确实绕不开，
//! 但它们进不了本产品的判断范围（`hari-invest` 的适用范围是美国市场上市的公司、ETF 与 ADR），
//! 摆在表里只会让读者以为可以据此下判断。它们的供给格局作用由各行的传导链与关注点承载。

use std::collections::HashMap;
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

use hone_core::industry_map::{IndustryMap, IndustryMember, RECENT_EDIT_LIMIT};

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

/// 市值只用来排序，分钟级新鲜度足够；这条缓存同时挡住研究台反复打开时的重复外呼。
const MARKET_CAP_CACHE_TTL: Duration = Duration::from_secs(600);

/// 成员 + 本轮市值，`market_cap` 缺失表示上游本轮没有覆盖这一家。
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct MarketFact {
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
}

fn cache() -> &'static Mutex<Option<(Instant, HashMap<String, MarketFact>)>> {
    static CACHE: OnceLock<Mutex<Option<(Instant, HashMap<String, MarketFact>)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

/// GET /api/public/industry-map
pub(crate) async fn handle_get_industry_map(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers).await {
        return response;
    }

    let data_root = state.core.config.storage.data_root();
    let (map, edits) = hone_core::industry_map::load(&data_root);
    let last_edited = hone_core::industry_map::last_edited(&edits);
    let facts = market_facts(&state, &map).await;

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
                "last_edited_at": last_edited.get(&industry.id),
            })
        })
        .collect::<Vec<_>>();

    // 面板上的「最近改动」卡片：只要尾部几条，且只带摘要不带正文。
    let recent = edits
        .iter()
        .rev()
        .take(RECENT_EDIT_LIMIT)
        .map(|edit| {
            json!({
                "at": edit.at,
                "by": edit.by,
                "industry": edit.industry,
                "industry_name": map
                    .industry(&edit.industry)
                    .map(|item| item.name.clone())
                    .unwrap_or_else(|| edit.industry.clone()),
                "summary": edit.op.summary(),
                "note": edit.note,
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
        "recent_edits": recent,
        "edit_count": edits.len(),
    }))
    .into_response()
}

/// 按市值降序；本轮没取到市值的排在最后并保持文件里的原顺序，便于人工维护时对照。
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

/// 树里每一家成员的代码，去重后保持稳定顺序，用于一次批量取行情。
fn member_symbols(map: &IndustryMap) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for industry in &map.industries {
        for member in &industry.members {
            if !seen.iter().any(|item| item == &member.symbol) {
                seen.push(member.symbol.clone());
            }
        }
    }
    seen
}

async fn market_facts(state: &AppState, map: &IndustryMap) -> HashMap<String, MarketFact> {
    if let Some(cached) = cached_facts() {
        return cached;
    }
    let pool = state.core.config.fmp.effective_key_pool();
    let symbols = member_symbols(map);
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
    use hone_core::industry_map::base_map;

    #[test]
    fn every_member_is_a_plain_us_ticker() {
        // 树里只收美股（含 ADR）。带交易所后缀的代码取不到 FMP 行情，也不在本产品的判断
        // 范围内（`hari-invest` 的适用范围是美国市场上市的公司、ETF 与 ADR），摆进表里
        // 只会让读者以为可以据此下判断。
        for industry in &base_map().industries {
            for member in &industry.members {
                assert!(
                    !member.symbol.contains('.') && !member.symbol.contains(':'),
                    "{} 里的 {} 不是美股代码",
                    industry.id,
                    member.symbol
                );
                assert!(
                    member
                        .symbol
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'),
                    "{} 里的 {} 不是规范的美股代码写法",
                    industry.id,
                    member.symbol
                );
            }
        }
    }

    #[test]
    fn industry_ids_and_member_symbols_are_unique_within_their_scope() {
        let map = base_map();
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
        let members = vec![
            IndustryMember {
                symbol: "STX".into(),
                name: "希捷".into(),
                role: "近线 HDD".into(),
            },
            IndustryMember {
                symbol: "MU".into(),
                name: "美光".into(),
                role: "一体化".into(),
            },
            IndustryMember {
                symbol: "SNDK".into(),
                name: "闪迪".into(),
                role: "企业级 NAND".into(),
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
            ["MU", "SNDK", "STX"]
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
