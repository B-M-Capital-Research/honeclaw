# Proposal: Signal Outcome Calibration Ledger for Investment Judgement Quality

status: proposed
priority: P1
created_at: 2026-05-26 20:07:50 +0800
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
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_notification-policy-backtest.md`
- `docs/proposal/auto_p1_mainline-distill-ledger.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/runners/multi_agent.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-event-engine/src/global_digest/curator.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `memory/src/company_profile/types.rs`
- `memory/src/portfolio.rs`
- `memory/src/session.rs`
- `memory/src/llm_audit.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `packages/app/src/pages/notifications-model.ts`

## 背景与现状

Hone 的核心定位不是让模型更会聊天，而是帮助投资者建立长期、克制、可复盘的判断体系。当前仓库已经具备几条关键基础：

- `crates/hone-channels/src/prompt.rs` 在所有渠道注入金融边界：不直接荐股、区分噪音和投资主线变化、时间敏感问题要使用当前时间并说明数据时点。
- `crates/hone-channels/src/agent_session/mod.rs` 统一对话运行、持久化、quota、compaction 和 runner 事件，`memory/src/session.rs` 保存会话消息与元数据。
- `memory/src/llm_audit.rs` 已记录 LLM 调用审计，能回答一次运行用了什么模型、耗时和错误。
- `memory/src/company_profile/types.rs` 和 company portrait 存储保存长期 thesis、风险、事件影响和研究轨迹。
- `crates/hone-event-engine/src/global_digest/curator.rs` 已把候选事件分成 `mainline_aligned`、`mainline_counter`、`macro_floor`，并输出 `mainline_relation`，说明系统已经在主动判断“这条信息是否印证或证伪用户主线”。
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs` 与 public `/portfolio` 把画像蒸馏成 `mainline_by_ticker` 和 `mainline_style`，供 digest personalize 使用。
- 现有提案已覆盖输出安全、用户反馈、通知策略回测、主线蒸馏版本、证据复盘和交易纪律日志。

这些能力让 Hone 能记录“当时说了什么”“为什么推送”“用户是否觉得有用”“某次主线蒸馏如何变化”。但系统仍缺少一个更根本的质量闭环：Hone 当时做出的投资判断、信号判断、噪音判断，后来是否被事实验证。

例如：

- Digest 把某条新闻标成 `mainline_counter`，后来公司基本面或股价路径是否确实出现了主线层面的变化？
- 回答中说“这是短期噪音，不足以推翻长期 thesis”，后来是否证明它只是噪音，还是被低估的早期风险？
- Earnings quality review 判定为 `mixed_positive`，后续财报电话会、指引修正或市场反应是否支持这个判断？
- 用户问某个宏观冲击，Hone 要求区分短期波动和投资主线，后续是否应该复盘这次分类是否过度自信？

当前这些“判断质量”只能靠人工读历史消息、通知记录、公司画像和后续新闻来回忆。对一个投资纪律产品来说，这会让最重要的能力无法量化：Hone 是否真的越来越善于识别主线、反证和噪音。

## 问题或机会

这是 P1，因为它直接影响产品信任、留存、模型/提示升级、事件引擎质量和商业化叙事，但第一版可以作为只追加的校准层落地，不需要改交易执行或通知路由主路径。

当前缺口主要有五类：

1. **判断没有 outcome anchor。**  
   会话、digest、画像事件和通知都可能包含判断，但没有一等对象记录“这是一个可复盘判断”，也没有 due date、验证指标、后续事实和最终校准结果。

2. **用户反馈不等于事实校准。**  
   `Response Feedback Learning Loop` 可以知道用户觉得回答有无帮助，但用户满意不代表判断正确；用户不满意也不代表判断错误。投资助手需要同时追踪主观体验和客观后验。

3. **通知回测不覆盖真实结果。**  
   `Notification Policy Backtest Lab` 能评估策略改动会多推或少推什么，但它不回答“当时被推送或过滤的信号后来是否真的重要”。

