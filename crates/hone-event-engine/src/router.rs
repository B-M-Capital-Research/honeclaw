//! NotificationRouter — 按 severity 分流。
//!
//! - `High` → 立即调 `OutboundSink::send`
//! - `Medium` / `Low` → 入 `DigestBuffer`，由 `DigestScheduler` 在 ET 盘前/盘后合并推送。
//!
//! MVP 的 `OutboundSink` 实现只打 `tracing::info` 日志（dryrun 语义）；真实
//! 渠道适配器在后续 step 接入。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hone_core::ActorIdentity;
use tracing::info;

use crate::digest::DigestBuffer;
use crate::event::{EventKind, MarketEvent, Severity};
use crate::news_classifier::{DEFAULT_IMPORTANCE_PROMPT, Importance, NewsClassifier};
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::prefs::{AllowAllPrefs, NotificationPrefs, PrefsProvider, kind_tag};
use crate::renderer::{self, RenderFormat};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;

/// 同日命中后可以把 Low 新闻升到 Medium 的硬信号 kind tag 集合。
/// 语义：ticker 当天已出现过这些"事实性"事件时,同 ticker 的低优先级新闻
/// 很可能是相关解读,升到 Medium 让它进 digest 而不是沉底。
const NEWS_CONVERGENCE_HARD_SIGNALS: &[&str] = &[
    "price_alert",
    "earnings_released",
    "earnings_upcoming",
    "sec_filing",
    "analyst_grade",
];

#[async_trait]
pub trait OutboundSink: Send + Sync {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()>;

    /// 成功送达后写入 delivery_log 的 status。真实 sink 返回 `sent`;dryrun sink
    /// 返回 `dryrun`，避免 dryrun 被统计成真实 ack。
    fn success_status(&self) -> &'static str {
        "sent"
    }

    /// 该 Sink 期望的消息格式。若渠道使用富文本,override 这里同时在 send()
    /// 带上对应的 parse_mode / msg_type,否则会出现 `<b>` 当字面量泄露。
    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }

    /// MultiChannelSink 这类按 actor.channel 分发的 sink 需要按目标渠道选择格式。
    fn format_for(&self, _actor: &ActorIdentity) -> RenderFormat {
        self.format()
    }
}

/// 默认 Sink：把渲染后的消息写 tracing::info，用于 dryrun 与测试。
pub struct LogSink;

#[async_trait]
impl OutboundSink for LogSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        info!(
            actor = %actor_key(actor),
            "[dryrun sink] {body}"
        );
        Ok(())
    }

    fn success_status(&self) -> &'static str {
        "dryrun"
    }
}

pub struct NotificationRouter {
    registry: Arc<SharedRegistry>,
    sink: Arc<dyn OutboundSink>,
    store: Arc<EventStore>,
    digest: Arc<DigestBuffer>,
    polisher: Arc<dyn BodyPolisher>,
    prefs: Arc<dyn PrefsProvider>,
    /// 每 actor 当日 sink=sent 且 severity=high 的条数上限。超过后新的 High
    /// 事件自动降级进 digest,并在 delivery_log 写 status="capped"。
    /// 0 = 不启用。
    high_daily_cap: u32,
    /// 解释"当日"起点所用的 UTC 偏移(小时)。
    tz_offset_hours: i32,
    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 防止同一 ticker 短时间内被价格异动 + 新闻 + SEC filing 三连推。
    /// 命中后降级到 digest,log_delivery 写 status="cooled_down"。
    same_symbol_cooldown_minutes: u32,
    /// 用户价格阈值覆盖的系统级最小即时推阈值。
    price_min_direct_pct: f64,
    /// 大仓位标的用用户敏感阈值直推的默认仓位权重门槛。
    large_position_weight_pct: f64,
    /// MacroEvent High 允许即时推的临近窗口。
    macro_immediate_lookahead_hours: i64,
    macro_immediate_grace_hours: i64,
    /// 部署方配置的全局 kind 黑名单。命中后 dispatch 直接返回 (0, 0),
    /// 任何 actor 的 prefs / cap / cooldown 都不再参与。
    disabled_kinds: Arc<HashSet<String>>,
    /// 单次 poller tick 内,同一 ticker 触发 NewsCritical 升级 (Low→Medium)
    /// 的次数上限。0 = 不启用。命中后该条 Low 维持 Low,从而不进 digest 顶端。
    news_upgrade_per_symbol_per_tick_cap: u32,
    /// 单次 poller tick 内 NewsCritical 升级 (Low→Medium) 的全局总上限。
    /// 0 = 不启用。用于防止多 ticker 同时提级造成摘要洪峰。
    news_upgrade_per_tick_cap: u32,
    /// 当 tick 内每个 symbol 已升级的次数。`reset_tick_counters()` 在每次
    /// `process_events` 入口被调用,清零后重新计数。
    news_upgrade_counter: Arc<Mutex<HashMap<String, u32>>>,
    news_upgrade_total_counter: Arc<Mutex<u32>>,
    /// `source_class=uncertain` 的 NewsCritical 仲裁器。`None` → 跳过 LLM 路径,
    /// 维持 poller 给的 Low(与历史行为兼容)。
    news_classifier: Option<Arc<dyn NewsClassifier>>,
    /// 全局默认重要性 prompt;per-actor `news_importance_prompt = None` 时回落。
    default_importance_prompt: String,
}

impl NotificationRouter {
    pub fn new(
        registry: Arc<SharedRegistry>,
        sink: Arc<dyn OutboundSink>,
        store: Arc<EventStore>,
        digest: Arc<DigestBuffer>,
    ) -> Self {
        Self {
            registry,
            sink,
            store,
            digest,
            polisher: Arc::new(NoopPolisher),
            prefs: Arc::new(AllowAllPrefs),
            high_daily_cap: 0,
            tz_offset_hours: 8,
            same_symbol_cooldown_minutes: 0,
            price_min_direct_pct: 6.0,
            large_position_weight_pct: 20.0,
            macro_immediate_lookahead_hours: 6,
            macro_immediate_grace_hours: 2,
            disabled_kinds: Arc::new(HashSet::new()),
            news_upgrade_per_symbol_per_tick_cap: 0,
            news_upgrade_per_tick_cap: 0,
            news_upgrade_counter: Arc::new(Mutex::new(HashMap::new())),
            news_upgrade_total_counter: Arc::new(Mutex::new(0)),
            news_classifier: None,
            default_importance_prompt: DEFAULT_IMPORTANCE_PROMPT.to_string(),
        }
    }

    pub fn with_polisher(mut self, polisher: Arc<dyn BodyPolisher>) -> Self {
        self.polisher = polisher;
        self
    }

    /// 注入用户偏好源。未注入时默认放行所有事件（维持旧行为）。
    pub fn with_prefs(mut self, prefs: Arc<dyn PrefsProvider>) -> Self {
        self.prefs = prefs;
        self
    }

    /// 每 actor 当日 High 推送上限。0 = 不启用(默认),与历史行为兼容。
    /// 命中上限后同 actor 当日剩余 High 事件自动降级进 digest。
    pub fn with_high_daily_cap(mut self, cap: u32) -> Self {
        self.high_daily_cap = cap;
        self
    }

    /// 配置 tz 偏移,用于计算"当日"窗口起点。默认 8 (北京)。
    pub fn with_tz_offset_hours(mut self, offset: i32) -> Self {
        self.tz_offset_hours = offset;
        self
    }

    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 命中冷却的事件降级到 digest,状态记 "cooled_down"。
    pub fn with_same_symbol_cooldown_minutes(mut self, minutes: u32) -> Self {
        self.same_symbol_cooldown_minutes = minutes;
        self
    }

