# Proposal: Trade Discipline Journal and Review Loop

status: proposed
priority: P1
created_at: 2026-05-06 23:03:34 CST
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
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `crates/hone-channels/src/prompt.rs`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `skills/portfolio_management/SKILL.md`
- `skills/position_advice/SKILL.md`
- `skills/company_portrait/SKILL.md`
- `skills/scheduled_task/SKILL.md`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/types.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `crates/hone-event-engine/src/global_digest/audience.rs`

## 背景与现状

Hone 的公开定位不是普通聊天助手，而是“投资纪律的无情捍卫者”。当前仓库已经把这条定位落进多个能力层：

- `README.md` 明确强调 Hone 应帮助用户保持理性、监控持仓、执行投资纪律，并抵抗情绪化交易冲动。
- `crates/hone-channels/src/prompt.rs` 的全局金融约束禁止直接荐股，要求用户寻求操作建议时改为分析买点、卖点、触发条件、失效条件、仓位与风险，并提醒用户不要未经独立思考照做。
- `skills/position_advice/SKILL.md` 已提供风险管理型仓位建议流程：先取 `portfolio`，再结合市场、个股风险、集中度、流动性、催化和下行场景，输出“减少、维持、重构暴露前应审视的问题”。
- `skills/portfolio_management/SKILL.md` 与 `crates/hone-tools/src/portfolio_tool.rs` 能写入持仓、关注、期权、持有期限、策略备注和 notes；关注标的与真实持仓都会进入主动推送链路。
- `memory/src/portfolio.rs` 以 actor 为边界持久化 holdings/watchlist，并保留 `holding_horizon`、`strategy_notes`、`notes` 等字段；但这些字段只是当前状态备注，不是决策过程记录。
- 公司画像以 `company_profiles/<profile_id>/profile.md` 和 `events/*.md` 存长期 thesis、证据、风险和证伪条件；event-engine 会把画像蒸馏成 per-actor thesis，用于 digest 和通知个性化。
- Public `/portfolio` 展示投资主线、画像和蒸馏状态，管理端 `PortfolioDetail` 支持维护持仓与关注。
- `scheduled_task` skill 和 cron job 可以创建盘前、盘后或条件型复盘任务。

这些能力已经能回答“我现在持有什么”“这个标的长期 thesis 是什么”“最近有什么值得关注”“我该如何看风险”。但系统还没有一个一等对象承接用户真正容易出错的时刻：用户准备买入、卖出、加仓、减仓、换仓、上杠杆、卖期权或追逐短期催化时，Hone 只能在当轮回答里提醒风险，无法把这次决策的理由、证据、反证、仓位假设、冷静期和后续复盘留下来。

当前 `portfolio` 是“状态表”，company portrait 是“长期研究资产”，cron 是“未来提醒”，notification/event proposals 处理“市场证据是否需要复盘”。它们之间缺一个“用户自己的投资动作决策记录”层。

## 问题或机会

这是 P1 级产品/架构机会，因为 Hone 的核心差异化不是更快给出买卖答案，而是在用户最容易冲动的时候迫使其完成高质量决策流程，并在事后复盘这套流程是否有效。

当前缺口主要体现在六个方面：

1. 操作建议只存在于回答文本里。
   `position_advice` 能给出风险管理问题，但用户关闭聊天后，这次“为什么想操作、反对证据是什么、什么情况算错、什么时候复盘”没有稳定记录。

2. `portfolio` 写入缺少决策来源。
   用户说“我买了 100 股 AAPL”时，系统能更新持仓；但无法区分这是长期 thesis 加仓、短线事件交易、止损失败后的补仓、还是单纯补录历史持仓。后续 digest 和画像只能看到持仓状态，看不到决策质量。

3. 公司画像与真实行动之间没有约束。
   画像可以写“若毛利率恶化则 thesis 失效”，但用户真的准备在毛利率恶化后加仓时，系统没有一个结构化机制把“这次动作是否违反旧证伪条件”拉出来审查。

4. 复盘任务依赖用户临时想起。
   用户可以创建“每日仓位复盘”或“财报后复盘”，但它不是围绕某次具体决策自动生成的 follow-up。系统无法在 7 天、30 天、财报后或触发失效条件时提醒用户回看当时的理由是否仍成立。

5. 多渠道和桌面体验不成闭环。
   IM 端适合用户在情绪高点发一句“我要不要现在追”；Web/desktop 适合做完整复盘和查看历史决策。当前没有统一 decision id 把这两个场景接起来。

