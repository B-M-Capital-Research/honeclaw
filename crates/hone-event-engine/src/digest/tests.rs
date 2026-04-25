use super::*;
use crate::event::{EventKind, MarketEvent, Severity};
use chrono::{TimeZone, Utc};
use hone_core::ActorIdentity;
use std::sync::Arc;
use tempfile::tempdir;

use super::curation::{
    DIGEST_MAX_ITEMS_PER_SYMBOL, DIGEST_MAX_SOCIAL_ITEMS, curate_digest_events_with_omitted_at,
    digest_score,
};

/// 测试专用 curation wrapper:原先以 `#[cfg(test)]` 形式住在 digest.rs 里,
/// 拆出 digest/ 之后挪到 tests 文件,避免 curation.rs 再暴露 `Utc::now()` 的
/// 入口。
fn curate_digest_events(events: Vec<MarketEvent>) -> Vec<MarketEvent> {
    curate_digest_events_with_omitted_at(events, Utc::now()).kept
}

fn actor(user: &str) -> ActorIdentity {
    ActorIdentity::new("imessage", user, None::<&str>).unwrap()
}

fn ev(id: &str, sym: &str) -> MarketEvent {
    MarketEvent {
        id: id.into(),
        kind: EventKind::EarningsUpcoming,
        severity: Severity::Medium,
        symbols: vec![sym.into()],
        occurred_at: Utc::now(),
        title: format!("{sym} earnings"),
        summary: String::new(),
        url: None,
        source: "test".into(),
        payload: serde_json::Value::Null,
    }
}

fn price_ev(id: &str, sym: &str, pct: f64) -> MarketEvent {
    MarketEvent {
        id: id.into(),
        kind: EventKind::PriceAlert {
            pct_change_bps: (pct * 100.0).round() as i64,
            window: "day".into(),
        },
        severity: Severity::Low,
        symbols: vec![sym.into()],
        occurred_at: Utc.with_ymd_and_hms(2026, 4, 24, 13, 45, 0).unwrap(),
        title: format!("{sym} {pct:+.2}%"),
        summary: String::new(),
        url: None,
        source: "fmp.quote".into(),
        payload: serde_json::json!({
            "changesPercentage": pct,
            "hone_price_trade_date": "2026-04-24"
        }),
    }
}

#[test]
fn enqueue_then_drain_returns_events_in_order() {
    let dir = tempdir().unwrap();
    let buf = DigestBuffer::new(dir.path()).unwrap();
    let a = actor("u1");
    buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
    buf.enqueue(&a, &ev("2", "MSFT")).unwrap();
    let drained = buf.drain_actor(&a).unwrap();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].id, "1");
    assert_eq!(drained[1].id, "2");
}

#[test]
fn price_enqueue_replaces_same_symbol_day_with_latest_event() {
    let dir = tempdir().unwrap();
    let buf = DigestBuffer::new(dir.path()).unwrap();
    let a = actor("u1");
    buf.enqueue(&a, &price_ev("price_low:AAOI:2026-04-24", "AAOI", 5.87))
        .unwrap();
    buf.enqueue(
        &a,
        &price_ev("price_band:AAOI:2026-04-24:up:1000", "AAOI", 10.35),
    )
    .unwrap();
    buf.enqueue(&a, &ev("news-1", "AAOI")).unwrap();

    let drained = buf.drain_actor(&a).unwrap();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].id, "price_band:AAOI:2026-04-24:up:1000");
    assert_eq!(drained[1].id, "news-1");
}

