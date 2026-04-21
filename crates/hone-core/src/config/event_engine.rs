//! 事件引擎配置。
//!
//! 与 `config.yaml` 的 `event_engine:` 节对应。放在 hone-core 内部，供
//! `hone-event-engine` 消费；hone-core 自身不依赖 engine 代码。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEngineConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub poll_intervals: PollIntervals,

    #[serde(default)]
    pub digest: DigestConfig,

    #[serde(default)]
    pub thresholds: Thresholds,

    #[serde(default)]
    pub renderer: RendererConfig,

    #[serde(default)]
    pub sources: Sources,

    #[serde(default)]
    pub earnings: EarningsConfig,

    /// 全局禁用的 event kind 标签列表（`kind_tag` 字符串，如 `"press_release"`）。
    /// Router 在 per-user prefs 之前先过一遍；入库仍然发生（便于日报统计），
    /// 只是不分发给任何 actor。部署方用于关闭噪音类事件。
    #[serde(default)]
    pub disabled_kinds: Vec<String>,

    #[serde(default = "default_dryrun")]
    pub dryrun: bool,
}

impl Default for EventEngineConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            poll_intervals: PollIntervals::default(),
            digest: DigestConfig::default(),
            thresholds: Thresholds::default(),
            renderer: RendererConfig::default(),
            sources: Sources::default(),
            earnings: EarningsConfig::default(),
            disabled_kinds: Vec::new(),
            dryrun: default_dryrun(),
        }
    }
}

/// 财报 poller 特有参数。
///
/// `window_days` 决定 EarningsPoller 每 tick 向 FMP earning_calendar 拉 `[today, today+N]`
/// 的天数；也就是 Hone 开始"关注"一家公司财报的提前量。`EarningsPoller` 在此基础上会
/// 对距今 T-3/T-2/T-1 的财报额外发送每日倒计时事件(id 带 `:countdown:N` 后缀避免 store
/// 去重折叠;T-1 升级为 High 立即推,T-2/T-3 维持 Medium 进 digest)。用户若 `blocked_kinds`
/// 包含 `earnings_upcoming`,则全程静音(初次预告 + 每日倒计时一并拦住)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsConfig {
    #[serde(default = "default_earnings_window_days")]
    pub window_days: i64,
}

impl Default for EarningsConfig {
    fn default() -> Self {
        Self {
            window_days: default_earnings_window_days(),
        }
    }
}

fn default_earnings_window_days() -> i64 {
    14
}

fn default_enabled() -> bool {
    false
}

fn default_dryrun() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollIntervals {
    #[serde(default = "default_news_interval")]
    pub news_secs: u64,
    #[serde(default = "default_price_interval")]
    pub price_secs: u64,
    #[serde(default = "default_daily_interval")]
    pub earnings_secs: u64,
    #[serde(default = "default_daily_interval")]
    pub corp_action_secs: u64,
    #[serde(default = "default_daily_interval")]
    pub macro_secs: u64,
    /// 分析师评级拉取间隔。默认 24h——评级变更基本是日频节奏,更频繁拉只会
    /// 放大 FMP 配额压力，不提升时效性。
    #[serde(default = "default_daily_interval")]
    pub analyst_grade_secs: u64,
    /// 财报 surprise 拉取间隔。默认 24h;真实的时效性靠 scheduler 在盘后
    /// (post-market 窗口附近)做一次集中扫即可。
    #[serde(default = "default_daily_interval")]
    pub earnings_surprise_secs: u64,
}

impl Default for PollIntervals {
    fn default() -> Self {
        Self {
            news_secs: default_news_interval(),
            price_secs: default_price_interval(),
            earnings_secs: default_daily_interval(),
            corp_action_secs: default_daily_interval(),
            macro_secs: default_daily_interval(),
            analyst_grade_secs: default_daily_interval(),
            earnings_surprise_secs: default_daily_interval(),
        }
    }
}

fn default_news_interval() -> u64 {
    15 * 60
}
fn default_price_interval() -> u64 {
    5 * 60
}
fn default_daily_interval() -> u64 {
    24 * 60 * 60
}

/// Digest 触发窗口配置。
///
/// `timezone` 默认 Asia/Shanghai（UTC+8）。两条固定窗口：
/// * `pre_market` — 本地"早班"窗口，默认 08:30，用于在 CN 用户开工前把待推送的
///   Medium/Low 事件合并推一条。
/// * `post_market` — 本地"盘后"窗口，默认 09:00。因为美股盘后收于北京时间凌晨，
///   直接在收盘时推送会把人吵醒；改到早上 8~10 点（可配置），让用户起床后看到
///   隔夜美股汇总。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestConfig {
    #[serde(default = "default_tz")]
    pub timezone: String,
    #[serde(default = "default_pre_market")]
    pub pre_market: String,
    #[serde(default = "default_post_market")]
    pub post_market: String,
    /// 单条摘要最多渲染多少事件，超出截断并附"另 N 条已省略"。0 = 不限制。
    #[serde(default = "default_max_items_per_batch")]
    pub max_items_per_batch: u32,
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self {
            timezone: default_tz(),
            pre_market: default_pre_market(),
            post_market: default_post_market(),
            max_items_per_batch: default_max_items_per_batch(),
        }
    }
}

