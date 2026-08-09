# Public 推送消息中心与未读状态闭环

- title: Public 推送消息中心与未读状态闭环
- status: in_progress
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - packages/app/src/pages/public-pushes.tsx
  - packages/app/src/components/public-push-inbox.tsx
  - packages/app/src/components/public-workspace-shell.tsx
  - packages/app/src/components/public-agent-workspace.tsx
  - packages/app/src/pages/chat.tsx
  - packages/app/src/lib/public-content.ts
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/public-push-inbox-state-closure.md

## Goal

恢复用户可见的推送消息内容，把 `/pushes` 收口为“推送消息 / 订阅管理”双视图；消息按订阅任务分类，顶部、侧栏和移动端底部入口显示一致的未读红点，并只在服务端确认已读后清除。

## Scope

- 复用现有 public push list/open API，不改变后端消息与已读语义。
- `/pushes` 默认展示消息列表和完整详情，并提供订阅分类筛选。
- 订阅管理保留为同页独立视图。
- 所有推送入口统一进入消息视图；移动端底部推送入口补未读红点。
- 删除入口处的乐观清零，避免路由切换或请求竞态让红点错误残留/回弹。

## Validation

- 推送分类、筛选和分页模型单元测试。
- Web API/样式契约相关测试。
- `bun run test:web` 与 public app production build。
- 部署后使用已登录浏览器验证桌面/移动端入口、双视图和红点状态。

## Documentation Sync

- 实施期间更新 `docs/current-plan.md`。
- 完成后将本计划移至 `docs/archive/plans/`，补充 handoff 和 `docs/archive/index.md`，并从活跃任务索引移除。
- 本次复用既有 API 和架构边界，不新增 ADR；若实现中出现跨模块长期决策再补 `docs/decisions.md`。

## Risks / Open Questions

- 某些用户可能暂无历史推送，生产验收需同时依赖自动化 fixture 验证分类和详情。
- 已删除或停用的订阅仍需以历史消息自带的 `job_id` / `title` 正常归类。
- 已读请求失败时红点必须保留，并允许下一次进入或聚焦页面重试。
