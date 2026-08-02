# Public 管理员统计完整日期选项

- title: Public 管理员统计完整日期选项
- status: archived
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - packages/app/src/components/public-admin-usage-panel.tsx
  - packages/app/src/components/public-admin-usage-panel.test.ts
- related_docs:
  - docs/decisions.md
  - docs/runbooks/public-user-admin.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md

## Goal

让管理员统计日期下拉始终展示报告覆盖的完整 14 个北京时间自然日，包括没有提问和推送记录的零活动日期。

## Scope

- 日期选项从报告 `period_start` / `period_end` 生成，不再依赖稀疏 rows。
- 刷新时只在所选日期超出报告时间窗时重置，不因当日 rows 为空而重置。
- 零活动日期复用现有摘要与空表状态，显示 0 人、0 个问题、0 条推送。
- 不改变 API、聚合口径或趋势图。

## Validation

- 统计与样式定向测试：14/14 通过。
- 完整 Web 测试：343/343 通过；TypeScript typecheck 通过。
- 管理员登录态真实页面显示从 8 月 2 日至 7 月 20 日的完整 14 天；8 月 1 日、2 日均可选，摘要为 0 人/0 问题/0 推送，表格显示空状态且上周同日比较保留。

## Documentation Sync

- `docs/decisions.md` 与 `docs/runbooks/public-user-admin.md` 已同步完整日期选项口径。
- 同日既有 handoff 已追加日期选项修复阶段；`docs/archive/index.md` 已更新。

## Risks / Open Questions

- 依赖服务端 `period_start` / `period_end` 为有效 ISO 日期；非法范围安全返回空选项。
- 本阶段未部署、提交、推送或发布。
