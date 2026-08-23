- title: Inactive User Scheduled Push Cleanup 2026-08-23
- status: in_progress
- created_at: 2026-08-23
- updated_at: 2026-08-23
- owner: Codex
- related_files:
  - docs/current-plan.md
  - crates/hone-tools/src/cron_job_tool.rs
  - docs/handoffs/2026-08-23-inactive-user-scheduled-push-cleanup.md
- related_docs:
  - docs/runbooks/backend-deployment.md
  - docs/invariants.md

## Goal

盘点生产 GCE 上仍有启用 cron/heartbeat 定时任务、但连续七天没有真实外部用户主动消息的 actor；安全停用这些任务并向对应用户发送一次说明，保留任务定义以便后续恢复。

## Scope

- 以 PostgreSQL `cloud_sessions`、`conversation_quota` 与 `cloud_cron_jobs` 为权威真相源；主动消息排除 scheduler/heartbeat、job metadata、compact 快照和旧 scheduler envelope。
- 排除管理员、测试 actor、group scope；只把命中直接用户的任务更新为 `enabled=false`，不删除任务，不关闭 event-engine 即时推送或 digest prefs。
- 给暂停任务追加 `paused_inactive_7d_20260823` 标签，并让 `cron_job update` 接受 `enabled`，确保用户后续在对话里要求恢复时可以原样重新启用。
- 每个命中 actor 只发一条通知；Web 使用 durable push，飞书/iMessage 要求渠道 ACK，失败项单独记录。

## Validation

- 固定 `Asia/Shanghai` 七天口径，变更前两次独立查询要求候选集合一致；交叉核对未计 scheduler 的 conversation quota。
- 事务内重新计算资格并验证 actor/job 计数；停用前保留权限受限的生产回滚快照。
- 停用后要求候选任务全部 `enabled=false`、定义仍存在、非候选启用任务未变化且没有候选任务处于 active run。
- 通知后核对每个 actor 的真实渠道 ACK 或 Web durable delivery；失败项不得误报成功。
- 为 conversational `cron_job` 启停参数补单元测试；生产部署遵循 revision provenance、空闲窗口和发布后探活要求。

## Documentation Sync

- 执行期间更新本计划和 `docs/current-plan.md`。
- 完成后新增 handoff、归档本计划并更新 `docs/archive/index.md`。
- 恢复能力只补齐既有 `CronJobUpdate.enabled` 的工具暴露，不改变模块边界或长期约束，因此无需更新 `docs/repo-map.md` / `docs/invariants.md`。

## Risks

- 会话压缩会移除旧消息原文，必须用不计 scheduled task 的 `conversation_quota` 保守排除近期真实交互。
- 盘点与停用之间用户可能主动发言；生产更新必须在 serializable 事务中重新计算。
- iMessage 只能从已登录 Messages.app 的 macOS 发出；GCE 本身不能完成该渠道 ACK。
