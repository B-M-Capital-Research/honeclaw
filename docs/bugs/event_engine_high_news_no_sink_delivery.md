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

## Severity

sev2. The affected events are high severity and one is a safety recall while another is a guidance cut; if they should match the user, the current evidence trail makes the miss silent rather than auditable.

## Date Observed

2026-04-22T10:11:32Z
