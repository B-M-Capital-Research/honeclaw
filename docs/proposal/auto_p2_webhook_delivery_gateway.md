# Proposal: Webhook Delivery Gateway for External Workflow Automation

- status: proposed
- priority: P2
- created_at: 2026-05-27 02:26 +0800
- owner: automation
- related_files:
  - `README.md`
  - `AGENTS.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/current-plan.md`
  - `docs/proposal/auto_p1_hone-cloud-api-contract.md`
  - `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
  - `docs/proposal/auto_p1_delivery_decision_loop.md`
  - `docs/proposal/auto_p1_redacted-support-bundle.md`
  - `docs/proposal/auto_p2_self-serve-billing-checkout.md`
  - `crates/hone-event-engine/src/sinks/mod.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/web_users.rs`
  - `packages/app/src/pages/public-me.tsx`
  - `packages/app/src/lib/api.ts`

## 背景与现状

Honeclaw 现在已经有三个相邻但不同的对外出口：

- 对话出口：Web、CLI、桌面、Feishu、Telegram、Discord、iMessage 都通过统一 `AgentSession`、runner、outbound 模型返回聊天结果。
- 主动通知出口：event-engine 根据 portfolio / watchlist / notification prefs / digest 策略，把市场事件投给 Feishu、Telegram、Discord、iMessage 等 sink，并在 `delivery_log` 留下 append-only 审计。
- 开发者出口：公开端已有 SMS 登录、invite user、API key、OpenAI-compatible `/api/public/v1/chat/completions`，并已有提案把 Hone Cloud API 做成稳定 contract 和 developer console。

但这些出口仍主要面向“人看消息”。如果用户想把 Hone 的高价值事件接入外部工作流，例如：

- 把 counter-thesis digest item 发到自己的 Notion / Airtable / Linear / Slack workflow；
- 把财报、SEC、价格 band 或研究交付物转入企业内部复盘队列；
- 用自写脚本把特定事件写进本地量化观察表或数据仓库；
- 让 Zapier / Make / n8n 之类的自动化平台根据 Hone 的投资信号触发后续动作；

当前只能复用聊天 API 拉取、手工复制 IM 消息、或给每个平台新增一个一等 channel。这样会把 Hone 的 channel 层越做越重，也会让用户把敏感 API key 和 workflow URL 散落在脚本里。

## 问题或机会

Hone 的核心价值正在从“回答问题”扩展到“持续观察、过滤、总结并沉淀投资判断”。这个价值如果只停留在 Web/IM 消息里，会限制高阶用户和小团队的采用：

- **外部工作流没有稳定接入点。** OpenAI-compatible chat API 适合提问，不适合订阅事件；PWA/IM 适合通知人，不适合机器消费。
- **新 channel 成本高。** 每接一个业务系统都写 sink、配置、UI 和排障逻辑，会重复 Feishu/Telegram/Discord 的运维复杂度。
- **自动化可信度不足。** 用户自写轮询脚本难以判断事件是否已投递、是否重试、是否签名、是否去重、失败后在哪里看。
- **增长入口缺失。** 高阶用户往往已经有自己的研究流程。提供 webhook 可以让 Hone 进入他们现有工具，而不是强迫所有流程回到 Hone UI。

本提案排 P2：收益明确，能扩大商业化和协作场景，但它应该依赖前置安全/治理能力逐步落地，例如 Hone Cloud API contract、operator audit、secret vault、entitlement、delivery decision loop。第一版可以先做本地优先、actor-scoped、低风险 payload，而不打断当前聊天和 IM 主线。

## 方案概述

新增 **Webhook Delivery Gateway**：一种用户或管理员配置的、签名的、可重试的 outbound delivery target，用于把 Hone 已经入库和完成路由裁决的事件投递到外部 HTTPS endpoint。

核心原则：

- webhook 是 **投递出口**，不是新的 agent runner，也不是新的聊天入口。
- webhook 默认只发送结构化摘要和 deep link，不发送完整聊天 transcript、原始 prompt、完整公司画像 Markdown 或本地文件绝对路径。
- webhook 由 actor / future workspace 拥有，遵守 notification prefs、entitlement、delivery caps、quiet/digest 策略和 kill switch。
- 每次投递都有 `event_id + delivery_id + attempt + signature + timestamp`，接收方可幂等处理。
- 管理端和 public `/me` 能看到 endpoint 状态、最近失败、签名 secret 轮换和测试投递结果。

第一版优先接 event-engine / digest 产生的事件与研究交付物引用，不把所有聊天回答都自动推 webhook。

## 用户体验变化

### 用户端

Public `/me` 增加 `Integrations` 或 `Developer` 区块：

- 创建一个 webhook endpoint，填写 HTTPS URL、事件类型、symbol/source/kind 过滤、是否只收 High、是否收 digest summary。
- 系统生成 signing secret，只显示一次；后续只展示 prefix、创建时间、最近成功、最近失败、失败原因。
- 提供 `Send test event`，用户能在自己的 endpoint 看到固定 fixture payload。
- 展示最近 10 次 delivery attempt：`delivered`、`retrying`、`failed_permanent`、`disabled_by_policy`。
- 对普通用户的默认文案是“把重要研究事件同步到你的工作流”，不暴露内部 router 细节。

### 管理端

Admin Settings 或 Users 详情页增加 Webhook 管理视图：

- 按 actor 查看 endpoints、enabled 状态、event filters、last success/failure、consecutive failure count。
- 允许管理员禁用高失败率 endpoint，或在用户请求支持时触发窄范围 test delivery。
- Notification / event-engine 日志中把 webhook 作为一个 delivery channel 展示，但不把 endpoint URL 明文显示在普通列表；只显示 host + hash/prefix。
- 诊断包可以包含 webhook delivery summary、status code、error class、delivery id，但不得包含 signing secret 或完整 URL query token。

### 桌面端

桌面 remote/bundled 模式仍复用 Web console，不新增 Tauri 原生命令。桌面用户如果使用本机自动化，可把 endpoint 指向本机安全代理，但 UI 必须提示：

- endpoint 必须是 HTTPS；本机 HTTP loopback 只允许 local deployment / developer mode；
- webhook payload 不应被当成交易执行指令；
- 外部自动化失败不会阻塞 Hone 继续把事件写入本地 store。

### 多渠道

IM 通道不直接配置 webhook。用户在 IM 中问“怎么接到我的工作流”时，agent 可以引导去 public `/me` 或 admin 设置页。后续如果落地 `delivery decision loop`，某条通知详情可显示“已同时投递到 webhook / 投递失败”。

## 技术方案

### 1. Webhook endpoint store

建议在 `memory` 或 `hone-web-api` 邻近的 SQLite store 中新增 actor-scoped 表；如果未来 workspace 落地，可把 owner 类型扩展为 `actor | workspace`。

```sql
webhook_endpoints (
  endpoint_id TEXT PRIMARY KEY,
  owner_kind TEXT NOT NULL,
  owner_key TEXT NOT NULL,
  url_encrypted_or_ref TEXT NOT NULL,
  url_host TEXT NOT NULL,
  signing_secret_ref TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  event_filters_json TEXT NOT NULL,
  max_attempts INTEGER NOT NULL,
  created_at_ts INTEGER NOT NULL,
  updated_at_ts INTEGER NOT NULL,
  last_success_at_ts INTEGER,
  last_failure_at_ts INTEGER,
  consecutive_failures INTEGER NOT NULL DEFAULT 0,
  disabled_reason TEXT
);

