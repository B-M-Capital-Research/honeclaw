# Earnings OpenCode signature 恢复安全分类修复

- title: Earnings OpenCode signature 恢复安全分类修复
- status: in_progress
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files:
  - crates/hone-core/src/tool_effect.rs
  - crates/hone-channels/src/tool_trace.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/agent_session/tests.rs
- related_docs:
  - docs/current-plan.md
  - docs/runbooks/opencode-setup.md
  - docs/handoffs/2026-08-10-earnings-opencode-signature-recovery.md

## Goal

恢复管理员财报前瞻/分析在 OpenCode + OpenRouter Gemini 返回精确 `400 Corrupted thought signature` 后的一次性全新会话重试，同时不放宽任何真实命令、未知工具或持久写入的自动重放边界。

## Scope

- 以 2026-08-10 RKLB 生产失败为边界样本：三次 `hone_data_fetch`、两次 `hone_web_search`、OpenCode 内置 `read` / `grep`，以及因 `bash` 未开放而被 OpenCode 明确转换成未执行 `invalid` 的记录。
- 为 dedicated earnings + verified OpenCode recovery 增加窄化分类：只接受 OpenCode 内置 `read` / `grep` / `glob`，以及 arguments 明确表明 `Model tried to call unavailable tool` 的 `invalid`。
- 真实 `bash`、`task`、未知名字、可执行 skill、持久用户/系统写入继续阻断自动重试。
- 不改变普通对话、其它 runner、模型路由、财报证据/PDF 完成契约或最大 token 配置。

## Validation

- `hone-core` 工具效果分类单元测试覆盖允许与拒绝边界。
- `hone-channels` 财报集成回归复现生产工具序列，并证明 signature failure 只 fresh retry 一次。
- 执行相关 Rust 格式、定向测试、crate check；部署精确 revision 后核对服务健康、日志中的 `agent.run.retry` 和真实 RKLB PDF 成功闭环。

## Documentation Sync

- 实施期间更新 `docs/current-plan.md` 与本计划。
- 完成后把本计划归档到 `docs/archive/plans/`，新增 handoff 并更新 `docs/archive/index.md`。
- 本修复恢复既有 D-2026-08-04 财报 isolated replay 契约，不改变长期架构决策；如实现扩大既有语义，再更新 `docs/decisions.md`。

## Risks / Open Questions

- `invalid` 只有在明确表示目标工具未开放、因此未执行时才可重放；不能把任意 malformed/unknown tool 当成只读。
- OpenCode 内置文件读取名字只在 dedicated earnings + OpenCode 边界使用，不能升级为所有 runner 的全局安全断言。
- 真实生产验证会产生一次 Gemini/OpenRouter 费用；只在代码、测试、部署和余额门均通过后运行一次。
