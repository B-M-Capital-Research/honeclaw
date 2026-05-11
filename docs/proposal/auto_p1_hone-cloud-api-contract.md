# Proposal: Hone Cloud API Contract and Developer Console

status: proposed
priority: P1
created_at: 2026-05-11 17:03:16 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p2_locale-content-contract.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/types.rs`
- `crates/hone-channels/src/runners/hone_cloud.rs`
- `memory/src/web_auth.rs`
- `memory/src/quota.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/types.ts`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/lib/admin-content/settings.ts`

## 背景与现状

Hone 已经不只是本地 Web / 桌面 / IM 聊天入口。仓库里已经有一条早期的 Hone Cloud API 链路：

- Public Web 端和 OpenAI-compatible API 都在公开端端口下服务，`routes/mod.rs` 暴露 `/api/public/v1/chat/completions`。
- `public.rs` 的 `handle_openai_chat_completions` 支持 Bearer API key、`stream=true/false`、OpenAI 风格 `choices[].message.content` 或 SSE chunk。
- API key 来源于 Web invite 用户，`web_users.rs` 可以为 invite 用户生成、重置、停用 API key，并在 settings invite table 展示 prefix、last used、额度和登录状态。
- `memory/src/web_auth.rs` 保存 public user、password hash、session token hash、API key hash/prefix/last used。
- `hone_cloud` runner 会把本地桌面或 CLI 的请求转成 OpenAI-compatible 请求，调用 `https://hone-claw.com/api/public/v1/chat/completions` 或自定义 base URL。
- Public roadmap 里已经把 `Open API for developers` 放进 long horizon，Settings 里也把 Hone Cloud 作为一个可选 runner。

这些能力说明 Hone 已经具备“可被其它客户端调用”的雏形。但当前 API 仍更像一个内部兼容入口，而不是一个可以承载商业化、开发者集成、桌面远端模式和第三方自动化的产品契约。

几个具体信号：

- `OpenAiChatCompletionRequest` 只抽取最后一条 `user` message；历史 `system` / `assistant` / 多轮 message 主要依赖 Hone 自己的 session restore，而不是 API request 本身的完整上下文契约。
- OpenAI content array 只拼接 `text` 字段；`image_url`、tool call、response format、metadata、idempotency、client request id 等开发者常见字段没有明确支持或拒绝说明。
- 非流式响应没有返回 `usage`、稳定 error code、request id、quota remaining、capability tags；流式错误只用 `finish_reason=error`，客户端很难机器判断应重试、降级还是提示用户联系管理员。
- API key 目前主要由管理员生成和复制；Public `/me` 展示额度，但没有开发者自助文档、示例代码、key rotation 记录、调用日志或最后失败原因。
- `hone_cloud` runner 把远端错误直接包装为文本。桌面用户知道“HTTP 403/502”，但不知道是 key revoked、额度耗尽、server runtime blocked、request unsupported，还是投资安全门禁拒绝。

## 问题或机会

### 问题

1. API 可用但契约不够稳定。
   对开发者来说，“兼容 OpenAI chat/completions”不仅是路径和响应形状，还包括错误语义、stream 事件、usage 字段、模型/capability 发现、输入边界、超时、重试、幂等和版本演进。当前缺少一份可测试的协议契约，未来每次修改 public chat 都可能无意破坏 API 客户端。

2. API key 是运营对象，不是用户自助对象。
   管理端能创建和重置 key，但 public 用户拿到 key 后缺少自助查看：如何调用、最后一次调用是否成功、今天还剩多少、哪些错误来自额度/鉴权/服务端、何时该 rotate。对付费和留存而言，用户把 Hone 嵌入自己的工作流后，API key 就是核心入口，不应只藏在管理员 settings table。

3. 桌面远端模式缺少可诊断的客户端协议。
   `hone_cloud` runner 是 Hone 自己的远端客户端，但它和第三方开发者一样需要稳定错误与能力发现。当前 runner 只能解析 `choices[0].message.content`，没有能力判断远端是否支持附件、公司画像写入、长上下文、streaming、quota header、source provenance 或安全拒绝。

