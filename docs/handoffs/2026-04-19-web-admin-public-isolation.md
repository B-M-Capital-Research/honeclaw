- title: Web 管理端 / 用户端端口隔离与公网暴露加固
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19 16:25 CST
- owner: Codex
- related_files:
  - crates/hone-web-api/src/lib.rs
  - crates/hone-web-api/src/routes/mod.rs
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-web-api/src/routes/web_users.rs
  - crates/hone-web-api/src/public_auth.rs
  - memory/src/web_auth.rs
  - packages/app/src/lib/api.ts
  - packages/app/src/pages/settings.tsx
  - packages/app/src/app.tsx
  - packages/app/vite.config.ts
  - package.json
  - launch.sh
  - docs/repo-map.md
- related_docs:
  - docs/archive/plans/web-admin-public-isolation.md
  - docs/archive/plans/public-web-security-hardening.md
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
- public 邀请码登录新增应用层失败冷却：同一来源连续失败达到阈值后会收到 `429` 和 `Retry-After`，不再无限制暴力尝试
- `web_auth` 现在支持邀请码 `revoked_at`、停用 / 恢复、重置邀请码，并在停用或重置时立即清理现有 public 登录态；同一邀请码重复登录时改为“最新登录覆盖旧 session”
- public 端登录 Cookie 改为 `HttpOnly + SameSite=Strict`，并在 HTTPS `Origin` / `Referer` / `X-Forwarded-Proto` 场景自动附带 `Secure`
- public API 默认不再挂 `CORS: *`；管理端设置页已补邀请码状态、活跃登录态、停用 / 启用 / 重置按钮

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `bun run typecheck:web`
- `bun run test:web`
- `./launch.sh --web`
- `curl http://127.0.0.1:8077/api/public/auth/me` -> `404`
- `curl http://127.0.0.1:8088/api/meta` -> `404`
- `cargo test -p hone-memory web_auth`
- `cargo test -p hone-web-api`
- `cargo check -p hone-web-api -p hone-memory`

## Risks / Follow-ups

- 当前管理端鉴权逻辑在 `deployment_mode=local` 且 `web.auth_token` 为空时仍会放行管理 API；这只适用于“管理端只监听本机”的前提。任何把管理端暴露到公网、局域网或跳板机后的场景，都必须先配 `web.auth_token`
- public 用户端已经补了应用层失败冷却，但当前限流状态仍只保存在进程内存里；若要长期公网暴露，仍建议在反向代理 / WAF 层加 IP 级限流和异常流量拦截
- public `Secure` cookie 依赖反代层或浏览器请求头能正确体现 HTTPS；上线时需要确认 `Origin` / `Referer` / `X-Forwarded-Proto` 至少有一条链路可靠透传

## Next Entry Point

- `crates/hone-web-api/src/routes/public.rs`
