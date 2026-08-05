# Codex ACP 缺失 Rollout 安全恢复

- title: Codex ACP 缺失 Rollout 安全恢复
- status: done
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

## Results

- 实现提交 `f819584cff2f5b386c89f0791f1488c149ad3dfe` 已进入 `main`；CI、Secret Scan 与不可变 GHCR runtime image 均通过。
- 生产 Caris 旧绑定只备份并移除了三个 ACP metadata 字段，聊天历史、用户消息与附件未改；其余 153 个陈旧绑定没有批量清除，将只在未来同 ID missing-rollout 的结构化证明下按需恢复。
- 精确 release `/opt/hone/releases/f819584cff2f5b386c89f0791f1488c149ad3dfe-ghcr-runtime` 已上线，`/api/meta` 回报同一 SHA、`ghcr_linux_oci`、健康 PostgreSQL/OSS、cloud authority 为真、本地持久依赖为 0，服务 `NRestarts=0`。
- 真实管理员 Caris Life Sciences 财报分析完成 Skill、数据与 Web 证据核验，并生成 `Caris_Life_Sciences_Q1_2026_Financial_Analysis.pdf-2ccf2ec2.pdf`；服务重启后历史报告与下载卡仍存在，两次点击均显示“已开始下载”。
- 首次 cutover 暴露系统盘在 staging 后达到 100%，effective config 无法落临时文件；切回旧 symlink 后仍受同一容量问题影响。清理五个已被替代、可从 GHCR 重建的旧 immutable release 后，系统盘恢复为 83%（约 5GB 可用），旧版恢复健康，再完成最终切换。当前直接回滚点保留为 `9d64c5967bf74a5126948c7b49f6b918128f951a-ghcr-runtime`。
