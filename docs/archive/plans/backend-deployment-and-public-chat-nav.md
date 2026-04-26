- title: 后端部署文档与 public chat 顶部菜单修复
- status: in_progress
- created_at: 2026-04-26
- updated_at: 2026-04-26
- owner: Codex
- related_files:
  - `docs/runbooks/backend-deployment.md`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/public/_redirects`
- related_docs:
  - `docs/current-plan.md`

## Goal

记录当前官网静态部署、动态 API 回源与后端 origin 的更新流程，使用“后端部署”口径，避免在公开文档中描述敏感的具体运行位置；同时修复 `https://hone-claw.com/chat` 顶部菜单无样式崩坏。

## Scope

- 新增后端部署 runbook，覆盖 Cloudflare Pages、Worker API proxy、后端 origin 更新与验证清单。
- 修复 public chat header 缺失共享样式的问题。
- 增加 Cloudflare Pages SPA fallback，保证 public routes 直接刷新可回到前端入口。

## Validation

- 复现线上 `/chat` 顶部菜单样式问题。
- 运行前端类型检查和 public build。
- 如可行，用浏览器截图确认 `/chat` 顶部菜单在桌面与移动宽度下恢复。

## Documentation Sync

- 新增 `docs/runbooks/backend-deployment.md`。
- 完成后从 `docs/current-plan.md` 移除本任务，将计划页归档到 `docs/archive/plans/`，并补 `docs/archive/index.md`。
- 如验证结论有后续价值，新增 handoff。

## Risks / Open Questions

- Worker 配置仍在 Cloudflare 控制台中维护，本仓库只记录操作约定与验证点。
- 后端 origin 域名、Cloudflare token、实际进程托管细节不得写入公开文档。
