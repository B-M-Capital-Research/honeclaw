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

fn analyst_grade_ev(id: &str, symbol: &str, firm: &str) -> MarketEvent {
    MarketEvent {
        id: id.into(),
        kind: EventKind::AnalystGrade,
        severity: Severity::High,
        symbols: vec![symbol.into()],
        occurred_at: Utc::now(),
        title: format!("{symbol} {firm} upgrade"),
        summary: String::new(),
        url: None,
        source: "fmp.grade".into(),
        payload: serde_json::json!({"gradingCompany": firm, "action": "upgrade"}),
    }
}

#[tokio::test]
async fn analyst_grade_two_firms_same_symbol_both_pass() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["SNDK".into()],
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

    let goldman = analyst_grade_ev("grade:SNDK:t1:Goldman Sachs", "SNDK", "Goldman Sachs");
    store.insert_event(&goldman).unwrap();
    assert_eq!(router.dispatch(&goldman).await.unwrap(), (1, 0));

    // 同 ticker 不同投行,60min 冷却内仍应直推
    let raymond = analyst_grade_ev("grade:SNDK:t2:Raymond James", "SNDK", "Raymond James");
    store.insert_event(&raymond).unwrap();
    assert_eq!(
        router.dispatch(&raymond).await.unwrap(),
        (1, 0),
        "不同投行不应被同 ticker cooldown 互相阻塞"
    );
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn analyst_grade_same_firm_same_symbol_demotes() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["SNDK".into()],
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

    let first = analyst_grade_ev("grade:SNDK:t1:Goldman Sachs", "SNDK", "Goldman Sachs");
    store.insert_event(&first).unwrap();
    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));

    // 同投行同 ticker 60min 内仍应降级 —— 防"同投行刷数据"
    let second = analyst_grade_ev("grade:SNDK:t2:Goldman Sachs", "SNDK", "Goldman Sachs");
    store.insert_event(&second).unwrap();
    assert_eq!(
        router.dispatch(&second).await.unwrap(),
        (0, 1),
        "同投行同 ticker 应被冷却"
    );
}