fn default_tz() -> String {
    "Asia/Shanghai".into()
}
fn default_pre_market() -> String {
    "08:30".into()
}
fn default_post_market() -> String {
    // 美股隔夜收盘摘要延后到北京时间早上 9 点推送，避免半夜打扰。
    "09:00".into()
}
fn default_max_items_per_batch() -> u32 {
    20
}

/// 粗粒度 IANA 时区名 → UTC 偏移小时数。不识别的名字返回 0（UTC）。
/// MVP 阶段不接 chrono-tz，夏令时按常用区域做固定近似。
pub fn tz_offset_hours(tz: &str) -> i32 {
    match tz.trim() {
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" | "Asia/Taipei" | "PRC" => 8,
        "Asia/Tokyo" | "Asia/Seoul" => 9,
        "Europe/London" | "UTC" | "GMT" | "" => 0,
        "Europe/Paris" | "Europe/Berlin" | "Europe/Amsterdam" | "Europe/Madrid" => 1,
        "America/New_York" | "America/Toronto" => -4, // 夏令时近似
        "America/Chicago" => -5,
        "America/Denver" => -6,
        "America/Los_Angeles" => -7,
        other => {
            tracing::warn!("未识别的 timezone '{other}'，回退 UTC");
            0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    #[serde(default = "default_low_pct")]
    pub price_alert_low_pct: f64,
    #[serde(default = "default_high_pct")]
    pub price_alert_high_pct: f64,
    #[serde(default = "default_sigma")]
    pub volume_sigma: f64,
    #[serde(default = "default_cap")]
    pub high_severity_daily_cap: u32,
    /// 同一 ticker 两次 High sink 推送的最小间隔（分钟）。0 = 不启用。
    /// 防止一个 ticker 在短时间内被价格异动、新闻、filing 连环轰炸同一用户。
    #[serde(default = "default_cooldown_minutes")]
    pub same_symbol_cooldown_minutes: u32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            price_alert_low_pct: default_low_pct(),
            price_alert_high_pct: default_high_pct(),
            volume_sigma: default_sigma(),
            high_severity_daily_cap: default_cap(),
            same_symbol_cooldown_minutes: default_cooldown_minutes(),
        }
    }
}

// 生产默认:
// - low_pct 2.5 — 美股日内 ±2.5% 已够上"值得关注"门槛;保留为 Medium/Low 入 digest
// - high_pct 6.0 — ±6% 才升级到 High 立即推。10% 在典型持仓几乎不触发,漏掉大部分异动
//   阈值可以通过 `event_engine.thresholds.price_alert_{low,high}_pct` 覆盖
fn default_low_pct() -> f64 {
    2.5
}
fn default_high_pct() -> f64 {
    6.0
}
fn default_sigma() -> f64 {
    2.0
}
fn default_cap() -> u32 {
    8
}
/// 默认 60 分钟:同一 ticker 每小时最多一次 High sink 推送;其它在摘要里合并。
fn default_cooldown_minutes() -> u32 {
    60
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RendererConfig {
    #[serde(default)]
    pub llm_polish_for: Vec<String>,
    #[serde(default)]
    pub template_dir: Option<String>,
}

/// Per-poller 开关。每个字段对应 `crates/hone-event-engine/src/lib.rs::start`
/// 里的一个 spawn_*_poller 调用,关闭即直接 skip 该 poller 的 tick(最省 FMP 配额)。
///
/// 想要更细粒度的"跑 poller 但不分发某 kind"的兜底关法,用 `EventEngineConfig.disabled_kinds`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sources {
    /// `spawn_news_poller` —— FMP /v3/stock_news,产出 NewsCritical
    #[serde(default = "default_true")]
    pub news: bool,
    /// `spawn_price_poller` —— FMP /v3/quote 按 watch pool 拉,产出 PriceAlert/52W/VolumeSpike
    #[serde(default = "default_true")]
    pub price: bool,
    /// `spawn_earnings_poller` —— FMP /v3/earning_calendar,产出 EarningsUpcoming
    #[serde(default = "default_true")]
    pub earnings_calendar: bool,
    /// `corp_action_poller` 内部的 dividend/split 全局日历分支
    #[serde(default = "default_true")]
    pub corp_action: bool,
    /// `corp_action_poller` 内部的 SEC 8-K per-ticker 分支
    #[serde(default = "default_true")]
    pub sec_filings: bool,
    /// `spawn_macro_poller` —— FMP /v3/economic_calendar,产出 MacroEvent
    #[serde(default = "default_true")]
    pub macro_calendar: bool,
    /// `spawn_analyst_grade_poller` —— 按 watch pool 拉,产出 AnalystGrade
    #[serde(default = "default_true")]
    pub analyst_grade: bool,
    /// `spawn_earnings_surprise_poller` —— 按 watch pool 拉,产出 EarningsReleased
    #[serde(default = "default_true")]
    pub earnings_surprise: bool,
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            news: true,
            price: true,
            earnings_calendar: true,
            corp_action: true,
            sec_filings: true,
            macro_calendar: true,
            analyst_grade: true,
            earnings_surprise: true,
        }
    }
}

fn default_true() -> bool {
    true
}