/// 回归:`price_digest_key` 之前带 `window`,导致 band(window=day)+ close
/// (window=close)在同 symbol 同日各占一条,digest 里就出现两条几乎重复的
/// 价格行。修复后无论 window 是什么,同 symbol 同日只留最后一条。
#[test]
fn price_enqueue_collapses_band_and_close_for_same_symbol_day() {
    let dir = tempdir().unwrap();
    let buf = DigestBuffer::new(dir.path()).unwrap();
    let a = actor("u1");

    // intraday band crossing
    let mut band = price_ev("price_band:AMD:2026-04-24:up:1200", "AMD", 13.92);
    if let EventKind::PriceAlert { ref mut window, .. } = band.kind {
        *window = "day".into();
    }
    // end-of-day close summary,window=close
    let mut close = price_ev("price_close:AMD:2026-04-24", "AMD", 13.91);
    if let EventKind::PriceAlert { ref mut window, .. } = close.kind {
        *window = "close".into();
    }

    buf.enqueue(&a, &band).unwrap();
    buf.enqueue(&a, &close).unwrap();

    let drained = buf.drain_actor(&a).unwrap();
    assert_eq!(drained.len(), 1, "band + close 应只剩一条");
    assert_eq!(drained[0].id, "price_close:AMD:2026-04-24");
}

#[test]
fn drain_leaves_no_unflushed_file() {
    let dir = tempdir().unwrap();
    let buf = DigestBuffer::new(dir.path()).unwrap();
    let a = actor("u1");
    buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
    let _ = buf.drain_actor(&a).unwrap();
    // 再次 drain 得到空
    assert!(buf.drain_actor(&a).unwrap().is_empty());
}

#[test]
fn list_pending_actors_dedups() {
    let dir = tempdir().unwrap();
    let buf = DigestBuffer::new(dir.path()).unwrap();
    let a = actor("u1");
    let b = actor("u2");
    buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
    buf.enqueue(&a, &ev("2", "MSFT")).unwrap();
    buf.enqueue(&b, &ev("3", "TSLA")).unwrap();
    let pending = buf.list_pending_actors();
    assert_eq!(pending.len(), 2);
}

#[test]
fn in_window_matches_local_time_exactly() {
    // 2026-04-21 12:30 UTC == 08:30 ET (UTC-4)
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    assert!(in_window(now, "08:30", -4));
    // 一分钟偏差不算命中
    let now_off = Utc.with_ymd_and_hms(2026, 4, 21, 12, 31, 0).unwrap();
    assert!(!in_window(now_off, "08:30", -4));
    // UTC+8（北京）下 2026-04-21 00:30 UTC == 08:30 上海
    let now_sh = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
    assert!(in_window(now_sh, "08:30", 8));
}

#[tokio::test]
async fn scheduler_respects_disabled_prefs_at_flush_time() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
    use crate::router::OutboundSink;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<(String, String)>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0
                .lock()
                .unwrap()
                .push((a.user_id.clone(), body.into()));
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let sink = Arc::new(SpySink::default());
    let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
    buf.enqueue(&actor("u1"), &ev("1", "AAPL")).unwrap();
    // u1 在 enqueue 之后把推送关了
    prefs
        .save(
            &actor("u1"),
            &NotificationPrefs {
                enabled: false,
                ..Default::default()
            },
        )
        .unwrap();

    let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
        .with_tz_offset_hours(-4)
        .with_prefs(prefs);
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    let mut fired = HashSet::new();
    let n = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n, 0, "prefs.enabled=false 下不应推送 digest");
    assert!(sink.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn scheduler_flushes_buffer_and_avoids_duplicate_fire() {
    use crate::router::OutboundSink;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<(String, String)>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0
                .lock()
                .unwrap()
                .push((a.user_id.clone(), body.into()));
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path()).unwrap());
    let sink = Arc::new(SpySink::default());
    buf.enqueue(&actor("u1"), &ev("1", "AAPL")).unwrap();
    buf.enqueue(&actor("u1"), &ev("2", "MSFT")).unwrap();
    buf.enqueue(&actor("u2"), &ev("3", "TSLA")).unwrap();

    // 显式按 ET (-4) 解释窗口，复用原有 UTC 12:30 == 08:30 ET 的测试向量。
    let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00").with_tz_offset_hours(-4);
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    let mut fired = HashSet::new();
    let n = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n, 2, "应为两个 actor 各 flush 一次");
    // 同一分钟再 tick 不应重复
    let n2 = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n2, 0);

    let calls = sink.0.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().any(|(_, b)| b.contains("AAPL")));
}

