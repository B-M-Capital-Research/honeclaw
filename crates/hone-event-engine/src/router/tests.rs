use super::*;
use crate::digest::DigestBuffer;
use crate::event::{EventKind, MarketEvent, Severity};
use crate::store::EventStore;
use crate::subscription::{PortfolioSubscription, SharedRegistry, SubscriptionRegistry};
use async_trait::async_trait;
use chrono::Utc;
use hone_core::ActorIdentity;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

use super::sink::actor_key;

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

fn price_band_ev(symbol: &str, direction: &str, band_bps: i64, pct: f64) -> MarketEvent {
    MarketEvent {
        id: format!("price_band:{symbol}:2026-04-24:{direction}:{band_bps}"),
        kind: EventKind::PriceAlert {
            pct_change_bps: (pct * 100.0).round() as i64,
            window: "day".into(),
        },
        severity: Severity::High,
        symbols: vec![symbol.into()],
        occurred_at: Utc::now(),
        title: format!("{symbol} {pct:+.2}%"),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({
            "changesPercentage": pct,
            "hone_price_event_scope": "band",
            "hone_price_direction": direction,
            "hone_price_band_bps": band_bps,
            "hone_price_trade_date": "2026-04-24"
        }),
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
        NotificationRouter::new(
            Arc::new(SharedRegistry::from_registry(reg)),
            sink.clone(),
            store,
            digest,
        ),
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

    let h1 = mk("h1");
    let h2 = mk("h2");
    let h3 = mk("h3");
    store.insert_event(&h1).unwrap();
    store.insert_event(&h2).unwrap();
    store.insert_event(&h3).unwrap();
    let (s1, _) = router.dispatch(&h1).await.unwrap();
    let (s2, _) = router.dispatch(&h2).await.unwrap();
    // 前两条正常走 sink
    assert_eq!(s1, 1);
    assert_eq!(s2, 1);
    assert_eq!(sink.calls.lock().unwrap().len(), 2);

    // 第三条触顶 → 降级到 digest,sink 不再收到,pending=1
    let (s3, p3) = router.dispatch(&h3).await.unwrap();
    assert_eq!(s3, 0, "触顶后 High 不应走 sink");
    assert_eq!(p3, 1, "应降级进 digest");
    assert_eq!(
        sink.calls.lock().unwrap().len(),
        2,
        "sink call count 不应增加"
    );

    // delivery_log 里应有 2 条 sent + 1 条 capped
    let since = Utc::now() - chrono::Duration::minutes(1);
    assert_eq!(
        store
            .count_high_sent_since("imessage::::u1", since)
            .unwrap(),
        2
    );
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
async fn price_band_uses_price_specific_gap_instead_of_generic_symbol_cooldown() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
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
    .with_same_symbol_cooldown_minutes(60)
    .with_price_intraday_min_gap_minutes(0)
    .with_price_symbol_direction_daily_cap(0);

    let first = price_band_ev("AAOI", "up", 600, 6.18);
    let second = price_band_ev("AAOI", "up", 800, 8.12);
    store.insert_event(&first).unwrap();
    store.insert_event(&second).unwrap();

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(
        router.dispatch(&second).await.unwrap(),
        (1, 0),
        "价格 band 应绕开通用同 ticker cooldown"
    );
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn price_band_min_gap_demotes_next_band_to_digest() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
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
    .with_price_intraday_min_gap_minutes(60)
    .with_price_symbol_direction_daily_cap(0);

    let first = price_band_ev("AAOI", "up", 600, 6.18);
    let second = price_band_ev("AAOI", "up", 800, 8.12);
    store.insert_event(&first).unwrap();
    store.insert_event(&second).unwrap();

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(router.dispatch(&second).await.unwrap(), (0, 1));
    assert_eq!(sink.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn price_band_symbol_direction_daily_cap_demotes_third_band() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
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
    .with_price_intraday_min_gap_minutes(0)
    .with_price_symbol_direction_daily_cap(2);

    let first = price_band_ev("AAOI", "up", 600, 6.18);
    let second = price_band_ev("AAOI", "up", 800, 8.12);
    let third = price_band_ev("AAOI", "up", 1000, 10.35);
    for event in [&first, &second, &third] {
        store.insert_event(event).unwrap();
    }

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(router.dispatch(&second).await.unwrap(), (1, 0));
    assert_eq!(router.dispatch(&third).await.unwrap(), (0, 1));
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn price_band_direction_cap_is_independent_for_reversal() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
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
    .with_price_intraday_min_gap_minutes(0)
    .with_price_symbol_direction_daily_cap(1);

    let up = price_band_ev("AAOI", "up", 600, 6.18);
    let down = price_band_ev("AAOI", "down", 600, -6.30);
    store.insert_event(&up).unwrap();
    store.insert_event(&down).unwrap();

    assert_eq!(router.dispatch(&up).await.unwrap(), (1, 0));
    assert_eq!(
        router.dispatch(&down).await.unwrap(),
        (1, 0),
        "正向 cap 不应挡住负向独立 lane"
    );
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
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
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
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                enabled: false,
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
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
        fn id(&self) -> &str {
            "always"
        }
        fn matches(&self, _e: &MarketEvent) -> bool {
            true
        }
        fn actors(&self) -> Vec<ActorIdentity> {
            vec![self.0.clone()]
        }
    }
    reg.register(Box::new(AlwaysMatch(actor("u1"))));

    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
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
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
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
async fn macro_high_is_digest_until_due_window_then_immediate() {
    struct AlwaysMatch(ActorIdentity);
    impl crate::subscription::Subscription for AlwaysMatch {
        fn id(&self) -> &str {
            "always"
        }
        fn matches(&self, _e: &MarketEvent) -> bool {
            true
        }
        fn actors(&self) -> Vec<ActorIdentity> {
            vec![self.0.clone()]
        }
    }

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(AlwaysMatch(actor("u1"))));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_macro_immediate_window(6, 2);

    let mut future_macro = ev(Severity::High);
    future_macro.id = "macro:future:cpi".into();
    future_macro.kind = EventKind::MacroEvent;
    future_macro.symbols.clear();
    future_macro.occurred_at = Utc::now() + chrono::Duration::days(3);
    future_macro.title = "[US] CPI YoY".into();
    future_macro.source = "fmp.economic_calendar".into();
    let (sent, pending) = router.dispatch(&future_macro).await.unwrap();
    assert_eq!(sent, 0, "未来 7 天日历不应即时推");
    assert_eq!(pending, 1);

    let mut near_macro = future_macro.clone();
    near_macro.id = "macro:near:cpi".into();
    near_macro.occurred_at = Utc::now() + chrono::Duration::hours(2);
    let (sent, pending) = router.dispatch(&near_macro).await.unwrap();
    assert_eq!(sent, 1, "临近发生窗口内的 high macro 才即时推");
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn far_earnings_preview_is_low_priority_digest() {
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
        store,
        digest.clone(),
    );
    let mut event = ev(Severity::Medium);
    event.id = "earnings:AAPL:far".into();
    event.kind = EventKind::EarningsUpcoming;
    event.occurred_at = Utc::now() + chrono::Duration::days(10);
    let (sent, pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1);
    let drained = digest.drain_actor(&actor("u1")).unwrap();
    assert_eq!(drained[0].severity, Severity::Low);
}

#[tokio::test]
async fn legal_ad_high_is_demoted_before_sink() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["SNOW".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    );
    let event = MarketEvent {
        id: "news:SNOW:legal-high".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::High,
        symbols: vec!["SNOW".into()],
        occurred_at: Utc::now(),
        title: "SHAREHOLDER ALERT class action lawsuit has been filed".into(),
        summary: String::new(),
        url: None,
        source: "fmp.stock_news:globenewswire.com".into(),
        payload: serde_json::json!({"legal_ad_template": true}),
    };
    let (sent, pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1, "法律广告即使误标 High 也应进 digest");
    assert!(sink.calls.lock().unwrap().is_empty());
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
        title: "AAPL stock jumps after price spike".into(),
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
async fn opinion_blog_news_does_not_upgrade_on_hard_signal() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AMD".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

    let hard = MarketEvent {
        id: "price:AMD:today".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 700,
            window: "day".into(),
        },
        severity: Severity::High,
        symbols: vec!["AMD".into()],
        occurred_at: Utc::now(),
        title: "AMD +7%".into(),
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
        digest.clone(),
    );

    let news = MarketEvent {
        id: "news:AMD:opinion".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["AMD".into()],
        occurred_at: Utc::now(),
        title: "AMD: Why I'm Going From Very Bearish To Confidently Bullish".into(),
        summary: String::new(),
        url: None,
        source: "fmp.stock_news:seekingalpha.com".into(),
        payload: serde_json::json!({"source_class": "opinion_blog"}),
    };
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1);

    let drained = digest.drain_actor(&actor("u1")).unwrap();
    assert_eq!(drained[0].severity, Severity::Low);
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