    pub fn with_price_min_direct_pct(mut self, pct: f64) -> Self {
        self.price_min_direct_pct = pct.max(0.0);
        self
    }

    pub fn with_large_position_weight_pct(mut self, pct: f64) -> Self {
        self.large_position_weight_pct = pct.max(0.0);
        self
    }

    pub fn with_macro_immediate_window(mut self, lookahead_hours: i64, grace_hours: i64) -> Self {
        self.macro_immediate_lookahead_hours = lookahead_hours.max(0);
        self.macro_immediate_grace_hours = grace_hours.max(0);
        self
    }

    /// 部署方 kind 黑名单——命中后 dispatch 直接丢弃,不下发也不入 digest。
    /// 事件仍然入库,便于统计;空列表 = 不启用。
    pub fn with_disabled_kinds<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.disabled_kinds = Arc::new(tags.into_iter().map(|t| t.into()).collect());
        self
    }

    /// 单 tick 内同 symbol 升级次数上限。0 = 不启用,与历史行为兼容。
    /// 命中后,Low NewsCritical 不再被升到 Medium,避免 burst 把 digest
    /// 顶端淹满同一 ticker 的 PR wire 报道。
    pub fn with_news_upgrade_per_symbol_per_tick_cap(mut self, cap: u32) -> Self {
        self.news_upgrade_per_symbol_per_tick_cap = cap;
        self
    }

    /// 单 tick 内所有 ticker 合计升级次数上限。0 = 不启用。
    pub fn with_news_upgrade_per_tick_cap(mut self, cap: u32) -> Self {
        self.news_upgrade_per_tick_cap = cap;
        self
    }

    /// 在每次 poller tick 入口被调用,清零升级计数。生产路径由
    /// `process_events` 在批处理开始时调用一次。
    pub fn reset_tick_counters(&self) {
        if let Ok(mut map) = self.news_upgrade_counter.lock() {
            map.clear();
        }
        if let Ok(mut n) = self.news_upgrade_total_counter.lock() {
            *n = 0;
        }
    }

    /// 注入 LLM-based 不确定来源新闻仲裁器。`None` 时维持 poller 给的 Low。
    pub fn with_news_classifier(mut self, classifier: Arc<dyn NewsClassifier>) -> Self {
        self.news_classifier = Some(classifier);
        self
    }

    /// 全局默认重要性 prompt。per-actor `news_importance_prompt` 缺失时回落到这里。
    pub fn with_default_importance_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.default_importance_prompt = prompt.into();
        self
    }

    /// 新闻多信号合流 + 财报窗口升级:当事件为 `NewsCritical + Low`,且同一 ticker
    /// 在 `[news_ts - 1d, news_ts + 2d]` 窗口内出现过硬信号
    /// (price_alert / earnings_released / earnings_upcoming / sec_filing /
    /// analyst_grade),把 severity 升到 Medium。
    ///
    /// 窗口既覆盖"前 24h 内已发生"的硬信号(#10 多信号合流),也覆盖"未来 48h 内"
    /// 的 earnings_upcoming(#13 财报窗口——因为 earnings_upcoming 的 occurred_at
    /// 是财报当日 00:00,T-1/T 新闻必须向未来扩窗才能命中)。
    ///
    /// 升级是幂等 clone,原事件不被修改。
    fn maybe_upgrade_news(&self, event: &MarketEvent) -> MarketEvent {
        if !matches!(event.kind, EventKind::NewsCritical) || event.severity != Severity::Low {
            return event.clone();
        }
        let mut trigger_tag: Option<String> = None;
        for sym in &event.symbols {
            let recent_start = event.occurred_at - chrono::Duration::hours(6);
            let recent_end = event.occurred_at + chrono::Duration::hours(1);
            match self
                .store
                .symbol_signal_kinds_in_window(sym, recent_start, recent_end)
            {
                Ok(tags) => {
                    if let Some(hit) = tags
                        .iter()
                        .find(|t| NEWS_CONVERGENCE_HARD_SIGNALS.contains(&t.as_str()))
                        .filter(|t| hard_signal_correlates(event, t))
                    {
                        trigger_tag = Some(hit.clone());
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("symbol_signal_kinds_in_window failed for {sym}: {e:#}");
                }
            }
            let earnings_start = event.occurred_at - chrono::Duration::hours(12);
            let earnings_end = event.occurred_at + chrono::Duration::days(2);
            match self
                .store
                .symbol_signal_kinds_in_window(sym, earnings_start, earnings_end)
            {
                Ok(tags) => {
                    if tags.iter().any(|t| t == "earnings_upcoming")
                        && hard_signal_correlates(event, "earnings_upcoming")
                    {
                        trigger_tag = Some("earnings_upcoming".to_string());
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("symbol_signal_kinds_in_window failed for {sym}: {e:#}");
                }
            }
        }
        let Some(tag) = trigger_tag else {
            return event.clone();
        };
        if self.news_upgrade_per_tick_cap > 0 {
            if let Ok(n) = self.news_upgrade_total_counter.lock() {
                if *n >= self.news_upgrade_per_tick_cap {
                    tracing::info!(
                        event_id = %event.id,
                        symbols = ?event.symbols,
                        cap = self.news_upgrade_per_tick_cap,
                        "news upgrade skipped (per-tick cap reached)"
                    );
                    return event.clone();
                }
            }
        }
        // per-symbol per-tick 升级上限:命中后维持 Low,不污染 digest 顶端。
        // 取 event.symbols 中已经升过最多次的那个 symbol 的计数代表本事件;
        // 若任一相关 symbol 都已超过 cap,则跳过升级,但所有相关 symbol 都不再
        // 计数(因为本事件未升级,不应推高计数)。
        if self.news_upgrade_per_symbol_per_tick_cap > 0 {
            if let Ok(map) = self.news_upgrade_counter.lock() {
                let already_capped = event.symbols.iter().any(|sym| {
                    map.get(sym)
                        .copied()
                        .map(|n| n >= self.news_upgrade_per_symbol_per_tick_cap)
                        .unwrap_or(false)
                });
                if already_capped {
                    tracing::info!(
                        event_id = %event.id,
                        symbols = ?event.symbols,
                        cap = self.news_upgrade_per_symbol_per_tick_cap,
                        "news upgrade skipped (per-symbol per-tick cap reached)"
                    );
                    return event.clone();
                }
            }
        }
        // 升级落地:对所有相关 symbol +1。即使某个 symbol 之前 0 次,
        // 这次升级也算它的一次"相关升级"。
        if let Ok(mut map) = self.news_upgrade_counter.lock() {
            for sym in &event.symbols {
                *map.entry(sym.clone()).or_insert(0) += 1;
            }
        }
        if let Ok(mut n) = self.news_upgrade_total_counter.lock() {
            *n += 1;
        }
        let mut upgraded = event.clone();
        upgraded.severity = Severity::Medium;
        tracing::info!(
            event_id = %event.id,
            symbols = ?event.symbols,
            trigger = %tag,
            "news severity upgraded Low→Medium (window convergence)"
        );
        upgraded
    }

    /// 检查该事件是否是"不确定来源 Low NewsCritical",需要 LLM 仲裁器
    /// 介入决定是否升级。返回 `Some(upgraded_event)` 表示 LLM 判 important,
    /// router 应使用升级后的 severity=Medium。返回 `None` 表示无需升级
    /// (源/类型/分类器/LLM 输出 均不满足)。
    async fn maybe_llm_upgrade_for_actor(
        &self,
        event: &MarketEvent,
        prefs: &NotificationPrefs,
    ) -> Option<MarketEvent> {
        // 仅对 NewsCritical / SocialPost 的 Low + uncertain 源走 LLM 路径;其它类型直接跳过。
        // SocialPost 由 Telegram / Truth Social 等社交 poller 产出,payload.source_class
        // 一律写 "uncertain",所以每条帖子都经 LLM 仲裁判是否升 Medium。
        if !matches!(event.kind, EventKind::NewsCritical | EventKind::SocialPost)
            || event.severity != Severity::Low
        {
            return None;
        }
        let source_class = event
            .payload
            .get("source_class")
            .and_then(|v| v.as_str())
            .unwrap_or("uncertain");
        if source_class != "uncertain" {
            return None;
        }
        // 律所模板已被 poller 强制 Low,LLM 也不应再"复活"它。
        let is_legal_ad = event
            .payload
            .get("legal_ad_template")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_legal_ad {
            return None;
        }
        let classifier = self.news_classifier.as_ref()?;
        let prompt = prefs
            .news_importance_prompt
            .as_deref()
            .unwrap_or(&self.default_importance_prompt);
        match classifier.classify(event, prompt).await {
            Some(Importance::Important) => {
                let mut upgraded = event.clone();
                upgraded.severity = Severity::Medium;
                tracing::info!(
                    event_id = %event.id,
                    "uncertain-source news upgraded Low→Medium by LLM classifier"
                );
                Some(upgraded)
            }
            _ => None,
        }
    }

    /// 系统级事件策略：在 per-actor prefs 之前先把明显不该即时推的事件降级。
    /// 这里不丢事件，只调整 severity，让它们进入 digest 或保持低优先级。
    fn apply_system_event_policy(&self, event: &MarketEvent) -> MarketEvent {
        let mut routed = event.clone();
        if is_legal_ad_event(&routed) && routed.severity != Severity::Low {
            routed.severity = Severity::Low;
            tracing::info!(
                event_id = %routed.id,
                source = %routed.source,
                "legal-ad news demoted to Low"
            );
        }
        if matches!(routed.kind, EventKind::MacroEvent) && routed.severity == Severity::High {
            let now = chrono::Utc::now();
            let earliest = now - chrono::Duration::hours(self.macro_immediate_grace_hours);
            let latest = now + chrono::Duration::hours(self.macro_immediate_lookahead_hours);
            if routed.occurred_at < earliest || routed.occurred_at > latest {
                routed.severity = Severity::Medium;
                tracing::info!(
                    event_id = %routed.id,
                    source = %routed.source,
                    occurred_at = %routed.occurred_at,
                    lookahead_hours = self.macro_immediate_lookahead_hours,
                    grace_hours = self.macro_immediate_grace_hours,
                    "macro high demoted to digest outside due window"
                );
            }
        }
        if matches!(routed.kind, EventKind::EarningsUpcoming) {
            let days_until =
                (routed.occurred_at.date_naive() - chrono::Utc::now().date_naive()).num_days();
            if days_until > 7 && routed.severity.rank() > Severity::Low.rank() {
                routed.severity = Severity::Low;
                tracing::info!(
                    event_id = %routed.id,
                    source = %routed.source,
                    days_until,
                    "far earnings preview demoted to Low"
                );
            }
        }
        routed
    }

    /// 按用户 prefs 重写 severity:价格阈值 / immediate_kinds。价格覆盖保留用户
    /// 敏感度，但默认不能低于系统最小直推阈值；大仓位标的可用用户阈值。
    fn apply_per_actor_severity_override(
        &self,
        event: &MarketEvent,
        sev: Severity,
        prefs: &NotificationPrefs,
    ) -> Severity {
        if matches!(sev, Severity::High) {
            return sev;
        }
        if let Some(threshold_pct) = price_override_threshold(event, prefs) {
            if matches!(
                event.kind,
                EventKind::PriceAlert { ref window, .. } if window != "close"
            ) {
                let pct = event
                    .payload
                    .get("changesPercentage")
                    .and_then(|v| v.as_f64());
                if let Some(p) = pct {
                    let min_direct = self.price_min_direct_pct.max(0.0);
                    let large_weight_threshold = prefs
                        .large_position_weight_pct
                        .unwrap_or(self.large_position_weight_pct);
                    let is_large_position = event_position_weight_pct(event)
                        .map(|w| w >= large_weight_threshold)
                        .unwrap_or(false);
                    let required = if is_large_position {
                        threshold_pct
                    } else {
                        threshold_pct.max(min_direct)
                    };
                    if p.abs() >= required {
                        return Severity::High;
                    }
                }
            }
        }
        if let Some(kinds) = prefs.immediate_kinds.as_deref() {
            let tag = kind_tag(&event.kind);
            if kinds.iter().any(|k| k == tag) {
                return Severity::High;
            }
        }
        sev
    }

    fn apply_quiet_mode(
        &self,
        event: &MarketEvent,
        sev: Severity,
        prefs: &NotificationPrefs,
    ) -> Severity {
        if !prefs.quiet_mode || sev != Severity::High {
            return sev;
        }
        if quiet_mode_allows_immediate(event) {
            return sev;
        }
        tracing::info!(
            event_id = %event.id,
            kind = %kind_tag(&event.kind),
            source = %event.source,
            "quiet mode demoted High to digest"
        );
        Severity::Medium
    }

    /// 对一个事件执行分发。High 立即推；其余当前只记 pending-digest 日志。
    ///
    /// 返回 `(immediate_sent, pending_digest)` 数量。
    pub async fn dispatch(&self, event: &MarketEvent) -> anyhow::Result<(u32, u32)> {
        // 全局 kind 黑名单：部署方 YAML 里关掉的 kind 直接短路,不走 resolve/prefs/cap。
        // 事件已经由调用方负责入库,这里只是不分发。
        let tag = kind_tag(&event.kind);
        if self.disabled_kinds.contains(tag) {
            tracing::info!(
                event_id = %event.id,
                kind = %tag,
                "event kind globally disabled; dispatch skipped"
            );
            return Ok((0, 0));
        }
        let upgraded = self.maybe_upgrade_news(event);
        let routed = self.apply_system_event_policy(&upgraded);
        let event = &routed;
        // 每次 dispatch 都拿最新快照——用户持仓更新后下一条事件即可感知。
        let hits = self.registry.load().resolve(event);
        if hits.is_empty() {
            let _ = self.store.log_delivery(
                &event.id,
                "event_engine::::no_actor",
                "router",
                event.severity,
                "no_actor",
                None,
            );
            info!(
                event_id = %event.id,
                kind = %kind_tag(&event.kind),
                source = %event.source,
                symbols = ?event.symbols,
                "dispatch skipped: no matching actor"
            );
            return Ok((0, 0));
        }
        let mut sent = 0u32;
        let mut pending = 0u32;
        for (actor, sev) in hits {
            let user_prefs = self.prefs.load(&actor);
            // LLM 仲裁:不确定来源的 Low NewsCritical,按 actor 重要性 prompt
            // 决定是否升 Medium。结果只影响本 actor 的本次分发,不污染原 event。
            let actor_event_buf;
            let (event, sev) = match self.maybe_llm_upgrade_for_actor(event, &user_prefs).await {
                Some(upgraded) => {
                    actor_event_buf = upgraded;
                    (&actor_event_buf, Severity::Medium)
                }
                None => (event, sev),
            };
            // per-actor severity override:用户可自定义
            //   (a) price_high_pct_override:价格异动绝对值触达即升 High 即时推;
            //   (b) immediate_kinds:某些 kind 无条件升 High 即时推(例如 52 周高/低、
            //       分析师评级)。
            // 升级后仍要走 high_daily_cap / cooldown,保持 burst 防护。
            let sev = self.apply_per_actor_severity_override(event, sev, &user_prefs);
            let sev = self.apply_quiet_mode(event, sev, &user_prefs);
            if !user_prefs.should_deliver(event) {
                let _ = self.store.log_delivery(
                    &event.id,
                    &actor_key(&actor),
                    "prefs",
                    sev,
                    "filtered",
                    None,
                );
                info!(
                    actor = %actor_key(&actor),
                    event_id = %event.id,
                    kind = %kind_tag(&event.kind),
                    source = %event.source,
                    symbols = ?event.symbols,
                    "skipped by user prefs"
                );
                continue;
            }
            // High daily cap:同一 actor 当日 sink-sent High 条数达到上限后,
            // 后续 High 一律降级到 digest,避免"某 ticker 一天连发 8-K + 财报 +
            // 价格异动"把用户淹没。降级路径不双写 log:digest 入队时 status 写
            // "capped" 而不是 "queued",便于复盘统计"今日被降级多少条"。
            // cap=0 关闭该逻辑,与历史行为兼容。
            let mut demoted_by_cap = false;
            let mut demoted_by_cooldown = false;
            let mut effective_sev = if matches!(sev, Severity::High) && self.high_daily_cap > 0 {
                let since = local_day_start(chrono::Utc::now(), self.tz_offset_hours);
                let category = event_category(event);
                match self.store.count_high_sent_since_for_category(
                    &actor_key(&actor),
                    since,
                    category,
                ) {
                    Ok(n) if n >= self.high_daily_cap as i64 => {
                        tracing::info!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            source = %event.source,
                            category = %category,
                            today_high = n,
                            cap = self.high_daily_cap,
                            "High 事件降级进 digest(已超当日上限)"
                        );
                        demoted_by_cap = true;
                        Severity::Medium
                    }
                    Ok(_) => sev,
                    Err(e) => {
                        tracing::warn!("count_high_sent_since failed: {e:#}");
                        sev
                    }
                }
            } else {
                sev
            };
            // 同 ticker 冷却:如果事件还是 High,且 cooldown>0,检查任一 symbol 最近一次
            // High+sink+sent 的时间戳,若在冷却窗口内则降级进 digest。
            if matches!(effective_sev, Severity::High)
                && self.same_symbol_cooldown_minutes > 0
                && !event.symbols.is_empty()
            {
                let cutoff = chrono::Utc::now()
                    - chrono::Duration::minutes(self.same_symbol_cooldown_minutes as i64);
                for sym in &event.symbols {
                    match self.store.last_high_sink_send_for_symbol_category(
                        &actor_key(&actor),
                        sym,
                        event_category(event),
                    ) {
                        Ok(Some(ts)) if ts >= cutoff => {
                            tracing::info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                source = %event.source,
                                symbol = %sym,
                                last_sent_at = %ts,
                                cooldown_min = self.same_symbol_cooldown_minutes,
                                "High 事件降级进 digest(同 ticker 冷却中)"
                            );
                            demoted_by_cooldown = true;
                            effective_sev = Severity::Medium;
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!(
                                "last_high_sink_send_for_symbol failed for {sym}: {e:#}"
                            );
                        }
                    }
                }
            }
            match effective_sev {
                Severity::High => {
                    let fmt = self.sink.format_for(&actor);
                    let default_body = renderer::render_immediate(event, fmt);
                    let body = if matches!(fmt, RenderFormat::Plain) {
                        match self.polisher.polish(event, &default_body).await {
                            Some(polished) => polished,
                            None => default_body,
                        }
                    } else {
                        default_body
                    };
                    if let Err(e) = self.sink.send(&actor, &body).await {
                        tracing::warn!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            kind = %kind_tag(&event.kind),
                            source = %event.source,
                            symbols = ?event.symbols,
                            body_len = body.chars().count(),
                            body_preview = %body_preview(&body),
                            "sink send failed: {e:#}"
                        );
                        let _ = self.store.log_delivery(
                            &event.id,
                            &actor_key(&actor),
                            "sink",
                            sev,
                            "failed",
                            Some(&body),
                        );
                        continue;
                    }
                    let success_status = self.sink.success_status();
                    let _ = self.store.log_delivery(
                        &event.id,
                        &actor_key(&actor),
                        "sink",
                        sev,
                        success_status,
                        Some(&body),
                    );
                    tracing::info!(
                        actor = %actor_key(&actor),
                        event_id = %event.id,
                        kind = %kind_tag(&event.kind),
                        source = %event.source,
                        symbols = ?event.symbols,
                        severity = ?sev,
                        status = %success_status,
                        body_len = body.chars().count(),
                        body_preview = %body_preview(&body),
                        "sink delivered"
                    );
                    sent += 1;
                }
                Severity::Medium | Severity::Low => {
                    match self.digest.enqueue(&actor, event) {
                        Ok(()) => {
                            // 被 cap 降级的条目记 status="capped",被同 ticker 冷却降级的
                            // 记 "cooled_down",正常流程记 "queued"。severity 仍记原始严重度
                            // (sev),方便事后 grep "high + capped/cooled_down" 对账。
                            let status = if demoted_by_cap {
                                "capped"
                            } else if demoted_by_cooldown {
                                "cooled_down"
                            } else {
                                "queued"
                            };
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                status,
                                None,
                            );
                            info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                kind = %kind_tag(&event.kind),
                                source = %event.source,
                                symbols = ?event.symbols,
                                severity = ?sev,
                                status = %status,
                                "digest queued"
                            );
                            pending += 1;
                        }
                        Err(e) => {
                            tracing::warn!("digest enqueue failed: {e:#}");
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                "failed",
                                None,
                            );
                        }
                    }
                }
            }
        }
        Ok((sent, pending))
    }
}

fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

/// 取 body 头 120 字符做 tracing 预览,换行折成单行 ⏎,避免日志多行难抓。
/// 全文一律已经在 SQLite `delivery_log` 里,这里只是肉眼速读用。
pub(crate) fn body_preview(body: &str) -> String {
    let mut s: String = body.chars().take(120).collect();
    if body.chars().count() > 120 {
        s.push('…');
    }
    s.replace('\n', " ⏎ ")
}

fn price_override_threshold(event: &MarketEvent, prefs: &NotificationPrefs) -> Option<f64> {
    let pct = event
        .payload
        .get("changesPercentage")
        .and_then(|v| v.as_f64());
    match pct {
        Some(p) if p >= 0.0 => prefs
            .price_high_pct_up_override
            .or(prefs.price_high_pct_override),
        Some(_) => prefs
            .price_high_pct_down_override
            .or(prefs.price_high_pct_override),
        None => prefs.price_high_pct_override,
    }
}

fn event_position_weight_pct(event: &MarketEvent) -> Option<f64> {
    let raw = event
        .payload
        .get("portfolio_weight_pct")
        .or_else(|| event.payload.get("portfolio_weight"))
        .and_then(|v| v.as_f64())?;
    Some(if raw <= 1.0 { raw * 100.0 } else { raw })
}

