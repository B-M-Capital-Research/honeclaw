# Proposal: Email Report Bridge for Durable Investment Briefings

status: proposed
priority: P2
created_at: 2026-05-28 08:05:57 +0800
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
- `config.example.yaml`
- `crates/hone-core/src/config/channels.rs`
- `crates/hone-channels/src/bootstrap.rs`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-event-engine/src/sinks/feishu.rs`
- `crates/hone-event-engine/src/router.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/web_auth.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `bins/hone-cli/src/main.rs`

## 背景与现状

Hone 当前已经形成多入口投资研究助理形态：README 明确列出 Web Console、Mac App、iMessage、Feishu、Telegram、Discord；`docs/repo-map.md` 也说明了 public/admin Web 分离、桌面 bundled/remote、channel sidecar、统一 `AgentSession`、event-engine、公司画像、portfolio、public chat 和 Hone Cloud API。

现有多渠道能力更偏即时对话和主动推送：

- `crates/hone-core/src/config/channels.rs` 定义 iMessage、Feishu、Telegram、Discord 的 enabled、凭证、allowlist、chat scope 和长度限制。
- `crates/hone-channels/src/bootstrap.rs` 为独立 channel binary 提供共享配置加载、process lock 和 heartbeat。
- `crates/hone-channels/src/outbound.rs` 已经抽象了 placeholder、progress、final response 和本地图片 marker 分段，适合 IM 对话式输出。
- `crates/hone-web-api/src/lib.rs` 的 `build_event_engine_sink` 把 Telegram、Discord、Feishu、iMessage 装配到 event-engine `MultiChannelSink`，未启用时回退 `LogSink`。
- `crates/hone-event-engine/src/sinks/feishu.rs` 已经处理 direct contact fallback 和 actor -> email/mobile -> open_id 的解析，但 email 只是 Feishu contact resolution 的输入，不是邮件投递渠道。
- `crates/hone-web-api/src/routes/public_digest.rs` 和 public `/portfolio` 已把用户投资主线、公司画像摘要、持仓和 mainline distill 暴露成 Web 端可读资产。
- 管理端 `/notifications` 能合并 cron job runs 与 event-engine delivery log，说明“投递结果”已经有审计入口。

这套基础能很好支持即时 chat、IM 私聊和 app 内阅读，但缺少一个投资产品里非常自然的低频渠道：邮件。对许多投资者来说，盘前/盘后摘要、周报、财报前检查表和研究结论更适合进入邮箱：可归档、可搜索、可转发、能在公司电脑和移动端稳定阅读，也不依赖用户安装或配置 IM bot。

## 问题或机会

这是 P2，而不是 P1：邮件渠道价值明确，但会引入 deliverability、unsubscribe、HTML 渲染、安全脱敏和 SMTP/API provider 配置等外部复杂度。它不应打断当前 ACP、skill runtime、event-engine、cloud storage 等主线，但值得作为可落地的产品架构提案进入机会池。

当前缺口主要有五类：

1. **低频研究报告缺少天然出口。**  
   盘前摘要、周报、财报后复盘、公司画像更新和持仓主线变化都比即时 IM 更适合邮件。当前用户要么在 Web 主动打开 `/portfolio`，要么依赖 IM 推送，缺少“每天/每周自动沉淀到 inbox”的体验。

2. **public Web 转化缺少离线留存触点。**  
   `memory/src/web_auth.rs` 已经有 public user、手机号登录、API key 和 session；`public-me.tsx` 展示账号状态。但用户离开 Web 后，Hone 没有一个不依赖 IM 绑定的默认回访渠道。邮件摘要可成为 public trial 和付费留存的低摩擦触点。

3. **报告型输出和 IM 输出混用同一心智。**  
   IM 输出应该短、即时、可点击回 Web；邮件输出则应完整、分节、可归档，并携带“查看公司画像 / 打开相关会话 / 调整通知偏好”的 return links。现有 `outbound.rs` 和 channel sinks 主要面向短消息，不适合直接复用为 HTML email。

4. **用户无法选择“即时推送 vs 汇总邮件”的打扰模型。**  
   Event-engine 已经有 digest、quiet hours、delivery log 和 notification prefs，但缺少邮件作为“低打扰汇总”目标。结果是用户要么打开 IM immediate，要么不收；中间层缺失。

5. **运营与商业化缺少可转发资产。**  
   `Shareable Investment Briefs` 提案解决外部分享；邮件桥解决的是用户自己的周期性报告和可归档收件箱。它能让高质量研究输出自然进入用户已有工作流，并为后续团队席位、专家复盘、周报订阅提供基础。