/// e2e:对 uncertain 源 NewsCritical Low,注入 LLM 仲裁器返回 Important
/// → router 升 Medium → 走 digest 而非 sink immediate。
#[tokio::test]
async fn llm_classifier_upgrades_uncertain_news_to_medium_for_actor() {
    use crate::news_classifier::{Importance, NewsClassifier};

    struct YesClassifier;
    #[async_trait]
    impl NewsClassifier for YesClassifier {
        async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
            Some(Importance::Important)
        }
    }

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["ACME".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_news_classifier(Arc::new(YesClassifier));

    // 模拟 poller 给的 uncertain Low NewsCritical
    let news = MarketEvent {
        id: "news:ACME:1".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["ACME".into()],
        occurred_at: Utc::now(),
        title: "ACME announces breakthrough".into(),
        summary: "ACME pioneers something".into(),
        url: None,
        source: "fmp.stock_news:smallblog.io".into(),
        payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
    };
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    // 升 Medium 后走 digest,immediate sink 仍为 0
    assert_eq!(sent, 0);
    assert_eq!(pending, 1, "LLM 升级后应进 digest");
    assert!(sink.calls.lock().unwrap().is_empty());
}

/// e2e:LLM 返回 NotImportant 时,uncertain 源新闻保持 Low,正常进 digest。
#[tokio::test]
async fn llm_classifier_keeps_low_when_not_important() {
    use crate::news_classifier::{Importance, NewsClassifier};

    struct NoClassifier;
    #[async_trait]
    impl NewsClassifier for NoClassifier {
        async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
            Some(Importance::NotImportant)
        }
    }

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["ACME".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_news_classifier(Arc::new(NoClassifier));

    let news = MarketEvent {
        id: "news:ACME:2".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["ACME".into()],
        occurred_at: Utc::now(),
        title: "ACME mundane news".into(),
        summary: "ACME has a meeting".into(),
        url: None,
        source: "fmp.stock_news:smallblog.io".into(),
        payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
    };
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1);
    assert!(sink.calls.lock().unwrap().is_empty());
}