4. **模型升级缺少业务质量标签。**  
   `Model Route Evaluation Lab` 可以评估模型路线，但如果没有长期的 `signal -> outcome` 样本，评测容易停留在格式、成本、速度或人工主观分数，难以衡量投资判断校准。

5. **产品价值难以证明。**  
   Hone 的差异化应是“减少噪音、识别反证、保护纪律”。如果没有校准台账，管理端很难展示过去 30 天哪些判断被验证、哪些过度自信、哪些被标记为学习样本。

机会是新增一个 **Signal Outcome Calibration Ledger**：把系统在回答、digest、事件引擎和画像复盘中产生的关键判断抽象为可追踪 claim，按预设窗口收集后续事实，最终生成校准结果和改进队列。

## 方案概述

新增 actor-scoped 的判断校准层，只记录高价值、可复盘、有限期的投资判断，不把每句话都变成任务。

核心对象：

1. `JudgementClaim`
   一条可复盘判断。包含 actor、source、session_id 或 event_id、ticker/symbols、claim_type、claim_text、confidence、time_horizon、created_at、due_at、evidence_refs、mainline_relation、model/runner。

2. `OutcomeProbe`
   一个后续事实检查计划。描述用什么证据判断 claim 是否被支持，例如后续财报、SEC filing、价格区间、分析师调整、公司画像事件、用户复盘输入、人工 review。

3. `OutcomeObservation`
   到期或触发后收集到的事实快照。可以来自 event-engine 已入库事件、tool result snapshot、company profile event、public source URL、用户手工备注或管理员补充。

4. `CalibrationResult`
   对 claim 的后验判断：`supported`、`partially_supported`、`refuted`、`too_early`、`not_verifiable`、`bad_claim_shape`。同时记录 error mode，例如 `overconfident_noise_call`、`missed_mainline_counter`、`stale_data_anchor`、`weak_evidence`、`unclear_time_horizon`。

5. `CalibrationReviewItem`
   管理端和后续 eval 使用的复盘项。高价值负样本可以转成 bug、eval case、prompt/routing 改进、company portrait update 或 model route benchmark。

第一版只做“声明、到期、记录、人工/半自动校准”，不自动改变 live prompt、通知策略或画像。后续再把高质量校准样本接入模型评测和 safety/output gate。

## 用户体验变化

### 用户端

- Public `/portfolio` 或 `/me` 增加“判断复盘”轻量入口：
  - 最近被跟踪的判断，例如“MU 财报后毛利率改善是否强化主线”“某条监管新闻是否构成主线证伪”。
  - 到期状态：待观察、已验证、部分验证、被证伪、不可验证。
  - 每条只展示短摘要、证据时间窗和结果，不展示运维 trace。
- 当用户问“你上次说这只是噪音，现在怎么看”时，agent 可以检索 calibration ledger，回答：
  - 当时的判断是什么。
  - 当时依据是什么。
  - 后来发生了什么。
  - 这次应归类为支持、证伪还是仍需观察。
- 普通聊天不弹出复杂表单。只有高影响判断才提示“已加入 30 天后复盘”，并允许用户关闭。

### 管理端

- 新增 `Calibration` 页面或放入 `Research` / `Notifications` 高级 tab：
  - 按 actor、ticker、claim_type、source、result、error_mode、model/runner 过滤。
  - 展开查看原始回答/digest、证据引用、后续 observation、校准结论。
  - 将 `refuted` 或 `bad_claim_shape` 标记为 eval candidate。
- `Notifications` 详情可显示：该 digest item 是否创建了 claim，后续 outcome 如何。
- `UserMainlineView` 可显示每个 ticker 最近的判断校准：哪些主线反证曾被错过，哪些噪音判断被验证。
- 管理端指标：
  - verifiable claim 比例。
  - supported / refuted / not_verifiable 分布。
  - 按模型、runner、source、claim_type 的 refuted 率。
  - 最常见 error modes。

### 桌面端

- Desktop bundled/remote 复用 Web API 和同一页面。
- 本地单用户默认只显示自己的 calibration ledger，适合做个人投资复盘。
- Dashboard 可以显示：“3 条判断本周到期复盘”“1 条上次噪音判断被后续证伪”。

