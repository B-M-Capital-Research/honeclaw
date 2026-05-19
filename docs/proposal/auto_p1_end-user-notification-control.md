# Proposal: End-User Notification Control Center

status: proposed
priority: P1
created_at: 2026-05-19 20:03:56 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/router/mod.rs`
- `crates/hone-event-engine/src/unified_digest/`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `skills/notification_preferences/SKILL.md`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/schedule.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/schedule.tsx`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/lib/api.ts`

## 背景与现状

Hone 的主动推送能力已经从简单定时任务进化成一套较完整的事件与通知系统：

- `NotificationPrefs` 支持 per-actor 总开关、持仓过滤、severity 阈值、kind allow/block、source allow/block、digest slots、价格阈值、quiet mode、quiet hours、timezone、主线蒸馏字段等运行时偏好。
- event-engine router 每次 dispatch 按 actor 读取 prefs，因此偏好变更可以在下一条事件立即生效。
- `notification_prefs` tool 和 `skills/notification_preferences/SKILL.md` 已允许用户在 IM / Web chat 中用自然语言调整自己的推送偏好。
- 管理端已有 `/api/notification-prefs`、`/api/admin/schedule`、`/api/admin/notifications`，可以代 actor 查看或修改 prefs、查看日程和推送记录。
- Public Web 已有 `/portfolio` 投资上下文页，用户可以看到 mainline distill、公司画像摘要和手动刷新入口；`/me` 目前主要是账户信息与导航。

这说明底层能力强，但产品面仍偏“隐藏在对话和管理后台里”。终端用户要知道自己现在会收到什么、什么时候收、为什么被静音、如何临时关闭、是否只收持仓相关事件，仍需要问 agent 或让管理员查看。对一个长期投资助理而言，推送信任不仅取决于模型质量，也取决于用户能否控制打扰节奏。

## 问题或机会

当前缺口集中在“用户自助、可见、可预览、可撤销”：

1. **自然语言修改不等于可审计设置面。**  
   `notification_prefs` tool 适合快速改偏好，但用户很难像查看手机通知设置一样确认最终状态。复杂字段如 digest slots、quiet hours、immediate kinds、价格上下行阈值和 source filters 不适合只靠 chat 复述。

2. **Public Web 没有用户级通知控制页。**  
   `/portfolio` 展示投资主线，`/me` 展示账户信息，但没有“我的推送”入口。用户不知道 Hone 今天会在几个窗口打扰自己，也不知道哪些事件会被 quiet hold、进入 digest 或直接送达。

3. **管理端是运维视角，不是用户体验。**  
   `packages/app/src/pages/schedule.tsx` 和 `notifications.tsx` 面向管理员排查任意 actor，包含 actor 选择、执行状态、记录来源等运维字段。它不能直接暴露给 public 用户，也不能表达“修改前预览”和“修改后确认”的产品流程。

4. **推送打扰直接影响留存和付费。**  
   Hone 的差异是持续守护投资纪律。推送过多会被静音，推送过少会被遗忘，推送不可解释会被认为不可靠。一个用户可控的通知中心能降低退订、减少管理员代配置成本，并为未来权益分层提供更清楚的功能面。

机会是：仓库已经有 prefs 数据结构、校验逻辑、schedule overview、delivery log、public auth、public digest context 和 chat tool。第一版不需要重构 event-engine，只需要把现有能力包装成终端用户能理解和确认的控制面。

## 方案概述

新增 **End-User Notification Control Center**：面向 public Web、desktop remote/bundled shell 和 IM 深链的用户自助推送设置页。

第一版定位为“设置 + 预览 + 最近效果”，不直接做复杂自动优化：

- Public Web 新增 `/notifications` 或 `/me/notifications`。
- 后端新增 public-auth 绑定的 notification prefs API，只能读写当前 web actor。
- 提供可视化日程预览：digest slots、cron jobs、quiet hours、immediate 规则、最近 held / skipped / sent 统计。
- 修改设置采用 preview/apply 二段式：先展示将改变什么，再保存。
- Chat 里的 `notification_prefs` skill 在完成修改后可以返回一个“查看当前推送设置”的 Web 链接或短入口。
- 管理端继续保留代用户排查能力，但用户页只暴露自己的 actor，不暴露任意 actor 查询。

## 用户体验变化

### 用户端

- `/me` 增加“推送设置”入口。
- `/notifications` 展示：
  - 总开关：暂停全部推送 / 恢复推送。
  - 打扰强度：全部、只看重要、极简模式。
  - 范围：全部市场事件、只看持仓相关、只看指定事件类型。
  - 时间：timezone、digest slots、quiet hours、quiet exceptions。
  - 价格提醒：默认阈值、上涨/下跌单独阈值、低仓位保护说明。
  - 最近 7 天效果：发送、digest、quiet held、prefs filtered、失败。
- 每次修改前显示一句清楚预览，例如：
  - “保存后，High 财报仍会立即推送；普通新闻将进入 09:00 / 20:30 digest。”
  - “23:00-07:00 期间不会立即推送，除非是 earnings_released。”

### 管理端

- 管理端 `schedule` 和 `notifications` 不替换，只补一个“以用户视角查看”的链接或 tab，帮助客服/运营解释用户看到的设置。
- 管理员代改 prefs 时，记录来源应区分 `admin_override` 与 `self_service`，便于未来 operator audit 和用户争议排查。

### 桌面端

- Desktop bundled/remote 都可以复用 public/user route 或内嵌同一页面。
- Bundled 模式可额外显示“本机 channel 进程是否在线”的只读提示，但不把启动/修复流程混入本提案；sidecar 可靠性仍由现有 desktop/runtime proposal 处理。

### 多渠道

- IM 中用户说“我想改推送”时，agent 仍可直接调用 `notification_prefs` tool。
- 对复杂修改，agent 可先调用 overview，再建议用户打开 Web 控制面确认。
- Digest 或事件卡片底部可逐步追加轻量操作入口，例如“少推这类”“只进摘要”“暂停今晚”，最终落到同一 prefs preview/apply API。

## 技术方案

### 数据模型

第一版复用 `NotificationPrefs`，不新增第二套偏好模型。为前端体验新增几个投影类型：

- `NotificationControlState`
  - `prefs`: 当前 `NotificationPrefs` 的安全投影。
  - `schedule_overview`: 复用 `schedule_view::build_overview` 的结果。
  - `recent_effect`: 从 `/api/admin/notifications` 同源聚合逻辑中提取当前 actor 最近效果。
  - `available_kind_tags`: 来自 `ALL_KIND_TAGS`，附中文标签和风险说明。
  - `capabilities`: 标记哪些字段当前可编辑，哪些只读或由系统蒸馏维护。

- `NotificationPrefsPatch`
  - 只允许 public 用户改自助字段：`enabled`、`portfolio_only`、`min_severity`、`allow_kinds`、`blocked_kinds`、`timezone`、`digest_slots`、`price_high_pct_*`、`immediate_kinds`、`quiet_hours`、`quiet_mode`、`allow_sources`、`blocked_sources`。
  - 不允许 public UI 直接写 `mainline_style`、`mainline_by_ticker`、`last_mainline_distilled_at`、`mainline_distill_skipped`。这些仍由公司画像和 mainline distill 管理。

- `NotificationPrefsPreview`
  - `before` / `after` 摘要。
  - `changed_fields`。
  - `delivery_examples`: 几个典型事件在保存后会 direct / digest / quiet held / filtered。
  - `warnings`: 例如 digest slots 为空会关闭摘要、allow list 会过滤其它事件、timezone 不合法。

### API

在 `crates/hone-web-api/src/routes/public.rs` 或独立 `public_notification_prefs.rs` 下新增：

- `GET /api/public/notification-prefs`
  - 从 `hone_web_session` 推导 `ActorIdentity(channel="web", user_id, None)`。
  - 返回 `NotificationControlState`。

- `POST /api/public/notification-prefs/preview`
  - 输入 `NotificationPrefsPatch`。
  - 复用 `notification_prefs.rs` 的校验逻辑；建议把当前 admin route 内的 `validate_prefs` 抽成共享函数，避免 admin/public 分叉。
  - 不写盘，只返回 `NotificationPrefsPreview`。

- `PUT /api/public/notification-prefs`
  - 输入同 patch，加可选 `preview_token` 或 `confirmed=true`。
  - 写入当前 actor 的 prefs 文件。
  - 返回保存后的 `NotificationControlState`。

Admin API 可后续补 `updated_by` 元信息；第一版如果不改存储结构，可以先把来源写入 delivery/admin audit 后续 proposal 的扩展点，不阻塞自助页。

### 前端

新增或复用：

- `packages/app/src/pages/public-notifications.tsx`
- `packages/app/src/lib/public-notification-prefs.ts`
- `packages/app/src/pages/public-notifications-model.ts`

页面不直接展示原始 JSON，而是使用几组稳定控件：

- segmented control：打扰强度。
- checkbox group：事件类型 allow/block。
- time inputs：digest slots 与 quiet hours。
- numeric inputs：价格阈值。
- preview panel：保存前说明影响。
- recent effect table：只显示自己的最近推送结果，不显示其它 actor。

### 兼容策略

- 缺少 prefs 文件时继续使用 `NotificationPrefs::default()`，保持现有默认全放行行为。
- 旧用户无需迁移；首次打开页面看到默认状态。
- 管理端和自然语言 tool 写出的 prefs 文件必须仍可被 public 页面读取。
- Public 页面保存时必须保留未知字段，避免未来 `NotificationPrefs` 扩展后被旧前端覆盖。后端应用 patch 时从当前 prefs merge，而不是前端提交完整对象覆盖。
- 如果 actor 未来通过 linked workspace 合并身份，第一版仍按当前 web actor 写 prefs；workspace 级 prefs 属于后续扩展。

## 实施步骤

1. **共享校验与投影层**
   - 将 admin `notification_prefs.rs` 中的 prefs 校验抽出为可复用函数。
   - 新增 `NotificationControlState` / `NotificationPrefsPatch` / `NotificationPrefsPreview` 类型。
   - 复用 `schedule_view::build_overview` 生成用户可理解的 schedule。

2. **Public API**
   - 新增 current-user-only 的 get/preview/apply 路由。
   - 从 `public_auth` 推导 web actor，禁止传入任意 actor。
   - 聚合最近推送效果时复用 notifications route 的数据源，但强制 actor filter。

3. **Public Web 页面**
   - 在 `APP_SURFACE=public` 下新增 `/notifications` route。
   - `/me` 增加入口。
   - 页面使用 preview/apply；保存后刷新 state。

4. **Chat 与多渠道入口**
   - `notification_prefs` skill 的最终回复增加“可在 Web 查看/调整”的短提示。
   - 对复杂修改建议走 preview 链接；简单开关仍由 tool 直接执行。

5. **Admin 辅助视图**
   - 在用户详情或 schedule 页增加“用户视角预览”入口。
   - 明确 admin 代改和用户自助的权限边界。

6. **后续扩展**
   - 增加一键操作：少推这类、只进摘要、暂停到明早。
   - 与权益系统联动：不同 plan 开放不同 digest slots、渠道数量或 source filters。
   - 与 response feedback 联动：用户认为某类推送无用时，可直接转为 prefs 建议而不是只记录差评。

## 验证方式

- 单元测试：
  - patch merge 不覆盖未知/系统字段。
  - public API 不能修改其它 actor。
  - `allow_kinds`、`blocked_kinds`、timezone、digest slot、quiet hours 校验与 admin API 一致。
  - preview 中典型事件分类符合 router/prefs 规则。

- 前端测试：
  - `public-notifications-model` 覆盖默认状态、只看重要、静音、digest slots 为空、quiet hours 跨午夜等状态转换。
  - route smoke：未登录跳转登录，登录后能读到默认设置，保存后页面刷新保持一致。

- 回归脚本：
  - 新增 CI-safe API contract 测试，使用本地临时 prefs dir 和 mock public session。
  - 不依赖真实短信、Feishu、Discord、Telegram 账号。

- 手工验收：
  - Public 用户通过 `/me` 打开设置，关闭全部推送，再触发一条 event-engine 测试事件，应在通知日志显示 prefs filtered。
  - 设置 quiet hours 覆盖当前时间，High 事件应进入 quiet held，用户页面最近效果能显示。
  - 在 chat 中让 Hone “只推财报和 SEC”，设置页能立即反映 allow list。

- 指标：
  - 用户自助修改次数。
  - 修改后 7 天内推送失败/静音/退订下降。
  - 用户打开通知设置页后继续使用 chat、portfolio、digest 的留存变化。

## 风险与取舍

- **风险：设置项太多，用户看不懂。**  
  第一版用 preset + advanced 折叠，不把完整 JSON 暴露给用户。高级字段只在用户主动展开时显示。

- **风险：自然语言 tool 和 Web UI 产生双重真相源。**  
  所有路径都写同一个 `NotificationPrefs` 文件；Web 使用 patch merge；tool 继续复用同一存储。

- **风险：用户误关关键推送。**  
  preview 明确说明影响；关闭全部推送时要求二次确认；允许保留 emergency exceptions，但第一版不强制引入。

- **风险：public API 误暴露 admin 能力。**  
  Public route 不接受 actor 参数，只从 session 推导当前 web actor；recent effect 聚合必须强制 actor filter。

- **风险：与 future workspace prefs 冲突。**  
  第一版只做 actor 级 prefs，不设计 workspace 继承。后续 linked workspace 落地后，可增加 workspace default + actor override 层。

- **不做的边界：**
  - 不重写 event-engine router。
  - 不新增通知智能推荐模型。
  - 不把 company mainline 字段开放给用户手工编辑。
  - 不把管理端完整 notifications log 直接暴露给 public 用户。
  - 不处理 channel sidecar 进程修复和 desktop startup，这属于现有 runtime/desktop 工作流。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 下全部 `auto_p*.md` 和历史 `docs/proposals/`：

- 不重复于 `auto_p1_delivery_decision_loop.md`：该提案关注单条通知为什么 direct / digest / skipped / suppressed；本提案关注用户如何自助设置未来通知规则。
- 不重复于 `auto_p1_temporal-operations-calendar.md`：该提案关注自动化与推送的未来日程可见性；本提案补充可编辑的用户通知偏好、预览和保存流程。
- 不重复于 `auto_p1_automation_intent_control_plane.md`：该提案治理 agent 创建自动化的 intent/preview；本提案治理 event-engine 和 digest 的打扰策略。
- 不重复于 `auto_p1_response-feedback-learning-loop.md`：该提案收集回答质量反馈；本提案是通知频率和类型控制面。
- 不重复于 `auto_p1_usage_entitlement_ledger.md`：权益 ledger 解决功能额度和成本；本提案只处理用户通知偏好，即使未来可被权益分层引用。
- 不重复于 `auto_p1_run_trace_workbench.md` 和 `auto_p1_runtime_readiness_matrix.md`：它们面向排障与运行准备度；本提案面向终端用户日常控制。
- 不重复于 `docs/proposals/desktop-bundled-runtime-startup-ux.md`：该历史提案处理桌面 bundled runtime 启动体验；本提案只在桌面壳里复用通知设置页。
- 不重复于 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该历史提案处理 skill/runtime 语义；本提案只把现有 `notification_preferences` skill 的结果产品化展示。

查重结论：现有 proposal 已覆盖通知解释、自动化日程、运行排障、用户反馈、权益、桌面启动和 skill runtime，但没有覆盖“终端用户自助通知设置中心 + preview/apply + 最近效果”这一独立产品/架构层。该主题能直接降低推送打扰造成的流失，并把 Hone 的主动守护能力从后台功能变成可被用户信任和控制的产品面。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/end-user-notification-control.md`，并在新增 public API、公共前端 route、prefs patch contract 或通知偏好长期约束时同步更新 `docs/repo-map.md`、`docs/invariants.md`，必要时补充 admin/public 权限边界 decision。
