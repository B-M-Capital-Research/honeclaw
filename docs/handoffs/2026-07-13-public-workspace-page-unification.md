# Public Workspace Page Unification

- title: Public Workspace 页面体验统一
- status: done
- created_at: 2026-07-13
- updated_at: 2026-07-13
- owner: Codex
- related_files: `packages/app/src/components/public-workspace-shell.tsx`, `packages/app/src/pages/public-workspace.css`, `packages/app/src/pages/public-community.tsx`, `packages/app/src/pages/public-portfolio.tsx`, `packages/app/src/pages/public-me.tsx`, `packages/app/src/components/public-chat-startup.tsx`
- related_docs: `docs/archive/plans/public-workspace-page-unification.md`, `docs/repo-map.md`
- related_prs: main commit `affa8836`

## Summary

恢复、洞察、跟踪/财经日历和个人页已统一到 HONE Agent 工作台视觉与交互体系。桌面共用侧栏与顶部搜索，移动共用品牌栏、通知/账户入口和五栏安全区导航；跨页切换不再回到旧版官网导航或重型账户页面。

## Behavior

- 洞察页改为参考稿风格的连续研究流，移除大 Hero 和独立悬浮社交卡，同时保留社区图片预览、受保护资源与加载更多。
- 跟踪页以真实 `/api/public/finance-calendar` 数据作为主视图：桌面渲染 7 列月历，移动端渲染独立日期议程，不压缩桌面日历。
- 原投资主线、持仓与公司画像继续保留在跟踪日历下方。
- 个人页重写为轻量账户面板，统一信息行、操作按钮和隐私说明。
- 对话恢复页从 `HONE CONVERSATION` 营销风格改为 Agent 工作台恢复骨架，并在手机端保留同款五栏占位。

## Verification

- `bun run typecheck:web`: passed.
- `bun run test:web`: 249 passed, 0 failed.
- `bun run build:web:public`: passed; only existing large-chunk warnings remain.
- Browser QA: desktop 1440 x 900 and mobile 390 x 844 checked for Insights, Tracking calendar/agenda, Account, and restore skeleton; mobile document width remained within the viewport and active tabs matched each page.

## Risks

- Tracking tabs are interactive: Today and History filter calendar events by date, Calendar owns the month/agenda views, and Tasks provides the Agent entry for creating a persistent tracking task. A future dedicated task-list API can replace that entry surface without changing the shared shell.
- Community media preview remains in its existing portal and intentionally keeps its specialized full-screen controls above the shared workspace chrome.

## Next Entry Point

Use `public-workspace-shell.tsx` for shared navigation changes, `public-workspace.css` for cross-page layout, and keep domain-specific behavior in the owning Community, Portfolio, or Account page.
