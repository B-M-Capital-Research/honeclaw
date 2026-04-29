# Proposal: Delivery Decision Loop for Notifications

- status: proposed
- priority: P1
- created_at: 2026-04-29 17:04 +0800
- owner: automation
- related_files:
  - `README.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/current-plan.md`
  - `docs/proposals/desktop-bundled-runtime-startup-ux.md`
  - `docs/proposals/skill-runtime-multi-agent-alignment.md`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-tools/src/notification_prefs_tool.rs`
  - `crates/hone-tools/src/missed_events_tool.rs`
  - `crates/hone-tools/src/schedule_view.rs`
  - `crates/hone-web-api/src/routes/notification_prefs.rs`
  - `crates/hone-web-api/src/routes/notifications.rs`
  - `crates/hone-web-api/src/routes/schedule.rs`
  - `packages/app/src/pages/notifications.tsx`
  - `packages/app/src/components/notification-preferences-card.tsx`
  - `packages/app/src/pages/schedule.tsx`
  - `packages/app/src/pages/public-portfolio.tsx`

## 背景与现状

Hone 的产品定位已经从普通聊天助手扩展成跨 Web、桌面和 IM 的投资研究工作台。当前事件与通知链路已经具备相当多的基础能力：

- `crates/hone-event-engine/src/router/dispatch.rs` 会把事件按订阅、偏好、严重度、cap、cooldown、quiet hours、digest 路径分流，并把结果写入 `delivery_log`。
- `crates/hone-event-engine/src/prefs.rs` 的 `NotificationPrefs` 已包含总开关、仅持仓、严重度、kind/source allow/block、digest slots、价格阈值、quiet mode、quiet hours、投资风格和 per-ticker thesis 等字段。
- `crates/hone-tools/src/notification_prefs_tool.rs` 允许终端用户通过自然语言修改自己的偏好，`missed_events_tool.rs` 可以查询被 curation、cap、cooldown 或偏好过滤掉的事件。
- 管理端已有 `packages/app/src/pages/notifications.tsx`，但它主要读取 `cron_job_runs` 聚合成推送日志；管理端设置页也有 `NotificationPreferencesCard` 用于修改 per-actor 偏好。
- 用户端 `/portfolio` 已展示 thesis 蒸馏上下文和公司画像，但尚未把“为什么这条事件推给我 / 为什么没推给我 / 我如何调整以后类似事件”做成闭环体验。

这些能力已经足以支撑可靠的通知系统，但它们在产品上仍是分散的：事件引擎知道决策原因，工具能查 missed events，管理端能看 cron 推送日志，偏好编辑器能改 JSON 对应字段，用户端能看 thesis，但用户和运维人员无法在一个对象上完成“追因 -> 调整 -> 验证”的闭环。

## 问题或机会

投资助手的主动通知不是普通消息推送。用户会用它来判断是否有 thesis-changing evidence、是否需要复盘持仓逻辑、是否应该降低噪音。只要出现“没收到我关心的事件”或“收到太多噪音”，信任感会很快下降。

当前主要缺口不是缺少过滤字段，而是缺少可解释的投递决策产品层：

- 用户收到一条推送后，很难知道它命中了哪个订阅、哪条偏好、为什么是 immediate 而不是 digest。
- 用户没收到事件时，只能通过自然语言触发 missed events，且这条能力没有在用户端或管理端形成可发现入口。
- 管理端 `推送日志` 关注 cron execution，event-engine 的 `delivery_log` 还不是同一张可排查时间线。
- `NotificationPreferencesCard` 能改偏好，但它不贴着具体事件给出“以后类似事件怎么处理”的调优动作。
- 商业化和留存角度，主动通知的价值需要被用户感知。只有“推了什么”不够，Hone 需要证明“为什么替你拦了什么、保留了什么、降噪依据是什么”。

值得投入 P1，是因为它直接提升核心体验、稳定性排障和用户信任，但不要求推翻现有事件引擎。它可以复用已有 `delivery_log`、prefs、missed tool、schedule overview 和前端页面，按产品闭环补齐。

## 方案概述

新增一条“投递决策闭环”产品能力：把每个 actor 的通知体验组织为一条可解释时间线，并允许从具体记录直接生成偏好调整建议。

核心对象建议叫 `DeliveryDecision`，不是替换 `delivery_log`，而是在 API / UI 层把现有事实聚合成统一视图：

- event 基础事实：kind、symbols、source、title、occurred_at、url、severity。
- actor 命中事实：actor、subscription source、portfolio/global 命中原因。
- 决策结果：sent、queued、filtered、capped、cooled_down、price_capped、price_cooled_down、quiet_held、failed、no_actor。
- 决策解释：用户可读的 reason label，以及机器可用的 reason code。
- 可操作建议：mute source、raise/lower price threshold、digest only、immediate kind、portfolio only、quiet hours、restore global default。
- 验证入口：调整后展示下一条事件立即生效，以及最近 24h/7d 类似事件会如何变化的 dry-run summary。

一期目标不要做复杂推荐系统，而是把现有确定性规则解释清楚。后续再加入基于历史事件的批量建议。

## 用户体验变化

用户端：

- `/portfolio` 增加“通知决策”子区块，展示最近 24h 的 `已推送 / 进入 digest / 被过滤 / 被降级 / quiet hold` 摘要。
- 每条记录可以展开，看到“为什么这样处理”：例如“命中 MU 持仓 + price_alert；达到 +6% 上行 band；未处于 quiet hours；因此 immediate 推送”。
- 对没收到的事件，用户不必知道 `/missed`，可以直接看到“被过滤/降级的事件”，并用按钮把相似来源加入黑名单、把某 kind 提升为 immediate、或只保留 digest。
- 对关键用户文案保持投资助手口径：强调“这是降噪/风控策略”，不是“系统漏发”。

管理端：

- 将现有 `推送日志` 从 cron execution-only 扩展为两条 tab：`任务推送` 和 `事件决策`。
- `事件决策` 读取 event-engine `delivery_log`，支持 actor、channel、kind、symbol、source、status、severity、时间范围过滤。
- 记录抽屉中合并展示当前 actor 的 `NotificationPrefs` 摘要、命中的决策规则、最近相似事件统计和一键跳转偏好编辑。
- `NotificationPreferencesCard` 可以从事件记录接收一个 draft patch，用户确认后写回 prefs，避免手动理解所有 JSON 字段。

桌面端：

- 桌面 bundled 模式下，渠道状态旁增加轻量健康提示：最近是否有 send failed、filtered 暴增、quiet held 未 flush。
- 保持桌面只承载 Web console，不新增独立存储；所有数据仍来自后端 API。

多渠道：

- IM 中继续保留自然语言 `notification_prefs` 与 `missed_events`，但响应里加入“可在 Web 用户端查看完整通知决策”的链接或短提示。
- Feishu / Telegram / Discord 不需要新增协议能力；只要最终消息中能解释 decision reason 即可。

## 技术方案

### 1. 扩展事件决策查询 API

在 `crates/hone-web-api` 增加只读路由，例如：

- `GET /api/event-engine/delivery-decisions`
- `GET /api/public/delivery-decisions`

admin 路由允许 query 指定 actor；public 路由从 `hone_web_session` 推导当前 web actor，禁止跨 actor 查询。

返回结构从 `EventStore` 聚合：

- `delivery_log` 行：actor、channel、severity、status、body、sent_at_ts。
- `events` 行：kind、symbols、title、summary、source、url、occurred_at。
- 可选读取当前 `NotificationPrefs`，生成 `prefs_snapshot_summary`，但不要把完整敏感配置塞进每条记录。

如果事件只存在于 `delivery_log`，例如合成事件或历史记录缺少 `events` 行，应沿用 `missed_events_tool.rs` 的容错思路，返回 degraded record，而不是 500。

### 2. 增加稳定 reason code 映射

当前 `delivery_log.status` 已经能表达关键分支，但用户解释分散在工具和日志中。建议在 `hone-event-engine` 或 `hone-tools` 共享一个小型映射：

- `sent`: 已即时送达
- `queued`: 已进入 digest
- `filtered`: 命中用户偏好过滤
- `capped`: 当日 High 上限降级
- `cooled_down`: 同类标的冷却中
- `price_capped`: 同 symbol + direction 价格推送日上限
- `price_cooled_down`: 同 symbol + direction 价格推送冷却
- `quiet_held`: 勿扰时段暂存
- `failed`: sink 发送失败
- `no_actor`: 没有匹配 actor

这份映射应该同时被 `missed_events_tool`、新 API 和前端使用，避免三处文案漂移。

### 3. 从记录生成偏好 patch

新增一个纯函数层，不直接自动改配置：

- 输入：`DeliveryDecision` + 当前 `NotificationPrefs`
- 输出：若干 `PreferencePatchSuggestion`

示例：

- 对 `filtered` 且用户主动点“以后接收类似事件”：建议从 `blocked_kinds` / `blocked_sources` 移除对应项，或降低 `min_severity`。
- 对 `sent` 且用户点“减少类似噪音”：建议加入 `blocked_sources`、把 kind 移出 `immediate_kinds`、打开 `quiet_mode`、提高价格阈值。
- 对 `capped/cooled_down` 且用户认为过度收敛：建议调整 cap/cooldown 需要进入 admin 全局配置，不在 public 端直接开放。
- 对 `queued` 且用户想即时看：建议加入 `immediate_kinds` 或降低 price threshold，但保留 cap/cooldown 保护。

public 端只允许 actor 自己的 prefs patch。admin 端可以代改任意 actor，但仍需要显式确认。

### 4. 保持数据边界

- 不改变 `ActorIdentity` / `SessionIdentity` 规则。
- 不把 company portrait 变成交易日志；只把 thesis 蒸馏结果作为 digest personalize 的解释上下文。
- 不把 public 端开放成完整管理后台；public 端只能看自己的记录与偏好建议。
- 不把 delivery decision 与 `cron_job_runs` 混写。两者可以在前端汇总展示，但底层仍保留各自事实源。

## 实施步骤

### Phase 1: 只读解释面

- 在 `EventStore` 增加按 actor/status/symbol/source/time 查询 `delivery_log` + `events` 的方法。
- 在 `hone-web-api` 增加 admin/public delivery decisions 路由。
- 抽出 reason code -> label/help text 映射，替换 `missed_events_tool.rs` 内部局部映射。
- 在管理端 `推送日志` 增加 `事件决策` tab，先只读展示。
- 在 public `/portfolio` 增加最近决策摘要和详情抽屉。

### Phase 2: 偏好调优闭环

- 新增 `PreferencePatchSuggestion` 生成函数及单元测试。
- 在详情抽屉提供“减少类似事件”“以后即时提醒”“只进 digest”“屏蔽来源”等 draft 操作。
- 复用 `PUT /api/notification-prefs` 保存，并在保存后展示“下一条事件生效”。
- 为 public 端增加 actor-scoped prefs patch API，禁止跨 actor。

### Phase 3: 验证与运营视图

- 增加 dry-run summary：基于最近 7 天同类事件估算 patch 后 sent/queued/filtered 数量变化。
- 管理端增加异常聚合：发送失败率、filtered 激增、quiet held 未 flush、no_actor 比例。
- 将关键异常接入 `task-health` 或 dashboard，只展示聚合，不打扰普通用户。

## 验证方式

- Rust 单元测试：
  - `EventStore` 查询能覆盖有事件行、只有 delivery_log 行、不同 actor 隔离、status/symbol/source/time 过滤。
  - reason code 映射覆盖所有当前 `delivery_log.status` 写入分支。
  - `PreferencePatchSuggestion` 对 sent、filtered、queued、capped、cooled_down、quiet_held 给出预期建议或明确不给建议。
- Web API 测试：
  - admin 可以查询指定 actor。
  - public 只能查询当前 session actor，不能通过 query 越权。
  - 缺少事件详情时返回 degraded record。
- 前端验证：
  - `bun run test:web` 覆盖数据转换、过滤条件和 patch draft。
  - 手工检查 admin `/notifications` 与 public `/portfolio` 在桌面和移动视口不溢出。
- 产品指标：
  - 用户使用 missed/prefs 调整后的 7 日内 notification disable 比例下降。
  - send failed 和 no_actor 可在管理端被定位到具体 channel/actor/source。
  - 用户能在 2 次点击内回答“为什么没收到这条事件”。

## 风险与取舍

- 风险：解释过细会把用户带入过度调参。取舍：默认展示人话原因，具体规则只在展开层显示。
- 风险：public 端偏好 patch 可能让用户误关重要事件。取舍：所有高影响操作需要确认，并提供恢复默认。
- 风险：dry-run summary 容易被误读为未来保证。取舍：明确标注为“基于最近历史估算”。
- 风险：delivery_log status 目前是字符串，新增状态容易漏映射。取舍：先用测试锁住现有状态，后续再考虑 enum 化。
- 不做：不改变事件分类模型、不新增外部数据源、不把公司画像 UI 改成直接编辑器、不把全局 cap/cooldown 暴露给普通 public 用户。

## 与已有提案的差异

查重范围：

- `docs/proposal/`：本轮开始时不存在既有 Markdown 提案。
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 本提案不涉及 desktop bundled 进程接管、启动锁、sidecar ownership 或组件级恢复，和 `desktop-bundled-runtime-startup-ux.md` 不重复。
- 本提案不涉及 skill frontmatter、active skill state、multi-agent runner 阶段传递或 Claude Code skill 对齐，和 `skill-runtime-multi-agent-alignment.md` 不重复。
- 本提案聚焦 `event-engine delivery_log + NotificationPrefs + missed_events + Web/public UI` 的投递决策解释与偏好调优闭环，是当前主动通知产品层的缺口，而不是底层 runner 或桌面启动治理。
