# Proposal: Self-Serve Billing Checkout and Subscription Lifecycle

status: proposed
priority: P2
created_at: 2026-05-23 20:03:27 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `docs/proposal/auto_p1_policy-consent-ledger.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `memory/src/web_auth.rs`
- `memory/src/quota.rs`
- `crates/hone-web-api/src/lib.rs`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/public-content.ts`

## 背景与现状

Honeclaw 目前已经具备从 invite trial 走向商业化的几个关键前置条件：

- Public Web 有手机号 + SMS 登录、HttpOnly session、TOS、邀请码白名单和 API key，核心存储在 `memory/src/web_auth.rs`。
- Public `/chat` 已经用 `getPublicAuthMe` 返回用户、额度、历史和附件能力，`memory/src/quota.rs` 按 actor + 北京日期维护每日成功对话和 in-flight。
- Public `/me` 已经有账户信息、登录状态、登出动作和一个 membership placeholder，但还没有真实 plan、付款入口、发票/收据、续费状态或取消入口。
- 管理端 Settings 能创建/revoke invite、重置 API key、观察每日额度和 session 状态，但不能区分 trial、paid、past_due、canceled，也不能处理付款失败后的用户恢复。
- `auto_p1_usage_entitlement_ledger.md` 已经提出 plan、grant、usage event 和 entitlement decision，但明确第一版“不接 Stripe/微信支付/支付宝，不做自动扣费”。这留下了一个清晰的下一层产品问题：权益可以表达以后，用户如何自助购买、续费、取消和恢复。
- `auto_p1_invite_activation_funnel.md` 关注激活里程碑，`auto_p1_hone-cloud-api-contract.md` 关注 API key 和 developer console；二者都能产生付费意图，但当前没有把 high-intent 用户导向可审计的自助付费生命周期。

这说明 Hone 的商业化还停在“管理员授予试用资格和额度”的阶段。对一个同时提供 public chat、Hone Cloud API、桌面 remote runner、多渠道通知和长期研究资产的投资助手来说，真实付费体验不应只是在 UI 上放一个付款链接，而应成为一条可回放、可撤销、可与 entitlement 同步的 subscription lifecycle。

## 问题或机会

如果未来直接把 payment link 塞到 public 页面，会很快遇到几类问题：

- 用户端：用户不知道当前是什么 plan、何时续费、付款失败是否还保留研究资产、取消后哪些能力仍可用、API key 和 desktop remote 是否会立即停用。
- 管理端：运营只能从第三方支付后台看订单，无法在 Hone 中按 invite user / actor / future workspace 看到 subscription 状态、grant 同步、付款失败、退款、取消原因和手工补偿。
- 桌面端：remote Hone Cloud runner 依赖 public API key；如果订阅过期或付款失败，桌面端需要知道是 billing blocked、quota blocked、key revoked 还是 server runtime blocked。
- 多渠道：Telegram / Feishu / Discord 用户可能在 IM 中表现出高意图，但付费主体仍在 Web invite 或 future workspace；没有统一 billing subject 会让渠道触达、权益和付款身份脱节。
- 数据安全和信任：付款事件、退款、取消、发票信息属于敏感运营数据，不能混进普通日志、support bundle 或 agent prompt。

机会是：Hone 现有 invite user、API key、quota、public `/me`、admin Settings 和即将规划的 entitlement ledger 已经足够支撑一个保守的 **self-serve billing lifecycle**。第一版可以不绑定具体支付厂商，通过 provider adapter、webhook verifier、billing state machine 和 entitlement sync 先把产品边界做对。

优先级定为 P2：它对商业化很重要，但应排在 entitlement ledger、operator audit、policy consent 和 API contract 之后；没有这些基础，直接接支付会把状态和权限做散。

## 方案概述

新增一个 `BillingLifecycle` 产品层，负责把外部支付事件转换为 Hone 内部可审计的 subscription 状态，再同步到 entitlement grant。核心原则：

- 支付供应商只作为外部事件源；Hone 内部以自己的 `BillingCustomer`、`BillingSubscription`、`BillingEvent` 为真相投影。
- 订阅状态只影响 entitlement grant，不直接散落到 chat、API、runner、channel 或 UI 条件判断。
- 第一版只支持 self-serve checkout、customer portal、webhook ingestion、read-only invoice/receipt link 和 admin override；不做复杂税务、企业合同、团队 seat 或多币种价格实验。
- 本地开源 / desktop bundled 模式默认不接 billing；只有 public/remote/Hone Cloud deployment 打开 billing provider 配置后才显示入口。

建议状态机：