4. 商业化计量缺少 API 维度。
   `quota.rs` 能记录每日成功/in-flight，`web_users.rs` 能展示剩余额度，未来 usage entitlement ledger 会扩展用量台账。但 API 客户端仍需要即时反馈：本次消耗了什么、剩余额度多少、何时 reset、请求是否算入额度、stream 中断是否计费。

5. 安全与体验边界不清。
   投资输出安全门禁、public auth、operator audit、data trust center 都会影响 API 行为。如果 API 没有稳定的拒绝 code 和用户态解释，客户端只能把安全拒绝、非金融问题拒绝、额度耗尽、runtime 不可用都显示成普通 500/502，降低信任。

### 机会

Hone 的定位很适合把 API 做成“投资研究 agent backend”而不是普通聊天补丁：

- 个人用户可以把 Hone Cloud 接到桌面端、Raycast、Obsidian、Notion、Slack workflow 或自写脚本。
- 运营可以用 API 使用情况判断哪些 invite 用户真正进入高留存工作流。
- 商业化可以围绕 API key、调用量、附件、定时任务、数据源和团队 workspace 形成清晰套餐。
- 开源仓库可以保留本地自托管能力，同时让 hosted API 成为低摩擦试用与增长入口。

因此本提案建议把 Hone Cloud API 提升为一等产品面：**稳定 API contract + public developer console + admin observability + desktop/client capability negotiation**。

## 方案概述

新增一个围绕 public API 的产品/架构层，不重写 agent runtime，也不立刻做完整 SDK。第一版目标是让 API 客户端知道四件事：

1. 我能发什么请求。
2. 服务端支持什么能力。
3. 失败时应该怎么处理。
4. 这次调用消耗了多少、是否进入 Hone 的长期 session/资产。

核心对象：

- `ApiContractVersion`
  当前 public API 协议版本，例如 `2026-05-01` 或 `v1`，暴露在 `/api/public/v1/capabilities` 和响应 header。

- `ApiCapabilityDescriptor`
  描述当前账号/服务端支持的能力：`chat`, `stream`, `attachments`, `image_input`, `company_profiles`, `scheduled_tasks`, `portfolio_context`, `source_provenance`, `safety_gate`, `quota_headers`。

- `ApiErrorEnvelope`
  稳定错误结构：`code`, `message`, `retryable`, `category`, `request_id`, `docs_url`, `details`。中文/英文文案可由 locale layer 处理，但 code 必须稳定。

- `ApiUsageSnapshot`
  每次调用返回轻量 usage：`quota_limit`, `quota_remaining`, `quota_reset_at`, `counted`, `input_tokens?`, `output_tokens?`, `attachments_count?`, `run_id?`。

- `Developer Console`
  Public `/me` 或新 `/developer` 页面：展示 API endpoint、key prefix、last used、last error、remaining quota、复制 curl/JS/Python 示例、rotate key、查看最近调用摘要。

## 用户体验变化

### 用户端

- Public `/me` 增加 Developer 区块：
  - endpoint: `/api/public/v1/chat/completions`
  - key prefix、created_at、last_used_at、last_error_at。
  - 今日剩余额度、reset 时间、最近 10 次 API 调用状态。
  - 一键复制 curl / Node / Python 示例。
  - 自助 rotate API key，要求二次确认并显示旧 key 立即失效。
- Public `/chat` 保持简单，不把开发者概念强塞给普通用户；只有检测到用户已有 API key 或点击 “API access” 时展示。
- API 错误对终端用户更清晰：额度耗尽、key revoked、runtime blocked、unsupported field、安全拒绝分别展示不同说明。

### 管理端

- Settings invite table 保持创建 invite/key 的能力，但新增 API health 列：
  - `last success`
  - `last failure code`
  - `7d calls`
  - `quota blocked count`
