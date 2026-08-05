# Codex ACP 缺失 Rollout 安全恢复

- title: Codex ACP 缺失 Rollout 安全恢复
- status: in_progress
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-channels/src/runners/acp_common/protocol.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/runbooks/backend-deployment.md`

## Goal

恢复生产 Caris Life Sciences 财报分析，并消除同类陈旧 Codex ACP 绑定：当且仅当受信任适配器在 `session/resume`、首个 prompt 之前明确证明所绑定的同一 thread ID 没有 rollout 时，创建并持久化一个替代 native session；其它 resume、传输、认证和超时错误继续 fail-closed。

## Scope

- 保留每个确定性 Hone SessionIdentity 单一有效 Codex native ID 的约束。
- 结构化保留并清洗 ACP `error.data.details`，只匹配 `no rollout found for thread id <exact persisted id>`。
- 替代 ID 必须在首个 `session/prompt` 前写入权威 session storage；不自动重发已经发出的 prompt。
- 生产先备份受影响的 session metadata，再删除确认无本地 rollout 的旧绑定；不删除聊天历史或用户消息。
- 部署精确 revision 后，用真实 Caris 财报分析验证 Skill、证据检索、PDF OSS 持久化及聊天下载。

## Validation

- 外部 stdio fixture 覆盖精确 missing-rollout 恢复、ID 不匹配拒绝、普通 resume 错误不新建、checkpoint-before-prompt。
- ACP 协议测试覆盖 `error.data.details` 的有界清洗与可诊断错误。
- 运行 `cargo test -p hone-channels --lib`、workspace check、Web/CI-safe 回归与改动文件格式检查。
- 生产更新前后检查 exact revision、零活跃会话、PostgreSQL/OSS/cloud-authoritative、服务 restart count 和本地 Codex thread/云端绑定一致性。
- 真实 Caris 运行不得出现 `no rollout found`；最终 PDF 在服务重启后仍可从历史聊天下载。

## Documentation Sync

- 更新 `docs/invariants.md`、`docs/decisions.md` 和 `docs/adr/0002-agent-runtime-acp-refactor.md`，记录“适配器明确证明同 ID 不存在”的唯一重绑定例外。
- 更新 `docs/runbooks/backend-deployment.md`，记录 rollout 清理后的绑定审计与修复边界。
- 完成后新增 handoff、更新 `docs/archive/index.md`，并把本计划移入 `docs/archive/plans/`。

## Risks / Open Questions

- 不能把 `Internal error`、超时、权限或任意 stderr 文本当作 rollout 不存在证明。
- 旧 native page 可能仍在外部任务列表，但本机适配器已无法恢复；替代只恢复可执行连续性，不能重建丢失的 native history。
- 生产批量修复必须精确排除当前本地仍存在的 native ID，并保留可回滚快照。
