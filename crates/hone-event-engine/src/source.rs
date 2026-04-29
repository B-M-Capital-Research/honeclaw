//! `EventSource` trait —— 事件源的统一抽象。
//!
//! 所有产出 `MarketEvent` 的上游(FMP poller / Telegram 频道 / RSS feed 等)
//! 都实现此 trait,由 [`crate::lib::spawn_event_source`] 按 `schedule()` 统一拉起。
//!
//! 关键设计:
//! - 调度策略(fixed interval vs cron-aligned)从 trait method 返回,而不是写死在
//!   各自的 `spawn_xxx_poller` 函数里——新增一个源只需实现 trait。
//! - `poll()` 失败由调用方 warn! 并在下一 tick 重试;trait 不规定重试策略。
//! - `name()` 返回的字符串会进 tracing 字段 `poller=...`,也是调用方辨识来源的 key。

use std::time::Duration;

use async_trait::async_trait;

use crate::event::MarketEvent;

/// 事件源调度方式。
///
/// - `FixedInterval`:冷启动立即拉一次,之后每 `duration` 周期拉一次。适合
///   持续有新内容的源(FMP news/price、Telegram 频道、RSS feed)。
/// - `CronAligned`:冷启动立即拉一次,之后每 60s 检查本地时刻是否命中
///   `prefetch_at` 列表中的任一时刻(`"HH:MM"` 格式,按 `tz_offset` 做本地化)。
///   适合按交易所开收盘时间对齐的源(FMP earnings/macro/corp_action 等)。
#[derive(Debug, Clone)]
pub enum SourceSchedule {
    FixedInterval(Duration),
    CronAligned {
        prefetch_at: Vec<String>,
        tz_offset: i32,
    },
}

#[async_trait]
pub trait EventSource: Send + Sync {
    /// 稳定标识,形如 `"fmp.news"` / `"telegram.watcherguru"` /
    /// `"rss:bloomberg_markets"`。进日志 `poller` 字段;不用于事件 id。
    fn name(&self) -> &str;

    /// 调度策略,由 `spawn_event_source` 决定用哪条循环跑。
    fn schedule(&self) -> SourceSchedule;

    /// 单次拉取——返回尚未去重的 `MarketEvent` 列表。实现方应保证 id 幂等
    /// (再次 poll 同条内容产出相同 id),由 `EventStore::insert_event` 去重。
    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>>;
}

/// 闭包驱动的 `EventSource` 实现——给 FMP pollers 等已有逻辑做零侵入包装。
///
/// 用法:把现有 `async fn ...() -> Result<Vec<MarketEvent>>` 包成闭包,
/// 和 `name` / `schedule` 一起构造,即可作为 `Arc<dyn EventSource>`
/// 送入 `spawn_event_source`。
///
/// 闭包必须 `Fn`(不是 `FnOnce`/`FnMut`),因为调度器会在每个 tick 调用。
/// 返回的 Future 必须 `Send`,以便 tokio::spawn。
pub struct FnSource<F> {
    name: String,
    schedule: SourceSchedule,
    poll_fn: F,
}

impl<F, Fut> FnSource<F>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<Vec<MarketEvent>>> + Send + 'static,
{
    pub fn new(name: impl Into<String>, schedule: SourceSchedule, poll_fn: F) -> Self {
        Self {
            name: name.into(),
            schedule,
            poll_fn,
        }
    }
}

#[async_trait]
impl<F, Fut> EventSource for FnSource<F>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<Vec<MarketEvent>>> + Send + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }
    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        (self.poll_fn)().await
    }
}