- 用户详情页或未来 usage 页面可按 actor 查看 API 调用摘要，帮助判断一个用户是 Web 试用、API 集成、桌面远端，还是完全未激活。
- 管理员可以复制“发给用户的接入说明”，但无需每次手写 endpoint、header、model 示例。

### 桌面端

- Hone Cloud runner 启动或连接测试时先请求 capabilities：
  - 如果 key 缺失/无效，给出 `api_key_invalid`。
  - 如果 server runtime 不 ready，给出 `server_runtime_blocked` 和 next action。
  - 如果当前服务不支持附件或图片输入，桌面端隐藏/降级相关入口。
- 远端错误不再只显示 HTTP body，而是显示 code、request_id 和用户可执行动作。

### 多渠道

- 多渠道本身不直接暴露开发者控制台，但当用户通过 IM 问“怎么接 API”时，agent 可以引用同一份 contract/doc snippet，而不是临时编写。
- 如果 channel runtime 使用 Hone Cloud runner，Feishu/Telegram/Discord/iMessage 的失败提示可以基于 API error code 做短文案映射，例如“远端额度已用完”或“服务端运行能力未就绪”。

## 技术方案

### 1. 定义 public API contract 模块

在 `crates/hone-web-api` 增加 public API contract 类型，供 route 和测试复用：

```rust
pub struct PublicApiError {
    pub code: String,
    pub message: String,
    pub category: String,
    pub retryable: bool,
    pub request_id: String,
    pub details: serde_json::Value,
}

pub struct PublicApiCapabilities {
    pub api_version: String,
    pub account: PublicApiAccountCapabilities,
    pub server: PublicApiServerCapabilities,
    pub limits: PublicApiLimits,
}
```

第一版不需要覆盖 OpenAI 全协议，只要明确支持/拒绝：

- 支持：`model`, `messages`, `stream`, text content。
- 明确忽略或拒绝：tool calls、JSON schema response format、parallel tool calls、audio、unknown multimodal parts。
- 可选支持：content array 中的 `image_url` 先返回 `unsupported_image_input`，直到附件桥接完成。

### 2. 新增 capabilities endpoint

新增：

- `GET /api/public/v1/capabilities`
- 鉴权：Bearer API key。
- 返回：账号状态、额度、支持字段、contract version、docs links。

`hone_cloud` runner 和外部客户端都先用它做轻量握手。为避免额外延迟，runner 可缓存成功结果一段时间，并在 401/403/429/5xx 时刷新。

### 3. 标准化 chat/completions 响应和错误

对 `/api/public/v1/chat/completions`：

- 每个请求生成 `request_id`，写入 response header 和错误 body。
- 成功响应增加可选 `usage` 与 `hone` 扩展字段：

```json
{
  "id": "chatcmpl_...",
  "object": "chat.completion",
  "created": 1778490196,
  "model": "hone-cloud",
  "choices": [{ "index": 0, "message": { "role": "assistant", "content": "..." }, "finish_reason": "stop" }],
  "usage": { "prompt_tokens": null, "completion_tokens": null, "total_tokens": null },
  "hone": {
    "request_id": "req_...",
    "run_id": "run_...",
    "quota_remaining": 8,
    "quota_reset_at": "2026-05-12T00:00:00+08:00",
    "contract_version": "2026-05-01"
  }
}
```

- 错误响应统一为：

```json
{
  "error": {
    "code": "quota_exhausted",
    "message": "今日 API 调用额度已用完",
    "category": "quota",
    "retryable": false,
    "request_id": "req_...",
    "docs_url": "https://hone-claw.com/docs/api/errors#quota_exhausted"
  }
}
```

HTTP 状态建议：

- `401` missing bearer。
- `403` invalid/revoked key。
- `402` 或 `429` quota exhausted / rate limited；如果暂不使用 402，统一 429。
- `400` unsupported request field。
- `422` safety/domain refusal when request is syntactically valid but cannot be answered as an investment assistant.
- `503` runtime readiness blocked。
- `502` upstream runner failed。

### 4. 增加 API call ledger 的最小可观测层

