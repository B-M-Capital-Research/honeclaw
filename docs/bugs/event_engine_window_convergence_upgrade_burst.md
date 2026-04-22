# Bug: event-engine window convergence upgraded more than 20 news items in one tick

## Summary

A single poller tick logged 27 `news severity upgraded Low->Medium (window convergence)` decisions, exceeding the巡检 threshold for an upgrade burst.

## Observed Symptoms

- `data/runtime/logs/web.log` showed a concentrated upgrade burst at `2026-04-21 22:34:32` local time, immediately after a normal poller tick:

```text
1350:[2026-04-21 22:29:31.877] INFO  poller ok
1351:[2026-04-21 22:34:31.897] INFO  poller ok
1352:[2026-04-21 22:34:32.014] INFO  news severity upgraded Low→Medium (window convergence)
1353:[2026-04-21 22:34:32.025] INFO  news severity upgraded Low→Medium (window convergence)
1354:[2026-04-21 22:34:32.050] INFO  news severity upgraded Low→Medium (window convergence)
1355:[2026-04-21 22:34:32.058] INFO  news severity upgraded Low→Medium (window convergence)
1356:[2026-04-21 22:34:32.069] INFO  news severity upgraded Low→Medium (window convergence)
1357:[2026-04-21 22:34:32.072] INFO  news severity upgraded Low→Medium (window convergence)
1358:[2026-04-21 22:34:32.076] INFO  news severity upgraded Low→Medium (window convergence)
1359:[2026-04-21 22:34:32.079] INFO  news severity upgraded Low→Medium (window convergence)
1360:[2026-04-21 22:34:32.083] INFO  news severity upgraded Low→Medium (window convergence)
1361:[2026-04-21 22:34:32.087] INFO  news severity upgraded Low→Medium (window convergence)
1362:[2026-04-21 22:34:32.090] INFO  news severity upgraded Low→Medium (window convergence)
1363:[2026-04-21 22:34:32.097] INFO  news severity upgraded Low→Medium (window convergence)
1364:[2026-04-21 22:34:32.102] INFO  news severity upgraded Low→Medium (window convergence)
1365:[2026-04-21 22:34:32.105] INFO  news severity upgraded Low→Medium (window convergence)
1366:[2026-04-21 22:34:32.108] INFO  news severity upgraded Low→Medium (window convergence)
1367:[2026-04-21 22:34:32.112] INFO  news severity upgraded Low→Medium (window convergence)
1368:[2026-04-21 22:34:32.116] INFO  news severity upgraded Low→Medium (window convergence)
1369:[2026-04-21 22:34:32.119] INFO  news severity upgraded Low→Medium (window convergence)
1370:[2026-04-21 22:34:32.124] INFO  news severity upgraded Low→Medium (window convergence)
1371:[2026-04-21 22:34:32.127] INFO  news severity upgraded Low→Medium (window convergence)
1372:[2026-04-21 22:34:32.131] INFO  news severity upgraded Low→Medium (window convergence)
1373:[2026-04-21 22:34:32.134] INFO  news severity upgraded Low→Medium (window convergence)
1374:[2026-04-21 22:34:32.138] INFO  news severity upgraded Low→Medium (window convergence)
1375:[2026-04-21 22:34:32.141] INFO  news severity upgraded Low→Medium (window convergence)
1376:[2026-04-21 22:34:32.144] INFO  news severity upgraded Low→Medium (window convergence)
1377:[2026-04-21 22:34:32.146] INFO  digest queued
1378:[2026-04-21 22:34:32.149] INFO  news severity upgraded Low→Medium (window convergence)
1379:[2026-04-21 22:34:32.153] INFO  news severity upgraded Low→Medium (window convergence)
1380:[2026-04-21 22:34:32.174] INFO  poller ok
```

- In the last 24h, `data/events.sqlite3` `delivery_log` had 34 `digest/medium/queued` rows and 47 recent `digest queued` log lines. The burst likely fed the digest path rather than immediate sends, but it can crowd out higher-value digest content because digest batches keep only the highest-severity and newest items.
- `data/events.jsonl` source counts for the same 24h window were dominated by high-volume feeds: `fmp.earning_calendar=13332`, `fmp.stock_dividend_calendar=3376`, `fmp.economic_calendar=478`, plus many `fmp.stock_news:*` rows.

## Hypothesis / Suspected Code Path

Suspected path: `crates/hone-event-engine/src/router.rs:24` defines the hard signal set used for news convergence. Any same-symbol signal in the wide window can upgrade low news to medium.

```rust
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
```

