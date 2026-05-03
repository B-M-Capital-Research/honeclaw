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

    #[serde(default)]
    pub sec_filings: SecFilingsConfig,

    #[serde(default)]
    pub global_digest: GlobalDigestConfig,

    /// 全局禁用的 event kind 标签列表（`kind_tag` 字符串，如 `"social_post"`）。
    /// Router 在 per-user prefs 之前先过一遍；入库仍然发生（便于日报统计），
    /// 只是不分发给任何 actor。部署方用于关闭噪音类事件。
    #[serde(default)]
    pub disabled_kinds: Vec<String>,

    /// 不确定来源新闻的全局默认"重要性"短语。Per-actor `NotificationPrefs.
    /// news_importance_prompt = None` 时回落到这里。Router 把每条 source_class=
    /// uncertain 的 NewsCritical 与该 prompt 一起送 LLM 仲裁,LLM 判 yes 即升 Medium。
    #[serde(default = "default_news_importance_prompt")]
    pub news_importance_prompt: String,

    /// 不确定来源新闻 LLM 仲裁模型。走 OpenRouter 兼容 chat completions。
    /// 留空时装配层回退到默认值。
    #[serde(default = "default_news_classifier_model")]
    pub news_classifier_model: String,
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
            sec_filings: SecFilingsConfig::default(),
            global_digest: GlobalDigestConfig::default(),
            disabled_kinds: Vec::new(),
            news_importance_prompt: default_news_importance_prompt(),
            news_classifier_model: default_news_classifier_model(),
        }
    }
}

fn default_news_importance_prompt() -> String {
    "公司或潜在影响公司长期逻辑和宏观叙事的重大事件".to_string()
}

fn default_news_classifier_model() -> String {
    "amazon/nova-lite-v1".to_string()
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

/// SEC filings poller 配置。
///
/// `forms` 决定 `SecFilingsPoller` 每 tick 对每只 watchlist ticker 拉哪些 form 类型。
/// 默认覆盖 8-K(突发披露,High)/ 10-Q(季报,Medium)/ 10-K(年报,Medium)/
/// S-1(IPO 或追加发行,High)/ DEF 14A(委托书,Low)。Severity 由 `events_from_sec_filings`
/// 在事件构造时按 form 类型映射,**不是**在 config 里配置。
///
/// `enrichment` 子配置控制是否调 LLM 给每条 filing 生成 ~200 字业务摘要(长期主线投资者
/// 视角,跳过 GAAP 数字、抓 backlog/资本配置/风险)。POC 实证 grok-4.1-fast 在 11 持仓
/// 一年 ~70 条 filing × $0.012 ≈ $0.82/年,质量、成本、延迟均第一。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFilingsConfig {
    #[serde(default = "default_sec_forms")]
    pub forms: Vec<String>,
    #[serde(default)]
    pub enrichment: SecFilingsEnrichmentConfig,
}

impl Default for SecFilingsConfig {
    fn default() -> Self {
        Self {
            forms: default_sec_forms(),
            enrichment: SecFilingsEnrichmentConfig::default(),
        }
    }
}

fn default_sec_forms() -> Vec<String> {
    vec![
        "8-K".into(),
        "10-Q".into(),
        "10-K".into(),
        "S-1".into(),
        "DEF 14A".into(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFilingsEnrichmentConfig {
    /// 是否给 SEC filing 事件调 LLM 生成业务摘要;关闭则只走原始 form/link body。
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// LLM 模型名(OpenRouter 风格)。POC 验证 `x-ai/grok-4.1-fast` 质量、成本、延迟均最佳。
    #[serde(default = "default_sec_summary_model")]
    pub model: String,
    /// 摘要 max_tokens 上限。grok-4.1-fast 在 ~200 字目标下,800 token 充足且不会被截断。
    #[serde(default = "default_sec_summary_max_tokens")]
    pub max_summary_tokens: u32,
    /// fetch SEC.gov 时使用的 User-Agent。**SEC 强制要求格式包含联系邮箱**,否则会被
    /// 限流或拒绝。空字符串则不调 enrichment(关闭通道)。
    #[serde(default = "default_sec_user_agent")]
    pub user_agent: String,
}

impl Default for SecFilingsEnrichmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model: default_sec_summary_model(),
            max_summary_tokens: default_sec_summary_max_tokens(),
            user_agent: default_sec_user_agent(),
        }
    }
}

