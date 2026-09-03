//! GET /api/public/industry-map — AI 数据中心行业树。
//!
//! 树的模型、编译进二进制的底稿、以及管理员在对话里改出来的那份改动日志，都在
//! `hone_core::industry_map`：channels 进程做每轮注入时读的是同一份，两边不会各看各的。
//! 本模块只负责这一层加不上去的两件东西——成员公司的市值，和它们的**官方股本**。
//!
//! 市值决定每一行的公司排序，所以必须取实时行情，不能写进静态文件（写进去当天就过期，
//! 而排序会跟着一起错）。取不到时整棵树照常返回，只是失去排序并在页面上说明。
//!
//! 官方股本来自 `hone_memory::CompanyFactsStorage`（web 侧 worker 每天 19:00 刷）。
//! 挂它的理由不是「多一列数据」：提供方的 `sharesOutstanding` 会整整落后一份申报，
//! 2026-08 的 LITE 就是这样让市值低估了 13.3%。所以只要官方封面股数口径成立、
//! 且本轮有价格，这一行就**同时给出两个市值**：按现价 × 官方股本重算的
//! `market_cap`（排序用的就是它）和提供方原样的 `provider_market_cap`。
//! 只覆盖不并列，页面上的市值会和任何外部站点都对不上，而读的人拿不到提供方数字
//! 去核对差在哪；两个都给，`market_cap_basis` 说明哪个是哪个。
//!
//! 口径判断（20-F 不参与重算、封面过期不参与重算、与提供方差到倍数级不参与重算）
//! **只有一处**：`hone_core::sec_shares`。库里那个 `cover_usable_for_market_cap`
//! 是 worker 写入当天求值的布尔，worker 一停它就会一直说 true，而同一份响应里的
//! `freshness` 已经写着 `cover_stale`——所以这里一律读取时现算，与对话侧
//! `data_fetch` 用同一个函数、同一个「今天」。
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

use chrono::{DateTime, NaiveDate, Utc};
use hone_core::industry_map::{
    EditOp, IndustryEdit, IndustryMap, IndustryMember, RECENT_EDIT_LIMIT,
};
use hone_core::sec_shares::{
    CoverShareRow, SEC_BASIS_MISMATCH_RATIO, cover_is_usable_for_market_cap,
    form_is_us_domestic_periodic,
};
use hone_memory::CompanyFacts;
use serde::Deserialize;

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

/// 市值只用来排序，分钟级新鲜度足够；这条缓存同时挡住研究台反复打开时的重复外呼。
const MARKET_CAP_CACHE_TTL: Duration = Duration::from_secs(600);

/// 官方股本一天才刷一次，但这是个公开端点，刷新一次页面就重打一次库（本地文件路径下
/// 是 59 次同步读直接跑在 handler 里）。TTL 取 15 分钟：既让 worker 跑完后的新股本
/// 一刻钟内就能看见，也把这条路径压到每小时最多 4 次。
const MEMBER_SHARES_CACHE_TTL: Duration = Duration::from_secs(900);

/// 成员 + 本轮市值 + 官方股本。`market_cap` 缺失表示上游本轮没有覆盖这一家。
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RankedMember {
    #[serde(flatten)]
    pub member: IndustryMember,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    /// 这一行的市值是怎么来的：`price_x_official_shares`（官方封面股数重算）
    /// 或 `provider`（提供方原样）。不写出来，读的人无从判断该不该信。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap_basis: Option<&'static str>,
    /// 提供方自己那个市值，只在**与 `market_cap` 不同**（即本行做了重算）时出现。
    /// 覆盖而不并列会让页面上的市值和任何外部站点都对不上，读的人也无从核对差在哪。
    /// 基准是 `provider` 时两者相等，再写一遍只是噪音，所以那种情况不写。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_market_cap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,
    /// 官方股本与新鲜度。没有（还没刷到、或属于拿不到封面股数的那 8 家）时不出现。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_outstanding: Option<Value>,
}