webhook_deliveries (
  delivery_id TEXT PRIMARY KEY,
  endpoint_id TEXT NOT NULL,
  source_event_id TEXT NOT NULL,
  payload_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  attempt_count INTEGER NOT NULL,
  next_attempt_at_ts INTEGER,
  last_attempt_at_ts INTEGER,
  last_status_code INTEGER,
  last_error_class TEXT,
  payload_hash TEXT NOT NULL,
  created_at_ts INTEGER NOT NULL,
  updated_at_ts INTEGER NOT NULL,
  UNIQUE(endpoint_id, source_event_id, payload_kind)
);
```

如果 secret vault 尚未落地，第一版至少要求 signing secret 只存 hash/加密值，不通过 GET API 回显。URL 字段也应避免在日志、support bundle 和普通列表中明文展开 query。

### 2. Payload contract

第一版定义稳定 JSON contract，避免把内部 Rust 类型直接暴露：

```json
{
  "type": "hone.event.delivered",
  "version": "2026-05-27",
  "delivery_id": "whd_...",
  "event": {
    "id": "price_band:MU:2026-05-27:up:600",
    "kind": "price_alert",
    "severity": "high",
    "title": "MU crosses +6% intraday band",
    "summary": "Short human-readable summary.",
    "symbols": ["MU"],
    "source": "fmp",
    "url": "https://...",
    "occurred_at": "2026-05-27T01:35:00Z"
  },
  "hone": {
    "actor_key_hash": "sha256:...",
    "delivery_channel": "webhook",
    "decision_status": "sent",
    "deep_link": "https://hone-claw.com/portfolio?event=...",
    "mainline_relation": "counter_or_aligned_if_available"
  }
}
```

需要刻意不包含：

- raw prompt、full assistant answer、full session transcript；
- company profile full Markdown；
- local `file://` paths；
- API keys、phone numbers、raw actor id；
- 交易指令或“自动买卖”字段。

