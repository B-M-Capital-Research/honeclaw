# Proposal: Operational Notice Center for System and Account Communications

status: proposed
priority: P1
created_at: 2026-06-12 02:04:22 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposal/auto_p1_durable-web-event-inbox.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_policy-consent-ledger.md`
- `docs/proposal/auto_p1_channel-activation-proof.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `memory/src/web_auth.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/events.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-event-engine/src/router/sink.rs`
- `crates/hone-event-engine/src/store.rs`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/context/console.tsx`

## 背景与现状

Hone 现在已经有公开 Web、邀请用户、短信登录、Hone Cloud API、桌面 remote/bundled 模式、多渠道 IM、主动事件引擎、定时任务和管理端用户表。它不再只是一个本地聊天工具，而是一个需要持续运营的投资助手服务。

当前仓库已有多条和“通知”相关的技术能力：

- `memory/src/web_auth.rs` 保存 invite user、登录态、TOS 接受版本、API key prefix 和 last used，是 public 用户与运营身份的基础。
- `crates/hone-web-api/src/routes/web_users.rs` 能创建、停用、重置 invite/API key，说明管理端已经能面向单个用户做账号级操作。
- `crates/hone-web-api/src/routes/notification_prefs.rs` 允许管理员查看/修改 actor 的事件通知偏好；事件引擎运行时读取相同偏好。
- `crates/hone-web-api/src/routes/notifications.rs` 合并 cron run 和 event-engine delivery log，能排查投资事件、定时任务和主动推送是否发送。
- `crates/hone-web-api/src/state.rs` 和 `/api/events` 使用 `PushEvent` + SSE broadcast 把 Web 端主动消息推给打开的 admin/public 页面。
- `crates/hone-event-engine` 已经有 `OutboundSink`、多渠道 sink、delivery log、digest 与 direct push 的分层。
- Public SPA 现在有 `/chat`、`/me`、`/portfolio`、`/terms`、`/privacy`；Admin SPA 有 settings、users、notifications、task-health、logs 等运营入口。

这些能力解决的是“投资事件/任务结果如何送达”和“某个用户/渠道是否能运行”。但还有一个重要产品层没有出现：**Hone 运营者如何安全、可审计、可撤回地向用户发送系统级或账号级通知**。

典型通知包括：

- 服务中断、恢复、维护窗口和降级说明。
- Release 后必须提示用户升级 desktop/CLI 或重新授权渠道。
- TOS / privacy / investment risk acknowledgement 更新后的用户提醒。
- API key 泄露风险、额度策略变化、账号即将停用或 invite 到期。
- Cloud/local 迁移、数据导出完成、备份恢复、channel credential 失效。
- 针对某批 trial 用户的激活提醒，例如“你还没有配置 portfolio / notification target / company portrait”。

现在这些信息只能靠邮件、人工 IM、公告文章、release notes 或让 agent 在聊天里临时解释。它们没有一个统一的 `Notice` 对象、受众选择、预览、审批、发送、用户确认、撤回、投递审计和偏好边界。

## 问题或机会

这是 P1 级机会。它不如 P0 安全边界那样立即决定系统是否会被攻破，但会显著影响 hosted/public 产品的信任、留存、运维效率和商业化准备。越往公开服务和多渠道方向走，Hone 越需要一个“系统通知不是普通投资推送，也不是 agent 临时发言”的控制面。

当前缺口集中在六个方面：

1. **系统通知和投资通知混在产品语义之外。**  
   Event-engine notification prefs 适合控制价格、财报、SEC、新闻等投资事件；它不适合表达“服务维护”“协议更新”“API key 即将失效”“桌面版本过旧”这类运营消息。把这些硬塞进事件引擎会污染投资通知语义。

2. **运营触达缺少确认与撤回。**  
   重要公告需要知道谁看过、谁未确认、是否要再次提醒、错误文案能否撤回或替换。当前 SSE broadcast 和 delivery log 更偏实时事件，不提供 notice 版本、ack、supersede 或 recall 语义。

3. **多渠道触达没有统一边界。**  
   用户可能在 Web、desktop、Feishu、Telegram、Discord 或 Hone Cloud API 使用 Hone。系统公告应该有可控目标：只发 Web inbox、只在 desktop 显示、只向已验证 channel target 发送、或只对受影响 user/API key 发送。当前没有这样的受众编排。

4. **管理员缺少低风险运营工具。**  
   Settings invite table 能停用账号和重置 key，但无法给同一批用户发送一条受控提醒。运营人员若手工复制消息到 IM 或 chat，容易越权、重复、泄露、发错人，也很难审计。

5. **用户端缺少“系统消息收件箱”。**  
   Public `/me` 适合承接账号、API key、额度、协议和数据导出状态；桌面 dashboard 适合承接本地运行状态和升级提醒。但目前用户只能从 toast、聊天、release notes 或外部渠道感知这些消息，容易遗漏关键操作。

6. **合规与信任边界不清。**  
   投资助手不能把运营公告伪装成投资建议，也不能让 agent 自由生成营销/系统通知。所有运营通知都应该是结构化模板、可审计发送者、固定类别、明确受众、可退订边界和必要时强制确认。

机会是新增 **Operational Notice Center**：把系统公告、账号提醒、服务状态、升级提醒、协议提示和低风险激活提醒作为独立产品面，复用现有用户、渠道、Web push、delivery log 和 notification prefs，但不进入投资事件路由，也不让 agent 自由广播。

## 方案概述

新增一个 actor/workspace 兼容的 `OperationalNotice` 控制面，第一版服务 public hosted / admin console / desktop remote，本地单用户模式可以保持关闭或只显示本地升级提醒。

核心对象：

- `OperationalNotice`
  - 通知正文的版本化对象。包含 `notice_id`、category、severity、title、body、cta、locale、status、created_by、created_at、scheduled_at、expires_at、supersedes。
- `NoticeAudience`
  - 受众选择规则。第一版支持 explicit actors、invite users、API key users、channel target readiness、plan/tag、last login window、capability affected。
- `NoticeDelivery`
  - 每个 actor/channel 的投递记录。包含 target surface、delivery status、error、sent_at、seen_at、ack_at、dismissed_at。
- `NoticeTemplate`
  - 受控模板库，防止运营者随意写出投资建议、隐私泄露或渠道不兼容文案。
- `NoticeInbox`
  - Web/desktop/public `/me` 的系统消息收件箱，与投资 notification feed 区分。

第一版建议只做四类 notice：

1. `service_status`：服务中断、恢复、维护窗口、降级。
2. `account_action`：API key reset、invite revoked/expiring、quota/plan 变化、TOS 需要确认。
3. `upgrade_required`：desktop/CLI/web client 版本或 channel credential 需要处理。
4. `activation_nudge`：低风险产品激活提醒，例如“未配置 portfolio”“未验证 channel target”。

明确不做：

- 不发送投资建议、买卖提醒或个股营销。
- 不替代 event-engine 投资事件通知。
- 不替代 release notes、roadmap、blog 或 changelog。
- 不让模型自动决定全量广播。
- 第一版不做复杂营销 campaign、A/B 文案和外部邮件服务。

## 用户体验变化

### 用户端

- Public `/me` 增加 “System notices” 区块：
  - 未读/需确认/已过期/已撤回状态。
  - 明确类别：服务状态、账号操作、升级要求、产品提醒。
  - CTA 只跳转到安全的现有页面，例如 `/settings`、`/portfolio`、`/chat`、`/terms`、API key 面板或外部 release note。
- Public `/chat` 不把系统公告混成 assistant message。若存在 P1/P0 notice，可在 composer 上方展示 banner；用户确认后记录 ack。
- Public `/portfolio` 只展示和该页面直接相关的 notice，例如 portfolio 迁移、数据导出、持仓数据源异常。
- 用户可以 dismiss 低风险提醒；涉及 TOS、credential、服务中断或安全的 notice 只能 ack，不能静默隐藏。

### 管理端

- 新增 Settings 子页或 `/notifications/notices`：
  - 创建 notice 草稿，选择模板、类别、严重程度、语言、CTA。
  - 选择受众：全部 active invite users、最近 N 天活跃用户、指定 actor、API key 用户、某渠道 target 已验证用户、某 capability 受影响用户。
  - 发送前显示受众预览和 dry-run 结果。
  - 支持 `draft -> scheduled -> sending -> sent -> superseded/recalled/expired` 状态机。
  - 查看 delivery/ack 统计，能定位未送达、未确认和发送失败原因。
- 管理员不能在自由文本里加入本地绝对路径、API key、手机号明文或投资建议。模板校验应在保存前阻断。
- 高风险 notice 需要二次确认；后续接 operator access proposal 后可加入审批流。

### 桌面端

- Desktop bundled 模式可显示本地 notice：
  - sidecar/backend 版本过旧。
  - bundled channel credential 失效。
  - 需要重新运行 `hone-cli onboard` 或打开 settings 修复配置。
- Desktop remote 模式从后端拉取当前 user/actor notices：
  - remote server 维护。
  - 当前 Hone Cloud API key 失效。
  - 远端 capability 降级，某些附件/通知能力暂不可用。
- 桌面托盘或 dashboard 可以显示未确认数量，但不弹出骚扰型营销通知。

### 多渠道

- Feishu/Telegram/Discord/iMessage 默认不接收所有运营 notice，只接收用户或管理员明确选择的类别：
  - `service_status` 和 `account_action` 可推送。
  - `activation_nudge` 默认只进 Web inbox。
  - `upgrade_required` 只推送到对应受影响 surface。
- Channel send 必须复用 channel activation proof 的 target readiness；没有验证 target 时只写 Web inbox，不猜测默认发送地。
- IM 文案使用短模板，附 Web/desktop 链接；不把长公告全文发到群聊。

## 技术方案

### 1. 新增 notice 存储

建议在 `memory` 增加 `operational_notice` 模块。local mode 使用 SQLite；cloud mode 使用 PG 表，和 `web_auth`、`notification_prefs` 的存储模式保持一致。

```text
operational_notices (
  notice_id TEXT PRIMARY KEY,
  category TEXT NOT NULL,
  severity TEXT NOT NULL,
  status TEXT NOT NULL,
  locale TEXT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  cta_json TEXT,
  template_id TEXT,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL,
  scheduled_at TEXT,
  expires_at TEXT,
  supersedes_notice_id TEXT,
  recalled_at TEXT
)