第一版可以在 `memory/src/web_auth.rs` 旁新增轻量 SQLite 表，或作为 usage entitlement ledger 的前置最小表：

```text
public_api_calls (
  request_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  api_key_prefix TEXT,
  route TEXT NOT NULL,
  model TEXT,
  stream INTEGER NOT NULL,
  status TEXT NOT NULL,
  error_code TEXT,
  http_status INTEGER NOT NULL,
  counted_quota INTEGER NOT NULL,
  started_at_ts INTEGER NOT NULL,
  finished_at_ts INTEGER,
  latency_ms INTEGER,
  input_chars INTEGER,
  output_chars INTEGER,
  client_name TEXT,
  user_agent_hash TEXT
)
```

不要记录 raw API key、完整 prompt、完整回答或敏感 header。需要排查内容时，应通过 run trace / session / LLM audit 的权限边界跳转，而不是把 API ledger 变成第二份聊天记录。

### 5. Developer console UI

在 public `/me` 或新 `/developer` 实现：

- `GET /api/public/auth/me` 可扩展返回 API key prefix / last used / quota；如果需要 rotate，则新增 public self-service endpoint。
- `GET /api/public/v1/capabilities` 直接给页面渲染支持能力。
- `GET /api/public/v1/usage?limit=10` 返回最近调用摘要。
- `POST /api/public/v1/api-key/rotate` 自助重置 key；需要当前 web session + password 二次确认，避免只靠被盗 cookie 轮换 key。

### 6. Hone Cloud runner 兼容策略

`crates/hone-channels/src/runners/hone_cloud.rs` 保持现有 `choices[0].message.content` 解析，新增可选解析：

- `error.code` -> 映射为稳定本地错误。
- `hone.request_id` -> 写入 runner metadata 或日志。
- `hone.quota_remaining` -> 在错误提示中简短展示。
- capabilities cache -> 决定是否允许未来附件/图片输入透传。

旧服务端没有 capabilities / hone 字段时，runner 仍可按当前逻辑工作，确保自托管旧版本不会立即断。

## 实施步骤

### Phase 1: API contract and errors

- 在 `hone-web-api` 增加 public API contract 类型与 helper。
- 给 `/api/public/v1/chat/completions` 增加 request id、稳定 error envelope、基础 `hone` 扩展字段。
- 明确校验 unsupported fields，先返回 `unsupported_request_field`，避免静默忽略开发者以为已经生效的参数。
- 添加 route 单元测试，覆盖 missing bearer、invalid key、empty messages、unsupported image/tool fields、stream/non-stream success shape。

### Phase 2: Capabilities and Hone Cloud runner

- 新增 `GET /api/public/v1/capabilities`。
- `hone_cloud` runner 增加 capabilities probe 与错误 code 解析；旧服务端兼容 fallback。
- Settings 的 Hone Cloud connection test 展示 code/request_id，而不是裸 HTTP body。
- 为 `resolve_hone_cloud_chat_url` 和 capabilities URL 增加测试。

### Phase 3: Developer console MVP

- Public `/me` 增加 API access 区块：endpoint、key prefix、last used、remaining quota、示例代码。
- 增加 public self-service key rotation；需要确认并保留 admin 侧 reset 能力。
- 增加最近调用摘要 API，不记录 prompt 正文。
- 管理端 invite table 展示 last failure code / 7d calls。

### Phase 4: Usage and commercial hooks

- 与 `auto_p1_usage_entitlement_ledger.md` 对齐，把 API call ledger 双写为 usage event。
- 成功响应或 header 返回 quota remaining/reset。
- 给 public docs / roadmap 增加 API guide 链接。
- 后续再考虑 SDK、webhook、workspace-scoped API key 和 OAuth，不作为 v1。

## 验证方式

- Rust route tests:
  - Bearer 缺失返回 `401` + `error.code=missing_api_key`。
  - 无效 key 返回 `403` + `invalid_api_key`。
  - `messages=[]` 返回 `400` + `missing_user_message`。
  - content array 中只含 unsupported image part 时返回明确 `unsupported_request_field` 或 `unsupported_image_input`。
  - 非流式成功包含 `choices`、`hone.request_id`、`hone.contract_version`。
  - 流式成功以 `[DONE]` 结束，错误不泄露 API key 或内部路径。