/// 一家成员在共享事实库里的官方股本切片。**全部字段都是读取时现算的**，
/// 库里那些写入当天求值的派生布尔一个都不读——见 [`MemberShares::from_facts`]。
#[derive(Debug, Clone, Default)]
pub(crate) struct MemberShares {
    pub official: Option<i64>,
    pub cover_end: Option<String>,
    pub cover_filed: Option<String>,
    pub cover_form: Option<String>,
    pub usable_for_market_cap: bool,
    pub cover_age_days: Option<i64>,
    pub cover_is_stale: bool,
    pub provider_difference_pct: Option<f64>,
    pub absent_reason: Option<String>,
    pub absent_note: Option<String>,
    pub facts_refreshed_at: Option<String>,
    pub facts_are_stale: bool,
}

/// 这一行的市值该怎么来。
#[derive(Debug, Clone, Copy)]
enum MarketCapBasis {
    /// 现价 × 官方封面股数。
    Recomputed(f64),
    /// 有官方股数但这一轮不能用它重算，附上原因（写进响应，别让读的人猜）。
    Blocked(&'static str),
    /// 没有官方股数，或本轮没有价格。
    NotApplicable,
}

impl MemberShares {
    /// 从事实库的一行现算出这一轮的口径结论。
    ///
    /// 口径判断走 [`cover_is_usable_for_market_cap`]——和对话侧 `data_fetch` 是
    /// 同一个函数、同一个「今天」。库里的 `cover_usable_for_market_cap` 是 worker
    /// 写入那一刻求值的：一行写入时封面 195 天、之后 worker 停了，读的时候封面已经
    /// 260 天，那个布尔仍是 true，于是照样按官方股本重算并据此排序，而同一份 JSON 里
    /// `freshness` 写着 `cover_stale`。一份响应自相矛盾，且排序键用的是被自己标为
    /// 陈旧的数——所以那个字段这里一律不读。
    fn from_facts(facts: &CompanyFacts, today: NaiveDate, now: DateTime<Utc>) -> Self {
        let shares = &facts.shares;
        let row = CoverShareRow {
            shares: shares.cover_shares.unwrap_or_default(),
            end: shares.cover_end.clone().unwrap_or_default(),
            filed: shares.cover_filed.clone().unwrap_or_default(),
            form: shares.cover_form.clone().unwrap_or_default(),
            accn: String::new(),
        };
        Self {
            official: shares.cover_shares,
            cover_end: shares.cover_end.clone(),
            cover_filed: shares.cover_filed.clone(),
            cover_form: shares.cover_form.clone(),
            usable_for_market_cap: shares.cover_shares.is_some()
                && cover_is_usable_for_market_cap(&row, today),
            cover_age_days: shares.cover_age_days(today),
            cover_is_stale: shares.cover_is_stale(today),
            provider_difference_pct: shares.provider_difference_pct,
            absent_reason: shares.cover_absent_reason.clone(),
            absent_note: shares.cover_absent_note.clone(),
            facts_refreshed_at: Some(facts.refreshed_at.clone())
                .filter(|value| !value.trim().is_empty()),
            facts_are_stale: facts.is_stale(now),
        }
    }

    /// 这一行的市值口径。
    ///
    /// 两道门，缺一不可：
    /// 1. 封面口径本身要成立（美国国内定期申报 + 未过期），现算。
    /// 2. 官方股数与**提供方市值隐含的股数**不能差到倍数级。多类别股发行人的
    ///    concept 可能只返回其中一类，ADR 与本土普通股更是整数倍关系——这种时候
    ///    重算比不重算危险得多（TSM 那一档会错 5 倍）。阈值与
    ///    `sec_shares::SEC_BASIS_MISMATCH_RATIO` 同一个数，因为它判的是同一件事：
    ///    这不是提供方过期，是两边根本不是一个口径。
    fn market_cap_basis(
        &self,
        price: Option<f64>,
        provider_market_cap: Option<f64>,
    ) -> MarketCapBasis {
        let Some(official) = self.official else {
            return MarketCapBasis::NotApplicable;
        };
        if !self.usable_for_market_cap {
            // 口径先于新鲜度：外国私人发行人一年才报一次，封面天然是旧的，
            // 说它「过期」会让人以为等一份新申报就能用——不能用的是口径本身。
            let domestic = self
                .cover_form
                .as_deref()
                .is_some_and(form_is_us_domestic_periodic);
            return MarketCapBasis::Blocked(if domestic {
                "cover_stale"
            } else {
                "basis_not_us_domestic_periodic"
            });
        }
        let Some(price) = price.filter(|value| value.is_finite() && *value > 0.0) else {
            return MarketCapBasis::NotApplicable;
        };
        let implied_provider_shares = provider_market_cap
            .filter(|value| value.is_finite() && *value > 0.0)
            .map(|cap| cap / price);
        if let Some(implied) = implied_provider_shares {
            let ratio = official as f64 / implied;
            if !ratio.is_finite()
                || ratio >= SEC_BASIS_MISMATCH_RATIO
                || ratio <= 1.0 / SEC_BASIS_MISMATCH_RATIO
            {
                return MarketCapBasis::Blocked("basis_mismatch_suspected");
            }
        }
        MarketCapBasis::Recomputed(price * official as f64)
    }