6. 商业化价值没有从“更多对话”转成“更少错误”。
   对投资纪律产品而言，最有说服力的留存体验不是回答次数，而是用户能看到 Hone 帮自己避免了多少冲动操作、哪些决策因为事前 checklist 质量更高而更容易复盘。

机会是：不用改交易执行，也不用接券商账户，就可以先做一个 actor-scoped 的 Trade Discipline Journal。它只记录和审查用户自己表达的操作意图，不代客下单，不给确定性买卖指令；第一版可以复用 `portfolio`、company portrait、scheduled task、public portfolio 和现有金融约束。

## 方案概述

新增 `TradeDisciplineJournal` 产品层：当用户表达明确或隐含的投资动作意图时，Hone 不直接进入“建议买/卖”，而是生成一个可确认的决策草稿，要求用户完成操作前纪律检查；用户确认后，草稿成为可复盘的 journal entry，并可挂接后续复盘提醒。

核心对象建议：

1. `DisciplineProfile`
   记录 actor 的决策纪律偏好，例如最大单标的风险暴露、是否允许短线事件交易、是否使用冷静期、常见错误类型、默认复盘周期。第一版可从 portfolio notes / notification prefs / company portraits 派生，不必强制新建完整配置页。

2. `TradeDecisionDraft`
   一次操作前草稿，包含 symbol、asset_type、action、direction、proposed_size、time_horizon、thesis_snapshot、evidence_for、evidence_against、invalidating_conditions、position_impact、cooldown_until、source_session_id。

3. `DisciplineReviewVerdict`
   对草稿的结构化审查结果：`ready_to_decide`、`needs_more_evidence`、`violates_thesis`、`position_risk_too_high`、`missing_exit_rule`、`emotion_or_fomo_risk`、`data_stale`。verdict 不替用户决定，只决定是否建议继续补证据、设置冷静期或建立 follow-up。

4. `TradeJournalEntry`
   用户确认后的长期记录。它不表示 Hone 建议了这笔交易，只表示用户完成了一次纪律化决策记录。entry 可关联 portfolio holding、company profile event、chat session、research artifact、document、evidence item 和 future cron job。

5. `DecisionFollowup`
   复盘计划和结果：例如 T+7、T+30、财报后、价格/基本面触发后，回看当时 thesis 是否成立、反证是否出现、用户是否偏离原计划、是否需要更新 company portrait 或 portfolio notes。

第一版应保持三个边界：

- 不接交易执行，不下单，不生成“买/卖/梭哈”指令。
- 不强制每次 portfolio 改动都创建 journal；只在用户表达操作判断、请求仓位建议、或主动要求记录时触发。
- 不把 journal 当作公司画像真相源。画像仍保存长期 thesis，journal 保存用户的动作前后决策过程。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“纪律日志”区块：
  - 最近决策草稿：待补证据、待确认、冷静期中。
  - 已确认 journal：按 ticker、动作、时间、结果状态展示。
  - 待复盘：T+7 / T+30 / 财报后 / 触发失效条件。
- 当用户在 `/chat` 说“我要不要加仓 MU”或“TSLA 跌这么多能抄底吗”，Hone 输出不再只是分析文本，而是：
  - 先读取当前 portfolio 与相关 company portrait。
  - 明确列出当前 thesis、反证、仓位影响、失效条件。
  - 生成一个“操作前检查草稿”，要求用户确认是否记录。
  - 若用户只是想问分析，可以跳过记录；若用户确认，则进入 journal。
- 对高风险动作展示强制追问，例如：
  - 没有退出条件。
  - 当前动作违反旧画像里的证伪条件。
  - 单一标的或期权风险暴露过高。
  - 证据主要来自短期价格或社交媒体情绪。
- 用户完成实际交易后，可以说“我按刚才计划买了 20 股”，Hone 再通过 `portfolio` 更新持仓，并把 journal entry 标记为 `executed_by_user`。这一步仍需用户确认，不由系统自动假设成交。

### 管理端

- 用户详情页增加 `Decision discipline` tab 或在 portfolio/detail 旁加入 journal 视图。
- 管理员可按 actor、ticker、状态、风险标签、复盘逾期、违反 thesis 条件过滤。
- 管理端不评价收益好坏为主，而看流程质量：
  - 有无 thesis snapshot。
  - 有无反方证据。
  - 有无退出/失效条件。
  - 是否设置 follow-up。
  - 复盘是否完成。
