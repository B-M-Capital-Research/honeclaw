# Proposal: Investment Output Safety Gate and Evaluation Loop

status: proposed
priority: P0
created_at: 2026-05-05 23:03:07 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/bugs/README.md`
- `docs/bugs/oil_price_scheduler_geopolitical_hallucination.md`
- `docs/bugs/scheduler_heartbeat_near_threshold_false_trigger.md`
- `docs/bugs/feishu_scheduler_stale_price_fallback_after_data_fetch_failure.md`
- `docs/bugs/feishu_direct_non_finance_query_misroutes_to_stock_research.md`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/response_finalizer.rs`
- `crates/hone-channels/src/scheduler.rs`
- `crates/hone-channels/examples/finance_consistency_llm_smoke.rs`
- `crates/hone-event-engine/src/news_classifier.rs`
- `crates/hone-event-engine/src/router/dispatch.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/lib/tos.ts`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 的核心定位是投资纪律与长期研究助手，不是泛聊天产品。仓库已经把这条产品边界写进多个层面：

- `README.md` 明确强调 Hone 是投资研究 co-pilot，会帮助用户保持理性、监控持仓和执行纪律。
- `docs/invariants.md` 要求全局金融领域约束由 `crates/hone-channels/src/prompt.rs` 注入，禁止直接荐股、非金融问题短路、宏观叙事要区分主线与噪音，并且时间敏感分析要使用实时当前时间。
- `prompt.rs` 的 `DEFAULT_FINANCE_DOMAIN_POLICY` 已包含不少高风险规则：非金融拒绝、禁止下指令式买卖、实体歧义澄清、旧上下文漂移约束、内部策略外泄约束、报价字段一致性约束、原油与大宗商品归因约束。
- `DEFAULT_COMPANY_PROFILE_POLICY` 进一步要求公司画像是长期研究资产，不应演变成交易建议清单，并要求模型参考已有画像和证伪条件。
- `response_finalizer.rs` 已有用户可见输出净化，能拦截系统提示泄漏、internal-only 输出、空成功、过渡计划句和不可用本地图表。
- `scheduler.rs` 对 heartbeat 输出已经积累了局部硬门禁，例如 JSON 状态解析、near-threshold 未触发抑制、trigger/current price 矛盾识别、内部 marker 抑制。
- `crates/hone-channels/examples/finance_consistency_llm_smoke.rs` 已经把原油报价字段一致性抽成真实 LLM 冒烟测试，说明团队已经认识到“仅靠 prompt 约束不够，需要输出级验证”。
- `crates/hone-event-engine/src/news_classifier.rs` 已把新闻重要性裁决做成可缓存、可基线化的 LLM 分类器，证明 Hone 可以把模型判断包成可测、可回归的产品组件。

同时，`docs/bugs/README.md` 里的活跃缺陷说明风险并非理论问题。当前活跃队列中仍有多条投资敏感输出问题：原油定时播报把未核验地缘叙述当作事实送达，晨报在行情数据失败后仍发送旧价格，Feishu 直聊非金融话题未按领域边界拒绝，heartbeat 在结构化状态和阈值触发语义之间漂移。这些问题的共同点是：模型已经生成了看似完整的正文，系统把它当作可送达内容；但正文在事实口径、证据来源、触发条件、领域边界或投资建议边界上不应直接送达。

当前系统有 prompt policy、局部正则、最终文本净化、bug 文档和少量 LLM smoke，但还没有一个一等的“投资敏感输出安全门禁”产品层。结果是每个缺陷都倾向于补一条 prompt 或一段路径专用规则，而不是形成可解释、可度量、可回归的交付前质量边界。

## 问题或机会

这是 P0 级问题，因为 Hone 的核心承诺依赖用户相信它不会把未核验市场叙事、错误价格、过期数据或直接操作指令包装成纪律建议。一次错误推送可能影响用户对持仓、风险暴露、宏观变量和次日交易计划的判断。

当前缺口集中在五类链路：

1. Prompt-only 约束不可验证。
   `DEFAULT_FINANCE_DOMAIN_POLICY` 越来越长，但系统无法在每次输出后判断模型是否真的遵守了“非金融拒绝”“不荐股”“报价口径一致”“原因归因需可追溯”等规则。

2. 高风险输出与普通输出没有统一分级。
   一句寒暄、一次公司长期研究、一次 heartbeat 触发、一次原油价格播报、一次大盘晨报和一次 public chat 试用回复，当前主要共享同一 finalizer。它能挡住内部泄漏和空成功，但不能按投资风险级别做不同送达策略。

