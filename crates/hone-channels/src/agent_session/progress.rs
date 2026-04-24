//! `agent.run` 的「还在跑」心跳 ticker。
//!
//! 独立成一个 module 的理由:生产里的 `run_runner_with_progress_watchdog`
//! 和 watchdog live smoke 测试共用同一段 select + interval 逻辑,
//! 避免测试代码自己再复制一份走偏。

use std::time::{Duration, Instant};

/// 返回 `agent.run` progress 心跳 tick 间隔。生产默认 60s，可通过
/// `HONE_AGENT_RUN_PROGRESS_TICK_SECS` 环境变量覆盖（仅供真 LLM e2e 冒烟用，
/// 避免为了观察 ticker 行为等满一分钟）。最小值 1s，防止 0 导致 busy-loop。
pub fn progress_watchdog_tick() -> Duration {
    let secs = std::env::var("HONE_AGENT_RUN_PROGRESS_TICK_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|v| *v >= 1)
        .unwrap_or(60);
    Duration::from_secs(secs)
}

/// 在 `run_fut` 未完成期间，按 `tick` 间隔重复调用 `on_tick(ticks, elapsed)`。
/// 一旦 `run_fut` 返回，立刻转发其结果，停止触发 tick。
///
/// `on_tick` 返回一个 `Future`，在 select 分支里被 `.await`——因此 tick 闭包
/// 里可以安全做异步 emit（log_message_step 是同步的，可直接放在闭包体外的同步前置段）。
pub async fn run_with_progress_ticks<Fut, T, OnTick, TickFut>(
    run_fut: Fut,
    tick: Duration,
    mut on_tick: OnTick,
) -> T
where
    Fut: std::future::Future<Output = T>,
    OnTick: FnMut(u64, Duration) -> TickFut,
    TickFut: std::future::Future<Output = ()>,
{
    tokio::pin!(run_fut);
    let mut ticker = tokio::time::interval(tick);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // 消耗 interval 在 t=0 立刻发出的首个 tick，下一次 tick 在 `tick` 之后。
    ticker.tick().await;

    let started = Instant::now();
    let mut ticks: u64 = 0;
    loop {
        tokio::select! {
            result = &mut run_fut => return result,
            _ = ticker.tick() => {
                ticks += 1;
                on_tick(ticks, started.elapsed()).await;
            }
        }
    }
}
