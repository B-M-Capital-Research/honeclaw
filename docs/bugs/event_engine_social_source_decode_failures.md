# Bug: event-engine social/event-source pollers log repeated decode failures

## Summary

The generic event-source poller repeatedly logged response-body decode failures after the last巡检, while the social source had no new stored events in the same window.

## Observed Symptoms

- `data/runtime/logs/web.log.2026-04-22` recorded repeated event-source poll failures after `2026-04-22T10:10:57Z`:

```text
data/runtime/logs/web.log.2026-04-22:430:[2026-04-22 18:55:58.876] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:473:[2026-04-22 19:55:58.686] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:527:[2026-04-22 20:37:42.502] WARN  initial poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:646:[2026-04-22 21:37:43.333] WARN  poll failed: error decoding response body: expected value at line 1 column 1
```

- `data/events.sqlite3` shows the configured Telegram social source had historical records, but none were created after the previous巡检:

```text
source=telegram.watcherguru
total_rows=14
last_created=2026-04-22 09:45:02
rows_after_2026-04-22T10:10:57Z=0
```

- Other FMP sources continued to store events and the main event loop continued emitting `poller ok`, so the symptom is scoped to one or more generic `EventSource` pollers rather than a full event-engine stop.

- 2026-04-22T18:13:04Z update: the decode failures continued in the next incremental window after `2026-04-22T14:12:03Z`:

```text
data/runtime/logs/web.log.2026-04-22:704:[2026-04-22 22:37:43.488] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:905:[2026-04-22 23:37:43.653] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:962:[2026-04-23 00:37:43.582] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:1063:[2026-04-23 01:37:43.598] WARN  poll failed: error decoding response body: expected value at line 1 column 1
```

- The same window did store four `telegram.watcherguru` events, so the latest evidence points to a recurring decode failure on at least one generic event source rather than a total social-source outage:

```text
telegram.watcherguru|4|2026-04-22 16:37:45
2026-04-22 14:37:45|telegram:watcherguru:13368|JUST IN: $79,000 Bitcoin@WatcherGuru
2026-04-22 15:37:45|telegram:watcherguru:13369|JUST IN: 🇷🇺 Russia advances bill to regulate & classify crypto as property and allow it in foreign trade.@WatcherGuru
2026-04-22 16:07:45|telegram:watcherguru:13370|JUST IN: 🇺🇸🇮🇷 Traders placed $430,000,000 in bets on lower oil prices minutes before President Trump announced ceasefire extension with Iran.@WatcherGuru
2026-04-22 16:37:45|telegram:watcherguru:13371|JUST IN: 🇺🇸 Treasury Secretary Bessent says crypto is going to be a "very important payment rail."@WatcherGuru
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/lib.rs:861` drives non-FMP `EventSource` implementations. It logs the poller name as a structured field, but the current text log output only preserves the generic message and decode error, making it hard to tell whether the failure came from Telegram channel preview or Truth Social without richer log formatting.

```rust
fn spawn_event_source(
    source: Arc<dyn EventSource>,
    store: Arc<EventStore>,
    router: Arc<NotificationRouter>,
) {
    let name: String = source.name().to_string();
    let schedule = source.schedule();
    tokio::spawn(async move {
        match schedule {
            SourceSchedule::FixedInterval(interval) => {
                if let Err(e) = run_once(&name, &*source, &store, &router).await {
                    warn!(poller = %name, "initial poll failed: {e:#}");
                }
                let mut ticker = tokio::time::interval(interval);
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    if let Err(e) = run_once(&name, &*source, &store, &router).await {
                        warn!(poller = %name, "poll failed: {e:#}");
                    }
                }
```

One likely source is `crates/hone-event-engine/src/pollers/social/truth_social.rs:61`, where `resp.json().await?` is called before checking HTTP success. If the upstream returns an HTML or empty error body, the user-facing log becomes the generic serde decode error instead of a status/body diagnostic.

```rust
async fn resolve_account_id(&self) -> anyhow::Result<String> {
    if let Some(id) = self.account_id.read().await.clone() {
        return Ok(id);
    }
    let url = format!(
        "{}/api/v2/search?q=%40{}&resolve=true&type=accounts&limit=1",
        self.base_url, self.username
    );
    let resp = self.http.get(&url).send().await?;
    let status = resp.status();
    let body: Value = resp.json().await?;
    if !status.is_success() {
        anyhow::bail!("truth_social search HTTP {status}: {body}");
    }
```

The configured Telegram channel path is `crates/hone-event-engine/src/pollers/social/telegram_channel.rs:54`; it fetches text and parses HTML, so it should not usually produce a serde JSON decode error by itself.

```rust
async fn fetch_html(&self) -> anyhow::Result<String> {
    let url = format!("{}/s/{}", self.base_url, self.handle);
    let resp = self.http.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        anyhow::bail!("telegram preview HTTP {status} for {url}");
    }
    Ok(body)
}
```

## Evidence Gap

- The current `web.log` text does not include the structured `poller` field, so this巡检 cannot definitively assign each decode failure to `telegram.watcherguru` or a Truth Social account.
- Need one local structured log sample or enhanced text formatting that includes `poller`, HTTP status, and a redacted body preview for non-JSON upstream responses.
- This巡检 did not call Telegram, Truth Social, FMP, or any other real network API; recovery and source health are inferred only from local logs and stored events.

## Severity

sev3. The failures are repeated and the social source produced no new events after the previous巡检, but FMP sources and sink sends continued normally, so current evidence points to a scoped source outage or observability gap rather than full event-engine failure.

Latest update: still sev3. The decode failures continued, but `telegram.watcherguru` recovered enough to persist four events in the latest window; remaining impact is a recurring source-specific failure or logging attribution gap.

Latest update 2026-04-22T22:16:25Z: still sev3. The next incremental window after `2026-04-22T18:12:34Z` logged four more generic event-source decode failures:

```text
data/runtime/logs/web.log.2026-04-22:1140:[2026-04-23 02:37:43.557] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:1183:[2026-04-23 03:37:43.618] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:1248:[2026-04-23 04:37:43.799] WARN  poll failed: error decoding response body: expected value at line 1 column 1
data/runtime/logs/web.log.2026-04-22:1310:[2026-04-23 05:37:43.624] WARN  poll failed: error decoding response body: expected value at line 1 column 1
```

The same window still produced six `telegram.watcherguru` rows in `data/events.sqlite3`, so this remains a repeated source-specific decode/observability defect rather than a full social-source outage. The text log still omits the structured `poller` field from `crates/hone-event-engine/src/lib.rs:881-882`, so attribution remains unresolved.

## Date Observed

2026-04-22T14:14:09Z