3. 定时任务和主动推送缺少“发送前最后一公里”审查。
   多数危险样本来自 scheduler / heartbeat / market briefing，因为这类内容会主动送达，用户未必有上下文纠错机会。当前 heartbeat 有局部抑制，但普通 cron、event digest、global digest、public chat 和 IM direct 没有统一的 safety verdict。

4. 事实来源、时间口径和数据失败状态没有进入可机读 verdict。
   当 `data_fetch` 失败、搜索降级、价格字段冲突、来源时间戳缺失时，最终正文可能仍写成确定性结论。系统需要知道“这条答复只能降级为不发送 / 只报告已核验事实 / 要求用户确认”，而不是只依赖模型自我克制。

5. 质量改进缺少长期评测集。
   bug 文档里有大量真实坏样本，但它们没有被统一抽成 regression fixture。每次修复只能验证单点规则，难以回答“本次 prompt / runner / model / scheduler 改动是否让投资安全整体变好或变坏”。

机会是：仓库已有足够材料，可以先做一条低侵入的 safety gate。它不需要改 runner 架构，不需要引入外部合规 SaaS，也不需要替换现有 prompt。第一版只要把高风险输出分类、规则检查、LLM judge、送达策略、人工复盘和回归样本串起来，就能显著降低核心信任风险。

## 方案概述

新增 `InvestmentOutputSafetyGate`：所有投资敏感的用户可见输出在持久化或送达前生成一个 safety verdict。verdict 不替代模型回答，而是决定这条内容能否原样送达、是否需要降级、是否需要补充风险说明、是否应转为失败并等待下一轮重试。

核心对象：

- `SafetySubject`：待审查输出，包含 actor、channel、session、origin、execution mode、user input、final text、tool/data 状态摘要、scheduler job metadata、event metadata、时间窗口、runner/model。
- `SafetyRiskProfile`：输出类型分级，例如 `casual_finance_chat`、`single_stock_analysis`、`portfolio_action`、`heartbeat_trigger`、`market_price_broadcast`、`macro/geopolitical_attribution`、`non_finance_boundary`、`public_trial_chat`。
- `SafetyVerdict`：`allow`、`allow_with_notice`、`downgrade_to_verified_facts`、`suppress_noop`、`ask_clarification`、`block_and_retry_later`、`fail_user_visible`。
- `SafetyFinding`：稳定 reason code，例如 `stale_price_fallback`、`unverified_geopolitical_causality`、`price_math_inconsistent`、`direct_buy_sell_instruction`、`non_finance_answered`、`near_threshold_without_crossing`、`old_context_symbol_drift`、`internal_policy_leak`。
- `SafetyEvalCase`：从真实 bug 和 synthetic cases 抽出的回归样本，记录输入、上下文、坏输出、期望 verdict 和允许的修复输出形态。

一期目标不是做完整金融合规系统，而是把 Hone 自己已经写下的产品纪律变成可执行的送达边界：

- 高风险定时推送先过 gate，再写 `sent/delivered`。
- Web/IM direct 可以先以 allow-only 观测模式接入，再对明确危险项启用 block。
- Public trial chat 对直接买卖指令、非金融问题和高风险操作建议更严格。
- 管理端能看到被 gate 拦截或降级的记录，帮助产品和工程复盘。
- 每个新增 bugfix 都必须把坏样本转为 safety eval case，避免同类问题换模型后复发。

## 用户体验变化

### 用户端

- 普通金融问答不增加负担；安全通过时用户无感。
- 当输出需要降级时，用户看到的是明确、可信的产品语义：
  - “行情数据源本轮不一致，我只保留已核验价格，不做原因归因。”
  - “当前条件尚未真正触发，Hone 不发送提醒；已记录为观察。”
  - “这类问题超出 Hone 的投资研究范围，我不能继续展开。”
  - “你在寻求操作决策，我会拆解条件和风险，但不会替你下买卖指令。”
- 定时任务如果被 suppress，不再把长篇 noop 内容送到 IM；如需要可在 Web/desktop 看到“本轮未送达原因”。
- 对 public Web 试用用户，安全提示应保持简短，不暴露内部 policy 名称或 runner 细节。

### 管理端

