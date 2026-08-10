# Earnings OpenCode signature 与结构结果恢复修复

- title: Earnings OpenCode signature 与结构结果恢复修复
- status: in_progress
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files:
  - crates/hone-channels/src/tool_trace.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/agent_session/tests.rs
  - crates/hone-channels/src/runners/opencode_acp.rs
- related_docs:
  - docs/current-plan.md
  - docs/runbooks/opencode-setup.md
  - docs/runbooks/backend-deployment.md
  - docs/handoffs/2026-08-10-earnings-opencode-signature-recovery.md

## Goal

恢复管理员财报前瞻/分析在 OpenCode + OpenRouter Gemini 返回精确 `400 Corrupted thought signature` 或 renderer 明确无副作用预检失败后的可靠续跑，同时不放宽任何真实命令、未知工具或持久写入的自动重放边界。无 compact 时优先在同一 ACP 会话携带精确校验错误继续；必须重建隔离会话时，把上一轮未落盘的完整报告草稿与校验错误作为服务端恢复材料带入，避免从零重做。

## Scope

- 以 2026-08-10 RKLB 首次生产失败为 signature 边界样本：三次 `hone_data_fetch`、两次 `hone_web_search`、OpenCode 内置 `read` / `grep`，以及因 `bash` 未开放而被 OpenCode 明确转换成未执行 `invalid` 的记录。
- dedicated earnings + verified OpenCode recovery 仅额外接受 OpenCode 内置 `read` / `grep` / `glob`，以及 arguments 明确表明 `Model tried to call unavailable tool` 的 `invalid`。
- 真实 `bash`、`task`、未知名字、可执行 skill、持久用户/系统写入继续阻断 signature 自动重试。
- OpenCode 隔离配置显式拒绝 `task`，与财报技能的“不得委派”约束一致，避免子代理调用把纯渲染预检失败升级成未知副作用轨迹。
- 真实 RKLB canary 已证明 renderer 两次均返回 `success=false`、`render_success=false`、`side_effect_status=not_started`、零 artifact；脱敏 ACP 事件日志又证明既有解析器已经正确解开 OpenCode 1.18.13 的 `rawOutput.output` 字符串 JSON。缺口是 safe PDF validation 仍使用通用只读白名单，因同轮 OpenCode 内置失败 `glob` 被排除而无法触发既有 fresh-session recovery，随后通用失败归一化覆盖了精确 renderer 错误。
- 不改变普通对话、其它 runner、模型路由、财报证据/PDF 完成契约或最大 token 配置。
- 2026-08-10 NBIS 生产复现证明：首次会话把 renderer 问题从 45 项收敛到 7 项后发生 compact 并结束；既有 fresh-session retry 丢失草稿，第二会话重新停在 12 项，且实际调用 `task` 后被安全边界归一化为“状态无法确定”。成功 RKLB 对照样本则在同一原生会话中连续约 16 次修正 renderer 后产出 PDF，说明根因是续跑丢失工作成果，而非余额。
- 同日 CRWV 生产复现提供第二条独立证据：OpenCode 在调用 `task` 后约一分钟收到精确 `400 Corrupted thought signature`；既有 signature recovery 因 `task` 不是可证明只读调用而安全拒绝重放，最终前端只显示通用失败。因此显式禁用 `task` 同时修复 renderer retry 污染和 Gemini signature 链损坏入口。

## Validation

- `hone-channels` 财报集成回归复现生产工具序列，并证明 signature failure 只 fresh retry 一次。
- 工具轨迹单元测试覆盖生产形状：OpenCode `glob`、HONE 只读取证、一个或多个 `side_effect_status=not_started` renderer 失败能够触发既有 safe PDF validation recovery；真实 shell、未知工具、已落盘 renderer 继续拒绝。
- runner 单元测试覆盖：隔离配置拒绝 `task`；未 compact 的安全预检失败可在同一 ACP session 继续；compact 后不提示耗尽的原生 session。
- AgentSession 回归证明 fresh-session recovery 的第二次 runtime input 含上一轮完整 `report_markdown` 与精确 `render_error`，但持久化用户输入和旧聊天历史均不受污染；缺少完整草稿时保持原有回退行为。
- 执行相关 Rust 格式、定向测试、crate check；部署精确 revision 后核对服务健康、日志中的 `agent.run.retry` 和真实 RKLB PDF 成功闭环。

## Documentation Sync

- 实施期间更新 `docs/current-plan.md` 与本计划。
- 完成后把本计划归档到 `docs/archive/plans/`，更新同日 handoff 与 `docs/archive/index.md`。
- 本修复恢复既有财报 isolated replay 契约，不改变长期架构决策；如实现扩大既有语义，再更新 `docs/decisions.md`。

## Risks / Open Questions

- `invalid` 只有在明确表示目标工具未开放、因此未执行时才可重放；不能把任意 malformed/unknown tool 当成只读。
- OpenCode 内置文件读取名字只在 dedicated earnings + OpenCode 边界使用，不能升级为所有 runner 的全局安全断言。
- OpenCode safe PDF validation 例外只参与 dedicated earnings + OpenCode 的一次 fresh-session recovery；不得把 `read` / `grep` / `glob` 提升为其它 runner 的通用安全工具。
- 再次生产验证会产生 Gemini/OpenRouter 费用；部署与余额门均通过后，只由用户从已登录页面触发一次。
