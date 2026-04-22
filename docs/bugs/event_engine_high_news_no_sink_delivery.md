# Bug: event-engine high news events had no sink delivery evidence

## Summary

Since the previous巡检, three `severity=high` trusted-source stock news events were stored, but no `sink delivered` log line and no `delivery_log channel=sink` row appeared for the same window.

## Observed Symptoms

- `data/events.sqlite3` recorded three high news events created after `2026-04-22T06:09:38Z`:

```text
2026-04-22 07:37:26|fmp.stock_news:reuters.com|high|news:https://www.reuters.com/legal/litigation/ford-recall-over-140000-us-vehicles-over-damaged-wires-2026-04-22/|Ford to recall over 140,000 US vehicles over damaged wires
2026-04-22 08:40:54|fmp.stock_news:reuters.com|high|news:https://www.reuters.com/legal/transactional/deutsche-telekom-shares-slip-after-t-mobile-merger-talks-reports-2026-04-22/|Deutsche Telekom shares slip after T-Mobile merger talks reports
2026-04-22 09:40:54|fmp.stock_news:wsj.com|high|news:https://www.wsj.com/business/earnings/tui-cuts-guidance-amid-uncertainty-over-u-s-iran-war-e9417edd|TUI Cuts Guidance Amid Uncertainty Over U.S.-Iran War
```

- `data/runtime/logs/web.log` / `data/runtime/logs/web.log.2026-04-22` had no `sink delivered` lines for the incremental window. `data/events.sqlite3` also had no `delivery_log` rows with `channel='sink'` and `sent_at_ts>=1776838178`.
- The active backend did assemble a real sink before the affected events:

```text
data/runtime/logs/web.log:2207:[2026-04-22 14:37:25.025] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-22:239:[2026-04-22 17:55:52.293] INFO  event engine sink: MultiChannelSink 已装配
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/router.rs:323` resolves actor subscriptions before it can send high events. If `registry.resolve(event)` returns no hits, a high event can be stored with no delivery log and no explicit skip reason.

```rust
pub async fn dispatch(&self, event: &MarketEvent) -> anyhow::Result<(u32, u32)> {
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
```

`crates/hone-event-engine/src/router.rs:440` only records successful high sends inside the hit loop. With zero hits, the function can return `(0, 0)` without durable evidence that a high event was intentionally unmatched.

```rust
match effective_sev {
    Severity::High => {
        let default_body = renderer::render_immediate(event, self.sink.format());
        let body = match self.polisher.polish(event, &default_body).await {
            Some(polished) => polished,
            None => default_body,
        };
        if let Err(e) = self.sink.send(&actor, &body).await {
            tracing::warn!(
                actor = %actor_key(&actor),
                event_id = %event.id,
                kind = %kind_tag(&event.kind),
                body_len = body.chars().count(),
                body_preview = %body_preview(&body),
                "sink send failed: {e:#}"
            );
```

The subscription registry currently broadcasts only `social_post` globally. Non-social high news still depends on portfolio symbol matching, so high market news can have no delivery path if the symbol is absent from direct holdings or represented under another ticker.

```rust
/// - 每个有持仓的 direct actor → `PortfolioSubscription`（按 ticker 命中）
/// - 所有 direct actor 汇总后 → 一个 `GlobalSubscription`(kinds=[`social_post`])
///   用于把 Telegram / Truth Social 等"无 ticker"社交事件广播给所有 actor,
///   让 router 有机会调 LLM 仲裁。未来若加 macro 全员播报,在 kinds 里追加即可。
pub fn registry_from_portfolios(storage: &PortfolioStorage) -> SubscriptionRegistry {
    let mut reg = SubscriptionRegistry::new();
    let mut direct_actors: Vec<ActorIdentity> = Vec::new();
    for (actor, portfolio) in storage.list_all() {
```

## Evidence Gap

- Need structured dispatch metrics for `hits=0` high events, including event id, symbols, and loaded subscription count.
- Need a snapshot of active direct portfolios/subscriptions to determine whether `F`, `DTEGY`, or `TUIFF` should have matched a user, alias, ADR, or ETF exposure.
- This巡检 did not call any real channel API, so it cannot prove user non-delivery; it can only show absence of local sink success evidence.

## Latest巡检 Update

- 2026-04-22T14:14:09Z: the same pattern recurred after the previous巡检 window. `data/events.sqlite3` stored a new `severity=high` WSJ stock news event with no matching `delivery_log` row:

```text
created=2026-04-22 13:52:42
occurred=2026-04-22 09:34:00
source=fmp.stock_news:wsj.com
severity=high
id=news:https://www.wsj.com/business/telecom/deutsche-telekom-shares-fall-on-reports-of-potential-merger-with-t-mobile-us-1ed8e3ba
title=Deutsche Telekom Shares Fall on Reports of Potential Merger With T-Mobile US
symbols=["DTEGY"]
delivery_rows=0
```

- The same incremental window did record successful sink sends for other High events, so this is not a global sink outage:

```text
2026-04-22 10:30:10|sec:GEV:https://www.sec.gov/Archives/edgar/data/1996810/000199681026000063/gev-20260422.htm|sink|high|sent
2026-04-22 12:37:47|earnings_surprise:GEV:2026-04-22|sink|high|sent
2026-04-22 13:32:43|price:BE:2026-04-22|sink|high|sent
2026-04-22 13:32:43|price:GEV:2026-04-22|digest|high|cooled_down
```

- Local logs also show real sink assembly and successful delivery in the same runtime:

```text
data/runtime/logs/web.log.2026-04-22:239:[2026-04-22 17:55:52.293] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-22:339:[2026-04-22 18:30:10.570] INFO  sink delivered
data/runtime/logs/web.log.2026-04-22:563:[2026-04-22 20:37:47.227] INFO  sink delivered
data/runtime/logs/web.log.2026-04-22:623:[2026-04-22 21:32:43.777] INFO  sink delivered
```

- 2026-04-22T18:13:04Z: the pattern recurred again in the next incremental window. `data/events.sqlite3` stored two new `severity=high` Reuters stock-news events created after `2026-04-22T14:12:03Z`, both with `delivery_rows=0`:

```text
created=2026-04-22 16:37:42
occurred=2026-04-22 12:15:29
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/business/united-airlines-ceo-plays-down-merger-talk-white-house-signals-skepticism-2026-04-22/
title=United Airlines CEO plays down merger talk as White House signals skepticism
symbols=["UAL"]
delivery_rows=0

created=2026-04-22 18:07:42
occurred=2026-04-22 13:44:35
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/legal/litigation/how-deutsche-telecom-t-mobile-us-could-pull-off-worlds-biggest-ma-deal-2026-04-22/
title=Explainer: How Deutsche Telecom and T-Mobile US could pull off the world's biggest M&A deal
symbols=["TMUS"]
delivery_rows=0
```

- The same window had `delivery_log` rows only for digest/prefs outcomes and no `sink` channel rows:

```text
high|118
low|419
medium|16471
delivery|filtered|low|prefs|3
delivery|queued|low|digest|6
delivery|queued|medium|digest|16
```

- The local `web.log` interval after `2026-04-22 22:12:03` had no `sink delivered`, `sink send failed`, or `[dryrun sink]` lines, while `config.yaml` and `data/runtime/effective-config.yaml` both had `event_engine.dryrun=false`.

## Severity

sev2. The affected events are high severity and one is a safety recall while another is a guidance cut; if they should match the user, the current evidence trail makes the miss silent rather than auditable.

## Date Observed

2026-04-22T10:11:32Z