- 在现有 `Task health` / `Notifications` / 未来 `Run Trace Workbench` 中增加 safety verdict 字段。
- 新增或扩展一个 `Safety Review` 视图，展示最近 24h 被拦截、降级、改写或仅观测的输出。
- 每条记录可展开查看：origin、actor、job/session、risk profile、findings、原始正文摘要、最终送达正文摘要、相关 bug/eval case。
- 管理员可以把误拦截标记为 false positive，把漏拦截样本一键导出为新的 eval case 草稿。

### 桌面端

- Desktop bundled 模式的 dashboard 显示“最近被拦截的高风险输出”和“安全门禁运行状态”。
- 当本地用户报告“某个定时任务没发”，桌面能说明是因为 no-op、数据失败、目标解析失败，还是 safety gate 拦截，而不是只看滚动日志。
- 不增加新 sidecar；所有数据来自 backend API 与本地 SQLite / JSONL。

### 多渠道

- Feishu / Telegram / Discord / iMessage 不直接暴露 gate 细节，只发送用户态结果：
  - `suppress_noop`：默认不发消息，必要时只在任务台账记录。
  - `block_and_retry_later`：发送短失败提示，说明系统将在下一次触发重试。
  - `downgrade_to_verified_facts`：只发送已核验事实，并明确不归因。
- 群聊中涉及个人持仓、交易单、仓位建议时，gate 复用群聊隐私约束，优先引导私聊。

## 技术方案

### 1. 新增 safety gate 模块

建议在 `crates/hone-channels` 新增 `output_safety` 模块，因为最终用户可见文本、scheduler 送达和 channel outbound 都从这里经过：

```text
crates/hone-channels/src/output_safety.rs
```

初始 API：

```rust
pub struct SafetySubject { ... }
pub struct SafetyVerdict { ... }

pub async fn evaluate_investment_output(
    core: &HoneBotCore,
    subject: SafetySubject,
) -> SafetyVerdict;
```

接入点：

- `AgentSession::run()` finalizer 之后、持久化 assistant final 前：先以 observe 模式写 verdict；对明确 dangerous 类别再 block。
- `scheduler.rs` heartbeat parse 之后、写 `completed + sent` 前：启用 enforce 模式，避免危险主动推送。
- event digest / unified digest 渲染后、sink 前：先接入市场价格、宏观归因和 stale-data 检查。
- public `/api/public/v1/chat/completions` 先以 observe + obvious block 接入，避免 API 形态输出买卖指令或非金融长答。

兼容策略：默认配置先让 direct chat `observe_only=true`，scheduler/heartbeat 可以对已有明确 bug 规则 enforce。这样不会一次性打断所有对话体验。

### 2. Risk profile 先规则化，后续再模型化

第一版不要把所有判断交给 LLM。先用确定性规则识别高风险 subject：

- `market_price_broadcast`：正文包含价格、日内高低、涨跌幅、合约、最新价、收盘价等字段。
- `macro/geopolitical_attribution`：正文包含原油、大宗商品、地缘、战争、航运、OPEC、库存、外交谈判等归因词。
- `portfolio_action`：正文包含买、卖、加仓、减仓、止损、止盈、仓位、满仓、梭哈、期权操作等。
- `heartbeat_trigger`：scheduler job 为 heartbeat 或正文含触发价 / 阈值 / 触发线。
- `non_finance_boundary`：用户输入明显为楼市、生活、技术闲聊等非金融问题，且输出没有拒绝。

这些 profile 只决定检查强度，不直接判错。

### 3. 分层 verdict：规则、来源状态、LLM judge

检查顺序建议：

1. **硬规则检查**：内部提示泄漏、绝对路径、空成功、过渡计划句继续复用现有 finalizer；新增价格数学矛盾、near-threshold 未穿越、直接买卖命令、非金融未拒绝等确定性规则。
2. **tool/data 状态检查**：如果 data_fetch/search 明确失败或过期，正文不得写成确定性最新价格或确定性归因；只允许“数据链路失败，因此不归因/不发送”。
3. **LLM judge 检查**：对宏观/地缘归因、复杂操作建议、旧上下文漂移等规则难以覆盖的输出，调用辅助模型做 yes/no JSON verdict。可复用 `llm.auxiliary`，失败时 conservative fallback。
4. **送达策略**：按 origin 决定最终动作。主动推送更严格，direct chat 更偏向补充说明或澄清，public API 更严格限制买卖指令。

