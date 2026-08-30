//! 行业树 59 家的公司基础数据 —— 唯一一份**共享**的公司事实库。
//!
//! 为什么要有它：2026-08-30 的 LITE 事故里，回答用了 7,780 万股本，而两周前报出的
//! 10-K 封面数已经是 8,970 万。根因不是某个字段读错，而是**全链路没有任何一处保存过
//! 「这家公司的官方股本是多少、什么时候更新的」**——每轮现取 provider，没有历史可比、
//! 没有新鲜度判断、没有第二个源。这个模块就是那个「一处」。
//!
//! 三条设计约束：
//!
//! 1. **只存不随盘中变化的东西。** 没有 `price`、没有 `market_cap`、没有涨跌幅。
//!    市值一律「现价 × 表里的股本」现算——股本一旦修对，所有倍数一起对；而把市值存进来
//!    只会得到一个当天就过期、还会被当成事实引用的数字。
//! 2. **每个数字都带自己的出处和时间**（[`FactProvenance`]）。「最近一季营收」不写清
//!    是哪一季、什么时候取的，就没法判断它是不是又落后了一份申报。
//! 3. **双路存储**，照 `portfolio.rs` 的形状：配了 PG 走 PG（两个进程看到同一份），
//!    没配就退回 data_dir 下的文件。本地开发不需要数据库也能跑。
//!
//! 官方股本的口径规则不在这里，在 `hone_core::sec_shares`：20-F 报的是本土普通股，
//! 只标注、不参与市值校验。这里只负责把结论存下来。

use hone_core::cloud_runtime::{CloudCompanyFactsRecord, CloudPgRuntime};
use hone_core::{HoneError, HoneResult};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// 存储格式版本。加字段不需要动它（全部 `#[serde(default)]`）；改字段含义才动。
pub const COMPANY_FACTS_SCHEMA_VERSION: u32 = 1;

/// 封面股数超过这个天数就算「陈旧」。与 `sec_shares::SEC_MAX_COVER_AGE_DAYS` 同一个数，
/// 理由也一样：美国国内定期申报人每季都报 10-Q，最新一条正常在 130 天以内。
pub const COVER_STALE_AFTER_DAYS: i64 = 200;

/// 整份数据超过这个小时数没刷新就算陈旧，读取方应当据此降级措辞。
/// 取 36 小时而不是 24：日更 worker 漏跑一次不该立刻让全站的股本变成「不可用」，
/// 但连着两天没跑就必须说出来。与 `company_ratings` 的 `STALE_AFTER_HOURS` 同值。
pub const FACTS_STALE_AFTER_HOURS: i64 = 36;

/// 一个数字的出处与时间。
///
/// `as_of` 是这个数字**本身**对应的口径日期（财季末、封面日），
/// `fetched_at` 是我们取到它的时刻。两者必须分开：provider 今天返回的数字，
/// 口径可能停在三个月前——LITE 事故里正是如此。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FactProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetched_at: Option<String>,
}

