//! 官方封面股数通道（SEC XBRL `dei:EntityCommonStockSharesOutstanding`）
//!
//! 上半部分是**纯函数**：CIK 补零、ticker→CIK 对照表解析、封面股数选最新一条、
//! 与 provider `sharesOutstanding` 的交叉校验。口径规则全部落在这里，可以被单元
//! 测试直接盯住，不用起网络。下半部分是 [`SecSharesClient`]——一层薄 HTTP 包装 +
//! 进程级缓存，把「取不到」按类型分成 [`SecSharesAbsence`]（正常降级）和
//! [`SecSharesError`]（真失败）。
//!
//! 放在 `hone-core` 而不是 `data_fetch.rs`，是因为现在有两个消费者：对话侧的
//! `data_fetch` 工具，和 web 侧每天刷 `company_facts` 的 worker。两边必须用同一套
//! 口径规则——尤其是 20-F 那道门——否则一边挂告警、另一边拿台股股数算 ADR 市值。
//!
//! 为什么需要它：provider 的 `sharesOutstanding` 会整整落后一份申报。2026-08-30
//! 的 LITE 事故里，provider 仍在报 2026-04-30 那份 10-Q 的封面数 77,800,000，
//! 而 2026-08-17 报出的 10-K 封面数已经是 89,700,000——市值因此低估 13.3%，
//! 由目标市值反推的每股目标价高估 15.3%。SEC 的这条免费公开端点不需要 API Key，
//! 是全链路唯一能证伪 provider 股本的一手来源。

use chrono::NaiveDate;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration as StdDuration;

/// ticker → CIK 对照表。约 776 KB / 10391 条，服务端支持 gzip（约 214 KB）。
pub const SEC_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
/// companyconcept 根路径；完整 URL 见 [`concept_url`]。
pub const SEC_COMPANYCONCEPT_BASE: &str = "https://data.sec.gov/api/xbrl/companyconcept";
/// 封面股数这个 XBRL 概念的分类与标签，大小写敏感。
pub const SEC_SHARES_CONCEPT_PATH: &str = "dei/EntityCommonStockSharesOutstanding";

/// 对照表 TTL。这张表本身按天级刷新，但 ticker→CIK 的映射几乎不动；退化方向
/// 是安全的（新上市代码最多晚 7 天进表，期间只是取不到官方股数、回落到 provider），
/// 而重下一次要 214 KB，所以宁可长。
pub const SEC_TTL_CIK_MAP: StdDuration = StdDuration::from_secs(7 * 24 * 60 * 60);
/// companyconcept TTL。封面股数一个季度才动一次，6 小时既不会让当天报出的
/// 10-K 迟到太久，也不会让每轮 snapshot 都打一次 SEC。
pub const SEC_TTL_CONCEPT: StdDuration = StdDuration::from_secs(6 * 60 * 60);
/// 404 的负缓存 TTL。多类别股发行人（GOOGL/META/DELL/CRWV/NBIS）是**长期** 404，
/// 不缓存等于每轮 snapshot 都替它们白打一次 SEC。
pub const SEC_TTL_CONCEPT_ABSENT: StdDuration = StdDuration::from_secs(24 * 60 * 60);

/// 单次 SEC 请求超时。比 FMP 的超时短：官方股本是锦上添花的交叉校验，
/// 不允许它决定 snapshot 的延迟。
pub const SEC_REQUEST_TIMEOUT_SECS: u64 = 8;
/// 整条 SEC 通道（对照表 + concept，最坏两次 8s 串行）的总预算。
///
/// 由 [`SecSharesClient::cover_shares`] / [`SecSharesClient::cover_shares_for_cik`]
/// **自己**罩上，不指望每个调用方记得包一层 `timeout`——对话侧的 snapshot 是在
/// 关键路径上串行 await 这条通道的，一个只写在注释里的预算比没有预算更误导。
/// 超时按传输失败处理：静默降级 + 进负缓存。
pub const SEC_SNAPSHOT_BUDGET_SECS: u64 = 12;

/// 传输类失败的负缓存 TTL。
///
/// 404 的负缓存是 24 小时（那是长期状态）；这一条针对的是 429 / 5xx / 超时 /
/// 挡板页这类**瞬时**故障：不缓存的话，SEC 一挂，每一次 snapshot 和一轮 sweep 里的
/// 每一家都要各自重新付满一次超时预算，而且永远不会收敛——退避是给对面留活路，
/// 也是给我们自己的延迟设上限。5 分钟足够短到故障一恢复就自动接上。
pub const SEC_TTL_FAILURE: StdDuration = StdDuration::from_secs(5 * 60);

/// 封面日期超过这个天数就不再用于交叉校验。美国国内定期申报人每季度都报
/// 10-Q，最新一条正常在 130 天以内（实测 51 家中位 34 天）；超过 200 天说明
/// 这家已经不按季申报，或者拿到的是多年前的历史口径（BRK 的最新一条停在
/// 2011-04-29）。HTTP 200 不等于数据新鲜，新鲜度必须盯 `end`。
pub const SEC_MAX_COVER_AGE_DAYS: i64 = 200;

/// 触发交叉校验块的相对偏差阈值。
///
/// 取 3% 的理由有两条：
/// 1. 与 `skills/valuation-audit/SKILL.md` 对账表里「市值与 quote 市值互验，
///    偏差 > 3% 要解释」用同一个数——工具和 skill 对「多大算不一致」保持一套标准。
/// 2. 回购与增发让股本逐季小幅漂移（激进回购约 1%/季，ESPP/RSU 增发同量级），
///    阈值定在 1% 会天天报警、把真信号淹掉；而本次事故这种「落后一整份申报」
///    的缺口是 15.3%，3% 有充分余量咬住它。
pub const SEC_SHARES_DISCREPANCY_THRESHOLD: f64 = 0.03;

/// 两个股本数字相差到这个倍数以上时，判为**口径不同**而不是 provider 过期。
/// ADR 与本土普通股（TSM 是 1:5）、多类别股只统计其中一类，都会造成整数倍
/// 级别的差距；这种情况下断言「provider 落后」并让模型拿官方数重算市值，
/// 比原事故更危险（TSM 会错 5 倍）。
pub const SEC_BASIS_MISMATCH_RATIO: f64 = 1.5;

/// 一条封面股数事实。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverShareRow {
    pub shares: i64,
    /// 封面日期（申报封面上「截至该日已发行股数」的那个日期）。
    pub end: String,
    pub filed: String,
    pub form: String,
    pub accn: String,
}

/// CIK 必须补零到 10 位，且前缀是大写 `CIK`。实测 `CIK1633978`、
/// `cik0001633978`、`CIK00001633978` 一律 404，只有 `CIK0001633978` 返回 200。
pub fn cik_path_segment(cik: u64) -> String {
    format!("CIK{cik:010}")
}

pub fn concept_url(base: &str, cik: u64) -> String {
    format!(
        "{}/{}/{}.json",
        base.trim_end_matches('/'),
        cik_path_segment(cik),
        SEC_SHARES_CONCEPT_PATH
    )
}

/// SEC 的 ticker 全部大写，且用 `-` 分隔股份类别（`BRK-B`），而 provider 常写
/// `BRK.B` 或 `BRK/B`。这里只做大小写与分隔符归一，不猜别名。
pub fn normalize_sec_ticker(raw: &str) -> String {
    raw.trim()
        .to_ascii_uppercase()
        .chars()
        .map(|c| if c == '.' || c == '/' { '-' } else { c })
        .collect()
}

