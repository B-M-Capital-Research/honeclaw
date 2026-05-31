# Proposal: LLM Cost and Margin Guard for Provider-Aware Operations

status: proposed
priority: P1
created_at: 2026-06-01T02:04:33+08:00
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
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_prompt-context-budget-inspector.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `docs/proposal/auto_p2_self-serve-billing-checkout.md`
- `docs/proposal/auto_p0_public-edge-abuse-guard.md`
- `memory/src/llm_audit.rs`
- `memory/src/quota.rs`
- `memory/src/web_auth.rs`
- `crates/hone-llm/src/resolver.rs`
- `crates/hone-llm/src/provider.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/runners/multi_agent.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/types.ts`

## 背景与现状

Honeclaw 的产品形态已经从本地聊天助手扩展成多入口投资研究系统：Public Web、Hone Cloud API、管理端、桌面 bundled / remote、多渠道 IM、定时任务、event-engine、company portraits、skills、深度研究代理和多 runner agent runtime 都会触发 LLM 调用。

当前仓库已经具备成本治理的基础原料：

- `memory/src/llm_audit.rs` 会记录 `provider`、`model`、`source`、`operation`、`latency_ms`、`success`、`prompt_tokens`、`completion_tokens` 和 `total_tokens`，本地走 SQLite，cloud mode 走 PG `cloud_llm_audit_records`。
- `crates/hone-web-api/src/routes/llm_audit.rs` 和 `packages/app/src/pages/llm-audit.tsx` 已经提供管理端 LLM audit 列表与详情，可以按 actor、session、success 等条件查看原始调用。
- `docs/invariants.md` 明确 LLM audit 是 append-oriented runtime evidence，不能在 cloud mode 下把 SQLite 当成 blob 替代。
- `docs/decisions.md` 的 `D-2026-05-11-01` 要求 LLM credentials config-only，说明模型路由和成本边界都应当由 Hone 自己可解释，而不是依赖外部环境变量隐式决定。
- `crates/hone-llm/src/resolver.rs` 和多个 runner 已经支持不同 provider / profile / model route；event-engine 还为 news classifier、SEC filing enrichment、earnings quality review、mainline distill 等后台任务选择独立 LLM route 和 completion budget。

但这些信息目前仍停留在“逐条审计记录”层。运营者可以看到某条调用用了多少 token，却不能稳定回答：

- 过去 24 小时 / 7 天哪个 actor、feature、model route 最贵？
- Public trial、Hone Cloud API、scheduled task、event-engine 和 desktop remote 的单位成本分别是多少？
- 某个 provider 换价、模型切换、prompt 变长或 event-engine 打开后，毛利是否突然被吞掉？
- 一次失败调用是否也产生了高额 token 成本？
- 当前 token 统计缺失是模型不返回 usage，还是采集链路坏了？

这说明 Hone 已经有审计事实，但还没有 provider-aware cost layer。随着 public chat、API key、billing checkout 和 entitlement proposals 推进，继续只看“每日次数”和“token 数”会低估商业风险。

## 问题或机会

这个提案值得作为 P1，是因为它同时影响四条关键链路：

1. **商业化判断**

   Self-serve billing 和 entitlement 可以定义 plan，但如果没有每个 plan / route / feature 的真实成本估算，就无法设定 trial 配额、API 价格、pro plan 上限或高成本功能的开关策略。

2. **模型升级安全**

   Model Route Evaluation Lab 关注质量与回归；Prompt Context Budget Inspector 关注运行前上下文大小。它们都不能替代“上线后真实用量按 provider 价格折算后的成本漂移”。模型升级如果只看答案质量，可能会把后台任务成本放大数倍。

3. **运维排障**

   LLM audit 页面适合查单条请求，但当 OpenRouter / OpenAI-compatible provider 返回 402、限额耗尽或 latency 上升时，维护者需要聚合视图判断是某个用户、某个自动化、某个模型 route，还是某个后台 enrichment path 在异常消耗。

4. **用户体验边界**

   Public Web 和 Hone Cloud API 用户未来会看到权益、额度和升级入口。若成本层缺失，产品只能用粗糙的对话次数限制，无法解释“为什么高成本长文档分析需要更高套餐”或“为什么本地 desktop 不受公共服务成本限制”。

机会是：第一版不需要改 runner 协议，也不需要接支付网关。只要在 LLM audit 之上增加 price catalog、cost estimate 和 margin guard，就能把现有 runtime evidence 转成可运营指标。

## 方案概述

新增一个 **LLM Cost and Margin Guard**，把原始 LLM audit token 记录归因为 provider-aware cost events，并提供管理端聚合、阈值预警和发布/模型切换前后的对比。

