# Proposal: External Egress Ledger for Third-Party Data Boundaries

status: proposed
priority: P1
created_at: 2026-05-27 08:05:44 +0800
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
- `docs/proposal/auto_p0_secrets-vault-rotation.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_policy-consent-ledger.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `crates/hone-core/src/audit.rs`
- `memory/src/llm_audit.rs`
- `crates/hone-channels/src/prompt_audit.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/runners/hone_cloud.rs`
- `crates/hone-channels/src/runners/multi_agent.rs`
- `crates/hone-llm/src/openai_compatible.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/settings.tsx`

## 背景与现状

Honeclaw 的核心价值已经不只是本地会话，而是把投资研究工作流连接到多个外部边界：LLM provider、Hone Cloud、Tavily 搜索、FMP 行情与财务数据、多渠道机器人、公开 Web API、桌面 sidecar、actor sandbox 里的长期研究资产。README 展示的产品心智是“跨 Web、Mac App、iMessage、Feishu、Telegram、Discord 的投资研究助理”，这意味着用户的敏感上下文会在多个执行链路中被读取、裁剪、发送、渲染或转发。

当前仓库已经具备几块相关基础：

- `crates/hone-core/src/audit.rs` 定义 `LlmAuditRecord`，包含 actor、session、source、provider、model、request、response、token 和 metadata。
- `memory/src/llm_audit.rs` 使用 SQLite 保存 LLM audit，支持按 actor、session、source、provider、日期、成功状态查询，并保留 request/response JSON。
- `crates/hone-channels/src/prompt_audit.rs` 会把每次运行的 system prompt 与 runtime input 写入 `data/runtime/prompt-audit/`，便于调试 prompt 组装。
- `crates/hone-llm/src/openai_compatible.rs`、`crates/hone-channels/src/runners/hone_cloud.rs`、`crates/hone-channels/src/runners/multi_agent.rs` 会把系统 prompt、历史上下文、runtime input 或搜索阶段 note 发送到外部兼容接口或 Hone Cloud。
- `crates/hone-tools/src/web_search.rs` 通过 Tavily 发送 query，并已对错误里的 token/key 做脱敏。
- `crates/hone-tools/src/data_fetch.rs` 通过 FMP 发送 ticker、data_type、日期窗口或 URL，并支持 key pool fallback。
- `crates/hone-channels/src/outbound.rs` 统一处理最终回复分段、本地图表 `file://` marker 和通道发送抽象。
- `docs/invariants.md` 已明确 public auth、LLM credentials、actor sandbox、company profile、prompt time anchoring、runtime runner 等长期约束，说明仓库对安全边界和数据隔离已经非常敏感。

这些基础更偏“调用审计”和“调试证据”。它们能回答某次 LLM 请求发了什么、某次 prompt 是如何组装的，却还不能稳定回答一个更产品化的问题：**某个用户/actor 的哪些数据在什么时候离开了 Hone 本地边界、去了哪类第三方、出于什么目的、以什么粒度记录、能否向用户解释或导出。**

投资研究场景里，这个问题比普通聊天产品更尖锐。持仓、公司画像、研究主线、上传文件、定时任务、模型调用轨迹、通知内容都可能包含用户的策略、仓位、风险偏好和商业判断。Hone 未来要做 Hone Cloud、自托管团队版、桌面 remote backend、公开 API 和多渠道主动推送时，单纯“有 LLM audit”还不足以支撑用户信任、管理员排障、数据导出、合规问询和 private/local-only 模式。

## 问题或机会

这是 P1 级提案。它不一定像 secret 泄露或投资输出安全那样立即触发 P0，但会显著影响用户信任、商业化、安全解释、企业/高净值用户转化和后续 privacy mode 能力。

主要缺口：

1. **LLM audit 只覆盖模型请求，不覆盖完整 egress。**
   Tavily query、FMP symbol/date/API path、Hone Cloud runner、公开 API 回复、多渠道 outbound、附件上传后被转交给模型的摘要、未来 webhook/PWA/邮件等，都属于数据离开本地或离开当前 surface 的边界事件。现有 LLM audit 不能统一呈现这些动作。

