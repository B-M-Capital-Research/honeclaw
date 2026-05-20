# Proposal: Privacy-Preserving Product Event Plane for Adoption Intelligence

status: proposed
priority: P1
created_at: 2026-05-20 14:03:26 +0800
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
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `memory/src/web_auth.rs`
- `memory/src/quota.rs`
- `memory/src/llm_audit.rs`
- `memory/src/session.rs`
- `memory/src/cron_job/mod.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/state.rs`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/dashboard.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/types.ts`
- `config.example.yaml`

## 背景与现状

Honeclaw 已经具备真实产品形态：公开 Web、管理端、桌面 bundled/remote、Hone Cloud API、多渠道 IM、定时任务、company portraits、portfolio、event engine、skills、LLM audit 和 runtime heartbeat 都已经在仓库中有明确入口。

现有代码里已经有不少“运行证据”：

- `memory/src/web_auth.rs` 保存 invite user、手机号、TOS、API key、session token 和 last login。
- `memory/src/quota.rs` 记录 actor + 北京日期的成功对话与 in-flight 配额。
- `memory/src/llm_audit.rs` 记录 LLM 调用、模型、latency、token 和错误。
- `memory/src/session.rs` / `session_sqlite.rs` 保存聊天 transcript、metadata、summary 与 compaction 信息。
- `memory/src/cron_job/mod.rs` 和 Web API 的 task health / notification routes 暴露自动化任务与执行历史。
- `crates/hone-web-api/src/routes/meta.rs` 返回 backend capabilities、deployment mode、语言和 channel 状态。
- `packages/app/src/app.tsx` 已经清晰分离 public surface 与 admin surface；`dashboard.tsx`、`public-me.tsx`、`chat.tsx`、`users.tsx`、`settings.tsx` 分别承担不同用户旅程。

但这些证据主要服务鉴权、额度、排障或功能展示。Hone 还没有一层明确的、隐私优先的产品事件平面来回答：

- 用户在哪些 surface 上真正发现并使用了核心能力？
- 从 public chat 到 portfolio、company portrait、automation、API key 或 desktop 的转化路径是否顺畅？
- 哪些功能被看到但没有被采用？
- 哪些空状态、错误、权限拒绝或设置步骤导致用户停住？
- 新增一个产品入口后，是否提升了 adoption，而不是只增加了页面或按钮？

现有提案已经覆盖若干相邻问题：invite activation 关注 invite 用户是否完成里程碑，usage entitlement 关注权益和成本，response feedback 关注单条回答质量，run trace 关注一次 agent run 的排障证据，journey replay 关注 release confidence，data trust 关注导出/删除和隐私执行面，rollout registry 关注功能灰度与 kill switch。它们都需要一种共同但克制的产品事件底座，否则后续会各自临时发明埋点、计数和 dashboard。

## 问题或机会

这是 P1，因为 Hone 已经进入“多入口、多部署、多用户阶段”，但产品决策还缺少可信的 adoption intelligence。继续只看登录数、会话数、quota、LLM audit 或人工反馈，会让产品优化偏向可见噪音，而不是核心价值路径。

### 问题

1. **运行审计不等于产品采纳。**
   LLM audit 能说明模型调用是否成功，quota 能说明对话是否消耗，session 能说明发生过聊天；但它们不能说明用户是否点击过 portfolio 空态、是否看见 API key 指引、是否理解 desktop remote/bundled 差异、是否尝试创建任务后放弃。

2. **不同提案会重复实现局部指标。**
   Invite activation 需要 milestone evidence，rollout 需要 feature decision 计数，feedback 需要 answer-level outcome，entitlement 需要 usage event，data trust 需要导出/删除操作记录。如果没有统一事件 schema，这些模块会各自写 SQLite 表和前端计数，后续难以组合。

3. **缺少隐私边界会阻碍产品观测。**
   Hone 处理的是投资上下文、持仓、研究主线和聊天内容。直接引入常规第三方 analytics 或全文埋点会与本地优先、ActorIdentity 隔离、数据可导出/删除等约束冲突。没有隐私优先设计，团队会在“不观测”和“过度观测”之间摇摆。

4. **开源 self-host 与 hosted/public 需要不同观测策略。**
   本地开源用户通常不希望默认上传遥测；hosted public 服务则需要聚合指标判断激活、留存、成本和功能采用。当前没有配置和 UI 让用户、管理员、桌面模式清楚知道哪些事件被本地记录、哪些可以上传、如何关闭。

5. **产品增长问题没有稳定事实源。**
   Public `/me`、`/portfolio`、`/chat`、admin `/users`、desktop dashboard、settings invite table、Hone Cloud API 都已经存在，但无法系统回答“哪个入口带来 first value”“哪个下一步 CTA 被忽略”“哪个设置错误最常见”“哪个功能只被管理员使用而没有 end-user adoption”。

### 机会

新增 **Privacy-Preserving Product Event Plane**：一个本地优先、schema 化、可脱敏汇总、可选择上传的产品事件层。它不记录完整聊天内容，不替代运行 trace，也不替代权益账本，而是把用户旅程和功能采用转换成可组合的事实。

第一版目标是三个：

1. 给 public/admin/desktop/channel/backend 统一一个 `ProductEvent` schema。
2. 默认只写本地 SQLite，支持保留期、导出/删除、红线字段校验和管理员可见开关。
3. 提供少量高价值 dashboard：activation evidence、feature adoption、drop-off、surface health、rollout decision counts。

## 方案概述

新增一个 `ProductEventPlane`，围绕 actor、surface、feature、journey step 和 outcome 记录轻量事件。

核心对象：

- `ProductEvent`
  一次产品级事实，例如 `public.chat.first_message_sent`、`portfolio.empty_state_cta_clicked`、`settings.api_key_revealed`、`desktop.backend_mode_selected`、`cron.create_preview_opened`、`company_profile.export_completed`。

- `ProductEventSubject`
  事件主体。第一版支持 `actor`、`web_invite_user`、`anonymous_session`、`deployment`。匿名 public landing page 事件只能使用短期匿名 id，不能和手机号或 actor 强绑，除非用户登录后明确归并。

- `FeatureKey`
  与 rollout registry 可共用稳定 feature id，例如 `public_chat`, `public_portfolio`, `hone_cloud.api_completions`, `company_profiles.transfer`, `desktop.bundled_runtime`, `skills.chart_visualization`。

- `JourneyStep`
  稳定旅程阶段，例如 `landing_viewed`、`login_started`、`login_completed`、`first_chat_sent`、`first_answer_received`、`portfolio_added`、`profile_viewed`、`task_created`、`api_key_used`、`desktop_connected`。

- `Outcome`
  `started` / `completed` / `failed` / `cancelled` / `blocked` / `dismissed` / `unsupported`。

- `PrivacyClass`
  `operational` / `product_usage` / `sensitive_metadata`。第一版禁止 `message_content`、raw phone、raw API key、portfolio position detail、raw prompt、raw file path 进入 product event。

默认事件粒度应非常克制：

- 记录“用户点击了导出公司画像”，不记录导出的公司名称，除非该名称已经是当前页面公开展示且明确属于用户可导出的 actor data。
- 记录“public chat 发送成功 / 失败原因 code”，不记录 message text。
- 记录“API key 被创建 / 使用 / 401”，不记录 key。
- 记录“portfolio 空态 CTA 被点击”，不记录持仓内容。
- 记录“桌面选择 remote/bundled”，不记录远端 URL 明文，可记录 URL host hash 或 category。

## 用户体验变化

### 用户端

- Public `/me` 增加“Privacy and product diagnostics”说明：Hone 可本地记录哪些产品使用事件、保留多久、是否上传、如何导出/删除。
- Public chat、portfolio、developer/API 区块可以更精准地给出下一步，因为系统知道用户已经看过哪些入口、完成了哪些步骤、在哪一步被 blocked。
- 如果 hosted 服务启用上传汇总，用户应能看到简短说明并选择退出；退出后仍保留必要 operational logs，但停止 product adoption upload。

### 管理端

- Dashboard 增加 adoption 摘要：7 天 active actors、first-value chat、portfolio/context adoption、company portrait adoption、automation adoption、API key adoption、desktop connected。
- Settings invite table 可以显示更可靠的 last product event 和 drop-off reason，而不是只看 last login / active session count。
- `/users/:actor` 详情页显示 actor journey timeline，但只展示产品事件摘要，不暴露聊天正文。
- Rollout / kill switch 页面可以显示每个 feature 的 exposure、attempt、completion、blocked counts，帮助判断灰度是否扩大或回滚。

### 桌面端

- Desktop bundled 默认只本地记录 product events；remote mode 遵循远端 backend 的 event policy。
- 桌面 onboarding 可以用本地事件恢复 checklist 状态：runner configured、backend started、first chat、channel enabled、task created。
- 用户可以在桌面设置中关闭非必要 product diagnostics upload；关闭不影响本地日志和运行能力。

### 多渠道

- Feishu / Telegram / Discord / iMessage 不需要复杂 UI，只需在关键节点写事件：allowed/blocked ingress、explicit trigger、group pretrigger used、reply delivered、attachment rejected、scheduled push delivered/blocked。
- 多渠道事件必须保留 `ActorIdentity` / `SessionIdentity` 边界，不能把群聊成员行为错误归到个人 direct actor。
- 外部 IM 不做第三方 analytics SDK；事件由 Hone backend 本地记录。

## 技术方案

### 1. SQLite product event store

建议在 `memory` 增加 `product_events.rs`，使用 SQLite WAL，与 `llm_audit` 和 `web_auth` 类似：

```text
product_events (
  event_id TEXT PRIMARY KEY,
  occurred_at TEXT NOT NULL,
  received_at TEXT NOT NULL,
  event_name TEXT NOT NULL,
  surface TEXT NOT NULL,
  feature_key TEXT,
  journey_step TEXT,
  outcome TEXT NOT NULL,
  actor_channel TEXT,
  actor_user_id_hash TEXT,
  actor_scope_hash TEXT,
  web_user_id_hash TEXT,
  anonymous_id_hash TEXT,
  session_id_hash TEXT,
  deployment_mode TEXT,
  app_version TEXT,
  privacy_class TEXT NOT NULL,
  metadata_json TEXT NOT NULL
)
```

设计约束：

- actor/user/session 默认存 hash，除非 admin 本地查询明确需要反查并且权限足够。
- `metadata_json` 只允许枚举、数字、布尔、短 reason code、feature id、route id，不允许自由文本。
- 建立 allowlist validator：未知 metadata key 默认拒绝或写入 redacted marker。
- 默认保留 90 天，可在 config 中调整；本地用户可设为 0 关闭 product event store。

### 2. Event schema registry

在 `hone-core` 或 `memory` 定义事件 manifest：

```rust
pub struct ProductEventDefinition {
    pub name: &'static str,
    pub surface: &'static str,
    pub feature_key: Option<&'static str>,
    pub allowed_outcomes: &'static [&'static str],
    pub privacy_class: ProductEventPrivacyClass,
    pub allowed_metadata_keys: &'static [&'static str],
}
```

第一批事件建议覆盖 25 个以内：

- public auth: `public.login.started`, `public.login.completed`, `public.login.failed`
- chat: `chat.message.sent`, `chat.answer.completed`, `chat.answer.failed`, `chat.quota.blocked`
- portfolio/context: `portfolio.empty_state.viewed`, `portfolio.context.saved`, `portfolio.import.completed`
- company profiles: `company_profile.viewed`, `company_profile.export.completed`, `company_profile.import.completed`
- automation: `cron.create.started`, `cron.create.completed`, `cron.run.delivered`, `cron.run.failed`
- API: `api_key.created`, `api_key.used`, `api_key.auth_failed`, `api.completion.completed`
- desktop: `desktop.backend_mode.selected`, `desktop.sidecar.started`, `desktop.sidecar.failed`
- channels: `channel.ingress.allowed`, `channel.ingress.blocked`, `channel.outbound.delivered`
- data trust: `data_export.requested`, `data_deletion.requested`
- rollout: `feature.exposed`, `feature.blocked`

### 3. Backend ingestion API

新增内部和公开路由：

- `POST /api/product-events`
  Admin/desktop authenticated, used by admin surface and local desktop frontend.
- `POST /api/public/product-events`
  Public cookie authenticated or anonymous with rate limit, only accepts public-safe event definitions.
- Internal Rust helper `record_product_event(...)`
  Used by backend routes, channel runtimes and scheduler without HTTP loopback.

Public anonymous events must use rate limiting and coarse metadata. If no anonymous id exists, the server can set a short-lived HttpOnly or same-site cookie specifically for product diagnostics, but hosted deployments must document this in privacy copy.

### 4. Aggregation and dashboards

Add read APIs:

- `GET /api/product-events/summary?from=&to=&group_by=surface|feature|journey_step|outcome`
- `GET /api/product-events/funnel?journey=public_activation`
- `GET /api/users/{actor}/product-journey`
- `GET /api/public/me/product-events-policy`

Dashboards should use aggregates by default. Raw event list is admin-only and should hide hash fields unless needed for a single actor drill-down.

Useful first dashboards:

- Public activation funnel: login completed -> first chat -> first answer -> portfolio/context -> profile viewed -> automation/API.
- Feature adoption: exposed -> attempted -> completed -> failed/blocked by feature key.
- Desktop setup funnel: backend mode selected -> backend connected -> runner configured -> first chat -> channel enabled.
- Channel reliability product view: allowed ingress -> run completed -> outbound delivered, grouped by channel and reason code.

### 5. Privacy, config and data trust integration

Config additions:

```yaml
product_events:
  enabled: true
  local_store_enabled: true
  upload_enabled: false
  retention_days: 90
  anonymous_public_events_enabled: false
