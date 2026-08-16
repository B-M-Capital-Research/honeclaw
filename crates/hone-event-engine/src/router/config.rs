//! `NotificationRouter` 结构体定义 + `new` + 链式 `with_*` builder + per-tick
//! 状态的清零/快照接口。
//!
//! 这里**只**承担「装/读配置」的职责;真正的事件分发 / 升级仲裁 / 策略覆盖
//! 分散在 sibling 文件里(`dispatch.rs` / `classify.rs` / `policy.rs`)。
//!
//! 字段一律 `pub(super)` —— sibling module 的方法实现要直接读这些常量,
//! 没必要写一堆 getter。`pub` 类型(`NotificationRouter`)的字段对外仍然
//! 只能通过 `new` + 链式 builder 配置,跨 crate 访问拿不到。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use hone_core::ActorIdentity;

use crate::digest::DigestBuffer;
use crate::earnings_continuity::EarningsContinuityReconciler;
use crate::event::{MarketEvent, Severity};
use crate::news_classifier::{DEFAULT_IMPORTANCE_PROMPT, NewsClassifier};
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::prefs::{AllowAllPrefs, PrefsProvider, PriceAlertPolicyDefaults};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;

use super::sink::OutboundSink;
use super::stats::NewsUpgradeTickStats;

pub struct NotificationRouter {
    pub(super) registry: Arc<SharedRegistry>,
    pub(super) sink: Arc<dyn OutboundSink>,
    pub(super) store: Arc<EventStore>,
    pub(super) digest: Arc<DigestBuffer>,
    pub(super) polisher: Arc<dyn BodyPolisher>,
    pub(super) prefs: Arc<dyn PrefsProvider>,
    /// A 级公司财报送达后的异步研究账本对账。不得阻塞 T0 推送。
    pub(super) earnings_continuity: Option<Arc<dyn EarningsContinuityReconciler>>,
    /// 每 actor 当日 sink=sent 且 severity=high 的条数上限。超过后新的 High
    /// 事件自动降级进 digest,并在 delivery_log 写 status="capped"。
    /// 0 = 不启用。
    pub(super) high_daily_cap: u32,
    /// 解释“当日”和 quiet-hours 所用的运行时时区。
    pub(super) runtime_timezone: hone_core::RuntimeTimezone,
    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 防止同一 ticker 短时间内被价格异动 + 新闻 + SEC filing 三连推。
    /// 命中后降级到 digest,log_delivery 写 status="cooled_down"。
    pub(super) same_symbol_cooldown_minutes: u32,
    /// 价格候选档、首次直推阈值、重复步长和大仓位边界的系统默认值。
    /// actor 偏好通过 `NotificationPrefs::effective_price_alert_policy` 与这组默认值
    /// 合成；路由和概览必须共用同一解析入口。
    pub(super) price_policy_defaults: PriceAlertPolicyDefaults,
    /// MacroEvent High 允许即时推的临近窗口。
    pub(super) macro_immediate_lookahead_hours: i64,
    pub(super) macro_immediate_grace_hours: i64,
    /// 部署方配置的全局 kind 黑名单。命中后 dispatch 直接返回 (0, 0),
    /// 任何 actor 的 prefs / cap / cooldown 都不再参与。
    pub(super) disabled_kinds: Arc<HashSet<String>>,
    /// 单次 poller tick 内,同一 ticker 触发 NewsCritical 升级 (Low→Medium)
    /// 的次数上限。0 = 不启用。命中后该条 Low 维持 Low,从而不进 digest 顶端。
    pub(super) news_upgrade_per_symbol_per_tick_cap: u32,
    /// 单次 poller tick 内 NewsCritical 升级 (Low→Medium) 的全局总上限。
    /// 0 = 不启用。用于防止多 ticker 同时提级造成摘要洪峰。
    pub(super) news_upgrade_per_tick_cap: u32,
    /// 当 tick 内每个 symbol 已升级的次数。`reset_tick_counters()` 在每次
    /// `process_events` 入口被调用,清零后重新计数。
    pub(super) news_upgrade_counter: Arc<Mutex<HashMap<String, u32>>>,
    pub(super) news_upgrade_total_counter: Arc<Mutex<u32>>,
    /// `source_class=uncertain` 的 NewsCritical 仲裁器。`None` → 跳过 LLM 路径,
    /// 维持 poller 给的 Low(与历史行为兼容)。
    pub(super) news_classifier: Option<Arc<dyn NewsClassifier>>,
    /// 全局默认重要性 prompt;per-actor `news_importance_prompt = None` 时回落。
    pub(super) default_importance_prompt: String,
    /// 单 tick 内 window convergence 升级/跳过统计,供 poller 级汇总日志消费。
    pub(super) news_upgrade_tick_stats: Arc<Mutex<NewsUpgradeTickStats>>,
    /// 盘中价格 band High 即时推的批内合流缓冲(2026-08 审计:开盘集体跳空
    /// 曾 10 秒内连发 7 条 DM)。`begin_dispatch_batch()`(`process_events`
    /// 入口)激活后,dispatch 遇到盘中 band High 不再逐条出站,而是按 actor
    /// 暂存于此;`flush_dispatch_batch()` 在批尾合并:同 actor ≥
    /// `PRICE_BURST_MIN_MERGE` 条合成一条汇总消息,更少则照旧逐条发送。
    /// `None` = 批模式未激活(直接调 `dispatch` 的调用方维持逐条即时行为)。
    pub(super) price_burst: Mutex<Option<PriceBurstBuffer>>,
    /// 共享润色 memo:`(event.id, 通用正文哈希) → 润色结果`。同一条 High 事件的
    /// 全部持有人只发一次润色 LLM(生产实测 RKLB 被 32 人持有 ⇒ 32 次降 1 次)。
    /// 失败(None)同样记忆——同一输入重试没有意义,只会把成本放大回 O(持有人)。
    pub(super) polish_memo: Mutex<HashMap<(String, u64), Option<String>>>,
}