/// `company_tickers.json` 是一个顶层 **对象**，键是无意义的连续下标，值形如
/// `{"cik_str":1045810,"ticker":"NVDA","title":"NVIDIA CORP"}`。ticker 在这份
/// 文件里全局唯一，所以 ticker→CIK 无损；反向不成立（1443 个 CIK 挂着多个
/// ticker，GOOG/GOOGL 共用一个 CIK），因此 concept 的缓存键用 CIK 而不是 ticker。
pub fn parse_ticker_cik_map(value: &Value) -> HashMap<String, u64> {
    let rows: Vec<&Value> = match value {
        Value::Object(map) => map.values().collect(),
        Value::Array(rows) => rows.iter().collect(),
        _ => return HashMap::new(),
    };
    let mut index = HashMap::with_capacity(rows.len());
    for row in rows {
        let Some(ticker) = row.get("ticker").and_then(Value::as_str) else {
            continue;
        };
        let Some(cik) = row.get("cik_str").and_then(sec_cik_number) else {
            continue;
        };
        let ticker = normalize_sec_ticker(ticker);
        if ticker.is_empty() {
            continue;
        }
        index.insert(ticker, cik);
    }
    index
}

/// `cik_str` 实测是 JSON 数字，但历史上这个字段名暗示过字符串，两种都收。
fn sec_cik_number(value: &Value) -> Option<u64> {
    if let Some(number) = value.as_u64() {
        return Some(number);
    }
    value
        .as_str()
        .and_then(|text| text.trim().parse::<u64>().ok())
}

/// 从 companyconcept 响应里取出全部封面股数事实。
///
/// 两个必须防的形状：
/// - `units.shares` 在 HTTP 200 下可能是**空对象** `{}`（实测 AMKR / BE / GLW），
///   所以不能用 `HashMap<String, Vec<Row>>` 反序列化，必须走 `as_array()`。
/// - `frame` 约 10% 的行没有，`fy`/`fp` 不可靠（UCTT 把同一个 `end` 同时标成
///   Q1 和 Q3），两者都不能进排序键。
pub fn parse_cover_rows(concept: &Value) -> Vec<CoverShareRow> {
    let Some(units) = concept.get("units").and_then(Value::as_object) else {
        return Vec::new();
    };
    let rows = units
        .get("shares")
        .or_else(|| units.values().next())
        .and_then(Value::as_array);
    let Some(rows) = rows else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|row| {
            let shares = row.get("val").and_then(cover_share_count)?;
            let end = row.get("end").and_then(Value::as_str)?.trim();
            if end.is_empty() {
                return None;
            }
            Some(CoverShareRow {
                shares,
                end: end.to_string(),
                filed: row
                    .get("filed")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                form: row
                    .get("form")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                accn: row
                    .get("accn")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect()
}

fn cover_share_count(value: &Value) -> Option<i64> {
    if let Some(count) = value.as_i64() {
        return (count > 0).then_some(count);
    }
    let count = value.as_f64()?.round();
    (count > 0.0 && count < i64::MAX as f64).then_some(count as i64)
}

/// 选最新一条，外加**上一个封面日期**的那条供模型看变化。
///
/// 排序键必须是 `(end, filed, accn)` 三段：
/// - 只按 `filed` 排是错的——STX 在 2012-10-31 重述了 `end=2012-04-25`，那个
///   `filed` 比好几个更新 `end` 的申报还晚，取 max(filed) 会拿到过期的封面日。
/// - `(end, filed)` 也不唯一——UCTT 有两行 `end` 与 `filed` 完全相同、只有
///   `accn` 不同，少了第三段选择就不确定。
/// - 同 `end` 不同 `val` 是真实存在的更正（SNDK 的 10-Q/A 把 114,863,251 改成
///   144,863,251），后报出的那条才对，所以 `filed` 大的胜出。
///
/// 「上一条」取的是 `end` 严格更早的最新一条，不是数组里的前一行——前一行可能
/// 只是同一封面日期的重述，摆出来看不出变化。
pub fn latest_and_previous(
    rows: &[CoverShareRow],
) -> Option<(CoverShareRow, Option<CoverShareRow>)> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|left, right| {
        left.end
            .cmp(&right.end)
            .then_with(|| left.filed.cmp(&right.filed))
            .then_with(|| left.accn.cmp(&right.accn))
    });
    let latest = sorted.last()?.clone();
    let previous = sorted
        .iter()
        .rev()
        .find(|row| row.end < latest.end)
        .cloned();
    Some((latest, previous))
}

/// 交叉校验只在美国国内定期申报（10-K / 10-Q，含 `/A` 修订）上成立。
///
/// 20-F / 40-F 报的是**本土市场普通股**，不是美股 ADR 股数：TSM 官方封面数
/// 259.3 亿是台股口径，1 ADR = 5 股普通股，拿它去校验或重算 ADR 市值会错 5 倍，
/// 比本次 LITE 落后一份申报的 13% 严重得多。而且外国私人发行人一年只报一次，
/// 「官方」在这里往往比 provider 更旧，不构成更权威。
pub fn form_is_us_domestic_periodic(form: &str) -> bool {
    let base = form
        .split('/')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    matches!(base.as_str(), "10-K" | "10-Q")
}

/// 封面日期距今天数；`end` 解析不出来时返回 None。
pub fn cover_age_days(end: &str, today: NaiveDate) -> Option<i64> {
    NaiveDate::parse_from_str(end.trim(), "%Y-%m-%d")
        .ok()
        .map(|date| (today - date).num_days())
}

/// 官方封面股数是否可以直接用来重算美股市值。
pub fn cover_is_usable_for_market_cap(row: &CoverShareRow, today: NaiveDate) -> bool {
    form_is_us_domestic_periodic(&row.form)
        && cover_age_days(&row.end, today).is_none_or(|age| age <= SEC_MAX_COVER_AGE_DAYS)
}

fn cover_row_json(row: &CoverShareRow, today: NaiveDate) -> Value {
    let mut payload = serde_json::json!({
        "shares_outstanding": row.shares,
        "cover_date": row.end,
        "filed": row.filed,
        "form": row.form,
        "accession_number": row.accn,
    });
    if let Some(age) = cover_age_days(&row.end, today) {
        payload["cover_age_days"] = Value::from(age);
    }
    payload
}

/// `shares_outstanding` data_type 的返回体。
pub fn build_shares_outstanding_payload(
    ticker: &str,
    cik: u64,
    entity_name: Option<&str>,
    latest: &CoverShareRow,
    previous: Option<&CoverShareRow>,
    today: NaiveDate,
) -> Value {
    let domestic = form_is_us_domestic_periodic(&latest.form);
    let usable = cover_is_usable_for_market_cap(latest, today);
    let mut data = serde_json::json!({
        "cik": cik_path_segment(cik),
        "latest": cover_row_json(latest, today),
        "basis": if domestic { "us_domestic_periodic_cover_page" } else { "foreign_or_non_periodic_filing" },
        "usable_for_market_cap": usable,
    });
    if let Some(name) = entity_name.map(str::trim).filter(|name| !name.is_empty()) {
        data["entity_name"] = Value::String(name.to_string());
    }
    if let Some(previous) = previous {
        data["previous"] = cover_row_json(previous, today);
    }

    let mut payload = serde_json::json!({
        "data_type": "shares_outstanding",
        "ticker": ticker,
        "data": data,
        "hone_shares_outstanding_semantics": "这是监管申报封面上「截至封面日期已发行普通股」的官方股数，不是加权平均股本、也不是完全摊薄股本：算市值和 EV 用它，算 EPS 用报表里的加权稀释股本。provider 的股本可能整整落后一份申报，两者不一致时以封面日期更新的为准。",
    });

    if previous.is_some() {
        payload["hone_shares_change_policy"] = Value::String(
            "previous 只是上一个封面日期的同一口径数字，摆出来是让你看得见变化，不是让你算变动百分比：股本序列可能因口径切换（ADR 与本土普通股、股份类别重述）出现数量级跳变，跨口径的「变动率」是假的。真要讲股本变化，先确认两条的 form 与口径一致。".to_string(),
        );
    }

    if !domestic {
        payload["hone_shares_basis_warning"] = Value::String(format!(
            "最新一条来自 {}，报的是发行人**本土市场的普通股**股数，不是美股 ADR/ADS 股数（比例常见 1:1 以外，如 1 ADR 兑 5 股普通股）。不得用它乘以美股股价推算市值、也不得用它去校正提供方的股本；要给美股市值口径，请另找 ADR 比例或直接用提供方数字并标注口径未核验。",
            latest.form
        ));
    } else if !usable {
        payload["hone_shares_basis_warning"] = Value::String(
            "最新一条的封面日期已经太旧，不能当作当期股本使用：HTTP 成功不代表数据新鲜，这家可能已经不按季申报或返回的是历史口径。请按「本轮未核验」处理。".to_string(),
        );
    }

    payload
}

/// 从 provider 的 quote 响应里取 `sharesOutstanding`（v3 quote 是数组）。
pub fn provider_shares_outstanding(quote: &Value) -> Option<f64> {
    provider_quote_number(quote, "sharesOutstanding")
}

pub fn provider_quote_number(quote: &Value, field: &str) -> Option<f64> {
    let row = match quote {
        Value::Array(rows) => rows.first()?,
        other => other,
    };
    row.get(field)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

/// provider 股本与官方封面股数的交叉校验。
///
/// 只在**该挂块**时返回 `Some`：口径成立（美国国内定期申报）、数据新鲜、
/// 两个数字都拿得到、且相对偏差超过阈值。口径不成立或两者一致时返回 `None`，
/// snapshot 保持原样——这条通道是加信息的，不是加噪音的。
pub fn build_shares_cross_check(
    provider_shares: f64,
    provider_market_cap: Option<f64>,
    price: Option<f64>,
    latest: &CoverShareRow,
    today: NaiveDate,
) -> Option<Value> {
    if !(provider_shares.is_finite() && provider_shares > 0.0) {
        return None;
    }
    if !cover_is_usable_for_market_cap(latest, today) {
        return None;
    }
    let official = latest.shares as f64;
    let relative = (official - provider_shares) / provider_shares;
    if relative.abs() < SEC_SHARES_DISCREPANCY_THRESHOLD {
        return None;
    }

    let ratio = official / provider_shares;
    let basis_mismatch =
        ratio >= SEC_BASIS_MISMATCH_RATIO || ratio <= 1.0 / SEC_BASIS_MISMATCH_RATIO;

    let mut block = serde_json::json!({
        "status": if basis_mismatch { "basis_mismatch_suspected" } else { "provider_behind_official_filing" },
        "provider_shares_outstanding": provider_shares,
        "official_shares_outstanding": latest.shares,
        "official_cover_date": latest.end,
        "official_filed": latest.filed,
        "official_form": latest.form,
        "difference_pct": (relative * 10_000.0).round() / 100.0,
    });
    if let Some(market_cap) = provider_market_cap.filter(|value| value.is_finite() && *value > 0.0)
    {
        block["provider_market_cap"] = Value::from(market_cap);
    }

    if basis_mismatch {
        block["note"] = Value::String(format!(
            "提供方股本与官方封面股数相差 {ratio:.2} 倍，这个量级通常是口径不同（ADR 与本土普通股、或多类别股只统计其中一类），不是提供方过期。不要直接拿其中任何一个乘股价重算市值；本轮把股本口径标为未核验，或另找一手来源确认后再算倍数。"
        ));
    } else {
        block["note"] = Value::String(format!(
            "提供方的股本落后于最近一期定期报告封面数（差 {:.2}%）。市值、EV，以及所有以股本为分母或乘数的倍数与每股目标价，一律改用官方封面股数 {} 股（截至 {}，{}，{} 报出）重算，不要沿用提供方的市值字段。写给用户时把这一行的来源写成「最近一期定期报告封面股数（截至 {}）」。",
            relative * 100.0,
            latest.shares,
            latest.end,
            latest.form,
            latest.filed,
            latest.end
        ));
        if let Some(price) = price.filter(|value| value.is_finite() && *value > 0.0) {
            block["recomputed_market_cap"] = Value::from((price * official).round());
            block["recomputed_market_cap_basis"] = Value::String("现价 × 官方封面股数".to_string());
        }
    }

    Some(block)
}

// ── HTTP 通道 ────────────────────────────────────────────────────────────────

/// SEC 通道真正失败了（网络、超时、5xx、响应不是 JSON）。调用方一律降级，
/// 不得让它影响 quote / snapshot / 研究台的既有路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecSharesError(pub String);

impl std::fmt::Display for SecSharesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SecSharesError {}

/// 「这家拿不到官方封面股数」的三种**正常**情形。它们不是错误：
/// 实测 59 家里有 8 家（AMKR BE CRWV DELL GLW GOOGL META NBIS）长期走到这里，
/// 多类别股发行人把封面股数打在 axis 维度上，`companyconcept` 不返回带维度的事实。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecSharesAbsence {
    /// `company_tickers.json` 里没有这个代码（新上市最多晚 7 天进表）。
    UnknownTicker,
    /// concept 端点 404。
    ConceptNotFound,
    /// HTTP 200，但 `units.shares` 是空对象或没有可用行。
    NoCoverRows,
}

impl SecSharesAbsence {
    pub fn as_str(self) -> &'static str {
        match self {
            SecSharesAbsence::UnknownTicker => "unknown_ticker",
            SecSharesAbsence::ConceptNotFound => "concept_not_found",
            SecSharesAbsence::NoCoverRows => "no_cover_rows",
        }
    }

    /// 给写进 `company_facts` 的一句人话，说明为什么这家没有官方股本。
    pub fn note(self) -> &'static str {
        match self {
            SecSharesAbsence::UnknownTicker => {
                "SEC 的 ticker→CIK 对照表里没有这个代码（新上市代码最多晚几天进表）。"
            }
            SecSharesAbsence::ConceptNotFound | SecSharesAbsence::NoCoverRows => {
                "SEC 的 companyconcept 端点没有这家的封面股数事实——多类别股发行人把封面股数打在 axis 维度上，该端点不返回带维度的事实。这不是故障，本轮股本以提供方数字为准并标注未经官方核验。"
            }
        }
    }
}

