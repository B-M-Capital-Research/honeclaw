# Bug: event-engine price poller logged a transient FMP quote fetch failure

状态：`Closed`

关闭原因：2026-04-25 不可复现。原始观察就是单 tick 网络抖动，下一 tick 自愈，`fmp.quote` 数据流未中断。fixed-interval ticker + warn-only 已经是合适的处理；想加重试要么覆盖到所有 poller 一起设计（参见 `event_engine_poller_cadence_stall_without_restart`），要么单独包装这一个会和其他 poller 行为不一致，留作后续 cadence supervisor 工作的一部分而不是单点修复。

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

## Latest巡检 Update 2026-04-24T18:34:27Z

- 2026-04-23T06:18:33Z: the incremental window after `2026-04-23T02:16:05Z` showed a local outbound network refusal window affecting FMP quote/news fetches:

```text
data/runtime/logs/web.log.2026-04-23:518:[2026-04-23 10:54:29.577] WARN  price poller failed: error sending request for url (https://financialmodelingprep.com/api/v3/quote/AAOI,AAPL,AMD,BE,CAI,COHR,GEV,GOOGL,MU,RKLB,SNDK,TEM,VST?apikey=<redacted>): client error (Connect): tunnel error: failed to create underlying connection: tcp connect error: Connection refused (os error 61)
data/runtime/logs/web.log.2026-04-23:529:[2026-04-23 10:59:29.576] WARN  price poller failed: error sending request for url (https://financialmodelingprep.com/api/v3/quote/AAOI,AAPL,AMD,BE,CAI,COHR,GEV,GOOGL,MU,RKLB,SNDK,TEM,VST?apikey=<redacted>): client error (Connect): tunnel error: failed to create underlying connection: tcp connect error: Connection refused (os error 61)
data/runtime/logs/web.log.2026-04-23:530:[2026-04-23 10:59:29.576] WARN  news poller failed: error sending request for url (https://financialmodelingprep.com/api/v3/stock_news?limit=50&apikey=<redacted>): client error (Connect): tunnel error: failed to create underlying connection: tcp connect error: Connection refused (os error 61)
```

- The same run still saw `fmp.quote` and `fmp.stock_news:*` rows created after the cutoff, so this is still a degraded tick/window rather than a full source断流:

```text
fmp.quote|7|2026-04-23 04:43:39
fmp.stock_news:reuters.com|16|2026-04-23 05:58:39
fmp.stock_news:seekingalpha.com|28|2026-04-23 06:13:39
```

- `crates/hone-event-engine/src/lib.rs:648-663` and `:852-882` still only log and wait for the next interval when `NewsPoller::poll` or `PricePoller::poll` returns a transport error. The 2026-04-23 sample also created a `poller ok` gap from `10:49:30.912` to `11:04:31.069` local time, just over the 15 minute巡检 threshold, with no restart line between those two ticks.

## Severity

sev3. One quote tick can be missed for the full watch pool, but current evidence shows later ticks resumed and `fmp.quote` was not断流.

## Latest巡检 Update

- 2026-04-24T18:34:27Z：上次巡检之后，`quote` 批量抓取又出现了同类超时，但仍然表现为“单 tick/transient”而不是 `fmp.quote` 断流：

```text
data/runtime/logs/web.log.2026-04-24:2927:[2026-04-24 23:05:40.610] WARN  poller fetch failed: error sending request for url (https://financialmodelingprep.com/api/v3/quote/AAOI,AAPL,AMD,BE,CAI,COHR,GEV,GOOGL,MU,RKLB,SNDK,TEM,VST?apikey=<redacted>): operation timed out
data/runtime/logs/web.log.2026-04-24:2973:[2026-04-24 23:10:40.613] WARN  poller fetch failed: error sending request for url (https://financialmodelingprep.com/api/v3/quote/AAOI,AAPL,AMD,BE,CAI,COHR,GEV,GOOGL,MU,RKLB,SNDK,TEM,VST?apikey=<redacted>): operation timed out
data/runtime/logs/web.log.2026-04-24:2979:[2026-04-24 23:15:27.159] INFO  digest queued
data/runtime/logs/web.log.2026-04-24:2980:[2026-04-24 23:15:27.159] INFO  poller ok
```

- 同一窗口里 `fmp.quote` 近 24h 仍然有 25 条事件，且本轮没有出现 `poller ok` 超过 15 分钟的停摆缺口：

```text
data/events.sqlite3
fmp.quote|25
```

- 因此这次补充仍不改变定级：这是重复出现的外部抓取抖动，但目前证据仍显示 event-engine 会在后续 tick 恢复，不是新的 source 级断流。

## Date Observed

2026-04-22T06:10:50Z
