# Bug: Telegram startup GetMe timeout leaves dead pid and heartbeat residue

## Summary

`telegram.enabled=true` 的最新一次重启里，`hone-telegram` 在 `bot.get_me()` 阶段超时后直接 `std::process::exit(1)`，导致进程已经退出，但 `data/runtime/telegram.pid` 与 `data/runtime/telegram.heartbeat.json` 仍残留并指向 dead pid；巡检会把它识别成 enabled 渠道僵尸 pid，后续如果无人处理还会很快演变成陈旧 heartbeat。

## Observed Symptoms

- `config.yaml:40-41` 启用了 Telegram：

```text
telegram:
  enabled: true
```

- 同一轮 `2026-04-24 18:23:46-18:23:55 +08:00` backend 已正常装配 event-engine sink，并启动 Telegram channel poller，说明不是整组 backend 没起来：

```text
data/runtime/logs/web.log.2026-04-24:2125:[2026-04-24 18:23:46.815] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-24:2144:[2026-04-24 18:23:46.822] INFO  telegram channel poller starting
data/runtime/logs/web.log.2026-04-24:2222:[2026-04-24 18:23:50.545] INFO  🚀 Hone Telegram Bot 启动
data/runtime/logs/web.log.2026-04-24:2232:[2026-04-24 18:23:55.565] ERROR 无法获取 Telegram Bot 信息: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetMe): error trying to connect: operation timed out
```

- 巡检时 `telegram.pid` 仍存在，但 `ps -p` 已无对应进程：

```text
$ cat data/runtime/telegram.pid
63694

$ ps -p 63694 -o pid=,etime=,comm=
# no output / exit code 1
```

- heartbeat 文件也残留在磁盘上，并且仍指向同一个 dead pid：

```text
$ cat data/runtime/telegram.heartbeat.json
{
  "channel": "telegram",
  "pid": 63694,
  "started_at": "2026-04-24T10:23:50.560303Z",
  "updated_at": "2026-04-24T10:23:50.563648Z"
}
```

- disabled 渠道在同一轮启动里明确记录了 `enabled=false` 跳过分支，而 Telegram 没有这类 WARN，因此这不是“巡检误把 disabled channel 当异常”：

```text
data/runtime/logs/web.log.2026-04-24:2202:[2026-04-24 18:23:48.506] WARN  discord.enabled=false，Discord Bot 不会启动。
data/runtime/logs/web.log.2026-04-24:2211:[2026-04-24 18:23:49.512] WARN  feishu.enabled=false，Feishu 渠道 不会启动。
```

## Hypothesis / Suspected Code Path

`crates/hone-channels/src/bootstrap.rs:61-72` 会在真正进入 Telegram 业务逻辑前先启动 `ProcessHeartbeat`。也就是说，只要 channel 进程过了 enabled / lock 检查，就会先写 heartbeat 初始快照：

```rust
let heartbeat = match hone_core::spawn_process_heartbeat(&core.config, channel) {
    Ok(heartbeat) => heartbeat,
    Err(err) => {
        error!("无法启动 {display_name} heartbeat: {err}");
        std::process::exit(1);
    }
};

ChannelRuntimeBootstrap {
    core,
    _process_lock: process_lock,
    _heartbeat: heartbeat,
}
```

`bins/hone-telegram/src/handler.rs:171-192` 随后在 `bot.get_me()` 失败时直接 `std::process::exit(1)`。这会绕过正常析构流程，因此不会触发 heartbeat 清理：

```rust
pub(crate) async fn run() {
    let runtime = hone_channels::bootstrap_channel_runtime(
        "telegram",
        "Telegram Bot",
        hone_core::PROCESS_LOCK_TELEGRAM,
        |config| config.telegram.enabled,
    );
    let core = runtime.core;

    let token = core.config.telegram.bot_token.trim().to_string();
    let bot = Bot::new(token);
    let me = match bot.get_me().await {
        Ok(me) => me,
        Err(e) => {
            error!("无法获取 Telegram Bot 信息: {e}");
            std::process::exit(1);
        }
    };
```

`crates/hone-core/src/heartbeat.rs:100-105` 只有在 `ProcessHeartbeat` 正常 drop 时才会删除 heartbeat 文件；`std::process::exit(1)` 不会运行这个 `Drop`：

```rust
impl Drop for ProcessHeartbeat {
    fn drop(&mut self) {
        self.task.abort();
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_file(&self.error_path);
    }
}
```

`launch.sh:224-240` 也会在子进程刚启动时立即写 pid 文件，只检查“1 秒后是否仍活着”。这次 `GetMe` 超时发生在约 5 秒后，已经晚于 startup probe，所以 pid 文件会被保留下来：

```bash
"$path" &
pid=$!
printf -v "$pid_var" '%s' "$pid"
echo "$pid" > "$(pid_file "$service_name")"

sleep 1
if ! pid_is_running "$pid"; then
  local status=0
  wait "$pid" || status=$?
  if [[ "$status" -eq "$CHANNEL_DISABLED_EXIT_CODE" ]]; then
    echo "[INFO] ${service_name} skipped by active config."
    rm -f "$(pid_file "$service_name")"
```

综合来看，这条链路更像是：

1. Telegram enabled，bootstrap 先写 heartbeat 初始快照。
2. `launch.sh` 写入 `telegram.pid`，1 秒后进程仍未退出，因此 startup probe 通过。
3. `bot.get_me()` 在几秒后超时，进程用 `std::process::exit(1)` 直接退出。
4. pid 文件和 heartbeat 文件都没有被后置清理，于是留下 dead pid + frozen heartbeat 残留。

## Evidence Gap

- 本轮巡检遵循只读约束，没有重启 Telegram，也没有主动调用 Telegram API；因此这里只能证明“运行结果层面留下了 dead pid/heartbeat 残留”，不能在本轮直接复现完整启动时序。
- 还缺一份 launch 层或 desktop supervisor 层的显式日志，说明它是否把这次 `hone-telegram` 退出视为 fatal restart 失败，还是只把残留文件留给下游巡检/状态页兜底。
- 若要最终坐实修复点，需要一次受控复现，记录 `launch.sh` 写 pid、`bot.get_me()` 超时、以及进程退出后 heartbeat/pid 是否仍残留的完整时序。

## Severity

`sev2`。理由：这是 enabled Telegram 渠道的真实不可用，巡检已经看到 dead pid 和启动期 `GetMe` 超时；用户侧会丢失 Telegram 入站处理能力，运维侧还会被残留 pid/heartbeat 误导。之所以不是 `sev1`，是因为当前 backend/event-engine 主进程仍在运行，`event engine sink: MultiChannelSink 已装配` 也存在，尚未看到整个 event-engine 主链路同时停摆。

## Date Observed

`2026-04-24T10:29:32Z`
