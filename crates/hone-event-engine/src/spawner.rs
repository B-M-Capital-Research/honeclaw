//! `spawn_event_source` —— 唯一的 EventSource 任务驱动入口。
//!
//! 所有 FMP / RSS / 社交源都用同一条路径接入:`impl EventSource` → 调一次
//! `spawn_event_source(Arc::new(poller), store, router)` 即可。FixedInterval / CronAligned 两种调度策略由
//! `source.schedule()` 返回值决定,本函数内嵌两条循环,不再有专属
//! `spawn_*_poller` 函数。
//!
//! 内部细节:
//! - 冷启动立即拉一次,避免用户重启 Hone 后等到下一 tick 才有数据
//! - `MissedTickBehavior::Delay` 防止长任务恢复后突发风暴
//! - CronAligned 分支跨日时清空 `fired` HashSet 避免内存堆积
//! - 失败只 `warn!` 不上抛(Tier-A 失败处理,见 docs/conventions/periodic_tasks.md)

use std::future::Future;
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

const MIN_POLLER_TICK_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_POLLER_TICK_TIMEOUT: Duration = Duration::from_secs(300);

fn clamp_duration(value: Duration, min: Duration, max: Duration) -> Duration {
    value.max(min).min(max)
}

fn poller_tick_timeout(schedule: &SourceSchedule) -> Duration {
    match schedule {
        SourceSchedule::FixedInterval(interval) => clamp_duration(
            *interval * 2,
            MIN_POLLER_TICK_TIMEOUT,
            MAX_POLLER_TICK_TIMEOUT,
        ),
        SourceSchedule::CronAligned { .. } => MAX_POLLER_TICK_TIMEOUT,
    }
}

async fn with_poller_timeout<F>(task: &str, timeout: Duration, fut: F) -> anyhow::Result<()>
where
    F: Future<Output = anyhow::Result<()>>,
{
    match tokio::time::timeout(timeout, fut).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!(
            "poller tick timed out after {}s: {task}",
            timeout.as_secs_f64()
        ),
    }
}

/// 跑一次 source.poll() 并把成败写入 task_runs.jsonl(若提供了 dir)。
/// 抽出来避免 FixedInterval / CronAligned 两条循环重复观测样板。
async fn run_once_observed(
    task: &str,
    timeout: Duration,
    source: &dyn EventSource,
    store: &EventStore,
    router: &NotificationRouter,
    task_runs_dir: Option<&PathBuf>,
) {
    let started_at = Utc::now();
    match with_poller_timeout(task, timeout, run_once(task, source, store, router)).await {
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
        let tick_timeout = poller_tick_timeout(&schedule);
        match schedule {
            SourceSchedule::FixedInterval(interval) => {
                run_once_observed(
                    &task_label,
                    tick_timeout,
                    &*source,
                    &store,
                    &router,
                    dir_ref,
                )
                .await;
                let mut ticker = tokio::time::interval(interval);
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                // 第一次 tick 立刻返回(已在 run_once_observed 做过冷启动),跳过一次。
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    run_once_observed(
                        &task_label,
                        tick_timeout,
                        &*source,
                        &store,
                        &router,
                        dir_ref,
                    )
                    .await;
                }
            }
            SourceSchedule::CronAligned {
                prefetch_at,
                tz_offset,
            } => {
                run_once_observed(
                    &task_label,
                    tick_timeout,
                    &*source,
                    &store,
                    &router,
                    dir_ref,
                )
                .await;
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
                        run_once_observed(
                            &task_label,
                            tick_timeout,
                            &*source,
                            &store,
                            &router,
                            dir_ref,
                        )
                        .await;
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poller_tick_timeout_is_bounded_by_schedule() {
        assert_eq!(
            poller_tick_timeout(&SourceSchedule::FixedInterval(Duration::from_secs(5))),
            MIN_POLLER_TICK_TIMEOUT
        );
        assert_eq!(
            poller_tick_timeout(&SourceSchedule::FixedInterval(Duration::from_secs(60))),
            Duration::from_secs(120)
        );
        assert_eq!(
            poller_tick_timeout(&SourceSchedule::FixedInterval(Duration::from_secs(600))),
            MAX_POLLER_TICK_TIMEOUT
        );
        assert_eq!(
            poller_tick_timeout(&SourceSchedule::CronAligned {
                prefetch_at: vec!["09:25".to_string()],
                tz_offset: 8,
            }),
            MAX_POLLER_TICK_TIMEOUT
        );
    }

    #[tokio::test]
    async fn with_poller_timeout_turns_stuck_tick_into_error() {
        let err = with_poller_timeout("poller.test", Duration::from_millis(5), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        })
        .await
        .expect_err("stuck tick should time out");

        assert!(err.to_string().contains("poller tick timed out"));
        assert!(err.to_string().contains("poller.test"));
    }
}