/// 一次取数的结果：要么拿到封面股数，要么是一个**说得清原因**的缺失。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecSharesOutcome {
    Cover(Box<SecCoverShares>),
    Absent(SecSharesAbsence),
}

/// 某个 CIK 的最新封面股数，外加上一个封面日期的那条。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecCoverShares {
    pub cik: u64,
    pub entity_name: Option<String>,
    pub latest: CoverShareRow,
    pub previous: Option<CoverShareRow>,
}

impl SecCoverShares {
    /// 这条封面数能不能直接乘股价算美股市值（美国国内定期申报 + 未过期）。
    pub fn usable_for_market_cap(&self, today: NaiveDate) -> bool {
        cover_is_usable_for_market_cap(&self.latest, today)
    }

    /// 口径标签。20-F/40-F 报的是本土普通股，只标注、不参与任何校验。
    pub fn basis(&self) -> &'static str {
        if form_is_us_domestic_periodic(&self.latest.form) {
            "us_domestic_periodic_cover_page"
        } else {
            "foreign_or_non_periodic_filing"
        }
    }

    pub fn age_days(&self, today: NaiveDate) -> Option<i64> {
        cover_age_days(&self.latest.end, today)
    }
}

/// 进程级缓存。对照表 214 KB、concept 每家一次，而 `DataFetchTool` 是每轮对话重建的，
/// 缓存挂在实例上等于没有缓存——所以这两张表是模块级的。
type CikMapCache =
    std::sync::Mutex<Option<(std::time::Instant, std::sync::Arc<HashMap<String, u64>>)>>;
