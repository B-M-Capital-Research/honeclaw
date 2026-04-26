//! 把各个 poller 包成独立 tokio 任务的 `spawn_*` 入口集合。
//!
//! 每个函数只做三件事:
//! 1. 构造 poller 实例并 `Arc::new`;
//! 2. 用 `move ||` 拼一个 action closure,捕获 `store`/`router`/`registry`;
//! 3. `tokio::spawn` 后调 `pipeline::cron_aligned_loop` 或自家 ticker 循环。
//!
//! 原来 `start()` 一坨 500+ 行的 spawn 代码全部挪到这里,`engine.rs::start()`
//! 只负责配置解析 + 决定是否 spawn。分离后两边都能独立读懂。

use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::digest;
use crate::fmp::FmpClient;
use crate::pipeline::{cron_aligned_loop, log_poller_error, process_events, run_once};
use crate::pollers::{
    AnalystGradePoller, CorpActionPoller, EarningsPoller, EarningsSurprisePoller, MacroPoller,
    NewsPoller, PricePoller,
};
use crate::router::NotificationRouter;
use crate::source::{EventSource, SourceSchedule};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;

pub(crate) fn spawn_earnings_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
    window_days: i64,
) {
    tokio::spawn(async move {
        let poller = Arc::new(EarningsPoller::new(client).with_window_days(window_days));
        let action = move || {
            let poller = poller.clone();
            let store = store.clone();
            let router = router.clone();
            Box::pin(async move {
                match poller.poll().await {
                    Ok(events) => process_events("earnings", events, &store, &router).await,
                    Err(e) => log_poller_error("earnings", "fmp.earning_calendar", "calendar", &e),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        };
        cron_aligned_loop("earnings", tz_offset, pre_prefetch, post_prefetch, action).await;
    });
}

pub(crate) fn spawn_news_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    interval: Duration,
) {
    tokio::spawn(async move {
        let poller = NewsPoller::new(client);
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            match poller.poll().await {
                Ok(events) => process_events("news", events, &store, &router).await,
                Err(e) => log_poller_error("news", "fmp.stock_news", "stock_news", &e),
            }
        }
    });
}

pub(crate) fn spawn_corp_action_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    registry: Arc<SharedRegistry>,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
    corp_action_enabled: bool,
    sec_filings_enabled: bool,
) {
    tokio::spawn(async move {
        if !corp_action_enabled {
            info!(
                "corp_action dividend/split fetch disabled by config.sources.corp_action=false; sec_filings 仍按设置运行"
            );
        }
        if !sec_filings_enabled {
            info!("sec filings fetch disabled by config.sources.sec_filings=false");
        }
        // sec_recent_hours=48:每次 tick 只把"过去 48h 新出现的"8-K 送入 store;
        // store.insert_event 幂等 IGNORE 保证同一 filing 不会触发两次 dispatch。
        let poller = Arc::new(CorpActionPoller::new(client).with_sec_recent_hours(48));
        let action = move || {
            let poller = poller.clone();
            let store = store.clone();
            let router = router.clone();
            let registry = registry.clone();
            Box::pin(async move {
                if corp_action_enabled {
                    match poller.poll().await {
                        Ok(events) => process_events("corp_action", events, &store, &router).await,
                        Err(e) => log_poller_error("corp_action", "fmp.calendar", "calendar", &e),
                    }
                }
                if !sec_filings_enabled {
                    return;
                }
                let symbols = registry.load().watch_pool();
                if symbols.is_empty() {
                    return;
                }
                let mut sec_events = Vec::new();
                for sym in &symbols {
                    match poller.fetch_sec_filings(sym).await {
                        Ok(v) => sec_events.extend(v),
                        Err(e) => {
                            warn!(
                                poller = "sec_filings",
                                source = "fmp.sec_filings",
                                url_class = "per_symbol",
                                symbol = %sym,
                                degraded = true,
                                "poller fetch failed: {e:#}"
                            );
                        }
                    }
                }
                if !sec_events.is_empty() {
                    process_events("sec_filings", sec_events, &store, &router).await;
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        };
        cron_aligned_loop(
            "corp_action",
            tz_offset,
            pre_prefetch,
            post_prefetch,
            action,
        )
        .await;
    });
}

pub(crate) fn spawn_macro_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
) {
    tokio::spawn(async move {
        let poller = Arc::new(MacroPoller::new(client));
        let action = move || {
            let poller = poller.clone();
            let store = store.clone();
            let router = router.clone();
            Box::pin(async move {
                match poller.poll().await {
                    Ok(events) => process_events("macro", events, &store, &router).await,
                    Err(e) => log_poller_error("macro", "fmp.economic_calendar", "calendar", &e),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        };
        cron_aligned_loop("macro", tz_offset, pre_prefetch, post_prefetch, action).await;
    });
}

pub(crate) fn spawn_analyst_grade_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    registry: Arc<SharedRegistry>,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
) {
    tokio::spawn(async move {
        let poller = Arc::new(AnalystGradePoller::new(client));
        let action = move || {
            let poller = poller.clone();
            let store = store.clone();
            let router = router.clone();
            let registry = registry.clone();
            Box::pin(async move {
                let symbols = registry.load().watch_pool();
                if symbols.is_empty() {
                    return;
                }
                match poller.poll(&symbols).await {
                    Ok(events) => process_events("analyst_grade", events, &store, &router).await,
                    Err(e) => {
                        log_poller_error("analyst_grade", "fmp.analyst_grade", "per_symbol", &e)
                    }
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        };
        cron_aligned_loop(
            "analyst_grade",
            tz_offset,
            pre_prefetch,
            post_prefetch,
            action,
        )
        .await;
    });
}

pub(crate) fn spawn_earnings_surprise_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    registry: Arc<SharedRegistry>,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
) {
    tokio::spawn(async move {
        let poller = Arc::new(EarningsSurprisePoller::new(client));
        let action = move || {
            let poller = poller.clone();
            let store = store.clone();
            let router = router.clone();
            let registry = registry.clone();
            Box::pin(async move {
                let symbols = registry.load().watch_pool();
                if symbols.is_empty() {
                    return;
                }
                match poller.poll(&symbols).await {
                    Ok(events) => {
                        process_events("earnings_surprise", events, &store, &router).await
                    }
                    Err(e) => log_poller_error(
                        "earnings_surprise",
                        "fmp.earnings_surprise",
                        "per_symbol",
                        &e,
                    ),
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        };
        cron_aligned_loop(
            "earnings_surprise",
            tz_offset,
            pre_prefetch,
            post_prefetch,
            action,
        )
        .await;
    });
}

pub(crate) fn spawn_price_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    registry: Arc<SharedRegistry>,
    low_pct: f64,
    high_pct: f64,
    realert_step_pct: f64,
    interval: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut last_pool: Vec<String> = vec![];
        loop {
            ticker.tick().await;
            // 每 tick 都取最新快照；registry 热刷新会自动反映新增/变更的持仓。
            let symbols = registry.load().watch_pool();
            if symbols.is_empty() {
                continue;
            }
            if symbols != last_pool {
                info!(size = symbols.len(), "price watch pool updated");
                last_pool = symbols.clone();
            }
            let poller = PricePoller::new(client.clone())
                .with_symbols(symbols)
                .with_thresholds(low_pct, high_pct)
                .with_realert_step_pct(realert_step_pct);
            match poller.poll().await {
                Ok(events) => process_events("price", events, &store, &router).await,
                Err(e) => log_poller_error("price", "fmp.quote", "quote", &e),
            }
        }
    });
}

/// 通用事件源 spawn 入口:依据 `source.schedule()` 分发到 FixedInterval 或
/// CronAligned 两条循环。新增第三方监听源(Telegram / RSS / ...)只需
/// 实现 `EventSource` trait,调用一次本函数即可接入 store + router 主链路,
/// 不需要再复制 spawn/ticker/process_events 的模板。
///
/// 注:FMP 旧链路(7 个 `spawn_*_poller`)暂未迁移到此路径——它们 action
/// 语义更复杂(watch pool 过滤、compound fetch 等),暂保持原样,后续可选
/// 择性用 `FnSource` 包装迁移。
pub(crate) fn spawn_event_source(
    source: Arc<dyn EventSource>,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
) {
    let name: String = source.name().to_string();
    let schedule = source.schedule();
    tokio::spawn(async move {
        match schedule {
            SourceSchedule::FixedInterval(interval) => {
                // 冷启动立即拉一次,避免用户重启后等到下一 tick 才有数据。
                if let Err(e) = run_once(&name, &*source, &store, &router).await {
                    warn!(
                        poller = %name,
                        source = %name,
                        url_class = "event_source",
                        degraded = true,
                        "initial poll failed: {e:#}"
                    );
                }
                let mut ticker = tokio::time::interval(interval);
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                // 第一次 tick 立刻返回(已在 run_once 做过冷启动),跳过一次。
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    if let Err(e) = run_once(&name, &*source, &store, &router).await {
                        warn!(
                            poller = %name,
                            source = %name,
                            url_class = "event_source",
                            degraded = true,
                            "poll failed: {e:#}"
                        );
                    }
                }
            }
            SourceSchedule::CronAligned {
                pre_prefetch,
                post_prefetch,
                tz_offset,
            } => {
                // 冷启动先跑一次,然后每 60s 检查是否命中 pre/post 窗口。
                if let Err(e) = run_once(&name, &*source, &store, &router).await {
                    warn!(
                        poller = %name,
                        source = %name,
                        url_class = "event_source",
                        degraded = true,
                        "initial poll failed: {e:#}"
                    );
                }
                let mut ticker = tokio::time::interval(Duration::from_secs(60));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                let mut fired: std::collections::HashSet<String> = std::collections::HashSet::new();
                let mut last_date = String::new();
                loop {
                    ticker.tick().await;
                    let now = chrono::Utc::now();
                    let today = digest::local_date_key(now, tz_offset);
                    if today != last_date {
                        fired.clear();
                        last_date = today.clone();
                    }
                    for (label, hhmm) in [("pre", &pre_prefetch), ("post", &post_prefetch)] {
                        if !digest::in_window(now, hhmm, tz_offset) {
                            continue;
                        }
                        let key = format!("{today}@{label}@{hhmm}");
                        if !fired.insert(key) {
                            continue;
                        }
                        info!(poller = %name, window = label, hhmm = %hhmm, "cron-aligned source firing");
                        if let Err(e) = run_once(&name, &*source, &store, &router).await {
                            warn!(
                                poller = %name,
                                source = %name,
                                url_class = "event_source",
                                degraded = true,
                                "poll failed: {e:#}"
                            );
                        }
                    }
                }
            }
        }
    });
}
