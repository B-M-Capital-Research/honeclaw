//! `NotificationRouter` 结构体定义 + `new` + 17 个 `with_*` builder + per-tick
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

use crate::digest::DigestBuffer;
use crate::news_classifier::{DEFAULT_IMPORTANCE_PROMPT, NewsClassifier};
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::prefs::{AllowAllPrefs, PrefsProvider};
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
    /// 每 actor 当日 sink=sent 且 severity=high 的条数上限。超过后新的 High
    /// 事件自动降级进 digest,并在 delivery_log 写 status="capped"。
    /// 0 = 不启用。
    pub(super) high_daily_cap: u32,
    /// 解释"当日"起点所用的 UTC 偏移(小时)。
    pub(super) tz_offset_hours: i32,
    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 防止同一 ticker 短时间内被价格异动 + 新闻 + SEC filing 三连推。
    /// 命中后降级到 digest,log_delivery 写 status="cooled_down"。
    pub(super) same_symbol_cooldown_minutes: u32,
    /// 用户价格阈值覆盖的系统级最小即时推阈值。
    pub(super) price_min_direct_pct: f64,
    /// 同一 actor + symbol + direction 两次价格 band 即时推的最小间隔。
    pub(super) price_intraday_min_gap_minutes: u32,
    /// 同一 actor + symbol + direction 每日本地日价格 band 即时推上限。
    pub(super) price_symbol_direction_daily_cap: u32,
    /// 收盘价格异动是否允许即时推；默认只进入摘要。
    pub(super) price_close_direct_enabled: bool,
    /// 大仓位标的用用户敏感阈值直推的默认仓位权重门槛。
    pub(super) large_position_weight_pct: f64,
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
            price_intraday_min_gap_minutes: 0,
            price_symbol_direction_daily_cap: 0,
            price_close_direct_enabled: false,
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
            news_upgrade_tick_stats: Arc::new(Mutex::new(NewsUpgradeTickStats::default())),
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

    pub fn with_price_intraday_min_gap_minutes(mut self, minutes: u32) -> Self {
        self.price_intraday_min_gap_minutes = minutes;
        self
    }

    pub fn with_price_symbol_direction_daily_cap(mut self, cap: u32) -> Self {
        self.price_symbol_direction_daily_cap = cap;
        self
    }

    pub fn with_price_close_direct_enabled(mut self, enabled: bool) -> Self {
        self.price_close_direct_enabled = enabled;
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