#[test]
fn render_digest_appends_overflow_footer_when_truncated() {
    let events: Vec<MarketEvent> = (0..3).map(|i| ev(&format!("e{i}"), "AAPL")).collect();
    let body = render_digest(
        "盘前摘要 · 08:30",
        &events,
        7,
        crate::renderer::RenderFormat::Plain,
    );
    // 标题里的总数应为 events + cap_overflow = 10 条
    assert!(body.contains("· 10 条"), "title 应显示总量,body = {body}");
    assert!(
        body.contains("另 7 条因数量上限未展示"),
        "应附加 cap-overflow footer,body = {body}"
    );
    assert!(
        body.contains("/missed"),
        "footer 应指向 /missed 斜杠命令,body = {body}"
    );
}

#[test]
fn render_digest_omits_footer_when_no_overflow() {
    let events: Vec<MarketEvent> = (0..2).map(|i| ev(&format!("e{i}"), "AAPL")).collect();
    let body = render_digest("盘前摘要", &events, 0, crate::renderer::RenderFormat::Plain);
    assert!(
        !body.contains("未展示"),
        "无 cap_overflow 时不应出现 footer"
    );
    assert!(
        !body.contains("/missed"),
        "无 cap_overflow 时不应推 /missed"
    );
}

#[test]
fn render_digest_recovers_social_title_from_raw_text() {
    let full = "JUST IN: Polymarket to launch 24/7 perpetual futures trading for crypto, equities, commodities, and FX markets next quarter.";
    let mut event = ev("social-1", "");
    event.kind = EventKind::SocialPost;
    event.title =
        "JUST IN: Polymarket to launch 24/7 perpetual futures trading for crypto, equiti…".into();
    event.payload = serde_json::json!({ "raw_text": full });

    let body = render_digest(
        "盘前摘要 · 19:00",
        &[event],
        0,
        crate::renderer::RenderFormat::Plain,
    );

    assert!(body.contains(full), "body = {body}");
    assert!(!body.contains("equiti…"), "body = {body}");
}

#[test]
fn render_digest_adds_compact_source_link_for_plain() {
    let mut event = ev("news-1", "AAPL");
    event.title = "Apple supplier update".into();
    event.url = Some("https://news.example.com/path/to/story".into());

    let body = render_digest(
        "盘前摘要 · 19:00",
        &[event],
        0,
        crate::renderer::RenderFormat::Plain,
    );

    assert!(body.contains("🔗 news.example.com"), "body = {body}");
    assert!(
        !body.contains("https://news.example.com/path/to/story"),
        "plain digest should not expand long source URLs: {body}"
    );
}

#[test]
fn render_digest_adds_icon_link_for_telegram_and_discord() {
    let mut event = ev("news-1", "AAPL");
    event.url = Some("https://news.example.com/path/to/story".into());

    let telegram = render_digest(
        "盘前摘要 · 19:00",
        &[event.clone()],
        0,
        crate::renderer::RenderFormat::TelegramHtml,
    );
    assert!(
        telegram.contains(r#"<a href="https://news.example.com/path/to/story">🔗</a>"#),
        "telegram = {telegram}"
    );

    let discord = render_digest(
        "盘前摘要 · 19:00",
        &[event],
        0,
        crate::renderer::RenderFormat::DiscordMarkdown,
    );
    assert!(
        discord.contains("[🔗](https://news.example.com/path/to/story)"),
        "discord = {discord}"
    );
}

#[test]
fn render_digest_feishu_post_uses_link_icon_element() {
    let mut event = ev("news-1", "AAPL");
    event.url = Some("https://news.example.com/path/to/story".into());

    let body = render_digest(
        "盘前摘要 · 19:00",
        &[event],
        0,
        crate::renderer::RenderFormat::FeishuPost,
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        parsed
            .pointer("/zh_cn/content/0/5")
            .and_then(|v| v.get("tag"))
            .and_then(|v| v.as_str()),
        Some("a")
    );
    assert_eq!(
        parsed
            .pointer("/zh_cn/content/0/5")
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str()),
        Some("🔗")
    );
    assert_eq!(
        parsed
            .pointer("/zh_cn/content/0/5")
            .and_then(|v| v.get("href"))
            .and_then(|v| v.as_str()),
        Some("https://news.example.com/path/to/story")
    );
}