type ConceptCache =
    std::sync::Mutex<HashMap<u64, (std::time::Instant, Option<std::sync::Arc<Value>>)>>;

/// 传输类失败的负缓存：对照表一个时刻，concept 按 CIK 各一个。
type FailureCache = std::sync::Mutex<Option<std::time::Instant>>;
type ConceptFailureCache = std::sync::Mutex<HashMap<u64, std::time::Instant>>;

fn cik_map_cache() -> &'static CikMapCache {
    static CACHE: std::sync::OnceLock<CikMapCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

fn concept_cache() -> &'static ConceptCache {
    static CACHE: std::sync::OnceLock<ConceptCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn cik_map_failure_cache() -> &'static FailureCache {
    static CACHE: std::sync::OnceLock<FailureCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

fn concept_failure_cache() -> &'static ConceptFailureCache {
    static CACHE: std::sync::OnceLock<ConceptFailureCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// 测试用：把三张进程级缓存清空，免得同一进程里的用例互相看见对方的负缓存。
#[doc(hidden)]
pub fn reset_sec_caches_for_tests() {
    if let Ok(mut guard) = cik_map_cache().lock() {
        *guard = None;
    }
    if let Ok(mut guard) = concept_cache().lock() {
        guard.clear();
    }
    if let Ok(mut guard) = cik_map_failure_cache().lock() {
        *guard = None;
    }
    if let Ok(mut guard) = concept_failure_cache().lock() {
        guard.clear();
    }
}

/// User-Agent 里那个邮箱的域名是不是一个**收不到信**的占位符。
///
/// SEC 要求 UA 带联系方式，目的是出问题时能找到人。`ops@honeclaw.local` 这种
/// 既满足「非空」又满足「含 @」，却指向一个按 RFC 2606 / 6761 永远不会被解析的
/// 域名——发出去等于告诉 SEC「查无此人」，换来的是限流甚至封禁。
fn is_placeholder_contact_domain(domain: &str) -> bool {
    const RESERVED_SUFFIXES: [&str; 6] = [
        ".local",
        ".localhost",
        ".invalid",
        ".test",
        ".example",
        ".internal",
    ];
    const RESERVED_DOMAINS: [&str; 4] = ["example.com", "example.net", "example.org", "localhost"];
    let domain = domain.trim_end_matches('.');
    RESERVED_DOMAINS.contains(&domain)
        || RESERVED_SUFFIXES
            .iter()
            .any(|suffix| domain.ends_with(suffix))
}

/// UA 里有没有一个**可联系**的邮箱。返回 `Err` 时附一句说明，供调用方写日志。
///
/// 这道门只做最低限度的判断（有 @、域名带点、域名不是保留占位符），不试图验证
/// 邮箱真实可达——那办不到。它挡的是「默认配置就把假联系方式发给 SEC」这一种，
/// 而那恰好是不改配置时的默认路径。
pub fn sec_user_agent_contact_error(user_agent: &str) -> Option<&'static str> {
    let user_agent = user_agent.trim();
    if user_agent.is_empty() {
        return Some("User-Agent 为空");
    }
    let Some(candidate) = user_agent
        .split_whitespace()
        .find(|token| token.contains('@'))
    else {
        return Some("User-Agent 里没有邮箱，SEC 要求带联系方式");
    };
    let candidate = candidate.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '@');
    let mut parts = candidate.splitn(2, '@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default().to_ascii_lowercase();
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Some("User-Agent 里的邮箱形态不完整");
    }
    if is_placeholder_contact_domain(&domain) {
        return Some(
            "User-Agent 里的邮箱指向一个保留/占位域名（.local、example.com 之类），SEC 无法据此联系到人",
        );
    }
    None
}

/// SEC 官方股本取数客户端。
///
/// 四条硬约束写在这里，不在调用方：
/// 1. **必须**带一个能真的联系到人的 User-Agent。空 UA、没有邮箱、或者邮箱域名是
///    `.local` / `example.com` 这类保留占位符，都直接关闭通道——发一个 SEC 联系不到人的
///    假 UA 不比不发好，它换来的是限流甚至封禁。
/// 2. 404 是正常降级路径，通过 [`SecSharesOutcome::Absent`] 返回，不是 `Err`。
/// 3. 单次请求超时比 FMP 短，整条通道另有 [`SEC_SNAPSHOT_BUDGET_SECS`] 的总预算：
///    官方股本是锦上添花的交叉校验，不允许它决定 snapshot 的延迟。
/// 4. 传输类失败进 [`SEC_TTL_FAILURE`] 负缓存：SEC 挂掉时每 5 分钟最多付一次，
///    而不是每一次 snapshot、每一家公司各付一次。
#[derive(Clone)]
pub struct SecSharesClient {
    http: reqwest::Client,
    tickers_url: String,
    concept_base: String,
}

impl SecSharesClient {
    /// 没有**可联系**的 User-Agent → 返回 `None`，这条通道就是未配置，调用方静默降级。
    /// 复用 `event_engine.sec_filings.enrichment.user_agent`：同一个 SEC，同一个联系方式，
    /// 不为这件事再加一份配置。
    ///
    /// 保留域名（`.local` / `.test` / `example.com` 等）一律不合格：它们永远解析不到，
    /// SEC 拿它找不到人。这类 UA 一出现就关闭通道——关着只是少一份交叉校验
    /// （读取方回落到 provider 并标注「未经官方核验」），开着而带假联系方式，
    /// 赌的是整个 IP 被 SEC 封掉。仓库的默认值因此用真实运营邮箱，
    /// 有 `the_shipped_default_user_agent_opens_the_channel` 钉住——否则这条通道
    /// 在不改配置的部署下永远是关的，官方股本这条校验等于不存在。
    pub fn new(user_agent: &str) -> Option<Self> {
        let user_agent = user_agent.trim();
        if let Some(reason) = sec_user_agent_contact_error(user_agent) {
            if !user_agent.is_empty() {
                tracing::warn!(
                    "SEC 官方股本通道已关闭：{reason}（event_engine.sec_filings.enrichment.user_agent = {user_agent:?}）。\
                     填一个真实可达的联系邮箱即可开启。"
                );
            }
            return None;
        }
        let http = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(StdDuration::from_secs(SEC_REQUEST_TIMEOUT_SECS))
            .build()
            .ok()?;
        Some(Self {
            http,
            tickers_url: SEC_TICKERS_URL.to_string(),
            concept_base: SEC_COMPANYCONCEPT_BASE.to_string(),
        })
    }

    /// 测试用：把两个端点指到本地 stub。
    #[doc(hidden)]
    pub fn with_endpoints(mut self, tickers_url: &str, concept_base: &str) -> Self {
        self.tickers_url = tickers_url.to_string();
        self.concept_base = concept_base.to_string();
        self
    }