`crates/hone-event-engine/src/router.rs:144` applies a `[news_ts - 1d, news_ts + 2d]` window, so one upcoming earnings or other hard signal can upgrade multiple unrelated same-symbol news items around the same poll:

```rust
/// News multi-signal convergence + earnings window upgrade: when event is
/// `NewsCritical + Low`, and the same ticker has a hard signal in
/// `[news_ts - 1d, news_ts + 2d]`, upgrade severity to Medium.
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
```

`crates/hone-event-engine/src/digest.rs:353` sorts digest items by severity before truncating, so mass-upgraded medium news can displace low-but-useful items and compress the digest evidence trail:

```rust
// Flood control: sort by severity desc (High->Medium->Low), then occurred_at desc.
filtered.sort_by(|a, b| {
    b.severity
        .rank()
        .cmp(&a.severity.rank())
        .then_with(|| b.occurred_at.cmp(&a.occurred_at))
});
let overflow =
    if self.max_items_per_batch > 0 && filtered.len() > self.max_items_per_batch {
        let dropped = filtered.len() - self.max_items_per_batch;
        filtered.truncate(self.max_items_per_batch);
        tracing::info!(
            actor = %format!(
                "{}::{}::{}",
                actor.channel,
                actor.channel_scope.clone().unwrap_or_default(),
                actor.user_id
            ),
```

## Evidence Gap

- The upgrade log line does not include structured fields in the plain log output; need JSON logs or event ids/symbols in text output to map the 27 upgrades to exact news records.
- Need a digest buffer or flushed archive for this actor to confirm which upgraded records were retained or truncated; no `data/runtime/telegram__direct__*.jsonl` or `.flushed-*` files were present during this巡检.
- Need user preference and subscription snapshots to decide whether the burst affected only one actor or all matching actors.

## Latest巡检 Update

- 2026-04-22T06:10:50Z: the 24h scan found 322 `news severity upgraded Low→Medium (window convergence)` log entries. The largest single-second burst is still `data/runtime/logs/web.log:1352-1379`, with 27 upgrades at `2026-04-21 22:34:32` local time, exceeding the巡检 threshold of 20.
- Later bursts remained below the threshold but frequent: `2026-04-22 06:19:32` had 13 upgrades, `2026-04-22 04:34:32` had 12, and `2026-04-22 04:19:32` / `06:49:32` / `07:04:32` each had 11.
- Poller cadence stayed healthy outside restart windows. The largest adjacent `poller ok` gap was about 1092 seconds from `data/runtime/logs/web.log:1253` to `data/runtime/logs/web.log:1281`, but that interval contains the `21:49:30` backend restart at `data/runtime/logs/web.log:1254-1256`, so it was not classified as poller停摆.
- Runtime digest buffer files were still absent (`data/runtime/telegram__direct__*.jsonl` and `.flushed-*` did not exist), while `data/events.sqlite3` recorded `digest-batch:2026-04-22@09:00:20` as `digest/medium/sent` at `2026-04-22 01:00:32` UTC. The exact retained/dropped records for the 27-upgrade burst remain unavailable.
- 2026-04-22T02:09:55Z: the maximum single-second burst in the scanned `web.log` remains `data/runtime/logs/web.log:1352-1379`, where `2026-04-21 22:34:32` local time logged 27 upgrades around one poller tick.
- Poller cadence was otherwise healthy in the recent window: adjacent `poller ok` entries stayed around five minutes or less, and no >15 minute poller停摆 without restart was observed.
- Runtime digest buffers were absent (`find data/runtime -name 'telegram__direct__*'` returned no files), while `data/events.sqlite3` recorded `digest-batch:2026-04-22@09:00:20` as `digest/medium/sent` at `2026-04-22 01:00:32` UTC. There is still no buffer archive available to map the 27 upgraded items to retained versus truncated digest content.
- 2026-04-21T22:08:04Z: the last 24h scan found 237 `news severity upgraded Low→Medium (window convergence)` log entries.
- The maximum single-second burst remains `data/runtime/logs/web.log:1352-1379`, where `2026-04-21 22:34:32` local time logged 27 upgrades around one poller tick.
- Subsequent bursts stayed below the巡检 threshold but remained frequent: `2026-04-22 04:34:32` had 12 upgrades, `2026-04-22 04:19:32` had 11, and `2026-04-22 00:04:32` had 10.
- No `data/runtime/telegram__direct__*.jsonl` or `.flushed-*` digest buffer files existed during the scan, so the latest retained/dropped digest payloads could not be inspected from runtime buffer files.

## Severity

sev3. The burst currently appears to feed digest rather than immediate push, but it can degrade digest quality and hide more relevant low/medium items behind generic convergence upgrades.

## Date Observed

2026-04-21T18:08:42Z
