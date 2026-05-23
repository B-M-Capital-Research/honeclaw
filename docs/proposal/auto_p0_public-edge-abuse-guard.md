# Proposal: Public Edge Abuse Guard for SMS, Chat, and Hone Cloud API

status: proposed
priority: P0
created_at: 2026-05-24 02:04:51 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `docs/proposal/auto_p1_policy-consent-ledger.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/public_auth.rs`
- `crates/hone-web-api/src/aliyun_sms.rs`
- `crates/hone-web-api/src/aliyun_captcha.rs`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-web-api/src/types.rs`
- `memory/src/web_auth.rs`
- `memory/src/quota.rs`
- `crates/hone-web-api/src/routes/chat.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/lib/admin-content/settings.ts`

## 背景与现状

Honeclaw 已经从本地投资助手扩展到公开 Web、短信登录、public `/chat`、public `/portfolio`、OpenAI-compatible `/api/public/v1/chat/completions`、Hone Cloud runner、多渠道 IM、桌面 remote backend 和管理员邀请用户体系。公开入口不再只是一个 UI 页面，而是会消耗短信、LLM token、上传存储、runner 并发、外部数据源和人工排障注意力的生产边缘。

当前代码已经有若干基础防护：

- `crates/hone-web-api/src/routes/public.rs` 的 `/api/public/auth/sms/send` 和 `/api/public/auth/sms/login` 会读取手机号、检查 invite whitelist、可选调用 Aliyun Captcha，再调用 Aliyun SMS。
- `crates/hone-web-api/src/public_auth.rs` 有 `PublicAuthLimiter`，用内存 `HashMap` 按 key 记录失败次数，10 分钟内 8 次失败后阻断 15 分钟。
- `public_client_key()` 从 `x-forwarded-for` 或 `x-real-ip` 取 IP key；SMS send/login 还会按手机号 key 做失败限制。
- `memory/src/web_auth.rs` 存储 invite user、HttpOnly session、hashed API key、API key prefix 和 last used。
- `memory/src/quota.rs` 提供 actor-scoped daily conversation quota；public chat 和 OpenAI-compatible API 进入 `AgentSession` 后会走 `AgentRunQuotaMode::UserConversation`。
- `handle_upload` 对 public chat 附件做每文件 10 MB、单次最多 4 个文件的限制，并把附件落在当前用户的 `public-uploads/<user>` 根目录下。
- `docs/invariants.md` 已要求 public auth 使用服务端 session、短信登录、captcha、config-owned credentials 和不泄露 raw token。

这些能力能挡住一部分错误输入和低频失败重试，但还不是一个公开产品所需的统一 edge abuse guard。现有防护主要是“登录失败计数”和“运行后 actor quota”，中间缺少面向登录前、短信成本、上传成本、API key 调用、并发、异常 IP、枚举、灰度封禁和运维观察的统一决策层。

## 问题或机会

这是 P0，因为它影响公开服务的核心可用性、短信和 LLM 成本、安全边界、付费转化根基以及用户信任。只要 public Web 或 Hone Cloud API 暴露在公网，滥用防线就不能只依赖 actor daily quota。

当前主要缺口：

1. **登录前流量没有持久化和全局视图。**  
   `PublicAuthLimiter` 是进程内存状态，重启后清空，多实例部署时互不共享，也不记录足够的 audit。它适合 MVP，但不能支持公开服务的 abuse 排查、灰度阈值和跨进程一致拦截。

2. **只限制失败，不限制成本型成功。**  
   SMS send 成功会 `record_success(&phone_key)` 清理失败记录；Aliyun 自身 `Interval=60` 能做 provider 级节流，但 Hone 侧缺少“每手机号每天最多 N 条、每 IP 每小时最多 N 条、每 invite user 每天最多 N 次、同 ASN/代理段异常增长”的本地策略。验证码发送成功本身就是成本。

3. **登录、上传、chat、API key 是分散策略。**  
   SMS send/login 走 `PublicAuthLimiter`，public chat 依赖会话与 actor quota，upload 只有单次大小限制，OpenAI-compatible API key 只验证 key 后进入同一 quota。它们没有共享 `request_id`、risk score、decision reason、retry_after、abuse event 或 admin 可见状态。

4. **API key 调用缺少 edge 级并发和 burst control。**  
   `/api/public/v1/chat/completions` 可被外部脚本长期调用。actor daily quota 能限制成功次数，但不能表达每分钟突发、同一 key 同时运行数、stream 长连接占用、失败重试风暴、无效 key 扫描或 key 泄露后的快速止血。

5. **上传入口缺少日预算和清理前置。**  
   每个附件 10 MB、单次 4 个文件能挡住单请求大包，但无法限制一个已登录账号一天上传数百次 10 MB 文件，也无法在 chat 未发送时及时回收 orphan upload。公开端附件会进入本地磁盘和后续 agent 上下文，属于成本和安全共同边界。

6. **手机号枚举和错误语义还没有产品化策略。**  
   当前非白名单手机号会返回“邀请制，请联系邮箱”。这对真实用户友好，但也能让攻击者区分 whitelist 命中与否。是否要统一返回、何时显示明确 invite 状态、如何记录枚举风险，需要成为可配置策略，而不是散落在 route 文案里。

7. **运维无法看到边缘阻断原因。**  
   管理端能看到用户、quota、API key prefix、任务和日志，但没有“过去 24 小时 public edge 发生了什么”：短信发送数、captcha 失败数、被限 IP、无效 key 扫描、upload 拒绝、chat burst、被临时封禁 actor、放行但高风险的请求。

## 方案概述

新增 **Public Edge Abuse Guard**，把公开入口的“是否允许继续”抽象为一个统一的边缘决策服务。它不替代现有 WebAuth、daily quota、entitlement、policy consent 或 product kill switch，而是在请求进入昂贵路径之前给出 `allow / throttle / deny / require_captcha / shadow_allow` 决策。

核心原则：

- 登录前也要有稳定防线：IP、phone hash、device cookie、path、user agent hash、API key prefix 都可作为维度，但不保存敏感原文。
- 成功请求也要计入预算：短信发送、上传、stream 连接、API key chat、captcha 失败和 auth 失败都应该进入同一 edge event ledger。
- 运行前先挡住高风险流量：LLM、SMS、文件写入、long stream、外部数据源调用都不应只依赖运行后的 actor quota。
- 用户体验要温和：普通用户看到清晰、短句、可重试时间；管理员看到结构化原因和可调策略。
- 本地开源默认轻量：single-process SQLite + conservative defaults；hosted/public 部署可切 Redis 或外部 edge gateway。

第一版建议覆盖五类入口：

1. `/api/public/auth/sms/send`
2. `/api/public/auth/sms/login`
3. `/api/public/upload`
4. `/api/public/chat`
5. `/api/public/v1/chat/completions`

## 用户体验变化

### 用户端

- 正常用户几乎无感；超过频率时看到明确提示：`请求过于频繁，请在 60 秒后重试`，而不是 provider 错误或 runner 失败。
- SMS 登录页在高风险情况下先要求 captcha，低风险白名单用户仍保持快速登录。
- public chat composer 在额度或 edge 限制触发前就禁用发送，并区分：
  - 今日对话额度用完。
  - 上传过多，请稍后再试。
  - 当前网络请求过于频繁。
  - API key 调用过快。
- 对非白名单手机号，可配置为统一提示“如果该手机号已获邀，将收到验证码”，减少枚举；邀请制运营需要明确提示时可继续保持现有文案。

### 管理端

- Settings / Users 增加 Public Edge 面板：
  - 最近 24 小时 SMS send/login 成功与拒绝。
  - 被 throttle 的 IP / phone hash / API key prefix / user id。
  - 无效 API key 尝试次数。
  - 上传拒绝、orphan upload 估算和清理建议。
  - top decision reasons：`sms_daily_limit`、`captcha_failed`、`api_key_burst`、`upload_budget_exceeded`、`invalid_api_key_scan`。
- 管理员可以对单个 invite user 或 API key 做临时封禁 / 解封，附带原因和过期时间。
- 支持 `shadow_allow` 灰度：先记录如果启用新阈值会阻断哪些请求，不直接影响用户。

### 桌面端

- Desktop remote / Hone Cloud runner 收到 API 429/403 时能显示稳定错误码：`rate_limited`、`api_key_suspended`、`edge_guard_denied`，而不是把远端限制误报成本地 runner 故障。
- Desktop bundled 本地模式默认可以关闭 public edge 面板；一旦开启 remote backend 或 public Web capability，就显示当前 edge guard 状态。

### 多渠道

- 该提案第一版不改变 Feishu/Telegram/Discord/iMessage 的入站节流，但可复用同一 Guard Service 的数据模型，为后续 channel abuse control 做准备。
- Hone Cloud runner 通过 public API 被 IM channel 调用时，API key 维度限制会保护 hosted backend，不影响本地 channel actor 的隔离语义。

## 技术方案

### 1. 新增 EdgeGuardService

在 `crates/hone-web-api` 增加边缘决策服务，初期可以放在 `public_edge_guard.rs`，后续稳定后下沉到 `memory` + `hone-core` 类型：

```rust
pub struct EdgeGuardRequest {
    pub surface: EdgeSurface,
    pub client_ip: Option<String>,
    pub phone_hash: Option<String>,
    pub user_id: Option<String>,
    pub api_key_prefix: Option<String>,
    pub session_id_hash: Option<String>,
    pub user_agent_hash: Option<String>,
    pub request_bytes: Option<u64>,
    pub stream: bool,
}

