//! Poller → store → router 的共享「一次处理 + 日志」小流水。
//!
//! 分离到独立文件的理由:
//! - 7 个 FMP poller + 通用 `EventSource` poller 都在复用同一段
//!   「插 store 去重 → 新事件派发 → 计数日志」逻辑(`process_events`);
//! - `cron_aligned_loop` 是 5 个日频 poller 共用的时钟外壳;
//! - `log_poller_error` / `run_once` 是两个刻意短小的胶水函数。
//!
//! 保持这个文件没有 `tokio::spawn`,只做「命中后怎么跑一次」;
//! 具体的 spawn 任务树在 sibling `spawner.rs` 里。

use std::time::Duration;

use tracing::{info, warn};

use crate::digest;
use crate::event::MarketEvent;
use crate::router::NotificationRouter;
use crate::source::EventSource;
use crate::store::EventStore;

/// 把一批事件写入 store 去重，然后交给 router 分发。返回 (new, duplicate, sent, pending)。
pub(crate) async fn process_events(
    name: &str,
    events: Vec<MarketEvent>,
    store: &EventStore,
    router: &NotificationRouter,
) {
    let total = events.len();
    // 按 tick 重置 per-symbol 升级计数,使新一批事件不会受到上一 tick 残留计数干扰。
    router.reset_tick_counters();
    let (mut new_cnt, mut dup_cnt, mut sent, mut pending) = (0u32, 0u32, 0u32, 0u32);
    for ev in &events {
        let is_new = match store.insert_event(ev) {
            Ok(is_new) => is_new,
            Err(e) => {
                warn!(poller = name, "insert_event failed: {e:#}");
                continue;
            }
        };
        if is_new {
            new_cnt += 1;
            match router.dispatch(ev).await {
                Ok((s, p)) => {
                    sent += s;
                    pending += p;
                }
                Err(e) => warn!(poller = name, "router dispatch failed: {e:#}"),
            }
        } else {
            dup_cnt += 1;
        }
    }
    info!(
        poller = name,
        total,
        new = new_cnt,
        duplicate = dup_cnt,
        sent,
        pending_digest = pending,
        "poller ok"
    );
    let news_stats = router.news_upgrade_tick_stats_snapshot();
    if news_stats.has_activity() {
        info!(
            poller = name,
            upgraded = news_stats.upgraded,
            skipped_per_tick_cap = news_stats.skipped_per_tick_cap,
            skipped_per_symbol_cap = news_stats.skipped_per_symbol_cap,
            triggers = ?news_stats.trigger_counts,
            top_symbols = ?news_stats.top_symbols(5),
            "news window convergence summary"
        );
    }
}

pub(crate) fn log_poller_error(name: &str, source: &str, url_class: &str, error: &anyhow::Error) {
    warn!(
        poller = name,
        source,
        url_class,
        degraded = true,
        "poller fetch failed: {error:#}"
    );
}

/// 通用 cron-aligned 循环:冷启动立即跑一次,然后每 60s 检查是否命中
/// `pre_prefetch` / `post_prefetch` 对应的本地时刻(60s 分辨率由 `in_window` 保证)。
/// `already_fired` 每日清空避免同分钟重复触发。
///
/// `action` 是 async closure:每次命中窗口就被 await 一次。签名用 `Fn` + 返回
/// `BoxFuture` 以便调用方捕获共享 Arc。
pub(crate) async fn cron_aligned_loop(
    name: &'static str,
    tz_offset: i32,
    pre_prefetch: String,
    post_prefetch: String,
    action: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    + Send
    + 'static,
) {
    // 冷启动:先跑一次,不等待 cron 窗口——保证用户重启 Hone 后马上能看到最新 teaser。
    action().await;
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
            info!(poller = name, window = label, hhmm = %hhmm, "cron-aligned poller firing");
            action().await;
        }
    }
}

/// `spawn_event_source` 内部复用的单次拉取 + 分发小工具,包一层把 poll 错误
/// 抽成 Result 供上层决定 warn! 粒度。
pub(crate) async fn run_once(
    name: &str,
    source: &dyn EventSource,
    store: &EventStore,
    router: &NotificationRouter,
) -> anyhow::Result<()> {
    let events = source.poll().await?;
    process_events(name, events, store, router).await;
    Ok(())
}
