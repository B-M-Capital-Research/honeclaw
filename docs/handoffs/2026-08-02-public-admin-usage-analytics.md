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
  - docs/archive/plans/public-admin-all-channel-usage.md
  - docs/decisions.md
  - docs/invariants.md
  - docs/runbooks/public-user-admin.md
- related_prs: `39ce9ce54f5cbfea26e664459cb70edf3fd97292`, `c4c217236fae8bbe571f259cd46b6b4768178bcf`

## Summary

Public `/me` 的管理员区域提供实时 HONE 使用统计。它按北京时间展示最近 14 天网页、飞书、Telegram、Discord、iMessage 支持渠道中每位渠道账号的真实提问、问题明细、定时任务执行、成功推送和失败投递，并生成随日期筛选变化的使用人数、问题数、成功推送数、同比与主要降频用户摘要。普通用户既不挂载组件，也会被服务端 `403` 拒绝。

## What Changed

- 新增 `GET /api/public/admin/usage`，复用现有 cookie session + 数据库管理员角色复核。
- 按 `(channel, user_id)` 聚合支持渠道的 session `user` 消息；群聊只使用具体 actor user id，不把共享 scope 当成用户，并排除 scheduler/heartbeat source、job metadata 与旧版触发 envelope。
- 定时数据来自 cron execution history，不把 noop 当推送，也不把 `should_deliver=false` 当失败。
- `/me` 提供摘要、手动刷新、14 天/单日筛选、两张趋势图和可展开问题明细；桌面与移动端均保留一张有界、可双向滚动的大表格，并明确展示渠道。
- Web 手机号展示为掩码；非 Web 用户仅展示渠道与标识后六位。每个问题预览最多 1000 字，不返回助手答案、凭据或不支持渠道内容。

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

先从 `crates/hone-web-api/src/routes/public_admin.rs::handle_usage_report` 和 `build_usage_report` 检查统计口径，再从 `packages/app/src/components/public-admin-usage-panel.tsx` 调整展示。任何扩大到导出、更长留存期或跨渠道身份合并的需求都需要重新评估管理员权限、身份真相源与隐私边界。

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
- 首次管理员授权验证误用了本机 `.env` 指向的转发 PostgreSQL，虽然本机读取为 `verified_is_admin=true`，但并未改变 GCE 生产实例实际使用的 PostgreSQL；该结论已在同日“生产管理员授权纠偏阶段”中更正。
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

## 2026-08-02 生产管理员授权纠偏阶段

### Summary

- 用户以 `181****4550` 登录生产 `/me` 后仍看不到管理员区域。Chrome 强制刷新确认账号为 `web-user-9b62484ff43d`、前端资产为当前版本，且“HONE 使用统计”和“会员白名单”均未渲染，因此排除普通前端缓存问题。
- GCE `hone-web.service` 实际使用 `/etc/hone/runtime.env`、`/srv/honeclaw/config.yaml` 与本机 `127.0.0.1:5432/db_bamang_research` PostgreSQL；直接读取该生产库确认三个目标账号的 `is_admin` 均为 `false`。此前本机 `127.0.0.1:55432` 读取到的 `true` 属于另一条数据库连接，不能作为生产授权证据。
- 在 GCE 主机上使用已部署的 `/opt/hone/current/bin/hone-cli`、生产 config 与 runtime env 对三个目标账号先 dry-run、再 `--apply`。三个账号均唯一、active，apply 均返回 `changed=true`、`verified_is_admin=true`。
- 权限接口每次请求重新读取服务端角色，无需重启。刷新已登录的生产 Chrome 页面后，`181****4550` 账号成功显示“HONE 使用统计”、两张 14 日趋势图、真实统计表和“会员白名单”。

### Verification