机会是新增 **Email Report Bridge**：把 Hone 的 digest、scheduled task、company portrait 和 portfolio mainline 组合成可配置、可审计、可退订的邮件报告通道。第一版只做 outbound report，不做完整 IMAP 收信机器人；回复桥只支持受控 command token 或 deep link，避免邮件线程直接变成无边界 chat。

## 方案概述

新增 `email` 作为独立的 report channel，而不是把它塞进现有 IM channel。

核心对象：

- `EmailChannelConfig`：SMTP 或 provider API 配置、from、reply-to、domain policy、enabled、max recipients、mode。
- `EmailReportTarget`：actor 或 workspace 的邮件收件配置，包含地址、验证状态、digest windows、退订状态和 report types。
- `EmailReport`：一次待发送的报告，来源可以是 event-engine digest、cron job、scheduled research、company portrait review 或 manual export。
- `EmailDeliveryRecord`：provider message id、recipient、status、bounce/complaint/unsubscribe、关联 actor、source event/task。
- `EmailReplyToken`：可选的受控回复桥 token，用于“打开 Web 继续问 / 标记有用 / 退订 / 创建复盘任务”，不是自由文本 agent 入口。

第一版原则：

- 只发送给已验证地址，不根据 Feishu allow_emails 自动启用邮件。
- 默认只发送 digest/report，不发送每条 high event immediate 邮件，避免 spam 体验。
- 邮件正文不能包含 raw session transcript、绝对本地路径、secret、完整 API key、未授权附件 URL。
- 所有邮件都必须有 unsubscribe 或 notification prefs deep link。
- 不做自动读信和自由文本邮件回复；邮件回复只处理安全 token 或引导回 Web/public chat。

## 用户体验变化

### 用户端

- Public `/me` 增加“邮件报告”设置：
  - 验证邮箱地址。
  - 选择报告类型：盘前摘要、盘后摘要、周报、财报提醒、公司画像复核。
  - 选择频率和语言。
  - 查看最近发送、退订或发送失败状态。
- Public `/portfolio` 可提供“订阅本组合周报”入口。周报包含：
  - 组合主线摘要。
  - 本周影响持仓或关注列表的关键事件。
  - 需要复核的公司画像或证据项。
  - 明确的数据时点和限制说明。
  - 回到 Web 的 deep links。
- 用户收到邮件后可以点击：
  - `查看完整画像`
  - `继续追问`
  - `标记有用/无用`
  - `调整偏好`
  - `退订`

### 管理端

- Settings 增加 Email provider readiness：provider、from domain、验证状态、bounce webhook、最近失败。
- `/notifications` 或新增 `Email` tab 展示 email delivery record，和 cron/event-engine delivery log 使用相同 actor、source、status 过滤心智。
- 用户详情页显示该 actor 的 verified email、订阅类型、最近发送和退订状态。
- 管理员可以重发一封失败报告到验证过的地址，但不能把邮件发送给未验证地址或任意输入地址。

### 桌面端

- Desktop bundled 默认不启用真实邮件 provider，避免本地用户误以为可以直接外发。
- 如果用户配置了 SMTP/provider，桌面 settings 显示本地发送模式、from 地址和测试邮件按钮。
- Remote backend 模式明确显示邮件由远端服务发送，并提供退订/删除订阅路径。

### 多渠道

- IM channel 继续承担即时问答和短提醒；邮件承担低频报告。
- IM 中可以说“把这份财报复盘发到我的邮箱”，但 agent 只创建 email report draft，并要求用户确认收件地址和内容范围。
- 群聊默认不允许把群内容邮件给个人，除非后续有 group workspace 与明确成员授权。
- 邮件里的 deep link 可以回到 Web chat，用当前 public session 或重新登录继续，不把邮件正文中的 free-form reply 直接送入 `AgentSession`。

## 技术方案

### 1. 配置与 provider 抽象

在 `crates/hone-core/src/config/channels.rs` 增加 `EmailConfig`：

```rust
pub struct EmailConfig {
    pub enabled: bool,
    pub provider: EmailProviderKind, // smtp | resend | ses | mailgun | log
    pub from_address: String,
    pub reply_to: Option<String>,
    pub max_recipients_per_batch: usize,
    pub sandbox_mode: bool,
}
```