pub enum EdgeDecision {
    Allow { request_id: String },
    ShadowAllow { request_id: String, reasons: Vec<String> },
    RequireCaptcha { request_id: String, retry_after_secs: Option<u64> },
    Throttle { request_id: String, retry_after_secs: u64, reason: String },
    Deny { request_id: String, reason: String },
}
```

`surface` 建议包含：

- `sms_send`
- `sms_login`
- `public_upload`
- `public_chat`
- `public_openai_chat`
- `invalid_api_key`
- `public_file_proxy`（第二阶段）

所有 public route 在执行昂贵动作前调用 `guard.check_and_record_start()`；请求完成后调用 `guard.record_outcome()` 写成功、失败、provider error、runner timeout 或 client abort。

### 2. 持久化 edge event ledger

在 `memory` 或 `hone-web-api` runtime DB 增加 SQLite 表。第一版只保留 7 到 30 天，避免变成长期隐私仓库。

```text
public_edge_events (
  event_id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  surface TEXT NOT NULL,
  decision TEXT NOT NULL,
  reason TEXT,
  client_ip_hash TEXT,
  phone_hash TEXT,
  user_id TEXT,
  api_key_prefix TEXT,
  user_agent_hash TEXT,
  request_bytes INTEGER,
  stream INTEGER NOT NULL DEFAULT 0,
  outcome TEXT,
  retry_after_secs INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

public_edge_blocks (
  block_id TEXT PRIMARY KEY,
  scope_kind TEXT NOT NULL,
  scope_hash TEXT NOT NULL,
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT,
  created_by TEXT
)
```

隐私边界：

- IP、phone、session token、user agent 只存 hash 或 prefix，不存原文。
- API key 只存现有 `api_key_prefix`，不存 raw key 或 hash。
- 不记录 prompt、assistant response、完整附件路径或 cookie。
- 管理端只展示 hash prefix、user id、api key prefix、surface 和 reason。

### 3. 策略配置

在 `config.yaml` 增加 `public_edge_guard`：

```yaml
public_edge_guard:
  enabled: true
  mode: enforce # observe | shadow | enforce
  retention_days: 14
  sms:
    per_ip_hour: 10
    per_phone_day: 5
    require_captcha_after_failures: 2
    hide_whitelist_miss: true
  chat:
    per_user_minute: 3
    per_user_concurrent_runs: 1
    per_ip_minute_logged_in: 20
  api:
    per_key_minute: 6
    per_key_concurrent_streams: 1
    invalid_key_per_ip_hour: 20
  upload:
    per_user_day_bytes: 100mb
    per_user_day_files: 40
```

默认值应保守但不打扰单人使用。`observe` 只记录不拦截；`shadow` 返回 allow 但附内部 reason；`enforce` 执行阻断。

### 4. Route 接入点

- `handle_sms_send_code`：
  - 在 captcha/provider 调用前检查 IP + phone hash + whitelist miss budget。
  - SMS provider 成功后记录 `outcome=sms_sent`，失败记录 provider kind。
  - 对非白名单手机号根据策略决定是否返回统一文案。
- `handle_sms_login`：
  - 在 Aliyun check 前检查 IP + phone hash。
  - 错误验证码触发 `sms_login_failed` 的同时写 edge event。
  - 成功登录清理失败状态，但不清理当日成功发送预算。
- `handle_upload`：
  - 读 multipart 前做粗粒度并发/频率检查。
  - 每写入一个文件后累加 user daily bytes/files。
  - 当 chat 未使用附件时，由后续 cleanup 根据 event ledger 清理 orphan uploads。
- `handle_chat`：
  - 进入 `build_chat_sse` 前检查 per-user minute + concurrent run。
  - SSE 完成、失败、断开后 release concurrent run。
- `handle_openai_chat_completions`：
  - API key 验证失败也走 `invalid_api_key` surface。
  - 有效 key 进入 per-key burst + concurrent stream 检查。
  - 响应带稳定 `request_id`、`error.code`、`retry_after`。

### 5. 与现有系统的关系

- `PublicAuthLimiter`：第一阶段保留为 in-memory fast path，但由 EdgeGuardService 包装；第二阶段用 SQLite/Redis 替代其 source of truth。
- `ConversationQuotaStorage`：继续决定“这个 actor 今天还能成功对话几次”；edge guard 决定“这个请求是否应在消耗 runner 之前被允许进入”。
- `Usage Entitlement Ledger`：未来负责套餐、grant、token/功能消耗；edge guard 负责 abuse/rate/pre-run 风险。两者可共享 usage event，但不互相替代。
- `Hone Cloud API Contract`：复用其稳定错误码和 API ledger；本提案补齐 pre-run burst/concurrency/invalid key 风控。
- `Product Rollout Kill Switch`：用于关功能或灰度能力；edge guard 用于请求级风险决策。
- `Privacy-Preserving Product Events`：记录产品采用漏斗；edge guard 记录安全与成本防线，不记录行为细节。

## 实施步骤

### Phase 1: Observe-only ledger

- 新增 EdgeGuard 类型、SQLite 表和 retention cleanup。
- 将 SMS send/login、public upload、public chat、OpenAI-compatible API 接入 observe 模式。
- 保留当前行为，只记录 decision、reason、outcome、hashed scopes。
- 增加基础 admin JSON endpoint，先不做复杂 UI。

### Phase 2: Enforce SMS and invalid API key limits

- 用 edge ledger 替代或包裹 `PublicAuthLimiter` 的失败计数。
- 增加 per-phone-day SMS success budget、per-IP-hour SMS budget、invalid API key scan budget。
- 支持 `hide_whitelist_miss`，并把枚举风险记录成 reason。
- 单元测试覆盖 captcha required、SMS success budget、whitelist miss、provider failure 不清错预算。

### Phase 3: Chat/API burst and concurrency

- 对 public chat 和 OpenAI-compatible API 加 per-user/per-key burst 和 concurrent stream guard。
- SSE / streaming abort 必须 release concurrent slot。
- API error body 增加稳定 code、request_id 和 retry_after。
- Desktop Hone Cloud runner 和前端 chat 映射这些错误码。

### Phase 4: Upload budget and cleanup

- 增加 per-user daily upload bytes/files budget。
- 记录 upload 与后续 chat request 的关联；定期清理未绑定 chat 的 orphan uploads。
- 管理端显示 upload budget、orphan bytes、清理结果。

### Phase 5: Admin workbench and deployment adapter

- 管理端 Public Edge 面板上线：summary、top scopes、manual blocks、shadow policy impact。
- 对 hosted 部署增加 Redis adapter 或外部 edge gateway adapter，保留 SQLite 作为本地/开源默认。
- 将关键阈值纳入 `hone-cli doctor` 或 runtime readiness 检查。

## 验证方式

自动化验证：

- `PublicAuthLimiter` 现有测试保留；新增 EdgeGuard 单元测试覆盖 allow/throttle/deny/require_captcha/shadow_allow。
- SQLite ledger roundtrip、retention cleanup、hash-only 存储、不记录 raw phone/IP/API key。
- SMS send 成功也计入 per-phone-day budget；连续成功发送超过阈值返回 429。
- 非白名单手机号在 `hide_whitelist_miss=true` 时返回统一文案，但 ledger 记录 whitelist miss reason。
- 无效 API key 扫描达到阈值后，同 IP 后续 invalid key 请求被 throttle。
- public chat/API stream 并发 slot 在成功、失败、超时和 client abort 路径释放。
- upload daily bytes/files 超阈值返回 429，单文件 10 MB 限制仍先于 runner 执行。

回归与手工验收：

- `bun run test:web` 覆盖前端错误码映射：quota exhausted、rate limited、captcha required、api key suspended。
- 本地启动 public Web，验证普通登录、发送验证码、登录、chat、upload、API key chat 正常。
- 模拟 burst 请求，确认 SMS provider 和 LLM runner 未被调用，响应带 `Retry-After`。
- 管理端能看到 edge summary，且不展示原始手机号、IP、cookie、prompt 或 API key。

指标：

- SMS provider 调用数 / 成功登录数比值下降。
- invalid API key 尝试被阻断比例上升。
- public chat runner 启动前被拒绝的高风险请求可解释。
- 正常用户登录完成率不下降，误伤可通过 shadow 模式回看。

## 风险与取舍

- 风险：阈值过紧会误伤真实用户。取舍：先 observe，再 shadow，最后 enforce；所有决策带 reason 和 retry_after。
- 风险：edge ledger 存储敏感轨迹。取舍：只存 hash/prefix 和 surface，不存原始 IP、手机号、cookie、prompt、回答或附件路径，默认短保留。
- 风险：SQLite 在多实例 hosted 部署下不够。取舍：本地/开源默认 SQLite，hosted 模式预留 Redis/external adapter；策略层和存储层分离。
- 风险：和 entitlement / billing 产生概念重叠。取舍：edge guard 管滥用和请求级速率，entitlement 管套餐权益和用量归属，billing 管付款生命周期。
- 风险：统一非白名单文案会降低邀请制用户自助理解。取舍：`hide_whitelist_miss` 做成部署策略；公开大流量时启用，私域试用可保持明确提示。
- 不做边界：第一版不做 WAF、Bot 指纹、设备指纹、自动封 ASN、跨渠道 IM rate limit、完整欺诈评分，也不改动现有 daily conversation quota 的 source of truth。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，重点比对了 usage entitlement、Hone Cloud API contract、product rollout kill switch、privacy-preserving product events、policy consent ledger、secrets vault、operator access audit、redacted support bundle 和 invite activation funnel。

- 不重复 `auto_p1_usage_entitlement_ledger.md`：该提案解决套餐、grant、跨功能消耗和成本归集；本提案解决登录前与运行前的 abuse/rate/concurrency 决策，很多请求还没有 actor usage event。
- 不重复 `auto_p1_hone-cloud-api-contract.md`：该提案定义稳定 API 契约、开发者体验和最小调用 ledger；本提案补齐 API key burst、invalid key scan、concurrent stream 和 SMS/upload/public chat 的统一边缘防护。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：kill switch 控制能力是否开放；edge guard 控制单个请求是否风险过高或过频。
- 不重复 `auto_p1_privacy-preserving-product-events.md`：product events 面向采用和增长漏斗；edge events 面向安全、成本和可用性，只保留最小 hash scope。
- 不重复 `auto_p1_policy-consent-ledger.md`：consent ledger 记录用户政策同意；edge guard 不判断法律同意，只在公开入口前做风险/速率防线。
- 不重复 `auto_p0_secrets-vault-rotation.md`：secrets vault 保护长期凭证；edge guard 保护公网请求入口免受滥用和成本攻击。

本轮只新增 proposal，不开始执行实现；因此不更新 `docs/current-plan.md`，也无需归档计划页。若后续落地，应新增或复用 `docs/current-plans/public-edge-abuse-guard.md`，并在新增配置、长期安全约束、API 错误语义或 public route 行为变化时同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md` 或相关 runbook。
