# Bug: 取消所有自动提醒后定时与心跳任务仍可能继续触发

- 发现时间：2026-07-27 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed
- GitHub Issue：无，非 P1

## 用户反馈

- 用户明确要求取消所有自动提醒，但之后仍有自动任务触发。
- 这是重复出现的功能性问题；仅关闭一类推送或仅删除单个任务，会让用户误以为所有自动通知都已经停止。

## 根因

1. `notification_prefs(action="disable")` 只关闭 event-engine 的事件推送，不会删除 cron / heartbeat 任务。
2. `cron_job` 原来只有单任务 `remove`，没有 actor 级原子 `remove_all`；模型需要循环删除，容易遗漏。
3. `CronJobStorage::remove_job` 吞掉 `save_jobs` 失败，持久化未成功时仍可能返回成功。
4. scheduler 事件入队后，渠道执行和发送前没有重新确认任务是否仍存在；用户在任务排队或模型运行期间取消，旧事件仍可能完成投递。

## 修复

- 新增 `cron_job(action="remove_all")`：actor-scoped、幂等地删除当前用户全部 cron 与 heartbeat 任务，并清理对应待确认更新。
- 新增 `notification_prefs(action="disable_all")`：关闭事件即时/摘要推送，清空 digest slots，并删除当前用户全部 cron / heartbeat 任务。
- 增加 fallible `try_load_jobs` 与 `remove_all_jobs`；删除读写失败会向上返回错误，不再伪报成功。
- scheduler 在模型执行前、执行完成后和各渠道发送前重新检查 durable job；已取消任务按 `job_cancelled / skipped_cancelled` 静默收口。
- 保留 one-shot 调度语义：一次性任务在成功 claim 后虽已标记停用，仍允许完成当前唯一一次运行。
- prompt 只补充动作路由，不改变既有回答格式；finalizer 在模型未产出可见正文时回收明确的取消结果。

## 验证

- `cargo test -p hone-memory remove_all_jobs -- --nocapture`
- `cargo test -p hone-tools --lib -- --nocapture`
- `cargo test -p hone-scheduler -- --nocapture`
- `cargo test -p hone-channels execute_scheduler_event_ --lib -- --nocapture`
- `cargo test -p hone-channels response_finalizer::tests --lib -- --nocapture`
- `cargo check -p hone-feishu -p hone-telegram -p hone-discord -p hone-web-api --all-targets`

覆盖 actor 隔离、幂等取消、损坏持久化失败、不再执行已取消队列事件、one-shot 当前 claim 正常完成及统一用户确认文案。

## 风险与后续复核

- 本修复不批量改动现有生产用户数据；只有用户再次明确发出“取消所有自动提醒”时，才执行 actor 级删除。
- 部署后需在下一轮真实用户取消请求和 scheduler 窗口复核：同 actor 不再出现新 `completed/sent`，其他 actor 的任务继续正常运行。