/// e2e:trusted 源 News 不走 LLM(LLM 即便返回 Important 也不应触发,
/// 因为前置守卫只放过 source_class=uncertain)。
#[tokio::test]
async fn llm_classifier_skipped_for_trusted_source() {
    use crate::news_classifier::{Importance, NewsClassifier};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingClassifier(Arc<AtomicUsize>);
    #[async_trait]
    impl NewsClassifier for CountingClassifier {
        async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Some(Importance::Important)
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
    let counter = Arc::new(AtomicUsize::new(0));
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_news_classifier(Arc::new(CountingClassifier(counter.clone())));

    let news = MarketEvent {
        id: "news:AAPL:trust".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "AAPL news".into(),
        summary: "ok".into(),
        url: None,
        source: "fmp.stock_news:reuters.com".into(),
        payload: serde_json::json!({"source_class": "trusted", "legal_ad_template": false}),
    };
    let (_sent, _pending) = router.dispatch(&news).await.unwrap();
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "trusted source 不应触发 LLM"
    );
}

/// e2e:即使 LLM 说 important,律所模板标题(legal_ad_template=true)
/// 也保持 Low,不被 LLM 复活。
#[tokio::test]
async fn llm_classifier_does_not_resurrect_legal_ad_templates() {
    use crate::news_classifier::{Importance, NewsClassifier};

    struct YesClassifier;
    #[async_trait]
    impl NewsClassifier for YesClassifier {
        async fn classify(&self, _e: &MarketEvent, _p: &str) -> Option<Importance> {
            Some(Importance::Important)
        }
    }

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["SNOW".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_news_classifier(Arc::new(YesClassifier));

    let news = MarketEvent {
        id: "news:SNOW:legal".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["SNOW".into()],
        occurred_at: Utc::now(),
        title: "SHAREHOLDER ALERT class action lawsuit has been filed".into(),
        summary: "...".into(),
        url: None,
        source: "fmp.stock_news:globenewswire.com".into(),
        payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": true}),
    };
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    // 不应升 Medium —— 仍按原 Low 走 digest
    assert_eq!(sent, 0);
    assert_eq!(pending, 1);
}

/// e2e:per-actor news_importance_prompt 覆盖全局默认,LLM 收到 actor 的版本。
#[tokio::test]
async fn per_actor_importance_prompt_overrides_default() {
    use crate::news_classifier::{Importance, NewsClassifier};
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    // 记录 LLM 收到的 prompt;断言取到的是 actor 的覆盖版而不是全局默认。
    struct RecordingClassifier(Arc<Mutex<Vec<String>>>);
    #[async_trait]
    impl NewsClassifier for RecordingClassifier {
        async fn classify(&self, _e: &MarketEvent, p: &str) -> Option<Importance> {
            self.0.lock().unwrap().push(p.to_string());
            Some(Importance::NotImportant)
        }
    }

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["ACME".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                news_importance_prompt: Some("仅与 SaaS 行业并购相关".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store)
    .with_default_importance_prompt("全局默认 prompt")
    .with_news_classifier(Arc::new(RecordingClassifier(captured.clone())));

    let news = MarketEvent {
        id: "news:ACME:per-actor".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["ACME".into()],
        occurred_at: Utc::now(),
        title: "ACME bulletin".into(),
        summary: "...".into(),
        url: None,
        source: "fmp.stock_news:smallblog.io".into(),
        payload: serde_json::json!({"source_class": "uncertain", "legal_ad_template": false}),
    };
    router.dispatch(&news).await.unwrap();
    let prompts = captured.lock().unwrap().clone();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0], "仅与 SaaS 行业并购相关");
}

