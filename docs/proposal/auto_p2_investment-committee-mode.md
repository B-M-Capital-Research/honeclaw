# Proposal: Investment Committee Mode for Multi-Perspective Research

status: proposed
priority: P2
created_at: 2026-05-09 23:03:43 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `config.example.yaml`
- `crates/hone-core/src/config/agent.rs`
- `crates/hone-channels/src/runners/multi_agent.rs`
- `crates/hone-channels/src/runners/types.rs`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `skills/stock_research/SKILL.md`
- `skills/deep_stock_research/SKILL.md`
- `skills/position_advice/SKILL.md`
- `skills/company_portrait/SKILL.md`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/research.tsx`
- `packages/app/src/components/research-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Hone 的产品定位已经很明确：不是顺着用户情绪说话的聊天机器人，而是投资纪律和长期研究的 co-pilot。当前仓库已经有几个支撑“专业投研工作台”的关键能力：

- `crates/hone-channels/src/prompt.rs` 注入全局金融约束，要求拒绝非金融问题、禁止直接荐股、操作建议必须转为条件和风险分析，并要求宏观/行业叙事保持逻辑连贯。
- `skills/stock_research/SKILL.md` 是统一个股研究入口，覆盖单公司研究、估值 framing 和 criteria-based screening，允许调用 `data_fetch`、`web_search` 与 `skill_tool`。
- `skills/position_advice/SKILL.md` 已经要求先读 portfolio，再围绕集中度、流动性、催化、下行场景给出风险管理型仓位建议。
- `skills/company_portrait/SKILL.md` 负责把长期 thesis、证据、风险、证伪条件沉淀到 actor sandbox 的公司画像。
- `deep_stock_research` 可以启动长耗时深度研究任务，但当前是 admin-only，产品语义更接近“外部研究任务启动器”。
- `config.example.yaml` 仍保留 `agent.multi_agent.search` 和 `agent.multi_agent.answer` 配置；`crates/hone-channels/src/runners/multi_agent.rs` 实现了两阶段执行：search 阶段决定是否用工具，answer 阶段消费 verified tool transcript 并生成最终回答。
- 现有提案已经覆盖了 safety gate、evidence review、research artifact library、trade discipline journal、cross-company thesis map、run trace workbench、response feedback 等治理层。

这些能力让 Hone 能做“一个 agent 的高质量研究回答”。但在用户真正做重要判断时，比如重仓公司财报后是否 thesis 改变、某行业链是否系统性恶化、两家公司谁的风险回报更好、当前仓位是否过于集中，单一最终回答仍然有一个天然缺陷：用户看不到反方观点、风险官视角、组合视角、长期画像视角之间的分歧，也无法知道模型是如何处理这些分歧后才给出最终结论。

业界 agent 产品正在从“单模型一次性回答”走向“可编排的多角色协作”：研究员收集资料，反方审稿人攻击假设，风险角色检查约束，主笔整合结论。Hone 已经有多阶段 runner 和投研技能基础，但还没有把这种协作显式产品化。

## 问题或机会

当前缺口主要不是“模型不会分析”，而是重大投资问题缺少结构化反方审查：

1. **用户看不到关键分歧。**  
   回答通常直接输出综合结论。即使正文里包含“风险”，用户也很难判断哪些是核心反证、哪些只是模板化 caveat。

2. **持仓视角和公司研究视角容易混在一段回答里。**  
   `stock_research` 擅长公司分析，`position_advice` 擅长组合风险，但重大问题往往需要两者并行：公司是否变差是一回事，用户当前仓位是否承受得起是另一回事。

3. **现有 multi-agent 是执行机制，不是用户体验。**  
   `multi_agent.rs` 的 search/answer handoff 是内部管线，且现有历史 proposal 主要讨论 skill runtime 与 multi-agent 语义对齐。用户不能选择“请用投委会模式审查这个 thesis”，管理端也没有记录每个角色的观点和合议结果。

4. **深度研究和日常聊天之间缺少中间档。**  
   Admin-only deep research 可能需要 1-2 小时；普通 chat 又可能太轻。用户经常需要的是 2-5 分钟内完成一次有反方、有风险、有组合影响的 committee-style review。

5. **留存价值没有完全表达。**  
   Hone 的差异化是“帮我变冷静”。如果产品能明确展示“多头观点 / 空头观点 / 风险官 / 组合影响 / 最终合议”，用户更容易感知它不是普通搜索问答，而是自己的投资研究委员会。

这是 P2：价值明确，但最好排在 safety gate、artifact library、trade discipline、runtime readiness 等基础治理之后。它依赖已有工具和 skill，不是核心可用性前置项，但会显著提升高价值投研体验、付费转化叙事和 agent 产品差异化。

