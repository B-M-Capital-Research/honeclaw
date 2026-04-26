//! `EventEngine` —— 事件引擎的 façade。
//!
//! 只负责两件事:
//! 1. `new()` + 一组 `with_*` builder:填路径 / 注入 sink / polisher / classifier;
//! 2. `start()`:根据配置决定哪些 poller 要 `spawn`,然后立即返回。
//!
//! 具体的 spawn 模板在 sibling `spawner.rs`;poller → store → router 的
//! 数据流胶水在 `pipeline.rs`。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::daily_report::DailyReport;
use crate::digest::{self, DigestBuffer, DigestScheduler};
use crate::fmp::FmpClient;
use crate::news_classifier;
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::pollers::{
    AnalystGradePoller, EarningsPoller, EarningsSurprisePoller, MacroPoller, NewsPoller,
    RssNewsPoller, TelegramChannelPoller,
};
use crate::prefs::{FilePrefsStorage, PrefsProvider};
use crate::router::{LogSink, NotificationRouter, OutboundSink};
use crate::source::SourceSchedule;
use crate::spawner::{spawn_corp_action_poller, spawn_event_source, spawn_price_poller};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;
use hone_core::config::{EventEngineConfig, FmpConfig};

/// 事件引擎句柄。`start()` 只 `spawn` 各 poller 任务并立即返回——
/// 调用方不会被阻塞；引擎任务随 tokio runtime 生命周期存续。
pub struct EventEngine {
    engine_cfg: EventEngineConfig,
    fmp_cfg: FmpConfig,
    store_path: PathBuf,
    events_jsonl_path: Option<PathBuf>,
    portfolio_dir: PathBuf,
    digest_dir: PathBuf,
    prefs_dir: PathBuf,
    daily_report_dir: PathBuf,
    sink: Arc<dyn OutboundSink>,
    polisher: Arc<dyn BodyPolisher>,
    news_classifier: Option<Arc<dyn news_classifier::NewsClassifier>>,
    /// LLM provider 用于 global_digest 的 Curator(Pass 1 + Pass 2)。
    /// 缺省 None → global_digest 调度器不会启动,即使 config.global_digest.enabled=true
    /// 也只 warn 不报错。
    global_digest_provider: Option<Arc<dyn hone_llm::LlmProvider>>,
    retention_days: i64,
}

impl EventEngine {
    pub fn new(engine_cfg: EventEngineConfig, fmp_cfg: FmpConfig) -> Self {
        Self {
            engine_cfg,
            fmp_cfg,
            store_path: PathBuf::from("./data/events.db"),
            events_jsonl_path: Some(PathBuf::from("./data/events.jsonl")),
            portfolio_dir: PathBuf::from("./data/portfolio"),
            digest_dir: PathBuf::from("./data/digest_buffer"),
            prefs_dir: PathBuf::from("./data/notif_prefs"),
            daily_report_dir: PathBuf::from("./data/daily_reports"),
            sink: Arc::new(LogSink),
            polisher: Arc::new(NoopPolisher),
            news_classifier: None,
            global_digest_provider: None,
            retention_days: 30,
        }
    }

