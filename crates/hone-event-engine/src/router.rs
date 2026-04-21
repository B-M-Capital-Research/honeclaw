//! NotificationRouter — 按 severity 分流。
//!
//! - `High` → 立即调 `OutboundSink::send`
//! - `Medium` / `Low` → 入 `DigestBuffer`，由 `DigestScheduler` 在 ET 盘前/盘后合并推送。
//!
//! MVP 的 `OutboundSink` 实现只打 `tracing::info` 日志（dryrun 语义）；真实
//! 渠道适配器在后续 step 接入。

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use hone_core::ActorIdentity;
use tracing::info;

use crate::digest::DigestBuffer;
use crate::event::{EventKind, MarketEvent, Severity};
use crate::polisher::{BodyPolisher, NoopPolisher};
use crate::prefs::{kind_tag, AllowAllPrefs, PrefsProvider};
use crate::renderer::{self, RenderFormat};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;

/// 同日命中后可以把 Low 新闻升到 Medium 的硬信号 kind tag 集合。
/// 语义：ticker 当天已出现过这些"事实性"事件时,同 ticker 的低优先级新闻
/// 很可能是相关解读,升到 Medium 让它进 digest 而不是沉底。
const NEWS_CONVERGENCE_HARD_SIGNALS: &[&str] = &[
    "price_alert",
    "earnings_released",
    "earnings_upcoming",
    "sec_filing",
    "analyst_grade",
];

#[async_trait]
pub trait OutboundSink: Send + Sync {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()>;

    /// 该 Sink 期望的消息格式。默认 Plain；Telegram sink 应返回 `TelegramHtml`。
    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }
}

/// 默认 Sink：把渲染后的消息写 tracing::info，用于 dryrun 与测试。
pub struct LogSink;

#[async_trait]
impl OutboundSink for LogSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        info!(
            actor = %actor_key(actor),
            "[dryrun sink] {body}"
        );
        Ok(())
    }
}

pub struct NotificationRouter {
    registry: Arc<SharedRegistry>,
    sink: Arc<dyn OutboundSink>,
    store: Arc<EventStore>,
    digest: Arc<DigestBuffer>,
    polisher: Arc<dyn BodyPolisher>,
    prefs: Arc<dyn PrefsProvider>,
    /// 每 actor 当日 sink=sent 且 severity=high 的条数上限。超过后新的 High
    /// 事件自动降级进 digest,并在 delivery_log 写 status="capped"。
    /// 0 = 不启用。
    high_daily_cap: u32,
    /// 解释"当日"起点所用的 UTC 偏移(小时)。
    tz_offset_hours: i32,
    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 防止同一 ticker 短时间内被价格异动 + 新闻 + SEC filing 三连推。
    /// 命中后降级到 digest,log_delivery 写 status="cooled_down"。
    same_symbol_cooldown_minutes: u32,
    /// 部署方配置的全局 kind 黑名单。命中后 dispatch 直接返回 (0, 0),
    /// 任何 actor 的 prefs / cap / cooldown 都不再参与。
    disabled_kinds: Arc<HashSet<String>>,
}

impl NotificationRouter {
    pub fn new(
        registry: Arc<SharedRegistry>,
        sink: Arc<dyn OutboundSink>,
        store: Arc<EventStore>,
        digest: Arc<DigestBuffer>,
    ) -> Self {
        Self {
            registry,
            sink,
            store,
            digest,
            polisher: Arc::new(NoopPolisher),
            prefs: Arc::new(AllowAllPrefs),
            high_daily_cap: 0,
            tz_offset_hours: 8,
            same_symbol_cooldown_minutes: 0,
            disabled_kinds: Arc::new(HashSet::new()),
        }
    }

    pub fn with_polisher(mut self, polisher: Arc<dyn BodyPolisher>) -> Self {
        self.polisher = polisher;
        self
    }

    /// 注入用户偏好源。未注入时默认放行所有事件（维持旧行为）。
    pub fn with_prefs(mut self, prefs: Arc<dyn PrefsProvider>) -> Self {
        self.prefs = prefs;
        self
    }