- `trialing`: invite 或促销试用中，可映射到 trial plan grant。
- `active`: 已付费或付费周期有效，可映射到 paid plan grant。
- `past_due`: 付款失败宽限期，保留数据和只读访问，可限制高成本能力。
- `canceled`: 用户主动取消，到 `current_period_end` 前保持 active，期末降级。
- `expired`: 周期结束且无有效付款，降级到 free/trial-expired grant。
- `refunded` / `disputed`: 需要管理员复核，可立即冻结 paid grant，但保留数据导出和客服入口。

## 用户体验变化

### 用户端

- Public `/me` 的 membership placeholder 升级为真实 membership panel：当前 plan、周期结束时间、下次续费、付款状态、额度摘要和管理订阅按钮。
- 当用户接近免费额度上限、API key 已使用或已完成激活里程碑时，`/chat` 和 `/me` 给出轻量升级入口，但不打断正在进行的投资研究。
- 付款成功后，用户回到 `/me?checkout=success`，看到 plan 已更新、额度恢复时间、API/desktop remote 能力是否可用。
- 付款失败或 past_due 时，用户看到明确恢复路径：更新付款方式、等待重试、联系支持；研究资产、画像和历史会话不会消失。
- 取消订阅时显示“到期后会降级什么、保留什么、如何导出数据”，避免用户误以为取消会删除研究资产。

### 管理端

- Settings 的 invite table 增加 billing status、plan、period end、last payment event、past_due count、manual grant override 标识。
- 新增 Billing 页面或 Settings 子页：按 status、plan、cohort、past_due、refund/dispute 过滤用户。
- 管理员能看到每个 subscription 同步出了哪个 entitlement grant，以及最后一次 webhook 的 `event_id`、签名校验、处理结果和幂等状态。
- 支持管理员手工补偿：延长 trial、临时授予 paid plan、冻结 suspicious account；所有操作必须进入 operator audit。

### 桌面端

- Remote Hone Cloud 模式读取 public API capabilities 或 entitlement snapshot，区分 `billing_past_due`、`quota_exhausted`、`api_key_invalid`、`server_runtime_blocked`。
- Bundled local 模式不显示付款要求，只在连接远端账号时显示 membership 状态。
- 如果订阅降级，桌面端应保留本地数据和配置，只限制远端 Hone Cloud API 或高成本 hosted capability。

### 多渠道

- IM 渠道不直接承载 checkout 流程，只能发送安全短链或提示去 Web `/me` 管理订阅。
- 对已绑定 future workspace 的用户，渠道里的额度或 billing 提示应引用 workspace subscription；绑定落地前，第一版仅支持 Web actor 付款主体。
- 定时任务和主动通知在 paid -> expired 降级时应给出一次清晰通知，之后按 entitlement decision 停止高成本任务，而不是静默失败。

## 技术方案

### 1. Billing storage

在 `memory` 新增 SQLite 存储，例如 `memory/src/billing.rs`。建议表结构：

```text
billing_customers (
  customer_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL,
  actor_channel TEXT,
  actor_user_id TEXT,
  actor_scope TEXT,
  external_customer_id TEXT,
  provider TEXT NOT NULL,
  email_or_phone_hint TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

billing_subscriptions (
  subscription_id TEXT PRIMARY KEY,
  customer_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  external_subscription_id TEXT,
  plan_id TEXT NOT NULL,
  status TEXT NOT NULL,
  current_period_start TEXT,
  current_period_end TEXT,
  cancel_at_period_end INTEGER NOT NULL,
  last_event_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

billing_events (
  event_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  external_event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  signature_valid INTEGER NOT NULL,
  idempotency_key TEXT NOT NULL,
  processing_status TEXT NOT NULL,
  customer_id TEXT,
  subscription_id TEXT,
  payload_redacted_json TEXT NOT NULL,
  error_message TEXT,
  received_at TEXT NOT NULL,
  processed_at TEXT
)
```

唯一约束：

- `(provider, external_event_id)` 唯一，保证 webhook 幂等。
- 第一版 `subject_kind=actor`，未来扩展 `workspace`。
- 不保存完整付款卡号、原始密钥、raw webhook payload 中的敏感字段；只保存 redacted payload 和必要外部 id。

### 2. Provider adapter

定义 provider-agnostic trait，避免第一版把业务逻辑绑定到某个厂商：

```rust
pub trait BillingProvider {
    fn create_checkout_session(&self, request: CheckoutRequest) -> HoneResult<CheckoutSession>;
    fn create_customer_portal_session(&self, request: PortalRequest) -> HoneResult<PortalSession>;
    fn verify_webhook(&self, headers: &HeaderMap, body: &[u8]) -> HoneResult<VerifiedBillingEvent>;
}
```

配置上只暴露：

- `billing.enabled`
- `billing.provider`
- `billing.public_base_url`
- `billing.webhook_secret`
- `billing.price_map.<plan_id>`

