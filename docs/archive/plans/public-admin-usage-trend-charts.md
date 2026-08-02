# Public 管理员使用趋势图

- title: Public 管理员使用趋势图
- status: archived
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - packages/app/src/components/public-admin-usage-panel.tsx
  - packages/app/src/components/public-admin-usage-panel.test.ts
  - packages/app/src/pages/public-workspace.css
- related_docs:
  - docs/decisions.md
  - docs/runbooks/public-user-admin.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md

## Goal

在管理员使用统计中增加最近两周的每日使用用户数和每日提问量折线图，同时保持页面紧凑、口径与表格一致。

## Scope

- 从现有管理员 usage report 派生截至 `period_end` 的连续 14 个北京时间日期，缺失日期补 0。
- 用户量统计当天 `question_count > 0` 的去重用户；提问量汇总当天所有真实问题数。
- 使用无第三方依赖的响应式 SVG 折线图，两张图共享同一横轴日期序列并放在现有可折叠统计区内。
- 不改变 API、管理员授权、存储或推送行为。

## Validation

- 趋势/样式定向测试：12/12 通过。
- 完整 Web 测试：341/341 通过。
- TypeScript typecheck 与 Public production build：通过；构建仅保留既有的大 chunk 警告。
- 管理员登录态真实浏览器验收：两张图各 14 个点，共 28 个；横轴连续覆盖 2026-07-20 至 2026-08-02，用户峰值 7 人、问题峰值 26 个，末两日无问题正确显示 0，页面中无 `codex`。

## Documentation Sync

- `docs/decisions.md` 与 `docs/runbooks/public-user-admin.md` 已同步趋势图口径和验收步骤。
- 同日既有 handoff 已追加趋势图阶段；`docs/archive/index.md` 已更新。

## Risks / Open Questions

- 图表基于当前完整 14 天 rows；未来若 API 分页，趋势序列必须改由服务端返回，不能基于当前页推导。
- 本阶段未部署、提交、推送或发布。
