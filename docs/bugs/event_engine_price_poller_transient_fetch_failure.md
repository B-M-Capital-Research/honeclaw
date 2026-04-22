# Bug: event-engine price poller logged a transient FMP quote fetch failure

## Summary

The price poller hit a transient FMP quote request transport failure during the 24h scan, while later poller ticks and `fmp.quote` events show the source did not fully断流.

## Observed Symptoms

- `data/runtime/logs/web.log:2123`:

```text
[2026-04-22 12:03:01.887] WARN  price poller failed: error sending request for url (https://financialmodelingprep.com/api/v3/quote/AAOI,AAPL,BE,CAI,COHR,GEV,GOOGL,MU,RKLB,SNDK,TEM,VST?apikey=<redacted>): client error (Connect): connection closed via error
```

- The same scan found `fmp.quote` records present in `data/events.jsonl`: 16 events in the last 24h and 16 total, so this looks like a single failed tick rather than an active source outage.
- Adjacent `poller ok` lines resumed after the warning (`data/runtime/logs/web.log:2126-2127` at `12:08:02` and `12:13:02` local time), and no `source断流` condition was observed for `fmp.quote`.

## Hypothesis / Suspected Code Path

Suspected path: `crates/hone-event-engine/src/lib.rs:779-809` spawns the price poller loop, calls `PricePoller::poll`, and only logs the transport error before the next interval.

```rust
fn spawn_price_poller(
    client: FmpClient,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
    registry: Arc<SharedRegistry>,
    low_pct: f64,
    high_pct: f64,
    interval: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut last_pool: Vec<String> = vec![];
        loop {
            ticker.tick().await;
            let symbols = registry.load().watch_pool();
            if symbols.is_empty() {
                continue;
            }
            let poller = PricePoller::new(client.clone())
                .with_symbols(symbols)
                .with_thresholds(low_pct, high_pct);
            match poller.poll().await {
                Ok(events) => process_events("price", events, &store, &router).await,
                Err(e) => warn!("price poller failed: {e:#}"),
            }
        }
    });
}
```

`crates/hone-event-engine/src/pollers/price.rs:47-60` constructs one batched `/v3/quote/{joined}` request. A transport failure for that single request drops the whole watch-pool quote tick until the next interval.

```rust
pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
    if self.symbols.is_empty() {
        return Ok(vec![]);
    }
    let joined = self.symbols.join(",");
    let path = format!("/v3/quote/{joined}");
    let raw = self.client.get_json(&path).await?;
    Ok(events_from_quotes(
        &raw,
        self.low_pct,
        self.high_pct,
        self.near_hi_lo_tolerance,
    ))
}
```

## Evidence Gap

- Need structured poller metrics to determine whether this was one failed HTTP attempt or part of a hidden retry sequence inside the FMP client.
- Need upstream/network telemetry around `2026-04-22T04:03:01Z` to separate local network closure from FMP-side transport failure.
- This巡检 did not call FMP or any real network API, so recovery is inferred only from later local logs and stored `fmp.quote` events.

## Severity

sev3. One quote tick can be missed for the full watch pool, but current evidence shows later ticks resumed and `fmp.quote` was not断流.

## Date Observed

2026-04-22T06:10:50Z
