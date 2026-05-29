# Proposal: Runtime Dependency Circuit Breaker for Degraded Agent Operations

status: proposed
priority: P1
created_at: 2026-05-29 20:06:25 +0800
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
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `config.example.yaml`
- `crates/hone-llm/src/resolver.rs`
- `crates/hone-llm/src/openai_compatible.rs`
- `crates/hone-llm/src/openrouter.rs`
- `memory/src/llm_audit.rs`
- `crates/hone-event-engine/src/spawner.rs`
- `crates/hone-event-engine/src/router/dispatch.rs`
- `crates/hone-event-engine/src/sinks/{feishu,telegram,discord,imessage}.rs`
- `crates/hone-web-api/src/routes/{llm_audit,task_runs,notifications,meta}.rs`
- `packages/app/src/pages/{task-health,llm-audit,notifications,settings}.tsx`

## 背景与现状

Honeclaw 已经把关键产品链路拆成多个外部依赖和运行时 route：

- `crates/hone-llm/src/resolver.rs` 通过 `llm.providers`、`llm.profiles`、OpenRouter fallback 和 OpenAI-compatible provider 创建后台 LLM route。
- `crates/hone-llm/src/openai_compatible.rs` 对 transport error 做一次重试，并按 key pool 逐个尝试；但这只是单次请求内的重试，不会在多次失败后隔离某个 provider/key/model。
- `memory/src/llm_audit.rs` 已记录 provider、model、success、latency、token 和 error 文本，可作为运行中故障趋势的事实来源。
- `crates/hone-event-engine/src/spawner.rs` 对 poller tick 设置 timeout，失败后写 task observer；但失败只停留在单个 tick，不会自动降低下游调用频率或暂停依赖同一坏源的非关键任务。
- `crates/hone-event-engine/src/router/dispatch.rs` 已经有 per-actor cap、cooldown、digest demotion、delivery log 等抗噪机制；这些主要控制通知强度，不处理 provider outage 或错误风暴。
- 管理端已有 `Task Health`、`LLM Audit`、`Notifications`、`Logs` 等观测面，可以看到失败，但缺少“系统应自动停手多久、降级到什么、何时恢复”的决策层。

现有 proposal 已经覆盖许多相邻能力：Runtime Readiness 解决运行前配置是否 ready，Model Lab 解决模型路线切换前的质量评估，Source Provenance 解决事实来源和新鲜度，Product Rollout Kill Switch 解决人工 feature 灰度/止血，Interrupted Recovery 解决已开始但未闭环的 run。剩下的关键缺口是：**运行中某个外部依赖进入短期故障、限流、配额耗尽、响应极慢或错误率飙升时，Hone 仍然主要靠每次请求自己失败、每个任务自己重试、管理员事后看日志。**

对一个多渠道投资 agent 来说，这会影响核心可信度：同一 provider outage 可能同时让 public chat 变慢、event-engine enrichment 失败、digest 质量下降、session compaction 卡住、后台任务反复消耗额度，并在多个渠道给用户制造不一致错误。

## 问题或机会

这是 P1，而不是 P0：它通常不直接泄露数据或破坏所有核心安全边界，但会显著影响可用性、成本、主动推送质量和运维效率，且越多 hosted/public 用户使用，问题越明显。

主要问题：

1. **单次重试不能防止错误风暴。**  
   LLM provider 已有 key pool 和一次 transport retry，但如果某个 provider 处于 429、402、5xx 或长 timeout，多个 chat、cron、event-engine route 仍会继续打同一坏依赖，放大延迟和成本。

2. **依赖故障没有统一状态机。**  
   LLM、FMP/Tavily、RSS/full-text、channel sink、Hone Cloud API、Aliyun SMS/Captcha 都可能失败，但系统没有统一的 `closed / open / half_open` 状态、冷却窗口、恢复探针和影响范围说明。

3. **降级策略分散且不可解释。**  
   某些失败应该让 public chat 直接提示稍后重试；某些后台 enrichment 可以跳过，保留原始事件进入 digest；某些主动推送应暂停 direct send 但保留 buffer；某些 session compaction 应延后而不是阻塞用户对话。目前这些策略缺少统一 reason code。