#[tokio::test]
async fn news_upgrade_per_symbol_cap_limits_burst_within_tick() {
    // 同一 ticker 在单 tick 内最多升级 N 条;超出的 Low NewsCritical 维持 Low,
    // 不进 digest 顶端。
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

    // 先落一条硬信号,使 maybe_upgrade_news 满足窗口条件
    let hard = MarketEvent {
        id: "earnings:AAPL:tomorrow".into(),
        kind: EventKind::EarningsUpcoming,
        severity: Severity::Medium,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now() + chrono::Duration::days(1),
        title: "AAPL earnings tomorrow".into(),
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
    )
    .with_news_upgrade_per_symbol_per_tick_cap(2);

    // 模拟一个 tick 入口
    router.reset_tick_counters();

    let mk = |id: &str| MarketEvent {
        id: id.into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: format!("AAPL earnings preview {id}"),
        summary: String::new(),
        url: None,
        source: "test".into(),
        payload: serde_json::Value::Null,
    };

    // 前 2 条命中升级 → Medium → digest pending=1
    let (s1, p1) = router.dispatch(&mk("n1")).await.unwrap();
    let (s2, p2) = router.dispatch(&mk("n2")).await.unwrap();
    assert_eq!(s1 + s2, 0);
    assert_eq!(p1 + p2, 2);

    // 第 3 条触顶 → 维持 Low → 仍入 digest(pending=1),但 severity 没升
    let (s3, p3) = router.dispatch(&mk("n3")).await.unwrap();
    assert_eq!(s3, 0);
    assert_eq!(p3, 1);

    // reset 后下一 tick 重新计数
    router.reset_tick_counters();
    let (s4, p4) = router.dispatch(&mk("n4")).await.unwrap();
    assert_eq!(s4, 0);
    assert_eq!(p4, 1);
}