    /// ticker → CIK。带 7 天进程级缓存，**失败也缓存** [`SEC_TTL_FAILURE`]。
    ///
    /// 失败进负缓存这一条不是优化，是止损：这张表 776 KB，一轮 sweep 里 59 家各自
    /// 无条件重试一次，最坏 59 × 8s 会把 15 分钟预算吃掉一半，同时把 429 打成一串。
    pub async fn ticker_cik_map(
        &self,
    ) -> Result<std::sync::Arc<HashMap<String, u64>>, SecSharesError> {
        if let Ok(guard) = cik_map_cache().lock()
            && let Some((at, map)) = guard.as_ref()
            && at.elapsed() < SEC_TTL_CIK_MAP
        {
            return Ok(map.clone());
        }
        if let Ok(guard) = cik_map_failure_cache().lock()
            && let Some(at) = guard.as_ref()
            && at.elapsed() < SEC_TTL_FAILURE
        {
            return Err(SecSharesError(
                "SEC ticker 对照表最近取数失败，退避中".to_string(),
            ));
        }
        let outcome = async {
            let value = self
                .get_json(&self.tickers_url)
                .await?
                .ok_or_else(|| SecSharesError("SEC ticker 对照表返回 404".to_string()))?;
            let map = std::sync::Arc::new(parse_ticker_cik_map(&value));
            if map.is_empty() {
                return Err(SecSharesError("SEC ticker 对照表解析后为空".to_string()));
            }
            Ok(map)
        }
        .await;
        match outcome {
            Ok(map) => {
                if let Ok(mut guard) = cik_map_cache().lock() {
                    *guard = Some((std::time::Instant::now(), map.clone()));
                }
                if let Ok(mut guard) = cik_map_failure_cache().lock() {
                    *guard = None;
                }
                Ok(map)
            }
            Err(error) => {
                if let Ok(mut guard) = cik_map_failure_cache().lock() {
                    *guard = Some(std::time::Instant::now());
                }
                Err(error)
            }
        }
    }

    /// 某个代码的官方封面股数。`Ok(Absent(..))` 是正常降级，不是错误。
    ///
    /// 整条通道（对照表 + concept）罩在 [`SEC_SNAPSHOT_BUDGET_SECS`] 的总预算里：
    /// 调用方之一是对话侧的 snapshot，它在关键路径上串行 await 这个方法，
    /// 两次 8 秒超时串起来就是用户干等 16 秒。
    pub async fn cover_shares(&self, ticker: &str) -> Result<SecSharesOutcome, SecSharesError> {
        self.within_budget(self.cover_shares_inner(ticker)).await
    }

    async fn cover_shares_inner(&self, ticker: &str) -> Result<SecSharesOutcome, SecSharesError> {
        let map = self.ticker_cik_map().await?;
        let Some(&cik) = map.get(&normalize_sec_ticker(ticker)) else {
            return Ok(SecSharesOutcome::Absent(SecSharesAbsence::UnknownTicker));
        };
        self.cover_shares_for_cik_inner(cik).await
    }

    /// 已知 CIK 时的取数——**不过对照表**。上一轮成功取到封面的那些公司会把 CIK
    /// 存进 `company_facts`，于是对照表挂掉的那天它们照样拿得到官方股本。
    ///
    /// 封面股数一个季度才动一次，命中缓存的 TTL 是 6 小时；404 走 24 小时**负缓存**
    /// （多类别股发行人是长期 404，不缓存等于每轮白打一次）；传输失败走
    /// [`SEC_TTL_FAILURE`] 的短负缓存。
    pub async fn cover_shares_for_cik(&self, cik: u64) -> Result<SecSharesOutcome, SecSharesError> {
        self.within_budget(self.cover_shares_for_cik_inner(cik))
            .await
    }

    async fn cover_shares_for_cik_inner(
        &self,
        cik: u64,
    ) -> Result<SecSharesOutcome, SecSharesError> {
        if let Some(cached) = self.cached_concept(cik) {
            return Ok(self.outcome_from_concept(cik, cached.as_deref()));
        }
        if let Ok(guard) = concept_failure_cache().lock()
            && let Some(at) = guard.get(&cik)
            && at.elapsed() < SEC_TTL_FAILURE
        {
            return Err(SecSharesError(format!(
                "SEC companyconcept CIK {cik} 最近取数失败，退避中"
            )));
        }
        let url = concept_url(&self.concept_base, cik);
        let value = match self.get_json(&url).await {
            Ok(value) => value.map(std::sync::Arc::new),
            Err(error) => {
                if let Ok(mut guard) = concept_failure_cache().lock() {
                    guard.insert(cik, std::time::Instant::now());
                }
                return Err(error);
            }
        };
        if let Ok(mut guard) = concept_failure_cache().lock() {
            guard.remove(&cik);
        }
        self.store_concept(cik, value.clone());
        Ok(self.outcome_from_concept(cik, value.as_deref()))
    }

    /// 总预算。超时按传输失败处理，让上面那两层的负缓存把它记下来。
    async fn within_budget<F>(&self, future: F) -> Result<SecSharesOutcome, SecSharesError>
    where
        F: std::future::Future<Output = Result<SecSharesOutcome, SecSharesError>>,
    {
        match tokio::time::timeout(StdDuration::from_secs(SEC_SNAPSHOT_BUDGET_SECS), future).await {
            Ok(result) => result,
            Err(_) => Err(SecSharesError(format!(
                "SEC 通道超过 {SEC_SNAPSHOT_BUDGET_SECS}s 总预算"
            ))),
        }
    }

    fn outcome_from_concept(&self, cik: u64, concept: Option<&Value>) -> SecSharesOutcome {
        let Some(concept) = concept else {
            return SecSharesOutcome::Absent(SecSharesAbsence::ConceptNotFound);
        };
        let rows = parse_cover_rows(concept);
        let Some((latest, previous)) = latest_and_previous(&rows) else {
            return SecSharesOutcome::Absent(SecSharesAbsence::NoCoverRows);
        };
        SecSharesOutcome::Cover(Box::new(SecCoverShares {
            cik,
            entity_name: concept
                .get("entityName")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string),
            latest,
            previous,
        }))
    }

    fn cached_concept(&self, cik: u64) -> Option<Option<std::sync::Arc<Value>>> {
        let guard = concept_cache().lock().ok()?;
        let (at, value) = guard.get(&cik)?;
        let ttl = if value.is_some() {
            SEC_TTL_CONCEPT
        } else {
            SEC_TTL_CONCEPT_ABSENT
        };
        (at.elapsed() < ttl).then(|| value.clone())
    }

    fn store_concept(&self, cik: u64, value: Option<std::sync::Arc<Value>>) {
        if let Ok(mut guard) = concept_cache().lock() {
            guard.insert(cik, (std::time::Instant::now(), value));
        }
    }

    /// `Ok(None)` = 404。其它非 2xx、传输失败、JSON 解析失败都是 `Err`。
    async fn get_json(&self, url: &str) -> Result<Option<Value>, SecSharesError> {
        let response = self
            .http
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| SecSharesError(format!("SEC 请求失败: {error}")))?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            return Err(SecSharesError(format!("SEC 返回 HTTP {}", status.as_u16())));
        }
        let body = response
            .text()
            .await
            .map_err(|error| SecSharesError(format!("SEC 响应读取失败: {error}")))?;
        serde_json::from_str::<Value>(&body)
            .map(Some)
            .map_err(|error| SecSharesError(format!("SEC 响应不是 JSON: {error}")))
    }
}