fn quiet_mode_allows_immediate(event: &MarketEvent) -> bool {
    match &event.kind {
        EventKind::EarningsReleased
        | EventKind::EarningsCallTranscript
        | EventKind::SecFiling { .. } => true,
        EventKind::PriceAlert { window, .. } if window != "close" => true,
        _ => false,
    }
}

fn is_legal_ad_event(event: &MarketEvent) -> bool {
    event
        .payload
        .get("legal_ad_template")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || {
            let text = format!("{} {}", event.title, event.summary).to_ascii_lowercase();
            [
                "shareholder alert",
                "investor alert",
                "class action lawsuit",
                "securities fraud class action",
                "law offices",
                "law firm",
            ]
            .iter()
            .any(|pat| text.contains(pat))
        }
}

fn hard_signal_correlates(event: &MarketEvent, tag: &str) -> bool {
    let text = format!("{} {}", event.title, event.summary).to_ascii_lowercase();
    let any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    match tag {
        "price_alert" => any(&[
            "price", "stock", "share", "shares", "surge", "jump", "rally", "fall", "drop", "slump",
            "plunge",
        ]),
        "earnings_released" | "earnings_upcoming" | "earnings_call_transcript" => any(&[
            "earnings",
            "results",
            "revenue",
            "profit",
            "eps",
            "guidance",
            "quarter",
            "transcript",
        ]),
        "sec_filing" => any(&["sec", "filing", "8-k", "10-k", "10-q", "investigation"]),
        "analyst_grade" => any(&["analyst", "upgrade", "downgrade", "price target", "rating"]),
        _ => true,
    }
}