#[tokio::test]
async fn analyst_grade_missing_grading_company_falls_back_to_global_cooldown() {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["SNDK".into()],
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

    // payload 没 gradingCompany,firm = None,走旧的 analyst 共冷却(防御性 fallback)
    let mk = |id: &str| MarketEvent {
        id: id.into(),
        kind: EventKind::AnalystGrade,
        severity: Severity::High,
        symbols: vec!["SNDK".into()],
        occurred_at: Utc::now(),
        title: "grade no firm".into(),
        summary: String::new(),
        url: None,
        source: "fmp.grade".into(),
        payload: serde_json::json!({"action": "upgrade"}),
    };
    let a = mk("grade:SNDK:t1:unknown_a");
    store.insert_event(&a).unwrap();
    assert_eq!(router.dispatch(&a).await.unwrap(), (1, 0));

    let b = mk("grade:SNDK:t2:unknown_b");
    store.insert_event(&b).unwrap();
    assert_eq!(
        router.dispatch(&b).await.unwrap(),
        (0, 1),
        "缺 gradingCompany 时应回落到全 analyst 共冷却"
    );
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
async fn price_band_bypasses_generic_same_symbol_cooldown() {
    // 价格 band 的限流走自己的 advance 规则,不应被通用 same_symbol_cooldown
    // 误伤。AAOI 6→8 即两条单调新高,advance=2 下都应直推。
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
    .with_price_band_min_advance_pct(2.0);

    let first = price_band_ev("AAOI", "up", 600, 6.18);
    let second = price_band_ev("AAOI", "up", 800, 8.12);
    store.insert_event(&first).unwrap();
    store.insert_event(&second).unwrap();

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(
        router.dispatch(&second).await.unwrap(),
        (1, 0),
        "价格 band 应绕开通用同 ticker cooldown 走自己的 advance 规则"
    );
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn price_band_advance_rule_demotes_band_below_min_advance() {
    // 6% 已 sink-sent 后,7% / 6% / 5% 都不满足 monotone 新高 + 2pct,应降级。
    // 8% 满足(8 ≥ 6 + 2),允许直推。这是 advance 规则的核心防震荡职责。
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
    .with_price_band_min_advance_pct(2.0);

    let first = price_band_ev("AAOI", "up", 600, 6.18);
    // 同档位再来一次 —— id 相同,理论上 INSERT IGNORE 已挡掉,但 dispatch 层
    // 也应直接降级(不依赖 store 防重)。
    let same_again = price_band_ev("AAOI", "up", 600, 6.50);
    let advanced = price_band_ev("AAOI", "up", 800, 8.12);
    store.insert_event(&first).unwrap();
    store.insert_event(&same_again).unwrap();
    store.insert_event(&advanced).unwrap();

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(
        router.dispatch(&same_again).await.unwrap(),
        (0, 1),
        "同档位再来一次应降级(未达新高 + 2pct)"
    );
    assert_eq!(
        router.dispatch(&advanced).await.unwrap(),
        (1, 0),
        "8% 满足 6+2=8 应直推"
    );
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn price_band_advance_rule_passes_full_aaoi_2026_05_01_sequence() {
    // POC 标志案例:AAOI 2026-05-01 序列 6→8→10→12→14→16,在 advance=2 规则下
    // 应当全部 6 条直推 —— 旧 cap=2 仅给 6/8 严重失声,这条测试锁死回归。
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
    .with_price_band_min_advance_pct(2.0);

    let bands = [
        price_band_ev("AAOI", "up", 600, 6.18),
        price_band_ev("AAOI", "up", 800, 8.12),
        price_band_ev("AAOI", "up", 1000, 10.35),
        price_band_ev("AAOI", "up", 1200, 12.50),
        price_band_ev("AAOI", "up", 1400, 14.20),
        price_band_ev("AAOI", "up", 1600, 16.30),
    ];
    for ev in &bands {
        store.insert_event(ev).unwrap();
    }
    for ev in &bands {
        assert_eq!(
            router.dispatch(ev).await.unwrap(),
            (1, 0),
            "band {} 应直推(monotone 新高 + 2pct)",
            ev.id,
        );
    }
    assert_eq!(sink.calls.lock().unwrap().len(), 6);
}

#[tokio::test]
async fn price_band_advance_rule_separates_up_and_down_lanes() {
    // 上行 lane 推过的最大档不应阻挡下行 lane 的首条 band —— direction 相反应
    // 视为独立信号(行情反转的开盘锤入,值得告知)。
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
    .with_price_band_min_advance_pct(2.0);

    let up = price_band_ev("AAOI", "up", 1200, 12.50);
    let down = price_band_ev("AAOI", "down", 600, -6.30);
    store.insert_event(&up).unwrap();
    store.insert_event(&down).unwrap();

    assert_eq!(router.dispatch(&up).await.unwrap(), (1, 0));
    assert_eq!(
        router.dispatch(&down).await.unwrap(),
        (1, 0),
        "down lane 是独立通道,不该被 up lane 的 max_band 影响"
    );
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn price_band_advance_rule_disabled_when_zero() {
    // advance=0 关闭单一规则,所有 band 都直推 —— 与「无脑全推」语义一致,
    // 仅靠 INSERT IGNORE 防同档位重复。
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
    .with_price_band_min_advance_pct(0.0);

    let first = price_band_ev("AAOI", "up", 800, 8.10);
    // 反过来推 6%(在 advance>0 下会被降级),advance=0 应允许直推。
    let lower = price_band_ev("AAOI", "up", 600, 6.20);
    store.insert_event(&first).unwrap();
    store.insert_event(&lower).unwrap();

    assert_eq!(router.dispatch(&first).await.unwrap(), (1, 0));
    assert_eq!(router.dispatch(&lower).await.unwrap(), (1, 0));
    assert_eq!(sink.calls.lock().unwrap().len(), 2);
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
    // 部署方把 social_post 放入全局黑名单。即便订阅命中,dispatch 也应
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
    .with_disabled_kinds(["social_post"]);

    let blocked = MarketEvent {
        id: "social:AAPL:1".into(),
        kind: EventKind::SocialPost,
        severity: Severity::High,
        symbols: vec!["AAPL".into()],
        occurred_at: Utc::now(),
        title: "AAPL chatter".into(),
        summary: String::new(),
        url: None,
        source: "test".into(),
        payload: serde_json::Value::Null,
    };
    let (sent, pending) = router.dispatch(&blocked).await.unwrap();
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
async fn per_actor_immediate_kinds_does_not_resurrect_low_signal_news() {
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
                immediate_kinds: Some(vec!["news_critical".into()]),
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

    let news = MarketEvent {
        id: "news:AAOI:low".into(),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["AAOI".into()],
        occurred_at: Utc::now(),
        title: "AAOI 低信号新闻".into(),
        summary: String::new(),
        url: None,
        source: "opinion_blog".into(),
        payload: serde_json::Value::Null,
    };
    let (sent, pending) = router.dispatch(&news).await.unwrap();
    assert_eq!(sent, 0, "Low news must not be forced into sink");
    assert_eq!(pending, 1, "Low news can still queue normally for digest");
    assert!(sink.calls.lock().unwrap().is_empty());
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

/// 构造一个跨当前 UTC 分钟的 quiet_hours 区间(±30min,跨午夜安全)。
/// 这样测试不论何时跑都在区间内。
fn quiet_hours_around_now() -> crate::prefs::QuietHours {
    use chrono::Timelike;
    let now = Utc::now();
    let now_min = now.hour() as i32 * 60 + now.minute() as i32;
    let from_m = ((now_min - 30).rem_euclid(24 * 60)) as u32;
    let to_m = ((now_min + 30).rem_euclid(24 * 60)) as u32;
    crate::prefs::QuietHours {
        from: format!("{:02}:{:02}", from_m / 60, from_m % 60),
        to: format!("{:02}:{:02}", to_m / 60, to_m % 60),
        exempt_kinds: Vec::new(),
    }
}

fn router_with_quiet_hours_for_aapl(
    qh: crate::prefs::QuietHours,
) -> (
    NotificationRouter,
    Arc<CapturingSink>,
    Arc<EventStore>,
    tempfile::TempDir,
) {
    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(PortfolioSubscription::new(
        actor("u1"),
        vec!["AAPL".into()],
    )));
    let sink = Arc::new(CapturingSink::default());
    let dir = tempdir().unwrap();
    let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
    let digest = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let prefs_dir = dir.path().join("prefs");
    let prefs_storage = crate::prefs::FilePrefsStorage::new(&prefs_dir).unwrap();
    let mut prefs = crate::prefs::NotificationPrefs::default();
    prefs.quiet_hours = Some(qh);
    // 测试统一用 UTC 解释 quiet 区间,避免 router 默认 CST 偏移让 around_now 窗口失准
    prefs.timezone = Some("UTC".into());
    crate::prefs::PrefsProvider::save(&prefs_storage, &actor("u1"), &prefs).unwrap();
    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store.clone(),
        digest,
    )
    .with_prefs(Arc::new(prefs_storage));
    (router, sink, store, dir)
}

#[tokio::test]
async fn quiet_held_logs_status_and_skips_sink() {
    let qh = quiet_hours_around_now();
    let (router, sink, store, _tmp) = router_with_quiet_hours_for_aapl(qh);
    let mut event = ev(Severity::High);
    event.id = "earnings_in_quiet".into();
    store.insert_event(&event).unwrap();
    let (sent, pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(
        sent, 0,
        "High event should NOT go to sink during quiet_hours"
    );
    assert_eq!(pending, 0, "should not enqueue to digest either");
    assert!(
        sink.calls.lock().unwrap().is_empty(),
        "sink must not be called"
    );
    // 用公开 API 验证 quiet_held 行存在
    let since = Utc::now() - chrono::Duration::minutes(1);
    let held = store
        .list_quiet_held_since("imessage::::u1", since)
        .unwrap();
    assert_eq!(held.len(), 1, "should have exactly 1 quiet_held event");
    assert_eq!(held[0].0.id, "earnings_in_quiet");
}

#[tokio::test]
async fn exempt_kind_bypasses_quiet_hold() {
    let mut qh = quiet_hours_around_now();
    qh.exempt_kinds = vec!["earnings_released".into()];
    let (router, sink, _store, _tmp) = router_with_quiet_hours_for_aapl(qh);
    let event = ev(Severity::High); // EarningsReleased
    let (sent, _pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 1, "exempt kind must still go to sink during quiet");
    assert_eq!(sink.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn quiet_outside_window_does_not_hold() {
    use chrono::Timelike;
    // 把 quiet 区间设到现在的反面(+11h..+12h),保证 now 不在内
    let now = Utc::now();
    let from_h = (now.hour() + 11) % 24;
    let to_h = (now.hour() + 12) % 24;
    let qh = crate::prefs::QuietHours {
        from: format!("{:02}:00", from_h),
        to: format!("{:02}:00", to_h),
        exempt_kinds: Vec::new(),
    };
    let (router, sink, _store, _tmp) = router_with_quiet_hours_for_aapl(qh);
    let event = ev(Severity::High);
    let (sent, _pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 1);
    assert_eq!(sink.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn quiet_does_not_hold_medium_to_digest() {
    // 验证 quiet_hours 只拦 High,Medium 仍走 digest enqueue
    let qh = quiet_hours_around_now();
    let (router, sink, _store, _tmp) = router_with_quiet_hours_for_aapl(qh);
    let event = ev(Severity::Medium);
    let (sent, pending) = router.dispatch(&event).await.unwrap();
    assert_eq!(sent, 0);
    assert_eq!(pending, 1, "Medium event should still enqueue to digest");
    assert!(sink.calls.lock().unwrap().is_empty());
}
