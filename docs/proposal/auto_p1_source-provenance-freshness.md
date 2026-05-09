# Proposal: Source Provenance and Freshness Registry

status: proposed
priority: P1
created_at: 2026-05-10 05:03:52 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `config.example.yaml`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-event-engine/src/source.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-event-engine/src/fmp.rs`
- `crates/hone-event-engine/src/pollers/{news,price,rss,sec_enrichment}.rs`
- `crates/hone-event-engine/src/global_digest/{fetcher,collector,curator}.rs`
- `crates/hone-event-engine/src/router/{classify,dispatch,policy}.rs`
- `crates/hone-web-api/src/routes/{notifications,events,llm_audit}.rs`
- `packages/app/src/pages/{notifications,task-health,llm-audit,settings}.tsx`
- `packages/app/src/lib/admin-content/{notifications,task-health,llm-audit,settings}.ts`

## 背景与现状

Hone 的核心产品承诺依赖“投资事实链路”可信：它需要在用户聊天、定时任务、全球 digest、公司画像和主动通知中使用金融行情、新闻、SEC filing、RSS、搜索和模型裁决结果。仓库已经有多个上游来源与局部可靠性处理：

- `DataFetchTool` 通过 FMP 获取 quote、profile、financials、news、earnings calendar、snapshot 等数据，并支持多 API key fallback。
- `WebSearchTool` 通过 Tavily 搜索最新信息，也支持多 key fallback，并区分 key 被拒绝与临时失败。
- `EventSource` 把 FMP poller、RSS feed、Telegram/social 来源等统一为 `poll()` + `schedule()`，由事件引擎定期拉取。
- `MarketEvent` 已包含 `source`、`url`、`occurred_at`、`payload`，`EventStore` 将事件落 SQLite 并保留 JSONL 镜像。
- `config.example.yaml` 对 event engine 的 FMP/RSS/source 开关、poll interval、full text fetch、Jina fallback、price/news 配额风险和 digest 策略已有大量配置注释。
- Router 已经能基于 source class、severity、prefs、cooldown、cap、quiet hours 做分流；`delivery_log` 记录投递状态。
- 管理端已有 notifications、task-health、llm-audit、logs 等排障页面，可以看到投递、任务运行、模型调用和日志。
- `docs/invariants.md` 明确要求时间敏感分析使用实时当前时间，宏观/新闻搜索要把相对时间改写为绝对日期，且投资输出不能在旧上下文或错误事实上漂移。

这些能力说明 Hone 已经有“来源字段”和“失败重试”，但还没有一等的来源可信度产品层。上游数据进入系统后，后续链路通常只能看到最终 JSON 或 `MarketEvent.source` 字符串，很难稳定回答：

- 这条行情/新闻/SEC filing 来自哪个 provider、哪个 endpoint、哪个 key pool attempt、何时拉取、是否经过 fallback。
- 这条数据相对当前回答或推送是否仍然新鲜。
- 某个 provider 当前是全局不可用、单 endpoint 抖动、配额耗尽，还是某个 symbol 缺数据。
- 一条 digest 或 agent 回答中哪些结论依赖 FMP、Tavily、RSS、SEC enrichment、LLM judge 或旧缓存。
- 用户看到“最新价格”“今日新闻”“未核验归因”时，系统有没有可机读证据支撑。

当前活跃工作中已经有 output safety gate、run trace、runtime readiness、delivery decision 等提案，但它们多数发生在“运行之后”或“投递之前”。本提案把重点前移到数据进入 Hone 的第一公里：让来源、时效、fallback 和可用性成为结构化事实，而不是散落在日志、payload 和最终回答文案中。

## 问题或机会

### 问题

1. 数据新鲜度没有统一契约。
   `MarketEvent::shelf_life()` 已经为事件类型定义了一部分保鲜期，但 `data_fetch`、`web_search`、global digest full-text fetch、SEC enrichment、RSS poller 等没有同一套 freshness policy。模型可能拿到一段看似“最新”的工具结果，却不知道 quote 是刚拉的、几分钟前缓存的，还是某个 endpoint 失败后的局部 snapshot。

2. 来源可靠性只存在于局部日志。
   `DataFetchTool` 和 `WebSearchTool` 知道 key fallback 和 provider 错误，但成功返回后不把 attempt、latency、status、provider error class 等作为业务元数据传给后续链路。event-engine poller 也有 tracing 字段，但管理端无法按 provider/source 看健康趋势。

3. 用户与运营无法解释“为什么事实不可信”。
   当 FMP quote 失败、Tavily key 耗尽、RSS full text fetch 降级、SEC summary 失败时，系统只能在局部错误里提示。最终用户看到的是缺失、旧数据或保守回答；管理员则需要从 logs、task-health、notifications 和 provider 配置中手工拼原因。

4. 安全门禁缺少上游证据摘要。
   output safety gate 可以拦截危险输出，但如果没有统一的 `SourceFreshness` / `ProvenanceEnvelope`，它仍要从文本和 ad hoc metadata 推断“是否 stale”“是否 fallback”“是否未核验”。这会增加误拦截和漏拦截。

5. 商业化和运维成本难以归因。
   Hone 的成本不只来自 LLM token，也来自 FMP、Tavily、RSS/full text、外部 API 配额和失败重试。当前 `llm_audit` 能看模型调用，但外部数据源没有类似的 health/usage ledger，难以判断“某个用户体验差是模型问题、数据源问题还是配置问题”。

### 机会

AI agent 产品正在从“能调用工具”走向“能解释依据和边界”。投资助手尤其需要在回答里区分：

- 已核验事实 vs. 来源不稳定的候选信息。
- 实时行情 vs. 延迟或缓存数据。
- 原始来源 vs. 聚合商转述。
- 直接 provider 数据 vs. fallback / full-text 抽取 / LLM summary。

Hone 已经具备多源采集、事件存储、审计页面和严格投资纪律。新增一个轻量的 **Source Provenance and Freshness Registry** 可以显著提升回答可信度、推送安全、排障效率和付费用户信任，同时不需要重写 runner、事件路由或 UI 主结构。

## 方案概述

新增一个来源证明与新鲜度注册层，目标是把所有“外部事实进入 Hone”的动作记录为统一的、可查询的 provenance/freshness 事实，并把关键摘要随工具结果、事件、digest item、scheduler task 和最终输出向后传递。

核心对象：

- `SourceObservation`
  一次外部数据访问或采集尝试。记录 provider、endpoint/source、symbol/query、attempt index、HTTP/status、latency、error class、fetched_at、payload_hash、result_count、key_pool_size、fallback_used。

- `ProvenanceEnvelope`
  附在工具结果、`MarketEvent.payload`、digest candidate 或 run metadata 上的轻量证明摘要。它不存完整响应，只存 source ids、retrieved_at、observed_at、freshness status、provider chain、confidence flags。

- `FreshnessPolicy`
  按数据类型定义保鲜期和降级规则，例如 quote 1-5 分钟、price alert 2 小时、RSS/news 12-24 小时、SEC filing 永不过期但 summary 可标记 stale、search result 以 query time 为准。

- `SourceHealthSnapshot`
  按 provider/source/endpoint 聚合最近窗口的可用性、错误率、配额拒绝、平均延迟、最近成功时间和最近失败原因。

- `EvidenceConfidence`
  面向产品和安全门禁的简化状态：`verified_fresh`、`verified_stale`、`partial_fallback`、`provider_unavailable`、`uncertain_source`、`llm_summarized`、`manual_or_legacy`.

第一版不做复杂数据血缘图，也不把完整第三方响应存进数据库。它只需要让后续链路稳定知道“这段事实来自哪里、何时取到、现在是否还能被当作最新事实使用”。

## 用户体验变化

### 用户端

- Public `/chat` 或 public `/portfolio` 不需要暴露复杂技术细节，但在时间敏感回答中可以显示轻量依据状态：
  - “行情来自 FMP，05:01 获取。”
  - “新闻来源为 RSS/Bloomberg，原文抓取失败，使用摘要。”
  - “当前实时数据源不可用，因此本轮不做最新价格判断。”
- 当用户问“为什么今天没有推送”或“这个价格准吗”时，agent 可以用统一 source health 摘要回答，而不是让用户理解后台日志。
- 在 digest 或事件卡片里，对高风险内容增加小型 provenance label：`fresh quote`、`RSS source`、`SEC filing`、`summary fallback`。

### 管理端

- 新增或扩展一个 `Source Health` 页面：
  - Provider 总览：FMP、Tavily、RSS feeds、SEC enrichment、full-text fetch、LLM classifier。
  - 最近 24h 成功率、错误率、配额拒绝、平均延迟、最近成功时间。
  - 按 endpoint/source 过滤：quote、stock_news、sec_filings、earning_calendar、rss:bloomberg_markets、tavily_search。
- 在 `Notifications` 详情抽屉中展示事件 provenance：来源、fetched_at、freshness、fallback_used、原始 URL、payload hash。
- 在 `Task Health` 中，当定时任务降级或未送达时显示上游数据状态，例如 `FMP quote stale`、`Tavily quota exhausted`、`RSS full text fallback`。
- 在 `LLM Audit` 或未来 `Run Trace Workbench` 中把本次 run 用到的 source observations 汇总为“事实来源包”，方便排查模型回答质量。

### 桌面端

- Desktop dashboard 在 backend/channel live count 之外显示关键数据源状态：
  - `Market data ok`
  - `Search quota exhausted`
  - `RSS degraded`
- 当本地 bundled runtime 的 FMP/Tavily key 未配置或失效时，桌面端给出明确的能力降级提示，而不是只表现为聊天工具失败。
- 不新增桌面 sidecar；只消费 console backend 的 source health API。

### 多渠道

- Feishu/Telegram/Discord/iMessage 回复中只暴露必要的用户态解释：
  - “实时行情源本轮不可用，我不会给出最新价判断。”
  - “这条提醒来自 SEC filing 原文，不是社交媒体传闻。”
- 群聊不展示个人配置细节或 API key 状态，只展示数据源级别健康和事件来源。
- 对主动推送，provenance label 应短而稳定，避免把 IM 消息变成排障日志。

## 技术方案

### 1. 新增 source observation 存储

建议在 `memory` 或 `hone-event-engine` 旁新增 SQLite 存储。第一版可以放在 `memory/src/source_health.rs`，因为 `data_fetch` / `web_search` / event-engine 都需要写入，不应只归属事件引擎。

表结构示意：

```text
source_observations (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  source_key TEXT NOT NULL,
  endpoint TEXT,
  subject TEXT,
  query_hash TEXT,
  attempt_index INTEGER NOT NULL,
  key_pool_size INTEGER,
  status TEXT NOT NULL,
  error_class TEXT,
  http_status INTEGER,
  result_count INTEGER,
  payload_hash TEXT,
  latency_ms INTEGER,
  fetched_at_ts INTEGER NOT NULL,
  observed_at_ts INTEGER NOT NULL,
  metadata_json TEXT
)