核心原则：

- **成本估算不是扣费真相源**：第一版只服务运营观测、毛利保护和模型路线决策，不直接向用户扣费。
- **价格目录显式配置**：不依赖运行时联网抓 provider 最新价格；管理员手工配置或随 release 更新 price catalog，并保留生效时间。
- **保留原始 audit**：不替代 `llm_audit_records`，只生成可重复计算的 cost projection。
- **区分缺失 usage 与零成本**：模型未返回 token usage 时必须显式标记 `usage_missing`，不能当作免费。
- **按 feature 归因**：chat、Hone Cloud API、scheduled task、event-engine、session compaction、deep research proxy、skill script LLM path 等要能分开看。

## 用户体验变化

### 用户端

- Public `/chat` 不展示内部成本，但在未来 entitlement / billing 页面可显示粗粒度用量解释，例如“长文档 / 高频自动化会消耗更高服务额度”。
- Hone Cloud API 响应仍可返回 token usage / quota summary；不暴露 provider 成本或 Hone 毛利。
- Desktop bundled local mode 可以在设置或诊断页展示“本地模型/API 由用户自付，Hone 不计入 cloud 服务成本”，减少用户把本地与公共服务混淆。

### 管理端

- LLM Audit 从单条记录表升级为两个视图：
  - **Audit Records**：保留当前逐条 JSON / token / latency 详情。
  - **Cost Overview**：按时间、actor、channel、source、operation、provider、model、route profile 聚合 input tokens、output tokens、estimated cost、usage missing ratio、failure cost。
- Settings / Users 增加 actor 级成本摘要：最近 24h、7d、30d 的 LLM 估算成本、主要模型、主要功能来源和异常 warning。
- 当某个模型 route 的单位成本超过阈值，管理端显示 margin warning，而不是等 provider 账单或 402 错误后才发现。
- Release / rollout 后可以对比“切换前 7 天 vs 切换后 24 小时”的 token、cost、latency 和 failure cost。

### 桌面端

- Remote backend 模式显示 backend 返回的服务成本状态，例如“云服务成本由当前账户权益控制”。
- Bundled local 模式只展示本机 token 用量和 provider/model，不用 cloud cost price catalog 误导用户。
- Diagnostic / support bundle 后续可纳入成本摘要，但不包含完整 prompt、API key 或 provider invoice。

### 多渠道与自动化

- Feishu / Telegram / Discord / iMessage 触发的对话成本按 actor/channel 聚合，便于发现某个群聊或自动化异常放大成本。
- Scheduled task、heartbeat、event-engine enrichment、mainline distill 等后台成本独立聚合，不与用户主动 chat 混在一起。
- Hone Cloud API 的成本聚合按 API key owner / actor / route 统计，支持未来商业套餐和滥用防护调参。

## 技术方案

### 1. Price catalog

新增一个小型价格目录，建议先放在 canonical config 或 `memory` SQLite/PG 表中：