#[tokio::test]
async fn news_upgrade_cap_zero_means_unlimited() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());

    let hard = MarketEvent {
        id: "earnings:AAPL:tomorrow".into(),
        kind: EventKind::EarningsUpcoming,
        severity: Severity::Medium,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now() + chrono::Duration::days(1),
        title: "AAPL earnings tomorrow".into(),
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
    )
    .with_news_upgrade_per_symbol_per_tick_cap(0);

    for i in 0..6 {
        let news = MarketEvent {
            id: format!("n{i}"),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: format!("AAPL earnings preview {i}"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (_s, p) = router.dispatch(&news).await.unwrap();
        assert_eq!(p, 1);
    }
}

#[tokio::test]
async fn news_upgrade_per_tick_cap_limits_cross_symbol_burst() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into(), "AMD".into(), "GEV".into(), "MU".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let now = Utc::now();
    for sym in ["AAPL", "AMD", "GEV", "MU"] {
        let hard = MarketEvent {
            id: format!("earnings:{sym}:tomorrow"),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![sym.into()],
            occurred_at: now + chrono::Duration::days(1),
            title: format!("{sym} earnings tomorrow"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        store.insert_event(&hard).unwrap();
    }

    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink,
        store,
        digest.clone(),
    )
    .with_news_upgrade_per_symbol_per_tick_cap(3)
    .with_news_upgrade_per_tick_cap(2);
    router.reset_tick_counters();

    for sym in ["AAPL", "AMD", "GEV", "MU"] {
        let news = MarketEvent {
            id: format!("news:{sym}:1"),
            kind: EventKind::NewsCritical,
            severity: Severity::Low,
            symbols: vec![sym.into()],
            occurred_at: now,
            title: format!("{sym} earnings preview"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        };
        let (_s, p) = router.dispatch(&news).await.unwrap();
        assert_eq!(p, 1);
    }

    let drained = digest.drain_actor(&actor("u1")).unwrap();
    let upgraded = drained
        .iter()
        .filter(|e| e.severity == Severity::Medium)
        .count();
    assert_eq!(upgraded, 2, "per-tick cap should limit total upgrades");
    assert_eq!(drained.len(), 4);
}

#[tokio::test]
async fn news_upgrade_tick_stats_capture_upgrades_and_skips() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into(), "AMD".into(), "GEV".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let now = Utc::now();
    for sym in ["AAPL", "AMD", "GEV"] {
        store
            .insert_event(&MarketEvent {
                id: format!("earnings:{sym}:tomorrow"),
                kind: EventKind::EarningsUpcoming,
                severity: Severity::Medium,
                symbols: vec![sym.into()],
                occurred_at: now + chrono::Duration::days(1),
                title: format!("{sym} earnings tomorrow"),
                summary: String::new(),
                url: None,
                source: "test".into(),
                payload: serde_json::Value::Null,
            })
            .unwrap();
    }

    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink,
        store,
        digest,
    )
    .with_news_upgrade_per_symbol_per_tick_cap(1)
    .with_news_upgrade_per_tick_cap(2);
    router.reset_tick_counters();

    let mk = |id: &str, sym: &str| MarketEvent {
        id: id.into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec![sym.into()],
        occurred_at: now,
        title: format!("{sym} earnings preview"),
        summary: String::new(),
        url: None,
        source: "test".into(),
        payload: serde_json::Value::Null,
    };

    router.dispatch(&mk("news:aapl:1", "AAPL")).await.unwrap();
    router.dispatch(&mk("news:aapl:2", "AAPL")).await.unwrap();
    router.dispatch(&mk("news:amd:1", "AMD")).await.unwrap();
    router.dispatch(&mk("news:gev:1", "GEV")).await.unwrap();

    let stats = router.news_upgrade_tick_stats_snapshot();
    assert_eq!(stats.upgraded, 2);
    assert_eq!(stats.skipped_per_symbol_cap, 1);
    assert_eq!(stats.skipped_per_tick_cap, 1);
    assert_eq!(stats.trigger_counts.get("earnings_upcoming"), Some(&2));
    assert_eq!(stats.symbol_counts.get("AAPL"), Some(&1));
    assert_eq!(stats.symbol_counts.get("AMD"), Some(&1));
    assert_eq!(
        stats.top_symbols(5),
        vec![("AAPL".to_string(), 1), ("AMD".to_string(), 1)]
    );
}

