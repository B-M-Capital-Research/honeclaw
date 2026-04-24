# Bug: event-engine poller cadence stalled for 104 minutes without restart

## Summary

After `2026-04-24T10:23:05Z`, the event-engine `poller ok` heartbeat disappeared for about 1 hour 44 minutes with no backend restart in between, and local event creation also stopped until the runtime restarted later.

## Observed Symptoms

- `data/runtime/logs/web.log.2026-04-24` showed the last pre-stall `poller ok` at `19:02:30.706` local time, then no new `poller ok` until `20:46:42.364` local time. The restart only happened later at `21:41:17.727`, so this gap is not explained by a backend restart:

```text
data/runtime/logs/web.log.2026-04-24:2417:[2026-04-24 19:02:30.706] INFO  poller ok
data/runtime/logs/web.log.2026-04-24:2418:[2026-04-24 19:02:53.848] INFO  retrying getting updates in 64s
data/runtime/logs/web.log.2026-04-24:2419:[2026-04-24 19:02:53.848] ERROR Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): error trying to connect: operation timed out
data/runtime/logs/web.log.2026-04-24:2420:[2026-04-24 19:04:26.883] INFO  retrying getting updates in 64s
data/runtime/logs/web.log.2026-04-24:2421:[2026-04-24 19:04:26.883] ERROR Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): connection closed before message completed
data/runtime/logs/web.log.2026-04-24:2422:[2026-04-24 19:22:37.383] INFO  retrying getting updates in 64s
data/runtime/logs/web.log.2026-04-24:2423:[2026-04-24 19:22:37.384] ERROR Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): error trying to connect: unsuccessful tunnel
data/runtime/logs/web.log.2026-04-24:2424:[2026-04-24 19:40:19.392] INFO  retrying getting updates in 64s
data/runtime/logs/web.log.2026-04-24:2425:[2026-04-24 19:40:19.392] ERROR Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): error trying to connect: unsuccessful tunnel
data/runtime/logs/web.log.2026-04-24:2426:[2026-04-24 20:46:42.364] INFO  poller ok
data/runtime/logs/web.log.2026-04-24:2431:[2026-04-24 21:41:17.727] INFO  UDP log server listening on 127.0.0.1:18118
data/runtime/logs/web.log.2026-04-24:2432:[2026-04-24 21:41:17.727] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-24:2434:[2026-04-24 21:41:17.727] INFO  event engine starting
```

- The same interval had no newly created events in `data/events.sqlite3`. A read-only query over `created_at_ts` returned rows at `2026-04-24 11:02:30 UTC`, then nothing until `2026-04-24 13:41:19 UTC`:

```text
select datetime(created_at_ts,'unixepoch'), source, severity, title
from events
where created_at_ts between 1777028550 and 1777034802;

2026-04-24 11:02:30|fmp.stock_news:prnewswire.com|low|Wheels Up Announces Changes to Board of Directors
...
2026-04-24 11:02:30|fmp.stock_news:businesswire.com|low|Arrowhead Pharmaceuticals Receives Positive CHMP Opinion ...
```

```text
select datetime(created_at_ts,'unixepoch'), source, severity, title
from events
where created_at_ts > 1777034802 and created_at_ts < 1777038077;

-- no rows --
```

- New event generation only resumed together with the next healthy delivery window:

```text
2026-04-24 13:41:19|fmp.quote|high|price:AMD:2026-04-24|AMD +13.55%
2026-04-24 13:41:21|telegram::::8039067465|sink|high|sent
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/lib.rs:572-579` only emits `poller ok` after the whole `source.poll() -> store.insert -> router.dispatch` path finishes. If a poller task blocks inside `poll().await` or `run_once().await`, there is no intermediate watchdog log:

```rust
// crates/hone-event-engine/src/lib.rs:568-579
        } else {
            dup_cnt += 1;
        }
    }
    info!(
        poller = name,
        total,
        new = new_cnt,
        duplicate = dup_cnt,
        sent,
        pending_digest = pending,
        "poller ok"
    );
```

The fixed-interval loops for news, price, and generic event sources await the full poll body directly, with `MissedTickBehavior::Delay` and no `tokio::time::timeout` / per-source supervisor. A single stuck await can therefore suppress cadence for far longer than the configured interval:

```rust
// crates/hone-event-engine/src/lib.rs:674-690
fn spawn_news_poller(...) {
    tokio::spawn(async move {
        let poller = NewsPoller::new(client);
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            match poller.poll().await {
                Ok(events) => process_events("news", events, &store, &router).await,
                Err(e) => log_poller_error("news", "fmp.stock_news", "stock_news", &e),
            }
        }
    });
}
```

```rust
// crates/hone-event-engine/src/lib.rs:878-905
fn spawn_price_poller(...) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ...
        loop {
            ticker.tick().await;
            ...
            match poller.poll().await {
                Ok(events) => process_events("price", events, &store, &router).await,
                Err(e) => log_poller_error("price", "fmp.quote", "quote", &e),
            }
        }
    });
}
```

```rust
// crates/hone-event-engine/src/lib.rs:930-955
SourceSchedule::FixedInterval(interval) => {
    if let Err(e) = run_once(&name, &*source, &store, &router).await {
        warn!(..., "initial poll failed: {e:#}");
    }
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ticker.tick().await;
    loop {
        ticker.tick().await;
        if let Err(e) = run_once(&name, &*source, &store, &router).await {
            warn!(..., "poll failed: {e:#}");
        }
    }
}
```

Given the concurrent Telegram transport errors in the same wall-clock interval, one plausible explanation is runtime-wide network/proxy turbulence that left one or more event-engine poll futures hanging long enough to skip multiple expected ticks. The current code has no local timeout boundary that would turn that hang into a bounded warning.

## Evidence Gap

- The file log format drops structured `poller=<name>` fields from `poller ok`, so this巡检 cannot identify which exact source task stalled.
- This was a read-only inspection: no live repro, stack dump, or task-level metrics were collected, so the stall could still be inside upstream HTTP, store I/O, router work, or broader Tokio runtime starvation.
- The surrounding Telegram listener errors show the process stayed alive, but they do not prove causality for the event-engine cadence gap.

## Severity

sev2. The event-engine lost its expected minute-to-minute / five-minute / fifteen-minute cadence for about 104 minutes without an intentional restart, which can delay or miss timely market alerts, but the chain later resumed and still delivered at least one High event successfully.

## Date Observed

2026-04-24T14:25:14Z
