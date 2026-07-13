# Public Agent Workspace Redesign

- title: Public Agent 工作台响应式重构
- status: done
- created_at: 2026-07-13
- updated_at: 2026-07-13
- owner: Codex
- related_files: `packages/app/src/components/public-agent-workspace.tsx`, `packages/app/src/pages/public-agent-workspace.css`, `packages/app/src/pages/chat.tsx`, `packages/app/src/lib/public-agent-workspace.ts`
- related_docs: `docs/archive/plans/public-agent-workspace-redesign.md`, `docs/repo-map.md`
- related_prs: main commit `63e91795`

## Summary

`/chat` 已从传统聊天壳层升级为响应式 HONE Agent 投资研究工作台。桌面端采用左侧工作台、中间 Agent 研究区、右侧事件与最近研究的三栏结构；移动端采用顶部品牌区、研究卡片、固定输入区与“投资 / 洞察 / Agent / 跟踪 / 我的”五栏底部导航。

## Behavior

- 默认进入 Agent 工作台首页，历史会话仍由原 bootstrap/history 状态恢复，但不再强制占据首屏。
- 点击历史研究或发送消息后无刷新切入原会话，继续复用唯一消息 store、SSE 流式回复、附件、分享和向上分页。
- “洞察”读取 `/api/public/community`，“重要事件”读取 `/api/public/finance-calendar`，“跟踪”打开现有持仓主动跟踪能力。
- 财经日历和持仓弹窗增加受控打开请求，允许桌面右栏、移动内容区和底部导航复用同一实现。
- 工作台搜索、研究历史、社区未读点、推送未读点和账户入口均保留真实交互。

## Verification

- `bun run typecheck:web`: passed.
- `bun run test:web`: 246 passed, 0 failed.
- `bun run build:web:public`: passed; only the repository's existing large-chunk warning remains.
- Browser QA: 1440 x 900 and 390 x 844 rendered without horizontal overflow; verified workspace landing, mobile safe-area navigation, tracking modal, finance-calendar modal, and no-refresh history selection.

## Risks

- The layout intentionally follows the supplied light visual direction. Existing conversation content and legacy modal internals retain their established component styling.
- Cloudflare Pages deployment begins after the two local commits are pushed to `main`; production should be checked for the new `chat-*.js` asset after Pages finishes.

## Next Entry Point

Continue visual or information-architecture work in `packages/app/src/components/public-agent-workspace.tsx` and `packages/app/src/pages/public-agent-workspace.css`; keep message runtime changes in `packages/app/src/pages/chat.tsx` and its existing helper modules.