impl FactProvenance {
    pub fn new(as_of: Option<String>, source: &str, fetched_at: &str) -> Self {
        Self {
            as_of,
            source: Some(source.to_string()),
            fetched_at: Some(fetched_at.to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_of.is_none() && self.source.is_none() && self.fetched_at.is_none()
    }
}

/// 身份。`is_adr` / `adr_ratio` / `home_symbol` 直接堵掉 TSM 那个 5 倍坑：
/// 官方封面数是台股普通股口径，1 ADR = 5 股普通股，不写下来就会有人拿它乘美股股价。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CompanyIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cik: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(default)]
    pub is_adr: bool,
    /// 1 ADR 兑几股本土普通股。只在能从一手来源确认时才填；猜不出来就留空，
    /// 留空比填错安全——读取方看到 `is_adr && adr_ratio.is_none()` 应当拒绝换算。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adr_ratio: Option<f64>,
    /// 本土市场的代码（如 TSM 的 2330.TW）。同样只在确认得了时才填。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub provenance: FactProvenance,
}

/// 股本。这是整张表存在的理由，字段最细。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ShareCounts {
    /// 官方封面股数（`dei:EntityCommonStockSharesOutstanding`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_shares: Option<i64>,
    /// 这条封面数是**从哪个 CIK 取到的**。写下来是为了下一轮不必再过一次
    /// `company_tickers.json`：那张 776 KB 的对照表一旦挂掉，只要这里有值，
    /// 官方股本这一段就还能走 `cover_shares_for_cik` 直取。
    /// 只在 SEC 真的返回过封面行时才有值，不接受 provider profile 里的 cik——
    /// 拿错 CIK 去取股本，错的方式和 TSM 那个 5 倍坑同级。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_cik: Option<u64>,
    /// 封面日期：申报封面上「截至该日已发行股数」的那个日期。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_filed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_form: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_accession: Option<String>,
    /// `us_domestic_periodic_cover_page` 或 `foreign_or_non_periodic_filing`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_basis: Option<String>,
    /// 能不能直接乘股价算美股市值。20-F 一律 false。
    #[serde(default)]
    pub cover_usable_for_market_cap: bool,
    /// 上一个封面日期的同一口径数字，摆出来是让人**看得见变化**。
    /// 不要拿它算变动百分比：跨口径（ADR 与本土普通股、股份类别重述）的变动率是假的。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_cover_shares: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_cover_end: Option<String>,
    /// 没有官方封面股数时，说清是哪一种缺失（8/59 家长期如此，是正常降级不是故障）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_absent_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_absent_note: Option<String>,

    /// 最近一季报表里的加权平均股本。算 EPS 用它，算市值**不要**用它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_shares: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diluted_shares: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weighted_period_end: Option<String>,
    /// **亏损季反稀释伪信号的标记。** GAAP 下亏损季要把全部潜在稀释证券排除，
    /// 于是「稀释 = 基本」甚至低于上一季一大截。LITE Q4 FY2026 净利 −71.6 亿，
    /// 稀释股本从上一季的 96.2M 掉到 74.6M，模型不但用了它，还替它编了一个
    /// 「转股清偿」的公司行为。`true` 表示这一季的稀释股本不能用来讲股本变化。
    ///
    /// 判据是**两条同时成立**：稀释 ≤ 基本，**且**本季净利为负。只看第一条会把
    /// 大量「压根没有潜在稀释证券」的盈利季误报成塌陷——那种公司稀释常年等于基本，
    /// 误报会让下游对着一个正常季讲一段与本季无关的话，等于把标记变成背景噪音。
    #[serde(default)]
    pub diluted_collapsed: bool,

    /// provider 的 `sharesOutstanding`，存下来只为了**看得见差异**，不作为权威。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_shares: Option<f64>,
    /// 官方相对 provider 的偏差百分比（正 = 官方更多 = provider 落后）。
    /// 只在口径成立（美国国内定期申报且未过期）时计算。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_difference_pct: Option<f64>,

    /// 封面股数那一组数字（`cover_*`）的出处与时间。
    ///
    /// 与 [`Self::weighted_provenance`] 分开，是因为这个结构体里装着**两个来源、
    /// 两个口径日期**的数字：封面股数来自 SEC 申报封面，加权平均股本来自 provider
    /// 的季度利润表。共用一个 `provenance` 会让落库那行声称加权股本也来自 SEC——
    /// 而这次事故的一半，正是这两个概念被混着读。
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub cover_provenance: FactProvenance,
    /// 加权平均股本（`basic_shares` / `diluted_shares`）的出处与时间。
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub weighted_provenance: FactProvenance,
}

impl ShareCounts {
    /// 算市值该用哪个股本。20-F、过期封面、以及压根没有官方数时返回 `None`——
    /// 调用方必须回落到 provider 并标注「口径未核验」，而不是硬用一个错口径的数。
    pub fn market_cap_shares(&self) -> Option<i64> {
        self.cover_shares
            .filter(|_| self.cover_usable_for_market_cap)
    }

    /// 封面日期距 `today` 多少天。
    pub fn cover_age_days(&self, today: chrono::NaiveDate) -> Option<i64> {
        let end = self.cover_end.as_deref()?;
        chrono::NaiveDate::parse_from_str(end.trim(), "%Y-%m-%d")
            .ok()
            .map(|date| (today - date).num_days())
    }

    pub fn cover_is_stale(&self, today: chrono::NaiveDate) -> bool {
        self.cover_age_days(today)
            .is_some_and(|age| age > COVER_STALE_AFTER_DAYS)
    }
}

