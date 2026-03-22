# 群聊共享 Session 落地

最后更新：2026-03-19
状态：已完成

## 目标

- 让群聊上下文从“每个用户一份”改成“每个群一份 session”。
- 保持 `ActorIdentity` 继续负责权限、quota、sandbox 和私有数据隔离，不被共享 session 混淆。
- 为群共享 session 增加独立的恢复窗口和压缩策略，避免上下文膨胀。

## 涉及文件

- `crates/hone-core/src/actor.rs`
- `memory/src/session.rs`
- `crates/hone-channels/src/agent_session.rs`
- `crates/hone-channels/src/core.rs`
- `crates/hone-channels/src/ingress.rs`
- `bins/hone-telegram/src/main.rs`
- `bins/hone-feishu/src/main.rs`
- `bins/hone-discord/src/handlers.rs`
- `bins/hone-discord/src/group_reply.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `packages/app/src/context/sessions.tsx`
- `packages/app/src/components/session-list.tsx`
- `packages/app/src/components/chat-view.tsx`

## 完成情况

- 已新增 `SessionIdentity / SessionKind`，专门表示“写入哪份 session”。
- 已明确 `ChatMode` 只表示消息形态，不表示 session 归属。
- 已把群 session 压缩规则切为独立配置：最近 18 条恢复、24 条/48KB 触发压缩、保留最近 8 条。
- 已把 Telegram / Feishu / Discord 群消息切到共享群 session，并在写入内容中保留发言人标识。
- 已把 Web 控制台的列表与历史读取改成按真实 `session_id` 浏览；群 session 当前设为只读浏览，避免 Web 误用个人 actor 代发。

## 验证

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`
- `cargo test -p hone-memory -p hone-channels`
- `cargo test -p hone-channels -p hone-memory -p hone-web-api --no-run`
- `bun run typecheck`（`packages/app`）

## 后续可选项

- 把 Telegram / Feishu 的群消息也做成和 Discord 一样的短窗口批处理，而不是当前的共享 session + 串行处理。
- 若需要从 Web 控制台直接进入群会话发言，需要额外设计“当前发言 actor”选择器，不能复用列表里的 session 身份。
