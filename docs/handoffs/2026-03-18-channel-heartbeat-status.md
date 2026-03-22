# 2026-03-18 渠道运行态心跳替代 pid 判活

## 背景

- 控制台后端原先通过读取 `data/runtime/*.pid` 并执行 `kill -0` 判断 Discord / Feishu / Telegram / iMessage 是否存活。
- 这种方式依赖外部 pid 文件正确维护；一旦 pid 文件陈旧、缺失或由包装进程写入，状态展示就会失真。

## 本次变更

- 在 `crates/hone-core/src/heartbeat.rs` 新增共享 heartbeat 模块：
  - 统一 runtime 目录推导
  - 定义 `*.heartbeat.json` 文件结构，包含 `channel`、`pid`、`started_at`、`updated_at`
  - 提供 30 秒一次的后台续期任务，以及进程退出时的文件清理
- 在 `bins/hone-discord`、`bins/hone-feishu`、`bins/hone-telegram`、`bins/hone-imessage` 启动时接入 heartbeat。
- `crates/hone-web-api/src/routes/meta.rs` 改为读取 heartbeat，并按 `updated_at` 是否在 75 秒内判断 `running`/`stopped`。
- `bins/hone-desktop` 不再为渠道 sidecar 维护 `runtime/*.pid`；停止或重启 sidecar 时会清理对应 heartbeat 文件。
- `ChannelStatusInfo` 新增 `last_heartbeat_at` 字段，前端类型已同步。

## 结果

- 控制台现在展示的是“最近是否持续收到该进程的心跳”，而不是“某个 pid 文件里写过什么”。
- 心跳中仍保留 pid，运行中的渠道状态详情会附带 pid；心跳超时时也会保留最近一次 pid 和 last seen，便于排障。
- 旧的 `launch.sh` / `restart_hone` 仍可继续使用自己的 pid 文件做启动/停止管理，但控制台状态展示已不再依赖这些 pid 文件。

## 验证

- `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`
- `cargo test -p hone-core -p hone-web-api`

## 后续关注

- 目前 Web 自身仍直接显示当前进程 pid，不走 heartbeat；如果后续要统一“所有服务都展示 last heartbeat”，可以再给 `hone-console-page` 补同样的 heartbeat。
- 若未来要在 UI 上直接展示 “最后心跳时间”，前端已经有 `lastHeartbeatAt` 字段可直接消费。
