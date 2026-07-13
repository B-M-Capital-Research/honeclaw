- title: Public Workspace 页面体验统一
- status: done
- created_at: 2026-07-13
- updated_at: 2026-07-13
- owner: Codex
- related_files:
  - packages/app/src/components/public-workspace-shell.tsx
  - packages/app/src/components/public-chat-startup.tsx
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/public-portfolio.tsx
  - packages/app/src/pages/public-me.tsx
  - packages/app/src/pages/public-workspace.css
- related_docs:
  - docs/repo-map.md
  - docs/handoffs/2026-07-13-public-workspace-page-unification.md

## Goal

以新的 HONE Agent 工作台为统一视觉与交互基线，重构恢复页、洞察页、跟踪/财经日历页与个人页，并完整覆盖桌面和移动端。

## Scope

- 抽取共享桌面侧栏、顶部栏、移动品牌栏和五栏底部导航。
- 洞察页改为连续、轻量的研究信息流，保留媒体预览与受保护文件能力。
- 跟踪页增加真实财经日历主视图；桌面展示月历，移动展示可读事件议程，并保留投资主线与公司画像。
- 恢复页使用同一工作台骨架，不再显示独立的 HONE CONVERSATION 营销页。
- 个人页改为轻量工作台账户页，统一卡片、按钮、字号与留白。

## Validation

- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web:public`
- 桌面与移动视口检查恢复、洞察、跟踪月历、个人页和跨页导航。

## Documentation Sync

- 完成后更新 `docs/repo-map.md`。
- 新增 handoff，归档本计划并更新 `docs/archive/index.md`。

## Risks / Open Questions

- 社区媒体预览和文件下载不能因信息流重构退化。
- 移动端不能压缩桌面七列月历；事件必须使用独立议程布局。
- 页面壳层不得复制会话状态或引入第二套认证真相源。