- GCE-hosted CLI dry-run：三个目标均为 `previous_is_admin=false`、`requested_is_admin=true`。
- GCE-hosted CLI apply：三个目标均为 `changed=true`、`verified_is_admin=true`。
- 生产 Chrome：当前账号 `web-user-9b62484ff43d`；“HONE 使用统计”“每日使用用户数”“每日提问量”“会员白名单”标题均存在；统计表加载真实脱敏行；控制台无 warning/error。
- `hone-web.service` 未重启，避免不必要的会话中断；角色刷新即时生效。

### Risks / Follow-ups

- 生产管理员授权必须在 GCE 上复用 `hone-web.service` 的有效 config/env 执行。本机端口转发仅可用于明确标识过目标实例的诊断，不能单独作为生产变更或 read-after-write 证据。
- `docs/runbooks/public-user-admin.md` 已增加 GCE 生产权威性检查和 host-local CLI 执行方式。

## 2026-08-02 全渠道统计与生产部署阶段

### Summary

- 在改逻辑前先只读统计 GCE 生产数据。北京时间 2026-07-20 至 2026-08-02：飞书 43 个提问账号、556 个真实问题，网页 23 个提问账号、116 个问题，Discord 1 个提问账号、7 个问题；飞书成功推送 1,266 条，网页 919 条，Discord 7 条。Telegram 与 iMessage 在该窗口无数据。
- API row 新增 `channel`，session 与 cron execution 都扩展到 `web`、`feishu`、`telegram`、`discord`、`imessage`；同一外部 id 在不同渠道分别计数。群聊仅在存在具体 actor user id 时纳入。
- 前端趋势、摘要和降频归因均改为以 `(channel, user_id)` 去重；表格新增渠道列，并明确提示跨渠道未绑定账号分别计数。
- 提交 `c4c217236fae8bbe571f259cd46b6b4768178bcf` 已推送 `main`。Cloudflare Pages 生产懒加载包包含新渠道文案；GCE 从 `39ce9ce54f5cbfea26e664459cb70edf3fd97292-admin-usage-20260802` 原子切换到 `/opt/hone/releases/c4c217236fae8bbe571f259cd46b6b4768178bcf-all-channel-usage-20260802`。

### Verification

- 本地：Public-admin 8/8、Web API 165 passed + 2 ignored、Web 344/344、TypeScript typecheck、`cargo check -p hone-web-api`、Public production build 和 diff/格式检查均通过。
- GCE：6 个 release 二进制 SHA256 全部通过；Web 与 Feishu 都内嵌精确 Git SHA。连续两次 active chat 均为 0；切换后 Web 第 2 次探测就绪，Web/Feishu active，飞书 WebSocket 重新连接成功，warning 级日志为空。
- 运行时 `/api/meta`：`git_sha=c4c21723...`、`profile=release`、`source=workspace`，PostgreSQL/R2 健康，`cloud_storage_authoritative=true`，`local_durable_dependency_count=0`。本地与公网未登录鉴权均返回预期 `401`。
- `181****4550` 的生产 Chrome：14 日总计 65 个渠道账号、401 个真实问题、1,874 条成功推送；其中飞书 43 人/289 问题/1,080 推送，网页 21 人/109 问题/788 推送，Discord 1 人/3 问题/6 推送。窗口已在北京时间跨到 8 月 3 日，因此与前述 7 月 20 日至 8 月 2 日的只读快照不同。
- 页面摘要为“最近 7 天比前 7 天少 10 人，主要是飞书用户 c6079c 使用频率降低 10 次”；8 月 2 日为 14 人、50 问题、81 推送，8 月 1 日为 9 人、19 问题、0 推送。334 行表格无 `codex*` actor，渠道列、两张图、白名单均存在，控制台无 warning/error。

### Rollback / Cleanup

- 直接回滚点为 `/opt/hone/releases/39ce9ce54f5cbfea26e664459cb70edf3fd97292-admin-usage-20260802`；按 Feishu → Web 停止、原子切换 `/opt/hone/current`、Web → Feishu 启动即可恢复。
- 临时远端 worktree、构建日志和验收临时文件已删除；新旧不可变 release 均保留，可恢复。未新增 release tag。