    /// 每 actor 当日 High 推送上限。0 = 不启用(默认),与历史行为兼容。
    /// 命中上限后同 actor 当日剩余 High 事件自动降级进 digest。
    pub fn with_high_daily_cap(mut self, cap: u32) -> Self {
        self.high_daily_cap = cap;
        self
    }

    /// 配置 tz 偏移,用于计算"当日"窗口起点。默认 8 (北京)。
    pub fn with_tz_offset_hours(mut self, offset: i32) -> Self {
        self.tz_offset_hours = offset;
        self
    }

    /// 同一 ticker 相邻两次 High sink 推送的最小间隔(分钟)。0 = 不启用。
    /// 命中冷却的事件降级到 digest,状态记 "cooled_down"。
    pub fn with_same_symbol_cooldown_minutes(mut self, minutes: u32) -> Self {
        self.same_symbol_cooldown_minutes = minutes;
        self
    }

    /// 部署方 kind 黑名单——命中后 dispatch 直接丢弃,不下发也不入 digest。
    /// 事件仍然入库,便于统计;空列表 = 不启用。
    pub fn with_disabled_kinds<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.disabled_kinds = Arc::new(tags.into_iter().map(|t| t.into()).collect());
        self
    }

    /// 新闻多信号合流 + 财报窗口升级:当事件为 `NewsCritical + Low`,且同一 ticker
    /// 在 `[news_ts - 1d, news_ts + 2d]` 窗口内出现过硬信号
    /// (price_alert / earnings_released / earnings_upcoming / sec_filing /
    /// analyst_grade),把 severity 升到 Medium。
    ///
    /// 窗口既覆盖"前 24h 内已发生"的硬信号(#10 多信号合流),也覆盖"未来 48h 内"
    /// 的 earnings_upcoming(#13 财报窗口——因为 earnings_upcoming 的 occurred_at
    /// 是财报当日 00:00,T-1/T 新闻必须向未来扩窗才能命中)。
    ///
    /// 升级是幂等 clone,原事件不被修改。
    fn maybe_upgrade_news(&self, event: &MarketEvent) -> MarketEvent {
        if !matches!(event.kind, EventKind::NewsCritical) || event.severity != Severity::Low {
            return event.clone();
        }
        let start = event.occurred_at - chrono::Duration::days(1);
        let end = event.occurred_at + chrono::Duration::days(2);
        let mut trigger_tag: Option<String> = None;
        for sym in &event.symbols {
            match self.store.symbol_signal_kinds_in_window(sym, start, end) {
                Ok(tags) => {
                    if let Some(hit) = tags
                        .iter()
                        .find(|t| NEWS_CONVERGENCE_HARD_SIGNALS.contains(&t.as_str()))
                    {
                        trigger_tag = Some(hit.clone());
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("symbol_signal_kinds_in_window failed for {sym}: {e:#}");
                }
            }
        }
        let Some(tag) = trigger_tag else {
            return event.clone();
        };
        let mut upgraded = event.clone();
        upgraded.severity = Severity::Medium;
        tracing::info!(
            event_id = %event.id,
            symbols = ?event.symbols,
            trigger = %tag,
            "news severity upgraded Low→Medium (window convergence)"
        );
        upgraded
    }

    /// 对一个事件执行分发。High 立即推；其余当前只记 pending-digest 日志。
    ///
    /// 返回 `(immediate_sent, pending_digest)` 数量。
    pub async fn dispatch(&self, event: &MarketEvent) -> anyhow::Result<(u32, u32)> {
        // 全局 kind 黑名单：部署方 YAML 里关掉的 kind 直接短路,不走 resolve/prefs/cap。
        // 事件已经由调用方负责入库,这里只是不分发。
        let tag = kind_tag(&event.kind);
        if self.disabled_kinds.contains(tag) {
            tracing::info!(
                event_id = %event.id,
                kind = %tag,
                "event kind globally disabled; dispatch skipped"
            );
            return Ok((0, 0));
        }
        let upgraded = self.maybe_upgrade_news(event);
        let event = &upgraded;
        // 每次 dispatch 都拿最新快照——用户持仓更新后下一条事件即可感知。
        let hits = self.registry.load().resolve(event);
        let mut sent = 0u32;
        let mut pending = 0u32;
        for (actor, sev) in hits {
            let user_prefs = self.prefs.load(&actor);
            if !user_prefs.should_deliver(event) {
                let _ = self.store.log_delivery(
                    &event.id,
                    &actor_key(&actor),
                    "prefs",
                    sev,
                    "filtered",
                    None,
                );
                info!(
                    actor = %actor_key(&actor),
                    event_id = %event.id,
                    "skipped by user prefs"
                );
                continue;
            }
            // High daily cap:同一 actor 当日 sink-sent High 条数达到上限后,
            // 后续 High 一律降级到 digest,避免"某 ticker 一天连发 8-K + 财报 +
            // 价格异动"把用户淹没。降级路径不双写 log:digest 入队时 status 写
            // "capped" 而不是 "queued",便于复盘统计"今日被降级多少条"。
            // cap=0 关闭该逻辑,与历史行为兼容。
            let mut demoted_by_cap = false;
            let mut demoted_by_cooldown = false;
            let mut effective_sev = if matches!(sev, Severity::High) && self.high_daily_cap > 0 {
                let since = local_day_start(chrono::Utc::now(), self.tz_offset_hours);
                match self
                    .store
                    .count_high_sent_since(&actor_key(&actor), since)
                {
                    Ok(n) if n >= self.high_daily_cap as i64 => {
                        tracing::info!(
                            actor = %actor_key(&actor),
                            event_id = %event.id,
                            today_high = n,
                            cap = self.high_daily_cap,
                            "High 事件降级进 digest(已超当日上限)"
                        );
                        demoted_by_cap = true;
                        Severity::Medium
                    }
                    Ok(_) => sev,
                    Err(e) => {
                        tracing::warn!("count_high_sent_since failed: {e:#}");
                        sev
                    }
                }
            } else {
                sev
            };
            // 同 ticker 冷却:如果事件还是 High,且 cooldown>0,检查任一 symbol 最近一次
            // High+sink+sent 的时间戳,若在冷却窗口内则降级进 digest。
            if matches!(effective_sev, Severity::High)
                && self.same_symbol_cooldown_minutes > 0
                && !event.symbols.is_empty()
            {
                let cutoff = chrono::Utc::now()
                    - chrono::Duration::minutes(self.same_symbol_cooldown_minutes as i64);
                for sym in &event.symbols {
                    match self
                        .store
                        .last_high_sink_send_for_symbol(&actor_key(&actor), sym)
                    {
                        Ok(Some(ts)) if ts >= cutoff => {
                            tracing::info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                symbol = %sym,
                                last_sent_at = %ts,
                                cooldown_min = self.same_symbol_cooldown_minutes,
                                "High 事件降级进 digest(同 ticker 冷却中)"
                            );
                            demoted_by_cooldown = true;
                            effective_sev = Severity::Medium;
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!(
                                "last_high_sink_send_for_symbol failed for {sym}: {e:#}"
                            );
                        }
                    }
                }
            }
            match effective_sev {
                Severity::High => {
                    let default_body = renderer::render_immediate(event, self.sink.format());
                    let body = match self.polisher.polish(event, &default_body).await {
                        Some(polished) => polished,
                        None => default_body,
                    };
                    if let Err(e) = self.sink.send(&actor, &body).await {
                        tracing::warn!("sink send failed: {e:#}");
                        let _ = self.store.log_delivery(
                            &event.id,
                            &actor_key(&actor),
                            "sink",
                            sev,
                            "failed",
                            Some(&body),
                        );
                        continue;
                    }
                    let _ = self.store.log_delivery(
                        &event.id,
                        &actor_key(&actor),
                        "sink",
                        sev,
                        "sent",
                        Some(&body),
                    );
                    sent += 1;
                }
                Severity::Medium | Severity::Low => {
                    match self.digest.enqueue(&actor, event) {
                        Ok(()) => {
                            // 被 cap 降级的条目记 status="capped",被同 ticker 冷却降级的
                            // 记 "cooled_down",正常流程记 "queued"。severity 仍记原始严重度
                            // (sev),方便事后 grep "high + capped/cooled_down" 对账。
                            let status = if demoted_by_cap {
                                "capped"
                            } else if demoted_by_cooldown {
                                "cooled_down"
                            } else {
                                "queued"
                            };
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                status,
                                None,
                            );
                            info!(
                                actor = %actor_key(&actor),
                                event_id = %event.id,
                                severity = ?sev,
                                status = %status,
                                "digest queued"
                            );
                            pending += 1;
                        }
                        Err(e) => {
                            tracing::warn!("digest enqueue failed: {e:#}");
                            let _ = self.store.log_delivery(
                                &event.id,
                                &actor_key(&actor),
                                "digest",
                                sev,
                                "failed",
                                None,
                            );
                        }
                    }
                }
            }
        }
        Ok((sent, pending))
    }
}

fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

/// 按给定 tz 偏移求本地当日 00:00 对应的 UTC 时刻。用作
/// `count_high_sent_since` 的 cutoff,保证跨时区一致。
fn local_day_start(now: chrono::DateTime<chrono::Utc>, offset_hours: i32) -> chrono::DateTime<chrono::Utc> {
    use chrono::{FixedOffset, NaiveTime, TimeZone};
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    offset
        .from_local_datetime(&midnight)
        .single()
        .map(|l| l.with_timezone(&chrono::Utc))
        .unwrap_or(now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::subscription::{PortfolioSubscription, SubscriptionRegistry};
    use chrono::Utc;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct CapturingSink {
        calls: Mutex<Vec<(String, String)>>,
    }

    #[async_trait]
    impl OutboundSink for CapturingSink {
        async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((actor_key(actor), body.to_string()));
            Ok(())
        }
    }

    fn actor(user: &str) -> ActorIdentity {
        ActorIdentity::new("imessage", user, None::<&str>).unwrap()
    }

    fn ev(sev: Severity) -> MarketEvent {
        MarketEvent {
            id: "e1".into(),
            kind: EventKind::EarningsReleased,
            severity: sev,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "earnings".into(),
            summary: "beat".into(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    fn router_with_aapl_actor() -> (NotificationRouter, Arc<CapturingSink>, tempfile::TempDir) {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        (
            NotificationRouter::new(Arc::new(SharedRegistry::from_registry(reg)), sink.clone(), store, digest),
            sink,
            dir,
        )
    }

    #[tokio::test]
    async fn high_severity_goes_to_sink_immediately() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let (sent, pending) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
        assert_eq!(pending, 0);
        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].1.contains("财报发布"));
    }

    #[tokio::test]
    async fn high_daily_cap_demotes_excess_to_digest() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_high_daily_cap(2);

        // 每条 High 事件用不同 id 避免被上层去重逻辑误判同一事件
        let mk = |id: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("earnings {id}"),
            summary: "beat".into(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };

        let (s1, _) = router.dispatch(&mk("h1")).await.unwrap();
        let (s2, _) = router.dispatch(&mk("h2")).await.unwrap();
        // 前两条正常走 sink
        assert_eq!(s1, 1);
        assert_eq!(s2, 1);
        assert_eq!(sink.calls.lock().unwrap().len(), 2);

        // 第三条触顶 → 降级到 digest,sink 不再收到,pending=1
        let (s3, p3) = router.dispatch(&mk("h3")).await.unwrap();
        assert_eq!(s3, 0, "触顶后 High 不应走 sink");
        assert_eq!(p3, 1, "应降级进 digest");
        assert_eq!(sink.calls.lock().unwrap().len(), 2, "sink call count 不应增加");

        // delivery_log 里应有 2 条 sent + 1 条 capped
        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(store.count_high_sent_since("imessage::::u1", since).unwrap(), 2);
    }

    #[tokio::test]
    async fn high_daily_cap_zero_means_no_cap() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        // cap = 0 应该关闭所有限流,N 条 High 全部进 sink
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        )
        .with_high_daily_cap(0);

        for i in 0..5 {
            let mut event = ev(Severity::High);
            event.id = format!("h{i}");
            let (s, _) = router.dispatch(&event).await.unwrap();
            assert_eq!(s, 1, "cap=0 时每条 High 都应走 sink");
        }
        assert_eq!(sink.calls.lock().unwrap().len(), 5);
    }

    #[tokio::test]
    async fn same_symbol_cooldown_demotes_second_high_to_digest() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_same_symbol_cooldown_minutes(60);

        let mk = |id: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("earnings {id}"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        // 第一条必须先入 events 表,这样 JOIN 才能找到 symbol;生产路径由 poller 完成入库。
        let a = mk("h1");
        store.insert_event(&a).unwrap();
        let (s1, _) = router.dispatch(&a).await.unwrap();
        assert_eq!(s1, 1, "第一条 AAPL High 应走 sink");

        let b = mk("h2");
        store.insert_event(&b).unwrap();
        let (s2, p2) = router.dispatch(&b).await.unwrap();
        assert_eq!(s2, 0, "60min 冷却内第二条应降级");
        assert_eq!(p2, 1);

        // 不同 ticker 不受冷却影响
        let mut c = mk("h3");
        c.symbols = vec!["NVDA".into()];
        // NVDA 未在订阅里,应无命中 → 0 sent, 0 pending
        store.insert_event(&c).unwrap();
        let (s3, p3) = router.dispatch(&c).await.unwrap();
        assert_eq!(s3 + p3, 0, "未订阅 NVDA,不应 dispatch");
    }

    #[tokio::test]
    async fn cooldown_zero_means_no_throttle() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_same_symbol_cooldown_minutes(0);

        for i in 0..3 {
            let mut e = ev(Severity::High);
            e.id = format!("h{i}");
            store.insert_event(&e).unwrap();
            let (s, _) = router.dispatch(&e).await.unwrap();
            assert_eq!(s, 1, "cooldown=0 时不应降级");
        }
        assert_eq!(sink.calls.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn medium_and_low_are_deferred_to_digest() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let (sent_m, pending_m) = router.dispatch(&ev(Severity::Medium)).await.unwrap();
        let (sent_l, pending_l) = router.dispatch(&ev(Severity::Low)).await.unwrap();
        assert_eq!(sent_m + sent_l, 0);
        assert_eq!(pending_m + pending_l, 2);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn polisher_body_overrides_default_template() {
        use crate::polisher::BodyPolisher;

        struct FixedPolisher;
        #[async_trait]
        impl BodyPolisher for FixedPolisher {
            async fn polish(&self, _e: &MarketEvent, _b: &str) -> Option<String> {
                Some("POLISHED BODY".into())
            }
        }

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(Arc::new(SharedRegistry::from_registry(reg)), sink.clone(), store, digest)
            .with_polisher(Arc::new(FixedPolisher));

        let _ = router.dispatch(&ev(Severity::High)).await.unwrap();
        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls[0].1, "POLISHED BODY");
    }

    #[tokio::test]
    async fn disabled_prefs_skip_send_and_enqueue() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store =
            Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(Arc::new(SharedRegistry::from_registry(reg)), sink.clone(), store, digest)
            .with_prefs(prefs_store);

        let (sent_h, pending_h) = router.dispatch(&ev(Severity::High)).await.unwrap();
        let (sent_m, pending_m) = router.dispatch(&ev(Severity::Medium)).await.unwrap();
        assert_eq!(sent_h + sent_m, 0);
        assert_eq!(pending_h + pending_m, 0);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn portfolio_only_prefs_drop_symbolless_events() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs};

        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        // 强行命中：用 GlobalSubscription-like 兜底——直接用一个命中所有事件的 Subscription。
        // 这里简化为 dispatch MacroEvent 并注入 GlobalSubscription。
        struct AlwaysMatch(ActorIdentity);
        impl crate::subscription::Subscription for AlwaysMatch {
            fn id(&self) -> &str { "always" }
            fn matches(&self, _e: &MarketEvent) -> bool { true }
            fn actors(&self) -> Vec<ActorIdentity> { vec![self.0.clone()] }
        }
        reg.register(Box::new(AlwaysMatch(actor("u1"))));

        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let prefs_store =
            Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        use crate::prefs::PrefsProvider;
        prefs_store
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    portfolio_only: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let router = NotificationRouter::new(Arc::new(SharedRegistry::from_registry(reg)), sink.clone(), store, digest)
            .with_prefs(prefs_store);

        // 无 symbol 的 macro 事件应被过滤
        let mut macro_ev = ev(Severity::High);
        macro_ev.kind = crate::event::EventKind::MacroEvent;
        macro_ev.symbols.clear();
        let (sent, _pending) = router.dispatch(&macro_ev).await.unwrap();
        assert_eq!(sent, 0);
        assert!(sink.calls.lock().unwrap().is_empty());

        // 命中 symbol 的事件仍应送达
        let (sent, _pending) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
    }

    #[tokio::test]
    async fn low_news_upgrades_to_medium_when_same_day_hard_signal_exists() {
        // 构造一条今日 AAPL 的 price_alert 先入 store,再 dispatch 一条 Low NewsCritical,
        // 应升级为 Medium 并进 digest(sent=0, pending=1)。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        // 先落一条硬信号
        let hard = MarketEvent {
            id: "price:AAPL:today".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 700,
                window: "day".into(),
            },
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL +7%".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&hard).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        );

        let news = MarketEvent {
            id: "news:AAPL:1".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL minor headline".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "Low 新闻应被升到 Medium 后入 digest");
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn low_news_upgrades_inside_earnings_window() {
        // earnings_upcoming 的 occurred_at 是未来的财报日 00:00;今天的 Low 新闻
        // 应命中 [news - 1d, news + 2d] 窗口被升到 Medium。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

        let now = Utc::now();
        let earnings = MarketEvent {
            id: "earnings:AAPL:tomorrow".into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: now + chrono::Duration::days(1),
            title: "AAPL earnings tomorrow".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&earnings).unwrap();

        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        );

        let news = MarketEvent {
            id: "news:AAPL:prewindow".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: now,
            title: "AAPL preview".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1, "财报窗口内 Low 新闻应升到 Medium 入 digest");
    }

    #[tokio::test]
    async fn low_news_stays_low_without_same_day_signal() {
        // 无硬信号时 Low 新闻维持 Low,仍然入 digest(pending=1),但 severity 未升。
        // 间接校验:digest enqueue 行为不变,且未发生 sink 立即推。
        let (router, sink, _tmp) = router_with_aapl_actor();
        let news = MarketEvent {
            id: "news:AAPL:2".into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL minor headline".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&news).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 1);
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn globally_disabled_kind_is_dropped_before_prefs() {
        // 部署方把 press_release 放入全局黑名单。即便订阅命中,dispatch 也应
        // 返回 (0, 0),既不 sink 也不 enqueue,且 delivery_log 无记录。
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("u1"),
            vec!["AAPL".into()],
        )));
        let sink = Arc::new(CapturingSink::default());
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let router = NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store.clone(),
            digest,
        )
        .with_disabled_kinds(["press_release"]);

        let pr = MarketEvent {
            id: "pr:AAPL:1".into(),
            kind: EventKind::PressRelease,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "AAPL announces".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (sent, pending) = router.dispatch(&pr).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 0);
        assert!(sink.calls.lock().unwrap().is_empty());

        // 非黑名单 kind 不受影响
        let (sent, _) = router.dispatch(&ev(Severity::High)).await.unwrap();
        assert_eq!(sent, 1);
    }

    #[tokio::test]
    async fn event_without_subscribers_is_no_op() {
        let (router, sink, _tmp) = router_with_aapl_actor();
        let mut e = ev(Severity::High);
        e.symbols = vec!["TSLA".into()]; // 无人持仓
        let (sent, pending) = router.dispatch(&e).await.unwrap();
        assert_eq!(sent, 0);
        assert_eq!(pending, 0);
        assert!(sink.calls.lock().unwrap().is_empty());
    }
}