    fn to_json(&self, blocked_reason: Option<&'static str>) -> Option<Value> {
        // 既没有官方数、也说不出为什么没有 —— 这一家还没被刷到，什么都不写，
        // 免得前端把「没跑过」显示成「查不到」。
        if self.official.is_none() && self.absent_reason.is_none() {
            return None;
        }
        let mut payload = json!({
            "available": self.official.is_some(),
            "usable_for_market_cap": self.usable_for_market_cap,
        });
        if let Some(official) = self.official {
            payload["official_shares_outstanding"] = Value::from(official);
        }
        for (key, value) in [
            ("cover_date", self.cover_end.as_ref()),
            ("filed", self.cover_filed.as_ref()),
            ("form", self.cover_form.as_ref()),
            ("absent_reason", self.absent_reason.as_ref()),
            ("absent_note", self.absent_note.as_ref()),
            ("facts_refreshed_at", self.facts_refreshed_at.as_ref()),
        ] {
            if let Some(value) = value {
                payload[key] = Value::String(value.clone());
            }
        }
        if let Some(age) = self.cover_age_days {
            payload["cover_age_days"] = Value::from(age);
        }
        if let Some(difference) = self.provider_difference_pct {
            payload["provider_difference_pct"] = Value::from(difference);
        }
        // 有官方股数却没拿它重算时，把原因写出来——否则读的人只看到一个
        // market_cap_basis: "provider" 和一个明明存在的官方股数，无从判断是漏了还是拦了。
        if let Some(reason) = blocked_reason {
            payload["recompute_blocked_reason"] = Value::String(reason.to_string());
        }
        // 新鲜度分两层：封面日期本身有多旧（口径新鲜度），和我们上次刷这一行是什么时候
        // （管道新鲜度）。混成一个字段就分不清是公司没申报还是 worker 没跑。
        payload["freshness"] = Value::String(
            match (
                self.official.is_some(),
                self.cover_is_stale,
                self.facts_are_stale,
            ) {
                (false, _, _) => "unavailable",
                (true, true, _) => "cover_stale",
                (true, false, true) => "pipeline_stale",
                (true, false, false) => "fresh",
            }
            .to_string(),
        );
        Some(payload)
    }
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
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let is_admin = state
        .web_auth
        .is_web_admin(&user.user_id)
        .await
        .unwrap_or(false);