4. **管理员只能观察，不能看到自动保护决策。**  
   `Task Health` 能展示失败率，`LLM Audit` 能筛 provider，`Notifications` 能看 delivery status，但没有一个 `Dependency Health` 或 `Circuit Breaker` 面板告诉维护者：“news_classifier route 已因 10 分钟内 80% 失败打开熔断，30 分钟后 half-open probe”。

5. **商业化成本和用户体验同时受影响。**  
   Hosted/public 场景下，provider outage 不只是不可靠，还会烧掉配额、拖慢排队、触发用户重复提交。Usage Entitlement 可以计量成本，但需要 circuit breaker 提供“这些消耗是故障重试导致”的归因。

机会是新增 **Runtime Dependency Circuit Breaker**：把外部依赖的运行时健康、熔断、降级、恢复探针和用户/管理员解释统一成一层共享服务。它不是新的 feature flag，也不是模型质量评测，而是运行中保护系统免受外部依赖故障扩散的可靠性层。

## 方案概述

新增一个 runtime-scoped 的 `DependencyCircuitBreaker`，按 dependency key 聚合失败趋势，并给调用方返回明确 decision：

- `closed`：正常调用。
- `open`：依赖短期不可用，直接拒绝或降级，不再打外部 provider。
- `half_open`：冷却到期，只允许少量 probe / low-risk call 验证恢复。
- `disabled`：由配置或人工 kill switch 明确关闭，不由自动恢复。

核心对象：

- `DependencyKey`：稳定依赖 ID，例如 `llm.provider.openrouter`、`llm.route.event.news_classifier`、`data.fmp.quote`、`search.tavily`、`channel.feishu.sink`、`sms.aliyun.send`、`hone_cloud.chat_api`。
- `CircuitPolicy`：失败窗口、最小样本数、错误率阈值、连续 timeout 阈值、open TTL、half-open probe 数、critical route 是否允许降级。
- `CircuitObservation`：一次调用结果，包含 dependency key、route、provider/model/endpoint、status、error class、latency、actor/surface 摘要、audit/task/log reference。
- `CircuitState`：当前状态、opened_at、expires_at、last_success_at、last_failure_at、reason code、影响中的 routes、最近样本摘要。
- `CircuitDecision`：调用前返回 `allow`、`allow_probe`、`degrade`、`reject_fast`、`queue_for_later`、`skip_optional_step`，并携带用户态和 operator reason。

第一版聚焦 4 类依赖：

1. LLM routes：auxiliary compaction、event news classifier、digest/pass routes、chat primary provider。
2. Market/search data：FMP、Tavily、RSS/full-text fetch。
3. Channel sinks：Feishu/Telegram/Discord/iMessage outbound send。
4. Public edge providers：Aliyun SMS/Captcha、Hone Cloud API upstream。

不要求一期做复杂分布式 consensus。local mode 可以用 SQLite/file-backed state；cloud mode 后续应迁到 PG，以避免多 worker 对同一依赖同时 half-open 探测。

## 用户体验变化

### 用户端

- Public `/chat` 如果主 LLM route 熔断，用户得到快速、稳定的短错误，而不是等待多次 timeout。
- 如果后台事实源熔断，回答可以明确说明“实时行情源暂不可用，本轮不会给出最新价判断”，而不是混入旧数据。
- 定时任务和主动推送在依赖故障时可进入 “delayed/degraded” 状态；恢复后下一窗口继续，不制造重复推送。
- API-key 用户可收到稳定 error code，例如 `dependency_circuit_open`、`llm_route_degraded`、`market_data_temporarily_unavailable`。

### 管理端

- 新增 `Dependency Health` 区块或页面：
  - 当前 open/half-open circuits。
  - 每个 dependency 最近 5/15/60 分钟成功率、timeout、429/402/5xx、p95 latency。
  - 影响的 route：public chat、cron、event-engine、digest、SMS login、channel send。
  - 当前降级动作和恢复倒计时。
- `LLM Audit` 增加 `circuit_state` / `dependency_key` filter，帮助区分模型本身失败、key pool 失败、熔断拒绝。
- `Task Health` 的失败摘要可以显示 `skipped_dependency_circuit_open`，避免把受控跳过误判为普通失败。
- `Notifications` 详情中可以显示“direct send 因 channel sink 熔断进入 digest / pending”。

### 桌面端

