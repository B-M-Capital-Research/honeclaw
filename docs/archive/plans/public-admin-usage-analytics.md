# Public 管理员使用统计

- title: Public 管理员使用统计
- status: archived
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/public_admin.rs
  - crates/hone-web-api/src/routes/mod.rs
  - crates/hone-web-api/src/types.rs
  - packages/app/src/components/public-admin-usage-panel.tsx
  - packages/app/src/pages/public-me.tsx
  - packages/app/src/pages/public-workspace.css
  - packages/app/src/lib/api.ts
  - packages/app/src/lib/types.ts
- related_docs:
  - docs/repo-map.md
  - docs/invariants.md
  - docs/runbooks/public-user-admin.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md

## Goal

在 Public `/me` 的“管理”栏目增加仅管理员可见的实时使用统计：按北京时间展示每日、每位 Web 用户的提问次数、问题明细、定时任务执行/投递数量，并在表格上方生成今日规模、提问总量、上周同比和主要降频用户摘要。

## Scope

- 服务端继续以当前 Web cookie session + PostgreSQL/SQLite 管理员角色复核作为唯一授权边界。
- 统计最近 14 个北京时间自然日；用户提问只统计 Web 直接会话中的真实 `user` 消息，排除 `source=scheduler|heartbeat`、job metadata 与旧版定时任务触发 envelope。
- 定时任务数量同时展示执行次数和成功投递次数；顶部“推送”采用成功投递数。
- “较上周”按今日与 7 天前同日比较；“本周降频”按本周截至当前时点与上周相同时间窗比较。
- 前端提供刷新、日期筛选和可展开的问题明细，移动端保持可读。

## Validation

- Rust 单元测试覆盖自动消息排除、北京时间日期归并、摘要同比与降频用户选择：完成。
- Web 单元测试覆盖摘要 API、筛选和管理员挂载：完成。
- Rust/Web 编译、Web API 163/163、Web 337/337、Public 构建与桌面/移动浏览器验收：完成，详见 handoff。

## Documentation Sync

- `docs/repo-map.md`、`docs/invariants.md`、`docs/runbooks/public-user-admin.md`、`docs/decisions.md`：已同步。
- handoff 与 archive index：已同步。

## Risks / Open Questions

- 大规模 session/cron 历史需要后续下推 PostgreSQL 过滤/聚合；当前 14 天窗口和 50,000 cron 上限记录在 handoff。
- 本次没有生产部署或真实用户数据验收。