    let data_root = state.core.config.storage.data_root();
    let (map, edits) = hone_core::industry_map::load(&data_root);
    let last_edited = hone_core::industry_map::last_edited(&edits);
    let facts = market_facts(&state, &map).await;
    let shares = member_shares(&data_root, &map).await;
    let official_shares_available = shares.values().any(|item| item.official.is_some());

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
                "upstream_signals": industry.upstream_signals,
                "members": rank_members(&industry.members, &facts, &shares),
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
        "official_shares_available": official_shares_available,
        "shares_policy": "shares_outstanding.official_shares_outstanding 是监管申报封面上的官方已发行股数，比提供方数字权威——提供方会整整落后一份申报。market_cap_basis 为 price_x_official_shares 的行，market_cap 已按现价 × 官方股本重算，同一行的 provider_market_cap 是提供方原样的市值，两个口径并列给出、排序用重算值；为 provider 的行只有提供方市值一个数。有官方股数却没重算的行会写出 recompute_blocked_reason：cover_stale（封面日期已过期）、basis_not_us_domestic_periodic（20-F 等外国私人发行人报的是本土普通股而非 ADR 股数）、basis_mismatch_suspected（官方股数与提供方隐含股数差到倍数级，通常是多类别股只统计其中一类）。这三种情况都不得用官方股数推算美股市值。",
        "root": map.root,
        "industries": industries,
        "recent_edits": recent,
        "edit_count": edits.len(),
        "is_admin": is_admin,
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct IndustryEditRequest {
    pub industry: String,
    #[serde(default)]
    pub note: String,
    pub op: EditOp,
}

/// POST /api/public/industry-map/edits —— 管理员在页面上直接改本体。
///
/// 与对话里的 `industry_map_edit` 工具写同一份追加式日志，走同一个 `append`（先在重放结果上
/// 试跑、被拒的不写日志）。响应带回完整快照，页面用它整体替换本地状态，「最近改动」卡片
/// 与树上的标记跟着一起更新。
pub(crate) async fn handle_post_industry_edit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    // 先鉴权再解析：请求体的形状错误只回给已登录的管理员，未登录的调用者一律 401。
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let is_admin = state
        .web_auth
        .is_web_admin(&user.user_id)
        .await
        .unwrap_or(false);
    if !is_admin {
        return crate::routes::json_error(
            axum::http::StatusCode::FORBIDDEN,
            "只有管理员可以改行业本体".to_string(),
        );
    }
    let request: IndustryEditRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return crate::routes::json_error(
                axum::http::StatusCode::BAD_REQUEST,
                format!("改动请求无法解析：{error}"),
            );
        }
    };
    let industry = match &request.op {
        EditOp::AddIndustry { industry } => industry.id.trim().to_string(),
        _ => request.industry.trim().to_string(),
    };
    if industry.is_empty() {
        return crate::routes::json_error(
            axum::http::StatusCode::BAD_REQUEST,
            "缺少 industry".to_string(),
        );
    }
    if let EditOp::AddMember { member } = &request.op {
        if member.symbol.contains('.') || member.symbol.contains(':') {
            return crate::routes::json_error(
                axum::http::StatusCode::BAD_REQUEST,
                format!("{} 不是美股代码：行业树只收美股与 ADR", member.symbol),
            );
        }
    }
    let edit = IndustryEdit {
        at: hone_core::local_now().to_rfc3339(),
        by: user.user_id.clone(),
        industry,
        op: request.op,
        note: request.note.trim().to_string(),
    };
    let applied = edit.op.summary();
    if let Err(error) =
        hone_core::industry_map::append(&state.core.config.storage.data_root(), edit)
    {
        return crate::routes::json_error(axum::http::StatusCode::BAD_REQUEST, error.to_string());
    }
    // 直接复用 GET 的组装（含市值与官方股本），保证页面替换进去的快照和刷新看到的一样。
    let snapshot = handle_get_industry_map(State(state), headers).await;
    let body = match axum::body::to_bytes(snapshot.into_body(), usize::MAX).await {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    Json(json!({ "ok": true, "applied": applied, "snapshot": body })).into_response()
}