#[tokio::test]
async fn per_actor_price_threshold_below_system_floor_stays_digest() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                price_high_pct_override: Some(3.0),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store);

    // 4% 价格异动:用户 override=3% 只表达"关注",但低于系统 6% 直推地板,
    // 应进入 digest 而不是即时打扰。
    let ev = MarketEvent {
        id: "price:AAOI:test".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 400,
            window: "day".into(),
        },
        severity: Severity::Low,
        symbols: vec!["AAOI".into()],
        occurred_at: Utc::now(),
        title: "AAOI +4.00%".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({"changesPercentage": 4.05}),
    };
    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 0, "低于系统直推地板不应即时推");
    assert_eq!(pending, 1);
    assert!(sink.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn large_position_can_use_sensitive_price_threshold() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                price_high_pct_override: Some(4.0),
                large_position_weight_pct: Some(20.0),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store)
    .with_price_min_direct_pct(6.0);

    let ev = MarketEvent {
        id: "price:AAOI:large".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 450,
            window: "day".into(),
        },
        severity: Severity::Low,
        symbols: vec!["AAOI".into()],
        occurred_at: Utc::now(),
        title: "AAOI +4.50%".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({
            "changesPercentage": 4.5,
            "portfolio_weight_pct": 25.0
        }),
    };
    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 1, "大仓位标的可使用用户敏感阈值直推");
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn directional_price_thresholds_use_move_direction() {
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
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                price_high_pct_up_override: Some(6.0),
                price_high_pct_down_override: Some(5.0),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store)
    .with_price_min_direct_pct(5.0);

    let mk = |id: &str, pct: f64| MarketEvent {
        id: id.into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: (pct * 100.0) as i64,
            window: "day".into(),
        },
        severity: Severity::Low,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: format!("AAPL {pct:+.2}%"),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({"changesPercentage": pct}),
    };
    let (sent_up, pending_up) = router.dispatch(&mk("price:up", 5.5)).await.unwrap();
    assert_eq!(sent_up, 0, "+5.5% 未达到上行 6% 阈值");
    assert_eq!(pending_up, 1);

    let (sent_down, pending_down) = router.dispatch(&mk("price:down", -5.5)).await.unwrap();
    assert_eq!(sent_down, 1, "-5.5% 达到下行 5% 阈值");
    assert_eq!(pending_down, 0);
}

#[tokio::test]
async fn price_close_direct_disabled_keeps_closing_move_in_digest() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AMD".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                price_high_pct_override: Some(4.0),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store);

    let ev = MarketEvent {
        id: "price_close:AMD:2026-04-22".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 667,
            window: "close".into(),
        },
        severity: Severity::Medium,
        symbols: vec!["AMD".into()],
        occurred_at: Utc::now(),
        title: "AMD +6.67%".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({"changesPercentage": 6.67}),
    };
    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 0, "默认不应在收盘时间即时推价格异动");
    assert_eq!(pending, 1);
    assert!(sink.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn price_close_direct_enabled_allows_closing_move_promotion() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AMD".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                price_high_pct_override: Some(4.0),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store)
    .with_price_close_direct_enabled(true);

    let ev = MarketEvent {
        id: "price_close:AMD:2026-04-22".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 667,
            window: "close".into(),
        },
        severity: Severity::Medium,
        symbols: vec!["AMD".into()],
        occurred_at: Utc::now(),
        title: "AMD +6.67%".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({"changesPercentage": 6.67}),
    };
    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 1);
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn per_actor_immediate_kinds_promotes_weekly52_high() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAOI".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                immediate_kinds: Some(vec!["weekly52_high".into()]),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store);

    let ev = MarketEvent {
        id: "52h:AAOI:test".into(),
        kind: EventKind::Weekly52High,
        severity: Severity::Medium,
        symbols: vec!["AAOI".into()],
        occurred_at: Utc::now(),
        title: "AAOI 触及 52 周新高".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::Value::Null,
    };
    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 1, "immediate_kinds 命中 weekly52_high 应即时推");
    assert_eq!(pending, 0);

    // NewsCritical Low 不在列表 → 仍走 digest。
    let news = MarketEvent {
        id: "news:AAOI:1".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["AAOI".into()],
        occurred_at: Utc::now(),
        title: "AAOI 普通新闻".into(),
        summary: String::new(),
        url: None,
        source: "test".into(),
        payload: serde_json::Value::Null,
    };
    let (sent2, pending2) = router.dispatch(&news).await.unwrap();
    assert_eq!(sent2, 0, "未在 immediate_kinds 列表的 kind 不应被升");
    assert_eq!(pending2, 1);
}

