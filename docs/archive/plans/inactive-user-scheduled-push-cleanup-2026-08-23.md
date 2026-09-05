- title: Inactive User Scheduled Push Cleanup 2026-08-23
- status: archived
- created_at: 2026-08-23
- updated_at: 2026-08-23
- owner: Codex
- related_files:
  - crates/hone-tools/src/cron_job_tool.rs
  - docs/handoffs/2026-08-23-inactive-user-scheduled-push-cleanup.md
- related_docs:
  - docs/runbooks/backend-deployment.md
  - docs/invariants.md

## Goal

盘点生产 GCE 上仍有启用 cron/heartbeat 定时任务、但连续七天没有真实外部用户主动消息的 actor；安全停用这些任务并向对应用户发送一次说明，保留任务定义以便后续恢复。

## Completed Scope

- 以 PostgreSQL `cloud_sessions`、`conversation_quota` 与 `cloud_cron_jobs` 为真相源，排除 automation、compact、管理员、测试和 group actor。
- serializable 事务命中并暂停 26 位用户的 45 个任务，保留定义并追加恢复标签；未修改 event-engine prefs。
- 飞书 18/18 与 Web 6/6 通知成功；2 位 iMessage 因渠道不可用未送达并显式留档。
- 一位飞书用户收到通知后主动对话并删除 2 个任务；最终稳定态是 25 位用户、43 个保留且暂停的任务。
- conversational `cron_job` 已补齐暂停/恢复参数，精确 revision `011d7311…` 已部署生产。

## Verification

- 事务 guard、任务读回、渠道 ACK、Web durable push、隔离 PostgreSQL 单测、精确 runtime image、cloud authority/public auth/active chat/channel reconnect 均完成；完整证据见 handoff。

## Documentation Sync

- 已新增 handoff、更新 archive index 并从 current plan index 移除。
- 无模块边界或长期约束变化，因此无需更新 `docs/repo-map.md` / `docs/invariants.md`。

## Risks

- iMessage 补发必须等真实 macOS channel 恢复，并在发送前重新核对用户当前任务状态。
- 后续同类盘点不能只读压缩后的 session 原文，必须保留非 scheduled 的交互交叉证据。