/// 资产负债。口径必须写出来：`total_debt` 各家 provider 含不含经营租赁不一样，
/// 不标口径的净现金拿去比同业就是在比两把不同的尺。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BalanceSheetFacts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cash_and_short_term_investments: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_debt: Option<f64>,
    /// 正 = 净现金，负 = 净债务。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net_cash: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capital_lease_obligations: Option<f64>,
    /// 一句话口径说明，跟着数字一起给出去。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debt_basis: Option<String>,
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub provenance: FactProvenance,
}

/// 利润表：最近四季合计 + 最近一季的同环比。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IncomeFacts {
    /// 参与 TTM 合计的四个财季末，写出来才能验证「最近四季」到底是哪四季。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ttm_period_ends: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_revenue: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_gross_profit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_operating_income: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_net_income: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_gross_margin_percent: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_quarter_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_quarter_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_quarter_revenue: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_quarter_net_income: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revenue_qoq_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revenue_yoy_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub provenance: FactProvenance,
}

/// 现金流：最近四季合计。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CashFlowFacts {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ttm_period_ends: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_operating_cash_flow: Option<f64>,
    /// provider 的 `capitalExpenditure` 是负数（现金流出），这里原样保留符号。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_capital_expenditure: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttm_free_cash_flow: Option<f64>,
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub provenance: FactProvenance,
}

/// 财报节奏。worker 的事件驱动刷新就是盯 `latest_reported_date` 有没有前进。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EarningsCadence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_reported_date: Option<String>,
    /// 最近一次已发布财报对应的**财季末**（不是发布日）。日历端点通常只给发布日，
    /// 所以这一项由 worker 从同一轮的季度利润表里那条最新记录补上。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_reported_period_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_earnings_date: Option<String>,
    #[serde(default, skip_serializing_if = "FactProvenance::is_empty")]
    pub provenance: FactProvenance,
}

/// 一家公司的全部基础事实。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanyFacts {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub symbol: String,
    #[serde(default)]
    pub identity: CompanyIdentity,
    #[serde(default)]
    pub shares: ShareCounts,
    #[serde(default)]
    pub balance_sheet: BalanceSheetFacts,
    #[serde(default)]
    pub income: IncomeFacts,
    #[serde(default)]
    pub cash_flow: CashFlowFacts,
    #[serde(default)]
    pub earnings: EarningsCadence,
    /// 距最新一次申报（封面 `filed`）的天数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_since_latest_filing: Option<i64>,
    /// 本轮**没**取到的组件。每家独立降级，所以这份名单是逐家的：
    /// 一家的 SEC 404 不该让另一家的利润表也被标成缺失。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degraded: Vec<String>,
    /// 本轮没取到、因此**沿用了上一轮**的组件。
    ///
    /// 与 `degraded` 是两件事：`degraded` 说的是「这一段本轮取数失败」，
    /// 这一份说的是「所以你现在读到的是上一轮的数」。旧值各自的
    /// [`FactProvenance`] 原样保留，`as_of` / `fetched_at` 就是它是哪一轮的证据。
    /// 上游抖一下不该变成我们这边的永久数据丢失——这张表存在的全部理由，
    /// 就是**保存**过官方股本是多少。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub carried_over: Vec<String>,
    /// 这一行整体最后刷新的时刻（RFC3339）。
    ///
    /// **只在本轮至少有一段取数成功时才前进。** 全军覆没的那一轮把它留在原地，
    /// 否则 [`Self::is_stale`] 会拿一行空数据报告「新鲜」，把新鲜度这个信号变成谎言。
    #[serde(default)]
    pub refreshed_at: String,
    /// 最后一次**尝试**刷新的时刻（RFC3339），成功与否都记。
    /// 事件驱动的重刷用它做退避：财报日历滞后时不至于把同一家每小时刷一遍。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh_attempt_at: Option<String>,
}

fn default_schema_version() -> u32 {
    COMPANY_FACTS_SCHEMA_VERSION
}

