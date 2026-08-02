# Public 管理员使用统计交接

- title: Public 管理员使用统计交接
- status: done
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
- related_docs:
  - docs/archive/plans/public-admin-usage-analytics.md
  - docs/archive/plans/public-admin-usage-production-rollout.md
  - docs/decisions.md
  - docs/invariants.md
  - docs/runbooks/public-user-admin.md
- related_prs: none

## Summary

Public `/me` 的管理员区域新增实时 HONE 使用统计。它按北京时间展示最近 14 天每位 Web 用户的真实提问、问题明细、定时任务执行、成功推送和失败投递，并生成今日活跃提问人数、问题数、成功推送数、上周同日同比与主要降频用户摘要。普通用户既不挂载组件，也会被服务端 `403` 拒绝。

## What Changed

- 新增 `GET /api/public/admin/usage`，复用现有 cookie session + 数据库管理员角色复核。
- 按 direct Web actor 聚合 session 的 `user` 消息；排除 scheduler/heartbeat source、job metadata 与旧版触发 envelope。
- 定时数据来自 cron execution history，不把 noop 当推送，也不把 `should_deliver=false` 当失败。
- `/me` 增加摘要、手动刷新、14 天/单日筛选和可展开问题明细；桌面使用横向表格，移动端转为逐用户卡片。
- 手机号展示为掩码；每个问题预览最多 1000 字，不返回助手答案或其它渠道内容。

## Verification

- `cargo check -p hone-web-api`：通过。
- `cargo test -p hone-web-api --lib`：163/163 通过，2 项凭证型 live smoke 按预期 ignored；其中 public-admin 6/6 通过。
- Web TypeScript typecheck：通过。
- 完整 Web 测试：337/337 通过。
- Public 生产构建：通过；仅保留仓库既有的大 chunk 警告。
- 本地 mocked API 浏览器验收：桌面 1280×720、移动 390×844 均无横向溢出；摘要、表格、移动卡片、问题展开正常，控制台无 warning/error。

## Risks / Follow-ups

- 当前报告在请求时读取完整 session 列表再截取 14 天窗口。现有用户规模可接受；若 Web session 数量显著增长，应在 PostgreSQL 层增加按消息时间过滤的只读投影，保持 API 和统计口径不变。
- cron execution 查询上限为 50,000 条/14 天。达到该量级前应改为数据库侧分组或分页，避免静默截断成为长期行为。
- 本次未部署、未读取生产用户问题，也未执行真实管理员/会员变更；生产验收按 `docs/runbooks/public-user-admin.md` 进行。

## Next Entry Point

先从 `crates/hone-web-api/src/routes/public_admin.rs::handle_usage_report` 和 `build_usage_report` 检查统计口径，再从 `packages/app/src/components/public-admin-usage-panel.tsx` 调整展示。任何扩大到非 Web 渠道、导出或更长留存期的需求都需要重新评估管理员权限与隐私边界。

## 2026-08-02 精简与日期联动阶段

### Summary

- 服务端现在对 session 与 cron execution 同时排除 trimmed、大小写不敏感的 `codex*` 用户标识；普通用户标识中间包含 `codex` 不会被误伤。
- 顶部摘要不再固定展示服务端的“今日”句子，而是从同一批权威 rows 按当前下拉选项派生：最近 14 天比较前后两个 7 天窗口，单日比较 7 天前同日，超出窗口则明确提示无可比数据。
- 使用统计与会员白名单改为独立可折叠区块；统计默认展开、白名单默认收起。统计仍是一张大表格，最大高度固定、内部双向滚动、表头吸顶，移动端不再展开为冗长卡片列表。

### Verification

- Public-admin Rust：7/7；完整 Web API：164/164，2 ignored。
- Web 定向：13/13；完整 Web：340/340；TypeScript typecheck 与 Public build 通过。
- 管理员登录态真实本地报告从 137 行变为 112 行，DOM 不含 `codex`；“最近 14 天”切换到 7 月 31 日后，摘要同步变为当天 3 人、6 个问题、0 条成功推送及上周同日对比。
- 两个折叠控件均经真实页面点击验收；后端使用 `HONE_RUNTIME_ROLE=web`，未启动 scheduler、event engine 或 channel workers，未产生推送和白名单写入。

### Risks / Follow-ups

- API 为兼容仍返回原有 server summary；当前 UI 以 rows + period boundaries 计算选中范围摘要。若未来改成服务端分页，必须同步把选中范围摘要下推 API，不能只基于当前页计算。
- 本阶段未部署、提交、推送或发布。

## 2026-08-02 两周趋势图阶段

### Summary

- 统计摘要与大表格之间新增“每日使用用户数”和“每日提问量”两张原生 SVG 折线图，无新增第三方依赖。
- 两张图共享截至报告 `period_end` 的连续 14 天横轴；没有 row 的日期补 0。用户量按当天有真实问题的 `user_id` 去重，提问量按 `question_count` 求和，push-only rows 不计入两项趋势。
- 桌面并排展示；移动端保持单行横向滑动，避免再次拉长管理页面。每个点含日期和数值的可访问标题。