## 方案概述

新增 **Investment Committee Mode**：一种可由用户、skill 或 playbook 显式启动的多视角研究模式。它不替换普通 chat，也不替代 deep research；它是介于普通问答和长耗时研究之间的结构化审查流程。

核心角色建议：

- `Lead Analyst`：整理问题、确认标的、调取基础数据和最新事实。
- `Bull Case Analyst`：提出支持 thesis 的最强论据、关键驱动和 upside 条件。
- `Bear Case Analyst`：提出反方论据、证伪条件、估值/竞争/周期风险。
- `Portfolio Risk Officer`：结合用户 portfolio、仓位集中度、时间 horizon、流动性和事件风险，判断风险暴露是否匹配。
- `Memory Steward`：读取相关 company portraits、跨公司 thesis map、历史证伪条件，检查本次结论是否与长期记忆一致。
- `Chair`：只负责整合分歧、列出需要补证据的点、给出非指令式行动框架。

第一版不需要真的并发启动多个外部 agent。可以先实现为一个受控的 committee prompt + structured internal sections，由现有 runner 顺序产出角色草稿，再由最终 answer stage 汇总。后续再把角色拆成真正的 sub-runner 或 parallel worker。

关键产物：

- `CommitteeSession`：一次 committee review 的元数据，包含 actor、session、topic、symbols、requested_mode、created_at、status。
- `CommitteeRoleNote`：每个角色的结构化观点，只保存摘要、findings、evidence refs 和 confidence，不保存完整 chain-of-thought。
- `CommitteeVerdict`：最终合议输出，包含共识、主要分歧、关键反证、组合影响、待补证据、建议的后续动作。
- `CommitteeArtifact`：可选的持久交付物，后续可与 research artifact library、company portrait event、trade discipline journal 关联。

边界：

- 不给用户直接买卖指令，沿用 `prompt.rs` 的金融约束。
- 不要求所有问题都走 committee；只有用户显式要求、问题高风险、或 skill/playbook 判定需要时才启动。
- 不把 committee note 自动写入 company profile；需要用户确认或后续 evidence/research handoff。
- 不解决 skill runtime multi-agent 机制对齐；本提案把它作为后续可选实现路径。

## 用户体验变化

### 用户端

- Public `/chat` 或 IM 私聊中，用户可以说：
  - “用投委会模式审查一下我对 NVDA 的 thesis”
  - “从多头、空头和仓位风险三个角度看 MU”
  - “这次财报后，我的长期主线有没有被证伪？”
- 普通回复变成明确分区：
  - 问题定义与使用的数据时间点。
  - 多头观点。
  - 空头/反方观点。
  - 组合与仓位风险。
  - 与已有画像/历史 thesis 的一致性。
  - Chair 合议：共识、分歧、待补证据、下一步可执行研究动作。
- 对移动端和 IM 端，默认输出短版 committee verdict；完整角色 notes 可在 Web/desktop 查看。
- 如果用户只是想快速问一句，系统不强行启动 committee，避免把轻量 chat 变重。

### 管理端

- 在 session detail 或未来 research artifact detail 中显示 committee run：
  - topic、symbols、角色数量、使用工具、耗时、是否写入 artifact。
  - 每个 role note 的短摘要和证据引用。
  - 最终 verdict 与用户反馈。
- 管理员可以把一次高质量 committee verdict 标记为 research handoff，进入 research artifact library 或 evidence review queue。
- 如果 `Run Trace Workbench` 后续落地，committee session 可以引用 trace id，而不是重复保存工具细节。

### 桌面端

- Desktop dashboard 可展示“最近委员会审查”和“待处理分歧”。
- 对本地 bundled 用户，committee artifacts 保存在本机数据目录，适合高隐私投研场景。
- Remote mode 只消费 backend 返回的 committee API，不运行本地额外 agent。

### 多渠道

- Feishu/Telegram/Discord 私聊可触发 committee short mode，回复保留简洁 verdict 和一个短 id，例如 `committee:NVDA:20260509`。
- 群聊中如果涉及个人持仓、成本、仓位或交易意图，沿用群聊隐私约束，引导到私聊。
- 长版 role notes 不直接推到 IM，避免刷屏和格式回归；多渠道只发摘要和 Web/desktop 查看入口。

## 技术方案

### 1. Committee mode 入口

新增轻量 skill 或扩展 `stock_research`：

- 新 skill 名称建议：`investment_committee`
- user-invocable: true
- allowed tools: `data_fetch`、`web_search`、`portfolio`、`local_list_files`、`local_search_files`、`local_read_file`、`skill_tool`
- 触发语义：committee、投委会、多视角、反方审查、bull/bear/risk review、thesis review。