impl CompanyFacts {
    pub fn new(symbol: &str) -> Self {
        Self {
            schema_version: COMPANY_FACTS_SCHEMA_VERSION,
            symbol: normalize_symbol(symbol),
            identity: CompanyIdentity::default(),
            shares: ShareCounts::default(),
            balance_sheet: BalanceSheetFacts::default(),
            income: IncomeFacts::default(),
            cash_flow: CashFlowFacts::default(),
            earnings: EarningsCadence::default(),
            days_since_latest_filing: None,
            degraded: Vec::new(),
            carried_over: Vec::new(),
            refreshed_at: chrono::Utc::now().to_rfc3339(),
            last_refresh_attempt_at: None,
        }
    }

    /// 这一行有没有任何一个实质字段。全空 = 从来没成功取到过东西。
    ///
    /// `degraded` / `carried_over` / `refreshed_at` 这些元数据不算——一行只有
    /// 「本轮什么都没取到」的记录，仍然是空行。
    pub fn is_empty(&self) -> bool {
        self.identity == CompanyIdentity::default()
            && self.shares == ShareCounts::default()
            && self.balance_sheet == BalanceSheetFacts::default()
            && self.income == IncomeFacts::default()
            && self.cash_flow == CashFlowFacts::default()
            && self.earnings == EarningsCadence::default()
    }

    /// 这一行整体是不是已经太旧。刷新时刻本身取不出来时按「陈旧」处理——
    /// 时间不可读的行不该被当成新鲜的。
    ///
    /// 空行一律算陈旧，哪怕 `refreshed_at` 是一秒前：一行没有任何数据的记录
    /// 自称新鲜，会让冷启动补刷的守卫跳过它、让研究台不报 `pipeline_stale`，
    /// 于是没数据这件事永远不会被任何人看见。
    pub fn is_stale(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        if self.is_empty() {
            return true;
        }
        match chrono::DateTime::parse_from_rfc3339(&self.refreshed_at) {
            Ok(at) => (now - at.with_timezone(&chrono::Utc)).num_hours() >= FACTS_STALE_AFTER_HOURS,
            Err(_) => true,
        }
    }

    /// 一句给读取方直接用的新鲜度说明。
    pub fn freshness_note(&self, now: chrono::DateTime<chrono::Utc>) -> String {
        let today = now.date_naive();
        match (self.shares.cover_shares, self.shares.cover_end.as_deref()) {
            (Some(shares), Some(end)) => {
                let age = self
                    .shares
                    .cover_age_days(today)
                    .map(|days| format!("，距今 {days} 天"))
                    .unwrap_or_default();
                format!(
                    "官方封面股数 {shares} 股（截至 {end}{age}，{}）",
                    self.shares.cover_form.as_deref().unwrap_or("申报表未知")
                )
            }
            _ => self
                .shares
                .cover_absent_note
                .clone()
                .unwrap_or_else(|| "本轮没有官方封面股数。".to_string()),
        }
    }
}

/// 代码归一：大写去空白。SEC 用 `-` 分隔股份类别，provider 常写 `.`；
/// 这里只归一大小写与空白，分隔符的归一留给 `sec_shares::normalize_sec_ticker`——
/// 那是查 SEC 时的事，存储键应当保持调用方给的形态（行业树里写的就是 `BRK-B` 这种）。
pub fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

// ── 存储 ────────────────────────────────────────────────────────────────────

static CLOUD_COMPANY_FACTS_STORAGE: OnceLock<RwLock<Option<CloudPgRuntime>>> = OnceLock::new();

/// 全局注入 PG 运行时。照 `configure_cloud_portfolio_storage` 的形状：
/// 在进程启动时调用一次，之后每个 `CompanyFactsStorage::new` 自动看到同一份。
pub fn configure_cloud_company_facts_storage(postgres: Option<CloudPgRuntime>) {
    let lock = CLOUD_COMPANY_FACTS_STORAGE.get_or_init(|| RwLock::new(None));
    match lock.write() {
        Ok(mut guard) => *guard = postgres,
        Err(error) => tracing::warn!("company facts cloud runtime lock poisoned: {error}"),
    }
}

fn cloud_company_facts_storage() -> Option<CloudPgRuntime> {
    CLOUD_COMPANY_FACTS_STORAGE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()))
}

