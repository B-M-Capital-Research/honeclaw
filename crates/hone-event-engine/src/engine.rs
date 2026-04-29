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
use crate::digest::{self, DigestBuffer};
use crate::fmp::FmpClient;
use crate::news_classifier;
use crate::pipeline;
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::pollers::{
    AnalystGradePoller, CorpActionCalendarPoller, EarningsPoller, EarningsSurprisePoller,
    MacroPoller, NewsPoller, PricePoller, RssNewsPoller, SecFilingsPoller, TelegramChannelPoller,
};
use crate::prefs::{FilePrefsStorage, PrefsProvider};
use crate::router::{LogSink, NotificationRouter, OutboundSink};
use crate::source::SourceSchedule;
use crate::spawner::spawn_event_source;
use crate::store::EventStore;
use crate::subscription::SharedRegistry;
use crate::unified_digest::UnifiedDigestScheduler;
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
    /// 周期任务观测落盘目录(对齐 heartbeat 的 `data/runtime/`)。
    /// `None` 关闭——所有 task 仍会跑、仍会 tracing,只是不写 task_runs.jsonl。
    /// 用 Arc 让多个 spawn 调用共用一份 PathBuf,避免每个 task 都 clone PathBuf。
    task_runs_dir: Option<Arc<PathBuf>>,
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
            task_runs_dir: None,
            sink: Arc::new(LogSink),
            polisher: Arc::new(NoopPolisher),
            news_classifier: None,
            global_digest_provider: None,
            retention_days: 30,
        }
    }

    /// 周期任务观测落盘目录(`data/runtime/task_runs.YYYY-MM-DD.jsonl`)。
    /// `None`(默认)关闭。建议传入 `hone_core::task_observer::task_runs_dir(&config)`
    /// 跟 heartbeat 同级。
    pub fn with_task_runs_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.task_runs_dir = dir.map(Arc::new);
        self
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

        let task_runs_dir = self.task_runs_dir.clone();
        if let Some(dir) = task_runs_dir.as_deref() {
            info!(
                task_runs = %dir.display(),
                retention_days = hone_core::TASK_RUNS_RETENTION_DAYS,
                "task observer enabled"
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

        // UnifiedDigestScheduler：取代旧 DigestScheduler + GlobalDigestScheduler 双 spawn,
        // 每 60s tick 一次,以 actor × digest_slots 触发,每个 slot 跨 actor 共享一份
        // `audience+pass1+fetch+baseline`,personalize fan-out 走 per-actor。
        let tz_offset = hone_core::config::tz_offset_hours(&self.engine_cfg.digest.timezone);
        info!(
            timezone = %self.engine_cfg.digest.timezone,
            offset_hours = tz_offset,
            "unified digest scheduler timezone resolved"
        );
        let portfolio_storage = Arc::new(hone_memory::PortfolioStorage::new(&self.portfolio_dir));
        let fmp_arc = Arc::new(client.clone());
        let audience_cache_dir = self
            .store_path
            .parent()
            .map(|p| p.join("company_profiles"))
            .unwrap_or_else(|| PathBuf::from("./data/company_profiles"));
        let fetcher = Arc::new(crate::global_digest::ArticleFetcher::new());
        let mut unified = UnifiedDigestScheduler::new(
            digest_buffer.clone(),
            self.sink.clone(),
            store.clone(),
            fmp_arc.clone(),
            portfolio_storage.clone(),
            prefs_storage.clone(),
            registry.clone(),
            fetcher.clone(),
            audience_cache_dir.clone(),
            self.daily_report_dir.clone(),
            self.engine_cfg.digest.pre_market.clone(),
            self.engine_cfg.digest.post_market.clone(),
        )
        .with_tz_offset_hours(tz_offset)
        .with_max_items_per_batch(self.engine_cfg.digest.max_items_per_batch as usize)
        .with_min_gap_minutes(self.engine_cfg.digest.min_gap_minutes)
        .with_lookback_hours(self.engine_cfg.global_digest.lookback_hours)
        .with_pass2_top_n(self.engine_cfg.global_digest.pass2_top_n)
        .with_final_pick_n(self.engine_cfg.global_digest.final_pick_n)
        .with_fetch_full_text(self.engine_cfg.global_digest.fetch_full_text)
        .with_event_dedupe_enabled(self.engine_cfg.global_digest.event_dedupe_enabled);
        if self.engine_cfg.global_digest.enabled {
            if let Some(provider) = self.global_digest_provider.clone() {
                let curator = Arc::new(crate::global_digest::Curator::new(
                    provider.clone(),
                    self.engine_cfg.global_digest.pass1_model.clone(),
                    self.engine_cfg.global_digest.pass2_model.clone(),
                ));
                unified = unified.with_curator(curator);
                if self.engine_cfg.global_digest.event_dedupe_enabled {
                    let event_deduper: Arc<dyn crate::global_digest::EventDeduper> =
                        Arc::new(crate::global_digest::LlmEventDeduper::new(
                            provider.clone(),
                            self.engine_cfg.global_digest.event_dedupe_model.clone(),
                        ));
                    unified = unified.with_event_deduper(event_deduper);
                }
                info!(
                    pass1_model = %self.engine_cfg.global_digest.pass1_model,
                    pass2_model = %self.engine_cfg.global_digest.pass2_model,
                    event_dedupe = self.engine_cfg.global_digest.event_dedupe_enabled,
                    event_dedupe_model = %self.engine_cfg.global_digest.event_dedupe_model,
                    "unified digest curator wired (global news enabled)"
                );
            } else {
                warn!(
                    "global_digest enabled but no LLM provider injected — global news section will be empty until .with_global_digest_provider() is called"
                );
            }
        } else {
            info!("global_digest disabled — unified digest will only ship buffered + synth items");
        }
        let scheduler = Arc::new(unified);
        tokio::spawn(pipeline::cron_minute_tick(
            "internal.unified_digest_scheduler",
            tz_offset,
            task_runs_dir.clone(),
            move |now, fired| {
                let scheduler = scheduler.clone();
                Box::pin(async move { scheduler.tick_once(now, fired).await.map(|_| ()) })
            },
        ));

        // DailyReport —— 本地 22:00 每 60s tick 一次,把当日分布落盘到
        // `data/daily_reports/YYYY-MM-DD.md` + 一行 tracing::info。
        // 不通过 sink 推给用户:这是给我自己看的引擎运营日志。
        let daily_report = Arc::new(
            DailyReport::new(store.clone(), self.daily_report_dir.clone())
                .with_tz_offset_hours(tz_offset),
        );
        tokio::spawn(pipeline::cron_minute_tick(
            "internal.daily_report",
            tz_offset,
            task_runs_dir.clone(),
            move |now, fired| {
                let daily_report = daily_report.clone();
                Box::pin(async move {
                    let n = daily_report.tick_once(now, fired).await?;
                    if n > 0 {
                        info!(
                            task = "internal.daily_report",
                            sent = n,
                            "daily report fanout"
                        );
                    }
                    Ok(())
                })
            },
        ));

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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
        } else if fmp_available {
            info!("news poller disabled by config.sources.news=false");
        }
        // corp_action 与 sec_filings 现在是两个独立 EventSource(原 CorpActionPoller
        // 已拆为 CorpActionCalendarPoller + SecFilingsPoller),各自的 enable flag
        // 直接控制是否 spawn 自己——不再把两者塞进同一个 task。
        if fmp_available && sources.corp_action {
            let poller = CorpActionCalendarPoller::new(
                client.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            );
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
        } else if fmp_available {
            info!("corp_action calendar poller disabled by config.sources.corp_action=false");
        }
        if fmp_available && sources.sec_filings {
            // sec_recent_hours=48:每次 tick 只把"过去 48h 新出现的"8-K 送入 store;
            // store.insert_event 幂等 IGNORE 保证同一 filing 不会触发两次 dispatch。
            let poller = SecFilingsPoller::new(
                client.clone(),
                registry.clone(),
                SourceSchedule::CronAligned {
                    pre_prefetch: pre_prefetch.clone(),
                    post_prefetch: post_prefetch.clone(),
                    tz_offset,
                },
            )
            .with_sec_recent_hours(48);
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
        } else if fmp_available {
            info!("sec filings poller disabled by config.sources.sec_filings=false");
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
        } else if fmp_available {
            info!("macro poller disabled by config.sources.macro_calendar=false");
        }
        // PricePoller 每 tick 从 SharedRegistry 读最新 watch pool;
        // 空则 EventSource::poll 直接返回 Ok(vec![]),用户新增持仓后下个 tick 就能生效。
        if fmp_available && sources.price {
            let poller = PricePoller::new(
                client.clone(),
                registry.clone(),
                SourceSchedule::FixedInterval(Duration::from_secs(
                    self.engine_cfg.poll_intervals.price_secs,
                )),
            )
            .with_thresholds(
                self.engine_cfg.thresholds.price_alert_low_pct,
                self.engine_cfg.thresholds.price_alert_high_pct,
            )
            .with_realert_step_pct(self.engine_cfg.thresholds.price_realert_step_pct);
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
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
            spawn_event_source(
                Arc::new(poller),
                store.clone(),
                router.clone(),
                task_runs_dir.clone(),
            );
        }

        // 注:thesis 蒸馏 cron 在 hone-web-api 启动时单独 spawn,
        // 这里不能 spawn 是因为 hone-event-engine 不依赖 hone-channels(避免循环)。

        Ok(())
    }
}