- Bundled mode 可在 dashboard/Settings 顶部展示本机依赖状态：LLM route、market data、channel sink 是否因近期失败被短暂熔断。
- Remote mode 只展示远端 backend 返回的 circuit state，不用本机探测远端 provider。
- 本地用户可以手动 reset 某个 circuit 以便确认配置修复，但 reset 需要明确提示“下一次调用会重新探测依赖”。

### 多渠道

- Direct chat 中主 runner 熔断时，channel adapter 发送短失败提示，不进入长等待。
- 主动推送中 channel sink 熔断时，不在每条事件上重复刷错误；写 delivery log，并将可保留内容进入 digest/pending。
- group chat 默认不广播内部依赖健康，只在 explicit request 受影响时回一条用户态解释。

## 技术方案

### 1. 依赖 key 与错误分类

先定义稳定的错误分类，避免用 provider 原始文本做策略：

```rust
pub enum DependencyErrorClass {
    Timeout,
    RateLimited,
    QuotaExhausted,
    AuthInvalid,
    ProviderUnavailable,
    BadRequest,
    ParseFailure,
    Transport,
    Unknown,
}
```

调用层把具体错误映射为 error class：

- OpenAI-compatible/OpenRouter：HTTP 429 -> `RateLimited`，402/insufficient credits -> `QuotaExhausted`，401/403 -> `AuthInvalid`，5xx/connection reset -> `ProviderUnavailable` / `Transport`。
- FMP/Tavily：key rejected、quota exhausted、timeout、empty provider response 分开。
- Channel sinks：auth failure、target not found、rate limit、upload failure、network timeout 分开。
- SMS/Captcha：captcha provider unavailable、SMS send rejected、verification failure 分开；用户输入错误不应触发 provider 熔断。

### 2. Circuit breaker 服务

建议在 `memory` 新增 `dependency_health` store，并在 `hone-core` 或 `hone-web-api` 定义纯类型：