### Verification

- 趋势与样式定向测试 12/12、完整 Web 341/341、TypeScript typecheck、Public production build 全部通过。
- 管理员登录态真实页面显示两张 chart、28 个总点位；横轴连续覆盖 7 月 20 日至 8 月 2 日，真实用户峰值 7 人、问题峰值 26 个，8 月 1 日和 8 月 2 日正确显示 0。
- 既有 `codex*` 过滤、摘要、表格和折叠交互保持正常；没有白名单写入、推送、部署、提交或发布。

### Risks / Follow-ups

- 趋势目前依赖 API 一次返回完整 14 天 rows；若以后引入分页，必须由服务端提供完整趋势序列。

## 2026-08-02 完整日期选项修复阶段

### Summary

- 修复日期下拉只从稀疏 rows 取值、因此遗漏零活动日期的问题；选项现在严格从报告 `period_end` 向前生成 14 个日期。
- 刷新校验改为判断所选日期是否仍在 `period_start..=period_end`，不再因当天没有 row 而退回“最近 14 天”。
- 零活动日期继续显示上周同日比较，并将本日用户、问题和成功推送显示为 0，表格显示空状态。

### Verification

- 统计与样式定向测试 14/14、完整 Web 343/343、TypeScript typecheck 通过。
- 管理员登录态页面的下拉框依次显示 8 月 2 日、8 月 1 日、7 月 31 日直到 7 月 20 日；8 月 1 日和 2 日均可选择并显示 0 人、0 问题、0 推送。
- 未改变后端、图表、白名单、推送或授权行为；没有部署、提交或发布。

## 2026-08-02 生产上线阶段

### Summary

- 功能提交 `39ce9ce54f5cbfea26e664459cb70edf3fd97292` 已推送到 `main`；本次没有创建 release tag。
- Cloudflare Pages 已切换到 `index-vHyTHbU6.js`。公网 `/`、`/me` 返回 `200`，新增 `/api/public/admin/usage` 在未登录时返回正确的 `401`，不再是旧后端的 `404`。
- GCE 在连续两次确认活动会话为 0 后，从 `/opt/hone/releases/d48c1f50-feishu-heartbeat-20260801` 原子切换到 `/opt/hone/releases/39ce9ce54f5cbfea26e664459cb70edf3fd97292-admin-usage-20260802`。Web 与 Feishu systemd 服务均为 active，运行时 Git SHA 与提交一致，PostgreSQL/R2 健康且本地持久化依赖数为 0。
- 三个指定账号 `181****4550`、`135****3292`、`139****9177` 均通过 PostgreSQL 权威身份路径授予管理员；二次 dry-run 读取均为 active、`previous_is_admin=true`、`changed=false`、`verified_is_admin=true`。
- 临时 2 GiB 构建 swap、远端构建/上传中间目录和本地 30 MiB 上传包均在稳定性复核后删除；GCE `enable-oslogin-2fa` 已恢复为 `TRUE`，临时 gcloud 配置已销毁。

### Verification

- GitHub：CI、Secret Scan、CodeQL、Release Cache Warm Linux 全部成功。
- 本地：`cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`、Web 343/343、Public build、Public Community Edge 45/45、CI-safe regression、Web API 164/164（另 2 个凭证 smoke ignored）与管理员统计 7/7 均通过。
- 云端：不可变发布包 1,860 项 checksum 通过；切换后 Web 第 2 次探测就绪，Feishu 第 1 次探测 active，持续检查无 warning/error，活动会话保持 0。
- 公网：`/api/public/auth/me`、`/api/public/admin/invites`、`/api/public/admin/usage` 均返回未登录 `401`，证明路由已生效且权限边界仍在。
- 完整本地 workspace Rust tests 中，未改动的 `hone-channels` FMP stub 有 10 项失败；隔离复现一致，GitHub CI 与本次涉及模块的测试均通过，因此未把无关测试夹具纳入本次改动。

### Rollback / Follow-ups

- GCE 回滚点保留为 `/opt/hone/releases/d48c1f50-feishu-heartbeat-20260801`；将 `/opt/hone/current` 原子指回该目录并按 Web → Feishu 顺序启动即可回退。首次切换因验证脚本 stdin 写法错误触发过一次自动回滚，旧版恢复健康后修正脚本并完成最终切换，证明回滚路径有效。
- GCE `/api/meta` 的 Git SHA 与二进制哈希可信，但 build source 仍显示 `unknown`；后续部署工具应补齐该 provenance 字段。
- 本机源码运行时为通过现有 `codex-acp --version` 前置检查保留了兼容 shim；云端 GCE 不依赖该 shim。应在 ACP umbrella 任务中统一探测契约，避免下次冷启动需要本机兼容层。