/// 按市值降序；本轮没取到市值的排在最后并保持文件里的原顺序，便于人工维护时对照。
///
/// 官方股本口径成立且本轮有价格时，`market_cap` 是**现价 × 官方股本**、
/// `provider_market_cap` 是提供方原样，两个并列给出。**排序用重算值**：提供方的股本
/// 会整整落后一份申报，按它排就是把 LITE 那 13.3% 的低估直接排进顺序里；两个数字并列
/// 之后，读的人仍拿得到提供方口径去核对差在哪。20-F、过期封面、以及与提供方差到倍数级的
/// 行一律不重算，并在 `shares_outstanding.recompute_blocked_reason` 里说明为什么。
pub(crate) fn rank_members(
    members: &[IndustryMember],
    facts: &HashMap<String, MarketFact>,
    shares: &HashMap<String, MemberShares>,
) -> Vec<RankedMember> {
    let mut ranked = members
        .iter()
        .map(|member| {
            let fact = facts.get(&member.symbol);
            let price = fact.and_then(|f| f.price);
            let provider_market_cap = fact.and_then(|f| f.market_cap);
            let member_shares = shares.get(&member.symbol);
            let basis = member_shares
                .map(|shares| shares.market_cap_basis(price, provider_market_cap))
                .unwrap_or(MarketCapBasis::NotApplicable);
            let (market_cap, basis_label, blocked_reason) = match basis {
                MarketCapBasis::Recomputed(value) => {
                    (Some(value), Some("price_x_official_shares"), None)
                }
                MarketCapBasis::Blocked(reason) => (
                    provider_market_cap,
                    provider_market_cap.map(|_| "provider"),
                    Some(reason),
                ),
                MarketCapBasis::NotApplicable => (
                    provider_market_cap,
                    provider_market_cap.map(|_| "provider"),
                    None,
                ),
            };
            RankedMember {
                member: member.clone(),
                market_cap,
                market_cap_basis: basis_label,
                // 只在两个数字真的不同（本行做了重算）时并列，基准是 provider 时
                // 两者相等，再写一遍只是噪音。
                provider_market_cap: provider_market_cap
                    .filter(|_| matches!(basis, MarketCapBasis::Recomputed(_))),
                price,
                change_percent: fact.and_then(|f| f.change_percent),
                shares_outstanding: member_shares.and_then(|shares| shares.to_json(blocked_reason)),
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

/// 每个成员的官方股本切片。事实库为空（worker 还没跑过、本地开发）时返回空表，
/// 整棵树照常返回——官方股本是加信息的，不是渲染这个页面的前提。
///
/// 缓存的是**事实库那一行**，不是算完的结论：新鲜度和口径要用「现在」求值，
/// 缓存住结论就等于把这次修掉的那个「写入时求值」的毛病换个地方再犯一次。
async fn member_shares(
    data_root: &std::path::Path,
    map: &IndustryMap,
) -> HashMap<String, MemberShares> {
    let symbols = member_symbols(map);
    if symbols.is_empty() {
        return HashMap::new();
    }
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let rows = match cached_member_facts() {
        Some(rows) => rows,
        None => {
            let storage = hone_memory::CompanyFactsStorage::new(data_root);
            let rows = Arc::new(storage.load_many(&symbols).await);
            // 空表不缓存：worker 第一次写完之后应当立刻看得见，而空表本身的查询代价
            // 就是一次命不中的查询。
            if !rows.is_empty() {
                store_member_facts(&rows);
            }
            rows
        }
    };
    rows.iter()
        .map(|(symbol, facts)| (symbol.clone(), MemberShares::from_facts(facts, today, now)))
        .collect()
}

type MemberFactsCache = Mutex<Option<(Instant, Arc<HashMap<String, CompanyFacts>>)>>;

fn member_facts_cache() -> &'static MemberFactsCache {
    static CACHE: OnceLock<MemberFactsCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn cached_member_facts() -> Option<Arc<HashMap<String, CompanyFacts>>> {
    let cache = member_facts_cache().lock().ok()?;
    let (at, rows) = cache.as_ref()?;
    (at.elapsed() < MEMBER_SHARES_CACHE_TTL).then(|| rows.clone())
}

fn store_member_facts(rows: &Arc<HashMap<String, CompanyFacts>>) {
    if let Ok(mut cache) = member_facts_cache().lock() {
        *cache = Some((Instant::now(), rows.clone()));
    }
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
        let ranked = rank_members(&members, &facts, &HashMap::new());
        assert_eq!(
            ranked
                .iter()
                .map(|m| m.member.symbol.as_str())
                .collect::<Vec<_>>(),
            ["MU", "SNDK", "STX"]
        );
        assert!(ranked[2].market_cap.is_none());
    }

    /// 页面发来的 body 与对话工具写进日志的是同一个 `EditOp`：`kind` 标签、字段名、嵌套结构
    /// 都必须能直接反序列化，否则前端契约和日志格式会悄悄分叉。
    #[test]
    fn edit_request_body_deserializes_every_op_kind_the_page_can_send() {
        let bodies = [
            r#"{"industry":"storage","note":"n","op":{"kind":"set_field","field":"one_liner","value":"x"}}"#,
            r#"{"industry":"storage","op":{"kind":"add_member","member":{"symbol":"MU","name":"美光","role":"r"}}}"#,
            r#"{"industry":"storage","op":{"kind":"remove_member","symbol":"MU"}}"#,
            r#"{"industry":"storage","op":{"kind":"set_member_role","symbol":"MU","role":"r"}}"#,
            r#"{"industry":"storage","op":{"kind":"add_source","source":{"house":"h","title":"t","date":"2026-09","url":"u","takeaway":"k"}}}"#,
            r#"{"industry":"storage","op":{"kind":"remove_source","url":"u"}}"#,
            r#"{"industry":"storage","op":{"kind":"add_watch","watch":{"what":"w","why":"y","cadence":"c"}}}"#,
            r#"{"industry":"storage","op":{"kind":"remove_watch","what":"w"}}"#,
            r#"{"industry":"storage","op":{"kind":"add_upstream_signal","signal":{"symbol":"NVDA","name":"英伟达","relation":"demand_source","why":"y","pull":["a","b"],"cadence":"q"}}}"#,
            r#"{"industry":"storage","op":{"kind":"remove_upstream_signal","symbol":"NVDA"}}"#,
            r#"{"industry":"storage","op":{"kind":"set_upstream_latest","symbol":"NVDA","latest":"FY27Q2：数据中心 $89.0B","as_of":"2026-08-26"}}"#,
            r#"{"industry":"cooling","op":{"kind":"add_industry","industry":{"id":"cooling","name":"散热","one_liner":"","aliases":["散热"]}}}"#,
            r#"{"industry":"cooling","op":{"kind":"remove_industry"}}"#,
        ];
        for body in bodies {
            let parsed: Result<IndustryEditRequest, _> = serde_json::from_str(body);
            assert!(parsed.is_ok(), "{body}: {:?}", parsed.err());
        }
        let bad: Result<IndustryEditRequest, _> =
            serde_json::from_str(r#"{"industry":"storage","op":{"kind":"nuke_everything"}}"#);
        assert!(bad.is_err());
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

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 30).expect("fixed test date")
    }

    fn now() -> DateTime<Utc> {
        "2026-08-30T12:00:00Z"
            .parse::<DateTime<Utc>>()
            .expect("fixed test time")
    }

    /// 造一行事实库记录。**库里那个 `cover_usable_for_market_cap` 一律设成 true**：
    /// 它是 worker 写入当天求值的布尔，读取侧如果还在读它，下面几条断言就会掉。
    fn facts_row(symbol: &str, official: i64, form: &str, cover_end: &str) -> CompanyFacts {
        let mut facts = CompanyFacts::new(symbol);
        facts.shares.cover_shares = Some(official);
        facts.shares.cover_end = Some(cover_end.to_string());
        facts.shares.cover_filed = Some("2026-08-17".to_string());
        facts.shares.cover_form = Some(form.to_string());
        facts.shares.cover_usable_for_market_cap = true;
        facts.shares.provider_difference_pct = Some(15.3);
        facts.refreshed_at = "2026-08-30T11:00:00Z".to_string();
        facts
    }

    fn shares_of(facts: &CompanyFacts) -> MemberShares {
        MemberShares::from_facts(facts, today(), now())
    }

    fn member(symbol: &str) -> IndustryMember {
        IndustryMember {
            symbol: symbol.into(),
            name: symbol.into(),
            role: "成员".into(),
        }
    }

    fn fact(market_cap: f64, price: f64) -> MarketFact {
        MarketFact {
            market_cap: Some(market_cap),
            price: Some(price),
            change_percent: Some(1.2),
        }
    }

    #[test]
    fn an_official_share_count_rewrites_the_market_cap_and_therefore_the_ranking() {
        // LITE：提供方股本落后一份申报，市值低估 13.3%。排序按市值来，
        // 所以股本修对之后这一行的位置也跟着修对。
        let members = vec![member("LITE"), member("SNDK")];
        let facts = HashMap::from([
            ("LITE".to_string(), fact(69_631_000_000.0, 895.0)),
            ("SNDK".to_string(), fact(75_000_000_000.0, 106.23)),
        ]);
        let shares = HashMap::from([(
            "LITE".to_string(),
            shares_of(&facts_row("LITE", 89_700_000, "10-K", "2026-08-14")),
        )]);

        let ranked = rank_members(&members, &facts, &shares);
        // 用提供方市值 LITE 排第二；用官方股本重算（895 × 89.7M = 802.8 亿）排第一。
        assert_eq!(ranked[0].member.symbol, "LITE");
        assert_eq!(ranked[0].market_cap, Some(80_281_500_000.0));
        assert_eq!(ranked[0].market_cap_basis, Some("price_x_official_shares"));
        // 提供方那个市值必须仍然在响应里：只覆盖不并列，页面上的市值会和任何外部
        // 站点都对不上，而读的人拿不到提供方数字去核对差在哪。
        assert_eq!(ranked[0].provider_market_cap, Some(69_631_000_000.0));
        assert_eq!(ranked[1].market_cap_basis, Some("provider"));
        assert_eq!(
            ranked[1].provider_market_cap, None,
            "基准就是 provider 时两个数字相等，再写一遍只是噪音"
        );
        assert!(
            ranked[1].shares_outstanding.is_none(),
            "没刷到的家不写这一块"
        );

        let block = ranked[0].shares_outstanding.as_ref().expect("shares block");
        assert_eq!(block["official_shares_outstanding"], 89_700_000);
        assert_eq!(block["cover_date"], "2026-08-14");
        assert_eq!(block["form"], "10-K");
        assert_eq!(block["cover_age_days"], 16);
        assert_eq!(block["provider_difference_pct"], 15.3);
        assert_eq!(block["freshness"], "fresh");
        assert!(block.get("recompute_blocked_reason").is_none());
    }

    #[test]
    fn a_20f_share_count_is_shown_but_never_used_to_recompute_a_market_cap() {
        // TSM 的官方封面数是台股普通股口径，1 ADR = 5 股：拿它乘美股股价会错 5 倍。
        let members = vec![member("TSM")];
        let facts = HashMap::from([("TSM".to_string(), fact(1.55e12, 300.0))]);
        let mut row = facts_row("TSM", 25_932_524_521, "20-F", "2025-12-31");
        row.shares.provider_difference_pct = None;
        let shares = HashMap::from([("TSM".to_string(), shares_of(&row))]);

        let ranked = rank_members(&members, &facts, &shares);
        assert_eq!(ranked[0].market_cap, Some(1.55e12));
        assert_eq!(ranked[0].market_cap_basis, Some("provider"));
        assert_eq!(ranked[0].provider_market_cap, None);
        let block = ranked[0].shares_outstanding.as_ref().expect("shares block");
        assert_eq!(block["official_shares_outstanding"], 25_932_524_521_i64);
        assert_eq!(block["usable_for_market_cap"], false);
        assert_eq!(
            block["recompute_blocked_reason"],
            "basis_not_us_domestic_periodic"
        );
    }

    /// worker 停跑之后的那一行：库里写着 `usable_for_market_cap: true`（写入那天算的），
    /// 封面已经 260 天。读取侧必须现算、判它不可用、退回提供方市值——否则同一份 JSON
    /// 会一边写 `freshness: cover_stale`、一边拿这张过期封面重算市值并据此排序。
    #[test]
    fn a_cover_page_that_went_stale_after_it_was_written_falls_back_to_the_provider() {
        let members = vec![member("BRK-B")];
        let facts = HashMap::from([("BRK-B".to_string(), fact(1.0e12, 500.0))]);
        // 2025-12-13 距 2026-08-30 是 260 天，超过 SEC_MAX_COVER_AGE_DAYS(200)。
        let row = facts_row("BRK-B", 2_000_000_000, "10-Q", "2025-12-13");
        assert!(
            row.shares.cover_usable_for_market_cap,
            "库里存的是写入当天的结论：true"
        );
        let shares = HashMap::from([("BRK-B".to_string(), shares_of(&row))]);

        let ranked = rank_members(&members, &facts, &shares);
        assert_eq!(ranked[0].market_cap, Some(1.0e12));
        assert_eq!(ranked[0].market_cap_basis, Some("provider"));
        let block = ranked[0].shares_outstanding.as_ref().expect("shares block");
        assert_eq!(block["cover_age_days"], 260);
        assert_eq!(block["freshness"], "cover_stale");
        // 同一份 JSON 里这两个字段不可能再打架：两者出自同一个年龄判断。
        assert_eq!(block["usable_for_market_cap"], false);
        assert_eq!(block["recompute_blocked_reason"], "cover_stale");
    }

    /// 国内申报人的 concept 只返回其中一类股时，官方数会比提供方隐含股数小一大截。
    /// 这不是「提供方过期」，是两边不是一个口径——照算会把排序一起改错。
    #[test]
    fn a_share_count_that_differs_by_a_multiple_is_treated_as_a_different_basis() {
        let members = vec![member("MULTI")];
        // 提供方市值隐含 10 亿股，官方 concept 只给出 4 亿股（2.5 倍差）。
        let facts = HashMap::from([("MULTI".to_string(), fact(100.0e9, 100.0))]);
        let shares = HashMap::from([(
            "MULTI".to_string(),
            shares_of(&facts_row("MULTI", 400_000_000, "10-K", "2026-08-14")),
        )]);

        let ranked = rank_members(&members, &facts, &shares);
        assert_eq!(ranked[0].market_cap, Some(100.0e9));
        assert_eq!(ranked[0].market_cap_basis, Some("provider"));
        let block = ranked[0].shares_outstanding.as_ref().expect("shares block");
        assert_eq!(block["usable_for_market_cap"], true, "口径本身是成立的");
        assert_eq!(
            block["recompute_blocked_reason"],
            "basis_mismatch_suspected"
        );

        // 同样的行，差异只有 15.3%（LITE 那一档）时照常重算。
        let facts = HashMap::from([("MULTI".to_string(), fact(34.7e9, 100.0))]);
        let ranked = rank_members(&members, &facts, &shares);
        assert_eq!(ranked[0].market_cap, Some(40.0e9));
        assert_eq!(ranked[0].market_cap_basis, Some("price_x_official_shares"));
    }

    #[test]
    fn freshness_separates_a_stale_cover_page_from_a_stale_pipeline() {
        let fresh = shares_of(&facts_row("LITE", 89_700_000, "10-K", "2026-08-14"));
        assert_eq!(fresh.to_json(None).expect("json")["freshness"], "fresh");

        // 公司自己很久没申报了。
        let stale_cover = shares_of(&facts_row("LITE", 89_700_000, "10-K", "2025-12-13"));
        assert_eq!(
            stale_cover.to_json(None).expect("json")["freshness"],
            "cover_stale"
        );

        // 封面是新的，但我们的 worker 连着两天没跑。
        let mut row = facts_row("LITE", 89_700_000, "10-K", "2026-08-14");
        row.refreshed_at = "2026-08-28T11:00:00Z".to_string();
        let stale_pipeline = shares_of(&row);
        assert!(stale_pipeline.facts_are_stale);
        assert_eq!(
            stale_pipeline.to_json(None).expect("json")["freshness"],
            "pipeline_stale"
        );

        // 拿不到官方数的那 8 家：说明原因，标成 unavailable。
        let absent = MemberShares {
            absent_reason: Some("concept_not_found".to_string()),
            absent_note: Some("多类别股发行人把封面股数打在 axis 维度上。".to_string()),
            ..MemberShares::default()
        };
        let json = absent.to_json(None).expect("json");
        assert_eq!(json["available"], false);
        assert_eq!(json["freshness"], "unavailable");
        assert!(json["absent_note"].as_str().expect("note").contains("axis"));

        // 还没刷到的家什么都不写，免得前端把「没跑过」显示成「查不到」。
        assert!(MemberShares::default().to_json(None).is_none());
    }
}
