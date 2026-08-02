# Public 管理员使用统计精简

- title: Public 管理员使用统计精简
- status: archived
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/public_admin.rs
  - packages/app/src/components/public-admin-usage-panel.tsx
  - packages/app/src/components/public-admin-whitelist-panel.tsx
  - packages/app/src/pages/public-workspace.css
  - packages/app/src/components/public-admin-usage-panel.test.ts
- related_docs:
  - docs/runbooks/public-user-admin.md
  - docs/decisions.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md

## Goal

排除管理员统计中的 `codex*` 自动化账号，压缩 `/me` 管理区域高度，并让顶部摘要严格跟随统计日期选择变化。

## Scope

- 服务端同时排除 `user_id` 以 `codex` 开头（忽略大小写和首尾空白）的 Web session 提问与 cron execution。
- 单日摘要按所选日期统计，并与 7 天前同日比较；最近 14 天摘要按整个窗口汇总，并比较前后两个 7 天窗口。
- 使用统计与会员白名单均支持收起/展开；统计表采用固定最大高度、表内滚动和粘性表头，避免拉长整页。
- 不改变管理员授权、白名单写操作和原始数据存储。

## Validation

- `cargo test -p hone-web-api --lib public_admin::tests`：7/7 通过。
- `cargo test -p hone-web-api --lib`：164/164 通过，2 项凭证型 live smoke 按预期 ignored。
- Web 定向测试：13/13 通过；完整 Web 测试：340/340 通过。
- TypeScript typecheck 与 Public production build：通过；构建仅保留既有的大 chunk 警告。
- 管理员登录态的本地真实数据浏览器验收：报告从 137 行降至 112 行，DOM 中无 `codex`；14 天与单日摘要联动、统计折叠、白名单折叠均通过。

## Documentation Sync

- `docs/runbooks/public-user-admin.md` 与 `docs/decisions.md` 已同步新筛选、摘要和布局口径。
- 同日既有 handoff 已追加精简阶段；`docs/archive/index.md` 已更新。

## Risks / Open Questions

- `codex` 过滤按用户标识前缀执行；不会模糊匹配普通用户名中间出现的 `codex`。
- 较早的单日若 7 天前已超出 14 天返回窗口，摘要会明确显示缺少可比数据，不把缺失误报为 0。
- 本次在本地进程读取了管理员授权的真实报告用于只读验收；没有执行白名单写操作、推送、部署、提交或发布。