### 3. Signing and replay protection

每次 POST 加固定 headers：

- `Hone-Delivery-Id`
- `Hone-Event-Id`
- `Hone-Timestamp`
- `Hone-Signature: v1=<hex hmac sha256(secret, timestamp + "." + raw_body)>`
- `User-Agent: Honeclaw/<version> webhook`

接收方可用 timestamp 窗口和 delivery id 幂等去重。Hone 侧也必须按 `(endpoint_id, source_event_id, payload_kind)` 去重，避免同一 digest item 因重启重复投递。

### 4. Delivery worker and retry

第一版不需要通用 workflow engine。可以新增一个轻量 worker：

- event-engine router / digest scheduler 决定某事件对某 actor 可投递后，写入 `webhook_deliveries` pending 行。
- worker 扫描 `next_attempt_at_ts <= now` 的 pending 行，构造 payload，执行 HTTP POST。
- 2xx 标记 `delivered`；408/429/5xx 进入指数退避；4xx 按错误类别决定是否 permanent fail。
- 连续失败超过阈值后 endpoint 自动 `disabled_reason=too_many_failures`，并在 public/admin UI 显示恢复动作。
- 所有 HTTP 错误复用现有 `sinks/http_error.rs` 的脱敏思路，不能把 query token 或 Authorization 泄露到日志。

### 5. API and UI

Public actor-scoped API：

- `GET /api/public/webhooks`
- `POST /api/public/webhooks`
- `PATCH /api/public/webhooks/:endpoint_id`
- `POST /api/public/webhooks/:endpoint_id/test`
- `POST /api/public/webhooks/:endpoint_id/rotate-secret`
- `GET /api/public/webhooks/:endpoint_id/deliveries`

Admin API：

- `GET /api/webhooks?actor=&status=`
- `POST /api/webhooks/:endpoint_id/test`
- `PATCH /api/webhooks/:endpoint_id/disable`
- `GET /api/webhooks/:endpoint_id/deliveries`

鉴权规则：

- public API 只能操作当前 `hone_web_session` 对应 actor；
- admin API 遵守现有 bearer，后续接 operator scopes；
- 创建 endpoint 时必须校验 URL scheme、host、长度、filter 大小和数量上限；
- SSRF 防护必须至少禁止 metadata IP、loopback、private RFC1918、Unix socket、file scheme；local developer mode 可显式放宽 loopback。

### 6. Entitlement and rollout boundary

Webhook 会把 Hone 从“消息产品”扩展成“自动化平台”，需要保守开放：

- 免费/试用用户默认 1 个 endpoint、低发送频率、只支持 High/digest summary；
- paid / owner 可放宽 endpoint 数、事件类型、attempt retention；
- 走 feature flag / rollout registry 时，kill switch 应能暂停 webhook worker，但不影响 event store 入库和普通 IM/PWA 投递；
- 不允许 webhook 触发任意 agent prompt 或写回 Hone 状态，避免形成无限自动化回路。

## 实施步骤

### Phase 1: Contract and read-only skeleton

- 定义 `WebhookEndpoint`、`WebhookDelivery`、payload contract 和 HMAC signing helper。
- 新增 store schema、CRUD、URL validation、secret generation / rotation。
- Public `/me` 只展示 endpoint list、create/disable/test 的最小 UI。
- Admin 只读列出 endpoints 和 delivery attempts。

### Phase 2: Event-engine queue integration

- 在 event-engine direct/digest 裁决后，为匹配 endpoint 写 pending delivery。
- 新增 webhook delivery worker，支持 2xx success、retryable、permanent failure、consecutive failure disable。
- `delivery_log` 或旁路 webhook delivery 表写入 `delivery_channel=webhook`，让 notification / support 页面能关联。
- 增加 `Send test event` fixture，不依赖真实市场事件。

### Phase 3: Filters, metrics, and support

- 支持 kind/source/symbol/severity/digest-only filters。
- Admin 增加 delivery failure histogram、endpoint health、retry queue depth。
- Redacted support bundle 纳入 webhook summary，不含 secret 与完整 URL。
- 接入 product events：`webhook_created`、`webhook_test_succeeded`、`webhook_delivery_failed`、`webhook_disabled_by_failures`。

### Phase 4: Developer docs and external automation examples

- Public `/me` 或 developer console 增加 payload example、signature verification snippet、retry/idempotency notes。
- 提供 n8n / Zapier / Make / Slack workflow 的示例，但不要把第三方平台 SDK 做成第一版依赖。
- 等 entitlement / billing 成熟后，把 endpoint 数和 delivery volume 接入 plan。

## 验证方式