fn default_sec_summary_model() -> String {
    "x-ai/grok-4.1-fast".into()
}

fn default_sec_summary_max_tokens() -> u32 {
    800
}

fn default_sec_user_agent() -> String {
    // 占位邮箱:部署方应改成自己的联系邮箱。SEC 不要求邮箱真实可达,但要求格式有
    // 公司/产品名 + 邮箱;长期不改有被 rate-limit 的风险。
    "honeclaw event-engine ops@honeclaw.local".into()
}

/// 全局 digest LLM 子配置 —— 从 commit 3 起,unified pipeline 复用本配置承载
/// curator / fetcher / event_dedupe 旋钮。触发改由 per-actor `prefs.digest_slots` 驱动。
///
/// 候选池(trusted-source High/Medium news + macro_event)由 unified scheduler
/// 在每个 slot 触发时拉取,经 Pass 1 聚类 + Pass 2 精读后,与 buffer/synth 候选
/// 在 per-actor fan-out 阶段合流。
///
/// 默认 `enabled=false`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDigestConfig {
    #[serde(default)]
    pub enabled: bool,

    /// admin 视角时区,目前仅用于历史日志解释;实际触发时刻由 actor `prefs.digest_slots` 决定。
    #[serde(default = "default_global_digest_tz")]
    pub timezone: String,

    /// 候选池的回看窗口(小时)。第一次推送 / 兜底用;后续以"距上次成功推送"为准。
    #[serde(default = "default_global_digest_lookback_hours")]
    pub lookback_hours: u32,

    /// Pass 1 模型 —— 候选池批量打分 + cluster + 一句话 takeaway。便宜模型即可。
    #[serde(default = "default_global_digest_pass1_model")]
    pub pass1_model: String,

    /// Pass 2 模型 —— 抓原文后精读、最终排序、写短评。需要相对聪明。
    #[serde(default = "default_global_digest_pass2_model")]
    pub pass2_model: String,

    /// Pass 1 排序后送 Pass 2 精读的候选数上限。
    #[serde(default = "default_global_digest_pass2_top_n")]
    pub pass2_top_n: u32,

    /// 单次推送最终保留的条数上限。Pass 2 可以再剔除,但不会超过这个数。
    #[serde(default = "default_global_digest_final_pick_n")]
    pub final_pick_n: u32,

    /// Pass 2 是否抓原文(GET 文章 URL → html2text)。失败 fallback 到事件本体的
    /// FMP `text` 摘要。关闭则始终只用 FMP 文本。
    #[serde(default = "default_true")]
    pub fetch_full_text: bool,

    /// **事件级去重**(POC 验证 2026-04-26 修):collector 之后、Pass1 之前,用强
    /// LLM 把同一具体事件的多源报道合成 1 条代表,避免 picks 被同事件不同包装挤满。
    /// 关闭(false)时退回 Pass1 自带的 cluster id dedup(已知不可靠)。
    #[serde(default = "default_true")]
    pub event_dedupe_enabled: bool,

    /// 事件级 dedup 用的 LLM 模型。POC 验证 grok-4.1-fast 在 17-236 候选量级上
    /// 稳定保守(只合明显同事件)。务必用强模型,nova-lite 这种会过度归类成 theme。
    #[serde(default = "default_event_dedupe_model")]
    pub event_dedupe_model: String,

    /// Jina Reader API key。Pass 2 直抓原文返回非 2xx(典型 reuters/wsj/barrons 401)
    /// 时,带 key 走 `https://r.jina.ai/<url>` 二次抓取;Jina 用无头浏览器渲染 + 抽
    /// 正文,对付费墙站点能拿到试读段落,对 Reuters 类反爬站点直接拿全文。空 / None
    /// 时跳过这一层 fallback,直接落到事件本体的 FMP `text`。免费层 1M tokens/月,
    /// jina.ai 邮箱注册即得。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jina_api_key: Option<String>,
}

