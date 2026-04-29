# Bug: disabled channels can leave stale pid files after skipped startup

## Summary

在本轮 `launch.sh --web` 重启后，`discord` 和 `feishu` 明确走了 `enabled=false` 跳过启动分支，但 `data/runtime/discord.pid` 与 `data/runtime/feishu.pid` 仍被保留，且指向已经不存在的进程，导致巡检把它们识别成新的僵尸 pid。

## Observed Symptoms

- 当前启动窗口里，日志明确显示两个 channel 没有启动：

```text
data/runtime/logs/web.log.2026-04-23:3276-3284
[2026-04-23 19:22:48.566] INFO  🚀 Hone Discord Bot 启动
...
[2026-04-23 19:22:48.569] WARN  discord.enabled=false，Discord Bot 不会启动。

data/runtime/logs/web.log.2026-04-23:3298-3306
[2026-04-23 19:22:49.677] INFO  🚀 Hone Feishu 渠道 启动
...
[2026-04-23 19:22:49.678] WARN  feishu.enabled=false，Feishu 渠道 不会启动。
```

- 但同一轮启动后，这两个 pid 文件仍存在，而且 mtime 就落在本次启动窗口；对比之下，`imessage.enabled=false` 这轮没有留下 `imessage.pid`：

```text
2026-04-23 19:22:47 +0800 data/runtime/discord.pid
2026-04-23 19:22:48 +0800 data/runtime/feishu.pid
2026-04-23 19:22:49 +0800 data/runtime/telegram.pid
absent: data/runtime/imessage.pid
```

- 巡检时读取 pid 文件并做 `ps -p`，发现 `discord.pid=98429`、`feishu.pid=98435` 都已经没有对应进程：

```text
2026-04-23T14:19:13Z-2026-04-23T14:26:54Z 巡检
data/runtime/discord.pid => 98429
data/runtime/feishu.pid => 98435
ps -p 98429 -o pid=,etime=,lstart=,command= => no process
ps -p 98435 -o pid=,etime=,lstart=,command= => no process
```

- 同一轮中 `telegram.pid=98446` 正常存活，说明这不是整组 pid 文件都无效，而是 disabled channel 的清理遗漏：

```text
data/runtime/telegram.pid => 98446
98446 02:59:36 Thu Apr 23 19:22:49 2026     /Users/bytedance/Library/Caches/honeclaw/target/debug/hone-telegram
```

## Hypothesis / Suspected Code Path

可疑主路径是 [`launch.sh:211`](../../launch.sh) 的 `start_hone_bin`。它在子进程启动后立刻写 pid 文件，只在“睡 1 秒后发现进程已退出”这个条件下才 `wait` 并删除 pid 文件；如果 disabled channel 在这 1 秒检查点仍被 `kill -0` 视为存活，或者在检查点之后才被 shell 回收，pid 文件就会残留。

```bash
start_hone_bin() {
  local bin_name="$1"
  local service_name="$2"
  local pid_var="$3"
  local path
  local pid

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

对应 child 侧 [`crates/hone-channels/src/bootstrap.rs:32`](../../crates/hone-channels/src/bootstrap.rs) 会在 `enabled=false` 时直接 `exit(CHANNEL_DISABLED_EXIT_CODE)`，因此 launch script 理论上应把这类 pid 文件清干净：

```rust
    hone_core::logging::setup_logging(&core.config.logging);
    info!("🚀 Hone {display_name} 启动");
    core.log_startup_routing(channel, &config_path);

    if !is_enabled(&core.config) {
        warn!("{channel}.enabled=false，{display_name} 不会启动。");
        std::process::exit(CHANNEL_DISABLED_EXIT_CODE);
    }

    let process_lock =
        match hone_core::acquire_runtime_process_lock(&core.config, process_lock_name) {
            Ok(lock) => lock,
```

## Evidence Gap

- 本轮只做只读巡检，没有给 `launch.sh` 增加额外 tracing，因此还不能百分百区分是“disabled child 在 `sleep 1` 检查点后才退出”，还是“已经退出但 shell 里的 `kill -0` 仍把未回收 child 视作存活”。
- 需要在 `start_hone_bin` 附近补一条 pid 生命周期日志，或用一次受控复现记录 `wait` / `kill -0` / `ps` 的时序，才能坐实是哪一种竞态。
- 当前证据已经足够证明结果层面的缺陷成立：本次启动后生成了新的 dead pid files，而不是单纯沿用旧残留。

## Severity

sev3。它不会直接中断已启用的 event-engine 链路，但会让巡检、健康检查和运维判断把 disabled channel 误识别成新的僵尸进程，增加误报并掩盖真正的运行态问题。

## Date Observed

2026-04-23T14:26:54Z

## Fix Update

- 2026-04-28: `launch.sh` 新增 `pid_is_zombie`，`start_hone_bin` 在启动后检查到 disabled child 已经成为 zombie 时会执行 `wait`，拿到 `CHANNEL_DISABLED_EXIT_CODE` 后删除对应 pid 文件。
- 这覆盖了 `kill -0` 对未回收子进程仍返回成功导致 pid 文件残留的竞态。
- 验证：`bash -n launch.sh`。