这些 secret 后续应接入 `auto_p0_secrets-vault-rotation.md`，在落地前至少不能进入普通 GET settings 响应或日志。

### 3. Web API

新增 public routes：

- `GET /api/public/billing/status`
- `POST /api/public/billing/checkout`
- `POST /api/public/billing/portal`
- `POST /api/public/billing/webhook/:provider`

权限约束：

- checkout / portal 必须从当前 `hone_web_session` 推导 subject，不能接受前端传入任意 actor。
- webhook 只信任 provider 签名，不依赖普通 session cookie。
- webhook 处理必须幂等；重复事件返回 200，但不重复发 grant。
- public status 只返回当前用户自己的 plan、status、period、capabilities hint，不返回支付后台外部 id。

新增 admin routes：

- `GET /api/billing/subscriptions?status=&plan=&actor=`
- `GET /api/billing/events?subscription_id=&status=`
- `POST /api/billing/subscriptions/:id/override`

admin override 必须接 operator audit；如果 operator audit 尚未落地，override 第一版只允许写入普通 admin log 并在 proposal 实施计划中标为阻塞风险。

### 4. Entitlement sync

Billing 不直接判断 chat/API/cron 能否运行，而是同步 entitlement grant：

- `subscription.active` -> create/update `EntitlementGrant(plan_id=paid, reason=billing_subscription_active, expires_at=current_period_end)`
- `trialing` -> trial grant
- `past_due` -> grace grant 或 reduced paid grant
- `canceled` with future period end -> 保持 paid grant 到期
- `expired/refunded/disputed` -> revoke paid grant and fallback to free/export-only grant

如果 `auto_p1_usage_entitlement_ledger.md` 尚未落地，可以先做一个兼容 adapter：billing status 写入 web invite metadata 或只读 API，但不得让 chat 主链路直接读取支付供应商状态。正式接入前应以 entitlement 为运行时 source of truth。

### 5. Frontend surfaces

- `packages/app/src/pages/public-me.tsx`：替换 membership placeholder，增加 plan card、checkout/portal 按钮、past_due/canceled/expired 状态文案。
- `packages/app/src/pages/chat.tsx`：仅在 quota near-limit、trial expired、API-heavy 用户场景显示低干扰升级入口。
- `packages/app/src/pages/settings.tsx`：invite table 增加 billing summary；详情面板展示 subscription/event/grant 同步。
- `packages/app/src/lib/api.ts`：新增 public/admin billing API client。
- `packages/app/src/lib/public-content.ts`：中英双语文案必须覆盖 checkout success、cancel、past_due、expired、refund/dispute、local desktop free mode 等状态。

### 6. 数据与隐私边界

- Billing 数据属于运营敏感数据，不进入 agent prompt、session transcript、company profile 或普通 support bundle。
- 用户数据导出可包含 subscription summary 和 invoice link metadata，但不包含 provider raw payload。
- 取消订阅不删除用户研究资产；删除账号或数据导出应走 User Data Trust Center 语义，而不是 billing cancellation 语义。
- 退款/争议不应自动删除历史会话和画像，只能影响未来 hosted capability 和 entitlement。

## 实施步骤

### Phase 1: Billing state skeleton

- 新增 `memory/src/billing.rs` 类型、SQLite schema、幂等事件处理测试。
- 增加 provider trait 和一个 `mock` provider，用本地测试覆盖 checkout success、active、past_due、cancel、expired。
- 新增 public `/billing/status`，未启用 billing 时返回 `disabled`，不影响当前 public `/me`。

### Phase 2: Checkout and portal

- 实现真实 provider adapter 之前，先通过配置和 mock route 验证 success/cancel redirect、session subject 绑定和 portal session 创建。
- Public `/me` 接入 membership panel，保留旧 placeholder fallback。
- Admin Settings 展示 billing summary，但不允许危险 override。

### Phase 3: Webhook to entitlement

- 接入真实 provider webhook verifier。
- 将 `billing_subscriptions` 状态同步到 entitlement grant；如果 entitlement ledger 未落地，则只记录待同步状态，不切主链路。
- 增加 webhook replay / duplicate / out-of-order 事件测试。
- `hone_cloud` runner 和 public API error envelope 增加 `billing_required` / `billing_past_due` code 映射。

### Phase 4: Operational controls

- 管理端增加 subscription list、billing event log、past_due recovery queue。
- 接入 operator audit 后开放 admin override、manual comp、refund/dispute freeze。
- 加入 product event instrumentation：checkout_started、checkout_completed、portal_opened、past_due_recovered、subscription_canceled。

### Phase 5: Workspace and team expansion

