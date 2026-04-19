# Handoff

- title: Web 邀请码用户端与管理端入口拆分
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: Codex
- related_files:
  - memory/src/web_auth.rs
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-web-api/src/routes/web_users.rs
  - packages/app/src/pages/chat.tsx
  - packages/app/src/pages/settings.tsx
- related_docs:
  - docs/archive/plans/web-invite-chat-user-surface.md
  - docs/repo-map.md
  - docs/archive/index.md
- related_prs:
  - N/A

## Summary

- 新增基于邀请码登录的 `/chat` 用户端，提供单会话、无左侧历史栏的 IM 式聊天窗口。
- 管理端在侧边栏“开始”旁增加用户端跳转 icon，并在设置页新增邀请码管理区块。
- 后端新增 `/api/public/*` 公开用户接口与 SQLite `web_auth` 存储，邀请码生成即创建 `web` 用户，继续复用现有 quota 逻辑。

## What Changed

- `memory/src/web_auth.rs`：新增邀请码用户与 Web 登录 session 存储，落在现有 `sessions.sqlite3`。
- `crates/hone-web-api/src/routes/public.rs`：新增邀请码登录、登出、当前用户、公开 history、公开 chat、公开 events。
- `crates/hone-web-api/src/routes/web_users.rs`：新增管理端邀请码列表与生成功能。
- `crates/hone-web-api/src/routes/chat.rs`：抽出共享 SSE chat 执行函数，管理端与用户端共用。
- `packages/app/src/pages/chat.tsx`：新增用户端页面，包含邀请码登录、单卡片 thinking/tool/final 状态展示、登出与 SSE 聊天。
- `packages/app/src/pages/settings.tsx`：新增邀请码管理区块，支持生成、复制、查看剩余次数与最近登录时间。

## Verification

- `cargo test -p hone-memory web_auth -- --nocapture`
- `cargo test -p hone-web-api -- --nocapture`
- `bun run test:web`
- `cd packages/app && bun run typecheck && bun run build`

## Risks / Follow-ups

- 当前邀请码管理只支持生成、列表、复制，不支持停用、删除、重置。
- 用户端会在每轮发送结束后重新拉取一次 `/api/public/auth/me` 与 `/api/public/history`；若后续要做更强实时性或更细粒度卡片更新，可再收口成更轻的增量同步。
- cookie 为 `HttpOnly + SameSite=Lax`，默认 30 天 rolling expiry；若后续需要设备管理或强制下线，需要继续扩展 `web_auth_sessions`。

## Next Entry Point

- `crates/hone-web-api/src/routes/public.rs`
