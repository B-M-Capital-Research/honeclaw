# Bug: event-engine logged dryrun high sends while config dryrun was false

状态：`Closed`

关闭原因：2026-04-25 不可复现。`LogSink` 已在 2026-04-21 起 override `success_status()` → `"dryrun"`（见 `crates/hone-event-engine/src/router/sink.rs:49-51`），因此后续即使误用 dryrun sink 也不会再被 `delivery_log` 标 `sent`，可观测性问题已闭环。当前进程稳定使用 `MultiChannelSink`，巡检窗口未再出现 `[dryrun sink]` 输出。仅遗留的历史风险是 2026-04-21 那两条被标 `sent` 实则未发的 high 事件——这两条一条是 GEV 财报预告（已过窗口）、一条是 AAPL SEC filing（已过当日），不需要补发。

## Summary

During the `20:29` event-engine start, high-severity events were recorded as `sent` while the only visible outbound evidence was `[dryrun sink]`, despite `config.yaml` setting `event_engine.dryrun=false`.

## Observed Symptoms

- `config.yaml:178-182` showed event-engine enabled and non-dryrun:

```yaml
event_engine:
  enabled: true
  dryrun: false
```

- `data/runtime/logs/web.log:1048-1090` showed event-engine starting at `2026-04-21 20:29:44` and then emitting dryrun output for a high GEV earnings reminder:

```text
[2026-04-21 20:29:44.012] INFO  event engine starting
[2026-04-21 20:29:48.003] INFO  [dryrun sink] 【要闻】 $GEV · 📅 财报预告
GEV earnings tomorrow (2026-04-22)
```

- `data/runtime/logs/web.log:1121-1124` emitted another dryrun output for a high AAPL SEC filing:

```text
[2026-04-21 20:29:54.453] INFO  [dryrun sink] 【要闻】 $AAPL · 📄 SEC 8-K
AAPL filed 8-K

2026-04-20 00:00:00
```

- `data/events.sqlite3` -> `delivery_log` recorded both high events as `status=sent` through `channel=sink`, matching the dryrun timestamps:

```text
earnings:GEV:2026-04-22:countdown:1|sink|high|sent|2026-04-21 12:29:48
sec:AAPL:https://www.sec.gov/Archives/edgar/data/320193/000114036126015711/ef20071035_8k.htm|sink|high|sent|2026-04-21 12:29:54
```

- A later restart at `data/runtime/logs/web.log:1255-1256` did attach the real multi-channel sink, so the mismatch appears scoped to the earlier runtime instance:

```text
[2026-04-21 21:49:30.415] INFO  event engine sink: MultiChannelSink 已装配
[2026-04-21 21:49:30.415] INFO  event engine starting
```

## Hypothesis / Suspected Code Path

Expected assembly path: `crates/hone-web-api/src/lib.rs:70` should return `LogSink` only for `dryrun=true` or for no registered channels. With Telegram enabled and a bot token present, the expected path is `TelegramSink` inside `MultiChannelSink`.

```rust
fn build_event_engine_sink(
    core_cfg: &HoneConfig,
    engine_cfg: &EventEngineConfig,
) -> Arc<dyn OutboundSink> {
    if engine_cfg.dryrun {
        info!("event engine sink: dryrun=true,使用 LogSink");
        return Arc::new(LogSink);
    }
    let mut multi = MultiChannelSink::with_log_fallback();
    if core_cfg.telegram.enabled && !core_cfg.telegram.bot_token.trim().is_empty() {
        multi = multi.with_channel(
            "telegram",
            Arc::new(TelegramSink::new(core_cfg.telegram.bot_token.clone())),
        );
    }
```

`crates/hone-web-api/src/lib.rs:106` logs successful real sink assembly. That line was absent from the `20:29` start but present after the `21:49` restart.

```rust
    let registered = multi.channels_registered();
    if registered.is_empty() {
        info!("event engine sink: 没有渠道启用,回退到 LogSink");
        return Arc::new(LogSink);
    }
    info!(
        channels = ?registered,
        "event engine sink: MultiChannelSink 已装配"
    );
    Arc::new(multi)
}
```