2. **prompt audit 保存了完整本地调试材料，但不是用户可解释的外流清单。**
   `prompt_audit` 会保存 system prompt 和 runtime input，这对研发排障有价值，但它不是一个最小披露 ledger。用户或管理员需要的是“发送给 OpenRouter 约 12KB prompt，包含 portfolio summary 和 2 个 company profile 摘要”，而不是直接暴露整段 prompt。

3. **第三方调用目的没有结构化。**
   同样是外发数据，目的可能是 `main_chat_llm`、`session_compaction`、`event_engine_classifier`、`web_search_query`、`market_data_fetch`、`hone_cloud_chat`、`channel_delivery`、`support_export`。如果没有 purpose，后续 privacy notice、data export、policy consent 和 billing 都只能靠事后推断。

4. **用户无法建立“本地/远端/第三方”的心智边界。**
   Desktop bundled、desktop remote、public Web、Hone Cloud runner、多渠道私聊/群聊的边界不同。用户需要知道哪些数据只在本机，哪些发给 Hone Cloud，哪些发给 LLM provider，哪些进入 IM 平台消息，哪些只是调用市场数据源。

5. **管理员缺少按 actor 的外流排障视图。**
   当用户问“我的公司画像有没有发给第三方模型”“为什么这个任务消耗了很多 token”“某个查询是否把附件内容发给了搜索服务”时，当前需要横跨 prompt audit、LLM audit、run trace、tool logs 和 channel logs 人工拼接。

6. **未来 private mode / local-only mode 缺少验收证据。**
   即使后续实现了 permission broker 或 local-only setting，也需要一个 ledger 来证明某段时间内没有外部 LLM、search、market-data、Hone Cloud 或 IM outbound egress。没有 ledger，private mode 很难被用户信任。

机会是：Hone 已经有 actor identity、session identity、LLM audit、prompt audit、tool registry、runner abstraction、channel outbound abstraction 和 Web admin pages。第一版可以很务实：新增一个统一的 `ExternalEgressLedger`，先记录 metadata、分类、hash、大小和目的，不保存完整 payload；再把完整 payload 继续留在现有 audit/debug store 中按权限查看。

## 方案概述

新增 **External Egress Ledger**，作为 Hone 的第三方数据边界账本。它不替代 LLM audit、prompt audit、run trace、permission broker 或 user data trust center，而是把“数据离开哪个边界”抽象成统一事件。

核心对象：

- `EgressEvent`：一次外发边界事件，例如 LLM request、Hone Cloud request、Tavily query、FMP request、channel outbound、public API response、webhook delivery。
- `EgressDestination`：目标类别和 provider，例如 `llm_provider/openrouter`、`hone_cloud`、`search/tavily`、`market_data/fmp`、`channel/feishu`、`channel/telegram`、`public_api_client`。
- `EgressPurpose`：外发目的，例如 `main_chat`、`scheduled_task`、`session_compaction`、`event_classifier`、`market_lookup`、`web_search`、`channel_delivery`、`artifact_download`。
- `EgressDataClass`：payload 里包含的数据类别，例如 `user_message`、`session_history`、`portfolio_summary`、`company_profile_summary`、`uploaded_attachment_excerpt`、`generated_artifact`、`tool_result`、`notification_text`、`system_prompt`、`market_query_only`。
- `EgressPayloadFingerprint`：不保存明文 payload，只记录 byte/char/token 估计、sha256 hash、redaction status、sample policy、linked audit id。
- `EgressPolicySnapshot`：当时的 surface、runner、actor、consent/policy version、private mode flag、permission decision id、config profile。

一期目标：

1. 在所有主要外发点记录 egress metadata，不扩大现有数据收集范围。
2. LLM audit 继续保存完整 request/response；egress ledger 只存最小摘要并关联 `llm_audit_record_id`。
3. 搜索/行情工具记录 query/ticker/date window 的分类和 hash，不默认保存完整 API response。
4. 通道 outbound 记录发送到外部 IM 平台的 segment 类型、字节数、附件数量和目标 channel，不记录完整消息正文，除非已有 run trace 或 channel log 权限允许查看。
5. Web 管理端和公开 `/me` 提供不同视图：管理员看跨 actor 诊断，用户看自己的第三方数据边界摘要。