```

Rules:

- Self-host/local default: local store enabled, upload disabled.
- Hosted public default: local store enabled; upload only if this deployment has explicit policy and user-facing notice.
- Data Trust Center must include product events in inventory, export and deletion/redaction preview.
- Redacted support bundle may include aggregate product event counts, not raw events by default.
- Operator access audit should record admin changes to event policy.

### 6. Relationship to existing stores

This event plane should not replace:

- `llm_audit`: model/provider/token/request diagnostics.
- `quota` / future entitlement ledger: billing and usage decisions.
- `session`: conversation truth.
- `run trace`: execution timeline.
- `web_auth`: identity and auth state.
- `cron history`: task execution truth.

Instead, product events should link to those stores through redacted ids or reason codes when useful. Example: `chat.answer.failed` can include `error_code=quota_exhausted` and `surface=public_chat`, but the detailed failure remains in run trace / session / logs.

## 实施步骤

### Phase 1: Schema and local store

- Add product event config with conservative defaults.
- Implement SQLite store, retention cleanup, schema allowlist validation and redaction tests.
- Add backend helper to record typed events from Rust without HTTP.
- Add a minimal admin summary API and capability flag, but keep UI small.

### Phase 2: Public/admin/desktop first events

- Instrument public login, public chat send/finish/fail/quota block, portfolio empty state CTA, API key create/use/fail.
- Instrument admin invite create/reset/disable, users actor view, settings runner save, desktop backend mode selection and sidecar status transitions.
- Instrument feature exposure/blocked only for features that already have stable capability or rollout decisions.
- Add dashboard cards for adoption summary and drop-off reasons.

### Phase 3: Channel and automation events

- Add channel ingress/outbound product events with reason codes, not message text.
- Add cron create/run/delivery events and event-engine push/digest adoption counts.
- Connect product event summaries to task health and notification pages.

### Phase 4: Privacy controls and optional upload

- Add public `/me` and desktop settings copy for product diagnostics policy.
- Include product events in Data Trust Center export/deletion once that proposal is implemented.
- Add optional aggregate upload path for hosted deployments, guarded by config, policy notice and user opt-out.

## 验证方式

- Rust unit tests:
  - Unknown event name and unknown metadata keys are rejected or redacted according to policy.
  - Raw phone numbers, API keys, message text, prompt text, absolute file paths and portfolio position details are rejected by validator.
  - Retention cleanup removes old rows without touching `web_auth`, `llm_audit` or sessions.
  - Actor/session hashes are stable for aggregation but do not store raw ids in product event rows.

- Web API tests:
  - Public event endpoint accepts only public-safe event definitions and is rate limited for anonymous events.
  - Admin summary API returns aggregates and does not expose raw sensitive metadata.
  - Config disabled path makes ingestion a no-op with clear status.

- Frontend tests:
  - Public chat and `/me` event calls do not block core UX if ingestion fails.
  - Dashboard aggregation model handles empty data, partial backend capability and disabled product events.

- Manual acceptance:
  - Start local admin + public UI, perform login, chat, portfolio CTA, API key creation and desktop mode selection; verify product event summary updates without storing message text.
  - Confirm `git grep` / SQLite inspection shows no raw phone, raw API key, prompt, message body or absolute upload path in product event metadata.

## 风险与取舍

- **风险：事件层变成隐形 surveillance。**
  取舍：本地优先、upload 默认关闭、schema allowlist、metadata validator、retention 和 Data Trust integration 必须先落地；禁止接入第三方 analytics SDK 作为第一版。

- **风险：与 usage entitlement / run trace / feedback 重复。**
  取舍：product events 只记录 adoption and journey facts。成本、执行细节、回答质量、用户数据导出删除仍由各自系统负责。

- **风险：埋点过多拖慢产品开发。**
  取舍：第一批不超过 25 个事件，只覆盖关键旅程和高价值 feature adoption。任何新事件必须进入 manifest 并有明确问题要回答。

- **风险：hash 后难以排障单个用户。**
  取舍：默认 dashboard 用 aggregate；admin actor drill-down 可以通过现有用户详情页上下文查询，不在事件表裸存 raw user id。

- **风险：前端事件丢失造成指标不准。**
  取舍：事件用于产品判断，不作为账单或安全真相源；关键后端事实仍从 auth/quota/session/cron/audit 派生。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下所有现有提案，重点比对了 invite activation、usage entitlement、response feedback、user journey replay、run trace、data trust、product rollout、Hone Cloud API、shareable briefs 和 zero-config demo workspace。

- 不重复 `auto_p1_invite_activation_funnel.md`：该提案为 invite 用户计算里程碑和 next action；本提案提供更底层的产品事件 schema，可作为 activation evidence 来源，但不定义 activation 阶段。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：该提案记录权益、消耗和成本控制；本提案不做 billing decision，只记录功能曝光、尝试、完成和阻塞。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：该提案收集 answer-level 质量反馈；本提案记录产品旅程和 feature adoption，不评价回答内容质量。
- 不重复 `auto_p1_run_trace_workbench.md`：该提案串联一次 agent run 的排障时间线；本提案只保存轻量产品事件，不保存 prompt、tool result 或完整执行链路。
- 不重复 `auto_p1_user-journey-replay-lab.md`：该提案用 fixture 回放核心旅程保障 release confidence；本提案记录真实使用中的产品事件和聚合指标。
- 不重复 `auto_p1_user-data-trust-center.md`：该提案负责数据清单、导出和删除；本提案会成为其中一个数据域，并必须接受其导出/删除约束。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：该提案决定功能对谁开放和如何紧急关闭；本提案记录 feature exposure/blocked/completed 的采用结果，帮助评估 rollout。
- 不重复 `auto_p2_shareable-investment-briefs.md`：该提案关注外部分享和回流 attribution；本提案覆盖全产品 surface 的隐私优先事件平面，share attribution 可作为后续事件类型之一。
