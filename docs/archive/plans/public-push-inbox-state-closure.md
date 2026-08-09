# Public 推送消息中心与未读状态闭环

- title: Public 推送消息中心与未读状态闭环
- status: archived
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - packages/app/src/pages/public-pushes.tsx
  - packages/app/src/components/public-push-inbox.tsx
  - packages/app/src/components/public-workspace-shell.tsx
  - packages/app/src/components/public-agent-workspace.tsx
  - packages/app/src/pages/chat.tsx
  - packages/app/src/lib/public-push-inbox.ts
  - packages/app/src/lib/public-push-unread.ts
- related_docs:
  - docs/handoffs/2026-08-09-public-push-inbox-state-closure.md
  - docs/archive/index.md

## Goal

恢复用户可见的推送消息内容，把 `/pushes` 收口为“推送消息 / 订阅管理”双视图；消息按订阅任务分类，顶部、侧栏和移动端底部入口显示一致的未读红点，并只在服务端确认已读后清除。

## Scope

- 复用现有 public push list/open API，不改变后端消息与已读语义。
- `/pushes` 默认展示消息列表和完整详情，并提供订阅分类筛选。
- 订阅管理保留为同页独立视图。
- 所有推送入口统一进入消息视图；移动端底部推送入口补未读红点。
- 删除入口处的乐观清零，避免路由切换或请求竞态让红点错误残留/回弹。

## Validation

- `bun run typecheck`：通过。
- `bun run test:web`：407 tests passed。
- `bash tests/regression/ci/test_navigation_responsiveness_contract.sh`：通过。
- `bun run build:web:public`：通过。
- Cloudflare Pages 生产资源：新 index 与 `public-pushes` chunk 均返回 `200`，chunk 包含消息视图与已读同步逻辑。
- 已登录生产浏览器移动端验收：`/pushes` 默认显示“推送消息”，可切换“订阅管理”，右上角“通知”从管理视图返回消息视图，底部保持四项导航。

## Documentation Sync

- 活跃计划已从 `docs/current-plan.md` 移除。
- 完成结论写入 `docs/handoffs/2026-08-09-public-push-inbox-state-closure.md` 和 `docs/archive/index.md`。
- 本次复用既有 API 和架构边界，无需新增 ADR 或后端运维 runbook。

## Risks / Open Questions

- 生产验收账号暂无历史推送，因此消息分类和详情用自动化 fixture/API contract 验证；有实际推送的账号会从相同 API 直接渲染。
- 已删除或停用订阅的历史消息继续依赖消息自带的 `job_id` / `title` 归类，这是刻意保留历史可发现性的行为。
