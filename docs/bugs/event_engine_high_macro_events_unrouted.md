# Bug: event-engine high macro events are stored but not routed

## Summary

`fmp.economic_calendar` stored many `severity=high` macro events after the last巡检, but none had delivery rows, and many were low-impact calendar items promoted only by broad keyword matching.

## Observed Symptoms

- Incremental SQLite scan after `2026-04-22T22:14:35Z` showed 77 new high macro events from `fmp.economic_calendar`, but delivery aggregation for the same window only had `sink/sent/high=5`, all from price alerts.

```text
created_at >= 2026-04-22T22:14:35Z
fmp.economic_calendar severity=high count=77
delivery_log since same cutoff:
digest|queued|low|2
digest|queued|medium|6
digest|sent|medium|2
sink|sent|high|5
```

- Sample high macro rows had zero delivery rows even when the FMP payload impact was `Low` or `Medium`:

```text
macro:BR:2026-04-30:gross-debt-to-gdp--mar|high|[BR] Gross Debt to GDP (Mar)|Low|0
macro:CL:2026-04-30:retail-sales-yoy--mar|high|[CL] Retail Sales YoY (Mar)|Low|0
macro:CO:2026-04-30:unemployment-rate--mar|high|[CO] Unemployment Rate (Mar)|Low|0
macro:EU:2026-04-30:cpi--apr|high|[EU] CPI (Apr)|Low|0
macro:EU:2026-04-30:gdp-growth-rate-qoq--q1|high|[EU] GDP Growth Rate QoQ (Q1)|High|0
macro:MX:2026-04-30:gdp-growth-rate-qoq--q1|high|[MX] GDP Growth Rate QoQ (Q1)|Medium|0
macro:NZ:2026-04-30:anz-roy-morgan-consumer-confidence--apr|high|[NZ] ANZ Roy Morgan Consumer Confidence (Apr)|Low|0
```

- Impact distribution for those 77 high macro events:

```text
High|13
Medium|17
Low|47
```

- The same log window did prove high sink delivery for price alerts, so the sink itself was not globally silent:

```text
data/runtime/logs/web.log.2026-04-23:[2026-04-23 08:02:47.940] INFO  sink delivered
data/runtime/logs/web.log.2026-04-23:[2026-04-23 08:02:48.443] INFO  sink delivered
data/runtime/logs/web.log.2026-04-23:[2026-04-23 08:02:48.944] INFO  sink delivered
data/runtime/logs/web.log.2026-04-23:[2026-04-23 08:02:49.460] INFO  sink delivered
data/runtime/logs/web.log.2026-04-23:[2026-04-23 08:02:49.968] INFO  sink delivered
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/pollers/macro_events.rs:14` classifies macro severity only by event-name keyword. It does not consider FMP `impact`, country, user preference, or whether the event is already a routine low-impact release.

```rust
/// 默认高影响宏观事件名关键词（小写匹配事件标题）。
const DEFAULT_HIGH_MACRO_KEYWORDS: &[&str] = &[
    "cpi",
    "ppi",
    "core pce",
    "pce",
    "nonfarm",
    "non-farm",
    "unemployment rate",
    "jobless claims",
    "fomc",
    "fed interest rate",
    "federal funds",
    "gdp",
    "ism manufacturing",
    "ism services",
    "retail sales",
    "consumer confidence",
];
```

`crates/hone-event-engine/src/pollers/macro_events.rs:93` applies that classification directly to every calendar row:

```rust
let severity = classify(&event_name, keywords);
let slug = slugify(&event_name);
let date_key = date_raw.chars().take(10).collect::<String>();

Some(MarketEvent {
    id: format!("macro:{country}:{date_key}:{slug}"),
    kind: EventKind::MacroEvent,
    severity,
    symbols: vec![],
    occurred_at,
    title: format!("[{country}] {event_name}"),
```

`crates/hone-event-engine/src/subscription.rs:296` currently builds a global subscription only for `social_post`, while its own comment says macro fanout would need to be added explicitly. That explains why stored high macro events have zero delivery rows.

```rust
/// - 每个有持仓的 direct actor → `PortfolioSubscription`（按 ticker 命中）
/// - 所有 direct actor 汇总后 → 一个 `GlobalSubscription`(kinds=[`social_post`])
///   用于把 Telegram / Truth Social 等"无 ticker"社交事件广播给所有 actor,
///   让 router 有机会调 LLM 仲裁。未来若加 macro 全员播报,在 kinds 里追加即可。
pub fn registry_from_portfolios(storage: &PortfolioStorage) -> SubscriptionRegistry {
    let mut reg = SubscriptionRegistry::new();
    let mut direct_actors: Vec<ActorIdentity> = Vec::new();
```

## Evidence Gap

- This巡检 did not call FMP or any real network API; the impact distribution is inferred from local `data/events.sqlite3` payloads.
- It is not yet clear whether product intent is "macro calendar enabled means global macro push" or "macro events are stored for future digest only." The code comments conflict: `macro_events.rs` says macro uses `GlobalSubscription`, while `subscription.rs` has not enabled `macro_event`.
- Need a user preference snapshot or product decision to choose between routing selected macro events, lowering routine low-impact rows, or explicitly documenting macro as storage-only.

## Latest Tuning Attempt

- 2026-04-23T02:16:48Z follow-up: local replay of the same SQLite sample showed this is an engineering rules/modeling issue, not an LLM prompt issue. Macro severity is produced by deterministic keyword rules; no LLM path is involved.
- First attempted adding a high-only global macro subscription, then reverted it: `MacroPoller` currently pulls `today..+7d`, and many high samples were future `2026-04-30` calendar rows. Opening immediate routing would push future macro calendar reminders too early.
- Safe partial tuning kept: `crates/hone-event-engine/src/pollers/macro_events.rs` now uses FMP `impact` plus broad-market country/region gating. Replaying the latest sample projects high macro rows dropping from 77 to 15; `impact=Low` rows no longer become `severity=high`.
- Remaining routing fix needs a due-window design: either emit a separate "macro upcoming" digest event and a due-time immediate event, or dispatch duplicate stored macro rows only when `occurred_at` enters a configured reminder window.

## Severity

sev2. If high macro events are meant to alert users, they are silently unrouted; if they are not meant to alert, the current high severity creates noisy and misleading routing state for 77 events in one incremental window.

## Date Observed

2026-04-23T02:16:48Z