凭证应接入未来 `Secrets Vault`；在 vault 未落地前，配置字段应保持最小且脱敏，优先支持 `log` provider 和本地 mock。不要把邮件 provider key 写入 prompt、session、support bundle 或普通 settings response。

新增 `crates/hone-integrations` 邮件 provider trait：

```rust
#[async_trait]
pub trait EmailProvider {
    async fn send(&self, message: EmailMessage) -> HoneResult<EmailProviderResult>;
    async fn verify_sender(&self) -> HoneResult<EmailProviderReadiness>;
}
```

第一阶段实现：

- `LogEmailProvider`：写 delivery record，不外发，供开发和 CI。
- `SmtpEmailProvider` 或一个 API provider adapter：只在手工配置后启用。

### 2. 邮件目标与订阅存储

在 `memory` 增加 `email_reports` 或复用 shared SQLite：

```text
email_targets (
  target_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  email_hash TEXT NOT NULL,
  email_redacted TEXT NOT NULL,
  verified_at TEXT,
  status TEXT NOT NULL,
  report_types_json TEXT NOT NULL,
  locale TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  unsubscribed_at TEXT
)

email_delivery_records (
  delivery_id TEXT PRIMARY KEY,
  target_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_id TEXT,
  subject TEXT NOT NULL,
  provider_message_id TEXT,
  status TEXT NOT NULL,
  error_code TEXT,
  error_message TEXT,
  sent_at TEXT,
  created_at TEXT NOT NULL
)
```

地址明文应尽量只在发送前短暂使用；持久层至少保存 redacted 版本和 hash。若必须保存明文收件地址，需在 `User Data Trust Center` 和未来 vault/secret 设计中明确纳入。

### 3. Report renderer

新增独立 renderer，不复用 IM 文本切分：

- 输入：`DigestPayload`、cron task result、company portrait summary、portfolio mainline context。
- 输出：`EmailMessage { subject, html, text, attachments, metadata }`。
- HTML 必须使用受控 template，不接受模型生成的任意 HTML。
- 本地图片 marker 默认不内联；图表/附件需要通过 artifact access layer 生成有权限、短 TTL 的链接，或降级为文本说明。
- 每封邮件必须包含：
  - actor/user 可识别但脱敏的报告范围。
  - 数据时点。
  - 投资风险边界。
  - unsubscribe / preferences link。
  - Web deep links。

### 4. Event-engine 与 scheduled task 接入

邮件桥应先作为 digest/report sink，不作为 high event immediate sink。

接入点：

- `build_event_engine_sink` 后续可装配 `EmailReportSink`，但第一版只消费 digest flush，不处理每条 immediate event。
- `UnifiedDigestScheduler` 在 flush 后，如果 actor 订阅 email digest，创建 `EmailReport` 并异步发送。
- `cron_job` 执行结果如果标记 `deliver_as=email_report`，先创建 draft/report，再发送到 verified target。
- `notifications.rs` 把 email delivery records 合并到现有通知审计，或新增 source=`email_report`。

### 5. Reply bridge

第一版只支持安全动作，不做 IMAP 收件：

- 邮件里每个 action link 带短期 token，token hash 存库。
- 支持动作：feedback、unsubscribe、open_chat、create_evidence_review_draft、view_profile。
- 如果用户直接回复邮件，reply-to 应指向 monitored support mailbox 或 no-reply，不能被默认 ingest 成 chat。

后续若要做 inbound email：

- 必须先有 sender verification、thread token、attachment gate、PII redaction、spam filtering 和 per-actor permission。
- 入站邮件应创建 `InvestmentDocument` 或 `CaptureItem`，而不是直接运行 agent。

## 实施步骤

### Phase 1: Log provider and subscription model

- 增加 `EmailConfig`、`LogEmailProvider`、email target / delivery record types。
- 增加 admin/public API：查看订阅、发送验证邮件、退订、列出 delivery records。
- 在 public `/me` 放置只读/配置入口，默认 disabled。
- 验证不外发时仍能生成 delivery record。

### Phase 2: Digest email renderer

- 为 event-engine digest 增加 text/html email template。
- 接入 public `/portfolio` mainline context，形成周报/盘前摘要。
- 处理 deep links、unsubscribe token、数据时点、风险边界。
- 增加 snapshot test，防止 HTML 模板泄露本地路径或缺 unsubscribe。

### Phase 3: Real provider and delivery callbacks

- 支持一个真实 provider 或 SMTP。
- 增加 sender readiness check 和测试邮件。
- 接收 bounce/complaint/unsubscribe webhook，更新 delivery record 和 target status。
- 把 provider error 映射成稳定 reason code，展示在 admin notifications。