### 多渠道

- Feishu / Telegram / Discord 私聊支持自然语言：
  - “复盘你上次对 MU 的判断”
  - “把这个判断 30 天后回看”
  - “这条推送后来有没有验证”
- 群聊里只返回短摘要，不展开用户私有持仓、成本和画像细节。
- 对主动到期复盘，默认进入 digest 或私聊提醒，不在群聊主动公开敏感结论。

## 技术方案

### 1. 新增存储模块

在 `memory` 新增 `signal_calibration` 模块，使用 SQLite。第一版只追加，不迁移旧 session 或 event 数据。

建议表：

```text
judgement_claims (
  claim_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_kind TEXT NOT NULL,
  source_id TEXT,
  session_id TEXT,
  event_id TEXT,
  ticker_symbols_json TEXT NOT NULL,
  claim_type TEXT NOT NULL,
  claim_text TEXT NOT NULL,
  confidence REAL,
  time_horizon TEXT NOT NULL,
  due_at TEXT,
  evidence_refs_json TEXT NOT NULL,
  mainline_relation TEXT,
  model_ref TEXT,
  runner_ref TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

outcome_observations (
  observation_id TEXT PRIMARY KEY,
  claim_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  observed_at TEXT NOT NULL,
  observation_text TEXT NOT NULL,
  evidence_refs_json TEXT NOT NULL,
  data_snapshot_json TEXT NOT NULL,
  created_at TEXT NOT NULL
)

calibration_results (
  result_id TEXT PRIMARY KEY,
  claim_id TEXT NOT NULL,
  result TEXT NOT NULL,
  confidence REAL,
  error_modes_json TEXT NOT NULL,
  reviewer_kind TEXT NOT NULL,
  rationale TEXT NOT NULL,
  created_at TEXT NOT NULL
)
```

`source_kind` examples:

- `chat_answer`
- `public_chat_completion_api`
- `global_digest_personalize`
- `event_engine_earnings_quality`
- `company_profile_event`
- `scheduled_task_answer`

`claim_type` examples:

- `noise_vs_mainline`
- `mainline_aligned`
- `mainline_counter`
- `earnings_quality`
- `macro_impact`
- `price_move_interpretation`
- `source_reliability`
- `portfolio_risk_signal`

### 2. Claim creation boundaries

不要让模型每次回答都自由写 claim。第一版采用明确入口：

- Digest personalize 产出 `mainline_counter` 或高置信 `mainline_aligned` 时，可以自动生成 claim candidate。
- Earnings quality review / SEC enrichment 中有结构化 judgement 时，可生成 claim candidate。
- Chat answer 只有在以下情况生成：
  - 用户要求“记住这个判断 / 之后复盘”。
  - 回答包含明确时间窗和可验证判断。
  - 模型通过新工具 `signal_calibration(create_claim, ...)` 显式写入。
- 管理员可以从 session / notification detail 手工创建 claim。

第一版应把 claim candidate 与 confirmed claim 分开：自动来源可先进入 `pending`，只有高价值类型或用户确认后才变 `active`。

### 3. Outcome collection

后续事实可以分层收集：

- Deterministic:
  - 从 `hone-event-engine` 的 event store 查询同 ticker 后续 earnings、SEC、price band、corp action、analyst grade、news critical。
  - 从 company profile `events/*.md` 引用用户已确认的长期事件。
- Tool-assisted:
  - 到期时由 scheduled task 或 admin action 触发一次受控工具查询，保存 snapshot。
  - 使用已有 `data_fetch` / `web_search`，但保存 query、timestamp、source URL 和数据时点。
- Human:
  - 用户或管理员写一句 observation，附上链接或画像事件。

第一版不需要全自动判定所有 claim。可以先让系统聚合候选 evidence，再由管理员或用户确认 result。

### 4. Agent tool and API

新增 `signal_calibration` tool，操作：

- `create_claim`
- `list_claims`
- `get_claim`
- `add_observation`
- `record_result`
- `due_claims`

Web API：