impl Default for GlobalDigestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timezone: default_global_digest_tz(),
            lookback_hours: default_global_digest_lookback_hours(),
            pass1_model: default_global_digest_pass1_model(),
            pass2_model: default_global_digest_pass2_model(),
            pass2_top_n: default_global_digest_pass2_top_n(),
            final_pick_n: default_global_digest_final_pick_n(),
            fetch_full_text: true,
            event_dedupe_enabled: true,
            event_dedupe_model: default_event_dedupe_model(),
            jina_api_key: None,
        }
    }
}

fn default_event_dedupe_model() -> String {
    "x-ai/grok-4.1-fast".into()
}

fn default_global_digest_tz() -> String {
    "Asia/Shanghai".into()
}
fn default_global_digest_lookback_hours() -> u32 {
    24
}
fn default_global_digest_pass1_model() -> String {
    "x-ai/grok-4.1-fast".into()
}
fn default_global_digest_pass2_model() -> String {
    "x-ai/grok-4.1-fast".into()
}
fn default_global_digest_pass2_top_n() -> u32 {
    15
}
fn default_global_digest_final_pick_n() -> u32 {
    8
}

fn default_enabled() -> bool {
    false
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

/// 单个默认 digest slot 时刻。`label` 缺省时,scheduler 渲染成 `定时摘要 · HH:MM`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultDigestSlot {
    pub time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Digest 触发窗口配置。
///
/// `timezone` 默认 Asia/Shanghai（UTC+8）。`default_slots` 是 actor 没自定义
/// `prefs.digest_slots` 时的兜底触发时刻。默认两个槽:
/// * 08:30 "盘前摘要" — CN 用户开工前合并推送一条。
/// * 09:00 "晨间摘要" — 美股盘后收于北京凌晨,延后到早上推送以免半夜打扰。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestConfig {
    #[serde(default = "default_tz")]
    pub timezone: String,
    #[serde(default = "default_default_slots")]
    pub default_slots: Vec<DefaultDigestSlot>,
    /// 单条摘要最多渲染多少事件，超出截断并附"另 N 条已省略"。0 = 不限制。
    #[serde(default = "default_max_items_per_batch")]
    pub max_items_per_batch: u32,
    /// **cron-aligned poller** 在 flush 窗口前多少分钟执行拉取。v0.1.46 新增:
    /// earnings / corp_action / macro / analyst_grade / earnings_surprise 这 5 个
    /// 24h 节奏的 poller 不再用固定 interval 轮询,而是在每个 default slot - offset 跑一次,
    /// 保证推送数据永远是 flush 前刚拉的。默认 30min。
    #[serde(default = "default_prefetch_offset_mins")]
    pub prefetch_offset_mins: u32,
    /// 同一 actor 两次 digest 之间的最小间隔。用于用户配置了很多窗口时避免
    /// 同一批主题在短时间内反复出现。0 = 不启用。
    #[serde(default = "default_min_gap_minutes")]
    pub min_gap_minutes: u32,
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self {
            timezone: default_tz(),
            default_slots: default_default_slots(),
            max_items_per_batch: default_max_items_per_batch(),
            prefetch_offset_mins: default_prefetch_offset_mins(),
            min_gap_minutes: default_min_gap_minutes(),
        }
    }
}

fn default_prefetch_offset_mins() -> u32 {
    30
}
fn default_min_gap_minutes() -> u32 {
    180
}

