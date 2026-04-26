- title: 后端部署文档与 public chat 顶部菜单修复
- status: done
- created_at: 2026-04-26
- updated_at: 2026-04-26
- owner: Codex
- related_files:
  - `docs/runbooks/backend-deployment.md`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/public/_redirects`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-home.tsx`
  - `packages/app/src/lib/public-content.ts`
- related_docs:
  - `docs/archive/plans/backend-deployment-and-public-chat-nav.md`
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

新增“后端部署”runbook，记录 Cloudflare Pages、Worker API proxy 与后端 origin 的更新/验证流程，公开文档统一使用“后端 origin / managed backend host”口径，避免暴露具体私有运行位置。同时修复 public chat 页面顶部菜单样式缺失问题。

## What Changed

- `docs/runbooks/backend-deployment.md`：新增后端部署与回滚手册，覆盖前端 Pages build、后端 origin 更新、Worker route、Cookie/SSE 验证和安全注意事项。
- `packages/app/src/pages/public-site.css`：补齐 chat/header 共享样式，包括固定顶部栏、社交按钮、star badge、语言切换、路由按钮和移动端收敛规则。
- `packages/app/public/_redirects`：为 Cloudflare Pages 增加 SPA fallback。
- `packages/app/src/pages/chat.tsx`：修正 `backdrop-filter` 类型拼写并移除未使用的 EventSource 清理路径。
- `packages/app/src/lib/public-content.ts` / `packages/app/src/pages/public-home.tsx`：收口 EN/ZH 内容里可空图片字段的类型。

## Verification

- 线上复现：`https://hone-claw.com/chat` 顶部 header 缺少样式，logo/社交/语言/导航纵向散落。
- `PATH="$HOME/.bun/bin:$PATH" bun run typecheck:web` passed。
- `PATH="$HOME/.bun/bin:$PATH" bun run build:web:public` passed。
- 本地预览 `packages/app/dist-public` 后用 Chrome/Playwright 检查：
  - `1440x900`：header 固定在顶部，logo、社交、语言、路线图、对话按钮横向对齐，无横向溢出。
  - `390x844`：header 高度 64px，隐藏社交和路线图，仅保留 logo、语言切换和对话按钮，无横向溢出。

## Risks / Follow-ups

- Cloudflare Worker 仍在控制台维护；若未来希望完全 IaC 化，应引入 `wrangler` 配置并把 Worker 脚本纳入仓库，但不能提交 token。
- `build:web:public` 仍会产生大 chunk warning；本次未处理代码分割。
- 线上 Pages 需要等本次改动 push 后重新部署才会看到修复。

## Next Entry Point

- 部署/回滚流程：`docs/runbooks/backend-deployment.md`
- public chat header 样式：`packages/app/src/pages/public-site.css`
