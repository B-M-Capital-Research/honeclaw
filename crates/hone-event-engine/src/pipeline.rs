//! Poller → store → router 的共享「一次处理 + 日志」小流水 +
//! 内部周期任务的「分钟级跨日 tick」骨架。
//!
//! 主要内容:
//! - `process_events` / `run_once`:被 `spawn_event_source` 调用,完成
//!   「插 store 去重 → 新事件派发 → 计数日志」
//! - `cron_minute_tick`:抽出 digest_scheduler / daily_report 共享的
//!   「60s ticker + fired HashSet + 跨日 reset」骨架,caller 只关心
//!   「拿到 now 和 fired 后干嘛」
//!
//! 这个文件没有 `tokio::spawn`,只做「命中后怎么跑一次」;
//! 具体的 spawn 任务树在 sibling `spawner.rs` 里。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use hone_core::task_observer;
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

/// 「60s ticker + fired HashSet + 跨日 reset」的通用骨架。
///
/// 适用对象:digest_scheduler / daily_report 这种「每分钟检查一次本地时间,
/// 命中某个 HH:MM 窗口就触发一次,同窗口同日不重复触发,跨日清空 fired 集合」
/// 的 internal task。
///
/// `on_tick` 回调拿到 `now` 和可写的 `fired` HashSet:
/// - 自己用 `digest::in_window(now, hhmm, tz_offset)` 判断窗口
/// - 用 `format!("{date}@{label}@{hhmm}")` 当 fired key,`fired.insert` 返回 false
///   说明本窗口本日已触发,跳过(可参考 `digest::scheduler::tick_once` 现有写法)
///
/// 失败只 `warn!(task = name)`,不上抛,不重试——下一 tick 自动恢复
/// (Tier-A,见 docs/conventions/periodic_tasks.md)。
///
/// 注:返回类型用 `Pin<Box<dyn Future + Send + 'a>>` 而不是裸 `impl Future`,
/// 是因为闭包内的 future 借走了 `&'a mut fired`,需要把 lifetime 显式带到
/// future trait object 里——Rust async closure 借用 `&mut` 参数的标准模式。
pub(crate) async fn cron_minute_tick<F>(
    name: &'static str,
    tz_offset: i32,
    task_runs_dir: Option<Arc<PathBuf>>,
    mut on_tick: F,
) where
    F: for<'a> FnMut(
            DateTime<Utc>,
            &'a mut HashSet<String>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>,
        > + Send
        + 'static,
{
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut fired: HashSet<String> = HashSet::new();
    let mut last_date = String::new();
    loop {
        ticker.tick().await;
        let now = Utc::now();
        let today = digest::local_date_key(now, tz_offset);
        if today != last_date {
            fired.clear();
            last_date = today;
        }
        let started_at = Utc::now();
        let pre_size = fired.len();
        match on_tick(now, &mut fired).await {
            Ok(()) => {
                if let Some(dir) = task_runs_dir.as_deref() {
                    // 触发与否用 fired HashSet 增量推断:本 tick 进入 fired 的 key 数 = items
                    // (每命中一个窗口会 insert 一次)。0 → outcome=skipped(本分钟没命中
                    // 任何窗口)。
                    let items = (fired.len() - pre_size) as u64;
                    if items > 0 {
                        task_observer::record_ok(dir, name, started_at, items);
                    } else {
                        task_observer::record_skipped(dir, name, started_at);
                    }
                }
            }
            Err(e) => {
                warn!(task = name, "tick failed: {e:#}");
                if let Some(dir) = task_runs_dir.as_deref() {
                    task_observer::record_failed(dir, name, started_at, &format!("{e:#}"));
                }
            }
        }
    }
}