/// 公司事实存储。配了 PG 走 PG，否则退回 `data_dir/company_facts/{SYMBOL}.json`。
pub struct CompanyFactsStorage {
    data_dir: PathBuf,
    cloud: Option<CloudPgRuntime>,
    _test_postgres_lease: Option<std::sync::Arc<crate::test_postgres::TestPostgresLease>>,
}

impl CompanyFactsStorage {
    /// `data_dir` 一般是 `config.storage.data_root()`；实际文件落在它下面的
    /// `company_facts/` 子目录。
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let dir = data_dir.as_ref().join("company_facts");
        let cloud = cloud_company_facts_storage();
        if cloud.is_none() {
            std::fs::create_dir_all(&dir).ok();
        }
        Self {
            data_dir: dir,
            cloud,
            _test_postgres_lease: None,
        }
    }

    /// PG 路径的测试构造器。`namespace` 只是隔离用的命名空间，不落文件。
    #[doc(hidden)]
    pub async fn new_isolated_postgres(namespace: impl AsRef<Path>) -> HoneResult<Self> {
        let (postgres, lease) = crate::test_postgres::isolated_postgres(namespace).await?;
        Ok(Self {
            data_dir: PathBuf::from("/nonexistent-company-facts-test"),
            cloud: Some(postgres),
            _test_postgres_lease: Some(lease),
        })
    }

    pub fn is_cloud(&self) -> bool {
        self.cloud.is_some()
    }

    fn path(&self, symbol: &str) -> PathBuf {
        self.data_dir.join(format!("{}.json", file_stem(symbol)))
    }

    pub async fn load(&self, symbol: &str) -> HoneResult<Option<CompanyFacts>> {
        let symbol = normalize_symbol(symbol);
        if let Some(postgres) = self.cloud.clone() {
            let Some(record) = postgres.get_company_facts(&symbol).await? else {
                return Ok(None);
            };
            return facts_from_value(record.facts).map(Some);
        }
        let path = self.path(&symbol);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str::<CompanyFacts>(&content)
            .map(Some)
            .map_err(|error| HoneError::Serialization(error.to_string()))
    }

    /// 一次取一批。整棵树 59 家是最常见的形状，逐个 `load` 会把一次页面渲染
    /// 变成 59 次数据库往返。取不到或解析不了的那家**跳过**，不让它拖垮整批。
    pub async fn load_many(&self, symbols: &[String]) -> HashMap<String, CompanyFacts> {
        let symbols: Vec<String> = symbols.iter().map(|item| normalize_symbol(item)).collect();
        if let Some(postgres) = self.cloud.clone() {
            let records = match postgres.list_company_facts(&symbols).await {
                Ok(records) => records,
                Err(error) => {
                    tracing::warn!("cloud company facts list failed: {error}");
                    return HashMap::new();
                }
            };
            return records
                .into_iter()
                .filter_map(|record| {
                    let symbol = record.symbol.clone();
                    match facts_from_value(record.facts) {
                        Ok(facts) => Some((symbol, facts)),
                        Err(error) => {
                            tracing::warn!(%symbol, "cloud company facts parse failed: {error}");
                            None
                        }
                    }
                })
                .collect();
        }
        let mut out = HashMap::with_capacity(symbols.len());
        for symbol in symbols {
            if let Ok(Some(facts)) = self.load(&symbol).await {
                out.insert(symbol, facts);
            }
        }
        out
    }

    pub async fn save(&self, facts: &CompanyFacts) -> HoneResult<()> {
        let mut payload = facts.clone();
        payload.symbol = normalize_symbol(&payload.symbol);
        if payload.symbol.is_empty() {
            return Err(HoneError::Config(
                "company_facts 需要一个非空的证券代码".to_string(),
            ));
        }
        if payload.refreshed_at.trim().is_empty() {
            payload.refreshed_at = chrono::Utc::now().to_rfc3339();
        }
        let value = serde_json::to_value(&payload)
            .map_err(|error| HoneError::Serialization(error.to_string()))?;

        if let Some(postgres) = self.cloud.clone() {
            let record = CloudCompanyFactsRecord {
                symbol: payload.symbol.clone(),
                // 提升出来的列只是 `facts` 的投影，永远从同一个结构体写，
                // 不接受调用方单独传——两处不一致就是下一次事故。
                cik: payload.identity.cik.and_then(|cik| i64::try_from(cik).ok()),
                exchange: payload.identity.exchange.clone(),
                is_adr: payload.identity.is_adr,
                cover_shares: payload.shares.cover_shares,
                cover_end: payload.shares.cover_end.clone(),
                cover_filed: payload.shares.cover_filed.clone(),
                cover_form: payload.shares.cover_form.clone(),
                facts: value,
                refreshed_at: payload.refreshed_at.clone(),
            };
            return postgres.upsert_company_facts(record).await;
        }

        std::fs::create_dir_all(&self.data_dir)?;
        let path = self.path(&payload.symbol);
        let json = serde_json::to_string_pretty(&value)
            .map_err(|error| HoneError::Serialization(error.to_string()))?;
        // 先写临时文件再 rename：读取方随时可能在读，半截 JSON 会被当成解析失败。
        let temp = path.with_extension(format!("json.tmp-{}", std::process::id()));
        std::fs::write(&temp, json)?;
        std::fs::rename(&temp, &path)?;
        Ok(())
    }

    /// 全部已存的公司。PG 下一次查询；本地扫目录。
    pub async fn list_all(&self) -> Vec<CompanyFacts> {
        if self.cloud.is_some() {
            let mut all: Vec<CompanyFacts> = self.load_many(&[]).await.into_values().collect();
            all.sort_by(|left, right| left.symbol.cmp(&right.symbol));
            return all;
        }
        let Ok(entries) = std::fs::read_dir(&self.data_dir) else {
            return Vec::new();
        };
        let mut all = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(facts) = serde_json::from_str::<CompanyFacts>(&content)
            {
                all.push(facts);
            }
        }
        all.sort_by(|left, right| left.symbol.cmp(&right.symbol));
        all
    }
}