- Rust 单元测试：
  - URL validator 拒绝 `file://`、loopback/private IP、metadata IP、超长 URL、带危险 scheme。
  - HMAC signing 对固定 fixture 产生稳定 signature，timestamp/body 变化会失败。
  - store 对 `(endpoint_id, source_event_id, payload_kind)` 幂等，不重复 pending delivery。
  - retry 状态机覆盖 2xx、4xx permanent、429/5xx retry、max attempts disable。
  - payload builder 不包含 raw actor id、phone、secret、local file path、full transcript。
- Web API 测试：
  - public 用户不能访问其他 actor endpoint。
  - rotate secret 后旧 secret 无法验证新投递。
  - test delivery 不写真实 event-engine event，也不消耗普通聊天 quota。
- Event-engine 集成测试：
  - High event 命中 actor + endpoint filter 后创建 webhook pending delivery。
  - filtered / capped / digest-only 事件是否投递符合 endpoint filter 和 notification prefs。
  - kill switch / endpoint disabled 时不发送 HTTP，但记录可解释状态。
- 前端验证：
  - `bun run test:web` 覆盖 webhook settings model、delivery status label、URL validation error 显示。
  - public `/me` 移动端可完成 create/test/disable，不泄露 signing secret。
- 手工验收：
  - 用本地 HTTPS mock endpoint 接收 test payload，验证 signature、headers、重试和 idempotency。
  - 断网或 endpoint 500 后能在 admin UI 看到 retrying，恢复后成功投递。

## 风险与取舍

- **风险：webhook 变成数据泄露出口。**  
  取舍：第一版 payload 最小化、actor-scoped、只发送摘要和 deep link；URL/secret 不回显；support bundle 脱敏；高风险内容需要后续 Data Trust / Consent / Entitlement 约束。
- **风险：SSRF 与内网探测。**  
  取舍：URL validator、DNS/IP re-check、禁止 private/metadata/loopback，local developer mode 才能显式放宽。
- **风险：外部自动化误当交易信号。**  
  取舍：payload contract 不提供 buy/sell/action 字段，文档强调 research notification，不支持交易执行 webhook。
- **风险：重试风暴和成本上升。**  
  取舍：endpoint 数量和发送频率受 plan/rollout 限制，指数退避、max attempts、自动 disable 是 Phase 2 门禁。
- **风险：和现有 channel sink 边界模糊。**  
  取舍：Webhook 只做机器可消费 HTTPS POST；人类消息仍走 IM/PWA/Web；不要为 Slack/Notion 等平台先写专用 sink。
- **不做范围：** 不做 inbound webhook，不让 webhook 触发 agent prompt，不接交易执行，不发送完整聊天记录，不在 v1 支持 OAuth app marketplace，不替代 billing provider webhook。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下所有现有提案，重点对照了 Hone Cloud API、Public PWA Notification Bridge、Delivery Decision Loop、End-User Notification Control、External MCP Workspace Gateway、Self-Serve Billing Checkout、Redacted Support Bundle、Skill Trust Marketplace、Product Rollout Kill Switch。

- 不重复 `auto_p1_hone-cloud-api-contract.md`：该提案定义客户端如何调用 Hone 进行 chat completion；本提案定义 Hone 如何向用户自有 HTTPS endpoint 主动推送事件。
- 不重复 `auto_p1_public-pwa-notification-bridge.md`：PWA bridge 把浏览器变成用户通知 target；本提案把 webhook 变成机器工作流 target，不涉及 service worker 或浏览器 push subscription。
- 不重复 `auto_p1_delivery_decision_loop.md`：delivery loop 解释为什么发或不发；本提案只新增一个 delivery channel，并把投递结果接入审计。
- 不重复 `auto_p2_external-mcp-workspace-gateway.md`：MCP gateway 让外部 agent 主动读取 Hone 资产；webhook gateway 让 Hone 主动把已裁决事件推给外部系统。
- 不重复 `auto_p2_self-serve-billing-checkout.md`：billing webhook 是支付提供商 inbound event；本提案是 Hone outbound webhook，且不处理订阅付款状态。
- 不重复 `auto_p1_redacted-support-bundle.md`：support bundle 汇总诊断证据；本提案新增 webhook runtime 和 delivery evidence，后续可被 support bundle 摘要引用。

查重结论：现有提案已覆盖 chat API、PWA 通知、通知解释、MCP 外部访问、支付 webhook、诊断包和 rollout 治理，但没有覆盖“actor-scoped outbound webhook + HMAC 签名 + delivery queue/retry + endpoint health + 外部自动化工作流”的产品/架构面。因此本主题是新的、可落地的 P2 提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/webhook-delivery-gateway.md`，并在新增 webhook store、public/admin API、event-engine worker、payload contract、secret handling、URL safety policy 或 entitlement/rollout 联动时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook、必要 decision/ADR 和 handoff/archive 索引。