```text
dependency_observations (
  id TEXT PRIMARY KEY,
  dependency_key TEXT NOT NULL,
  route_key TEXT,
  surface TEXT,
  status TEXT NOT NULL,
  error_class TEXT,
  latency_ms INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  created_at TEXT NOT NULL
)

dependency_circuit_states (
  dependency_key TEXT PRIMARY KEY,
  state TEXT NOT NULL,
  reason_code TEXT,
  opened_at TEXT,
  half_open_at TEXT,
  expires_at TEXT,
  last_success_at TEXT,
  last_failure_at TEXT,
  sample_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

服务接口：

```rust
pub trait DependencyCircuitBreaker {
    fn before_call(&self, key: &DependencyKey, context: &DependencyContext) -> CircuitDecision;
    fn record_success(&self, key: &DependencyKey, observation: CircuitObservation);
    fn record_failure(&self, key: &DependencyKey, observation: CircuitObservation);
    fn list_states(&self, filter: CircuitFilter) -> HoneResult<Vec<CircuitState>>;
}
```

策略评估可先使用滑动窗口查询，后续优化为 rollup。

### 3. 调用前 decision 接入点

第一阶段只接入高收益位置：

- `LlmResolver` 或外层 LLM call wrapper：调用前检查 `llm.provider.*` 和 `llm.route.*`；成功/失败写 observation，并继续写原有 LLM audit。
- `session_compactor`：auxiliary route 熔断时跳过本轮 compaction，记录 `deferred_dependency_open`，不阻塞用户主对话。
- event-engine LLM enrichment / classifier：熔断时跳过可选 enrichment，保留原始事件进入更保守 severity 或 digest。
- FMP/Tavily 工具和 event poller：熔断时短路，返回结构化 unavailable，而不是反复打 provider。
- channel sink：发送前检查 sink circuit；open 时写 delivery log `sink_circuit_open`，必要时进入 digest/pending。
- public SMS/Captcha：provider 熔断时快速返回“验证服务暂不可用”，不把 provider 故障计入用户失败次数。

### 4. 降级策略矩阵

不同 route 不应同样处理：

| route | circuit open 行为 |
| --- | --- |
| public chat primary LLM | reject fast，用户可稍后重试 |
| admin/local chat primary LLM | reject fast + operator detail |
| session compaction | defer，不消耗 quota，不写用户消息 |
| event news classifier | skip optional LLM，保守降级到 digest 或 keep low severity |
| global digest polish | 使用未润色文本或延后下一窗口 |
| market quote fetch | 返回 unavailable，不生成“最新价”结论 |
| channel direct sink | delivery log + pending/digest，不重复刷用户 |
| SMS send | public auth provider unavailable，不扣用户错误尝试 |

这些策略应是确定性配置/代码规则，不调用 LLM。

### 5. API、CLI 与前端

Admin API：

- `GET /api/dependencies/health`
- `GET /api/dependencies/observations?dependency=&state=&from=&to=`
- `POST /api/dependencies/:key/reset`
- `POST /api/dependencies/:key/probe`

CLI：

- `hone-cli doctor` 可显示 active circuit 摘要。
- `hone-cli dependencies list`
- `hone-cli dependencies reset <key>`

前端：

- `packages/app/src/pages/task-health.tsx` 显示受 circuit 影响的 task。
- `packages/app/src/pages/llm-audit.tsx` 增加 circuit/dependency filter。
- `packages/app/src/pages/notifications.tsx` 详情显示 delivery 被 circuit 降级的原因。
- Settings/Dashboard 增加 dependency health panel。

### 6. 与 cloud 模式兼容

local mode：

- SQLite/file state 即可。
- 删除 `data/runtime/` 后 circuit 状态清空，符合 runtime reset 语义。

cloud mode：

- circuit state 应迁到 PG，并带 worker fencing / lease，避免多 worker 同时 half-open probe。
- observation 可以按短 retention 存 PG，长期统计后续进入 usage/product event。
- Public API 只返回粗粒度 service status，不暴露 provider/key 详情。

## 实施步骤

### Phase 1: LLM route circuit MVP

- 定义 `DependencyKey`、`DependencyErrorClass`、`CircuitPolicy`、`CircuitState`、`CircuitDecision`。
- 新增 SQLite store 和 deterministic evaluator。
- 包装后台 LLM routes：news classifier、auxiliary compaction、digest polish。
- 将 LLM audit metadata 写入 dependency/circuit 信息。
- Admin 先通过 API/CLI 查看 active circuits。

### Phase 2: Event/data/channel 降级

- FMP/Tavily 工具和 event poller 接入 `before_call` / `record_*`。
- channel sinks 接入 sink circuit，delivery log 增加 `sink_circuit_open`。
- Task Health 和 Notifications 展示 circuit reason。
- 为 optional enrichment 定义保守降级规则。

### Phase 3: Public and desktop service status

- Public chat/SMS/API 返回稳定 user-facing reason code。
- Desktop dashboard 显示本地/远端 dependency health。
- Settings 增加 reset/probe 操作，要求 admin/desktop 权限。
- 与 Runtime Readiness Matrix 对接：readiness 可读取当前 active circuits，但不拥有熔断状态。

### Phase 4: Cloud PG and policy tuning

- cloud mode 将 circuit state 移到 PG，增加 lease/fencing。
- 增加 route-specific policy 配置，避免所有依赖共用一套阈值。
- 将 dependency failure 与 Usage Entitlement / Product Events 对齐，用于成本和 incident 分析。
- 支持 incident report：某个 provider outage 期间受影响的 actor、task、delivery、cost。

## 验证方式

### 自动化测试

- Circuit evaluator：
  - 连续 timeout 达阈值后 state 从 `closed` 到 `open`。
  - open TTL 到期后进入 `half_open`，只允许有限 probe。
  - half-open probe 成功后关闭；失败后重新 open 并延长冷却。
  - `BadRequest` 不触发 provider-level 熔断，`AuthInvalid` / `QuotaExhausted` 触发长 TTL。
- LLM wrapper：
  - HTTP 429/402/401/5xx/timeout 映射到稳定 error class。
  - circuit open 时不调用 mock provider，直接返回 `CircuitDecision`。
  - LLM audit 仍写原始失败，同时 metadata 有 dependency key。
- Event/channel：
  - news classifier 熔断时事件不丢失，只跳过 optional LLM upgrade。
  - channel sink 熔断时 delivery log status 为 `sink_circuit_open`，不重复发送。
  - SMS provider 熔断不增加用户验证码失败计数。

### 前端/API 测试

- `GET /api/dependencies/health` 不泄露 raw API key、手机号、provider 原始响应正文。
- Task Health 能区分 `failed` 与 `skipped_dependency_circuit_open`。
- LLM Audit filter 可按 dependency/circuit state 聚合失败。
- Public service status 只返回 coarse reason。

### 手工验收

- 用 mock OpenAI-compatible server 连续返回 429，触发 LLM route circuit；下一次 chat/background route 快速失败或降级，mock server 不再收到请求。
- 让 mock server 恢复 200，等待 TTL 后 probe 成功，circuit 自动关闭。
- 模拟 FMP timeout，event poller 记录受控失败，Task Health 显示 dependency reason。
- 模拟 Feishu sink 5xx，direct push 被记录为 sink circuit open，不刷用户重复错误。

### 指标

- Provider outage 期间外部失败请求数下降。
- 主对话 timeout p95 下降，快速失败比例上升。
- 后台任务因同一依赖重复失败的次数下降。
- 管理员能在 1 分钟内从 Dependency Health 定位 dependency、route、error class 和影响面。

## 风险与取舍

- **风险：误熔断导致可用依赖被短路。**  
  取舍：每个 policy 必须有最小样本数，低流量 route 使用连续失败而不是百分比；half-open 自动恢复。

- **风险：不同错误被错误归类。**  
  取舍：先覆盖常见 HTTP/status/transport 错误，未知错误只记录不立即 provider-level 熔断，避免误伤。

- **风险：降级掩盖真实问题。**  
  取舍：所有 circuit decision 必须进入 task/notification/audit metadata；管理端默认展示 active circuits。

- **风险：cloud 多 worker 状态竞争。**  
  取舍：local MVP 用 SQLite；cloud rollout 必须迁到 PG lease/fencing 后再对 hosted 多 worker 启用自动 half-open。

- **风险：用户看到更多服务状态文案。**  
  取舍：public 端只给短 reason code 和下一步，不展示 provider 细节；admin/desktop 才看完整状态。

- **不做：**
  - 不自动切换到未经评估的替代模型；模型替换仍由 Model Route Evaluation Lab 或人工配置治理。
  - 不替代 Product Rollout Kill Switch；人工关闭 feature 仍由 rollout registry 表达。
  - 不保存第三方完整响应正文；只保留摘要、error class、hash 和引用。
  - 不把投资输出安全判断放进 circuit breaker；内容安全仍归 Output Safety Gate。

## 与已有提案的差异

本轮查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点全文核对了 `readiness`、`model route`、`source health`、`kill switch`、`recovery`、`delivery`、`usage`、`run trace`、`provider`、`fallback`、`retry`、`circuit` 等相关主题。

- 不重复 `auto_p1_runtime_readiness_matrix.md`：Readiness 关注运行前配置和 capability 是否满足；本提案关注运行中依赖错误率升高后的自动熔断、降级和恢复。
- 不重复 `auto_p1_model-route-evaluation-lab.md`：Model Lab 评估候选模型质量和升级风险；本提案不比较模型质量，只处理已选 route 在运行中故障时如何保护系统。
- 不重复 `auto_p1_source-provenance-freshness.md`：Source Provenance 记录事实来源、新鲜度和 provider health 证据；本提案使用类似 observation，但核心是调用前 decision、熔断状态机和降级动作。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：Rollout/Kill Switch 是人工 feature 开关和灰度控制；本提案是自动依赖保护，可被 kill switch 禁用或覆盖，但不决定 feature 面向谁开放。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：Recovery Inbox 处理已经开始但未闭环的 run；本提案在调用前或调用失败趋势中尽早短路，减少产生 interrupted item。
- 不重复 `auto_p1_delivery_decision_loop.md`：Delivery Decision 解释事件为什么推、过滤或降级；本提案只处理外部 sink/provider 故障导致的受控跳过或 pending。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：Usage Entitlement 管权益和成本计量；本提案提供依赖故障导致的成本/失败归因，但不定义 plan 或额度。

查重结论：现有提案覆盖了运行前检查、模型评估、事实来源、人工 kill switch、运行恢复和用量权益，但没有覆盖“外部 provider / route / sink 在运行中进入故障风暴时，系统如何自动打开熔断、选择降级动作、半开探测并向用户和管理员解释”的可靠性层。因此本主题是新的、可落地的 P1 产品/架构提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。该任务属于自动化单次提案产出，不进入动态计划、不新增 handoff、无需归档计划页。若后续开始实现，应按动态计划准入标准新增或复用 `docs/current-plans/runtime-dependency-circuit-breaker.md`，并在新增 circuit state store、API、CLI、route wrapper、channel sink 降级或 cloud PG lease 后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要 decision/ADR。
