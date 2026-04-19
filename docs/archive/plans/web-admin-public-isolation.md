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
  - docs/archive/index.md
  - docs/handoffs/2026-04-19-web-admin-public-isolation.md
  - docs/repo-map.md

## Goal

- 将 Web 管理端与用户端按监听端口、可访问路由和前端构建产物彻底拆开，减少公网暴露面，并补一轮用户端安全检查结论。

## Scope

- 后端拆分 admin/public Axum app 与独立 listener
- 前端拆分 admin/public surface 与构建输出
- 本地启动脚本支持双前端端口
- 更新 repo map 与活跃任务索引

## Validation

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `bun run typecheck:web`
- `bun run test:web`
- 本地启动后检查 `127.0.0.1:3000` 仅管理端、`127.0.0.1:3001` 仅用户端、`127.0.0.1:8077` 不再暴露 `/api/public/*`、`127.0.0.1:8088` 不再暴露 `/api/*`

## Documentation Sync

- 更新 `docs/current-plan.md` 增加活跃任务索引
- 更新 `docs/repo-map.md` 记录管理端 / 用户端分离后的入口与部署结构

## Risks / Open Questions

- 管理端若未来被反向代理到公网，`deployment_mode=local` 且未配置 `web.auth_token` 时仍存在高风险
- 用户端目前仍缺少显式 rate limiting / brute-force 防护
