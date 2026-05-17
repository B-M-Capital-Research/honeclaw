# Proposal: Model Route Evaluation Lab for Safe LLM Upgrades

status: proposed
priority: P1
created_at: 2026-05-18 02:05:11 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_prompt-context-budget-inspector.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `config.example.yaml`
- `crates/hone-llm/src/resolver.rs`
- `crates/hone-core/src/config/agent.rs`
- `crates/hone-core/src/config/event_engine.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/prompt_audit.rs`
- `memory/src/llm_audit.rs`
- `crates/hone-event-engine/src/news_classifier.rs`
- `crates/hone-event-engine/src/global_digest/{curator,event_dedupe,mainline_distill}.rs`
- `tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/settings.tsx`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Hone 已经把模型路线从单一 provider 扩展成多层运行矩阵：

- `config.example.yaml` 中 `agent.runner` 默认是 `hone_cloud`，同时支持 `codex_acp`、`opencode_acp`、`multi-agent`、`function_calling`、`gemini_cli` 和 `codex_cli`。
- `llm.profiles` 已经拆出 `aux`、`news_classifier`、`filing_summary`、`earnings_quality`、`digest_fast`、`digest_strong`、`mainline_short` 等背景任务 profile，并为不同路径配置独立 `max_tokens`、temperature、reasoning effort 和 provider。
- `crates/hone-llm/src/resolver.rs` 已把 profile 解析、OpenRouter / OpenAI-compatible provider、key pool、legacy model fallback 和 per-call max token override 收到一个入口。
- `crates/hone-web-api/src/lib.rs` 会为 event engine 装配 news classifier、SEC filing enrichment、earnings quality review、global digest pass1/pass2、event dedupe、mainline distill 和 renderer polisher 等多个 LLM 子路径。
- `memory/src/llm_audit.rs` 已记录每次 LLM 调用的 source、operation、provider、model、success、latency、request/response、metadata 和 token usage，并由 `/llm-audit` 页面展示。
- `crates/hone-channels/src/prompt_audit.rs` 会保存最终 system prompt 和 runtime input，给回答质量排查留下证据。
- `tests/regression/manual/test_event_engine_news_classifier_baseline.sh` 与 `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json` 已经把 43 条真实新闻样本、推荐模型和期望分类沉淀成可重跑 baseline。
- `docs/bugs/README.md` 里已有 “Event-engine still uses deprecated x-ai/grok-4.1-fast and loses LLM-backed enrichment” 这类模型下线/默认模型漂移造成的真实问题，说明模型选择不是纯配置喜好，而会影响关键链路可用性和通知质量。

这些能力说明 Hone 已经具备模型观测和局部回归的原料，但还没有一个面向模型升级、route 调整和灰度实验的产品/架构层。现在维护者想把 `news_classifier` 从 A 模型换到 B 模型、调整 `digest_fast` / `digest_strong` 的模型组合、让 Hone Cloud 默认模型升级，或比较 `codex_acp` 与 `opencode_acp` 的回答质量时，主要依赖手工运行脚本、看单次 LLM audit、读 prompt audit、再凭经验改配置。

这在 AI agent 产品里会逐渐成为 P1 问题：模型供应商、模型版本、价格、上下文长度、工具调用行为和安全风格都会频繁变化。Hone 的核心承诺是投资纪律和长期研究可信度，不能把“模型换了以后是否仍然可靠”留给单次 smoke test。

## 问题或机会

1. **模型是否可用与模型是否适合混在一起。**  
   Runtime Readiness Matrix 可以回答 API key、runner CLI、profile 和 channel 是否 ready，但它不能回答某个模型在新闻分类、digest 精读、company portrait 更新、public chat 投资边界、工具调用稳定性上的质量是否优于当前生产路线。

2. **现有 LLM audit 是事后日志，不是评估基线。**  
   `/llm-audit` 能看到调用成功、耗时、tokens 和原始请求响应，但没有把一组代表性样本转成可比较的 `accuracy / unsafe_rate / parse_failure_rate / latency / cost` 报告，也没有对候选模型做同题对照。