- 对运营和产品来说，这能形成“纪律价值”指标：记录了多少次冲动风险、多少次被冷静期拦住、多少次复盘后更新画像。

### 桌面端

- Desktop 复用同一 Web API，在 dashboard 或 portfolio 页面提示：
  - “2 条决策草稿待确认”
  - “1 条 MU 加仓决策进入 T+30 复盘”
  - “1 条期权动作缺少退出条件”
- 本地 bundled 用户可把 journal 默认保存在本机数据目录，降低隐私顾虑。
- Remote/public 用户应有明确删除和导出入口，避免把高敏投资意图变成不可控长期数据。

### 多渠道

- Feishu / Telegram / Discord 私聊支持轻量命令或自然语言：
  - “记录一个 MU 加仓决策”
  - “复盘我上次 TSLA 减仓”
  - “把这次操作设为 30 天后复盘”
- IM 回复只展示简洁 checklist 和 short decision id，不把完整私密仓位明细在群聊里展开。
- 群聊沿用 `DEFAULT_GROUP_PRIVACY_GUARD`：如果用户在群聊里要求记录具体持仓、成本或交易意图，应引导到私聊。
- 绑定 workspace 提案落地前，journal 第一版严格 actor-scoped；跨渠道共享只通过显式 export/import 或未来 workspace link。

## 技术方案

### 存储模型

建议在 `memory` 新增 `trade_discipline` 模块。第一版使用 SQLite 存 metadata 与结构化字段，长文本 snapshot 可放 JSON 字段；不需要迁移旧 portfolio。

建议表：