#[test]
fn curation_caps_social_and_source_noise() {
    let mut events = Vec::new();
    for (i, topic) in ["bitcoin", "ethereum", "fed", "oil", "tesla", "spacex"]
        .iter()
        .enumerate()
    {
        let mut event = ev(&format!("social-{i}"), "");
        event.kind = EventKind::SocialPost;
        event.severity = Severity::Medium;
        event.source = "telegram.watcherguru".into();
        event.title = format!("JUST IN: {topic} market update");
        event.payload = serde_json::json!({ "raw_text": event.title });
        events.push(event);
    }

    let curated = curate_digest_events(events);
    assert_eq!(curated.len(), DIGEST_MAX_SOCIAL_ITEMS);
    assert!(
        curated
            .iter()
            .all(|e| matches!(e.kind, EventKind::SocialPost))
    );
}

#[test]
fn curation_omits_low_opinion_blog_news() {
    let mut event = ev("news-opinion", "AAPL");
    event.kind = EventKind::NewsCritical;
    event.severity = Severity::Low;
    event.source = "fmp.stock_news:zacks.com".into();
    event.title = "Apple Earnings Preview: Q2 2026".into();
    event.payload = serde_json::json!({"source_class": "opinion_blog"});

    let curation = curate_digest_events_with_omitted_at(
        vec![event],
        Utc.with_ymd_and_hms(2026, 4, 24, 1, 0, 0).unwrap(),
    );

    assert!(curation.kept.is_empty());
    assert_eq!(curation.omitted.len(), 1);
}

#[test]
fn curation_omits_low_news_after_importance_arbitration() {
    let mut event = ev("news-low", "AAPL");
    event.kind = EventKind::NewsCritical;
    event.severity = Severity::Low;
    event.source = "fmp.stock_news:businessinsider.com".into();
    event.title = "Tim Cook had bold visions for Apple. See which ones came true.".into();
    event.payload = serde_json::json!({"source_class": "uncertain"});

    let curation = curate_digest_events_with_omitted_at(
        vec![event],
        Utc.with_ymd_and_hms(2026, 4, 24, 1, 0, 0).unwrap(),
    );

    assert!(curation.kept.is_empty());
    assert_eq!(curation.omitted.len(), 1);
}

#[test]
fn curation_omits_low_quality_social_after_llm_no_even_with_symbols() {
    let now = Utc.with_ymd_and_hms(2026, 4, 24, 1, 0, 0).unwrap();
    let mut no_symbol = ev("social-no-symbol", "");
    no_symbol.kind = EventKind::SocialPost;
    no_symbol.severity = Severity::Low;
    no_symbol.symbols.clear();
    no_symbol.source = "telegram.watcherguru".into();
    no_symbol.title = "JUST IN: generic political update".into();

    let mut symbol_low = no_symbol.clone();
    symbol_low.id = "social-tsla-low".into();
    symbol_low.symbols = vec!["TSLA".into()];
    symbol_low.title = "JUST IN: Tesla $TSLA rises 7% today".into();

    let mut symbol_medium = no_symbol.clone();
    symbol_medium.id = "social-usdt".into();
    symbol_medium.severity = Severity::Medium;
    symbol_medium.symbols = vec!["USDT".into()];
    symbol_medium.title = "JUST IN: Tether freezes $USDT".into();

    let curation =
        curate_digest_events_with_omitted_at(vec![no_symbol, symbol_low, symbol_medium], now);

    let kept_ids: Vec<&str> = curation.kept.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(kept_ids, vec!["social-usdt"]);
    assert_eq!(curation.omitted.len(), 2);
}