3. **event-engine 有局部 baseline，但没有推广成 route lab。**  
   新闻分类 baseline 证明仓库接受“真实样本 + 期望输出 + 可重跑模型”的质量策略，但它目前是单脚本、单任务、手工 env 开关。global digest、SEC enrichment、earnings review、mainline distill、chat runner 和 skill tool 调用没有同级基线。

4. **模型下线、价格变化和默认模型升级缺少安全流程。**  
   当前 default model 可以在配置、示例、桌面设置和装配层多个位置出现。模型不可用时 readiness 会发现失败，但更早的问题是：替代模型怎样筛选、用哪些样本验收、是否需要灰度、失败时如何回滚。

5. **用户反馈和人工判断没有进入模型路由决策。**  
   Response Feedback Learning Loop 会收集用户对答案的评价，Run Trace Workbench 会解释一次运行，但模型选择仍缺少一个把 feedback、audit、baseline 和 canary 结果汇总成 route recommendation 的控制面。

6. **商业化成本和质量不可分开治理。**  
   Usage Entitlement Ledger 可以计费控量，但真正的产品取舍是“这条后台 digest 是否值得用强模型”“public trial 是否需要低延迟模型”“高净值用户是否可以对某些路径启用更贵但更稳的 route”。没有评估层，成本优化容易退化成盲目换便宜模型。

## 方案概述

新增 **Model Route Evaluation Lab**：一个专门评估和灰度 Hone 内部 LLM routes 的工作台与数据模型。它不替代 `llm.profiles`、Readiness、LLM audit 或用户反馈，而是在它们之上提供“候选模型是否值得切换”的证据层。

核心对象：

1. `ModelRoute`
   稳定标识一个模型使用场景，例如 `chat.primary`、`chat.multi_agent.search`、`chat.multi_agent.answer`、`background.auxiliary_compact`、`event.news_classifier`、`event.global_digest.pass1`、`event.global_digest.pass2`、`event.sec_filing_summary`、`event.earnings_quality_review`、`event.mainline_distill`。

2. `EvaluationSuite`
   一组带版本的样本和断言，包含 route、fixture path、输入脱敏级别、期望输出 schema、人工标注、允许漂移规则和禁用项。例如 `news_classifier_2026_04_23`、`public_chat_investment_boundary_v1`、`digest_curation_noise_v1`、`company_portrait_update_v1`。

3. `CandidateRoute`
   待评估的 provider/model/profile 参数，包括 model id、base_url/provider、temperature、max_tokens、reasoning effort、tool budget、是否允许真实工具、是否只跑 dry-run。

4. `EvaluationRun`
   一次评估执行记录，保存 suite、candidate、baseline route、样本数、成功率、结构化解析失败率、安全违规率、平均延迟、token/cost、关键 drift、人工判定和是否允许上线。

5. `RouteCanaryPolicy`
   将一个候选 route 暂时应用到小流量或特定 actor / route / task 的规则，包含比例、时间窗、回滚条件、失败阈值和用户不可见标记。

6. `RouteRecommendation`
   由评估结果和线上 audit/feedback 汇总出的建议：`keep_current`、`upgrade_candidate`、`canary_candidate`、`rollback`、`collect_more_samples`、`block_due_to_safety`。

第一版应保持保守：

- 默认只做离线 fixture eval 和手工 live eval，不自动改生产配置。
- 不把真实用户完整对话直接写入 fixture；从 prompt audit / session / trace 最小化后再入库。
- 不把一次模型胜出当成全局路线胜出；route 必须按使用场景分别评估。
- 不在 P1 里做复杂在线实验平台；canary 先支持 owner/admin 手动启用、低流量和可回滚。

## 用户体验变化

### 用户端

- 普通用户不会看到模型实验细节，但会受益于更稳定的升级：模型切换前经过代表性样本验证，投资边界、来源新鲜度、digest 去噪和公司画像更新不因模型替换突然退化。
- Public chat 或 API 发生模型切换导致的短期问题时，错误响应可以关联 stable route id，而不是只暴露 provider/model 字符串。
- 如果后续 Feedback Loop 落地，用户负反馈可被聚合到某个 `ModelRoute`，形成“这个 route 最近质量下降”的产品信号。

### 管理端

- Settings 或新增 `Model Lab` 页面展示 route matrix：
  - 当前生产 profile / model / provider。
  - 最近 7 天 success、latency、token usage、parse failures。
  - 最近一次 evaluation run 和是否通过。
  - 可比较的候选模型与推荐动作。