fn default_tz() -> String {
    "Asia/Shanghai".into()
}
fn default_default_slots() -> Vec<DefaultDigestSlot> {
    vec![
        DefaultDigestSlot {
            time: "08:30".into(),
            label: Some("盘前摘要".into()),
        },
        DefaultDigestSlot {
            // 美股隔夜收盘摘要延后到北京时间早上 9 点推送,避免半夜打扰。
            time: "09:00".into(),
            label: Some("晨间摘要".into()),
        },
    ]
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
    /// 单次 poller tick 内 NewsCritical 升级 (Low→Medium) 的总次数上限。
    /// 0 = 不启用。防止多 ticker 同时提级形成摘要洪峰。
    #[serde(default = "default_news_upgrade_per_tick")]
    pub news_upgrade_per_tick: u32,
    /// 用户价格阈值覆盖不能低于该系统级最小直推阈值，除非事件 payload 标明
    /// portfolio_weight / portfolio_weight_pct 达到大仓位阈值。
    #[serde(default = "default_price_min_direct_pct")]
    pub price_min_direct_pct: f64,
    /// 价格异动跨档后的再提醒步长百分比。例如 high=6, step=2 时,盘中跨
    /// +6/+8/+10 或 -6/-8/-10 会形成独立 band 事件。
    #[serde(default = "default_price_realert_step_pct")]
    pub price_realert_step_pct: f64,
    /// **价格 band 单一推送规则(v0.5.2 起替代旧 cap+gap 双保险)**:
    /// 同 actor + symbol + direction 内,新到 band 的档位 pct 必须比当日已 sink-sent
    /// 的最大档 pct 至少高出本字段值,才被允许直推;否则降级进 digest。
    ///
    /// 与旧机制相比:band id 已自带「同档位 INSERT IGNORE」,所以不需要时间 gap 兜底
    /// 防重;daily cap 在 N=2 时退化为「监控所有 monotone 新高」—— 既不会在大行情
    /// 长尾失声,也不会被同档位震荡刷屏。POC(2026-05-02)实证 AAOI 6→8→10→12→14→16
    /// 序列下,N=2 给出全部 6 档(用户原 cap=2 仅 6/8 严重失声)。
    ///
    /// 0 = 关闭推送限制(等于无脑全推);默认 2.0 与 `price_realert_step_pct`
    /// 一致,意为「每跨一个新 band 必推」。
    #[serde(default = "default_price_band_min_advance_pct")]
    pub price_band_min_advance_pct: f64,
    /// 收盘 price_close 是否允许即时推。默认 false,避免美股收盘在北京凌晨打扰。
    #[serde(default = "default_price_close_direct_enabled")]
    pub price_close_direct_enabled: bool,
    /// 高仓位标的使用用户自定义价格阈值直推的最小仓位权重百分比。
    #[serde(default = "default_large_position_weight_pct")]
    pub large_position_weight_pct: f64,
    /// High 宏观事件只有在发生前该小时数内才允许即时推；更远期降级摘要。
    #[serde(default = "default_macro_immediate_lookahead_hours")]
    pub macro_immediate_lookahead_hours: i64,
    /// High 宏观事件发生后该小时数内仍允许即时推；更旧事件降级摘要。
    #[serde(default = "default_macro_immediate_grace_hours")]
    pub macro_immediate_grace_hours: i64,
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
            news_upgrade_per_tick: default_news_upgrade_per_tick(),
            price_min_direct_pct: default_price_min_direct_pct(),
            price_realert_step_pct: default_price_realert_step_pct(),
            price_band_min_advance_pct: default_price_band_min_advance_pct(),
            price_close_direct_enabled: default_price_close_direct_enabled(),
            large_position_weight_pct: default_large_position_weight_pct(),
            macro_immediate_lookahead_hours: default_macro_immediate_lookahead_hours(),
            macro_immediate_grace_hours: default_macro_immediate_grace_hours(),
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
/// 默认 12:单 tick 内最多升级 12 条 Low→Medium。多于此的 NewsCritical
/// 维持 Low,避免跨 ticker 的收敛洪峰挤占摘要。
fn default_news_upgrade_per_tick() -> u32 {
    12
}
fn default_price_min_direct_pct() -> f64 {
    6.0
}
fn default_price_realert_step_pct() -> f64 {
    2.0
}
fn default_price_band_min_advance_pct() -> f64 {
    2.0
}
fn default_price_close_direct_enabled() -> bool {
    false
}
fn default_large_position_weight_pct() -> f64 {
    20.0
}
fn default_macro_immediate_lookahead_hours() -> i64 {
    6
}
fn default_macro_immediate_grace_hours() -> i64 {
    2
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
    /// `spawn_price_poller` —— FMP /v3/quote 按 watch pool 拉,产出 PriceAlert/52W
    #[serde(default = "default_true")]
    pub price: bool,
    /// `spawn_extended_hours_poller` —— FMP /v3/historical-chart/1min?extended=true 按
    /// watch pool 拉,30min cadence,只在 ET 04:00-09:30 / 16:00-20:00 工作。盘前/盘后
    /// 振幅 ≥ 用户阈值时产出 PriceAlert{window: "pre"|"post"}。FMP 常规 quote endpoint
    /// 不在 extended hours 更新 timestamp,会被 PricePoller 判 stale 跳过(根因:GOOGL
    /// 财报夜整夜无推送),本通道补这块盲区。
    #[serde(default = "default_true")]
    pub extended_hours: bool,
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

    /// Telegram 公开频道监听(web preview `t.me/s/<handle>`),产出 `SocialPost`。
    /// 空列表 = 不启用。每条配置对应一个独立 poller loop。
    #[serde(default)]
    pub telegram_channels: Vec<TelegramChannelConfig>,

    /// 通用 RSS 新闻源(global_digest 用)。POC 验证 FMP 漏掉 Bloomberg(93%)/
    /// SpaceNews(100%)/STAT(100%)的关键料,这些 RSS 直接把高 ROI 的料补回来。
    /// 入库 source 标 `rss:{handle}`,collector 一并拉。空列表 = 不启用。
    #[serde(default)]
    pub rss_feeds: Vec<RssFeedConfig>,
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            news: true,
            price: true,
            extended_hours: true,
            earnings_calendar: true,
            corp_action: true,
            sec_filings: true,
            macro_calendar: true,
            analyst_grade: true,
            earnings_surprise: true,
            telegram_channels: Vec::new(),
            rss_feeds: Vec::new(),
        }
    }
}

/// Telegram 公开频道配置。
///
/// 通过 `https://t.me/s/<handle>` 无 token 抓取最新 ~20 条帖子。`extract_cashtags`
/// 开启时会把正文里的 `$TICKER` 提取到 `MarketEvent.symbols`,便于 actor 订阅命中;
/// 关闭时 symbols 为空,依赖 social GlobalSubscription + LLM 仲裁升级走全局分发。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChannelConfig {
    pub handle: String,
    #[serde(default = "default_social_interval")]
    pub interval_secs: u64,
    #[serde(default)]
    pub extract_cashtags: bool,
}

fn default_social_interval() -> u64 {
    30 * 60
}

/// 通用 RSS 新闻源配置。`handle` 是该源的稳定标签,会写进 `MarketEvent.source =
/// "rss:{handle}"`,后续 collector / dedup / log 都按它寻找。`url` 是 RSS 2.0 feed
/// 的 GET 地址。`interval_secs` 默认 30 分钟,与社交源同档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeedConfig {
    pub handle: String,
    pub url: String,
    #[serde(default = "default_rss_interval")]
    pub interval_secs: u64,
}

fn default_rss_interval() -> u64 {
    30 * 60
}

fn default_true() -> bool {
    true
}
