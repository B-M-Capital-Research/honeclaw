# Inactive User Scheduled Push Cleanup

- title: Inactive User Scheduled Push Cleanup
- status: done
- created_at: 2026-08-23
- updated_at: 2026-08-23
- owner: Codex
- related_files:
  - crates/hone-tools/src/cron_job_tool.rs
  - docs/archive/plans/inactive-user-scheduled-push-cleanup-2026-08-23.md
- related_docs:
  - docs/runbooks/backend-deployment.md
- related_prs: direct `main` commit `011d73118dbf1d6b0cc09a793882dc796f23aa9f`; no PR, release, or tag

## Summary

生产按 `Asia/Shanghai` 七天窗口筛出 26 个直接用户、45 个启用中的 cron/heartbeat 任务。筛选同时使用保留的真实外部 user ingress 和不计 scheduled task 的 `conversation_quota`，并排除管理员、测试 actor、group scope、compact 快照与 scheduler envelope。45 个任务在 serializable 事务中全部改为 `enabled=false`，定义未删除，event-engine prefs 未变。

## What Changed

- 暂停任务追加 `paused_inactive_7d_20260823` 标签；变更前在 managed host 留存 owner-only 的 89 行启用任务回滚快照，SHA-256 为 `617c574aab3fb7d13d595de4fb3c3f700f5fdab693d07718467804b4081aaee0`。
- 飞书 18 位用户全部取得唯一 message-id ACK；Web 6 位用户各写入一条 durable push。2 位历史 iMessage 用户因生产无可用 macOS Messages.app 通道未送达，未伪报成功。
- 一位飞书用户在通知后主动完成两轮对话并删除自己的 2 个任务；因此最终稳定态为 25 位用户保留 43 个暂停任务，而不是恢复已被用户删除的定义。
- `cron_job update` 新增 `enabled=false/true`，并允许按名称定位已暂停任务，用户后续说“恢复定时推送”即可原样启用。
- 精确 revision `011d7311…` 已通过 Runtime Image `32612578070` 构建并部署；前一 revision `36785584…` 保留为 immediate rollback。

## Verification

- 生产更新硬校验：26 actors / 45 jobs 命中并更新；更新时 active runs 为 0；非候选启用任务保持 44 个。
- 最终读回：25 actors / 43 tagged jobs，`enabled=true` 为 0；飞书 17/30、Web 6/9、iMessage 2/4；Web durable notices 6/6。
- 通知证据：飞书 ACK 审计文件为 mode `0600`，18 行且 18 个唯一 message-id hash；通知后主动操作用户的当日 conversation quota 为 2。
- 隔离临时 PostgreSQL 上 `cron_job_tool_add_list_update_remove_flow` 1/1 通过；临时集群已停止并删除。Rust 文件 `rustfmt --check` 与 `cargo test -p hone-tools --no-run` 通过。
- GitHub Secret Scan、CodeQL、Release Cache Warm、Runtime Image 成功。CI 唯一失败是父提交已存在的 `soul_prompt_keeps_the_full_investment_contract` 长度预算，和本次 cron diff 无关。
- 生产 `/api/meta` 精确报告 `011d7311…` / `ghcr_linux_oci`，PostgreSQL/OSS healthy、cloud authoritative、local durable dependency 0；active chats 0、`NRestarts=0`、public auth 401、近端无 panic/fatal，integrated Feishu stream 已重连。

## Risks / Follow-ups

- iMessage 两位用户的通知仍未送达；若未来恢复 macOS channel，应按 paused tag 补发一次，发送前先确认用户没有通过其它渠道恢复或删除任务。
- 本次只暂停用户自建 cron/heartbeat；event immediate push 与 digest prefs 按用户要求保持原状。
- 会话原文会被 compaction 移除；未来重复执行同类清理必须继续使用 conversation quota 或新的不可压缩 ingress ledger 做保守交叉验证。

## Next Entry Point

用户要求恢复时调用 `cron_job(action="list")` 找到带暂停标签的定义，再按精确 `job_id` 调用 `cron_job(action="update", enabled=true)`；多任务恢复应逐条确认成功并受现有启用任务上限约束。生产二进制回滚入口是保留的 `3678558483628b605aa927cfa168539a22eca84a-ghcr-runtime`。