第一版可以作为 skill 层 prompt，不需要先改 runner：

1. 解析 topic、symbols、用户是否要求组合视角。
2. 读取 portfolio 和相关 company portraits。
3. 调用 `data_fetch` / `web_search` 获取当前事实。
4. 产出角色 notes 的结构化 JSON block。
5. 生成用户可见 verdict。

后续如果 `skill-runtime-multi-agent-alignment.md` 的 active skill state 和 stage handoff 落地，可让 committee skill 显式驱动多阶段或并行角色执行。

### 2. 结构化 committee artifact

建议在 `memory` 或 `crates/hone-web-api` 后端层新增 actor-scoped 存储，第一版可用 SQLite：

```text
committee_sessions (
  committee_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_session_id TEXT,
  topic TEXT NOT NULL,
  symbols_json TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  completed_at TEXT
)

committee_role_notes (
  note_id TEXT PRIMARY KEY,
  committee_id TEXT NOT NULL,
  role TEXT NOT NULL,
  summary TEXT NOT NULL,
  findings_json TEXT NOT NULL,
  evidence_refs_json TEXT NOT NULL,
  confidence REAL,
  created_at TEXT NOT NULL
)

committee_verdicts (
  verdict_id TEXT PRIMARY KEY,
  committee_id TEXT NOT NULL,
  consensus TEXT NOT NULL,
  disagreements_json TEXT NOT NULL,
  open_questions_json TEXT NOT NULL,
  portfolio_implications_json TEXT,
  followup_actions_json TEXT NOT NULL,
  user_visible_markdown TEXT NOT NULL,
  created_at TEXT NOT NULL
)
```

不保存完整隐藏推理，只保存用户可审计的角色摘要、证据引用和最终 verdict。这样能降低隐私和 chain-of-thought 风险，也方便后续导出/删除。

### 3. Runner 集成路径

一期：skill-prompt orchestration。

- 不新增 runner 类型。
- committee skill 把角色输出约束为 structured sections。
- finalizer 和 safety gate 继续处理最终用户可见文本。

二期：复用现有 `multi_agent`。

- Search stage 负责事实收集和本地记忆读取。
- Answer stage 生成 role notes 和 verdict。
- `multi_agent.rs` 的 handoff 增加 committee-specific schema，避免只传一段自然语言 working note。

三期：真正多角色执行。

- 引入 `CommitteeRunner` 或 `AgentRunner` 的 stage plan。
- 每个 role 使用同一个工具 transcript 或受限工具 allowlist。
- 角色可并发，但最终 Chair 必须串行整合，并经过 safety gate。

### 4. API 与前端

新增 API：

- `GET /api/committee-sessions?actor=&symbol=&status=`
- `GET /api/committee-sessions/:id`
- `POST /api/committee-sessions/:id/handoff`
- `GET /api/public/committee-sessions`
- `GET /api/public/committee-sessions/:id`

Public 端只允许当前 web actor 读取自己的 committee sessions；admin 可按 actor 查。

前端落点：

- `packages/app/src/pages/chat.tsx`：当回答中含 committee id，可渲染一个 compact summary card。
- `packages/app/src/pages/public-portfolio.tsx`：按 ticker 展示最近 committee verdict。
- `packages/app/src/pages/research.tsx` 或未来 artifact library：把 committee session 作为轻量研究交付物的一类。
- `packages/app/src/components/research-detail.tsx`：复用 role-note timeline 展示结构。

### 5. 与画像、证据和纪律日志的关系

- Company portrait：committee verdict 可建议“写入画像”，但不自动修改 `profile.md`。
- Evidence review：如果 committee 发现某条新证据可能改变 thesis，可生成 review item。
- Trade discipline journal：如果 committee 是围绕用户明确操作意图触发，最终 verdict 可作为 `TradeDecisionDraft` 的 evidence input，但不是交易决定。
- Research artifact library：committee session 是轻量 artifact；长耗时 deep research 仍作为完整报告 artifact。

## 实施步骤

### Phase 1: Skill-only MVP

- 新增 `skills/investment_committee/SKILL.md`，定义触发、角色、输出结构和边界。
- 复用现有 `data_fetch`、`web_search`、`portfolio`、本地文件读取工具。
- 在 skill prompt 中要求输出 `committee_id` 占位和 short verdict，但不新增持久化。
- 增加手工 regression：单公司 thesis review、组合风险 review、群聊隐私拒绝。

### Phase 2: Committee artifact 存储

