# Public 推送消息中心与未读状态闭环

- title: Public 推送消息中心与未读状态闭环
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - packages/app/src/pages/public-pushes.tsx
  - packages/app/src/components/public-push-inbox.tsx
  - packages/app/src/lib/public-push-inbox.ts
  - packages/app/src/lib/public-push-unread.ts
  - packages/app/src/components/public-agent-workspace.tsx
- related_docs:
  - docs/archive/plans/public-push-inbox-state-closure.md
  - docs/archive/index.md
- related_prs:
  - direct main commit `e451dd3b9a20f98777f888bf6b0e040c7fcdc386`

## Summary

`/pushes` 不再只是订阅管理页。它默认展示真实推送消息、摘要和完整详情，并保留“订阅管理”作为同页第二视图。消息按稳定 `job_id` 分类；顶部、侧栏和移动端底部入口共享未读状态。

## What Changed

- 新增消息收件箱和“全部 / 各订阅任务”分类，点击消息复用既有 open API 展示完整 Markdown 内容。
- 顶部通知、侧栏推送和底部推送统一进入 `/pushes` 的消息视图；管理入口使用 `?view=manage`，再次点击通知会回到消息。
- 移动端底部推送按钮加入红点，四项菜单结构保持不变。
- 未读数改成路由间共享，但不把前端状态当真相源：进入消息视图先保留原红点，list API 返回后更新，只有 read-through POST 成功返回的 `unread_count` 才能关闭红点；失败时保留并在聚焦时重试。
- 删除 chat/workspace 原有不可达的推送 modal 状态，避免旧页面与新页面并发请求导致红点回弹。

## Verification

- TypeScript typecheck、407 个 Web 测试、导航响应回归、public production build 全部通过。
- Cloudflare Pages 已发布新 index 和消息 chunk，静态资源均为 `200`。
- 生产已登录移动端浏览器实测消息默认页、管理切换、顶部通知回消息页和四项底部菜单。
- 生产验收账号没有历史推送；分类、顺序、历史任务保留和状态闭环由新增模型测试及既有 API 测试覆盖。

## Risks / Follow-ups

- 当前没有后端或数据库改动，不需要 GCE 重启；消息内容和已读 read-through 继续由现有 public push API 负责。
- 若未来要显示“尚未产生消息”的订阅分类，需要把订阅列表与历史消息分类显式合并；当前分类只展示实际有消息的任务。

## Next Entry Point

从 `PublicPushInbox` 的 `loadPushes` / `acknowledgeVisiblePushes` 开始排查消息与红点；分类纯逻辑在 `public-push-inbox.ts`，跨路由未读信号在 `public-push-unread.ts`。
