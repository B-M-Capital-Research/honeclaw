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

    /// 不确定来源新闻的全局默认"重要性"短语。Per-actor `NotificationPrefs.
    /// news_importance_prompt = None` 时回落到这里。Router 把每条 source_class=
    /// uncertain 的 NewsCritical 与该 prompt 一起送 LLM 仲裁,LLM 判 yes 即升 Medium。
    #[serde(default = "default_news_importance_prompt")]
    pub news_importance_prompt: String,

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
            news_importance_prompt: default_news_importance_prompt(),
            dryrun: default_dryrun(),
        }
    }
}

fn default_news_importance_prompt() -> String {
    "公司或潜在影响公司长期逻辑和宏观叙事的重大事件".to_string()
}

/// 财报 poller 特有参数。
///
/// `window_days` 决定 EarningsPoller 每 tick 向 FMP earning_calendar 拉 `[today, today+N]`
/// 的天数;也就是 Hone 开始"关注"一家公司财报的提前量。**v0.1.46 起**,Poller 只产出
/// 稳定 id 的 `earnings:{SYM}:{DATE}` teaser(Medium);T-3/T-2/T-1 倒计时由 DigestScheduler
/// 在每次 flush 时刻从 EventStore 现算(见 `pollers::earnings::synthesize_countdowns`),
/// 这样 poller cron 漂移不会让倒计时 off-by-one。整条 lifecycle 仍共享 `earnings_upcoming`
/// kind,用户把它放进 `blocked_kinds` 就能一次静音 teaser + 所有倒计时。
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

/// **v0.1.46 破坏性简化**:只保留 `news_secs` / `price_secs` 这两类**真实时效性敏感**
/// 的 poller 配置。原来的 `earnings_secs` / `corp_action_secs` / `macro_secs` /
/// `analyst_grade_secs` / `earnings_surprise_secs` 5 个 24h 间隔字段被删除——对应
/// poller 改成 **cron-aligned**:在 `digest.pre_market` / `digest.post_market` 的前
/// `digest.prefetch_offset_mins` 分钟各执行一次拉取,这样推送的数据永远是 flush 之前
/// 刚拉的,不会因为用户重启时机而漂到几小时前。
///
/// 旧 config 里这 5 个字段即使仍存在也会被 `#[serde(default)]` + unknown-field tolerant
/// 悄悄忽略(serde 默认 deny_unknown_fields=false),YAML 不用改就能继续工作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollIntervals {
    #[serde(default = "default_news_interval")]
    pub news_secs: u64,
    #[serde(default = "default_price_interval")]
    pub price_secs: u64,
}

impl Default for PollIntervals {
    fn default() -> Self {
        Self {
            news_secs: default_news_interval(),
            price_secs: default_price_interval(),
        }
    }
}

fn default_news_interval() -> u64 {
    15 * 60
}
fn default_price_interval() -> u64 {
    5 * 60
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
    /// **cron-aligned poller** 在 flush 窗口前多少分钟执行拉取。v0.1.46 新增:
    /// earnings / corp_action / macro / analyst_grade / earnings_surprise 这 5 个
    /// 24h 节奏的 poller 不再用固定 interval 轮询,而是在 `pre_market - offset` /
    /// `post_market - offset` 各跑一次,保证推送数据永远是 flush 前刚拉的。
    /// 默认 30min;数值越小,数据越新但留给 EventStore/Router 处理的缓冲越紧。
    #[serde(default = "default_prefetch_offset_mins")]
    pub prefetch_offset_mins: u32,
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self {
            timezone: default_tz(),
            pre_market: default_pre_market(),
            post_market: default_post_market(),
            max_items_per_batch: default_max_items_per_batch(),
            prefetch_offset_mins: default_prefetch_offset_mins(),
        }
    }
}

fn default_prefetch_offset_mins() -> u32 {
    30
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
    /// 单次 poller tick 内,同一 ticker 触发 NewsCritical 升级 (Low→Medium)
    /// 的次数上限。0 = 不启用。防止一波 PR wire 把 digest 顶端淹满同一 ticker
    /// 的相关报道。
    #[serde(default = "default_news_upgrade_per_symbol_per_tick")]
    pub news_upgrade_per_symbol_per_tick: u32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            price_alert_low_pct: default_low_pct(),
            price_alert_high_pct: default_high_pct(),
            volume_sigma: default_sigma(),
            high_severity_daily_cap: default_cap(),
            same_symbol_cooldown_minutes: default_cooldown_minutes(),
            news_upgrade_per_symbol_per_tick: default_news_upgrade_per_symbol_per_tick(),
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
/// 默认 3:单 tick 内,同一 ticker 最多升级 3 条 Low→Medium。多于此的 NewsCritical
/// 维持 Low、不进 digest 顶端。0 关闭限流。
fn default_news_upgrade_per_symbol_per_tick() -> u32 {
    3
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