- Frontend unit tests:
  - Public `/me` 能根据 API key prefix/last used/quota 渲染 Developer 区块。
  - Rotate key 确认流程不会在未确认时调用 API。
  - 示例代码中的 endpoint 与 capabilities 返回一致。
- Manual smoke:
  - 用 curl 调 `/api/public/v1/capabilities` 和 `/chat/completions`。
  - 用一把 revoked key 验证 403 code。
  - 桌面 Hone Cloud runner 对 missing/invalid key 显示稳定错误。
- Regression:
  - 加 `tests/regression/ci/test_public_api_contract.sh`，只依赖本地测试 server 和 fixture key，不依赖外部 Hone Cloud 账号。
- Metrics:
  - API 首次成功调用率。
  - key issued but unused 比例。
  - API error code 分布。
  - 桌面 Hone Cloud connection test 成功率。

## 风险与取舍

- 风险：过早承诺完整 OpenAI 兼容。
  取舍：文档和 capabilities 必须写清楚 v1 是 “OpenAI-compatible chat subset”，先稳定文本对话、stream、错误和额度，不承诺 tool calling / response_format / image input。

- 风险：API ledger 变成隐私扩散点。
  取舍：只记录 metadata、状态、长度、耗时、错误 code 和 request id，不记录完整 prompt/answer；内容排查走已有 session / audit 权限。

- 风险：和 usage entitlement ledger 重叠。
  取舍：本提案只做 API 产品契约和最小 call ledger；套餐、计费、跨 feature 消耗归一化仍属于 usage entitlement。

- 风险：和 runtime readiness 重叠。
  取舍：readiness 判断服务/runner 是否可用；API contract 定义客户端如何发现能力、如何理解错误、如何自助接入。

- 风险：public self-service key rotation 增加账号被盗后的破坏面。
  取舍：rotation 需要 web session + password 或二次确认；管理员仍保留停用和重置能力；operator audit 后续记录 admin 侧高危操作。

- 不做：不在 v1 提供 OAuth、多 key per user、团队 workspace token、细粒度 scopes、SDK 发布、webhook、tool calling、外部 billing provider 接入。

## 与已有提案的差异

- 不重复 `auto_p1_invite_activation_funnel.md`：该提案关注 invite 用户是否完成激活里程碑和下一步引导；本提案关注 API 协议、错误、能力发现、开发者控制台和客户端集成稳定性。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：该提案关注跨 chat / schedule / attachment / notification / token 的用量与套餐决策；本提案只定义 API 调用如何返回 quota/usage、如何记录最小调用摘要。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：该提案判断 runner、model、channel、data source 是否 ready；本提案把 readiness 结果转译为 public API capabilities 和稳定 error code。
- 不重复 `auto_p0_operator-access-audit.md`：该提案治理管理员权限、审计和高危操作；本提案治理 public user/API client 的自助接入和调用契约。
- 不重复 `auto_p1_user-data-trust-center.md`：该提案关注用户数据清单、导出、删除和隐私权利；本提案只处理 API key、调用摘要和客户端可诊断性。
- 不重复 `auto_p2_locale-content-contract.md`：该提案关注跨产品文案和 error code 本地化；本提案需要稳定 API error code，但重点是 developer-facing contract 与 capabilities。

查重结论：现有 proposal 多次提到 OpenAI-compatible endpoint、Hone Cloud API key 和 Open API roadmap，但尚未有一篇把它作为独立产品/架构面来设计。这个提案填补的是“可被外部客户端长期依赖的 hosted API 契约和开发者体验”。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`，也不归档计划页。若后续实际落地本提案，应新增或复用一份 `docs/current-plans/hone-cloud-api-contract.md`，并在改动 public API contract、错误语义、API key 自助流程或 Hone Cloud runner 行为时同步更新 `docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。