- `GET /api/calibration/claims?actor=&symbol=&status=&result=`
- `GET /api/calibration/claims/:claim_id`
- `POST /api/calibration/claims`
- `POST /api/calibration/claims/:claim_id/observations`
- `POST /api/calibration/claims/:claim_id/results`
- `GET /api/calibration/summary`

Public API 只限当前登录 actor：

- `GET /api/public/calibration/claims`
- `GET /api/public/calibration/claims/:claim_id`
- `POST /api/public/calibration/claims/:claim_id/observations`

兼容策略：

- 旧 session、旧 digest、旧画像不补建 claim。
- 新 claim 只引用 source id 和摘要，不复制完整私密会话正文。
- 运行时 prompt 不读取全部 calibration ledger，只在用户明确要求复盘、或后续设计 actor summary 时读取小摘要。

### 5. 到期复盘调度

复用现有 cron / scheduler 思路，但保持轻量：

- 每日检查 due claims，生成 `calibration_due` task run。
- 到期只创建 review item，不默认发大量主动消息。
- 对用户明确要求的 claim，可以创建私聊提醒。
- 对 `not_verifiable` 或 `bad_claim_shape`，优先进入管理端质量队列，不打扰用户。

### 6. 与 eval 和产品质量的连接

校准结果应作为后续评测资产：

- `refuted + overconfident_noise_call` 可以转成 prompt/runner eval。
- `not_verifiable + unclear_time_horizon` 说明回答应该更明确时间窗和可验证条件。
- `missed_mainline_counter` 可进入 evidence review 或 company portrait update。
- 按模型/runner 聚合校准结果，为 model route 选择提供真实业务标签。

第一版只输出 `summary.json` / API 聚合，不自动调整模型路线。

## 实施步骤

### Phase 1: Ledger and manual workflow

- 新增 `memory::signal_calibration` SQLite 存储、类型和单元测试。
- 新增 admin claim list/detail API。
- 新增最小管理端页面，支持手工创建 claim、补 observation、记录 result。
- 从 notification/session detail 手工 deep link 到 claim creation。

### Phase 2: Agent tool and user-facing recall

- 新增 `signal_calibration` tool，并纳入 tool registry。
- 用户明确要求“之后复盘 / 记住这个判断”时创建 claim。
- 用户问“上次判断怎么样”时，agent 先读取 claim，再回答。
- Public `/portfolio` 或 `/me` 展示当前用户的 active/due/recent results。

### Phase 3: Event-engine candidate integration

- Digest personalize 对 `mainline_counter` 和高置信 `mainline_aligned` 生成 claim candidate。
- Earnings quality review 和 SEC enrichment 可生成结构化 candidate。
- 到期任务从 event store 收集 deterministic observations。
- 管理端支持批量确认或关闭低价值 candidate。

### Phase 4: Calibration metrics and eval export

- 增加 summary API 和管理端图表。
- 将 `refuted` / `bad_claim_shape` 结果导出为 eval fixture candidate。
- 与 Model Route Evaluation Lab 对接，按 claim_type 比较模型路线。
- 与 Response Feedback Learning Loop 对接，区分“用户不满意但后验支持”和“用户满意但后验证伪”的样本。

## 验证方式

### 自动化测试

- Rust 单元测试：
  - claim CRUD、status transition、actor isolation。
  - due claim 查询按 `due_at` 和 status 正确过滤。
  - observation/result 只能写入同 actor claim。
  - `not_verifiable` 与 `bad_claim_shape` 等结果枚举可序列化/反序列化。
- API 测试：
  - public 只能读取当前 cookie actor 的 claims。
  - admin 可按 actor/ticker/status 查询。
  - claim detail 不返回未授权 session 原文。
- Tool 测试：
  - 缺 `claim_text`、缺时间窗、缺 ticker 的高风险 claim 创建被拒绝或标 `bad_claim_shape_candidate`。
  - `list_claims` 能返回 compact summary，不把长 evidence 泄露到 prompt。

### 前端测试

- claim list 空态、active、due、resulted 三种状态稳定渲染。
- public 页面移动端不溢出，长 claim text 截断后可展开。
- admin summary filter 对 result/error_mode 过滤正确。