fn event_category(event: &MarketEvent) -> &'static str {
    match event.kind {
        EventKind::PriceAlert { .. }
        | EventKind::Weekly52High
        | EventKind::Weekly52Low
        | EventKind::VolumeSpike => "price",
        EventKind::NewsCritical | EventKind::PressRelease | EventKind::SocialPost => "news",
        EventKind::SecFiling { .. } => "filing",
        EventKind::EarningsUpcoming
        | EventKind::EarningsReleased
        | EventKind::EarningsCallTranscript => "earnings",
        EventKind::MacroEvent => "macro",
        EventKind::Dividend | EventKind::Split | EventKind::Buyback => "corp_action",
        EventKind::AnalystGrade => "analyst",
        EventKind::PortfolioPreMarket | EventKind::PortfolioPostMarket => "portfolio",
    }
}

/// 按给定 tz 偏移求本地当日 00:00 对应的 UTC 时刻。用作
/// `count_high_sent_since` 的 cutoff,保证跨时区一致。
fn local_day_start(
    now: chrono::DateTime<chrono::Utc>,
    offset_hours: i32,
) -> chrono::DateTime<chrono::Utc> {
    use chrono::{FixedOffset, NaiveTime, TimeZone};
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    offset
        .from_local_datetime(&midnight)
        .single()
        .map(|l| l.with_timezone(&chrono::Utc))
        .unwrap_or(now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::subscription::{PortfolioSubscription, SubscriptionRegistry};
    use chrono::Utc;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct CapturingSink {
        calls: Mutex<Vec<(String, String)>>,
    }

    #[async_trait]
    impl OutboundSink for CapturingSink {
        async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((actor_key(actor), body.to_string()));
            Ok(())
        }
    }

    fn actor(user: &str) -> ActorIdentity {
        ActorIdentity::new("imessage", user, None::<&str>).unwrap()
    }

    fn ev(sev: Severity) -> MarketEvent {
        MarketEvent {
            id: "e1".into(),
            kind: EventKind::EarningsReleased,
            severity: sev,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "earnings".into(),
            summary: "beat".into(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    fn router_with_aapl_actor() -> (NotificationRouter, Arc<CapturingSink>, tempfile::TempDir) {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        (
            NotificationRouter::new(
                Arc::new(SharedRegistry::from_registry(reg)),
                sink.clone(),
                store,
                digest,
            ),
            sink,
            dir,
        )
    }

    #[tokio::test]
    async fn high_severity_goes_to_sink_immediately() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let (sent, pending) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
        assert_eq!(pending, 0);
        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].1.contains("财报发布"));
    }

    #[tokio::test]
    async fn high_daily_cap_demotes_excess_to_digest() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_high_daily_cap(2);

        // 每条 High 事件用不同 id 避免被上层去重逻辑误判同一事件
        let mk = |id: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("earnings {id}"),
            summary: "beat".into(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };

        let h1 = mk("h1");
        let h2 = mk("h2");
        let h3 = mk("h3");
        store.insert_event(&h1).unwrap();
        store.insert_event(&h2).unwrap();
        store.insert_event(&h3).unwrap();
        let (s1, _) = router.dispatch(&h1).await.unwrap();
        let (s2, _) = router.dispatch(&h2).await.unwrap();
        // 前两条正常走 sink
        assert_eq!(s1, 1);
        assert_eq!(s2, 1);
        assert_eq!(sink.calls.lock().unwrap().len(), 2);

        // 第三条触顶 → 降级到 digest,sink 不再收到,pending=1
        let (s3, p3) = router.dispatch(&h3).await.unwrap();
        assert_eq!(s3, 0, "触顶后 High 不应走 sink");
        assert_eq!(p3, 1, "应降级进 digest");
        assert_eq!(
            sink.calls.lock().unwrap().len(),
            2,
            "sink call count 不应增加"
        );

        // delivery_log 里应有 2 条 sent + 1 条 capped
        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(
            store
                .count_high_sent_since("imessage::::u1", since)
                .unwrap(),
            2
        );
    }

    #[tokio::test]
    async fn high_daily_cap_zero_means_no_cap() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        // cap = 0 应该关闭所有限流,N 条 High 全部进 sink
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_high_daily_cap(0);

        for i in 0..5 {
            let mut event = ev(Severity::High);
            event.id = format!("h{i}");
            let (s, _) = router.dispatch(&event).await.unwrap();
            assert_eq!(s, 1, "cap=0 时每条 High 都应走 sink");
        }
        assert_eq!(sink.calls.lock().unwrap().len(), 5);
    }

    #[tokio::test]
    async fn same_symbol_cooldown_demotes_second_high_to_digest() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_same_symbol_cooldown_minutes(60);

        let mk = |id: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("earnings {id}"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        // 第一条必须先入 events 表,这样 JOIN 才能找到 symbol;生产路径由 poller 完成入库。
        let a = mk("h1");
        store.insert_event(&a).unwrap();
        let (s1, _) = router.dispatch(&a).await.unwrap();
        assert_eq!(s1, 1, "第一条 AAPL High 应走 sink");

        let b = mk("h2");
        store.insert_event(&b).unwrap();
        let (s2, p2) = router.dispatch(&b).await.unwrap();
        assert_eq!(s2, 0, "60min 冷却内第二条应降级");
        assert_eq!(p2, 1);

        // 不同 ticker 不受冷却影响
        let mut c = mk("h3");
        c.symbols = vec!["NVDA".into()];
        // NVDA 未在订阅里,应无命中 → 0 sent, 0 pending
        store.insert_event(&c).unwrap();
        let (s3, p3) = router.dispatch(&c).await.unwrap();
        assert_eq!(s3 + p3, 0, "未订阅 NVDA,不应 dispatch");
    }

    #[tokio::test]
    async fn cooldown_zero_means_no_throttle() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_same_symbol_cooldown_minutes(0);

        for i in 0..3 {
            let mut e = ev(Severity::High);
            e.id = format!("h{i}");
            store.insert_event(&e).unwrap();
            let (s, _) = router.dispatch(&e).await.unwrap();
            assert_eq!(s, 1, "cooldown=0 时不应降级");
        }
        assert_eq!(sink.calls.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn medium_and_low_are_deferred_to_digest() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let (sent_m, pending_m) = router.dispatch(&ev(Severity::Medium)).await.unwrap();
        let (sent_l, pending_l) = router.dispatch(&ev(Severity::Low)).await.unwrap();
        assert_eq!(sent_m + sent_l, 0);
        assert_eq!(pending_m + pending_l, 2);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn polisher_body_overrides_default_template() {
        use crate::polisher::BodyPolisher;

        struct FixedPolisher;
        #[async_trait]
        impl BodyPolisher for FixedPolisher {
            async fn polish(&self, _e: &MarketEvent, _b: &str) -> Option<String> {
                Some("POLISHED BODY".into())
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_polisher(Arc::new(FixedPolisher));

        let _ = router.dispatch(&ev(Severity::High)).await.unwrap();
        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls[0].1, "POLISHED BODY");
    }

    #[tokio::test]
    async fn disabled_prefs_skip_send_and_enqueue() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        let (sent_h, pending_h) = router.dispatch(&ev(Severity::High)).await.unwrap();
        let (sent_m, pending_m) = router.dispatch(&ev(Severity::Medium)).await.unwrap();
        assert_eq!(sent_h + sent_m, 0);
        assert_eq!(pending_h + pending_m, 0);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn portfolio_only_prefs_drop_symbolless_events() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        // 强行命中：用 GlobalSubscription-like 兜底——直接用一个命中所有事件的 Subscription。
        // 这里简化为 dispatch MacroEvent 并注入 GlobalSubscription。
        struct AlwaysMatch(ActorIdentity);
        impl crate::subscription::Subscription for AlwaysMatch {
            fn id(&self) -> &str {
                "always"
            }
            fn matches(&self, _e: &MarketEvent) -> bool {
                true
            }
            fn actors(&self) -> Vec<ActorIdentity> {
                vec![self.0.clone()]
            }
        }
        reg.register(Box::new(AlwaysMatch(actor("u1"))));

        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        use crate::prefs::PrefsProvider;
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    portfolio_only: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        // 无 symbol 的 macro 事件应被过滤
        let mut macro_ev = ev(Severity::High);
        macro_ev.kind = crate::event::EventKind::MacroEvent;
        macro_ev.symbols.clear();
        let (sent, _pending) = router.dispatch(&macro_ev).await.unwrap();
        assert_eq!(sent, 0);
        assert!(sink.calls.lock().unwrap().is_empty());

        // 命中 symbol 的事件仍应送达
        let (sent, _pending) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
    }

    #[tokio::test]
    async fn macro_high_is_digest_until_due_window_then_immediate() {
        struct AlwaysMatch(ActorIdentity);
        impl crate::subscription::Subscription for AlwaysMatch {
            fn id(&self) -> &str {
                "always"
            }
            fn matches(&self, _e: &MarketEvent) -> bool {
                true
            }
            fn actors(&self) -> Vec<ActorIdentity> {
                vec![self.0.clone()]
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(AlwaysMatch(actor("u1"))));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_macro_immediate_window(6, 2);

        let mut future_macro = ev(Severity::High);
        future_macro.id = "macro:future:cpi".into();
        future_macro.kind = EventKind::MacroEvent;
        future_macro.symbols.clear();
        future_macro.occurred_at = Utc::now() + chrono::Duration::days(3);
        future_macro.title = "[US] CPI YoY".into();
        future_macro.source = "fmp.economic_calendar".into();
        let (sent, pending) = router.dispatch(&future_macro).await.unwrap();
        assert_eq!(sent, 0, "未来 7 天日历不应即时推");
        assert_eq!(pending, 1);

        let mut near_macro = future_macro.clone();
        near_macro.id = "macro:near:cpi".into();
        near_macro.occurred_at = Utc::now() + chrono::Duration::hours(2);
        let (sent, pending) = router.dispatch(&near_macro).await.unwrap();
        assert_eq!(sent, 1, "临近发生窗口内的 high macro 才即时推");
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn far_earnings_preview_is_low_priority_digest() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest.clone(),
        );
        let mut event = ev(Severity::Medium);
        event.id = "earnings:AAPL:far".into();
        event.kind = EventKind::EarningsUpcoming;
        event.occurred_at = Utc::now() + chrono::Duration::days(10);
        let (sent, pending) = router.dispatch(&event).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
        let drained = digest.drain_actor(&actor("u1")).unwrap();
        assert_eq!(drained[0].severity, Severity::Low);
    }

    #[tokio::test]
    async fn legal_ad_high_is_demoted_before_sink() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["SNOW".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        );
        let event = MarketEvent {
            id: "news:SNOW:legal-high".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::High,
            symbols: vec!["SNOW".into()],
            occurred_at: Utc::now(),
            title: "SHAREHOLDER ALERT class action lawsuit has been filed".into(),
            summary: String::new(),
            url: None,
            source: "fmp.stock_news:globenewswire.com".into(),
            payload: serde_json::json!({"legal_ad_template": true}),
        };
        let (sent, pending) = router.dispatch(&event).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "法律广告即使误标 High 也应进 digest");
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn low_news_upgrades_to_medium_when_same_day_hard_signal_exists() {
        // 构造一条今日 AAPL 的 price_alert 先入 store,再 dispatch 一条 Low NewsCritical,
        // 应升级为 Medium 并进 digest(sent=0, pending=1)。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        // 先落一条硬信号
        let hard = MarketEvent {
            id: "price:AAPL:today".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 700,
                window: "day".into(),
            },
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL +7%".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&hard).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        );

        let news = MarketEvent {
            id: "news:AAPL:1".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL stock jumps after price spike".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "Low 新闻应被升到 Medium 后入 digest");
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn low_news_upgrades_inside_earnings_window() {
        // earnings_upcoming 的 occurred_at 是未来的财报日 00:00;今天的 Low 新闻
        // 应命中 [news - 1d, news + 2d] 窗口被升到 Medium。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        let now = Utc::now();
        let earnings = MarketEvent {
            id: "earnings:AAPL:tomorrow".into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: now + chrono::Duration::days(1),
            title: "AAPL earnings tomorrow".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&earnings).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        );

        let news = MarketEvent {
            id: "news:AAPL:prewindow".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: now,
            title: "AAPL preview".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "财报窗口内 Low 新闻应升到 Medium 入 digest");
    }

    #[tokio::test]
    async fn low_news_stays_low_without_same_day_signal() {
        // 无硬信号时 Low 新闻维持 Low,仍然入 digest(pending=1),但 severity 未升。
        // 间接校验:digest enqueue 行为不变,且未发生 sink 立即推。
        let (router, sink, _tmp) = router_with_aapl_actor();
        let news = MarketEvent {
            id: "news:AAPL:2".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL minor headline".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn globally_disabled_kind_is_dropped_before_prefs() {
        // 部署方把 press_release 放入全局黑名单。即便订阅命中,dispatch 也应
        // 返回 (0, 0),既不 sink 也不 enqueue,且 delivery_log 无记录。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_disabled_kinds(["press_release"]);

        let pr = MarketEvent {
            id: "pr:AAPL:1".into(),
            kind: EventKind::PressRelease,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL announces".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&pr).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 0);
        assert!(sink.calls.lock().unwrap().is_empty());

        // 非黑名单 kind 不受影响
        let (sent, _) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
    }

    /// e2e:对 uncertain 源 NewsCritical Low,注入 LLM 仲裁器返回 Important
    /// → router 升 Medium → 走 digest 而非 sink immediate。
    #[tokio::test]
    async fn llm_classifier_upgrades_uncertain_news_to_medium_for_actor() {
        use crate::news_classifier::{Importance, NewsClassifier};

        struct YesClassifier;
        #[async_trait]
        impl NewsClassifier for YesClassifier {
            async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
                Some(Importance::Important)
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["ACME".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_classifier(Arc::new(YesClassifier));

        // 模拟 poller 给的 uncertain Low NewsCritical
        let news = MarketEvent {
            id: "news:ACME:1".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["ACME".into()],
            occurred_at: Utc::now(),
            title: "ACME announces breakthrough".into(),
            summary: "ACME pioneers something".into(),
            url: None,
            source: "fmp.stock_news:smallblog.io".into(),
            payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        // 升 Medium 后走 digest,immediate sink 仍为 0
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "LLM 升级后应进 digest");
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    /// e2e:LLM 返回 NotImportant 时,uncertain 源新闻保持 Low,正常进 digest。
    #[tokio::test]
    async fn llm_classifier_keeps_low_when_not_important() {
        use crate::news_classifier::{Importance, NewsClassifier};

        struct NoClassifier;
        #[async_trait]
        impl NewsClassifier for NoClassifier {
            async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
                Some(Importance::NotImportant)
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["ACME".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_classifier(Arc::new(NoClassifier));

        let news = MarketEvent {
            id: "news:ACME:2".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["ACME".into()],
            occurred_at: Utc::now(),
            title: "ACME mundane news".into(),
            summary: "ACME has a meeting".into(),
            url: None,
            source: "fmp.stock_news:smallblog.io".into(),
            payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    /// e2e:trusted 源 News 不走 LLM(LLM 即便返回 Important 也不应触发,
    /// 因为前置守卫只放过 source_class=uncertain)。
    #[tokio::test]
    async fn llm_classifier_skipped_for_trusted_source() {
        use crate::news_classifier::{Importance, NewsClassifier};
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingClassifier(Arc<AtomicUsize>);
        #[async_trait]
        impl NewsClassifier for CountingClassifier {
            async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Some(Importance::Important)
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let counter = Arc::new(AtomicUsize::new(0));
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_classifier(Arc::new(CountingClassifier(counter.clone())));

        let news = MarketEvent {
            id: "news:AAPL:trust".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL news".into(),
            summary: "ok".into(),
            url: None,
            source: "fmp.stock_news:reuters.com".into(),
            payload: serde_json::json!({"source_class": "trusted", "legal_ad_template": false}),
        };
        let (_sent, _pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "trusted source 不应触发 LLM"
        );
    }

    /// e2e:即使 LLM 说 important,律所模板标题(legal_ad_template=true)
    /// 也保持 Low,不被 LLM 复活。
    #[tokio::test]
    async fn llm_classifier_does_not_resurrect_legal_ad_templates() {
        use crate::news_classifier::{Importance, NewsClassifier};

        struct YesClassifier;
        #[async_trait]
        impl NewsClassifier for YesClassifier {
            async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
                Some(Importance::Important)
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["SNOW".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_classifier(Arc::new(YesClassifier));

        let news = MarketEvent {
            id: "news:SNOW:legal".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["SNOW".into()],
            occurred_at: Utc::now(),
            title: "SHAREHOLDER ALERT class action lawsuit has been filed".into(),
            summary: "...".into(),
            url: None,
            source: "fmp.stock_news:globenewswire.com".into(),
            payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": true}),
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        // 不应升 Medium —— 仍按原 Low 走 digest
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
    }

    /// e2e:per-actor news_importance_prompt 覆盖全局默认,LLM 收到 actor 的版本。
    #[tokio::test]
    async fn per_actor_importance_prompt_overrides_default() {
        use crate::news_classifier::{Importance, NewsClassifier};
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        // 记录 LLM 收到的 prompt;断言取到的是 actor 的覆盖版而不是全局默认。
        struct RecordingClassifier(Arc<Mutex<Vec<String>>>);
        #[async_trait]
        impl NewsClassifier for RecordingClassifier {
            async fn classify(&self, _e: &MarketEvent, p: &str) -> Option<Importance> {
                self.0.lock().unwrap().push(p.to_string());
                Some(Importance::NotImportant)
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["ACME".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    news_importance_prompt: Some("仅与 SaaS 行业并购相关".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store)
        .with_default_importance_prompt("全局默认 prompt")
        .with_news_classifier(Arc::new(RecordingClassifier(captured.clone())));

        let news = MarketEvent {
            id: "news:ACME:per-actor".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["ACME".into()],
            occurred_at: Utc::now(),
            title: "ACME bulletin".into(),
            summary: "...".into(),
            url: None,
            source: "fmp.stock_news:smallblog.io".into(),
            payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
        };
        router.dispatch(&news).await.unwrap();
        let prompts = captured.lock().unwrap().clone();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0], "仅与 SaaS 行业并购相关");
    }

    #[tokio::test]
    async fn news_upgrade_per_symbol_cap_limits_burst_within_tick() {
        // 同一 ticker 在单 tick 内最多升级 N 条;超出的 Low NewsCritical 维持 Low,
        // 不进 digest 顶端。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        // 先落一条硬信号,使 maybe_upgrade_news 满足窗口条件
        let hard = MarketEvent {
            id: "earnings:AAPL:tomorrow".into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now() + chrono::Duration::days(1),
            title: "AAPL earnings tomorrow".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&hard).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_upgrade_per_symbol_per_tick_cap(2);

        // 模拟一个 tick 入口
        router.reset_tick_counters();

        let mk = |id: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("AAPL earnings preview {id}"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };

        // 前 2 条命中升级 → Medium → digest pending=1
        let (s1, p1) = router.dispatch(&mk("n1")).await.unwrap();
        let (s2, p2) = router.dispatch(&mk("n2")).await.unwrap();
        assert_eq!(s1 + s2, 0);
        assert_eq!(p1 + p2, 2);

        // 第 3 条触顶 → 维持 Low → 仍入 digest(pending=1),但 severity 没升
        let (s3, p3) = router.dispatch(&mk("n3")).await.unwrap();
        assert_eq!(s3, 0);
        assert_eq!(p3, 1);

        // reset 后下一 tick 重新计数
        router.reset_tick_counters();
        let (s4, p4) = router.dispatch(&mk("n4")).await.unwrap();
        assert_eq!(s4, 0);
        assert_eq!(p4, 1);
    }

    #[tokio::test]
    async fn news_upgrade_cap_zero_means_unlimited() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        let hard = MarketEvent {
            id: "earnings:AAPL:tomorrow".into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now() + chrono::Duration::days(1),
            title: "AAPL earnings tomorrow".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&hard).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_news_upgrade_per_symbol_per_tick_cap(0);

        for i in 0..6 {
            let news = MarketEvent {
                id: format!("n{i}"),
                kind: EventKind::NewsCritical,
                severity: Severity::Low,
                symbols: vec!["AAPL".into()],
                occurred_at: Utc::now(),
                title: format!("AAPL earnings preview {i}"),
                summary: String::new(),
                url: None,
                source: "test".into(),
                payload: serde_json::Value::Null,
            };
            let (_s, p) = router.dispatch(&news).await.unwrap();
            assert_eq!(p, 1);
        }
    }

    #[tokio::test]
    async fn news_upgrade_per_tick_cap_limits_cross_symbol_burst() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into(), "AMD".into(), "GEV".into(), "MU".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let now = Utc::now();
        for sym in ["AAPL", "AMD", "GEV", "MU"] {
            let hard = MarketEvent {
                id: format!("earnings:{sym}:tomorrow"),
                kind: EventKind::EarningsUpcoming,
                severity: Severity::Medium,
                symbols: vec![sym.into()],
                occurred_at: now + chrono::Duration::days(1),
                title: format!("{sym} earnings tomorrow"),
                summary: String::new(),
                url: None,
                source: "test".into(),
                payload: serde_json::Value::Null,
            };
            store.insert_event(&hard).unwrap();
        }

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink,
            store,
            digest.clone(),
        )
        .with_news_upgrade_per_symbol_per_tick_cap(3)
        .with_news_upgrade_per_tick_cap(2);
        router.reset_tick_counters();

        for sym in ["AAPL", "AMD", "GEV", "MU"] {
            let news = MarketEvent {
                id: format!("news:{sym}:1"),
                kind: EventKind::NewsCritical,
                severity: Severity::Low,
                symbols: vec![sym.into()],
                occurred_at: now,
                title: format!("{sym} earnings preview"),
                summary: String::new(),
                url: None,
                source: "test".into(),
                payload: serde_json::Value::Null,
            };
            let (_s, p) = router.dispatch(&news).await.unwrap();
            assert_eq!(p, 1);
        }

        let drained = digest.drain_actor(&actor("u1")).unwrap();
        let upgraded = drained
            .iter()
            .filter(|e| e.severity == Severity::Medium)
            .count();
        assert_eq!(upgraded, 2, "per-tick cap should limit total upgrades");
        assert_eq!(drained.len(), 4);
    }

    #[tokio::test]
    async fn per_actor_price_threshold_below_system_floor_stays_digest() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAOI".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    price_high_pct_override: Some(3.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        // 4% 价格异动:用户 override=3% 只表达"关注",但低于系统 6% 直推地板,
        // 应进入 digest 而不是即时打扰。
        let ev = MarketEvent {
            id: "price:AAOI:test".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 400,
                window: "day".into(),
            },
            severity: Severity::Low,
            symbols: vec!["AAOI".into()],
            occurred_at: Utc::now(),
            title: "AAOI +4.00%".into(),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"changesPercentage": 4.05}),
        };
        let (sent, pending) = router.dispatch(&ev).await.unwrap();
        assert_eq!(sent, 0, "低于系统直推地板不应即时推");
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn large_position_can_use_sensitive_price_threshold() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAOI".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    price_high_pct_override: Some(4.0),
                    large_position_weight_pct: Some(20.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store)
        .with_price_min_direct_pct(6.0);

        let ev = MarketEvent {
            id: "price:AAOI:large".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 450,
                window: "day".into(),
            },
            severity: Severity::Low,
            symbols: vec!["AAOI".into()],
            occurred_at: Utc::now(),
            title: "AAOI +4.50%".into(),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({
                "changesPercentage": 4.5,
                "portfolio_weight_pct": 25.0
            }),
        };
        let (sent, pending) = router.dispatch(&ev).await.unwrap();
        assert_eq!(sent, 1, "大仓位标的可使用用户敏感阈值直推");
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn directional_price_thresholds_use_move_direction() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    price_high_pct_up_override: Some(6.0),
                    price_high_pct_down_override: Some(5.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store)
        .with_price_min_direct_pct(5.0);

        let mk = |id: &str, pct: f64| MarketEvent {
            id: id.into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: (pct * 100.0) as i64,
                window: "day".into(),
            },
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("AAPL {pct:+.2}%"),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"changesPercentage": pct}),
        };
        let (sent_up, pending_up) = router.dispatch(&mk("price:up", 5.5)).await.unwrap();
        assert_eq!(sent_up, 0, "+5.5% 未达到上行 6% 阈值");
        assert_eq!(pending_up, 1);

        let (sent_down, pending_down) = router.dispatch(&mk("price:down", -5.5)).await.unwrap();
        assert_eq!(sent_down, 1, "-5.5% 达到下行 5% 阈值");
        assert_eq!(pending_down, 0);
    }

    #[tokio::test]
    async fn per_actor_price_threshold_does_not_promote_closing_move() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AMD".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    price_high_pct_override: Some(4.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        let ev = MarketEvent {
            id: "price_close:AMD:2026-04-22".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 667,
                window: "close".into(),
            },
            severity: Severity::Medium,
            symbols: vec!["AMD".into()],
            occurred_at: Utc::now(),
            title: "AMD +6.67%".into(),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"changesPercentage": 6.67}),
        };
        let (sent, pending) = router.dispatch(&ev).await.unwrap();
        assert_eq!(sent, 0, "收盘异动不应被个人 price override 直推");
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn per_actor_immediate_kinds_promotes_weekly52_high() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAOI".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    immediate_kinds: Some(vec!["weekly52_high".into()]),
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        let ev = MarketEvent {
            id: "52h:AAOI:test".into(),
            kind: EventKind::Weekly52High,
            severity: Severity::Medium,
            symbols: vec!["AAOI".into()],
            occurred_at: Utc::now(),
            title: "AAOI 触及 52 周新高".into(),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&ev).await.unwrap();
        assert_eq!(sent, 1, "immediate_kinds 命中 weekly52_high 应即时推");
        assert_eq!(pending, 0);

        // NewsCritical Low 不在列表 → 仍走 digest。
        let news = MarketEvent {
            id: "news:AAOI:1".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAOI".into()],
            occurred_at: Utc::now(),
            title: "AAOI 普通新闻".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent2, pending2) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent2, 0, "未在 immediate_kinds 列表的 kind 不应被升");
        assert_eq!(pending2, 1);
    }

    #[tokio::test]
    async fn quiet_mode_demotes_news_but_keeps_sec_immediate() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    quiet_mode: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_prefs(prefs_store);

        let mut news = ev(Severity::High);
        news.id = "news:AAPL:quiet".into();
        news.kind = EventKind::NewsCritical;
        news.title = "AAPL high news".into();
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "quiet mode 下新闻 High 应进 digest");

        let mut filing = ev(Severity::High);
        filing.id = "sec:AAPL:8k".into();
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        let (sent, pending) = router.dispatch(&filing).await.unwrap();
        assert_eq!(sent, 1, "SEC filing 仍应即时推");
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn dryrun_sink_success_is_not_counted_as_sent_ack() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            Arc::new(LogSink),
            store.clone(),
            digest,
        );
        let event = ev(Severity::High);
        store.insert_event(&event).unwrap();
        let (sent, pending) = router.dispatch(&event).await.unwrap();
        assert_eq!(sent, 1, "dispatch 计数代表 sink 调用成功");
        assert_eq!(pending, 0);
        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(
            store
                .count_high_sent_since("imessage::::u1", since)
                .unwrap(),
            0,
            "dryrun status 不应被 count_high_sent_since 当成真实 sent"
        );
    }

    #[tokio::test]
    async fn per_actor_overrides_default_off_keeps_legacy_behavior() {
        // 不设 prefs override 时,Low PriceAlert 与 Medium Weekly52High 仍走 digest。
        let (router, sink, _tmp) = router_with_aapl_actor();
        let price_low = MarketEvent {
            id: "price:AAPL:legacy".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 400,
                window: "day".into(),
            },
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL +4%".into(),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"changesPercentage": 4.0}),
        };
        let (sent, pending) = router.dispatch(&price_low).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn event_without_subscribers_is_no_op() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let mut e = ev(Severity::High);
        e.symbols = vec!["TSLA".into()]; // 无人持仓
        let (sent, pending) = router.dispatch(&e).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 0);
        assert!(sink.calls.lock().unwrap().is_empty());
    }
}