#[tokio::test]
async fn per_actor_immediate_kinds_skips_noop_analyst_grade() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["GEV".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                immediate_kinds: Some(vec!["analyst_grade".into()]),
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store);

    let ev = MarketEvent {
        id: "grade:GEV:no-op".into(),
        kind: EventKind::AnalystGrade,
        severity: Severity::Low,
        symbols: vec!["GEV".into()],
        occurred_at: Utc::now(),
        title: "GEV · RBC Capital hold · Outperform".into(),
        summary: "Outperform → Outperform".into(),
        url: Some("https://thefly.com/ajax/news_get.php?id=4335563".into()),
        source: "fmp.upgrades_downgrades".into(),
        payload: serde_json::json!({
            "action": "hold",
            "previousGrade": "Outperform",
            "newGrade": "Outperform"
        }),
    };

    let (sent, pending) = router.dispatch(&ev).await.unwrap();
    assert_eq!(sent, 0, "评级没有变化时不应被 immediate_kinds 强制直推");
    assert_eq!(pending, 1);
    assert!(sink.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn quiet_mode_demotes_news_but_keeps_sec_immediate() {
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
    let prefs_store = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    prefs_store
        .save(
            &actor("u1"),
            &NotificationPrefs {
                quiet_mode: true,
                ..Default::default()
            },
        )
        .unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store,
        digest,
    )
    .with_prefs(prefs_store);

    let mut news = ev(Severity::High);
    news.id = "news:AAPL:quiet".into();
    news.kind = EventKind::NewsCritical;
    news.title = "AAPL high news".into();
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1, "quiet mode 下新闻 High 应进 digest");

    let mut filing = ev(Severity::High);
    filing.id = "sec:AAPL:8k".into();
    filing.kind = EventKind::SecFiling { form: "8-K".into() };
    let (sent, pending) = router.dispatch(&filing).await.unwrap();
    assert_eq!(sent, 1, "SEC filing 仍应即时推");
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn dryrun_sink_success_is_not_counted_as_sent_ack() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into()],
    )));
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        Arc::new(LogSink),
        store.clone(),
        digest,
    );
    let event = ev(Severity::High);
    store.insert_event(&event).unwrap();
    let (sent, pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 1, "dispatch 计数代表 sink 调用成功");
    assert_eq!(pending, 0);
    let since = Utc::now() - chrono::Duration::minutes(1);
    assert_eq!(
        store
            .count_high_sent_since("imessage::::u1", since)
            .unwrap(),
        0,
        "dryrun status 不应被 count_high_sent_since 当成真实 sent"
    );
}

#[tokio::test]
async fn per_actor_overrides_default_off_keeps_legacy_behavior() {
    // 不设 prefs override 时,Low PriceAlert 与 Medium Weekly52High 仍走 digest。
    let (router, sink, _tmp) = router_with_aapl_actor();
    let price_low = MarketEvent {
        id: "price:AAPL:legacy".into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: 400,
            window: "day".into(),
        },
        severity: Severity::Low,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "AAPL +4%".into(),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({"changesPercentage": 4.0}),
    };
    let (sent, pending) = router.dispatch(&price_low).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1);
    assert!(sink.calls.lock().unwrap().is_empty());
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