## 用户体验变化

### 用户端

- `/me` 增加 `Data boundaries` 区域：
  - 最近 7/30 天发送到 LLM provider、Hone Cloud、Tavily、FMP、IM 平台的数据类别。
  - 每类只展示摘要：次数、最近时间、purpose、数据类别、provider、是否包含用户上传内容或公司画像摘要。
  - 不直接展示完整 prompt，避免把敏感内容二次暴露在账号页。
- 在 public chat 或 desktop chat 的高级信息里，可显示“本轮使用了哪些外部边界”：
  - `LLM provider: OpenRouter, purpose=main_chat, data=user_message+session_history+company_profile_summary`
  - `Market data: FMP, purpose=quote_lookup, data=ticker+date_window only`
  - `Search: Tavily, purpose=fresh_news, data=rewritten query only`
- 如果用户开启未来的 local-only/private mode，界面可以用 ledger 给出明确验收：
  - `本轮未发生 external LLM/search/market/channel egress`
  - 或 `已阻止 Tavily search，因为 private mode 不允许 external_search`

### 管理端

- 在现有 LLM Audit 或 Run Trace 旁新增 `Egress` 视图：
  - 按 actor、session、purpose、destination、provider、surface、runner、data class、日期过滤。
  - 每条 egress event 显示 linked run id、linked LLM audit id、linked tool call id、linked channel delivery id。
  - 支持导出最小 ledger report，用于 support 和企业/自托管审查。
- 用户详情页可看到该用户的外流摘要：
  - 哪些 provider 被用过。
  - 是否有上传附件、company profile、portfolio context 出现在 LLM egress 中。
  - 是否有外发失败或被 policy/permission 阻断的事件。
- 管理员处理“为什么扣了额度”“为什么搜索了这个词”“是否把这份附件发给模型”时，不再需要翻完整 prompt audit。

### 桌面端

- Desktop bundled 模式突出本机边界：
  - 本地 actor sandbox 读写不记为 external egress，只可选记为 local data access。
  - 调用外部 LLM/search/market data/Hone Cloud 才进入 external ledger。
- Desktop remote backend 明确标注数据先发到 remote backend，再由 backend 发往第三方；`EgressDestination` 可记录 `remote_backend` 和 downstream provider。
- Settings 中给出“外发透明度”说明：开启 LLM/search/market data 功能时，ledger 会记录最小 metadata；不会默认保存完整正文到 egress ledger。

### 多渠道

- Feishu/Telegram/Discord/iMessage 发送最终回复时，记录 channel egress：
  - channel、chat_scope、segment count、image/file attachment count、message chars、delivery success/failure。
  - 不保存完整私聊内容，除非已有 channel log 或 run trace 明确保留。
- 群聊场景可标记 `egress_surface=group_channel`，帮助后续判断“个人数据是否被发到群聊”。
- 如果未来 permission broker 拒绝群聊发送个人 portfolio 细节，ledger 可记录 `blocked_egress` 事件，供用户理解为什么回答被降级。

## 技术方案

### 1. 数据模型

建议在 `hone-core` 定义稳定类型，在 `memory` 用 SQLite 存储：

```rust
pub struct EgressEvent {
    pub event_id: String,
    pub created_at: String,
    pub actor: Option<ActorIdentity>,
    pub session_identity: Option<SessionIdentity>,
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub surface: EgressSurface,
    pub destination: EgressDestination,
    pub purpose: EgressPurpose,
    pub data_classes: Vec<EgressDataClass>,
    pub payload: EgressPayloadFingerprint,
    pub linked_records: EgressLinkedRecords,
    pub outcome: EgressOutcome,
    pub policy_snapshot: serde_json::Value,
}
```

SQLite schema 第一版：