/// 挂在 `snapshot` / `financials` 返回里的 `hone_shares_outstanding` 块。
///
/// 与 [`build_shares_outstanding_payload`] 的区别：那个是 `shares_outstanding`
/// data_type 的**整个**返回体；这个是塞进别人返回里的一个字段，因此不带
/// `data_type` / `ticker`，并且把与 provider 的差异合进同一个块里——读的人不必
/// 在两处之间对照。
pub fn build_shares_outstanding_block(
    cover: &SecCoverShares,
    provider_shares: Option<f64>,
    provider_market_cap: Option<f64>,
    price: Option<f64>,
    today: NaiveDate,
) -> Value {
    let domestic = form_is_us_domestic_periodic(&cover.latest.form);
    let usable = cover.usable_for_market_cap(today);
    let mut block = serde_json::json!({
        "official_shares_outstanding": cover.latest.shares,
        "cover_date": cover.latest.end,
        "filed": cover.latest.filed,
        "form": cover.latest.form,
        "accession_number": cover.latest.accn,
        "cik": cik_path_segment(cover.cik),
        "basis": cover.basis(),
        "usable_for_market_cap": usable,
        "source": "SEC XBRL dei:EntityCommonStockSharesOutstanding",
        "semantics": "监管申报封面上「截至封面日期已发行普通股」的官方股数，不是加权平均股本、也不是完全摊薄股本：算市值和 EV 用它，算 EPS 用报表里的加权稀释股本。",
    });
    if let Some(age) = cover.age_days(today) {
        block["cover_age_days"] = Value::from(age);
    }
    if let Some(entity) = cover.entity_name.as_deref() {
        block["entity_name"] = Value::String(entity.to_string());
    }
    if let Some(previous) = cover.previous.as_ref() {
        block["previous"] = serde_json::json!({
            "official_shares_outstanding": previous.shares,
            "cover_date": previous.end,
            "form": previous.form,
        });
    }

    if !domestic {
        // 20-F 只标注口径，**绝不**参与市值校验：TSM 官方封面数是台股普通股，
        // 1 ADR = 5 股，拿它重算 ADR 市值会错 5 倍。
        block["basis_warning"] = Value::String(format!(
            "最新一条来自 {}，报的是发行人本土市场的普通股股数，不是美股 ADR/ADS 股数（比例常见 1:1 以外，如 1 ADR 兑 5 股普通股）。不得用它乘以美股股价推算市值、也不得用它去校正提供方的股本。",
            cover.latest.form
        ));
        return block;
    }
    if !usable {
        block["basis_warning"] = Value::String(
            "最新一条的封面日期已经太旧，不能当作当期股本使用：HTTP 成功不代表数据新鲜。请按「本轮未核验」处理。"
                .to_string(),
        );
        return block;
    }

    match provider_shares.and_then(|shares| {
        build_shares_cross_check(shares, provider_market_cap, price, &cover.latest, today)
    }) {
        Some(cross_check) => block["provider_cross_check"] = cross_check,
        None => {
            block["provider_cross_check"] = serde_json::json!({
                "status": "consistent_or_unavailable",
                "note": "提供方股本与官方封面股数没有超过阈值的差异，或本轮拿不到提供方股本。按提供方数字使用即可。",
            });
        }
    }
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 30).expect("fixed test date")
    }

    fn row(shares: i64, end: &str, filed: &str, form: &str, accn: &str) -> CoverShareRow {
        CoverShareRow {
            shares,
            end: end.to_string(),
            filed: filed.to_string(),
            form: form.to_string(),
            accn: accn.to_string(),
        }
    }

    #[test]
    fn cik_is_zero_padded_to_ten_digits_with_an_uppercase_prefix() {
        // 实测：CIK1633978 / cik0001633978 / CIK00001633978 全部 404。
        assert_eq!(cik_path_segment(1_633_978), "CIK0001633978");
        assert_eq!(cik_path_segment(320_193), "CIK0000320193");
        assert_eq!(cik_path_segment(1_750), "CIK0000001750");
        assert_eq!(cik_path_segment(2_149_111), "CIK0002149111");
        assert_eq!(
            concept_url(SEC_COMPANYCONCEPT_BASE, 1_633_978),
            "https://data.sec.gov/api/xbrl/companyconcept/CIK0001633978/dei/EntityCommonStockSharesOutstanding.json"
        );
        // 末尾多一个斜杠不应该产生 `//`。
        assert_eq!(
            concept_url("https://data.sec.gov/api/xbrl/companyconcept/", 1_633_978),
            concept_url(SEC_COMPANYCONCEPT_BASE, 1_633_978)
        );
    }

    #[test]
    fn ticker_lookup_normalizes_case_and_share_class_separators() {
        let map = parse_ticker_cik_map(&json!({
            "0": {"cik_str": 1045810, "ticker": "NVDA", "title": "NVIDIA CORP"},
            "1": {"cik_str": 1633978, "ticker": "LITE", "title": "Lumentum Holdings Inc."},
            "2": {"cik_str": 1067983, "ticker": "BRK-B", "title": "BERKSHIRE HATHAWAY INC"},
            "3": {"cik_str": 1652044, "ticker": "GOOGL", "title": "Alphabet Inc."},
        }));
        assert_eq!(map.get(&normalize_sec_ticker("lite")), Some(&1_633_978));
        assert_eq!(map.get(&normalize_sec_ticker(" NVDA ")), Some(&1_045_810));
        // provider 写 BRK.B / BRK/B，SEC 写 BRK-B。
        assert_eq!(map.get(&normalize_sec_ticker("BRK.B")), Some(&1_067_983));
        assert_eq!(map.get(&normalize_sec_ticker("brk/b")), Some(&1_067_983));
        assert_eq!(map.get(&normalize_sec_ticker("NOSUCH")), None);
        // cik_str 万一变成字符串也要收得住。
        let as_string =
            parse_ticker_cik_map(&json!({"0": {"cik_str": "320193", "ticker": "AAPL"}}));
        assert_eq!(as_string.get("AAPL"), Some(&320_193));
    }

    #[test]
    fn empty_units_object_on_a_200_is_no_data_not_a_parse_failure() {
        // 实测 AMKR / BE / GLW 在 HTTP 200 下返回 `"units": {"shares": {}}`。
        assert!(parse_cover_rows(&json!({"units": {"shares": {}}})).is_empty());
        assert!(parse_cover_rows(&json!({"units": {}})).is_empty());
        assert!(parse_cover_rows(&json!({"cik": 24741})).is_empty());
    }

    #[test]
    fn latest_row_is_max_end_then_filed_then_accession() {
        // STX 在 2012-10-31 重述了 end=2012-04-25，那个 filed 比更新的 end 还晚；
        // 只按 filed 排会选中过期的封面日。
        let rows = vec![
            row(425_234_957, "2012-04-25", "2012-04-27", "10-Q", "a"),
            row(377_494_162, "2012-04-25", "2012-10-31", "10-Q/A", "b"),
            row(400_000_000, "2012-07-25", "2012-08-01", "10-Q", "c"),
        ];
        let (latest, previous) = latest_and_previous(&rows).expect("rows");
        assert_eq!(latest.end, "2012-07-25");
        assert_eq!(latest.shares, 400_000_000);
        // 上一条取 end 严格更早的最新一条，即那份重述后的值。
        let previous = previous.expect("previous row");
        assert_eq!(previous.end, "2012-04-25");
        assert_eq!(previous.shares, 377_494_162);
    }

    #[test]
    fn same_cover_date_prefers_the_later_filing_then_the_accession_number() {
        // SNDK：10-Q/A 把 114,863,251 更正成 144,863,251。
        let rows = vec![
            row(
                114_863_251,
                "2025-02-27",
                "2025-03-07",
                "10-Q",
                "0001-25-000001",
            ),
            row(
                144_863_251,
                "2025-02-27",
                "2025-03-17",
                "10-Q/A",
                "0001-25-000002",
            ),
        ];
        let (latest, previous) = latest_and_previous(&rows).expect("rows");
        assert_eq!(latest.shares, 144_863_251);
        assert!(previous.is_none(), "同一封面日期不算「上一条」");

        // UCTT：end 与 filed 都相同，只有 accn 不同——排序仍必须确定。
        let rows = vec![
            row(
                45_129_294,
                "2025-01-24",
                "2025-02-11",
                "10-Q",
                "0000-25-004714",
            ),
            row(
                45_129_294,
                "2025-01-24",
                "2025-02-11",
                "10-Q",
                "0000-25-004711",
            ),
        ];
        let (latest, _) = latest_and_previous(&rows).expect("rows");
        assert_eq!(latest.accn, "0000-25-004714");
    }

    #[test]
    fn the_lite_incident_numbers_come_out_of_the_real_response_shape() {
        let concept = json!({
            "cik": 1633978,
            "entityName": "Lumentum Holdings Inc.",
            "units": {"shares": [
                {"end": "2026-01-27", "val": 71400000, "accn": "0001628280-26-005129", "fy": 2026, "fp": "Q2", "form": "10-Q", "filed": "2026-02-04", "frame": "CY2025Q4I"},
                {"end": "2026-04-30", "val": 77800000, "accn": "0001628280-26-030777", "fy": 2026, "fp": "Q3", "form": "10-Q", "filed": "2026-05-06"},
                {"end": "2026-08-14", "val": 89700000, "accn": "0001628280-26-057358", "fy": 2026, "fp": "FY", "form": "10-K", "filed": "2026-08-17", "frame": "CY2026Q2I"}
            ]}
        });
        let rows = parse_cover_rows(&concept);
        assert_eq!(rows.len(), 3, "`frame` 缺失的那行不得被丢掉");
        let (latest, previous) = latest_and_previous(&rows).expect("rows");
        assert_eq!(latest.shares, 89_700_000);
        assert_eq!(latest.form, "10-K");
        assert_eq!(previous.as_ref().expect("previous").shares, 77_800_000);

        let payload = build_shares_outstanding_payload(
            "LITE",
            1_633_978,
            concept.get("entityName").and_then(Value::as_str),
            &latest,
            previous.as_ref(),
            today(),
        );
        assert_eq!(payload["data"]["cik"], "CIK0001633978");
        assert_eq!(payload["data"]["latest"]["shares_outstanding"], 89_700_000);
        assert_eq!(payload["data"]["latest"]["cover_age_days"], 16);
        assert_eq!(
            payload["data"]["previous"]["shares_outstanding"],
            77_800_000
        );
        assert_eq!(payload["data"]["usable_for_market_cap"], true);
        assert!(payload.get("hone_shares_basis_warning").is_none());
    }

    #[test]
    fn a_foreign_private_issuer_cover_count_is_labelled_and_never_cross_checked() {
        // TSM 的官方封面数是台股普通股口径；1 ADR = 5 股，直接用它重算 ADR
        // 市值会错 5 倍，比本次事故严重得多。
        let latest = row(25_932_524_521, "2025-12-31", "2026-04-15", "20-F", "z");
        let payload = build_shares_outstanding_payload(
            "TSM",
            1_046_179,
            Some("TSMC"),
            &latest,
            None,
            today(),
        );
        assert_eq!(payload["data"]["usable_for_market_cap"], false);
        assert_eq!(payload["data"]["basis"], "foreign_or_non_periodic_filing");
        let warning = payload["hone_shares_basis_warning"]
            .as_str()
            .expect("basis warning");
        assert!(warning.contains("ADR"));
        assert!(warning.contains("本土市场"));

        // 交叉校验对 20-F 必须完全沉默，哪怕差 5 倍。
        assert!(
            build_shares_cross_check(5_186_504_904.0, None, Some(300.0), &latest, today())
                .is_none()
        );
        assert!(!form_is_us_domestic_periodic("20-F"));
        assert!(!form_is_us_domestic_periodic("20-F/A"));
        assert!(!form_is_us_domestic_periodic("40-F"));
        assert!(!form_is_us_domestic_periodic("8-K"));
        assert!(form_is_us_domestic_periodic("10-K"));
        assert!(form_is_us_domestic_periodic("10-Q"));
        assert!(form_is_us_domestic_periodic("10-Q/A"));
    }

    #[test]
    fn a_stale_domestic_cover_count_is_not_allowed_to_override_the_provider() {
        // BRK 的最新一条停在 2011-04-29：HTTP 200 不等于新鲜。
        let latest = row(941_481, "2011-04-29", "2011-05-06", "10-Q", "z");
        assert!(
            build_shares_cross_check(1_450_000_000.0, None, Some(500.0), &latest, today())
                .is_none()
        );
        let payload =
            build_shares_outstanding_payload("BRK-A", 1_067_983, None, &latest, None, today());
        assert_eq!(payload["data"]["usable_for_market_cap"], false);
        assert!(
            payload["hone_shares_basis_warning"]
                .as_str()
                .expect("stale warning")
                .contains("封面日期已经太旧")
        );
    }

    #[test]
    fn the_lite_gap_raises_a_block_and_a_normal_buyback_drift_does_not() {
        let latest = row(89_700_000, "2026-08-14", "2026-08-17", "10-K", "z");
        let block = build_shares_cross_check(
            77_800_000.0,
            Some(69_631_000_000.0),
            Some(895.0),
            &latest,
            today(),
        )
        .expect("15% gap must raise a block");
        assert_eq!(block["status"], "provider_behind_official_filing");
        assert_eq!(block["official_shares_outstanding"], 89_700_000);
        assert_eq!(block["provider_shares_outstanding"], 77_800_000.0);
        assert_eq!(block["difference_pct"], 15.3);
        assert_eq!(block["official_cover_date"], "2026-08-14");
        assert_eq!(block["official_form"], "10-K");
        assert_eq!(block["recomputed_market_cap"], 80_281_500_000.0);

        // 回购/增发造成的 1% 级漂移不该每天报警。
        let latest = row(78_600_000, "2026-08-14", "2026-08-17", "10-Q", "z");
        assert!(build_shares_cross_check(77_800_000.0, None, None, &latest, today()).is_none());
        // 阈值两侧：2.9% 沉默、3.1% 挂块。
        let quiet = row(80_056_200, "2026-08-14", "2026-08-17", "10-Q", "z");
        assert!(build_shares_cross_check(77_800_000.0, None, None, &quiet, today()).is_none());
        let loud = row(80_211_800, "2026-08-14", "2026-08-17", "10-Q", "z");
        assert!(build_shares_cross_check(77_800_000.0, None, None, &loud, today()).is_some());
    }

    #[test]
    fn an_order_of_magnitude_gap_is_reported_as_a_basis_mismatch_not_a_stale_provider() {
        // 即便 form 是国内定期申报，整数倍级差距也不能断言 provider 过期并让
        // 模型拿官方数重算市值。
        let latest = row(500_000_000, "2026-06-30", "2026-07-23", "10-Q", "z");
        let block = build_shares_cross_check(100_000_000.0, None, Some(50.0), &latest, today())
            .expect("5x gap must be visible");
        assert_eq!(block["status"], "basis_mismatch_suspected");
        assert!(block.get("recomputed_market_cap").is_none());
        assert!(
            block["note"]
                .as_str()
                .expect("note")
                .contains("不要直接拿其中任何一个乘股价重算市值")
        );
    }

    #[test]
    fn cross_check_needs_a_usable_provider_number() {
        let latest = row(89_700_000, "2026-08-14", "2026-08-17", "10-K", "z");
        assert!(build_shares_cross_check(0.0, None, None, &latest, today()).is_none());
        assert!(build_shares_cross_check(-1.0, None, None, &latest, today()).is_none());
        assert!(build_shares_cross_check(f64::NAN, None, None, &latest, today()).is_none());
    }

    #[test]
    fn provider_shares_are_read_from_the_first_row_of_a_quote_array() {
        let quote = json!([{"symbol": "LITE", "price": 895.0, "sharesOutstanding": 77800000_i64, "marketCap": 69631000000_i64}]);
        assert_eq!(provider_shares_outstanding(&quote), Some(77_800_000.0));
        assert_eq!(provider_quote_number(&quote, "price"), Some(895.0));
        assert_eq!(
            provider_quote_number(&quote, "marketCap"),
            Some(69_631_000_000.0)
        );
        assert_eq!(provider_shares_outstanding(&json!([])), None);
        assert_eq!(
            provider_shares_outstanding(&json!([{"symbol": "LITE"}])),
            None
        );
        assert_eq!(provider_shares_outstanding(&Value::Null), None);
    }

    #[test]
    fn an_absent_concept_is_a_named_degrade_path_not_an_error() {
        // 8/59 家长期走到这里；每一种都要能对用户说清为什么没有官方股本。
        assert_eq!(
            SecSharesAbsence::ConceptNotFound.as_str(),
            "concept_not_found"
        );
        assert!(SecSharesAbsence::ConceptNotFound.note().contains("axis"));
        assert!(SecSharesAbsence::NoCoverRows.note().contains("axis"));
        assert!(SecSharesAbsence::UnknownTicker.note().contains("对照表"));
    }

    #[test]
    fn a_user_agent_without_a_reachable_contact_closes_the_channel() {
        // SEC 要求带联系方式的 User-Agent，目的是出问题时能找到人。
        // 「非空」这道门形同虚设：仓库的默认值 ops@honeclaw.local 就能过。
        assert!(SecSharesClient::new("").is_none());
        assert!(SecSharesClient::new("   ").is_none());
        // 默认配置值——一个永远解析不到的域名。必须关掉通道，不许发出去。
        assert!(
            SecSharesClient::new("honeclaw event-engine ops@honeclaw.local").is_none(),
            "默认占位 UA 不得开启通道"
        );
        assert!(SecSharesClient::new("honeclaw company-facts ops@example.com").is_none());
        assert!(SecSharesClient::new("honeclaw company-facts ops@example.test").is_none());
        assert!(SecSharesClient::new("honeclaw company-facts ops@box.invalid").is_none());
        // 没有邮箱、邮箱不完整，一样不发。
        assert!(SecSharesClient::new("honeclaw company-facts").is_none());
        assert!(SecSharesClient::new("honeclaw ops@localhost").is_none());
        assert!(SecSharesClient::new("honeclaw @honeclaw.ai").is_none());

        // 一个真的能收到信的地址才开通道。
        assert!(SecSharesClient::new("honeclaw company-facts ops@honeclaw.ai").is_some());
        assert!(SecSharesClient::new("Honeclaw/1.0 (ops@honeclaw.ai)").is_some());
    }

    /// 本轮的 blocker：联系方式闸门加上之后，仓库默认值仍是 `.local` 占位符，
    /// 于是默认部署下整条 SEC 通道关着、官方股本校验等于不存在。改默认值容易，
    /// 忘了改也一样容易——所以把它钉在这里。
    #[test]
    fn the_shipped_default_user_agent_opens_the_channel() {
        let shipped = crate::config::EventEngineConfig::default()
            .sec_filings
            .enrichment
            .user_agent;
        assert!(
            sec_user_agent_contact_error(&shipped).is_none(),
            "仓库默认 UA 被联系方式闸门拒绝，SEC 通道默认关闭：{shipped}"
        );
        assert!(SecSharesClient::new(&shipped).is_some());
    }

    /// 失败必须进负缓存。不进的话，SEC 一挂（429 / 5xx / 挡板页），一轮 sweep 里
    /// 59 家会各自重新付满一次超时预算，对话侧则是每一次 snapshot 都无退避地重试
    /// 一条正在失败的通道——上游越忙，我们打得越凶。
    #[tokio::test]
    async fn a_failing_channel_backs_off_instead_of_retrying_on_every_call() {
        reset_sec_caches_for_tests();
        let client = SecSharesClient::new("honeclaw company-facts ops@honeclaw.ai")
            .expect("client")
            // 必然连不上的端点：第一次真的会外呼并失败。
            .with_endpoints("http://127.0.0.1:1/tickers.json", "http://127.0.0.1:1/c");

        assert!(client.cover_shares("LITE").await.is_err(), "第一次该失败");

        let started = std::time::Instant::now();
        let error = client
            .cover_shares("LITE")
            .await
            .expect_err("退避期内仍然是失败");
        assert!(
            error.to_string().contains("退避"),
            "第二次应当直接返回退避，而不是再打一次: {error}"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_millis(200),
            "退避路径不该再有网络往返"
        );

        // 缓存是进程级的，别留给同一进程里的其它用例。
        reset_sec_caches_for_tests();
    }

    #[test]
    fn the_contact_check_explains_why_it_refused() {
        // 关掉通道不解释，运维只会看到「SEC 没数据」，而不是「你的 UA 不合格」。
        assert!(
            sec_user_agent_contact_error("honeclaw event-engine ops@honeclaw.local")
                .expect("refused")
                .contains("占位域名")
        );
        assert!(
            sec_user_agent_contact_error("honeclaw event-engine")
                .expect("refused")
                .contains("邮箱")
        );
        assert_eq!(
            sec_user_agent_contact_error("honeclaw ops@honeclaw.ai"),
            None
        );
    }

    fn cover(shares: i64, end: &str, filed: &str, form: &str) -> SecCoverShares {
        SecCoverShares {
            cik: 1_633_978,
            entity_name: Some("Lumentum Holdings Inc.".to_string()),
            latest: row(shares, end, filed, form, "z"),
            previous: Some(row(77_800_000, "2026-04-30", "2026-05-06", "10-Q", "y")),
        }
    }

    #[test]
    fn the_snapshot_block_carries_freshness_and_the_provider_gap() {
        let cover = cover(89_700_000, "2026-08-14", "2026-08-17", "10-K");
        let block = build_shares_outstanding_block(
            &cover,
            Some(77_800_000.0),
            Some(69_631_000_000.0),
            Some(895.0),
            today(),
        );
        assert_eq!(block["official_shares_outstanding"], 89_700_000);
        assert_eq!(block["cover_date"], "2026-08-14");
        assert_eq!(block["filed"], "2026-08-17");
        assert_eq!(block["form"], "10-K");
        assert_eq!(block["cover_age_days"], 16);
        assert_eq!(block["usable_for_market_cap"], true);
        assert_eq!(block["previous"]["official_shares_outstanding"], 77_800_000);
        assert_eq!(
            block["provider_cross_check"]["status"],
            "provider_behind_official_filing"
        );
        assert_eq!(block["provider_cross_check"]["difference_pct"], 15.3);
        assert!(block.get("basis_warning").is_none());
    }

    #[test]
    fn a_20f_block_labels_the_basis_and_never_grows_a_cross_check() {
        // TSM：官方 259.3 亿是台股普通股口径，1 ADR = 5 股。哪怕差 5 倍也不许校验。
        let cover = SecCoverShares {
            cik: 1_046_179,
            entity_name: Some("TSMC".to_string()),
            latest: row(25_932_524_521, "2025-12-31", "2026-04-15", "20-F", "z"),
            previous: None,
        };
        let block = build_shares_outstanding_block(
            &cover,
            Some(5_186_504_904.0),
            None,
            Some(300.0),
            today(),
        );
        assert_eq!(block["basis"], "foreign_or_non_periodic_filing");
        assert_eq!(block["usable_for_market_cap"], false);
        assert!(
            block.get("provider_cross_check").is_none(),
            "20-F 不得参与校验"
        );
        let warning = block["basis_warning"].as_str().expect("basis warning");
        assert!(warning.contains("ADR"));
    }

    #[test]
    fn a_matching_provider_number_still_says_so_out_loud() {
        // 沉默会被读成「没查」。一致也要写出来，否则读的人无法区分。
        let cover = cover(78_000_000, "2026-08-14", "2026-08-17", "10-Q");
        let block =
            build_shares_outstanding_block(&cover, Some(77_800_000.0), None, Some(895.0), today());
        assert_eq!(
            block["provider_cross_check"]["status"],
            "consistent_or_unavailable"
        );
        // 没有 provider 股本时也走同一分支，不得凭空报警。
        let block = build_shares_outstanding_block(&cover, None, None, None, today());
        assert_eq!(
            block["provider_cross_check"]["status"],
            "consistent_or_unavailable"
        );
    }
}
