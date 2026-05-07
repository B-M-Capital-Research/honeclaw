# Bug: event-engine enabled channel heartbeat write hit ENOSPC

状态：`Closed`

关闭原因：2026-04-25 不可复现。仅有 2026-04-21 21:28 一条 ENOSPC 警告，进程随后自愈、磁盘也回到 ~46Gi available；后续巡检窗口未再出现 ENOSPC。「无 durable unhealthy state」是设计取舍——磁盘满本就是宿主机问题，加 unhealthy 状态会把所有 channel 一起拖死，不划算。

## Summary

An enabled runtime channel lost a heartbeat write tick because the heartbeat snapshot write returned `No space left on device`.

## Observed Symptoms

- `data/runtime/logs/web.log:1251`

```text
[2026-04-21 21:28:01.828] WARN  failed to write heartbeat: No space left on device (os error 28)
```

- The same runtime later recovered enough to keep running: `data/runtime/telegram.heartbeat.json` was fresh during巡检 (`updated_at=2026-04-21T14:08:34Z`, `pid=75095`), and `ps -p 75095` showed the Telegram process alive.
- Current disk state during巡检 was no longer full (`df -h data/runtime` showed about `46Gi` available), so the observed failure is a transient or recently cleared ENOSPC event rather than an active full-disk state.

## Hypothesis / Suspected Code Path

Suspected path: `crates/hone-core/src/heartbeat.rs:101` logs write failures but does not surface a durable unhealthy state beyond the warning.

```rust
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let http_client = heartbeat_http_client(console_token.as_deref());

        loop {
            ticker.tick().await;
            let now = Utc::now();
            let snapshot = ProcessHeartbeatSnapshot {
                channel: channel.clone(),
                pid,
                started_at,
                updated_at: now,
            };
            if let Err(err) = write_snapshot(&task_path, &snapshot) {
                tracing::warn!(
                    channel = %channel,
                    pid,
                    path = %task_path.display(),
                    "failed to write heartbeat: {err}"
                );
            }
```

`crates/hone-core/src/heartbeat.rs:125` writes the full JSON snapshot directly to the heartbeat path, so a full disk leaves the previous heartbeat file in place and only emits the warning.

```rust
fn write_snapshot(path: &Path, snapshot: &ProcessHeartbeatSnapshot) -> io::Result<()> {
    let content = serde_json::to_vec_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, content)
}
```

## Evidence Gap

- Need disk-usage telemetry or filesystem logs around `2026-04-21T13:28:01Z` to confirm what filled the volume.
- Need a follow-up check of other runtime writers (`events.jsonl`, `events.sqlite3`, digest buffers, web log rotation) around the same timestamp to determine whether heartbeat was the only affected write.
- This巡检 did not use sudo or OS-level diagnostics, per task constraints.

## Latest巡检 Update

- 2026-04-22T06:10:14Z: enabled Telegram heartbeat was fresh: `data/runtime/telegram.heartbeat.json` had `pid=75490`, `started_at=2026-04-22T03:58:06.994362Z`, and `updated_at=2026-04-22T06:10:07.098463Z`; `ps -p 75490` showed `/Users/bytedance/Library/Caches/honeclaw/target/debug/hone-telegram` alive with `etime=02:12:10`.
- `data/runtime/backend.pid=75329` was alive with `etime=02:12:14`; the Telegram heartbeat `started_at` was within the same process lifetime, so no enabled-channel heartbeat跨重启残留 was observed.
- `data/runtime/discord.heartbeat.json` remains stale from `2026-04-16`, and `discord.pid` / `feishu.pid` / `imessage.pid` point to dead processes, but current startup logs include `discord.enabled=false`, `feishu.enabled=false`, and `imessage.enabled=false`; those stale files were not counted as enabled-channel failures.
- The original `data/runtime/logs/web.log:1251` `failed to write heartbeat: No space left on device (os error 28)` remains the only ENOSPC heartbeat write warning in this scan.
- 2026-04-22T02:09:10Z: all pid files under `data/runtime/*.pid` resolved to live processes; `backend.pid=75069` had `etime=12:19:41` and `telegram.pid=75095` had `etime=12:19:37`.
- Enabled Telegram heartbeat was fresh: `data/runtime/telegram.heartbeat.json` had `started_at=2026-04-21T13:49:34.236871Z` and `updated_at=2026-04-22T02:09:04.218383Z`, matching the live Telegram process lifetime within normal drift.
- `data/runtime/discord.heartbeat.json` remained stale from `2026-04-16`, but `config.yaml` has `discord.enabled=false`, so it was not counted as an enabled-channel heartbeat failure. `df -h data/runtime` showed `68Gi` available, and no new ENOSPC warning was found after `data/runtime/logs/web.log:1251`.
- 2026-04-21T22:08:04Z: the latest enabled Telegram heartbeat was fresh: `data/runtime/telegram.heartbeat.json` had `pid=75095`, `started_at=2026-04-21T13:49:34.236871Z`, and `updated_at=2026-04-21T22:07:34.297115Z`.
- `ps -p 75095` showed `/Users/bytedance/Library/Caches/honeclaw/target/debug/hone-telegram` alive with `etime=08:18:02`, matching the heartbeat lifetime within normal drift.
- Disabled Discord still had a stale heartbeat file from `2026-04-16`, but `data/runtime/logs/web.log` repeatedly logged `discord.enabled=false`, so it was not counted as an enabled-channel heartbeat failure.
- The original `data/runtime/logs/web.log:1251` ENOSPC warning remains the only heartbeat write failure in the current 24h scan.

## Severity

sev2. The process stayed alive and the heartbeat later refreshed, but a full-disk heartbeat write failure can make enabled channels appear stale or healthy based on old data, and the current code does not persist an explicit degraded state.

## Date Observed

2026-04-21T14:10:40Z