```sql
CREATE TABLE external_egress_events (
  event_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  actor_channel TEXT,
  actor_user_id TEXT,
  actor_scope TEXT,
  session_id TEXT,
  session_channel TEXT,
  session_user_id TEXT,
  session_scope TEXT,
  run_id TEXT,
  surface TEXT NOT NULL,
  destination_kind TEXT NOT NULL,
  destination_provider TEXT,
  purpose TEXT NOT NULL,
  data_classes_json TEXT NOT NULL,
  payload_hash TEXT,
  payload_chars INTEGER,
  payload_bytes INTEGER,
  token_estimate INTEGER,
  redaction_status TEXT NOT NULL,
  linked_records_json TEXT NOT NULL,
  outcome TEXT NOT NULL,
  error_class TEXT,
  policy_snapshot_json TEXT NOT NULL
);

CREATE INDEX idx_external_egress_actor
  ON external_egress_events(actor_channel, actor_user_id, actor_scope, created_at);
CREATE INDEX idx_external_egress_session
  ON external_egress_events(session_id, created_at);
CREATE INDEX idx_external_egress_destination
  ON external_egress_events(destination_kind, destination_provider, created_at);
```

Retention:

- Default retention 90 days for metadata, configurable.
- Full payload remains governed by existing LLM audit / prompt audit / log retention.
- User data export can include ledger metadata without exposing full prompt bodies.

### 2. 插入点

优先复用集中点，避免每个 channel 自己发明格式：

- `crates/hone-channels/src/execution.rs`
  - 创建 `EgressContext`，包含 actor、session、surface、runner、run id、policy snapshot。
  - 将 context 注入 runner、tool registry 或 observer。
- `crates/hone-core/src/audit.rs` / `memory/src/llm_audit.rs`
  - `LlmAuditRecord` 写入后同步写一条 `destination_kind=llm_provider` 的 egress event。
  - `linked_records_json` 记录 `llm_audit_record_id`。
  - `data_classes` 初期可由 source/operation/metadata 粗分类，后续再精细化。
- `crates/hone-channels/src/prompt_audit.rs`
  - 不把 prompt audit 本身纳入 external egress；它是 local debug artifact。
  - 但 egress event 可记录 `prompt_audit_ref`，让管理员在有权限时跳转查看完整 payload。
- `crates/hone-tools/src/web_search.rs`
  - 在 Tavily 请求前后记录 `destination_kind=search`、`provider=tavily`、`purpose=web_search`、`data_classes=["search_query"]`。
  - query 明文不默认进 ledger，可存 hash + chars；debug mode 可关联 tool call result。
- `crates/hone-tools/src/data_fetch.rs`
  - 记录 `destination_kind=market_data`、`provider=fmp`、`purpose=market_lookup`、`data_classes=["ticker","date_window","endpoint_kind"]`。
  - ticker 本身敏感程度低于持仓明细，但在投资场景仍可能暴露关注标的；默认可保存规范化 ticker，或配置为 hash-only。
- `crates/hone-channels/src/runners/hone_cloud.rs`
  - 记录 `destination_kind=hone_cloud`，并标记它同时是远端 agent boundary，不只是普通 LLM provider。
- `crates/hone-channels/src/outbound.rs` 和 channel adapters
  - 记录 channel delivery egress：目标 channel、segment count、attachment count、message chars、outcome。
  - 对 `file://` 本地图表上传，标记 `data_classes=["generated_artifact"]`。
- `crates/hone-web-api/src/routes/public.rs`
  - OpenAI-compatible public API 可以记录 `destination_kind=public_api_client` 的 response egress，说明数据离开 Hone 服务返回给 API caller。

### 3. 数据分类策略

第一版不需要完美 NLP 分类，用 deterministic hints 即可：

- `user_message`：当前用户输入进入 LLM/Hone Cloud。
- `session_history`：restore context 或 historical messages 进入 LLM/Hone Cloud。
- `system_prompt`：system prompt 进入 LLM/Hone Cloud。
- `portfolio_context`：tool result 或 prompt metadata 标记包含 portfolio。
- `company_profile_context`：runtime input 或 local file marker 涉及 `company_profiles/`。
- `uploaded_attachment_excerpt`：attachment ingest / vision / vector store 产物进入 prompt。
- `search_query`：Tavily query。
- `market_query`：FMP ticker/date/endpoint。
- `generated_artifact`：图表、CSV、PDF 等文件被发送或下载。
- `notification_text`：最终通知正文进入 channel platform。

