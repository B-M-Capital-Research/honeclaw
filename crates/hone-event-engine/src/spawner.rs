//! `spawn_event_source` —— 唯一的 EventSource 任务驱动入口。
//!
//! Stage 1 重构完成后(2026-04-26),所有 FMP / RSS / 社交源都用同一条
//! 路径接入:`impl EventSource` → 调一次 `spawn_event_source(Arc::new(poller),
//! store, router)` 即可。FixedInterval / CronAligned 两种调度策略由
//! `source.schedule()` 返回值决定,本函数内嵌两条循环,不再有专属
//! `spawn_*_poller` 函数。
//!
//! 内部细节:
//! - 冷启动立即拉一次,避免用户重启 Hone 后等到下一 tick 才有数据
//! - `MissedTickBehavior::Delay` 防止长任务恢复后突发风暴
//! - CronAligned 分支跨日时清空 `fired` HashSet 避免内存堆积
//! - 失败只 `warn!` 不上抛(Tier-A 失败处理,见 docs/conventions/periodic_tasks.md)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use hone_core::task_observer;
use tracing::{info, warn};

use crate::digest;
use crate::pipeline::run_once;
use crate::router::NotificationRouter;
use crate::source::{EventSource, SourceSchedule};
use crate::store::EventStore;

/// 跑一次 source.poll() 并把成败写入 task_runs.jsonl(若提供了 dir)。
/// 抽出来避免 FixedInterval / CronAligned 两条循环重复观测样板。
async fn run_once_observed(
    task: &str,
    source: &dyn EventSource,
    store: &EventStore,
    router: &NotificationRouter,
    task_runs_dir: Option<&PathBuf>,
) {
    let started_at = Utc::now();
    match run_once(task, source, store, router).await {
        Ok(()) => {
            if let Some(dir) = task_runs_dir {
                // run_once 内部自己 process_events,不返回 items 数;落盘只标 outcome=ok。
                // 真实条数已经在 process_events 的 tracing 行里(total= / new= / sent=),
                // task_runs.jsonl 的角色是"任务级心跳",不重复存事件级数据。
                task_observer::record_ok(dir, task, started_at, 0);
            }
        }
        Err(e) => {
            warn!(
                poller = %task,
                source = %task,
                url_class = "event_source",
                degraded = true,
                "poll failed: {e:#}"
            );
            if let Some(dir) = task_runs_dir {
                task_observer::record_failed(dir, task, started_at, &format!("{e:#}"));
            }
        }
    }
}

pub(crate) fn spawn_event_source(
    source: Arc<dyn EventSource>,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    task_runs_dir: Option<Arc<PathBuf>>,
) {
    let name: String = source.name().to_string();
    let task_label = format!("poller.{name}");
    let schedule = source.schedule();
    tokio::spawn(async move {
        let dir_ref = task_runs_dir.as_deref();
        match schedule {
            SourceSchedule::FixedInterval(interval) => {
                run_once_observed(&task_label, &*source, &store, &router, dir_ref).await;
                let mut ticker = tokio::time::interval(interval);
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                // 第一次 tick 立刻返回(已在 run_once_observed 做过冷启动),跳过一次。
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    run_once_observed(&task_label, &*source, &store, &router, dir_ref).await;
                }
            }
            SourceSchedule::CronAligned {
                prefetch_at,
                tz_offset,
            } => {
                run_once_observed(&task_label, &*source, &store, &router, dir_ref).await;
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
                    for hhmm in &prefetch_at {
                        if !digest::in_window(now, hhmm, tz_offset) {
                            continue;
                        }
                        let key = format!("{today}@{hhmm}");
                        if !fired.insert(key) {
                            continue;
                        }
                        info!(poller = %name, hhmm = %hhmm, "cron-aligned source firing");
                        run_once_observed(&task_label, &*source, &store, &router, dir_ref).await;
                    }
                }
            }
        }
    });
}