notice_audiences (
  audience_id TEXT PRIMARY KEY,
  notice_id TEXT NOT NULL,
  rule_json TEXT NOT NULL,
  estimated_count INTEGER,
  created_at TEXT NOT NULL
)

notice_deliveries (
  delivery_id TEXT PRIMARY KEY,
  notice_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  surface TEXT NOT NULL,
  target_ref TEXT,
  status TEXT NOT NULL,
  error_message TEXT,
  sent_at TEXT,
  seen_at TEXT,
  ack_at TEXT,
  dismissed_at TEXT,
  updated_at TEXT NOT NULL
)
```

`surface` 第一版建议包含：

- `web_inbox`
- `desktop_banner`
- `public_chat_banner`
- `feishu_dm`
- `telegram_dm`
- `discord_dm`
- `imessage`

### 2. 受众解析和安全预览

新增 `NoticeAudienceResolver`，输入 `rule_json`，输出 actor list 和 channel target preview。

可复用来源：

- `web_auth.list_invite_users()`：active/revoked、last login、API key prefix/last used。
- channel target directory：已验证 Feishu/Telegram/Discord/iMessage target。
- notification prefs：判断用户是否允许非关键类别进入 IM；关键 `service_status` 可绕过普通投资通知偏好但仍记录原因。
- usage/activation/future product event 数据：后续可接入，不作为第一版硬依赖。

预览必须显示：

- 命中用户数。
- 按 surface 分布。
- 将被跳过的原因：no target、revoked user、expired user、locale mismatch、category disabled。
- 样例前 20 个 actor，避免误发。

### 3. API 设计

Admin API：

- `GET /api/admin/notices`
- `POST /api/admin/notices`
- `POST /api/admin/notices/:id/preview`
- `POST /api/admin/notices/:id/schedule`
- `POST /api/admin/notices/:id/send-now`
- `POST /api/admin/notices/:id/recall`
- `GET /api/admin/notices/:id/deliveries`

Public API：

- `GET /api/public/notices`
- `POST /api/public/notices/:id/seen`
- `POST /api/public/notices/:id/ack`
- `POST /api/public/notices/:id/dismiss`

Desktop/admin current-user API 可以复用 public inbox shape，但鉴权来自当前 backend mode。

### 4. 投递路径

第一版优先保证 Web inbox durable：

1. notice 进入 `scheduled`。
2. 后台 worker 或 admin send-now 解析 audience。
3. 对每个 actor 创建 `notice_deliveries(surface=web_inbox,status=pending)`。
4. 若用户当前在线，通过现有 `PushEvent` broadcast 发 `notice_created`，前端再按 cursor 拉取。
5. 对需要 IM 的类别，调用受控 `OperationalNoticeSink`，内部复用各 channel outbound client，但禁止自由文本/附件/群发目标。
6. 写入 delivery status 和 error。

这条路径应与 `crates/hone-event-engine` 的投资事件 delivery log 分开，避免把服务公告计入投资通知统计。管理端可以在通知页面合并展示，但数据模型要明确区分 `record_source=operational_notice`。

### 5. 模板和内容校验

模板字段建议：

```json
{
  "template_id": "api_key_rotation_required",
  "category": "account_action",
  "severity": "high",
  "allowed_cta": ["open_api_settings", "contact_support"],
  "requires_ack": true,
  "surfaces": ["web_inbox", "public_chat_banner"],
  "body_params": ["key_prefix", "deadline"]
}
```

保存 notice 前做静态校验：

- 禁止出现 `buy` / `sell` / `目标价` / `建仓` / `清仓` 等投资行动词，除非 category 是未来专门的合规投资教育模板。
- 禁止手机号、API key、session token、local absolute path、raw cookie。
- 禁止外链到非 allowlist 域名，第一版只允许 hone-claw.com、GitHub release、docs 或 mailto support。
- 多语言 locale 必须有 fallback。

### 6. 兼容和迁移

- 不迁移历史 release notes、blog 或通知日志。
- `notification_prefs` 第一版只新增一个 `operational_notice` 可选偏好层，不改变投资事件 kind tags。
- `TOS_VERSION` 仍在 `public.rs` 中控制登录接受；Notice Center 只负责“提醒用户协议已更新”，真正的登录拦截仍由 auth path 执行。
- 若 durable web event inbox proposal 尚未落地，Notice Center 仍应自己持久化 `notice_deliveries`；SSE 只是实时提示，不是唯一真相源。

## 实施步骤

### Phase 1: 存储、模板和 Web inbox

- 新增 `memory::operational_notice` local SQLite 存储和基础类型。
- 定义 notice category/severity/status/template schema。
- 增加 admin 创建草稿、预览受众、send-now API。
- 增加 public `/api/public/notices` list/seen/ack/dismiss API。
- Public `/me` 显示系统消息收件箱。

### Phase 2: 管理端投递审计

- 增加 Notice Center 管理页或 Settings 子页。
- 展示 audience preview、delivery stats、ack rate、failures。
- 支持 recall/supersede。
- 将 `record_source=operational_notice` 合并到 notifications 页面筛选，但保持独立存储。

### Phase 3: Desktop 和关键 IM surface

- Desktop dashboard/banner 拉取 notices。
- 增加 service/account 类 notice 的 Feishu/Telegram/Discord direct delivery。
- 投递前检查 channel target readiness；未验证 target 自动降级到 Web inbox。
- 增加短模板渲染，避免长公告刷屏。

### Phase 4: Cloud/PG、权限和运营联动

- cloud mode 增加 PG notice 表和迁移。
- 接入 operator access/audit 后，为 high severity notice 加审批和操作者审计。
- 接入 privacy-preserving product events 或 invite activation funnel 后，允许更精细的受众规则，但仍保持 dry-run。

## 验证方式

- 单元测试：
  - notice 状态机：draft/scheduled/sending/sent/recalled/superseded/expired。
  - audience resolver：active invite、revoked invite、API key user、missing target、locale fallback。
  - template validator：阻断投资建议词、secret-like 字符串、本地绝对路径和非 allowlist URL。
- API 测试：
  - admin 创建 notice、preview、send-now 后生成 expected deliveries。
  - public actor 只能读取自己的 notices，不能 ack 其它 actor。
  - recall 后 public list 显示 recalled 或隐藏策略符合设计。
- 前端测试：
  - Public `/me` 未读、需确认、已读、已撤回状态展示。
  - Chat banner 不遮挡 composer，不把 notice 写入 session transcript。
  - Admin preview 在 0 受众、部分 skipped、全部可投递时都有清晰状态。
- 回归脚本：
  - 构造两个 invite users、一个有 API key、一个 revoked，发送 `account_action` notice，确认 only active user 收到。
  - 构造 missing channel target 的 IM notice，确认降级到 Web inbox 且记录 skip reason。
- 指标：
  - 发送成功率、ack rate、dismiss rate、recall count、delivery failure count。
  - 高频 notice 创建被限流，避免运营误用。

## 风险与取舍

- 风险：运营 notice 变成骚扰用户的营销工具。  
  取舍：第一版限定模板和类别；activation nudges 默认只进 Web inbox；不做营销 campaign/A-B。

- 风险：绕过用户通知偏好。  
  取舍：服务状态、安全、账号动作可高优先级送达，但必须记录 bypass reason；普通产品提醒尊重偏好和 surface 选择。

- 风险：系统公告和投资建议边界变模糊。  
  取舍：内容校验禁止投资行动词；Notice Center 不允许 agent 自动广播，不进入 company profile 或 investment thread。

- 风险：多渠道群发误发。  
  取舍：第一版只支持已验证 direct target；群聊默认不发运营 notice，除非后续 workspace/team admin 语义明确。

- 风险：新增一套通知系统造成复杂度。  
  取舍：只为 operational/system/account 类消息建立独立模型；投资事件继续走 event-engine；Web 实时提示复用 PushEvent，但 durable 真相在 notice_deliveries。

- 风险：受众规则错误导致大范围误发。  
  取舍：必须 dry-run preview，高 severity 二次确认；后续接 operator approval。

## 与已有提案的差异

- 与 `auto_p1_end-user-notification-control.md` 不重复：该提案面向投资事件通知偏好、quiet hours 和渠道选择；本提案面向服务状态、账号动作、升级和系统公告，不改变投资事件 kind tags。
- 与 `auto_p1_durable-web-event-inbox.md` 不重复：durable inbox 解决 Web SSE 断线补拉和事件持久化；本提案定义运营 notice 的创建、受众、模板、确认、撤回和审计。即使 durable inbox 未落地，notice 也需要自己的 delivery truth。
- 与 `auto_p1_product-rollout-kill-switch.md` 不重复：rollout/kill-switch 控制功能是否启用；本提案负责把影响用户的系统变化通知出去，并记录用户是否看到或确认。
- 与 `auto_p1_update-compatibility-center.md` 不重复：update compatibility 解释版本/安装兼容状态；本提案只发送升级/兼容相关 notice，不替代版本检测和 release provenance。
- 与 `auto_p1_policy-consent-ledger.md` 不重复：policy consent 是用户接受协议/风险条款的真相源；Notice Center 只负责提醒和追踪阅读/确认，不决定登录是否放行。
- 与 `auto_p1_channel-activation-proof.md` 不重复：channel activation proof 验证一个 target 能收到测试消息；Notice Center 复用该结果，避免把公告发到未验证目标。
- 与 `auto_p1_privacy-preserving-product-events.md` 不重复：product events 用于采集行为指标；Notice Center 是面向用户的可见消息和 delivery/ack 状态。

查重结论：现有 proposal 已覆盖投资通知偏好、Web event durability、发布兼容、rollout、协议接受、渠道验证和产品事件采集，但没有覆盖“运营者如何受控创建系统/账号公告、选择受众、预览、发送、撤回并让用户确认”的产品和架构控制面。因此本主题是新的、可执行的 P1 提案。