- 等 `Linked User Workspace` 或 collaborative room/seat 模型落地后，把 billing subject 从 actor 扩展到 workspace。
- 增加 seat、team member、API-only plan、advisor/team room plan，但不在第一版实现。

## 验证方式

- Rust 单元测试：
  - billing schema 初始化、幂等 webhook、out-of-order event 不回退较新的 subscription 状态。
  - checkout / portal request 只能使用当前 session 推导出的 subject。
  - active、past_due、canceled、expired、refunded、disputed 与 entitlement grant 同步规则正确。
  - disabled billing 不影响现有 invite login、public chat、API key 和 quota 行为。
- Web API 测试：
  - webhook 签名失败返回明确错误且不写 subscription。
  - 重复 webhook 返回成功但不重复创建 grant。
  - public billing status 不能查询别的 actor。
  - admin billing event list 不返回 raw provider payload 或 secret。
- 前端验证：
  - `bun run test:web` 覆盖 membership panel 的 disabled/trialing/active/past_due/canceled/expired 状态。
  - checkout success/cancel redirect 不破坏 `/me` 登录态。
  - chat near-limit upgrade CTA 不遮挡 composer，不影响 remaining=0 的既有禁用逻辑。
- 手工验收：
  - 创建 invite -> 登录 `/me` -> 启动 checkout -> 模拟 webhook active -> public status 和 admin summary 更新。
  - 模拟 past_due -> desktop/remote API 返回 billing-specific code -> 用户 portal 更新付款后恢复 active。
  - 取消订阅 -> 到期前仍可使用 paid grant -> 到期后降级但历史资产仍可查看/导出。
- 指标：
  - trial -> checkout conversion、checkout abandoned、past_due recovery、paid retention、API-heavy user conversion。
  - webhook failure rate、event processing latency、billing/entitlement mismatch count。

## 风险与取舍

- 风险：过早接支付会把产品复杂度推高。取舍：本提案排 P2，明确依赖 entitlement、consent、audit 和 API contract，第一版仅做 self-serve subscription，不做团队合同。
- 风险：支付供应商状态与 Hone entitlement 不一致。取舍：billing 只同步 grant；运行时看 entitlement，并保留 mismatch health check。
- 风险：付款失败时误伤用户研究资产。取舍：billing 只限制 hosted/high-cost capability，不删除会话、画像、portfolio 或导出权限。
- 风险：webhook 安全和幂等处理不严会导致越权升级或重复授权。取舍：签名校验、event id 唯一约束、状态机版本检查是 Phase 1/3 的门禁。
- 风险：不同地区支付、税务、发票要求差异很大。取舍：第一版只保存 receipt/invoice link metadata，不做税务计算，不承诺企业发票。
- 不做：不在本地开源模式强制付费，不做券商交易扣费，不做团队 seat，不做多币种价格实验，不把 billing event 放进 agent prompt，不让 IM 直接收集支付信息。

## 与已有提案的差异

- 不重复 `auto_p1_usage_entitlement_ledger.md`：entitlement ledger 定义权益、额度和消耗，并明确不接支付；本提案处理外部 checkout、subscription webhook、付款失败、取消、退款和 entitlement grant 同步。
- 不重复 `auto_p1_invite_activation_funnel.md`：activation funnel 判断用户是否跨过价值里程碑；本提案只在高意图用户准备付费时提供自助订阅生命周期。
- 不重复 `auto_p1_hone-cloud-api-contract.md`：API contract 稳定开发者调用、错误和 capabilities；本提案把 billing status 映射成 API/desktop 可理解的 `billing_required` / `billing_past_due` 状态，但不定义 chat/completions 协议。
- 不重复 `auto_p1_policy-consent-ledger.md`：consent ledger 记录用户对政策、数据和通知的同意；本提案只处理订阅购买、取消、退款和付款状态，不替代法律同意记录。
- 不重复 `auto_p0_operator-access-audit.md`：operator audit 记录后台人员操作；本提案依赖它审计 manual billing override，但核心是用户自助 checkout 和 provider webhook 投影。
- 不重复 `auto_p1_privacy-preserving-product-events.md`：product events 记录漏斗行为；本提案会发出 checkout 相关 product events，但不定义整体埋点体系。

查重结论：`docs/proposal/` 与 `docs/proposals/` 已覆盖 entitlement、activation、API contract、consent、operator audit、product events、public PWA 和 trust center，但没有覆盖“外部支付 checkout + subscription webhook + billing 状态机 + entitlement 同步 + 付款失败恢复”的端到端产品架构。因此本主题是新的、可落地的 P2 提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或运行配置。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/self-serve-billing-checkout.md`，并在新增 billing store、public/admin API、provider adapter、webhook security contract、entitlement 同步和前端 membership surface 时同步更新 repo map、invariants、相关 runbook 与必要 decision/ADR。
