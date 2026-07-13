- title: Public Agent 工作台响应式重构
- status: done
- created_at: 2026-07-13
- updated_at: 2026-07-13
- owner: Codex
- related_files:
  - packages/app/src/pages/chat.tsx
  - packages/app/src/components/public-agent-workspace.tsx
  - packages/app/src/pages/public-agent-workspace.css
  - packages/app/src/lib/public-agent-workspace.ts
  - packages/app/src/lib/public-agent-workspace.test.ts
- related_docs:
  - docs/repo-map.md
  - docs/handoffs/public-agent-workspace-redesign.md

## Goal

按照已确认的桌面端与移动端交互稿，把 `/chat` 从传统聊天页重构为 HONE Agent 投资研究工作台，同时保留现有会话、流式回复、附件、社区洞察、持仓跟踪、财经日历、推送和账户能力。

## Scope

- 桌面端采用左侧工作台、中间 Agent 主区、右侧事件与研究栏的三栏结构。
- 移动端采用顶部品牌区、研究内容卡片、固定输入区和五栏底部导航。
- 将社区动态映射为“洞察”、用户持仓映射为“跟踪”、财经日历映射为“重要事件”。
- 进入页面默认展示 Agent 工作台；选择历史记录或发送消息后无刷新进入会话。
- 工作台数据来自现有社区、财经日历与会话接口，不新增静态演示数据后端。

## Validation

- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web:public`
- 使用桌面与移动视口检查首页、会话切换、洞察、跟踪、日历、通知和账户入口。

## Documentation Sync

- 完成后更新 `docs/repo-map.md` 的公共用户端边界说明。
- 新增 handoff，计划归档到 `docs/archive/plans/`，并更新 `docs/archive/index.md`。

## Risks / Follow-up Notes

- 工作区首页与历史会话必须共享同一恢复和流式状态，不能引入第二套消息真相源。
- 日历与跟踪弹窗原为组件内部状态，需要最小化改造为可由工作台入口受控打开。
- 移动端固定底栏必须兼容 Safari 安全区、软键盘和现有图片预览手势。