- 管理员可以对某个 route 运行评估：
  - 选择 suite，例如 `event.news_classifier baseline`。
  - 选择 candidate profile 或临时模型参数。
  - 查看逐样本结果、drift、失败样本、成本和延迟。
- 对 chat route，可先只跑离线 prompt fixture；对 event-engine route，可跑现有真实样本 baseline；对 canary，可只允许 owner 指定少数 actor 或后台 route。
- `/llm-audit` 增加 route 维度过滤和“转入评估样本”的动作，避免维护者每次手工复制 request/response。

### 桌面端

- Desktop bundled 模式可以显示本机当前模型路线是否有推荐升级或风险，但默认不自动切换。
- Remote backend 模式只展示远端返回的 route verdict；本机 OpenCode/Codex 可用性仍归 Runtime Readiness。
- 对本地用户，Model Lab 可以帮助比较 `opencode_acp` 继承的本机模型和 Hone Cloud 路线，但要明确“评估会消耗本地/云端模型额度”。

### 多渠道

- Feishu / Telegram / Discord / iMessage 不需要暴露 Model Lab UI。
- channel 回复、scheduled task 和 digest 的 LLM audit 应带 route id，这样当某个渠道投诉“最近回答变差”时，管理员能按渠道和 route 聚合，而不是只按 provider/model 搜索。
- 多渠道主动推送相关 route 的 canary 必须默认 digest-only 或低风险 actor，避免候选模型直接影响高严重度即时推送。

## 技术方案

### 1. Route id 和 profile 解析对齐

在 `hone-core` 或 `hone-llm` 定义轻量 route id，不替换现有 config：

```rust
pub enum ModelRouteKind {
    ChatPrimary,
    ChatMultiAgentSearch,
    ChatMultiAgentAnswer,
    BackgroundAuxiliary,
    EventNewsClassifier,
    EventGlobalDigestPass1,
    EventGlobalDigestPass2,
    EventDedupe,
    EventSecFilingSummary,
    EventEarningsQualityReview,
    EventMainlineDistill,
    EventRendererPolisher,
}
```

装配层仍从 `config.yaml` 的 `llm.profiles` 和 legacy fallback 读取，但每次创建 provider 时附带 route id：

- `LlmResolver::provider_for_profile_or_openrouter_model(...)` 可保持不变，外层装配传入 route id。
- `build_event_engine_*` 路径在 audit metadata 中写入 `route_id`、`profile_name`、`model` 和 `max_tokens_override`。
- chat runner、multi-agent、auxiliary compaction 路径也写入 `route_id`，供 LLM audit 和未来 trace 聚合。

### 2. Evaluation suite 存储

新增 fixture 目录，逐步从现有手工 baseline 演进：

```text
tests/fixtures/model_eval/
  event_news_classifier_v1.json
  global_digest_curation_v1.json
  public_chat_boundary_v1.json
  company_portrait_update_v1.json
```

每个 suite 至少包含：

- `route_id`
- `schema_version`
- `created_at`
- `source_policy`: synthetic / minimized_real / public_sample
- `input`
- `expected`
- `assertions`
- `safety_tags`
- `notes`

现有 `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json` 可以先作为兼容 suite 被读取，不需要立即迁移文件。

### 3. Evaluation runner

第一版用 CLI / admin API 双入口，但执行逻辑共享：

```text
hone-cli model-lab eval --route event.news_classifier \
  --suite tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json \
  --candidate-profile news_classifier_candidate \
  --limit 20 --json
```

runner 行为：

- 从 canonical `config.yaml` 读取候选 profile 或临时 candidate 参数。
- 对每个样本构造 route-specific prompt。
- 调用真实 LLM provider，记录 response、latency、usage、parse result。
- 执行 suite assertions。
- 输出 `EvaluationRun` JSON，并可保存到 `data/runtime/model-eval/runs/*.json`。

第一版仅把 live model eval 放入 `tests/regression/manual/` 或手工命令，不进入默认 CI。无外部账号依赖的 schema lint、fixture parser、assertion evaluator 可进 CI。

### 4. 管理端 Model Lab