### Phase 4: Controlled reply actions

- 邮件 action token 接入 feedback、open chat、evidence draft、preferences。
- IM 中创建 email report draft 时要求 Web 确认。
- 评估是否需要 inbound email document/capture 入口，但不默认启用自由文本 email chat。

## 验证方式

- Unit tests：
  - email address normalization/redaction/hash。
  - target verification status 和 unsubscribe 状态机。
  - provider error -> stable reason code。
  - report renderer 必含 text fallback、unsubscribe、data timestamp、risk boundary。
- Integration/regression：
  - 使用 `LogEmailProvider` 构造 actor、portfolio、mainline、digest payload，断言生成一条 delivery record 且不外发。
  - public actor 只能查看和修改自己的 email target。
  - 未验证邮箱、退订邮箱、sandbox mode 下不会真实发送。
  - notification audit 能按 source=`email_report` 查询记录。
- Frontend tests：
  - `/me` 在未配置 provider、待验证、已验证、退订、发送失败状态下显示正确 CTA。
  - admin settings provider readiness 和最近错误展示不泄露 secret。
- 手工验收：
  - 在 staging provider 下发送测试邮件，确认 HTML/text 邮件可读、移动端不溢出、所有链接有效。
  - 触发一次 digest，确认邮件与 Web `/portfolio` 主线一致，且退订后不再发送。
- 指标：
  - verified email conversion。
  - digest open/click/unsubscribe/bounce rate。
  - email report 后回到 Web chat / portfolio 的比例。
  - 因邮件导致的 support complaint 或 spam complaint。

## 风险与取舍

- **Deliverability 复杂。** 取舍：第一版支持 log/mock 和单 provider，不同时支持多个 provider；生产前必须有 sender verification、bounce/complaint 处理和 unsubscribe。
- **邮件可能泄露敏感投资数据。** 取舍：默认只发摘要，不发 shares、avg_cost、完整交易流水、raw transcript、私有附件；敏感内容用 Web deep link 承接。
- **与 IM digest 重复打扰。** 取舍：邮件是低频报告目标，默认不接 high immediate event；用户可选择 IM immediate + email daily/weekly。
- **HTML 渲染与附件链接安全面扩大。** 取舍：HTML 只走受控模板；本地文件不直接嵌入，必须通过 artifact access layer 或降级。
- **回复桥容易变成邮件机器人。** 取舍：第一版只支持 tokenized actions，不做自由文本 inbound agent。
- **P2 依赖前置安全能力。** 取舍：真实 provider 上线前应优先对齐 secrets vault、artifact access boundary、data trust center 和 notification prefs。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点全文检索了 `email`、`mail`、`digest`、`webhook`、`PWA`、`notification`、`render`、`shareable`、`inbox`、`delivery` 等关键词。

- 不重复 `auto_p1_public-pwa-notification-bridge.md`：PWA 解决移动 Web push 和回访；本提案解决邮箱里的低频可归档报告。
- 不重复 `auto_p2_webhook_delivery_gateway.md`：webhook 面向外部系统自动化；邮件面向人类阅读、归档和低频留存。
- 不重复 `auto_p1_multichannel-render-preview.md`：该提案验证现有 channel 输出渲染；本提案新增一个 report channel 和 email-specific renderer。
- 不重复 `auto_p1_end-user-notification-control.md`：通知控制中心是偏好与打扰治理；本提案是一个可被偏好治理的新增邮件报告出口。
- 不重复 `auto_p2_shareable-investment-briefs.md`：shareable briefs 面向外部分享和增长 attribution；本提案面向用户自己订阅的周期性邮件报告。
- 不重复 `auto_p1_context-return-links.md`：return links 是跨 surface 跳转契约；本提案会消费这些 links，但核心是邮件投递、模板、订阅和 deliverability。
- 不重复 `auto_p1_delivery_decision_loop.md`：delivery loop 解释为什么 sent/queued/filtered；本提案新增 source=`email_report` 的投递对象和审计记录。

本轮只创建 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。该任务属于定期产品/架构提案产出，未进入执行态，因此无需把计划落盘到 `docs/current-plans/`，也无需归档计划页。若后续开始实施，应按动态计划准入标准新增或复用 `docs/current-plans/email-report-bridge.md`，并在新增 channel config、provider adapter、delivery store、public/admin API、renderer 或退订策略后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