- 新增 committee session / role note / verdict 类型和 actor-scoped SQLite 存储。
- 增加 tool 或 API，让 skill 在最终输出前写入 committee artifact。
- Web API 支持 admin/public 读取当前 actor 的 committee sessions。
- 增加 actor 隔离、空 role note、重复写入幂等测试。

### Phase 3: UI 与 handoff

- Chat 页面渲染 committee compact card。
- Public `/portfolio` 按 ticker 展示最近 verdict 和 open questions。
- Admin research/session detail 展示 role notes。
- 支持从 committee verdict 创建 research handoff、evidence review item 或 trade discipline draft。

### Phase 4: Multi-agent runner 优化

- 在 `multi_agent` search -> answer handoff 中增加 committee schema。
- 如果 role 执行拆分为多 stage，明确每个 role 的 allowed tools 和 token budget。
- 将 role notes 与 run trace / LLM audit 关联，便于排查成本和质量。

## 验证方式

- Skill regression：
  - “用投委会模式分析 NVDA”必须输出多头、空头、组合风险、记忆一致性和 Chair verdict。
  - 用户只问普通寒暄时不得触发 committee。
  - 群聊中要求记录个人持仓时必须引导私聊。
- 工具和存储测试：
  - committee artifact 按 `ActorIdentity` 隔离。
  - 同一 session 重试不会生成重复 role notes，或能通过 idempotency key 合并。
  - 不保存完整隐藏推理，只保存摘要和证据 refs。
- API 测试：
  - Public 用户只能读自己的 committee sessions。
  - Admin 查询支持 actor/symbol/status filter。
  - handoff 动作不会直接修改 company profile。
- 前端测试：
  - Chat compact card 能处理缺失 role、长 symbol、移动端窄屏。
  - Public portfolio 在没有 committee session 时保持现有空态。
- 质量验证：
  - 抽取 10 个真实高价值问题，对比普通 stock_research 与 committee mode，人工标注反方观点覆盖率、风险具体度、可执行 follow-up 数量。
  - 统计平均耗时和 token 成本，确认 committee mode 不应成为默认轻量 chat 路径。

## 风险与取舍

- **风险：输出变长、成本变高。**  
  取舍：committee mode 必须显式触发或由高风险场景触发，默认走 short mode；IM 只发摘要。

- **风险：角色扮演流于形式。**  
  取舍：每个 role 必须产出结构化 finding、evidence refs 和 confidence；没有证据时必须标为 open question，而不是填模板。

- **风险：和 deep research / research artifact 重叠。**  
  取舍：committee 是分钟级轻量审查，deep research 是小时级完整报告；artifact library 可统一展示两者，但不混淆产物类型。

- **风险：模型把 Chair verdict 写成交易指令。**  
  取舍：沿用 `prompt.rs` 金融边界，并在 safety gate 落地后把 committee verdict 作为高风险输出类型检查。

- **风险：多角色 note 可能暴露不该保存的推理。**  
  取舍：只保存面向用户的摘要、证据、分歧和 open questions，不保存完整 chain-of-thought。

- **风险：早期 runner 不支持真正并行。**  
  取舍：第一版用 skill prompt 顺序模拟角色，先验证产品价值；并行和专用 runner 留到后续。

## 与已有提案的差异

- 与 `auto_p0_investment_output_safety_gate.md` 不重复：safety gate 判断最终输出是否可送达；本提案定义用户主动启动的多视角投研体验和 artifact。
- 与 `auto_p1_trade_discipline_journal.md` 不重复：discipline journal 记录用户准备采取的交易动作和复盘；committee mode 可以在没有具体交易动作时审查 thesis、行业或组合风险。
- 与 `auto_p1_research_artifact_library.md` 不重复：research artifact library 解决报告留存与 handoff；committee mode 产生一种轻量、多角色 verdict，可作为 artifact 输入。
- 与 `auto_p1_evidence_review_queue.md` 不重复：evidence queue 处理单条事件是否需要复盘；committee mode 是一次主动审查，可以发现需要进入 evidence queue 的 open item。
- 与 `auto_p1_cross-company-thesis-map.md` 不重复：thesis map 维护跨公司长期主线一致性；committee mode 可读取它，但输出的是一次具体问题的多角色审查。
- 与 `auto_p1_response-feedback-learning-loop.md` 不重复：feedback loop 收集用户对回答质量的评价；committee mode 是回答生成前/生成中的结构化协作模式。
- 与 `auto_p1_run_trace_workbench.md` 不重复：trace workbench 排查执行过程；committee mode 是用户可见研究产品形态，trace 只是后续观测支撑。
- 与 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不重复：该历史提案讨论 skill 与 multi-agent 执行语义；本提案面向投资研究用户体验，第一版可在现有 skill 层落地，不要求先改 runner。
