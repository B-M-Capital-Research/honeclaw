- title: Web 管理端 / 用户端端口隔离与公网暴露加固
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: Codex
- related_files:
  - crates/hone-web-api/src/lib.rs
  - crates/hone-web-api/src/routes/mod.rs
  - packages/app/src/app.tsx
  - packages/app/vite.config.ts
  - package.json
  - launch.sh
  - docs/repo-map.md
- related_docs:
  - docs/archive/plans/web-admin-public-isolation.md
  - docs/archive/index.md
  - docs/repo-map.md
- related_prs: N/A

## Summary

- 管理端和用户端现在已按监听端口、可访问 API 和前端构建面拆开，用户端准备走公网反代时不会再顺手把管理端 `/api/*` 和 console 路由一起暴露出去。

## What Changed

- `hone-web-api::start_server` 改为共享同一个 `AppState` 启动两个 listener：管理端走 `HONE_WEB_PORT`，用户端走 `HONE_PUBLIC_WEB_PORT`
- 管理端 app 只暴露 `/api/*`；用户端 app 只暴露 `/api/public/*`。互相探测对方 API 现在都是 `404`
- `packages/app` 增加 `VITE_HONE_APP_SURFACE`，同一套代码可分别构建管理端和用户端；root scripts 新增 `dev:web:public` 与 `build:web:public`
- `launch.sh --web` 现在会同时拉起 `3000` 管理端前端和 `3001` 用户端前端，并做 ready-check
- `docs/repo-map.md` 已补充新的入口和部署结构

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `bun run typecheck:web`
- `bun run test:web`
- `./launch.sh --web`
- `curl http://127.0.0.1:8077/api/public/auth/me` -> `404`
- `curl http://127.0.0.1:8088/api/meta` -> `404`

## Risks / Follow-ups

- 当前管理端鉴权逻辑在 `deployment_mode=local` 且 `web.auth_token` 为空时仍会放行管理 API；这只适用于“管理端只监听本机”的前提。任何把管理端暴露到公网、局域网或跳板机后的场景，都必须先配 `web.auth_token`
- 用户端目前仍缺少显式 rate limiting / 邀请码暴力尝试防护。如果真的直接对公网开放，下一步建议在反向代理层加 IP 级限流，并在应用层补失败次数与冷却时间
- public cookie 目前主要依赖 `HttpOnly` / `SameSite` 语义；正式 HTTPS 暴露前，建议复核 `Secure` 标记和反向代理 TLS 终止后的 cookie 行为

## Next Entry Point

- `crates/hone-web-api/src/routes/auth.rs`