source_health_rollups (
  provider TEXT NOT NULL,
  source_key TEXT NOT NULL,
  window_start_ts INTEGER NOT NULL,
  window_seconds INTEGER NOT NULL,
  success_count INTEGER NOT NULL,
  failure_count INTEGER NOT NULL,
  quota_rejected_count INTEGER NOT NULL,
  avg_latency_ms INTEGER,
  last_success_ts INTEGER,
  last_failure_ts INTEGER,
  last_error_class TEXT,
  PRIMARY KEY(provider, source_key, window_start_ts, window_seconds)
)
```

保留期可短于业务事件，例如 observation 默认 14-30 天，rollup 默认 90 天。payload hash 只用于关联和去重，不保存第三方完整原文，避免扩大隐私和版权风险。

### 2. 给工具结果附 provenance envelope

改造 `DataFetchTool` 和 `WebSearchTool` 的返回格式时要保持兼容：

- 保持现有业务字段不变。
- 额外增加 `_hone_provenance` 字段，包含 provider、source_observation_ids、fetched_at、freshness、fallback_used、errors summary。
- 对 snapshot 这种聚合结果，分别记录 quote/profile/news 的 observation，并给每个子结果标注 freshness。
- 当所有 key 失败时，仍写 failure observation，让 source health 能解释“没有数据”。

示意：

```json
{
  "data_type": "snapshot",
  "ticker": "NVDA",
  "data": { "quote": [], "profile": [], "news": [] },
  "_hone_provenance": {
    "provider": "fmp",
    "retrieved_at": "2026-05-10T05:01:12+08:00",
    "freshness": "partial_fallback",
    "observations": ["srcobs_..."],
    "components": {
      "quote": { "freshness": "verified_fresh" },
      "news": { "freshness": "provider_unavailable" }
    }
  }
}
```

### 3. 给事件和 digest item 传递 provenance

`MarketEvent` 不必新增顶层字段，第一版可以把 provenance 摘要放入 `payload._hone_provenance`，避免迁移 `events` 表结构。后续若查询性能需要，再把 `freshness_status`、`source_observation_ids` 提升为 SQLite 索引列。

接入点：

- FMP price/news/SEC/earnings poller 记录 provider observation。
- RSS poller 记录 feed-level observation 和 item-level source URL。
- Global digest full-text fetch/Jina fallback 记录 full-text provenance。
- SEC enrichment / LLM summarizer 记录 `llm_summarized` 与 summarizer model，不把 summary 当作原始事实。
- Router 在 delivery_log body 或 metadata 中保留 compact provenance summary，供 notifications 页面展示。

### 4. 新增 freshness policy 层

在 `hone-event-engine` 或 `hone-tools` 增加纯函数：

```rust
pub enum SourceDataKind {
    Quote,
    IntradayPrice,
    CompanyProfile,
    FinancialStatement,
    News,
    RssItem,
    SecFiling,
    SearchResult,
    LlmSummary,
}