fn facts_from_value(value: serde_json::Value) -> HoneResult<CompanyFacts> {
    serde_json::from_value(value).map_err(|error| HoneError::Serialization(error.to_string()))
}

/// 文件名只允许 `A-Z0-9-`，其余一律换成 `_`。代码本身已经归一成大写，
/// 这一层只是防住 `../` 这类被当成路径的输入。
fn file_stem(symbol: &str) -> String {
    let cleaned: String = symbol
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "_".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn lite_facts() -> CompanyFacts {
        let mut facts = CompanyFacts::new("lite");
        facts.identity.cik = Some(1_633_978);
        facts.identity.exchange = Some("NASDAQ".to_string());
        facts.identity.company_name = Some("Lumentum Holdings Inc.".to_string());
        facts.shares.cover_shares = Some(89_700_000);
        facts.shares.cover_end = Some("2026-08-14".to_string());
        facts.shares.cover_filed = Some("2026-08-17".to_string());
        facts.shares.cover_form = Some("10-K".to_string());
        facts.shares.cover_basis = Some("us_domestic_periodic_cover_page".to_string());
        facts.shares.cover_usable_for_market_cap = true;
        facts.shares.previous_cover_shares = Some(77_800_000);
        facts.shares.previous_cover_end = Some("2026-04-30".to_string());
        facts.shares.provider_shares = Some(77_800_000.0);
        facts.shares.provider_difference_pct = Some(15.3);
        facts.shares.basic_shares = Some(74_600_000.0);
        facts.shares.diluted_shares = Some(74_600_000.0);
        facts.shares.diluted_collapsed = true;
        facts.shares.cover_cik = Some(1_633_978);
        facts.shares.cover_provenance = FactProvenance::new(
            Some("2026-08-14".to_string()),
            "SEC XBRL dei:EntityCommonStockSharesOutstanding",
            "2026-08-30T11:00:00Z",
        );
        facts.shares.weighted_provenance = FactProvenance::new(
            Some("2026-06-28".to_string()),
            "provider income-statement (quarter)",
            "2026-08-30T11:00:00Z",
        );
        facts.refreshed_at = "2026-08-30T11:00:00Z".to_string();
        facts
    }

    #[tokio::test]
    async fn a_file_backed_roundtrip_keeps_every_share_field() {
        configure_cloud_company_facts_storage(None);
        let dir = make_temp_dir("hone_company_facts_roundtrip");
        let storage = CompanyFactsStorage::new(&dir);
        assert!(!storage.is_cloud());

        assert!(storage.load("LITE").await.expect("load empty").is_none());
        storage.save(&lite_facts()).await.expect("save");

        // 大小写与空白不该产生第二行。
        let loaded = storage
            .load("  lite ")
            .await
            .expect("load")
            .expect("exists");
        assert_eq!(loaded.symbol, "LITE");
        assert_eq!(loaded.shares.cover_shares, Some(89_700_000));
        assert_eq!(loaded.shares.previous_cover_shares, Some(77_800_000));
        assert_eq!(loaded.shares.cover_form.as_deref(), Some("10-K"));
        assert!(loaded.shares.diluted_collapsed);
        assert_eq!(
            loaded.shares.cover_provenance.source.as_deref(),
            Some("SEC XBRL dei:EntityCommonStockSharesOutstanding")
        );
        assert_eq!(
            loaded.shares.cover_provenance.as_of.as_deref(),
            Some("2026-08-14")
        );
        // 加权股本来自另一个源、另一个口径日期，出处必须是自己那一份。
        assert_eq!(
            loaded.shares.weighted_provenance.source.as_deref(),
            Some("provider income-statement (quarter)")
        );
        assert_eq!(
            loaded.shares.weighted_provenance.as_of.as_deref(),
            Some("2026-06-28")
        );

        assert_eq!(storage.list_all().await.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn saving_twice_replaces_the_row_instead_of_appending_one() {
        configure_cloud_company_facts_storage(None);
        let dir = make_temp_dir("hone_company_facts_replace");
        let storage = CompanyFactsStorage::new(&dir);
        storage.save(&lite_facts()).await.expect("first save");

        let mut updated = lite_facts();
        updated.shares.cover_shares = Some(90_100_000);
        storage.save(&updated).await.expect("second save");

        let all = storage.list_all().await;
        assert_eq!(all.len(), 1, "同一个代码只应有一行");
        assert_eq!(all[0].shares.cover_shares, Some(90_100_000));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn an_empty_symbol_is_refused_rather_than_written_to_a_junk_file() {
        configure_cloud_company_facts_storage(None);
        let dir = make_temp_dir("hone_company_facts_empty");
        let storage = CompanyFactsStorage::new(&dir);
        let mut facts = CompanyFacts::new("   ");
        facts.shares.cover_shares = Some(1);
        assert!(storage.save(&facts).await.is_err());
        assert!(storage.list_all().await.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PG 路径必须和文件路径读出同一份 —— 这是「两个进程看到同一份」的全部意义。
    /// 提升出来的那几列（cover_shares/cover_end/...）只是 `facts` 的投影，
    /// 所以这里读回来比对的是完整结构体，投影列写歪了也照样能被 SQL 查到。
    #[tokio::test]
    async fn a_postgres_roundtrip_matches_the_file_backed_one() {
        let Ok(storage) =
            CompanyFactsStorage::new_isolated_postgres("company_facts_pg_roundtrip").await
        else {
            // 没有 HONE_POSTGRES_* / DATABASE_URL 的环境跳过，不让本地开发变成必须起库。
            return;
        };
        assert!(storage.is_cloud());

        assert!(storage.load("LITE").await.expect("load empty").is_none());
        let saved = lite_facts();
        storage.save(&saved).await.expect("save");

        let loaded = storage.load("lite").await.expect("load").expect("exists");
        assert_eq!(loaded, saved);

        let batch = storage
            .load_many(&["LITE".to_string(), "NOSUCH".to_string()])
            .await;
        assert_eq!(batch.len(), 1);
        assert_eq!(batch["LITE"].shares.cover_shares, Some(89_700_000));

        let mut updated = saved.clone();
        updated.shares.cover_shares = Some(90_100_000);
        storage.save(&updated).await.expect("upsert");
        let all = storage.list_all().await;
        assert_eq!(all.len(), 1, "upsert 不该产生第二行");
        assert_eq!(all[0].shares.cover_shares, Some(90_100_000));
    }

    #[test]
    fn a_20f_cover_count_is_never_offered_as_market_cap_shares() {
        // TSM：官方 259.3 亿是台股普通股口径，1 ADR = 5 股。存下来可以，
        // 但 `market_cap_shares()` 必须拒绝把它交出去。
        let mut shares = ShareCounts {
            cover_shares: Some(25_932_524_521),
            cover_form: Some("20-F".to_string()),
            cover_basis: Some("foreign_or_non_periodic_filing".to_string()),
            cover_usable_for_market_cap: false,
            ..ShareCounts::default()
        };
        assert_eq!(shares.market_cap_shares(), None);

        shares.cover_usable_for_market_cap = true;
        shares.cover_form = Some("10-K".to_string());
        assert_eq!(shares.market_cap_shares(), Some(25_932_524_521));
    }

    #[test]
    fn cover_freshness_is_measured_from_the_cover_date_not_the_fetch_time() {
        // HTTP 200 不等于数据新鲜：BRK 的最新一条停在 2011-04-29。
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let fresh = ShareCounts {
            cover_end: Some("2026-08-14".to_string()),
            ..ShareCounts::default()
        };
        assert_eq!(fresh.cover_age_days(today), Some(16));
        assert!(!fresh.cover_is_stale(today));

        let ancient = ShareCounts {
            cover_end: Some("2011-04-29".to_string()),
            ..ShareCounts::default()
        };
        assert!(ancient.cover_is_stale(today));

        // 日期读不出来时不许假装新鲜，但也不该谎报陈旧。
        let unknown = ShareCounts::default();
        assert_eq!(unknown.cover_age_days(today), None);
        assert!(!unknown.cover_is_stale(today));
    }

    #[test]
    fn a_row_that_missed_two_daily_refreshes_reports_itself_as_stale() {
        let now = Utc.with_ymd_and_hms(2026, 8, 30, 12, 0, 0).unwrap();
        let mut facts = lite_facts();

        facts.refreshed_at = "2026-08-30T11:00:00Z".to_string();
        assert!(!facts.is_stale(now));
        facts.refreshed_at = "2026-08-28T11:00:00Z".to_string();
        assert!(facts.is_stale(now));
        // 时间不可读的行按陈旧处理，不按新鲜。
        facts.refreshed_at = "not-a-timestamp".to_string();
        assert!(facts.is_stale(now));
    }

    #[test]
    fn an_empty_row_never_reports_itself_as_fresh() {
        // 一轮里所有取数都失败、又没有上一轮可沿用时会落下这样一行。
        // 它自称新鲜就等于把「没数据」藏起来：冷启动补刷会跳过它，
        // 研究台也不会报 pipeline_stale。
        let now = Utc.with_ymd_and_hms(2026, 8, 30, 12, 0, 0).unwrap();
        let mut blank = CompanyFacts::new("LITE");
        blank.refreshed_at = "2026-08-30T11:59:00Z".to_string();
        blank.degraded = vec!["sec_cover_shares".to_string()];
        blank.last_refresh_attempt_at = Some("2026-08-30T11:59:00Z".to_string());
        assert!(blank.is_empty(), "只有降级记录的一行仍然是空行");
        assert!(blank.is_stale(now));

        // 有一个实质字段就不是空行了。
        let mut has_one = blank.clone();
        has_one.shares.cover_shares = Some(89_700_000);
        assert!(!has_one.is_empty());
        assert!(!has_one.is_stale(now));
    }

    #[test]
    fn a_missing_official_count_still_says_why() {
        let mut facts = CompanyFacts::new("GOOGL");
        facts.shares.cover_absent_reason = Some("concept_not_found".to_string());
        facts.shares.cover_absent_note =
            Some("多类别股发行人把封面股数打在 axis 维度上。".to_string());
        let now = Utc.with_ymd_and_hms(2026, 8, 30, 12, 0, 0).unwrap();
        assert!(facts.freshness_note(now).contains("axis"));
        assert!(lite_facts().freshness_note(now).contains("89700000"));
    }

    #[test]
    fn a_symbol_can_never_escape_the_facts_directory() {
        assert_eq!(file_stem("LITE"), "LITE");
        assert_eq!(file_stem("BRK-B"), "BRK-B");
        let escaped = file_stem("../../etc/passwd");
        assert!(
            !escaped.contains('/') && !escaped.contains('.'),
            "{escaped}"
        );
        assert_eq!(escaped.len(), "../../etc/passwd".len());
        assert_eq!(file_stem(""), "_");
    }
}
