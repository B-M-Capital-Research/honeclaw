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
    let (mut new_count, mut duplicate_count, mut sent_count, mut pending_digest_count) =
        (0u32, 0u32, 0u32, 0u32);
    for mut event in events {
        if let Err(error) = store.link_earnings_research_object(&mut event) {
            warn!(
                poller = name,
                event_id = %event.id,
                "earnings research object linking failed: {error:#}"
            );
        }
        let is_new = match store.insert_event(&event) {
            Ok(is_new) => is_new,
            Err(e) => {
                warn!(poller = name, "insert_event failed: {e:#}");
                continue;
            }
        };
        if is_new {
            new_count += 1;
            match router.dispatch(&event).await {
                Ok((dispatch_sent, dispatch_pending)) => {
                    sent_count += dispatch_sent;
                    pending_digest_count += dispatch_pending;
                }
                Err(e) => warn!(poller = name, "router dispatch failed: {e:#}"),
            }
        } else {
            duplicate_count += 1;
        }
    }
    info!(
        poller = name,
        total,
        new = new_count,
        duplicate = duplicate_count,
        sent = sent_count,
        pending_digest = pending_digest_count,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::TimeZone;
    use hone_core::ActorIdentity;
    use tempfile::tempdir;

    use crate::digest::DigestBuffer;
    use crate::event::{EventKind, Severity};
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
    use crate::router::{NotificationRouter, OutboundSink};
    use crate::store::DeliveryLogFilter;
    use crate::subscription::{PortfolioSubscription, SharedRegistry, SubscriptionRegistry};

    #[derive(Default)]
    struct CapturingSink {
        calls: Mutex<Vec<(String, String)>>,
    }

    #[async_trait]
    impl OutboundSink for CapturingSink {
        async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.calls.lock().unwrap().push((
                format!(
                    "{}::{}::{}",
                    actor.channel,
                    actor.channel_scope.clone().unwrap_or_default(),
                    actor.user_id
                ),
                body.to_string(),
            ));
            Ok(())
        }
    }

    struct FailingSink;

    #[async_trait]
    impl OutboundSink for FailingSink {
        async fn send(&self, _actor: &ActorIdentity, _body: &str) -> anyhow::Result<()> {
            anyhow::bail!("simulated sink failure")
        }
    }

    fn actor(user_id: &str) -> ActorIdentity {
        ActorIdentity::new("discord", user_id, None::<&str>).unwrap()
    }

    fn sec_release_event(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::SecFiling { form: "8-K".into() },
            severity: Severity::Medium,
            symbols: vec!["SNDK".into()],
            occurred_at: Utc.with_ymd_and_hms(2026, 8, 5, 20, 9, 6).unwrap(),
            title: "SNDK filed 8-K".into(),
            summary: "earnings release supporting document".into(),
            url: Some("https://www.sec.gov/Archives/sndkq4-26ex991xpressrelease.htm".into()),
            source: "fmp.sec_filings".into(),
            payload: serde_json::json!({
                "hone_earnings_release_document": true,
                "hone_earnings_release_document_key":
                    "https://www.sec.gov/archives/sndkq4-26ex991xpressrelease.htm"
            }),
        }
    }

    fn reviewed_earnings_event() -> MarketEvent {
        MarketEvent {
            id: "earnings_surprise:SNDK:2026-08-05".into(),
            kind: EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["SNDK".into()],
            occurred_at: Utc.with_ymd_and_hms(2026, 8, 5, 20, 9, 6).unwrap(),
            title: "数据中心强劲，消费端仍承压".into(),
            summary: "结论：数据中心驱动增长\n关键证据：订单增长；毛利率改善\n反向项：消费端下滑\n尚未确认：量价贡献\n后续核验：电话会核验客户采用".into(),
            url: Some(
                "https://www.sec.gov/Archives/sndkq4-26ex991xpressrelease.htm".into(),
            ),
            source: "fmp.earnings_surprises".into(),
            payload: serde_json::json!({
                "earnings_quality_review_applied": true,
                "earnings_quality_review": {"conclusion": "mixed_positive"},
                "hone_earnings_release_document_key":
                    "https://www.sec.gov/archives/sndkq4-26ex991xpressrelease.htm"
            }),
        }
    }

    #[tokio::test]
    async fn earnings_chain_is_personalized_idempotent_and_single_delivery_per_document() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        let mut registry = SubscriptionRegistry::new();
        for (user_id, mainline) in [
            ("ai", "AI 数据层与企业级 SSD 客户采用"),
            ("cycle", "NAND ASP、库存与供给纪律"),
        ] {
            let actor = actor(user_id);
            registry.register(Box::new(PortfolioSubscription::new(
                actor.clone(),
                vec!["SNDK".into()],
            )));
            prefs
                .save(
                    &actor,
                    &NotificationPrefs {
                        mainline_by_ticker: Some(HashMap::from([("SNDK".into(), mainline.into())])),
                        ..NotificationPrefs::default()
                    },
                )
                .unwrap();
        }
        let sink = Arc::new(CapturingSink::default());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(registry)),
            sink.clone(),
            store.clone(),
            digest.clone(),
        )
        .with_prefs(prefs);

        let first_sec = sec_release_event("sec:SNDK:release");
        process_events("fmp.sec_filings", vec![first_sec], &store, &router).await;
        process_events(
            "fmp.earnings_surprises",
            vec![reviewed_earnings_event()],
            &store,
            &router,
        )
        .await;
        // 同 id 重复轮询不再投递。
        process_events(
            "fmp.earnings_surprises",
            vec![reviewed_earnings_event()],
            &store,
            &router,
        )
        .await;

        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        let ai = calls.iter().find(|(key, _)| key.ends_with("::ai")).unwrap();
        let cycle = calls
            .iter()
            .find(|(key, _)| key.ends_with("::cycle"))
            .unwrap();
        assert!(ai.1.contains("AI 数据层"));
        assert!(!ai.1.contains("NAND ASP"));
        assert!(cycle.1.contains("NAND ASP"));
        assert!(!cycle.1.contains("AI 数据层"));
        drop(calls);

        // 财报卡成功送达后，先到的 SEC 兜底从摘要清除。
        let ai_remaining = digest.drain_actor(&actor("ai")).unwrap();
        let cycle_remaining = digest.drain_actor(&actor("cycle")).unwrap();
        assert!(ai_remaining.is_empty(), "ai remaining: {ai_remaining:?}");
        assert!(
            cycle_remaining.is_empty(),
            "cycle remaining: {cycle_remaining:?}"
        );

        // 后到的同文档 SEC 也不再重新入摘要。
        process_events(
            "fmp.sec_filings",
            vec![sec_release_event("sec:SNDK:release:amended")],
            &store,
            &router,
        )
        .await;
        assert!(digest.drain_actor(&actor("ai")).unwrap().is_empty());
        assert!(digest.drain_actor(&actor("cycle")).unwrap().is_empty());

        let superseded = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                status: Some("superseded".into()),
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(superseded.len(), 4);
        assert_eq!(store.count_events().unwrap(), 3);
    }

    #[tokio::test]
    async fn failed_structured_earnings_delivery_keeps_sec_digest_fallback() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let mut registry = SubscriptionRegistry::new();
        let target = actor("failed");
        registry.register(Box::new(PortfolioSubscription::new(
            target.clone(),
            vec!["SNDK".into()],
        )));
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(registry)),
            Arc::new(FailingSink),
            store.clone(),
            digest.clone(),
        );

        process_events(
            "fmp.sec_filings",
            vec![sec_release_event("sec:SNDK:fallback")],
            &store,
            &router,
        )
        .await;
        process_events(
            "fmp.earnings_surprises",
            vec![reviewed_earnings_event()],
            &store,
            &router,
        )
        .await;

        let buffered = digest.drain_actor(&target).unwrap();
        assert_eq!(buffered.len(), 1);
        assert_eq!(buffered[0].id, "sec:SNDK:fallback");
        let failures = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                actor: Some("discord::::failed".into()),
                status: Some("failed".into()),
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].event_id, "earnings_surprise:SNDK:2026-08-05");
    }
}
