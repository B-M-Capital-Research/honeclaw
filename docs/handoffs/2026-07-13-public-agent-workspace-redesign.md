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

- 已有历史时默认进入最近对话并保持在最后一条；新账号或空历史进入 Agent 工作台首页。
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

## 2026-07-13 静默恢复与移动历史入口

- `/chat` 首次 bootstrap 改为在完整工作台壳层内静默执行，不再出现独立“正在恢复对话”页面；失败时使用不改变布局的内联重试提示。
- 已有历史时直接显示最近 20 条并定位到底部；移动顶部新增会话历史入口，抽屉按用户提问定位当前窗口，并复用 `before` cursor 加载更早记录。
- Browser QA 覆盖 390 x 844 和 1365 x 850；验证无 loading 卡片、最近消息恢复、抽屉开关、历史定位和分页合并。