```text
llm_price_catalog (
  price_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  model_pattern TEXT NOT NULL,
  currency TEXT NOT NULL,
  input_per_million REAL NOT NULL,
  output_per_million REAL NOT NULL,
  cache_read_per_million REAL,
  cache_write_per_million REAL,
  effective_from TEXT NOT NULL,
  effective_to TEXT,
  source_note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

第一版只要求 input/output token 价格。`model_pattern` 支持精确 model id 和简单 wildcard，例如 `openrouter/openai/gpt-*`，但匹配顺序必须稳定：exact > provider-specific wildcard > fallback unknown。

未知模型不阻断运行，只输出：

- `price_status=unknown_model`
- `estimated_cost=null`
- `tokens_observed=true|false`

这样可以先观测，再补 price catalog。

### 2. Cost projection

在 `memory/src/llm_audit.rs` 之外新增可重算的 projection，而不是把成本硬写死进原 audit：

```text
llm_cost_estimates (
  estimate_id TEXT PRIMARY KEY,
  audit_record_id TEXT NOT NULL,
  price_id TEXT,
  actor_channel TEXT,
  actor_user_id TEXT,
  actor_scope TEXT,
  session_id TEXT,
  source TEXT NOT NULL,
  operation TEXT NOT NULL,
  provider TEXT NOT NULL,
  model TEXT,
  prompt_tokens INTEGER,
  completion_tokens INTEGER,
  total_tokens INTEGER,
  estimated_input_cost REAL,
  estimated_output_cost REAL,
  estimated_total_cost REAL,
  currency TEXT,
  price_status TEXT NOT NULL,
  usage_status TEXT NOT NULL,
  success INTEGER NOT NULL,
  created_at TEXT NOT NULL
)
```

也可以先不物化表，直接在查询时 join audit + catalog 做 projection；但物化有两个优点：

- 历史价格可冻结，避免 provider 调价后历史成本被误改。
- 管理端聚合更快，cloud PG 部署下也更容易做分页和时间窗口查询。

推荐第一阶段采用“写 audit 后异步 projection，失败可重建”的方式。

### 3. Feature attribution

当前 LLM audit 有 `source` 和 `operation`，但不足以表达所有产品功能。建议在写 audit metadata 时逐步补稳定字段：

- `feature`: `public_chat | admin_chat | channel_chat | hone_cloud_api | scheduled_task | heartbeat | event_engine_news_classifier | event_engine_sec_filing | event_engine_earnings_review | event_engine_mainline_distill | session_compaction | deep_research | unknown`
- `route_profile`: 来自 `llm.profiles` 或具体 runner route，例如 `agent.opencode`, `multi_agent.search`, `multi_agent.answer`, `event_engine.news_classifier_llm`
- `task_id` / `cron_job_id` / `run_id` / `api_key_prefix`：按可用性写入，不要求一期全部覆盖。

第一版可用 source/operation 映射得到 feature，后续再让各调用点显式传 metadata。

### 4. Margin guard rules

新增轻量规则配置：

```yaml
llm_cost:
  currency: USD
  unknown_model_warn: true
  usage_missing_warn_threshold_pct: 10
  daily_total_cost_warn: 50.0
  daily_public_cost_warn: 20.0
  actor_daily_cost_warn: 5.0
  failure_cost_warn_pct: 20
  route_unit_cost_warn:
    multi_agent.answer: 0.08
    event_engine.sec_filing: 0.03
```

这些阈值只产生 warning，不直接拒绝用户请求。真正的 request allow/deny 仍归 Usage Entitlement Ledger 或 Public Edge Abuse Guard 管。

### 5. API

新增 admin API：

- `GET /api/llm-cost/overview?from=&to=&group_by=feature|actor|model|provider|route`
- `GET /api/llm-cost/actors/:actor_key`
- `GET /api/llm-cost/routes`
- `GET /api/llm-cost/price-catalog`
- `POST /api/llm-cost/price-catalog`
- `POST /api/llm-cost/recompute?from=&to=`

权限沿用管理端 `/api` bearer token；未来 Operator Access Control 落地后，将 price catalog mutation 归为 operator-scoped admin action。

### 6. UI

管理端可以先复用 `LLM Audit` 页面增加 tab，不必新增大型导航：

- Cost Overview cards：total estimated cost、tokens、failure cost、usage missing ratio、unknown model count。
- Breakdown table：feature / model / provider / actor / route 维度可切换。
- Warning list：unknown model、usage missing、daily threshold crossed、failure cost spike。
- Drill-down：点击某个聚合行跳到现有 audit records filter。

这样能复用当前 API 和页面心智，不把一线排障入口拆散。

### 7. Cloud / local storage

- Local mode：price catalog 和 cost estimates 可使用 SQLite，路径跟随 `storage.llm_audit_db_path` 或新增 `storage.llm_cost_db_path`。
- Cloud mode：使用 PG，和 `cloud_llm_audit_records` 同一 cloud runtime helper 管理 schema。
- Migration：不迁移历史账单，只从现有 audit token 字段回算最近 N 天；token 缺失的历史记录标记 `usage_status=missing`。

## 实施步骤

1. **梳理 audit metadata 映射**

   盘点 `source` / `operation` 在 chat、runner、event-engine、compaction 和 Hone Cloud API 下的取值，定义第一版 `feature` 映射表。只写文档和测试 fixture，不先改运行行为。

2. **新增 price catalog 与 projection**

   在 `memory` 增加本地 SQLite schema、PG schema helper、price matcher、cost calculator。单元测试覆盖 exact/wildcard/fallback、effective date、unknown model、usage missing、历史价格冻结。

3. **写入与回算**

   在 LLM audit 写入后触发 cost projection；新增 recompute API/CLI 用于价格目录补齐后回算最近窗口。projection 失败只记录 warning，不影响主请求。

4. **管理端 Cost Overview**

   在 `packages/app/src/pages/llm-audit.tsx` 增加 Cost tab，展示聚合 cards、breakdown table、warnings 和跳转到 audit records 的链接。

5. **阈值预警**

   加入 margin guard rules，先只在管理端和 logs 中提示，不做 request blocking。后续再把严重成本异常接入 Product Rollout Kill-Switch 或 Usage Entitlement。

6. **发布和运营校准**

   对比至少 7 天真实 audit：补齐未知模型 price catalog，检查 usage missing ratio，给 trial / API / event-engine 提供默认成本阈值建议。

## 验证方式

- Rust 单元测试：
  - price catalog exact / wildcard / fallback 匹配顺序稳定。
  - `effective_from` / `effective_to` 能选择正确历史价格。
  - prompt/completion token 计算成本精度稳定，未知模型不 panic。
  - usage missing 与 zero token 区分明确。
  - local SQLite migration 不破坏现有 `llm_audit_records` 查询。
- API 测试：
  - `/api/llm-cost/overview` 能按 feature、actor、model 聚合。
  - price catalog mutation 需要 admin auth。
  - recompute 不重复生成同一 audit 的 estimate，或使用 deterministic upsert。
- 前端测试：
  - Cost Overview 空态、unknown model warning、usage missing warning、聚合行跳转 filter。
  - 大数字和货币格式在桌面宽屏和窄屏下不溢出。
- 手工验收：
  - 用 fixture 生成 public chat、scheduled task、event-engine 三类 audit，确认成本分布能区分。
  - 人为配置一个高价模型，确认 margin warning 出现但不阻断聊天。
  - 删除 price catalog 后，确认系统显示 unknown model 而不是把成本显示为 0。
- 指标：
  - 95% 以上成功 LLM audit 能生成 cost projection。
  - `usage_missing` 低于可接受阈值；若某 provider 长期缺 usage，需要在 model route 评估中标记。
  - 模型切换后 24 小时内能看到单位成本、失败成本和 latency 的变化。

## 风险与取舍

- 风险：provider 价格经常变化，手工 catalog 可能过期。取舍：第一版追求运营估算，不做对账系统；价格记录必须带 `effective_from` 和 `source_note`，管理端提示 catalog stale。
- 风险：成本视图可能被误解成用户账单。取舍：UI 明确标注 `estimated provider cost`，不在 public 用户端展示内部毛利。
- 风险：token usage 不完整导致估算偏低。取舍：缺失 usage 单独计入 warning，不能显示为 0；高缺失 route 进入模型路线评估。
- 风险：增加 audit 写后处理会影响主链路。取舍：projection 必须异步或 best-effort，失败不影响回答。
- 风险：过早优化成本可能牺牲答案质量。取舍：本提案只提供观测和 guard，不自动降级模型；模型质量仍由 Model Route Evaluation Lab 和人工灰度决策负责。
- 不做：不接 Stripe/支付宝/微信支付，不生成用户账单，不替代 Usage Entitlement Ledger，不做 provider invoice reconciliation，不自动联网抓最新价格。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/`，重点比对了 usage entitlement、prompt context budget、model route evaluation、Hone Cloud API contract、self-serve billing、public edge abuse guard、run trace、privacy product events、storage budget 和 support bundle。

- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：Usage Entitlement 定义用户权益、额度、grant 和 allow/deny 决策；本提案只把 LLM audit token 转成 provider-aware estimated cost 和 margin warning，不决定用户能不能用。
- 与 `auto_p1_prompt-context-budget-inspector.md` 不重复：Prompt Budget 在运行前估算上下文规模和裁剪风险；本提案在运行后按 provider price catalog 归因真实 token 成本。
- 与 `auto_p1_model-route-evaluation-lab.md` 不重复：Model Route Evaluation 关注答案质量、回归和模型升级可用性；本提案关注上线后的成本漂移、failure cost 和 unknown price 风险。
- 与 `auto_p1_hone-cloud-api-contract.md` 不重复：Hone Cloud API Contract 定义外部 API shape、quota usage 和 developer console；本提案是内部运营成本层，不扩展 public API 计费语义。
- 与 `auto_p2_self-serve-billing-checkout.md` 不重复：Billing Checkout 处理用户购买、续费、取消和支付生命周期；本提案不接支付，只为定价和套餐设计提供成本依据。
- 与 `auto_p0_public-edge-abuse-guard.md` 不重复：Edge Abuse Guard 处理 pre-run rate/abuse 防护；本提案处理已经发生的 LLM 调用成本归因和运营预警。
- 与 `auto_p1_privacy-preserving-product-events.md` 不重复：Product Events 统计 adoption journey；本提案统计 LLM provider cost，不采集点击流或用户行为路径。

## 文档同步说明

本轮只新增 proposal，不开始执行方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续进入实施，应按动态计划准入标准新增或复用 `docs/current-plans/llm-cost-margin-guard.md`，并在新增存储、API、管理端页面、配置项或 cloud schema 后同步更新 `docs/repo-map.md`、`docs/invariants.md`、必要的 runbook / decision / handoff。