LLM judge 必须是受控小输出，不要让它重写全文。建议 schema：

```json
{
  "verdict": "allow|block|downgrade|clarify",
  "findings": ["unverified_geopolitical_causality"],
  "confidence": 0.0,
  "user_safe_summary": "..."
}
```

### 4. 观测与审计存储

新增本地 SQLite 或 JSONL 记录，优先 SQLite：

```text
safety_verdicts (
  verdict_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  actor TEXT,
  channel TEXT,
  origin TEXT NOT NULL,
  session_id TEXT,
  job_id TEXT,
  trace_id TEXT,
  risk_profile TEXT NOT NULL,
  verdict TEXT NOT NULL,
  findings_json TEXT NOT NULL,
  subject_hash TEXT NOT NULL,
  raw_preview TEXT,
  delivered_preview TEXT,
  judge_model TEXT,
  mode TEXT NOT NULL
)
```

如果 Run Trace Workbench 后续落地，`trace_id` 可直接串联；在此之前使用 session/job/time window 关联即可。

隐私策略：

- 默认只存 preview 和 hash，不存完整 prompt、完整用户输入或完整 tool result。
- 对 public 用户和外部导出只暴露用户可理解原因，不暴露内部 policy 文本。
- 管理端 full detail 仍遵守本地 admin 边界。

### 5. Safety eval fixture 与回归命令

新增长期保留 fixture：

```text
tests/regression/ci/fixtures/investment_output_safety/*.json
```

每个 case 包含：

- `name`
- `origin`
- `user_input`
- `final_text`
- `tool_status_summary`
- `job_metadata`
- `expected_verdict`
- `expected_findings`
- `source_bug`

新增 CI-safe 回归脚本：

```text
tests/regression/ci/test_investment_output_safety.sh
```

CI-safe 部分只跑确定性规则，不调用外部 LLM。现有 `finance_consistency_llm_smoke.rs` 继续保留为手工 / 模型基线验证，必要时迁到 `tests/regression/manual/` 或增加 wrapper。

### 6. 配置与灰度

在 `config.example.yaml` 增加可选配置：

```yaml
output_safety:
  enabled: true
  direct_chat_mode: observe
  scheduler_mode: enforce
  public_chat_mode: enforce_obvious
  llm_judge_enabled: false
  store_preview_chars: 600
```

第一阶段 `llm_judge_enabled=false`，先落确定性 gate、存储和 UI。第二阶段再启用辅助模型 judge，并用 baseline fixture 评估 false positive / false negative。

## 实施步骤

### Phase 1: 确定性 safety gate 与主动推送保护

- 新增 `output_safety` 类型、规则检查和单元测试。
- 把 `scheduler.rs` 里 heartbeat near-threshold、trigger/current contradiction 等局部逻辑收拢成可复用 finding。
- 在 scheduler / heartbeat 送达前启用 enforce：危险输出不写 `sent`，而是写 `safety_blocked` 或 `skipped_noop`。
- 把原油口径、stale price fallback、non-finance boundary、direct buy/sell instruction 抽成 fixture。

### Phase 2: 观测存储与管理端复盘

- 增加 `safety_verdicts` 存储和 `/api/safety-verdicts` 只读 API。
- `Task health`、`Notifications`、`Logs` 页面显示 safety finding 链接。
- 新增 Safety Review 列表，支持 actor、origin、verdict、finding、time range 过滤。
- 每条 active bug 修复时，把坏样本转成 eval case 并关联 bug 文档。

### Phase 3: Direct chat / public chat 灰度

- `AgentSession::run()` finalizer 之后接入 observe verdict。
- 对 `internal_policy_leak`、`direct_buy_sell_instruction`、`non_finance_answered` 这类高置信 finding 在 public chat 启用 enforce。
- IM direct 先只在 admin logs 和 safety review 中观测，等 false positive 降低后再逐类 enforce。

### Phase 4: LLM judge 与质量指标

- 使用 `llm.auxiliary` 增加可选 LLM judge，先覆盖宏观/地缘归因与复杂操作建议。
- 增加人工 review 标注，用于统计 false positive / false negative。
- 形成 dashboard 指标：blocked/suppressed 数、误拦截数、漏拦截复发数、主动推送危险输出率、safety gate latency。
- 后续与 Run Trace Workbench 联动，把 verdict 作为每次 run 的一个阶段。

## 验证方式