`crates/hone-event-engine/src/router.rs:49` explains the observed `[dryrun sink]` output; `crates/hone-event-engine/src/router.rs:302-321` then records `status="sent"` after any sink returns `Ok(())`, including `LogSink`.

```rust
impl OutboundSink for LogSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        info!(
            actor = %actor_key(actor),
            "[dryrun sink] {body}"
        );
        Ok(())
    }
}
```

```rust
if let Err(e) = self.sink.send(&actor, &body).await {
    tracing::warn!("sink send failed: {e:#}");
    let _ = self.store.log_delivery(
        &event.id,
        &actor_key(&actor),
        "sink",
        sev,
        "failed",
        Some(&body),
    );
    continue;
}
let _ = self.store.log_delivery(
    &event.id,
    &actor_key(&actor),
    "sink",
    sev,
    "sent",
    Some(&body),
);
```

## Evidence Gap

- Need startup-time logging of the resolved `event_engine.dryrun` value and the selected sink type for every backend start.
- Need to confirm whether the `20:29` process used an older binary, stale effective config, or a launch path that bypassed `build_event_engine_sink`.
- This巡检 did not call Telegram or any real network API, so it cannot prove whether the two high events reached the user; the only durable evidence is dryrun logging plus `delivery_log.status=sent`.

## Latest巡检 Update

- 2026-04-22T06:10:50Z: the two historical `[dryrun sink]` lines are still within the current 24h scan window at `data/runtime/logs/web.log:1090-1093` and `data/runtime/logs/web.log:1121-1126`, while `config.yaml` still has `event_engine.dryrun=false`.
- The active backend process is now on the real sink assembly path: the latest restart shows `data/runtime/logs/web.log:2058` `event engine sink: MultiChannelSink 已装配`, and no new `[dryrun sink]` lines were observed after `data/runtime/logs/web.log:1121`.
- `data/events.sqlite3` still records 4 `sink/high/sent` rows in the last 24h, including real post-assembly rows such as `price:AAOI:2026-04-22` at `2026-04-22 00:04:33` UTC. The remaining risk is the earlier dryrun-marked-as-sent rows, not a currently continuing dryrun stream.
- 2026-04-22T02:09:55Z: current `config.yaml` still has `event_engine.enabled=true` and `event_engine.dryrun=false`; Telegram is enabled and Discord is disabled.
- No additional `[dryrun sink]` lines were observed after `data/runtime/logs/web.log:1121`. The latest backend start still shows `data/runtime/logs/web.log:1255-1256` with `event engine sink: MultiChannelSink 已装配` immediately before `event engine starting`.
- `data/events.sqlite3` now has later real high `sink/sent` rows, including `price:AAOI:2026-04-22` at `2026-04-22 00:04:33` UTC and an AAPL news item at `2026-04-21 21:19:33` UTC. The active process therefore appears to be on the real sink path; the unresolved risk remains the earlier two dryrun-sent high rows.
- 2026-04-21T22:08:04Z: current `config.yaml` still had `event_engine.enabled=true` and `event_engine.dryrun=false`.
- `data/runtime/logs/web.log:1048` and `data/runtime/logs/web.log:1158` showed event-engine starts without a nearby `event engine sink: MultiChannelSink 已装配` line; `data/runtime/logs/web.log:1090` and `data/runtime/logs/web.log:1121` still show `[dryrun sink]` output for high events in that non-dryrun configuration window.
- `data/runtime/logs/web.log:1255-1256` shows the latest backend start did attach `MultiChannelSink`, and no further `[dryrun sink]` lines were observed after that point. The anomaly therefore remains a historical missed-delivery risk for the two high events already logged as `sent`, not an active dryrun stream in the current process.

## Severity

sev2. High-severity events can appear delivered in `delivery_log` while only being printed by `LogSink`; the later `21:49` restart suggests the current process may be corrected, but the affected high events were not automatically replayed.

## Date Observed

2026-04-21T14:10:40Z
