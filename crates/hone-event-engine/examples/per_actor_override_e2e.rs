//! End-to-end smoke for **per-actor severity overrides** on the real router.
//!
//! Unlike `push_smoke` (which only checks `prefs.should_deliver` then directly
//! POSTs to Telegram), this example wires the real `NotificationRouter` with
//! the production `TelegramSink` + `FilePrefsStorage(./data/notif_prefs)` and
//! dispatches three synthetic events to verify that:
//!
//!   A. `price_high_pct_override` upgrades a Low PriceAlert to High → sink immediate
//!   B. `immediate_kinds` upgrades a Medium AnalystGrade to High → sink immediate
//!   C. A Low NewsCritical (not in immediate_kinds) stays in digest buffer
//!
//! The actor under test is `telegram__direct__8039067465`. Its prefs file is
//! the real production one; if you don't want to be pinged, edit/move it
//! before running.
//!
//! Run:  cargo run --example per_actor_override_e2e -p hone-event-engine

use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use hone_core::ActorIdentity;
use hone_event_engine::{
    DigestBuffer, EventStore, NotificationRouter, TelegramSink,
    digest::render_digest,
    event::{EventKind, MarketEvent, Severity},
    prefs::FilePrefsStorage,
    router::OutboundSink,
    subscription::{GlobalSubscription, SharedRegistry, SubscriptionRegistry},
};

const PREFS_DIR: &str = "./data/notif_prefs";
const CONFIG_PATH: &str = "./config.yaml";
const ACTOR_USER: &str = "8039067465";

fn read_bot_token() -> Result<String> {
    let raw =
        std::fs::read_to_string(CONFIG_PATH).with_context(|| format!("read {CONFIG_PATH}"))?;
    let cfg: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    cfg.get("telegram")
        .and_then(|t| t.get("bot_token"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .context("telegram.bot_token missing in config.yaml")
}

fn ts_tag() -> String {
    Utc::now().format("%H:%M:%S").to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = read_bot_token()?;
    let actor = ActorIdentity::new("telegram", ACTOR_USER, None::<&str>)?;

    let tmp = tempfile::tempdir()?;
    let store = Arc::new(EventStore::open(tmp.path().join("events.db"))?);
    let digest = Arc::new(DigestBuffer::new(tmp.path().join("digest"))?);

    let mut reg = SubscriptionRegistry::new();
    reg.register(Box::new(GlobalSubscription::new(
        "e2e-global",
        vec![actor.clone()],
    )));

    let prefs = Arc::new(FilePrefsStorage::new(PREFS_DIR)?);
    let sink: Arc<dyn OutboundSink> = Arc::new(TelegramSink::new(token));

    let router = NotificationRouter::new(
        Arc::new(SharedRegistry::from_registry(reg)),
        sink.clone(),
        store.clone(),
        digest.clone(),
    )
    .with_prefs(prefs);

    let now = Utc::now();
    let tag = ts_tag();

    // A. PriceAlert -7% Low → expect upgrade to High by price_high_pct_override
    let price_alert = MarketEvent {
        id: format!("e2e-price-{}", now.timestamp()),
        kind: EventKind::PriceAlert {
            pct_change_bps: -700,
            window: "1d".into(),
        },
        severity: Severity::Low,
        symbols: vec!["TEM".into()],
        occurred_at: now,
        title: format!("[E2E {tag}] TEM 模拟 -7% 价格异动"),
        summary: "本条用于验证 per-actor price_high_pct_override(=4.0)将 Low 升 High 即时推。"
            .into(),
        url: None,
        source: "e2e_per_actor_override".into(),
        payload: serde_json::json!({"changesPercentage": -7.0}),
    };

    // B. AnalystGrade Medium → expect upgrade to High by immediate_kinds
    let analyst = MarketEvent {
        id: format!("e2e-analyst-{}", now.timestamp()),
        kind: EventKind::AnalystGrade,
        severity: Severity::Medium,
        symbols: vec!["TEM".into()],
        occurred_at: now,
        title: format!("[E2E {tag}] TEM 模拟评级变动"),
        summary: "本条用于验证 per-actor immediate_kinds 包含 analyst_grade 时即时推。".into(),
        url: None,
        source: "e2e_per_actor_override".into(),
        payload: serde_json::Value::Null,
    };

    // C. NewsCritical Low → not in immediate_kinds, no override → expect digest queue
    let news = MarketEvent {
        id: format!("e2e-news-{}", now.timestamp()),
        kind: EventKind::NewsCritical,
        severity: Severity::Low,
        symbols: vec!["TEM".into()],
        occurred_at: now,
        title: format!("[E2E {tag}] TEM 模拟普通新闻 (digest)"),
        summary: "本条不应即时推,应进入 digest buffer。".into(),
        url: None,
        source: "e2e_per_actor_override".into(),
        payload: serde_json::Value::Null,
    };

    println!("== per_actor_override_e2e ==");
    println!("actor: telegram::direct::{ACTOR_USER}");
    println!("prefs: {PREFS_DIR}/telegram__direct__{ACTOR_USER}.json");
    println!();

    let cases = [
        ("A · PriceAlert -7% Low → expect SENT", &price_alert, (1, 0)),
        ("B · AnalystGrade Medium → expect SENT", &analyst, (1, 0)),
        ("C · NewsCritical Low → expect QUEUED", &news, (0, 1)),
    ];

    let mut failures = 0;
    for (label, event, expected) in cases {
        let result = router.dispatch(event).await?;
        let ok = result == expected;
        if !ok {
            failures += 1;
        }
        println!(
            "{label}: got (sent={}, pending={}) expected (sent={}, pending={})  {}",
            result.0,
            result.1,
            expected.0,
            expected.1,
            if ok { "OK" } else { "FAIL" }
        );
    }

    // D. Digest flush — drain whatever C(及之前)入了 buffer 的 events,渲染成
    //     一条 digest 摘要,经 TelegramSink 真发,完成 digest 全链路验收。
    let pending = digest.drain_actor(&actor)?;
    println!();
    println!(
        "D · Digest flush: drained {} pending event(s) from buffer for actor",
        pending.len()
    );
    if pending.is_empty() {
        println!("D · FAIL: 没东西可 flush;C 这步应至少 enqueue 1 条");
        failures += 1;
    } else {
        // 与生产 digest.rs:441 一致:用 sink 自己声明的 format,避免渲染产物
        // 与 sendMessage 的 parse_mode 不匹配(例如 <b> 当字面量泄露)。
        let body = render_digest(
            &format!("[E2E {tag}] 模拟盘后 digest"),
            &pending,
            0,
            sink.format(),
        );
        match sink.send(&actor, &body).await {
            Ok(()) => println!(
                "D · Digest sent via Telegram (events={})  OK",
                pending.len()
            ),
            Err(e) => {
                println!("D · FAIL: telegram send error: {e:#}");
                failures += 1;
            }
        }
    }

    println!();
    if failures > 0 {
        anyhow::bail!("{failures} case(s) failed");
    }
    println!(
        "All 4 cases passed. Telegram should have received 3 messages tagged [E2E {tag}] \
         (price alert + analyst grade + digest summary)."
    );
    Ok(())
}