- Rust 单元测试：
  - 价格数学矛盾、不同合约混用、near-threshold 未触发、直接买卖指令、非金融未拒绝、内部策略外泄都能产出稳定 finding。
  - scheduler enforce 模式下，`block_and_retry_later` 不会写成 `completed + sent`。
  - observe 模式只记录 verdict，不改变原有输出。
  - 旧记录缺少 `trace_id` 或 `tool_status_summary` 时返回 degraded verdict，不 panic。
- 回归脚本：
  - `bash tests/regression/ci/test_investment_output_safety.sh` 跑 fixture，不依赖外部账号或 LLM。
  - 真实 LLM smoke 保留手工执行，用于比较不同模型是否仍遵守报价一致性和归因边界。
- 前端测试：
  - `bun run test:web` 覆盖 safety verdict 数据转换、finding badge、Safety Review 过滤。
  - `Task health` 中 safety-blocked / safety-suppressed 状态不与普通 failed 混淆。
- 手工验收：
  - 构造原油价格字段矛盾的 scheduler 输出，应被降级或阻断。
  - 构造 heartbeat “接近但未触发阈值”输出，应不送达。
  - Public chat 中问“直接告诉我明天买哪只”，应改为条件与风险分析，或给出边界说明。
  - Feishu direct 中问明显非金融问题，不应进入 stock_research 或给出长篇答案。
- 指标：
  - 新增投资敏感 bug 必须能归入某个 finding 或新增 finding。
  - 主动推送路径 100% 写入 safety verdict。
  - safety gate p95 延迟在确定性规则模式下低于 20ms；启用 LLM judge 的路径必须有超时和保守降级。

## 风险与取舍

- 风险：误拦截会让用户觉得 Hone 变迟钝。取舍：direct chat 先 observe，主动推送先 enforce；finding 必须可解释、可在管理端标 false positive。
- 风险：LLM judge 也会漂移。取舍：第一版以确定性规则和 fixture 为主；LLM judge 只做高复杂度补充，并且输出受 JSON schema 限制。
- 风险：门禁层可能演变成又一套难维护 prompt。取舍：把核心纪律转成 reason code、fixture 和规则函数；prompt 只是 judge 的补充，不是唯一真相源。
- 风险：存储 raw preview 可能包含敏感输入。取舍：默认截断、hash、脱敏；public 和导出不显示完整内容。
- 风险：主动推送被阻断后用户错过真实重要事件。取舍：`block_and_retry_later` 与 `suppress_noop` 分开；高价值事件可降级为“数据未核验，暂不归因”的短提示，而不是完全静默。
- 风险：规则过度针对历史 bug。取舍：每条规则必须抽象为产品纪律，如“数据源不一致不得精确播报”，而不是匹配单个坏文案。
- 不做：不提供投资合规法律意见，不自动重写用户投资策略，不引入云端审核服务，不让 UI 直接编辑公司画像或交易决策。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案解释 event-engine 为什么推/不推以及如何调偏好；本提案判断“即将送达的投资敏感正文是否安全可信”。
- 与 `auto_p1_evidence_review_queue.md` 不重复：该提案把事件变成 thesis 复盘待办；本提案阻止未核验事实、错误口径或越界建议直接进入用户可见输出。
- 与 `auto_p1_investment_context_intake.md` 不重复：该提案解决新用户投资上下文缺口；本提案解决任意阶段输出的安全送达边界。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案处理跨渠道真实用户资产归属；本提案不合并 actor，只在当前输出路径生成 safety verdict。
- 与 `auto_p1_research_artifact_library.md` 不重复：该提案把深度研究报告持久化为交付物；本提案对报告、chat、scheduler、digest 等所有投资敏感输出统一做送达前审查。
- 与 `auto_p1_run_trace_workbench.md` 不重复：该提案聚合一次 run 的可观测证据；本提案是运行链路中的决策门禁。未来 trace 可以引用 safety verdict，但二者职责不同。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：该提案管理权益和成本；本提案管理输出可信度和投资风险边界。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案处理桌面启动接管；本提案只让桌面可见 safety 状态。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案对齐 skill schema 与执行语义；本提案把 skill 或 runner 产出的最终内容纳入统一安全判定。

本轮选择该主题，是因为当前 bug 台账显示投资敏感输出仍会以成功态送达，且现有修复多依赖 prompt 增补和路径局部补丁。把输出安全转为 P0 一等产品/架构层，能保护 Hone 最核心的用户信任和留存根基。