为了降低实现成本，初期可在各调用点显式传 `data_classes`，而不是自动解析完整 payload。后续再由 `Prompt Context Budget Inspector` 或 `Run Trace Workbench` 提供更精细的 context composition 标签。

### 4. API 与 UI

后端新增只读 API：

- `GET /api/egress-events`
- `GET /api/egress-events/:id`
- `GET /api/egress-summary?actor=...&days=30`
- `GET /api/public/egress-summary`

管理端 UI：

- `packages/app/src/pages/llm-audit.tsx` 可先增加 egress tab，避免新建过多导航。
- 列表列：time、actor、surface、destination、provider、purpose、data classes、size、outcome、linked audit/run。
- 详情页默认只展示 metadata；完整 payload 通过 linked LLM audit 或 prompt audit 进入，受现有 admin 权限控制。

公开端 UI：

- `/me` 只展示 summary，不展示逐条敏感 event。
- 文案要克制：这是透明度，不是营销。重点是“这些类型的数据可能被用于完成你的请求”。

### 5. 与 private/local-only mode 的关系

本提案不直接实现 private mode，但为它提供验收层：

- 增加 `egress_policy_snapshot.private_mode=true/false`。
- 如果 private mode 开启且发生外部 egress，ledger 必须能标出 violation。
- 如果某次请求被 permission broker 或 policy 拦截，记录 `outcome=blocked`，destination 仍记录预期目标，方便解释“本来会发给哪里，但被阻止了”。

## 实施步骤

1. **定义类型与存储**
   - 在 `hone-core` 增加 egress 类型。
   - 在 `memory` 增加 SQLite storage、migration、retention pruning。
   - 增加单元测试覆盖 insert/list/filter/retention。

2. **接入 LLM audit**
   - `LlmAuditSink::record` 成功后写 egress event。
   - 给 function_calling、multi-agent、session compaction、OpenAI-compatible provider record 补 metadata，使 data class 不完全依赖猜测。

3. **接入搜索和行情工具**
   - 为 `WebSearchTool`、`DataFetchTool` 增加可选 `EgressObserver`。
   - 测试 key fallback 下只记录一次 logical egress，或记录 attempts 但标明 same logical request，避免误读为多次用户主动外发。

4. **接入 Hone Cloud 与 channel outbound**
   - `HoneCloudRunner` 写专门 destination。
   - outbound adapter 发送最终消息后记录 channel egress summary。
   - 本地图表 marker 上传按 generated artifact 记录。

5. **后端查询 API**
   - 新增 `/api/egress-events` 和 `/api/egress-summary`。
   - public `/me` 只暴露当前登录用户 summary。
   - 支持 actor/session/date/provider/purpose/data_class filters。

6. **前端管理视图**
   - 在 LLM Audit 或 Run Trace 页面加 Egress tab。
   - 用户详情页加入 egress summary block。
   - Public `/me` 增加 Data boundaries summary。

7. **文档与 runbook**
   - 更新 `docs/repo-map.md`：标注 egress ledger 存储和接入点。
   - 更新 `docs/invariants.md`：明确 egress ledger 不保存完整 payload，完整内容仍由 LLM audit/prompt audit 管控。
   - 如果引入 private/local-only mode，再补 ADR 或 decision。

## 验证方式

自动化验证：

- Rust unit tests：
  - `EgressEvent` serialization roundtrip。
  - SQLite insert/list/filter/retention。
  - LLM audit 写入后生成 linked egress event。
  - Tavily/FMP mock client 下记录 logical egress，错误与 fallback 不泄露 API key。
  - outbound text/image segments 记录 chars、segments、generated artifact count。
- Frontend model tests：
  - egress summary rows group by destination/purpose/data class。
  - public `/me` summary 不展示 payload 或 raw prompt。
- CI-safe regression：
  - `tests/regression/ci/test_external_egress_ledger.sh` 使用 fake actor、fake LLM audit、mock search/market/outbound，断言 summary 可查询且无 raw secret/raw prompt。

