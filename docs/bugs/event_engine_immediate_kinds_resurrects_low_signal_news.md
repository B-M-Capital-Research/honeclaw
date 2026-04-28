# Bug: event-engine immediate_kinds resurrects low-signal news into immediate high pushes

- **状态**: Fixed

## Summary

`pollers/news.rs` and `router.rs::maybe_upgrade_news` both intentionally keep `source_class=opinion_blog` news at `low`, but a per-actor `immediate_kinds=["news_critical"]` override later rewrites the same event to `high` and sends it immediately. In the latest window, this caused a Zacks "Top Momentum Stock" article for `VST` to bypass digest and hit the real Telegram sink.

## Fix / Verification

- 2026-04-28: `crates/hone-event-engine/src/router/policy.rs` now refuses to apply `immediate_kinds` upgrades for `news_critical` / `press_release` when the current event severity is already `Low`.
- This keeps actor-level immediate preferences from resurrecting news the classifier and source policy deliberately demoted as low-signal.
- 2026-04-28: `cargo test -p hone-event-engine per_actor_immediate_kinds_does_not_resurrect_low_signal_news --lib`

## Observed Symptoms

- The stored event itself stayed `low` and carried a clearly low-signal `opinion_blog` payload:

```text
data/events.sqlite3
2026-04-24 15:05:22|fmp.stock_news:zacks.com|low|Why Vistra Corp. (VST) is a Top Momentum Stock for the Long-Term
payload_json.source_class=opinion_blog
payload_json.fmp.text=Wondering how to pick strong, market-beating stocks for your investment portfolio? Look no further than the Zacks Style Scores.
```

- The same batch contained peer Zacks "Top Momentum Stock" listicle articles that were also stored as `low`, showing the upstream classifier was working as designed:

```text
data/events.sqlite3
2026-04-24 15:05:22|fmp.stock_news:zacks.com|low|Why UnitedHealth Group (UNH) is a Top Momentum Stock for the Long-Term
2026-04-24 15:05:22|fmp.stock_news:zacks.com|low|Why CBRE Group (CBRE) is a Top Momentum Stock for the Long-Term
2026-04-24 15:05:22|fmp.stock_news:zacks.com|low|Why Vistra Corp. (VST) is a Top Momentum Stock for the Long-Term
```

- Actor `telegram::::6996473277` has a broad kind-level override that includes `news_critical`:

```text
data/notif_prefs/telegram__direct__6996473277.json:11-16
  "immediate_kinds": [
    "news_critical",
    "press_release",
    "sec_filing",
    "earnings_released",
    "price_alert"
  ]
```

- That one `low` Zacks article then split into two different delivery paths for two direct actors:

```text
data/events.sqlite3 -> delivery_log
2026-04-24 15:05:22|news:https://www.zacks.com/stock/news/2907572/why-vistra-corp-vst-is-a-top-momentum-stock-for-the-long-term?cid=CS-STOCKNEWSAPI-FT-tale_of_the_tape|zacks_education_momentum_score-2907572|telegram::::8039067465|digest|low|queued
2026-04-24 15:05:24|news:https://www.zacks.com/stock/news/2907572/why-vistra-corp-vst-is-a-top-momentum-stock-for-the-long-term?cid=CS-STOCKNEWSAPI-FT-tale_of_the_tape|zacks_education_momentum_score-2907572|telegram::::6996473277|sink|high|sent
```

- The local runtime log confirms the event-engine did both a digest enqueue and a real sink send in the same second:

```text
data/runtime/logs/web.log.2026-04-24:2890-2907
[2026-04-24 23:05:22.688] INFO  dispatch skipped: no matching actor
[2026-04-24 23:05:22.859] INFO  digest queued
[2026-04-24 23:05:24.304] INFO  sink delivered
[2026-04-24 23:05:24.323] INFO  digest queued
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/pollers/news.rs:362-396` deliberately demotes `opinion_blog` / `pr_wire` articles to `Severity::Low`:

```rust
fn classify_severity(
    title: &str,
    text: &str,
    keywords: &[String],
    source_class: NewsSourceClass,
) -> Severity {
    if is_legal_ad_title(title) {
        return Severity::Low;
    }
    if matches!(
        source_class,
        NewsSourceClass::PrWire | NewsSourceClass::OpinionBlog
    ) {
        return Severity::Low;
    }
    let t = title.to_lowercase();
    let body = text.to_lowercase();
    let matched = keywords
        .iter()
        .any(|kw| t.contains(kw) || body.contains(kw));
    if !matched {
        return Severity::Low;
    }
    if matches!(source_class, NewsSourceClass::Trusted) {
        return Severity::High;
    }
    Severity::Low
}
```

`crates/hone-event-engine/src/router.rs:320-324` also explicitly prevents low-signal news from being upgraded by window convergence:

```rust
if !matches!(event.kind, EventKind::NewsCritical) || event.severity != Severity::Low {
    return event.clone();
}
if news_source_class_is_low_signal(event) {
    return event.clone();
}
```

But `crates/hone-event-engine/src/router.rs:539-586` runs later in the per-actor path and blindly rewrites any matching `kind_tag()` to `Severity::High`, with no guard for `source_class=opinion_blog` / `pr_wire` and no exception for already-demoted low-signal news:

```rust
fn apply_per_actor_severity_override(
    &self,
    event: &MarketEvent,
    sev: Severity,
    prefs: &NotificationPrefs,
) -> Severity {
    if matches!(sev, Severity::High) {
        return sev;
    }
    if let Some(kinds) = prefs.immediate_kinds.as_deref() {
        let tag = kind_tag(&event.kind);
        if kinds.iter().any(|k| k == tag) {
            if is_noop_analyst_grade(event) {
                tracing::info!(
                    event_id = %event.id,
                    kind = %tag,
                    source = %event.source,
                    "immediate_kinds override skipped for no-op analyst grade"
                );
                return sev;
            }
            return Severity::High;
        }
    }
    sev
}
```

This ordering means `immediate_kinds=["news_critical"]` can resurrect any low-signal `NewsCritical` article into the immediate sink path, even though earlier pipeline stages intentionally kept that source class out of high/medium escalation.

## Evidence Gap

- This巡检 only has local sink success evidence (`delivery_log` + `web.log`); it did not call the real Telegram API, so it cannot prove user-side read/receipt state.
- Current evidence is from one actor (`telegram::::6996473277`) and one holding (`VST`). Need a wider portfolio/prefs sample to quantify how many users are exposed when they enable `immediate_kinds=["news_critical"]`.
- Product intent is still ambiguous: if `immediate_kinds=["news_critical"]` is supposed to mean "all news for my holdings, regardless of source quality", then this is a policy issue; if low-signal demotions are meant to be hard guards, then the current ordering is a routing bug.

## Severity

`sev2`。理由：这会把本应停留在 digest 的低价值 `opinion_blog` 新闻直接打进即时提醒，既增加用户噪声，也会占用同 actor 的 high/cooldown 配额，进而影响真正重要事件的送达时机；但当前证据仍局限在特定 prefs 组合下的单 actor 样本，没有证明是全量用户都会触发的 `sev1`。

## Date Observed

`2026-04-24T15:05:24Z`