### 手工验收

- 创建一个 `noise_vs_mainline` claim，due date 为明天；到期查询能出现在 review 队列。
- 给 claim 添加一条 event-store observation 和一条人工 observation；记录 `partially_supported` result。
- 在聊天中问“复盘这条判断”，agent 能引用 claim、observation 和 result，而不是重新编造历史。
- 从 digest detail 创建 `mainline_counter` claim，不影响原有通知投递。

### 指标

- active claims 数量和 due completion rate。
- verifiable claim 比例。
- supported / refuted / not_verifiable 分布。
- `unclear_time_horizon` error mode 占比是否下降。
- 按 runner/model 的 refuted 率和 bad-shape 率。

## 风险与取舍

- 风险：把投资判断“打分”可能被误读为收益承诺。取舍：结果只评价当时判断与后续事实是否一致，不评价用户是否该交易，也不计算保证收益。
- 风险：自动抽取 claim 会产生大量低价值复盘项。取舍：第一版只支持手工/显式工具写入，event-engine 自动生成先进入 candidate，并设数量 cap。
- 风险：后续事实归因复杂。取舍：第一版允许 `partially_supported`、`too_early`、`not_verifiable`，不强行二分对错。
- 风险：模型可能为提高校准率而变得含糊。取舍：同时统计 `bad_claim_shape` 和 `unclear_time_horizon`，鼓励明确但不过度自信的判断。
- 风险：隐私敏感。取舍：actor-scoped，默认只存摘要和 evidence refs，不复制完整会话或私密 portfolio 内容；群聊不主动公开私人 calibration。
- 风险：维护面增加。取舍：先做独立 ledger 和手工 workflow，不改 event router、prompt 主路径或 NotificationPrefs 生效逻辑。
- 不做：不自动交易，不用结果反向承诺模型能力，不把校准结果自动写入 company portrait，不自动调整通知策略，不替代用户反馈或安全门禁。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下全部现有提案，并重点检查了 feedback、trace、notification、mainline、evidence、model eval、trade journal、portfolio、source provenance 相关主题。

- 不重复 `auto_p1_response-feedback-learning-loop.md`：该提案收集用户对回答是否有用的主观反馈；本提案记录投资判断与后续事实的客观/半客观校准。
- 不重复 `auto_p1_notification-policy-backtest.md`：该提案在策略上线前反事实回放通知路由结果；本提案在信号发出后追踪其后验事实结果。
- 不重复 `auto_p1_mainline-distill-ledger.md`：该提案保护主线蒸馏版本、diff 和回滚；本提案评估“基于主线做出的判断”后来是否被支持。
- 不重复 `auto_p1_evidence_review_queue.md`：该提案把外部事件转成待复盘证据；本提案把系统已做出的判断转成可到期验证的 claim，并记录 outcome。
- 不重复 `auto_p1_trade_discipline_journal.md`：该提案记录用户的投资动作意图、纪律检查和事后复盘；本提案记录 Hone 自身对信号/主线/噪音的判断质量，不追踪用户是否执行交易。
- 不重复 `auto_p1_model-route-evaluation-lab.md`：该提案评估模型路线；本提案提供真实业务 outcome 标签，后续可作为模型评测输入。
- 不重复 `auto_p0_investment_output_safety_gate.md`：安全门禁决定输出是否允许送达；本提案在输出送达后，长期复盘其中的可验证判断。
- 不重复 `auto_p1_delivery_decision_loop.md`：delivery loop 解释单条通知为什么推或不推；本提案解释当时认为重要或不重要的信号后来是否真的重要。
- 不重复 `auto_p1_source-provenance-freshness.md`：source provenance 追踪事实来源和新鲜度；本提案追踪判断与后续事实的校准结果。

差异结论：现有提案已经覆盖运行、反馈、安全、通知策略、主线版本、证据队列和交易纪律，但还没有覆盖“投资判断本身的后验校准”。本提案填补的是 Hone 从“会解释和记录”走向“能学习自己哪些判断可靠”的关键产品架构缺口。