#[test]
fn curation_omits_low_or_far_future_macro_calendar() {
    let now = Utc.with_ymd_and_hms(2026, 4, 24, 1, 0, 0).unwrap();
    let mut near_medium = ev("macro-near-medium", "");
    near_medium.kind = EventKind::MacroEvent;
    near_medium.severity = Severity::Medium;
    near_medium.symbols.clear();
    near_medium.occurred_at = now + chrono::Duration::hours(12);
    near_medium.title = "[US] ISM Manufacturing PMI (Apr)".into();

    let mut near_low = near_medium.clone();
    near_low.id = "macro-near-low".into();
    near_low.severity = Severity::Low;
    near_low.title = "[US] Baker Hughes Oil Rig Count".into();

    let mut far_medium = near_medium.clone();
    far_medium.id = "macro-far-medium".into();
    far_medium.occurred_at = now + chrono::Duration::days(7);
    far_medium.title = "[CH] Retail Sales YoY (Mar)".into();

    let curation =
        curate_digest_events_with_omitted_at(vec![near_medium, near_low, far_medium], now);

    let kept_ids: Vec<&str> = curation.kept.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(kept_ids, vec!["macro-near-medium"]);
    assert_eq!(curation.omitted.len(), 2);
}

#[test]
fn curation_omits_noop_analyst_grade() {
    let mut event = ev("grade-noop", "GEV");
    event.kind = EventKind::AnalystGrade;
    event.severity = Severity::Low;
    event.source = "fmp.upgrades_downgrades".into();
    event.title = "GEV · RBC Capital hold · Outperform".into();
    event.summary = "Outperform → Outperform".into();
    event.payload = serde_json::json!({
        "action": "hold",
        "previousGrade": "Outperform",
        "newGrade": "Outperform"
    });

    let curation = curate_digest_events_with_omitted_at(
        vec![event],
        Utc.with_ymd_and_hms(2026, 4, 24, 1, 0, 0).unwrap(),
    );

    assert!(curation.kept.is_empty());
    assert_eq!(curation.omitted.len(), 1);
}

#[test]
fn curation_dedupes_repeated_news_titles() {
    let mut first = ev("news-1", "GEV");
    first.kind = EventKind::NewsCritical;
    first.severity = Severity::Medium;
    first.source = "fmp.stock_news:site-a.example".into();
    first.title = "GE Vernova stock soars as data center demand lifts outlook".into();
    first.url = Some("https://site-a.example/story".into());

    let mut duplicate = first.clone();
    duplicate.id = "news-2".into();
    duplicate.source = "fmp.stock_news:site-b.example".into();
    duplicate.url = Some("https://site-b.example/story".into());

    let mut distinct = first.clone();
    distinct.id = "news-3".into();
    distinct.title = "GE Vernova raises annual revenue forecast".into();
    distinct.url = Some("https://site-c.example/story".into());

    let curated = curate_digest_events(vec![first, duplicate, distinct]);
    let ids: Vec<&str> = curated.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["news-1", "news-3"]);
}

#[test]
fn curation_dedupes_similar_same_symbol_news_titles() {
    let mut first = ev("news-1", "AMD");
    first.kind = EventKind::NewsCritical;
    first.severity = Severity::Medium;
    first.title = "AMD shares rally after data center demand lifts outlook".into();
    first.source = "fmp.stock_news:site-a.example".into();

    let mut similar = first.clone();
    similar.id = "news-2".into();
    similar.title = "AMD stock jumps as data center demand boosts outlook".into();
    similar.source = "fmp.stock_news:site-b.example".into();

    let curated = curate_digest_events(vec![first, similar]);
    assert_eq!(curated.len(), 1, "同 symbol 同主题相似标题应折叠");
}