新增后端只读与执行 API：

- `GET /api/model-lab/routes`
- `GET /api/model-lab/suites`
- `GET /api/model-lab/runs`
- `POST /api/model-lab/evaluate`
- `POST /api/model-lab/candidates/preview`

前端落点可先放在 Settings 的 Agent 区域下：

- route matrix。
- candidate form。
- run result table。
- drift sample drawer。
- links to LLM audit / prompt audit / trace。

执行 live eval 必须需要 admin/operator 权限；如果 Operator Access proposal 尚未落地，则先复用现有 admin auth，并在文案中标注会消耗模型额度。

### 5. Canary 和回滚策略

P1 第一阶段不自动写生产 config。第二阶段可以支持手动 canary overlay：

```yaml
model_lab:
  canaries:
    - route: event.news_classifier
      candidate_profile: news_classifier_candidate
      actor_allowlist: ["web:direct:u_123"]
      percent: 5
      expires_at: "2026-06-01T00:00:00+08:00"
      rollback_on:
        parse_failure_rate_gt: 0.05
        llm_error_rate_gt: 0.10
```

实现上建议把 canary overlay 放在 generated runtime 或单独 runtime state，不直接改 canonical `llm.profiles`。只有当管理员明确 promote 时，才生成 config mutation / decision 记录。

### 6. 与现有提案协作

- Runtime Readiness Matrix：提供 candidate 是否可连通、API key 是否可用、runner CLI 是否满足版本；Model Lab 在 ready 后评估质量。
- Run Trace Workbench：把线上失败 trace 最小化为 eval sample；eval run 也可生成 trace-like evidence。
- Response Feedback Learning Loop：把高价值负反馈汇总为 suite 候选样本或 route degradation 信号。
- Prompt Context Budget Inspector：为 chat route eval 固化输入上下文和裁剪摘要，避免样本不可复现。
- Output Safety Gate：提供 safety assertions，阻止 unsafe candidate 被 promote。
- User Journey Replay Lab：验证产品状态机和 fake runner；Model Lab 验证真实模型 route 的输出质量和成本。

## 实施步骤

### Phase 1: Route id 与 eval metadata

- 定义 `ModelRouteKind` 和稳定 string id。
- 在 event-engine LLM 装配、auxiliary compaction、chat runner 和 multi-agent 路径的 LLM audit metadata 中写入 route id、profile、model、max_tokens 和 candidate/baseline 标记。
- 扩展 `/llm-audit` filter 支持 route id。
- 验证目标：不改变任何模型调用结果，只增加 metadata。

### Phase 2: Fixture schema 与现有 baseline 适配

- 定义 `EvaluationSuite` schema 和 parser。
- 让现有 news classifier baseline 可被新 runner 读取。
- 增加 schema lint，拒绝密钥、真实手机号、绝对个人路径和未脱敏大段用户文本。
- 保留原有手工 baseline 脚本，直到新 runner 输出等价报告。

### Phase 3: CLI evaluation runner

- 新增 `hone-cli model-lab eval` 或独立 `tests/regression/manual/test_model_route_eval.sh`。
- 支持 candidate profile、limit、JSON report、allow drift、cost/latency summary。
- 首批覆盖 `event.news_classifier`，再覆盖 `event.mainline_distill` 或 `event.global_digest.pass1`。

### Phase 4: Admin Model Lab view

- 增加 route matrix 与 evaluation run list。
- 支持从 LLM audit detail 创建最小化样本草稿。
- 支持运行手工 live eval 并保存 report。
- 暂不自动改配置，只给 recommendation。

### Phase 5: 手动 canary 与 promote gate

- 支持 route-scoped canary overlay，默认关闭。
- canary 只允许低风险 route 或 actor allowlist。
- promote 前要求最近 evaluation run 通过、safety assertions 通过、readiness ready，并记录 config mutation / admin audit。

## 验证方式

- 静态验证：
  - `docs/proposal/auto_p1_model-route-evaluation-lab.md` 文件名符合 `auto_p[0-4]_*.md`。
  - 本提案包含 `status`、`priority`、`related_files`、`verification` 和 `risks` 字段。
  - 查重覆盖 `docs/proposal/` 与 `docs/proposals/`。
