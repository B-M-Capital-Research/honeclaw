# Plan

- title: Web 邀请码用户端与管理端入口拆分
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/
  - crates/hone-web-api/src/state.rs
  - packages/app/src/pages/
  - packages/app/src/components/
  - packages/app/src/lib/
- related_docs:
  - docs/archive/index.md
  - docs/handoffs/2026-04-19-web-invite-chat-user-surface.md
  - docs/repo-map.md

## Goal

- 保留现有 `/start` 管理端能力。
- 新增基于邀请码登录的 `/chat` 用户端，提供单会话 IM 风格聊天体验。
- 让邀请码生成、用户创建、登录态与 Web 用户会话隔离落到同一套后端与 SQLite 存储里。

## Scope

- 新增邀请码/登录 session 的 SQLite 存储与公开接口。
- 新增 `/api/public/*` 用户端鉴权、history、chat、events。
- 新增用户端页面、管理端跳转入口、设置页邀请码管理。
- 复用现有 quota 与 SSE 聊天链路，不引入 WebSocket。

## Validation

- Rust 单测覆盖邀请码生成、登录、登出、公开接口鉴权隔离。
- 前端 `test:web`、`typecheck`、`build` 均通过。

## Documentation Sync

- 从 `docs/current-plan.md` 移出活跃索引。
- 归档到 `docs/archive/plans/`。
- 更新 `docs/repo-map.md`、`docs/archive/index.md` 与 handoff。

## Risks / Open Questions

- 当前用户端复用 SSE，不含 WebSocket。
- 公开接口严格从 cookie 登录态反解 actor；后续若扩展多会话或停用邀请码，需要继续扩展 `web_auth` 数据模型。