/// 回归:同一国家同一指标的多个 Macro 条目(如加拿大零售销售
/// `Retail Sales MoM` / `Retail Sales MoM (Mar)`)以前不进 jaccard
/// 去重,会把 digest 顶端被同主题宏观噪音占满。
#[test]
fn curation_dedupes_macro_topics_for_same_country_indicator() {
    fn macro_ev(id: &str, title: &str) -> MarketEvent {
        let mut m = ev(id, "");
        m.kind = EventKind::MacroEvent;
        m.severity = Severity::Medium;
        m.symbols = Vec::new();
        m.title = title.to_string();
        m.source = "fmp.economic_calendar".into();
        m
    }

    let curated = curate_digest_events(vec![
        macro_ev("m1", "[CA] Retail Sales MoM"),
        macro_ev("m2", "[CA] Retail Sales MoM (Mar)"),
        macro_ev("m3", "[CA] Retail Sales Ex Autos MoM (Feb)"),
    ]);
    assert_eq!(
        curated.len(),
        1,
        "三条同国同指标的 macro 应折叠成一条,实际 {curated:?}"
    );
    assert_eq!(curated[0].id, "m1");
}

#[test]
fn digest_score_prefers_trusted_portfolio_signal_over_social_noise() {
    let mut social = ev("social-1", "");
    social.kind = EventKind::SocialPost;
    social.severity = Severity::Medium;
    social.source = "telegram.watcherguru".into();
    social.title = "JUST IN: generic crypto headline".into();

    let mut filing = ev("sec-1", "AAPL");
    filing.kind = EventKind::SecFiling { form: "8-K".into() };
    filing.severity = Severity::Medium;
    filing.source = "sec.gov".into();
    filing.title = "AAPL files 8-K".into();

    assert!(digest_score(&filing) > digest_score(&social));
}

#[test]
fn curation_keeps_high_items_even_when_caps_are_hit() {
    let mut events = Vec::new();
    for i in 0..DIGEST_MAX_ITEMS_PER_SYMBOL {
        let mut event = ev(&format!("aapl-low-{i}"), "AAPL");
        event.severity = Severity::Low;
        event.source = format!("source-{i}");
        events.push(event);
    }
    let mut high = ev("aapl-high", "AAPL");
    high.severity = Severity::High;
    high.title = "AAPL critical filing".into();
    high.source = "source-high".into();
    events.push(high);

    let curated = curate_digest_events(events);
    assert!(
        curated.iter().any(|e| e.id == "aapl-high"),
        "high severity digest item must not be dropped by curation caps"
    );
}

#[tokio::test]
async fn scheduler_caps_batch_and_prioritizes_high_severity() {
    use crate::event::Severity;
    use crate::router::OutboundSink;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.into());
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let sink = Arc::new(SpySink::default());
    let a = actor("u1");
    // 5 条 Low(每条 distinct ticker + distinct source,绕过 per-symbol/source
    // curation cap)+ 1 条 Medium。max_items_per_batch=3 应留 3 条,1 Medium
    // + 2 条 Low(按 score 排序)。剩下 3 条进 cap_overflow,在 footer 提示。
    let lows = ["AAPL", "MSFT", "GOOGL", "AMZN", "META"];
    for (i, sym) in lows.iter().enumerate() {
        let mut e = ev(&format!("low-{i}"), sym);
        e.severity = Severity::Low;
        e.title = format!("LOW-{i}");
        e.source = format!("test-source-{i}");
        buf.enqueue(&a, &e).unwrap();
    }
    let mut mev = ev("mid-1", "AAPL");
    mev.severity = Severity::Medium;
    mev.title = "MID-KEEP".into();
    mev.source = "test-source-mid".into();
    buf.enqueue(&a, &mev).unwrap();

    let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
        .with_tz_offset_hours(-4)
        .with_max_items_per_batch(3);
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    let mut fired = HashSet::new();
    let n = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n, 1);

    let calls = sink.0.lock().unwrap();
    assert_eq!(calls.len(), 1);
    let body = &calls[0];
    // 6 条事件全部通过 curation(distinct symbol + source);cap=3 砍掉 3 条进
    // cap_overflow。title 应为 3 kept + 3 cap_overflow = 6 条。
    assert!(body.contains("· 6 条"), "body = {body}");
    // Medium 优先保留
    assert!(body.contains("MID-KEEP"), "Medium 应被保留,body = {body}");
    // 溢出提示反映 cap_overflow
    assert!(body.contains("另 3 条因数量上限未展示"), "body = {body}");
    assert!(
        body.contains("/missed"),
        "footer 应推 /missed,body = {body}"
    );
}