- 单元测试：
  - route id string 稳定性。
  - suite parser、assertion evaluator、sample redaction lint。
  - LLM audit metadata 序列化兼容旧记录。
- 手工回归：
  - 不带外部账号时，model eval fixture lint 通过，并提示 live eval 需要 config-owned provider key。
  - 带 OpenRouter key 时，news classifier suite 对当前 baseline 模型和 candidate 模型输出对照报告。
  - candidate parse failure、HTTP 402/404、timeout、unparseable answer 都进入 report，而不是让整轮评估崩溃。
- 产品验收：
  - 管理端能看到每个 route 当前 model/profile、最近 audit 指标和最近 eval run。
  - 管理员能解释“为什么暂不升级某个模型”：样本漂移、解析失败、安全断言失败、成本过高或 readiness blocked。
  - 不运行 eval 时，生产模型路径输出完全不变。
- 成功指标：
  - 默认模型升级前至少有一份 route evaluation report。
  - event-engine 关键 LLM routes 的 parse failure 和 noisy push regression 可被 suite 捕获。
  - 模型不可用或退化时，从发现到替代模型验证的人工步骤减少。

## 风险与取舍

- 风险：live eval 会消耗模型额度。取舍：默认只跑 lint 和离线 fixture；真实模型 eval 必须显式开启，并显示预计样本数、route 和 provider。
- 风险：fixture 过拟合旧模型，阻碍合理升级。取舍：每个 suite 标注 intent 和允许漂移；报告区分 critical drift、acceptable wording drift 和 open review。
- 风险：真实用户样本脱敏不彻底。取舍：从 LLM audit / prompt audit 转样本时必须最小化，并用 lint 拒绝密钥、手机号、绝对路径和长原文。
- 风险：管理端误把 eval 分数当成唯一真相。取舍：Model Lab 只给 recommendation，promote 仍需要人工确认、readiness ready 和 safety gate 通过。
- 风险：canary 增加运行时分支复杂度。取舍：第一版不做自动 canary；第二阶段只支持 route-scoped、actor allowlist、过期时间和明确回滚阈值。
- 风险：不同 route 的评估标准差异很大。取舍：先从结构化输出强的 event-engine routes 开始，chat route 只做边界和安全类小 suite，不追求完整答案文本匹配。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 下全部自动提案和历史 `docs/proposals/`：

- 不重复 `auto_p1_runtime_readiness_matrix.md`：Readiness 判断 route 是否可运行、依赖是否齐全；本提案评估可运行 route 的质量、成本、漂移和升级风险。
- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 解释一次线上运行发生了什么；本提案对候选模型进行离线/手工 live 对照，并为 promote/canary 提供证据。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：Feedback 收集用户对已发生回答的满意度；本提案把反馈和样本转化为 route evaluation suite，并比较候选模型。
- 不重复 `auto_p1_prompt-context-budget-inspector.md`：Context inspector 关注一次 prompt 带了什么、裁剪了什么；本提案关注相同输入在不同模型/profile 下的输出质量。
- 不重复 `auto_p1_user-journey-replay-lab.md`：Replay Lab 用 fake runner 验证产品状态机和接口契约；Model Lab 使用真实或候选 LLM route 验证模型输出质量。
- 不重复 `auto_p0_investment_output_safety_gate.md`：Safety Gate 是运行时拦截危险输出；Model Lab 是上线前和灰度中的模型评估与升级决策层，可复用 safety assertions。
- 不重复历史 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该提案关注 skill 状态和 multi-agent runner 语义；本提案只评估模型路由，不改变 skill 注入契约。

查重结论：现有 proposal 覆盖了 readiness、trace、feedback、prompt budget、journey replay 和 safety，但没有覆盖“模型/provider/profile 替换前如何用代表性样本评估、对照、灰度和回滚”的控制面。这个主题直接回应当前模型供应商快速变化、默认模型下线、后台 LLM routes 增多和成本/质量权衡的问题，适合作为 P1 提案进入机会池。

## 本轮文档同步说明

本轮只创建 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续执行本提案，应按动态计划准入标准新增或复用 `docs/current-plans/model-route-evaluation-lab.md`，并在引入 route id、eval fixture schema、CLI/API、canary overlay 或 promote gate 时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