手工验收：

- Web chat 发送一次需要市场数据的请求，管理员能看到 LLM provider egress + FMP market lookup egress。
- 发送一次需要 Tavily 的新闻请求，能看到 search query egress；query 与用户输入的关系可通过 hash/link 跳到 run trace，但默认列表不展示完整正文。
- 通过 Feishu/Telegram/Discord 发送一次图表回复，能看到 channel delivery egress 和 generated artifact egress。
- Public `/me` 只展示当前用户 30 天 summary，不泄露其它 actor。
- 删除或禁用 LLM audit 完整 payload 后，egress metadata 仍能回答“发生过什么类型的外发”，但不能恢复正文。

成功指标：

- 95% 以上 LLM/search/market/channel external calls 有 egress event。
- support 排查“某用户数据去了哪里”的平均步骤从跨 4 个页面/文件降低到 1 个 summary + linked detail。
- private/local-only mode 未来可以用 ledger 证明 external egress 为 0 或列出 blocked events。
- egress ledger 自身不引入新的 raw secret/raw prompt 泄露面。

## 风险与取舍

1. **ledger 变成新的敏感数据库。**
   取舍：第一版只存 metadata、hash、size、data class、linked ids，不存完整 prompt/response。完整 payload 继续由现有 LLM audit/prompt audit 管控。

2. **数据分类不够准确。**
   取舍：先用 deterministic hints 和调用点显式标签，避免过早引入复杂内容扫描。分类未知时标 `unknown_context`，不要假装精确。

3. **事件量增加。**
   取舍：metadata 行很小，可按 retention pruning；channel delivery 可聚合成每次 response 一条，不按每个底层 API request 记。

4. **用户看到 egress summary 后产生焦虑。**
   取舍：文案必须解释“这是为了透明度”，并给出关闭某类能力的入口或设置链接。不要用吓人的安全警报样式呈现正常调用。

5. **与 run trace / LLM audit / user data trust center 边界混淆。**
   取舍：egress ledger 只回答“数据离开边界了吗、去了哪类目标、包含什么类型、关联哪条证据”；run trace 回答“agent 怎么运行”；LLM audit 保存完整模型请求；user data trust center 处理导出/删除。

6. **市场数据 query 是否算敏感存在争议。**
   取舍：Hone 是投资研究产品，ticker query 可能暴露关注标的；默认纳入 ledger，但允许对 ticker 做 hash-only 或按 self-host policy 配置。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 和 `docs/proposals/`，重点对比以下相邻提案：

- `auto_p1_agent-permission-broker.md`：关注运行前是否允许 read/write/shell/tool/network 等动作。本提案关注动作发生或被阻断后的第三方外发账本，不负责批准流程。
- `auto_p1_user-data-trust-center.md`：关注用户数据 inventory、导出、删除。本提案可作为其中的一个数据域，但主题是外部 egress metadata 与第三方边界解释。
- `auto_p1_policy-consent-ledger.md`：关注用户是否接受条款、隐私和投资风险确认。本提案关注接受之后实际发生了哪些第三方数据边界事件。
- `auto_p1_privacy-preserving-product-events.md`：关注产品行为分析和增长指标，要求隐私保护。本提案不是产品 analytics，而是用户/管理员可解释的数据外流审计。
- `auto_p1_source-provenance-freshness.md`：关注外部事实来源的新鲜度、来源质量和引用。本提案关注 Hone 把用户/上下文数据发送到哪些外部目标。
- `auto_p1_run_trace_workbench.md`：关注 agent run 的步骤、事件和调试。本提案只抽取 external egress 边界，并用 linked ids 跳回 run trace。
- `auto_p0_secrets-vault-rotation.md`：关注 provider/channel credentials 的存储和轮换。本提案关注使用这些凭证时发生的数据外发，不保存或展示 raw secret。

因此，本提案不是重复隐私政策、权限审批、运行 trace 或来源可信度，而是补齐 Hone 在多 runner、多工具、多渠道架构中缺少的“第三方数据边界账本”。