#[tokio::test]
async fn scheduler_logs_omitted_digest_items_for_truncated_batches() {
    use crate::router::OutboundSink;
    use crate::store::EventStore;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.into());
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let db_path = dir.path().join("events.db");
    let store = Arc::new(EventStore::open(&db_path).unwrap());
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let sink = Arc::new(SpySink::default());
    let a = actor("u1");
    for i in 0..3 {
        let mut e = ev(&format!("mid-{i}"), &format!("SYM{i}"));
        e.severity = Severity::Medium;
        e.title = format!("MID-{i}");
        buf.enqueue(&a, &e).unwrap();
    }

    let sched = DigestScheduler::new(buf, sink, "08:30", "17:00")
        .with_tz_offset_hours(-4)
        .with_max_items_per_batch(1)
        .with_store(store);
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    let mut fired = HashSet::new();
    let n = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n, 1);

    let conn = rusqlite::Connection::open(db_path).unwrap();
    let sent_items: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM delivery_log WHERE channel='digest_item' AND status='sent'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let omitted_items: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM delivery_log WHERE channel='digest_item' AND status='omitted'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(sent_items, 1);
    assert_eq!(omitted_items, 2);
}

#[tokio::test]
async fn scheduler_min_gap_skips_close_digest_windows_without_draining() {
    use crate::router::OutboundSink;
    use crate::store::EventStore;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.into());
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
    let sink = Arc::new(SpySink::default());
    let a = actor("u1");
    buf.enqueue(&a, &ev("first", "AAPL")).unwrap();

    let sched = DigestScheduler::new(buf.clone(), sink.clone(), "08:30", "12:00")
        .with_tz_offset_hours(8)
        .with_store(store)
        .with_min_gap_minutes(240);
    let mut fired = HashSet::new();
    let morning = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
    assert_eq!(sched.tick_once(morning, &mut fired).await.unwrap(), 1);

    buf.enqueue(&a, &ev("second", "MSFT")).unwrap();
    let noon = Utc.with_ymd_and_hms(2026, 4, 21, 4, 0, 0).unwrap();
    assert_eq!(sched.tick_once(noon, &mut fired).await.unwrap(), 0);
    assert_eq!(
        buf.drain_actor(&a).unwrap().len(),
        1,
        "min-gap skip 不应 drain buffer"
    );
}

#[tokio::test]
async fn scheduler_suppresses_recently_delivered_similar_topic() {
    use crate::router::OutboundSink;
    use crate::store::EventStore;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.into());
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
    let sink = Arc::new(SpySink::default());
    let a = actor("u1");
    let mut first = ev("news-amd-1", "AMD");
    first.kind = EventKind::NewsCritical;
    first.title = "AMD shares rally after data center demand lifts outlook".into();
    first.source = "fmp.stock_news:site-a.example".into();
    let mut second = first.clone();
    second.id = "news-amd-2".into();
    second.title = "AMD stock jumps as data center demand boosts outlook".into();
    second.source = "fmp.stock_news:site-b.example".into();

    store.insert_event(&first).unwrap();
    buf.enqueue(&a, &first).unwrap();
    let sched = DigestScheduler::new(buf.clone(), sink.clone(), "08:30", "09:00")
        .with_tz_offset_hours(8)
        .with_store(store.clone());
    let mut fired = HashSet::new();
    let morning = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
    assert_eq!(sched.tick_once(morning, &mut fired).await.unwrap(), 1);

    store.insert_event(&second).unwrap();
    buf.enqueue(&a, &second).unwrap();
    let later = Utc.with_ymd_and_hms(2026, 4, 21, 1, 0, 0).unwrap();
    assert_eq!(sched.tick_once(later, &mut fired).await.unwrap(), 0);
    assert_eq!(
        sink.0.lock().unwrap().len(),
        1,
        "相似主题 24h 内不应再次形成摘要"
    );
}

