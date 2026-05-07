# Bug: 飞书定时任务重复投递

- **发现时间**: 2026-03-25
- **严重等级**: P1
- **状态**: 已于 2026-03-25 修复

## 现象

飞书定时任务在同一个调度窗口内可能向同一用户重复发送多条内容相同的消息。

## 根因

1. `data/cron_jobs/` 中存在内容与 actor 文件名不匹配的旧 cron 文件副本，调度器此前会把这些副本也当成有效任务源。
2. 调度器此前没有对跨文件重复的同一 job 做扫描期去重。
3. 飞书发送链路此前为每次投递都生成随机 `uuid`，即使同一 job 在同一窗口被重复触发，飞书侧也会把它们当成全新的消息。

## 修复

1. `memory/src/cron_job.rs`
   - 跳过文件名与 `actor.storage_key()` 不匹配的 cron 文件。
   - 在单次 `get_due_jobs()` 扫描中，对同一 `channel + job_id + channel_target` 的任务做去重。
2. `crates/hone-scheduler/src/lib.rs`
   - 为每次调度事件生成稳定的 `delivery_key`。
3. `bins/hone-feishu/src/handler.rs`
   - 对定时任务发送增加进程内 15 分钟去重拦截。
   - 为定时任务的每个消息分片生成稳定的幂等 `uuid`，避免重复触发时再次真正发出。

## 验证

- `cargo test -p hone-memory`
- `cargo test -p hone-scheduler`
- `cargo test -p hone-feishu`