/// 共享润色 memo 容量上限。事件在时间上成簇,满了整体清空即可,
/// 代价至多是每个活跃事件多润色一次;不值得为此上 LRU。
pub(super) const POLISH_MEMO_CAP: usize = 256;

/// actor_key → (actor, 本批暂存的盘中 band High 及其原始 severity)。
pub(super) type PriceBurstBuffer = HashMap<String, (ActorIdentity, Vec<(MarketEvent, Severity)>)>;

/// 同一批 poll 内、同一 actor 的盘中 band High 达到该条数时合并为一条汇总消息。
pub(super) const PRICE_BURST_MIN_MERGE: usize = 3;

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
            earnings_continuity: None,
            high_daily_cap: 0,
            runtime_timezone: hone_core::runtime_timezone(),
            same_symbol_cooldown_minutes: 0,
            // `NotificationRouter::new` 保留历史上的“重复 band 不限流”测试/嵌入语义；
            // 生产 EventEngine 总是通过 `with_price_policy_defaults` 注入 canonical 配置。
            price_policy_defaults: PriceAlertPolicyDefaults {
                repeat_step_pct: 0.0,
                ..PriceAlertPolicyDefaults::default()
            },
            macro_immediate_lookahead_hours: 6,
            macro_immediate_grace_hours: 2,
            disabled_kinds: Arc::new(HashSet::new()),
            news_upgrade_per_symbol_per_tick_cap: 0,
            news_upgrade_per_tick_cap: 0,
            news_upgrade_counter: Arc::new(Mutex::new(HashMap::new())),
            news_upgrade_total_counter: Arc::new(Mutex::new(0)),
            news_classifier: None,
            default_importance_prompt: DEFAULT_IMPORTANCE_PROMPT.to_string(),
            news_upgrade_tick_stats: Arc::new(Mutex::new(NewsUpgradeTickStats::default())),
            price_burst: Mutex::new(None),
            polish_memo: Mutex::new(HashMap::new()),
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

    pub fn with_earnings_continuity(
        mut self,
        reconciler: Arc<dyn EarningsContinuityReconciler>,
    ) -> Self {
        self.earnings_continuity = Some(reconciler);
        self
    }

    /// 每 actor 当日 High 推送上限。0 = 不启用(默认),与历史行为兼容。
    /// 命中上限后同 actor 当日剩余 High 事件自动降级进 digest。
    pub fn with_high_daily_cap(mut self, cap: u32) -> Self {
        self.high_daily_cap = cap;
        self
    }

    /// 配置 tz 偏移,用于计算"当日"窗口起点。默认 8 (北京)。
    #[cfg(test)]
    pub fn with_tz_offset_hours(mut self, offset: i32) -> Self {
        self.runtime_timezone = hone_core::RuntimeTimezone::fixed_offset_seconds(offset * 3600);
        self
    }

    pub fn with_runtime_timezone(mut self, timezone: hone_core::RuntimeTimezone) -> Self {
        self.runtime_timezone = timezone;
        self
    }

    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 命中冷却的事件降级到 digest,状态记 "cooled_down"。
    pub fn with_same_symbol_cooldown_minutes(mut self, minutes: u32) -> Self {
        self.same_symbol_cooldown_minutes = minutes;
        self
    }

    pub fn with_price_policy_defaults(mut self, defaults: PriceAlertPolicyDefaults) -> Self {
        self.price_policy_defaults = defaults;
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
        if let Ok(mut stats) = self.news_upgrade_tick_stats.lock() {
            *stats = NewsUpgradeTickStats::default();
        }
    }

    pub(crate) fn news_upgrade_tick_stats_snapshot(&self) -> NewsUpgradeTickStats {
        self.news_upgrade_tick_stats
            .lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
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
}