pub fn evaluate_freshness(kind: SourceDataKind, fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> FreshnessStatus;
```

初始策略：

- Quote / intraday price：默认 5 分钟内 fresh，超过后 stale；若用于收盘/历史回顾可由 caller 指定 relaxed mode。
- Price alert：沿用 `EventKind::shelf_life()` 的 2 小时语义。
- News / RSS / search：默认 24 小时内 fresh，但回答“今天/最新”时要求更短窗口或标注 retrieval time。
- SEC filing / earnings released / corporate action：事实永不过期，但 enrichment summary 要标注生成时间。
- Company profile / financial statement：不标为“最新事实”，而是带 statement period 或 profile updated_at。

### 5. Source health API 与 UI

新增 API：

- `GET /api/source-health/summary?hours=24`
- `GET /api/source-health/sources?provider=&source_key=&hours=`
- `GET /api/source-health/observations?provider=&status=&subject=&limit=`
- `GET /api/source-health/observations/:id`

Public 端可只暴露非常有限的只读摘要，例如：

- `GET /api/public/source-health/summary`
- 只返回 `market_data_available`、`search_available`、`last_success_at`、`degraded_reason`，不暴露 key pool 和内部 endpoint。

前端：

- 管理端 Settings 或新页面加入 `Source Health` tab。
- Notifications / Task Health / LLM Audit 详情页展示 provenance summary。
- Desktop dashboard 消费 summary capability。

### 6. 与 output safety gate 协作

本提案不替代安全门禁。它为安全门禁提供更可靠输入：

- `verified_stale`：不得生成“最新价/今日走势”的确定性结论。
- `partial_fallback`：允许回答已核验事实，但必须说明缺口。
- `provider_unavailable`：主动推送默认降级或不发送，direct chat 可提示稍后重试。
- `uncertain_source`：宏观/社交归因需要更严格 judge 或只进入 digest。
- `llm_summarized`：不得把 summary 当作原始披露文本。

## 实施步骤

### Phase 1: 记录与只读健康面

- 新增 `SourceObservation` 类型和 SQLite 存储。
- 在 `DataFetchTool`、`WebSearchTool` 成功/失败/fallback 处写 observation。
- 增加 source health summary API。
- 管理端先做只读 `Source Health` 页面，展示 provider/source 最近 24h 健康。
- 验证不改变现有工具业务返回结构，只新增 `_hone_provenance`。

### Phase 2: 事件链路 provenance

- FMP pollers、RSS poller、global digest full-text fetch 写 observation。
- `MarketEvent.payload` 增加 `_hone_provenance` compact summary。
- `EventStore` 查询和 notifications API 返回 provenance summary。
- Notifications 详情抽屉展示来源、fetched_at、freshness、fallback。
- 为 price/news/SEC/RSS 典型事件补单元测试，确认旧事件缺 provenance 时仍可展示 degraded state。

### Phase 3: Freshness policy 与安全协作

- 抽出 `FreshnessPolicy` 纯函数并覆盖 quote/news/search/sec 等数据类型。
- Scheduler / digest / output safety gate 读取 provenance/freshness，执行降级策略。
- 让 direct chat 在来源 stale 时优先要求重新获取或明确说明 retrieval time。
- 增加回归样本：FMP quote stale、Tavily quota exhausted、RSS full-text fallback、SEC summary stale。

### Phase 4: 产品化解释与运维指标

- Public 端展示轻量 source availability。
- Desktop dashboard 增加数据源状态。
- LLM Audit / Run Trace Workbench 汇总每次 run 的 source observation ids。
- 增加 provider usage/cost 估算，为后续 entitlement ledger 和商业化成本分析提供输入。

## 验证方式

- 单元测试：
  - `FreshnessPolicy` 覆盖 quote/news/rss/sec/search/llm_summary 的 fresh/stale 边界。
  - `SourceObservationStore` 覆盖成功、失败、quota rejected、rollup 聚合、保留期清理。
  - `DataFetchTool` snapshot 部分成功时 `_hone_provenance.components` 正确标注。
  - `WebSearchTool` key rejected 与 temporary failure 均写入 observation。

- 集成/回归：
  - 在无外部账号的 CI 中用 mock provider 验证 tool result 兼容旧字段且新增 provenance。
  - 在 `tests/regression/manual/` 保留真实 FMP/Tavily source health smoke，不进入默认 CI。
  - 构造旧 `MarketEvent.payload` 无 provenance 的记录，确认 notifications UI/API 不 500。

- 手工验收：
  - 配置空 FMP key，运行一次 data_fetch，管理端 Source Health 显示 `provider_unavailable`。
  - 配置 Tavily 多 key，其中一个失效，确认 fallback 被记录但用户仍得到结果。
  - 触发 RSS feed 拉取，确认事件详情显示 feed URL、fetched_at 和 freshness。
  - 模拟 quote 超过 freshness 窗口，确认安全门禁/回答不会称其为“最新价”。

- 指标：
  - Source health 页面能在 1 分钟内解释最近一次 provider 失败。
  - 90% 以上新 `MarketEvent` 带 provenance summary。
  - 输出安全门禁中的 `stale_data_unknown` 类 finding 减少，转为明确 reason code。
  - 用户反馈“价格/新闻来源不清楚”的排障路径能从 notification/session 详情直接跳到 source observation。

## 风险与取舍

- 存储膨胀：
  每次 provider 调用都会产生 observation。第一版必须只保存摘要和 hash，并设置短保留期与 rollup，避免把 source health 变成第二套原始数据仓库。

- 版权与隐私：
  不保存完整第三方响应正文，不把搜索 raw content 或新闻全文写入 observation。只保留 URL、source、hash、结果数量和状态。

- 兼容性：
  工具返回新增 `_hone_provenance` 可能影响模型上下文长度和旧测试快照。第一版应确保业务字段不变，并在 prompt/skill 中说明 `_hone_provenance` 是可读元数据。

- 过度保守：
  Freshness policy 太严格会让系统频繁拒答或不推送。需要按 origin 区分：主动推送严格，direct chat 可以提示并请求重新拉取。

- 多源冲突：
  本提案不解决不同 provider 价格不一致的最终裁决，只记录来源和时效。跨源一致性检查可作为 output safety gate 或未来 market data reconciliation 的扩展。

- API key 泄漏：
  Source health UI/API 不能暴露 key 值、key 前后缀或完整 provider error 中的敏感信息。只展示 key pool index、error class 和用户态原因。

## 与已有提案的差异

- 不重复 `auto_p0_investment_output_safety_gate.md`：
  安全门禁关注“最终输出能否送达、是否降级”。本提案关注“外部事实进入系统时的来源、时效、fallback 与健康记录”，为安全门禁提供结构化输入。

- 不重复 `auto_p1_delivery_decision_loop.md`：
  Delivery decision 解释事件为什么被推送、过滤、冷却或 quiet held；本提案解释事件和工具事实最初来自哪里、是否新鲜、上游 provider 是否健康。

- 不重复 `auto_p1_evidence_review_queue.md`：
  Evidence queue 把 thesis-changing 事件变成用户可处理待办；本提案不创建研究待办，不判断 thesis 是否改变，只提供来源可信度和新鲜度。

- 不重复 `auto_p1_run_trace_workbench.md`：
  Run trace 串联一次 agent 运行的 prompt、runner、tool、日志和输出；本提案是跨 run 的 source health/provenance registry，可被 run trace 引用但不以单次运行排障为中心。

- 不重复 `auto_p1_runtime_readiness_matrix.md`：
  Runtime readiness 关注 runner/model/channel/capability 是否可用；本提案关注外部金融与搜索数据源的观测、时效和事实链路。

- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：
  Skill runtime alignment 关注技能披露、执行和多 agent 协作；本提案不改变 skill runtime，只让工具和事件源输出更可解释的 provenance metadata。

## 文档同步说明

本轮仅创建产品/架构提案，没有开始执行该提案，也没有改变模块边界、运行约定或长期约束。因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续开始实施，本提案应先进入 `docs/current-plan.md` 与单独计划页，并在落地 source observation 存储、API、UI 后同步更新 repo map 与必要决策记录。