    pub fn with_store_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.store_path = path.into();
        self
    }

    /// JSONL 镜像路径；传 `None` 可关闭。默认 `./data/events.jsonl`。
    pub fn with_events_jsonl_path(mut self, path: Option<PathBuf>) -> Self {
        self.events_jsonl_path = path;
        self
    }

    /// events / delivery_log 保留天数，默认 30。传 0 禁用自动清理。
    pub fn with_retention_days(mut self, days: i64) -> Self {
        self.retention_days = days;
        self
    }

    pub fn with_portfolio_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.portfolio_dir = path.into();
        self
    }

    pub fn with_digest_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.digest_dir = path.into();
        self
    }

    /// 每 actor 一个 JSON 文件的通知偏好目录；默认 `./data/notif_prefs`。
    /// 用户直接编辑该目录下文件即可运行时改推送策略。
    pub fn with_prefs_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.prefs_dir = path.into();
        self
    }

    /// 日报(Markdown 快照)输出目录;默认 `./data/daily_reports`。
    /// 每日本地 22:00 自动写一个 `YYYY-MM-DD.md`,只作为运维日志,不推送。
    pub fn with_daily_report_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.daily_report_dir = path.into();
        self
    }

    pub fn with_sink(mut self, sink: Arc<dyn OutboundSink>) -> Self {
        self.sink = sink;
        self
    }

    pub fn with_polisher(mut self, polisher: Arc<dyn BodyPolisher>) -> Self {
        self.polisher = polisher;
        self
    }

    /// 注入"不确定来源"NewsCritical 的 LLM 仲裁器(典型实现:`LlmNewsClassifier`)。
    /// `None`(默认)→ 该路径关闭,uncertain 源新闻保持 poller 给的 Low。
    pub fn with_news_classifier(
        mut self,
        classifier: Arc<dyn news_classifier::NewsClassifier>,
    ) -> Self {
        self.news_classifier = Some(classifier);
        self
    }

    /// 注入 LLM provider 用于 global_digest 的 Curator(Pass 1 + Pass 2)。
    /// 缺省 None → global_digest 不会启动(即使 config.global_digest.enabled=true 也只 warn)。
    pub fn with_global_digest_provider(mut self, provider: Arc<dyn hone_llm::LlmProvider>) -> Self {
        self.global_digest_provider = Some(provider);
        self
    }

    /// 启动事件引擎。非阻塞：内部 spawn 后立即返回 Ok。
    pub async fn start(&self) -> anyhow::Result<()> {
        if !self.engine_cfg.enabled {
            info!("event engine disabled via config");
            return Ok(());
        }
        info!(
            news_secs = self.engine_cfg.poll_intervals.news_secs,
            price_secs = self.engine_cfg.poll_intervals.price_secs,
            prefetch_offset_mins = self.engine_cfg.digest.prefetch_offset_mins,
            dryrun = self.engine_cfg.dryrun,
            store = %self.store_path.display(),
            portfolio = %self.portfolio_dir.display(),
            "event engine starting"
        );
        info!(
            news_upgrade_per_symbol_per_tick =
                self.engine_cfg.thresholds.news_upgrade_per_symbol_per_tick,
            news_upgrade_per_tick = self.engine_cfg.thresholds.news_upgrade_per_tick,
            "event engine news upgrade guards configured"
        );
        if self.engine_cfg.sources.news
            && self.engine_cfg.thresholds.news_upgrade_per_symbol_per_tick == 0
            && self.engine_cfg.thresholds.news_upgrade_per_tick == 0
        {
            warn!(
                "event engine news upgrade guards disabled; window convergence bursts will not be capped"
            );
        }

        let client = FmpClient::from_config(&self.fmp_cfg);
        let fmp_available = client.has_keys();
        if !fmp_available {
            warn!(
                "event engine: FMP key missing — FMP pollers 不会启动,仅非 FMP 源(Telegram/RSS)照常运行"
            );
        }

        let mut store_builder = EventStore::open(&self.store_path)?;
        if let Some(jsonl) = &self.events_jsonl_path {
            store_builder = store_builder.with_jsonl_path(jsonl);
            info!(jsonl = %jsonl.display(), "events jsonl mirror enabled");
        }
        let store = Arc::new(store_builder);
        info!(baseline = ?store.baseline_at().ok(), "event store ready");

        // 清理任务：每 24h 扫一次 events / delivery_log。retention_days==0 禁用。
        if self.retention_days > 0 {
            let store_cleanup = store.clone();
            let days = self.retention_days;
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    ticker.tick().await;
                    match store_cleanup.purge_events_older_than(days) {
                        Ok(n) if n > 0 => info!(removed = n, days, "events retention sweep"),
                        Ok(_) => {}
                        Err(e) => warn!("events purge failed: {e:#}"),
                    }
                    match store_cleanup.purge_delivery_log_older_than(days) {
                        Ok(n) if n > 0 => {
                            info!(removed = n, days, "delivery_log retention sweep")
                        }
                        Ok(_) => {}
                        Err(e) => warn!("delivery_log purge failed: {e:#}"),
                    }
                }
            });
        }

        // 基于持仓构建订阅注册中心，封装在 SharedRegistry 里支持运行时热刷新。
        // 初次读盘在 start() 内完成；之后后台任务每 60s 重建一次（下面 spawn）。
        let registry = Arc::new(SharedRegistry::from_portfolio_dir(&self.portfolio_dir));
        info!(
            subscribers = registry.load().len(),
            "subscription registry initialized (hot-refreshable)"
        );

        // 热刷新任务：定期从 portfolio 目录重建 registry，用户改持仓后下次推送即可命中。
        {
            let registry_bg = registry.clone();
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(60));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                let mut last_size = registry_bg.load().len();
                loop {
                    ticker.tick().await;
                    if let Some(new_size) = registry_bg.refresh() {
                        if new_size != last_size {
                            info!(
                                subscribers = new_size,
                                previous = last_size,
                                "subscription registry refreshed"
                            );
                            last_size = new_size;
                        }
                    }
                }
            });
        }

        let digest_buffer = Arc::new(DigestBuffer::new(&self.digest_dir)?);
        info!(
            digest = %self.digest_dir.display(),
            pre_market = %self.engine_cfg.digest.pre_market,
            post_market = %self.engine_cfg.digest.post_market,
            "digest buffer ready"
        );

        let prefs_storage: Arc<dyn PrefsProvider> =
            Arc::new(FilePrefsStorage::new(&self.prefs_dir)?);
        info!(
            prefs_dir = %self.prefs_dir.display(),
            "notification prefs dir ready (edit per-actor JSON to change runtime)"
        );

        let tz_offset_for_router =
            hone_core::config::tz_offset_hours(&self.engine_cfg.digest.timezone);
        let mut router_builder = NotificationRouter::new(
            registry.clone(),
            self.sink.clone(),
            store.clone(),
            digest_buffer.clone(),
        )
        .with_polisher(self.polisher.clone())
        .with_prefs(prefs_storage.clone())
        .with_tz_offset_hours(tz_offset_for_router)
        .with_high_daily_cap(self.engine_cfg.thresholds.high_severity_daily_cap)
        .with_same_symbol_cooldown_minutes(self.engine_cfg.thresholds.same_symbol_cooldown_minutes)
        .with_price_min_direct_pct(self.engine_cfg.thresholds.price_min_direct_pct)
        .with_price_intraday_min_gap_minutes(
            self.engine_cfg.thresholds.price_intraday_min_gap_minutes,
        )
        .with_price_symbol_direction_daily_cap(
            self.engine_cfg.thresholds.price_symbol_direction_daily_cap,
        )
        .with_price_close_direct_enabled(self.engine_cfg.thresholds.price_close_direct_enabled)
        .with_large_position_weight_pct(self.engine_cfg.thresholds.large_position_weight_pct)
        .with_macro_immediate_window(
            self.engine_cfg.thresholds.macro_immediate_lookahead_hours,
            self.engine_cfg.thresholds.macro_immediate_grace_hours,
        )
        .with_disabled_kinds(self.engine_cfg.disabled_kinds.clone())
        .with_news_upgrade_per_symbol_per_tick_cap(
            self.engine_cfg.thresholds.news_upgrade_per_symbol_per_tick,
        )
        .with_news_upgrade_per_tick_cap(self.engine_cfg.thresholds.news_upgrade_per_tick)
        .with_default_importance_prompt(self.engine_cfg.news_importance_prompt.clone());
        if let Some(classifier) = self.news_classifier.clone() {
            router_builder = router_builder.with_news_classifier(classifier);
            info!("event engine: news LLM classifier 已装配 (uncertain-source 升 Medium)");
        }
        let router = Arc::new(router_builder);
        if !self.engine_cfg.disabled_kinds.is_empty() {
            info!(
                disabled = ?self.engine_cfg.disabled_kinds,
                "event-kind global blacklist active; matching events will be stored but not dispatched"
            );
        }

        // DigestScheduler：每 60s 检查一次本地时间，命中 pre/post-market 触发 flush。
        // 分钟级分辨率已由 in_window 保障；`already_fired_today` 防止同分钟重触发。
        let tz_offset = hone_core::config::tz_offset_hours(&self.engine_cfg.digest.timezone);
        info!(
            timezone = %self.engine_cfg.digest.timezone,
            offset_hours = tz_offset,
            "digest scheduler timezone resolved"
        );
        let scheduler = DigestScheduler::new(
            digest_buffer.clone(),
            self.sink.clone(),
            self.engine_cfg.digest.pre_market.clone(),
            self.engine_cfg.digest.post_market.clone(),
        )
        .with_tz_offset_hours(tz_offset)
        .with_store(store.clone())
        .with_registry(registry.clone())
        .with_prefs(prefs_storage.clone())
        .with_max_items_per_batch(self.engine_cfg.digest.max_items_per_batch as usize)
        .with_min_gap_minutes(self.engine_cfg.digest.min_gap_minutes);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(60));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut fired = std::collections::HashSet::new();
            let mut last_date = String::new();
            loop {
                ticker.tick().await;
                let now = chrono::Utc::now();
                // 跨日后清空 fired 集合，避免长期堆积。
                let today = digest::local_date_key(now, tz_offset);
                if today != last_date {
                    fired.clear();
                    last_date = today;
                }
                if let Err(e) = scheduler.tick_once(now, &mut fired).await {
                    warn!("digest scheduler tick failed: {e:#}");
                }
            }
        });

        // DailyReport —— 本地 22:00 每 60s tick 一次,把当日分布落盘到
        // `data/daily_reports/YYYY-MM-DD.md` + 一行 tracing::info。
        // 不通过 sink 推给用户:这是给我自己看的引擎运营日志。
        let daily_report = DailyReport::new(store.clone(), self.daily_report_dir.clone())
            .with_tz_offset_hours(tz_offset);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(60));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut fired = std::collections::HashSet::new();
            let mut last_date = String::new();
            loop {
                ticker.tick().await;
                let now = chrono::Utc::now();
                let today = digest::local_date_key(now, tz_offset);
                if today != last_date {
                    fired.clear();
                    last_date = today;
                }
                match daily_report.tick_once(now, &mut fired).await {
                    Ok(n) if n > 0 => info!(sent = n, "daily report fanout"),
                    Ok(_) => {}
                    Err(e) => warn!("daily report tick failed: {e:#}"),
                }
            }
        });

        info!(
            watch_pool_size = registry.load().watch_pool().len(),
            "initial watch pool snapshot (price poller 每 tick 取最新)"
        );

        // ── 各 poller 独立 spawn ──────────────────────────────────────
        // sources.* 是 per-poller 1:1 的"最省钱"关法:直接不 spawn 对应 poller;
        // 事件既不入库也不分发。需要"poller 仍跑、只是 router 丢弃某 kind"
        // 这种兜底关法,改用 EventEngineConfig.disabled_kinds。
        //
        // v0.1.46 开始:earnings / corp_action / macro / analyst_grade / earnings_surprise
        // 这 5 个日频 poller 改为 **cron-aligned**:在 `pre_market - offset` /
        // `post_market - offset` 各跑一次,保证 digest flush 时用到的数据永远是刚拉的。
        // 同时冷启动时每个 poller 立即跑一次,避免用户重启后等到下一个 flush 窗口。
        // news / price 节奏本来就快(分钟级),继续用固定 interval。
        let sources = &self.engine_cfg.sources;
        let pre_prefetch = digest::shift_hhmm_earlier(
            &self.engine_cfg.digest.pre_market,
            self.engine_cfg.digest.prefetch_offset_mins,
        );
        let post_prefetch = digest::shift_hhmm_earlier(
            &self.engine_cfg.digest.post_market,
            self.engine_cfg.digest.prefetch_offset_mins,
        );
        info!(
            pre_market = %self.engine_cfg.digest.pre_market,
            post_market = %self.engine_cfg.digest.post_market,
            prefetch_offset_mins = self.engine_cfg.digest.prefetch_offset_mins,
            pre_prefetch = %pre_prefetch,
            post_prefetch = %post_prefetch,
            "cron-aligned poller prefetch windows resolved"
        );

        if fmp_available && sources.earnings_calendar {
            let poller = EarningsPoller::new(
                client.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            )
            .with_window_days(self.engine_cfg.earnings.window_days);
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        } else if fmp_available {
            info!("earnings_calendar poller disabled by config.sources.earnings_calendar=false");
        }
        if fmp_available && sources.news {
            let poller = NewsPoller::new(
                client.clone(),
                SourceSchedule::FixedInterval(Duration::from_secs(
                    self.engine_cfg.poll_intervals.news_secs,
                )),
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        } else if fmp_available {
            info!("news poller disabled by config.sources.news=false");
        }
        if fmp_available && (sources.corp_action || sources.sec_filings) {
            spawn_corp_action_poller(
                client.clone(),
                store.clone(),
                router.clone(),
                registry.clone(),
                tz_offset,
                pre_prefetch.clone(),
                post_prefetch.clone(),
                sources.corp_action,
                sources.sec_filings,
            );
        } else if fmp_available {
            info!(
                "corp_action poller fully disabled by config.sources.corp_action=false and sources.sec_filings=false"
            );
        }
        if fmp_available && sources.macro_calendar {
            let poller = MacroPoller::new(
                client.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        } else if fmp_available {
            info!("macro poller disabled by config.sources.macro_calendar=false");
        }
        // PricePoller 每 tick 从 SharedRegistry 读最新 watch pool。
        // 若此刻为空就 skip tick；用户新增持仓后下个 tick 就能生效。
        if fmp_available && sources.price {
            spawn_price_poller(
                client.clone(),
                store.clone(),
                router.clone(),
                registry.clone(),
                self.engine_cfg.thresholds.price_alert_low_pct,
                self.engine_cfg.thresholds.price_alert_high_pct,
                self.engine_cfg.thresholds.price_realert_step_pct,
                Duration::from_secs(self.engine_cfg.poll_intervals.price_secs),
            );
        } else if fmp_available {
            info!("price poller disabled by config.sources.price=false");
        }
        // 分析师评级、财报 surprise：两个都按 watch pool 逐 ticker 拉。
        // 初次 tick 之前 watch pool 为空就跳过——用户新增持仓后下一个 tick 生效。
        // poller 自身持 Arc<SharedRegistry>,每次 poll 内部取最新 watch_pool,
        // 不需要 spawn 端再做 capture/clone。
        if fmp_available && sources.analyst_grade {
            let poller = AnalystGradePoller::new(
                client.clone(),
                registry.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        } else if fmp_available {
            info!("analyst_grade poller disabled by config.sources.analyst_grade=false");
        }
        if fmp_available && sources.earnings_surprise {
            let poller = EarningsSurprisePoller::new(
                client.clone(),
                registry.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        } else if fmp_available {
            info!("earnings_surprise poller disabled by config.sources.earnings_surprise=false");
        }

        // ── 社交源监听(通用 EventSource trait)─────────────────────────
        // Telegram channel web preview。
        // 事件一律 Low + payload.source_class="uncertain",交给 router 的
        // LLM 仲裁链路按"是否重要"决定升 Medium 即时推(见 router.rs
        // maybe_llm_upgrade_for_actor)。symbols 多数为空,靠 social
        // GlobalSubscription(见 subscription.rs registry_from_portfolios)
        // 把事件 fanout 给所有 actor 后再过 LLM。
        for cfg in &sources.telegram_channels {
            let poller = TelegramChannelPoller::new(
                cfg.handle.clone(),
                Duration::from_secs(cfg.interval_secs),
                cfg.extract_cashtags,
            );
            info!(
                handle = %cfg.handle,
                interval_secs = cfg.interval_secs,
                extract_cashtags = cfg.extract_cashtags,
                "telegram channel poller starting"
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        }
        // 通用 RSS 源 —— global_digest 的核心数据补充。事件直接落 source="rss:{handle}"
        // 入 events 表,collector 与 FMP news 一同拉,curator 不区分来源。
        for cfg in &sources.rss_feeds {
            let poller = RssNewsPoller::new(
                cfg.handle.clone(),
                cfg.url.clone(),
                Duration::from_secs(cfg.interval_secs),
            );
            info!(
                handle = %cfg.handle,
                url = %cfg.url,
                interval_secs = cfg.interval_secs,
                "rss feed poller starting"
            );
            spawn_event_source(Arc::new(poller), store.clone(), router.clone());
        }

        // ── 全局 digest scheduler ─────────────────────────────────────
        // LLM 精读后每天 N 次的"今日全球要闻"。每分钟 tick 检查 schedule 命中。
        if self.engine_cfg.global_digest.enabled {
            if self.engine_cfg.global_digest.schedules.is_empty() {
                warn!(
                    "global_digest enabled 但 schedules 为空 —— 不会触发。在 config 里加 schedules: [\"HH:MM\", ...]"
                );
            } else if let Some(provider) = self.global_digest_provider.clone() {
                let portfolio_storage =
                    Arc::new(hone_memory::PortfolioStorage::new(&self.portfolio_dir));
                let fmp_arc = Arc::new(client.clone());
                let curator = Arc::new(crate::global_digest::Curator::new(
                    provider.clone(),
                    self.engine_cfg.global_digest.pass1_model.clone(),
                    self.engine_cfg.global_digest.pass2_model.clone(),
                ));
                let fetcher = Arc::new(crate::global_digest::ArticleFetcher::new());
                let event_deduper: Arc<dyn crate::global_digest::EventDeduper> =
                    if self.engine_cfg.global_digest.event_dedupe_enabled {
                        Arc::new(crate::global_digest::LlmEventDeduper::new(
                            provider.clone(),
                            self.engine_cfg.global_digest.event_dedupe_model.clone(),
                        ))
                    } else {
                        Arc::new(crate::global_digest::PassThroughDeduper)
                    };
                let audience_cache_dir = self
                    .store_path
                    .parent()
                    .map(|p| p.join("company_profiles"))
                    .unwrap_or_else(|| PathBuf::from("./data/company_profiles"));
                let scheduler = Arc::new(
                    crate::global_digest::GlobalDigestScheduler::new(
                        self.engine_cfg.global_digest.clone(),
                        store.clone(),
                        fmp_arc,
                        portfolio_storage,
                        prefs_storage.clone(),
                        self.sink.clone(),
                        curator,
                        fetcher,
                        audience_cache_dir,
                        self.daily_report_dir.clone(),
                    )
                    .with_event_deduper(event_deduper),
                );
                info!(
                    schedules = ?self.engine_cfg.global_digest.schedules,
                    timezone = %self.engine_cfg.global_digest.timezone,
                    pass1_model = %self.engine_cfg.global_digest.pass1_model,
                    pass2_model = %self.engine_cfg.global_digest.pass2_model,
                    event_dedupe = self.engine_cfg.global_digest.event_dedupe_enabled,
                    event_dedupe_model = %self.engine_cfg.global_digest.event_dedupe_model,
                    "global_digest scheduler starting"
                );
                tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(Duration::from_secs(60));
                    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    loop {
                        ticker.tick().await;
                        let now = chrono::Utc::now();
                        let _ = scheduler.tick(now).await;
                    }
                });
                // 注:thesis 蒸馏 cron 在 hone-web-api 启动时单独 spawn,
                // 这里不能 spawn 是因为 hone-event-engine 不依赖 hone-channels(避免循环)。
            } else {
                warn!(
                    "global_digest enabled 但未注入 LLM provider —— 调度器跳过。在 EventEngine builder 里调 .with_global_digest_provider()"
                );
            }
        }

        Ok(())
    }
}
