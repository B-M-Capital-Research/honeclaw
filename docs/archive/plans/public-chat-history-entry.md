# Public Chat 静默恢复与历史入口

- status: `done`
- created_at: `2026-07-13`
- updated_at: `2026-07-13`
- owner: Codex
- related_files: `packages/app/src/pages/chat.tsx`, `packages/app/src/components/public-agent-workspace.tsx`, `packages/app/src/pages/public-agent-workspace.css`

## 目标

- 进入已登录对话时立即展示工作台壳层，后台静默恢复最近消息，不再展示独立恢复 loading 页。
- 有历史消息时默认打开最近对话，并在移动端提供可定位、可继续加载的会话历史入口。
- 恢复失败使用壳层内提示，不打断页面结构。

## 结果

- 首次 bootstrap 不再替换工作台；有消息时原位恢复最近 20 条并保持在底部，无消息时进入新研究首页。
- 桌面端复用左栏最近研究，移动端顶部新增会话历史抽屉，支持定位当前窗口和分页加载更早记录。
- 非鉴权恢复失败显示壳层内重试提示；401 仍切换登录页。

## 验证

- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web:public`
- Browser QA: 390 x 844 与 1365 x 850；验证静默恢复、最近消息、历史抽屉、定位与更早记录加载。

## 风险

- 未登录用户在认证请求返回前会短暂看到无数据工作台壳层，收到 401 后立即切换登录页。
- 当前后端是单一连续会话，历史入口按用户提问定位消息，并非创建多条独立 session。