#[tokio::test]
async fn per_actor_windows_and_timezones_fire_independently() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
    use crate::router::OutboundSink;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<(String, String)>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0
                .lock()
                .unwrap()
                .push((a.user_id.clone(), body.into()));
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let sink = Arc::new(SpySink::default());
    let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());

    let sh = actor("sh");
    let ny = actor("ny");
    // 给两人各 enqueue 一条 Medium
    buf.enqueue(&sh, &ev("e-sh", "AAPL")).unwrap();
    buf.enqueue(&ny, &ev("e-ny", "MSFT")).unwrap();

    // sh: 上海时区,只在本地 19:00 推一次。
    prefs
        .save(
            &sh,
            &NotificationPrefs {
                timezone: Some("Asia/Shanghai".into()),
                digest_windows: Some(vec!["19:00".into()]),
                ..Default::default()
            },
        )
        .unwrap();
    // ny: 纽约时区,只在本地 07:00 推一次。
    prefs
        .save(
            &ny,
            &NotificationPrefs {
                timezone: Some("America/New_York".into()),
                digest_windows: Some(vec!["07:00".into()]),
                ..Default::default()
            },
        )
        .unwrap();

    // 全局兜底窗口设个不会命中的 ("00:00") + 偏移设 0 即 UTC,确保命中由 prefs 决定。
    let sched = DigestScheduler::new(buf.clone(), sink.clone(), "00:00", "00:00")
        .with_tz_offset_hours(0)
        .with_prefs(prefs.clone());

    // T1: 2026-04-21 11:00 UTC == 19:00 上海 (CST=UTC+8) == 07:00 纽约 (EDT=UTC-4 in April)
    // 两个 actor 同时命中各自窗口。
    let now1 = Utc.with_ymd_and_hms(2026, 4, 21, 11, 0, 0).unwrap();
    let mut fired = HashSet::new();
    let n1 = sched.tick_once(now1, &mut fired).await.unwrap();
    assert_eq!(n1, 2, "两个 actor 各自命中本地窗口,应都 flush");

    let calls = sink.0.lock().unwrap();
    let users: Vec<&str> = calls.iter().map(|(u, _)| u.as_str()).collect();
    assert!(users.contains(&"sh"));
    assert!(users.contains(&"ny"));
    drop(calls);

    // 同一分钟再 tick 不重复
    let n_again = sched.tick_once(now1, &mut fired).await.unwrap();
    assert_eq!(n_again, 0);

    // T2: 同一天 ny actor 又来一条事件,sh 已经过 19:00 但还没到次日。
    // 23:00 UTC == 07:00 (next day) 上海 / 19:00 纽约 — 两边都不命中。
    buf.enqueue(&ny, &ev("e-ny-2", "GOOG")).unwrap();
    let now2 = Utc.with_ymd_and_hms(2026, 4, 21, 23, 0, 0).unwrap();
    let n2 = sched.tick_once(now2, &mut fired).await.unwrap();
    assert_eq!(n2, 0, "23:00 UTC 两个本地窗口都不命中");
}

#[tokio::test]
async fn per_actor_empty_windows_disables_digest_entirely() {
    use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
    use crate::router::OutboundSink;
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[derive(Default)]
    struct SpySink(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for SpySink {
        async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.into());
            Ok(())
        }
    }

    let dir = tempdir().unwrap();
    let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
    let sink = Arc::new(SpySink::default());
    let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());

    let a = actor("quiet");
    buf.enqueue(&a, &ev("e1", "AAPL")).unwrap();
    prefs
        .save(
            &a,
            &NotificationPrefs {
                digest_windows: Some(vec![]), // 显式关 digest
                ..Default::default()
            },
        )
        .unwrap();

    // 把全局窗口设成 08:30,UTC 偏移 -4 → UTC 12:30 命中。但该 actor 应被 prefs 关闭。
    let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
        .with_tz_offset_hours(-4)
        .with_prefs(prefs);
    let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
    let mut fired = HashSet::new();
    let n = sched.tick_once(now, &mut fired).await.unwrap();
    assert_eq!(n, 0, "digest_windows=Some(vec![]) 应彻底关 digest");
    assert!(sink.0.lock().unwrap().is_empty());
}
