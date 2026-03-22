# Handoff：定时任务跨进程竞态修复

日期：2026-03-14

## 问题描述

- iMessage 定时消息完全不发送
- 飞书定时消息偶尔不发送

## 根本原因

系统同时运行多个进程（`hone-console-page`、`hone-feishu`、`hone-telegram`），每个进程都独立创建 `HoneScheduler`，共享同一个 `data/cron_jobs/` 文件目录。

**竞态时序（示例）**：
```
hone-console-page scheduler（每分钟检查一次）:
  get_due_jobs() → 读到所有任务（包括 feishu/telegram 的）
  → mark_job_run() → 写入 last_run_at 到 JSON 文件
  → send event → handle_scheduler_events 丢弃（channel != imessage）

hone-feishu scheduler（稍晚几十毫秒运行）:
  get_due_jobs() → 读文件 → last_run_at 已被 console-page 写入
  → already_ran = true → 跳过 → 飞书任务永久丢失
```

哪个进程先 `mark_job_run` 是随机的，由操作系统调度决定，因此失效表现为"偶尔"。

## 修复内容

### Fix 1：Scheduler 按 channel 过滤（核心）

- `memory/src/cron_job.rs`：`get_due_jobs` 新增 `channels: &[&str]` 参数，只返回 channel 在列表中的任务
- `crates/hone-scheduler/src/lib.rs`：`HoneScheduler` 新增 `channels: Vec<String>` 字段，`check_due_jobs` 传入过滤器
- `crates/hone-channels/src/core.rs`：`create_scheduler` 新增 `channels: Vec<String>` 参数
- 各 bin 调用时传入自己的渠道：
  - `hone-console-page` → `["imessage", "web_test", ""]`
  - `hone-feishu` → `["feishu"]`
  - `hone-telegram` → `["telegram"]`

### Fix 2：先发送事件，成功后再标记（一并完成）

原来顺序：mark → send（若 send 失败，任务永久丢失）
现在顺序：send → 成功后 mark（若 send 失败，任务在 DUE_WINDOW 内会被下轮重拾）

### Fix 3：iMessage HTTP 回调健壮性

- 复用 `AppState.http_client`（不再每次 `reqwest::Client::new()`）
- 加入 1 次重试 + 2 秒间隔
- 失败日志增加「请确认 hone-imessage 进程正在运行」提示
- 2 次均失败时升级为 `error!` 日志

### 顺带修复：预存在编译错误

`bins/hone-console-page/src/main.rs` 中 `core` 被移入 `AppState` struct 后，同一初始化块仍使用 `core.config.web.auth_token`，导致 use-after-move 编译错误。提取 `bearer_token` 到 struct 初始化之前修复。

## 验证

- `cargo check` 各受影响 crate 全部通过
- `cargo test -p hone-memory` 11/11 测试通过（含 `due_job_and_mark_run_prevents_immediate_duplicate`）

## 未覆盖 / 注意事项

- `hone-discord` 目前没有独立 scheduler，无需修改
- DUE_WINDOW_MINUTES = 5：若进程重启或发送失败，任务最多重试 5 次（5 分钟内），之后 `already_ran` 检查会阻止重复触发
- `hone-imessage` HTTP 服务启动稍慢时，首次回调可能还是失败；重试间隔 2 秒可覆盖大多数情况
