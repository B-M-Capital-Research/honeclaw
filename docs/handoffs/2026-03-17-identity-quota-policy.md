# 2026-03-17 Identity 限额策略

## 结果

- 新增 `memory/src/quota.rs`，使用按 `ActorIdentity` 分目录、按北京时间日期分文件的 JSON 存储对话额度，字段为 `success_count` 和 `in_flight`
- `AgentSession::run()` 在落用户消息和创建 runner 前先做额度预占；成功回复后提交，失败/超时后释放
- `AgentRunOptions` 新增 `quota_mode`，scheduler 统一走 `ScheduledTask`，不消耗 20/天额度
- cron enabled 上限改为 5，并收口到 `CronJobStorage` 的 add / update / toggle 路径；管理员可绕过
- Web chat 在失败路径补发 `run_finished(false)`；Web cron 命中限额返回 `429`
- 前端 API 错误优先展示后端 JSON `error` 字段

## 存储位置

- 对话额度：`storage.conversation_quota_dir`，默认 `./data/conversation_quota`
- 会话：`storage.sessions_dir`
- 定时任务：`storage.cron_jobs_dir`

## 验证

- `cargo test -p hone-memory`
- `cargo test -p hone-channels`
- `cargo test -p hone-tools`
- `cargo test -p hone-web-api`
- `bun run typecheck:web`

## 风险与未覆盖项

- Web 侧没有新增 quota 查询接口，当前只在命中上限时提示，不展示剩余额度
- cron 限额错误仍以统一错误文案协议传递，`429` 判定依赖该文案保持一致
