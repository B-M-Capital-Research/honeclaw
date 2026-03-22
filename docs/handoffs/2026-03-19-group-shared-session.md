# Handoff: Group Shared Session

日期：2026-03-19
状态：已完成

## 本次目标

- 落地“每个群一个 session”的真群聊模式。
- 避免 `ChatMode`、`ActorIdentity`、session 归属三个概念继续混淆。
- 给群共享 session 加上更紧的上下文窗口与压缩策略。

## 已完成

- 新增 `SessionIdentity / SessionKind`，把“执行身份”和“会话归属”拆开：
  - `ActorIdentity` 仍用于权限、quota、sandbox、私有数据隔离
  - `SessionIdentity` 用于上下文恢复与消息持久化
- `IncomingEnvelope`、`AgentSession`、session JSON 已接入显式 `session_identity`
- 群 session JSON 升级为 v3，新增 `session_identity`
- 群共享 session 压缩策略独立：
  - 最近 18 条恢复
  - 超过 24 条或 48KB 触发压缩
  - 压缩后保留最近 8 条
  - 群摘要模板改为“进行中议题 / 已形成结论 / 未决问题 / 群约定待办”
- Telegram / Feishu / Discord 群消息现统一写入共享群 session
- Telegram / Feishu / Discord 群消息写入内容会带发言人标签，如 `[Alice] ...`
- Web 控制台的 session 列表与历史接口改为按真实 `session_id` 浏览，群共享 session 在 UI 中明确标记为只读浏览

## 验证

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`
- `cargo test -p hone-memory -p hone-channels`
- `cargo test -p hone-channels -p hone-memory -p hone-web-api --no-run`
- `bun run typecheck`（`packages/app`）

## 影响范围

- 核心 identity / session：
  - `crates/hone-core/src/actor.rs`
  - `memory/src/session.rs`
  - `crates/hone-channels/src/agent_session.rs`
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/ingress.rs`
- 渠道入口：
  - `bins/hone-telegram/src/main.rs`
  - `bins/hone-feishu/src/main.rs`
  - `bins/hone-discord/src/handlers.rs`
  - `bins/hone-discord/src/group_reply.rs`
  - `bins/hone-imessage/src/main.rs`
- Web 控制台：
  - `crates/hone-web-api/src/routes/users.rs`
  - `crates/hone-web-api/src/routes/history.rs`
  - `packages/app/src/context/sessions.tsx`
  - `packages/app/src/components/session-list.tsx`
  - `packages/app/src/components/chat-view.tsx`

## 剩余风险 / 后续建议

- Telegram / Feishu 目前仍是“共享群 session + 串行处理”，还没有像 Discord 一样做真正的群短窗批处理。
- `group_context.shared_session_enabled` 配置面已加到样例与配置结构，但当前实现主路径默认就是共享群 session；若后续要支持热切回旧模式，需要继续补“非共享群 session”的完整兼容语义。
- Web 控制台目前只把群共享 session 作为只读历史浏览；若要支持从 Web 直接往群里发言，必须设计“当前发言 actor”选择器，不能直接复用 session 身份。