```text
trade_decision_drafts (
  draft_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_session_id TEXT,
  source_message_id TEXT,
  symbol TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  action_kind TEXT NOT NULL,
  direction TEXT,
  proposed_size_json TEXT,
  time_horizon TEXT,
  thesis_snapshot TEXT,
  evidence_for_json TEXT NOT NULL,
  evidence_against_json TEXT NOT NULL,
  invalidating_conditions_json TEXT NOT NULL,
  position_impact_json TEXT,
  review_verdict TEXT NOT NULL,
  risk_tags_json TEXT NOT NULL,
  status TEXT NOT NULL,
  cooldown_until TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

trade_journal_entries (
  entry_id TEXT PRIMARY KEY,
  draft_id TEXT,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  symbol TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  action_kind TEXT NOT NULL,
  user_decision TEXT NOT NULL,
  execution_status TEXT NOT NULL,
  linked_holding_symbol TEXT,
  linked_profile_id TEXT,
  linked_profile_event_id TEXT,
  linked_research_artifact_id TEXT,
  linked_evidence_id TEXT,
  rationale_snapshot TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

trade_decision_followups (
  followup_id TEXT PRIMARY KEY,
  entry_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  trigger_kind TEXT NOT NULL,
  due_at TEXT,
  trigger_condition_json TEXT,
  cron_job_id TEXT,
  status TEXT NOT NULL,
  review_summary TEXT,
  profile_update_needed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

文件系统不应存交易敏感原文散落副本。若需要导出，走 actor-scoped export API，包含 JSON/Markdown 两种格式。

### 后端 API

新增 `crates/hone-web-api/src/routes/trade_discipline.rs`：

- `GET /api/trade-discipline/drafts?actor=...`
- `POST /api/trade-discipline/drafts/preview`
- `POST /api/trade-discipline/drafts/:draft_id/confirm`
- `POST /api/trade-discipline/drafts/:draft_id/cancel`
- `GET /api/trade-discipline/journal?actor=...&symbol=&status=`
- `GET /api/trade-discipline/journal/:entry_id`
- `POST /api/trade-discipline/journal/:entry_id/followups`
- `POST /api/trade-discipline/followups/:followup_id/complete`

Public 端提供同构但只限当前登录 actor：

- `GET /api/public/trade-discipline/journal`
- `POST /api/public/trade-discipline/drafts/preview`
- `POST /api/public/trade-discipline/drafts/:draft_id/confirm`

第一版 preview 可以由 agent 产出结构化 JSON 后调用 tool/API 写草稿；后续再提供纯后端 deterministic review，避免所有判断都依赖模型。

### Agent 与工具

新增工具建议名 `trade_discipline`，可见操作：

- `preview_decision`
- `confirm_decision`
- `list_journal`
- `get_entry`
- `add_followup`
- `complete_followup`

触发路径：

- 用户明确说“买/卖/加仓/减仓/换仓/抄底/止损/追/卖 put/买 call”等操作意图。
- 用户调用 `position_advice` 后继续表达“那我就这么做”。
- 用户要求“帮我记录这个投资决策”。
- 用户更新 portfolio 且提供 rationale 时，系统建议是否同时创建 journal。

`position_advice` skill 可改为：

1. 先取 `portfolio(action="view")`。
2. 查公司画像和相关证据。
3. 输出风险管理分析。
4. 若用户要求具体动作或记录，调用 `trade_discipline(preview_decision, ...)`，而不是只给自由文本。

`portfolio_management` skill 保持写入工具职责，但在用户新增真实持仓且带有明确理由时，提示可同时记录 journal。不要让 portfolio 写入自动暗示 Hone 认可该交易。

### 数据流

1. 用户在 Web/IM/Desktop 表达操作意图。
2. `AgentSession` 仍按现有 runner、prompt、skill 流程执行。
3. 模型通过 `portfolio`、company profile 文件、data/search 工具补齐上下文。
4. 模型调用 `trade_discipline(preview_decision)` 创建草稿，返回 checklist、risk tags、decision id。
5. 用户确认后调用 `confirm_decision`，生成 journal entry。
6. 如用户要求后续复盘，创建 `DecisionFollowup`，可复用 `cron_job` 写入提醒任务。
7. 到期复盘时，scheduler 触发一轮带 entry context 的 transient task；模型只复盘当时理由是否仍成立，并建议是否更新画像或 portfolio notes，不自动改持仓。

### 兼容策略

- 不迁移旧 portfolio；旧 holdings 只在 journal 页面显示“无决策记录”。
- `strategy_notes` 和 `notes` 仍保留为轻量持仓备注；journal 是补充层，不替代。
- `company_portrait` 不直接读取全部 journal，避免把短期操作噪音污染长期画像。只有用户在复盘中明确确认“这改变了长期 thesis”时，才通过画像 skill 写入事件或主画像。
- 公开部署默认关闭交易日志导出中的敏感字段，用户手动导出时才包含完整 rationale。

## 实施步骤

### Phase 1: 最小 journal 存储与 Web 只读视图

- 在 `memory` 增加 `trade_discipline` SQLite 存储、类型和单元测试。
- 新增 admin/public journal list/detail API。
- 在 public `/portfolio` 与 admin 用户详情中显示 journal 空状态、列表和详情。
- 暂不改 agent 行为，先允许手工创建测试 entry，验证数据模型和隐私边界。

### Phase 2: Agent preview 工具与 position advice 串联

- 新增 `trade_discipline` tool。
- 更新 `skills/position_advice/SKILL.md`，让操作意图走 preview checklist。
- 在 prompt 或 skill 层明确：verdict 是纪律审查，不是交易许可。
- 增加 CI-safe 单元测试覆盖 preview 参数校验、actor 隔离、状态转换。

### Phase 3: Follow-up 复盘与 cron 联动

- 为 journal entry 创建 follow-up。
- 复用 `cron_job` 或新增轻量 scheduler adapter，在 due_at 触发复盘。
- 复盘结果可建议更新 company portrait，但必须由用户确认进入画像更新流程。
- Web/desktop 增加“待复盘”视图和 overdue 状态。

### Phase 4: 纪律指标与增长体验

- 聚合每个 actor 的纪律指标：草稿数、确认数、取消数、冷静期数、逾期复盘数、画像更新数。
- Public `/me` 或 `/portfolio` 展示“本月完成 6 次操作前检查，2 次因证据不足延后”这类价值反馈。
- 与 usage entitlement 后续衔接：高级 plan 可解锁更多 journal 历史、自动 follow-up 和跨渠道 workspace 聚合。

## 验证方式

- 单元测试：
  - `TradeDecisionDraft` 状态转换：draft -> confirmed / cancelled。
  - actor 隔离：A actor 不能读取或确认 B actor draft。
  - follow-up 与 journal entry 外键关系。
  - high-risk 参数校验：缺 symbol、缺 action、缺 rationale 时拒绝 confirm。
- API 测试：
  - Public session 只能访问当前 `web` actor。
  - Admin actor 可按指定 actor 查询。
  - 删除或取消 draft 不影响 portfolio。
- Skill / agent 回归：
  - 用户问“我要不要加仓 NVDA”时不输出直接买卖指令，而生成纪律 checklist。
  - 用户说“我买了 100 股 AAPL，记录原因”时先确认 portfolio 写入和 journal 记录边界。
  - 群聊中请求记录成本价时触发隐私引导到私聊。
- UI 验收：
  - Public `/portfolio` 在无 journal、有 draft、有 overdue follow-up 三种状态下展示清晰。
  - Admin 用户详情能从 portfolio holding 跳到相关 journal。
  - Desktop bundled/remote 只消费同一 API，不引入本地分叉状态。
- 指标：
  - journal draft -> confirmed 转化率。
  - confirmed entry 的 follow-up 设置率和完成率。
  - 因 `needs_more_evidence` / `missing_exit_rule` 被用户取消或延后的次数。
  - journal 后续触发 company portrait 更新的比例。

## 风险与取舍

- 风险：用户误以为 journal verdict 是投资许可。
  取舍：文案始终表达为“纪律检查结果”和“供你独立决策”，不使用 allow / approve / recommended buy 这类词。

- 风险：记录交易意图带来隐私敏感性。
  取舍：严格 actor-scoped，public 默认提供删除/export，群聊不采集具体成本和持仓明细，workspace 聚合必须等待显式绑定。

- 风险：把短期交易噪音写进公司画像。
  取舍：journal 和 company portrait 分层；只有复盘确认影响长期 thesis 时才走 `company_portrait` 更新。

- 风险：增加用户操作摩擦。
  取舍：只对明确操作意图、用户主动记录、或高风险仓位建议触发；普通研究问答不强制创建草稿。

- 风险：模型把 checklist 写得模板化。
  取舍：第一版要求结构化字段与缺口 reason code，后续用真实 journal 样本做回归评测，避免只靠长 prompt。

- 风险：与未来券商交易功能边界混淆。
  取舍：本提案明确不接交易执行、不自动下单、不读取券商账户；只服务投资纪律和复盘。

## 与已有提案的差异

- 与 `auto_p0_investment_output_safety_gate.md` 不重复：安全门禁处理用户可见投资输出能否送达；本提案处理用户自己的操作意图如何被记录、审查和事后复盘。
- 与 `auto_p1_evidence_review_queue.md` 不重复：证据队列处理市场事件是否改变公司 thesis；本提案处理用户准备采取的买卖/加减仓/期权动作是否符合纪律。
- 与 `auto_p1_investment_context_intake.md` 不重复：intake 解决 portfolio/profile/prefs/task 的初始化缺口；本提案解决已有上下文下的具体动作决策流程。
- 与 `auto_p1_investment_document_inbox.md` 不重复：document inbox 管用户上传证据的身份和治理；本提案可引用这些证据，但核心对象是 action decision。
- 与 `auto_p1_research_artifact_library.md` 不重复：research artifact 留存深度研究交付物；本提案留存用户基于研究材料形成的纪律化行动记录。
- 与 `auto_p1_delivery_decision_loop.md` 不重复：delivery loop 解释为什么推送或不推送；本提案解释用户为什么准备操作、是否满足自己的纪律。
- 与 `auto_p1_automation_intent_control_plane.md` 不重复：automation intent 管自动化任务创建/修改；本提案只在 follow-up 阶段可复用 cron，不管理一般自动化意图。
- 与 `auto_p1_linked-user-workspace.md` 不重复：workspace 处理跨渠道资产归属；本提案第一版严格 actor-scoped，未来可成为 workspace asset。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：entitlement 处理商业权益和成本；本提案处理投资纪律价值闭环。
- 与 `auto_p1_run_trace_workbench.md` 不重复：run trace 复盘 agent 执行链路；本提案复盘用户投资决策链路。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：skill runtime 提案处理 skill 机制与 runner 对齐；本提案只使用 skill/tool 作为实现入口，不改变 skill 基础架构。

查重结论：现有 proposal 已覆盖通知、证据、研究材料、用户上下文、身份、权益、运行排障、自动化控制和输出安全，但没有覆盖“用户操作意图 -> 操作前纪律 checklist -> 用户确认的决策日志 -> 到期复盘 -> 画像/持仓备注交接”的闭环。因此本主题是新的、可落地的 P1 产品/架构提案。
