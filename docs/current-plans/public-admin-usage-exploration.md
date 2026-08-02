# Public Admin Usage 数据探索与统一上线

- title: Public Admin Usage 数据探索与统一上线
- status: in_progress
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public_admin.rs`
  - `crates/hone-web-api/src/types.rs`
  - `packages/app/src/components/public-admin-usage-panel.tsx`
  - `packages/app/src/components/public-admin-usage-panel.test.ts`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/styles.css`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/public-admin-usage-exploration.md`
  - `docs/archive/index.md`

## Goal

把管理员使用统计扩展为口径一致的数据探索页：支持渠道分类、14/30/90 天追溯、可点击折线数据点及精确横纵值，并让摘要、图表、日期与大表格统一跟随当前筛选；完成测试、提交推送和生产统一更新。

## Scope

- 后端统计接口接收受控统计周期，返回真实周期长度，并让定时执行查询容量随周期安全扩展，避免长周期静默截断。
- 前端增加周期和渠道筛选；所有摘要、趋势、日期选项和表格使用同一渠道/日期口径。
- 两张趋势图支持鼠标和键盘选择数据点，展示日期、使用人数和提问量的精确数字，并可跳转到当天明细。
- 补齐 API、统计纯函数、渠道隔离、长周期零填充和可交互图表契约测试。
- 按精确 revision 构建并以可回滚、零活跃会话切换更新 GCE；确认 Cloudflare Pages 对应前端已上线。

## Validation

- `cargo test -p hone-web-api public_admin`
- `bun test packages/app/src/components/public-admin-usage-panel.test.ts packages/app/src/lib/api.test.ts`
- `bun run --cwd packages/app build`
- `cargo check -p hone-web-api`
- 上线前核对 revision/provenance、健康状态与活跃会话；上线后验证 Web、飞书、管理员接口和生产页面筛选/点击行为。

## Documentation Sync

- 实施期间更新本计划与 `docs/current-plan.md`。
- 完成后新增 `docs/handoffs/public-admin-usage-exploration.md`，记录统计口径、踩坑、验证、部署与回滚信息。
- 任务退出活跃态时把本计划移到 `docs/archive/plans/`，从 `docs/current-plan.md` 移除并更新 `docs/archive/index.md`。
- 本次不改变仓库模块边界或长期架构约束，预计无需更新 `docs/repo-map.md`、`docs/invariants.md` 或 ADR；若实现中发生变化再补充。

## Risks / Open Questions

- 90 天数据量显著增加，必须避免固定执行记录上限导致少算，同时控制最大请求范围。
- 渠道用户身份不能只按 `user_id` 去重；跨渠道未绑定账号继续按 `(channel, user_id)` 分开计数。
- 北京时间日界、无活动日期、刷新跨日和选中点过期都要保持可解释且不产生虚假零基线。
- 生产源码树可能有运维改动，必须从精确提交在独立目录构建，不覆盖远端工作区；切换前后都保留可验证 rollback 目标。
