# Proposal: Agent Clarification Queue for Deferred Human Input

status: proposed
priority: P1
created_at: 2026-06-01 20:04:41 +0800
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
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-channels/src/scheduler.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/chat.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/routes/task_runs.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/tasks.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/pages/dashboard.tsx`
- `skills/scheduled_task/SKILL.md`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Honeclaw 已经具备多种主动和半主动能力：

- `AgentSession::run()` 是 Web、public chat、IM channel 和工具调用的统一会话入口，负责 quota、runner、listener、session persistence 和 final response。
- `crates/hone-channels/src/ingress.rs` 统一 direct/group actor scope、dedup、session lock、pretrigger window 和 busy lifecycle。
- `crates/hone-channels/src/scheduler.rs` 与 `memory/src/cron_job` 支撑 daily、weekly、once、trading day、holiday 和 heartbeat 自动任务。
- `hone-event-engine` 能从市场事件、RSS、Telegram source、portfolio subscription 和 notification prefs 生成 direct push 或 digest。
- `skills/scheduled_task`、`skills/company_portrait`、portfolio tool、notification prefs tool 和 cron tool 都已经让 agent 可以维护长期投资工作台状态。
- Public Web 已有 `/chat`、`/me`、`/portfolio`；admin console 已有 sessions、tasks、task-health、notifications、schedule、users、skills、LLM audit 和 logs；desktop bundled mode 也能展示 backend/channel 状态。

这些能力说明 Hone 正在从“聊天入口”走向“持续协作的投资助理”。但当前产品架构里，agent 一旦缺少用户输入，通常只有三种选择：

1. 在当前 chat turn 里直接追问。
2. 猜一个保守答案或放弃操作。
3. 在自动任务、event-engine、digest 或后台研究链路里写失败/跳过状态，等待用户自己发现。

这对同步聊天勉强够用，但对多渠道、自动化和长期记忆不够。真实投资助理经常需要异步追问：某个持仓是长期核心仓还是观察仓、某条事件是否值得更新画像、某个定时任务应该发到哪个渠道、某份文档对应哪个 ticker、某个 runner 权限请求是否应继续。现在这些“需要用户补一句话才能继续”的状态没有一等产品对象。

## 问题或机会

### 主要问题

1. **澄清问题只存在于一次聊天上下文。**
   如果用户当时没有回答，问题会埋在 session 历史里。用户从 Feishu 转到 Web、从 public chat 转到 desktop、或过几小时打开 `/me` 时，看不到 Hone 正在等什么。

2. **自动化和后台任务缺少可恢复的人机接力。**
   scheduled task、heartbeat、event-engine、research task 在发现缺少上下文时，只能失败、跳过或输出泛化提示。它们不能生成一个持久化的“请确认这个选择”，等用户稍后回答后继续或修复。

3. **多个现有提案会产生待用户输入，但没有通用收口。**
   Automation Intent 需要用户确认任务草稿，Mutation Ledger 需要确认状态变更，Permission Broker 需要审批 runtime action，Evidence Review Queue 需要判断证据是否改变 thesis，Investment Context Intake 需要补全缺口。它们各自都有局部 pending 状态，但缺少一个面向用户的统一问题收件箱。

4. **多渠道回复无法可靠落到正确对象。**
   用户在 IM 中回复“确认”“不是噪音”“改成 8:30”时，系统需要知道这句话是在回答哪个待办问题，而不只是发起一个新 chat turn。当前 `SessionIdentity` 解决会话归属，但没有 `QuestionIdentity` 解决跨入口的待回答项。

5. **用户体验上缺少“助理正在等我”的清晰信号。**
   Public `/me` 目前主要展示账号信息和 membership placeholder；dashboard 展示 backend/channel/research 状态；task-health 展示后台任务健康。它们都没有把“待我回答的问题”作为留存和协作入口。

### 机会

新增 **Agent Clarification Queue**，把 agent、工具、自动化和后台任务产生的澄清请求统一成 actor-scoped、可过期、可回答、可继续处理的对象。它不替代 mutation approval、permission approval 或 evidence review，而是在它们之上提供一个用户可见的人类输入层：

- 用户能在 Web/desktop/IM 看到 Hone 正在等哪些回答。
- 后台任务能以结构化方式请求补充信息，而不是静默失败。
- 管理端能看到用户激活卡在哪里、自动化为什么没有闭环。
- 多渠道回复能绑定到具体 clarification item，减少误确认和上下文串线。

这是 P1，因为它直接提升核心体验、自动化可靠性和留存。它可以先以薄存储和 UI badge 落地，不需要重写 runner 或引入外部服务。

## 方案概述

新增 actor-scoped 的 `ClarificationItem` 与 `ClarificationQueueService`。

一个 clarification item 表示“系统需要用户提供一段明确输入，才能继续某个上下文”。它不是普通通知，也不是状态变更记录，而是一个可回答的问题对象。

核心字段：

- `clarification_id`：稳定 ID。
- `actor`：沿用 `ActorIdentity`，不改变数据隔离。
- `session_id` / `task_id` / `execution_id` / `event_id`：可选来源指针。
- `source`：`chat`、`scheduled_task`、`event_engine`、`research_task`、`tool`、`skill`、`admin_web`、`desktop`。
- `kind`：`missing_context`、`choose_option`、`confirm_preference`、`resolve_ambiguity`、`supply_identifier`、`retry_decision`、`human_review`.
- `question`：用户可见问题，必须短、具体、无 chain-of-thought。
- `options`：可选结构化选项，例如 `core_holding` / `watchlist_only` / `ignore`.
- `freeform_allowed`：是否允许自由文本回答。
- `default_action`：过期后的行为，例如 `skip`、`keep_pending`、`use_safe_default`、`mark_failed`.
- `status`：`open`、`answered`、`expired`、`dismissed`、`superseded`、`applied`、`failed`.
- `answer`：用户回答、回答来源、回答时间、answer actor。
- `continuation`：回答后的处理计划，例如 resume prompt、apply intent、mark evidence、update draft。
- `expires_at` / `snoozed_until` / `priority`：控制展示和过期。

第一版范围应保持保守：

- 不自动恢复长时间过期的市场敏感任务。
- 不把用户回答直接当作交易指令。
- 不让 UI 直接编辑 company profile 正文。
- 不把 permission approval 或 mutation approval 混同为普通 clarification；它们可以在用户收件箱里展示，但仍由各自服务执行审批语义。

## 用户体验变化

### Public 用户端

`/me` 增加一个“待回答”区域：

- 顶部显示 open count、最紧急问题、来源。
- 每个问题显示短标题、来源、关联标的/任务、提出时间和过期时间。
- 支持单选、多选、简短文本、稍后提醒、忽略。
- 回答后展示结果：“已用于更新任务草稿”“已标记这条证据为噪音”“Hone 会在下一次任务运行时继续”。

`/chat` 中如果当前 session 有 open clarification：

- composer 上方显示内联问题卡。
- 用户回答卡片后，消息带 `clarification_id` metadata 进入 `AgentSession`，而不是只作为普通聊天文本。
- 如果用户直接打字且文本明显匹配 open question，可以提示“是否作为对上一个问题的回答？”

### 管理端

新增或扩展 actor detail 的 `Clarifications` tab：

- 按 actor、source、kind、status、age、priority 过滤。
- 从 task-health、sessions、notifications、research task 跳转到相关 clarification。
- 管理员能 dismiss、snooze、代用户回答低风险运营问题；高风险投资上下文默认只能由用户回答或明确记录 admin 代答。
- 支持看“最近 7 天哪些问题最常卡住用户”，为 onboarding 和产品文案优化提供依据。

### 桌面端

Dashboard 增加轻量 badge：

- `2 questions waiting for you`
- 点击进入 Web console 或 public `/me` 的 clarification 区域。
- Bundled local mode 直接消费本地 backend；remote mode 消费远端 API，不扫描本机文件推断。

桌面通知可以只对 P1/open 超过阈值的问题弹出，避免把每个 agent 追问变成打扰。

### 多渠道

IM 端不需要复杂卡片第一版：

- 当 agent 生成 clarification 时，发送一条短消息：“Hone 需要你确认：NVDA 是真实持仓还是关注？回复 `1` 真实持仓，`2` 仅关注，或稍后在 Web 处理。”
- 回复必须绑定最近的 open clarification token；token 过期后不再应用，改为普通 chat。
- 群聊中默认不创建个人投资上下文问题，除非触发用户可被明确识别且问题不涉及私密持仓；否则提示去私聊处理。

## 技术方案

### 1. 存储与服务

建议在 `memory` 新增 `clarification.rs`，本地模式使用 SQLite，云模式未来映射 PG。原因是 clarification 需要分页、按 actor/source/status 查询、过期扫描和幂等更新。

建议表结构：

```sql
CREATE TABLE clarification_items (
  id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source TEXT NOT NULL,
  kind TEXT NOT NULL,
  priority TEXT NOT NULL,
  status TEXT NOT NULL,
  question TEXT NOT NULL,
  options_json TEXT NOT NULL,
  freeform_allowed INTEGER NOT NULL,
  default_action TEXT NOT NULL,
  context_json TEXT NOT NULL,
  continuation_json TEXT NOT NULL,
  answer_json TEXT,
  session_id TEXT,
  task_id TEXT,
  execution_id TEXT,
  event_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  expires_at TEXT,
  snoozed_until TEXT,
  answered_at TEXT,
  superseded_by TEXT
);

CREATE INDEX idx_clarification_actor_status
  ON clarification_items(actor_channel, actor_user_id, actor_scope, status, updated_at);
```

服务接口：

```rust
pub trait ClarificationStore {
    fn upsert_item(&self, item: ClarificationItem) -> HoneResult<ClarificationItem>;
    fn list_items(&self, filter: ClarificationFilter) -> HoneResult<Vec<ClarificationItem>>;
    fn answer_item(&self, id: &str, answer: ClarificationAnswer) -> HoneResult<ClarificationItem>;
    fn mark_status(&self, id: &str, status: ClarificationStatus) -> HoneResult<ClarificationItem>;
}
```

`question` 必须是用户可见文本，不保存模型推理过程。`context_json` 只能保存最小引用和摘要，不保存 raw secret、raw session token、完整附件内容或 sandbox 外绝对路径。

### 2. 生成入口

第一阶段只接入低风险、高价值入口：

- `InvestmentContextGap` 类问题：portfolio 为空、ticker 无法识别、真实持仓/关注不明确。
- `scheduled_task` 创建/更新时缺少 channel target、时区、频率或 task prompt。
- event/evidence 需要用户选择：`mark_noise`、`review_later`、`send_to_agent`.
- interrupted run 恢复时需要用户决定是否重新发送旧问题。
- document/capture 类流程需要确认文件对应 ticker 或用途。

暂不接入：

- runtime permission approval 的实际授权动作。
- 高风险 mutation apply。
- 交易相关买卖确认。

这些可以在 UI 中统一展示，但继续走各自 proposal 的安全服务。

### 3. AgentSession 和 turn metadata

在 `AgentSession::run()` 入参或 metadata 中增加可选 `answered_clarification_id`：

- Web/public API 在用户回答问题卡时传入。
- IM channel listener 在解析确认 token 后传入。
- `turn_builder` 把 clarification answer 放在当前 turn input 的动态块中，不写入 static system prefix。
- session persistence 保存一条可读的 user message，例如“回答了待确认问题：NVDA 是真实持仓”，并在 metadata 保存 item id，方便历史恢复。

如果回答会触发 continuation：

- `Continuation::ResumePrompt`：生成一条受控 prompt 进入当前 session。
- `Continuation::ApplyDraft`：调用对应 intent/mutation/evidence 服务，仍由目标服务校验权限。
- `Continuation::MarkOnly`：只更新 clarification 状态。

### 4. API 设计

Admin API：

- `GET /api/clarifications?actor=&source=&kind=&status=&from=&to=`
- `GET /api/clarifications/{id}`
- `POST /api/clarifications/{id}/answer`
- `POST /api/clarifications/{id}/dismiss`
- `POST /api/clarifications/{id}/snooze`

Public API：

- `GET /api/public/clarifications`
- `POST /api/public/clarifications/{id}/answer`
- `POST /api/public/clarifications/{id}/dismiss`

Public API 必须从 `hone_web_session` 推导当前 actor，不接受 actor query。回答时校验：

- item 属于当前 actor。
- status 是 `open`。
- 未过期，或 default action 允许 late answer。
- option id 合法，freeform 长度受限。
- continuation 所需权限仍满足当前 runtime 状态。

### 5. 前端落点

新增：

- `packages/app/src/lib/clarifications.ts`
- `packages/app/src/context/clarifications.tsx`
- `packages/app/src/components/clarification-card.tsx`

修改：

- `packages/app/src/pages/public-me.tsx`：展示当前用户 open clarification。
- `packages/app/src/pages/chat.tsx`：渲染 session/open question 卡，并把回答提交到 public/admin API。
- `packages/app/src/pages/dashboard.tsx`：显示 open count badge。
- `packages/app/src/pages/tasks.tsx` / `task-health.tsx`：从失败或待确认任务跳到 clarification。
- `packages/app/src/pages/users.tsx`：actor detail 增加 clarification tab 或 activity section。

### 6. 多渠道接入

在 `hone-channels` 增加共享解析辅助：

- 创建 channel-specific short token，例如 `cq_7F3K`，映射到 open item。
- token 只绑定 actor + channel + item + expiry，不跨 actor 生效。
- direct message 中“1/2/稍后/忽略”可解析为 answer/dismiss/snooze。
- group message 只有 reply-to-bot 且 sender 匹配 item actor 时才应用。

Feishu/Telegram/Discord 可先只支持文本 token；Web/desktop 支持完整卡片。

## 实施步骤

### Phase 1: Queue 底座和 Web 只读

- 在 `memory` 增加 clarification 类型、SQLite store、过期扫描和单元测试。
- 增加 admin/public list API。
- 在 public `/me` 和 admin dashboard 展示 open count 与列表。
- 手工或测试 fixture 创建 clarification item，验证 UI 展示和权限边界。

### Phase 2: 回答闭环

- 增加 answer/dismiss/snooze API。
- `chat.tsx` 支持 answer card。
- `AgentSession` 支持 `answered_clarification_id` metadata 注入当前 turn。
- 加入 `Continuation::MarkOnly` 和 `Continuation::ResumePrompt` 两种最小 continuation。

### Phase 3: 接入首批来源

- Investment context 缺口生成 `missing_context` 问题。
- scheduled task 创建/更新缺字段时生成 `choose_option` 或 `supply_identifier` 问题。
- interrupted run recovery 需要用户确认 retry 时生成 `retry_decision`。
- evidence review 的低风险处理选择可生成 `human_review` 问题。

### Phase 4: 多渠道和运营指标

- Feishu/Telegram/Discord direct message 支持 short token answer。
- Dashboard 和 task-health 展示 clarification age、source breakdown、answer rate。
- 增加过期 default action 执行器，避免 open item 无限积压。

## 验证方式

- 单元测试：
  - `ClarificationStore` upsert/list/answer/dismiss/snooze/expire。
  - public actor 不能读取或回答其他 actor 的 item。
  - option answer、freeform answer、过期 answer、superseded item 行为正确。
  - continuation payload 不保存 raw secret 或 sandbox 外绝对路径。

- API 测试：
  - `GET /api/public/clarifications` 只返回当前 cookie actor 的 open item。
  - `POST /api/public/clarifications/{id}/answer` 对错误 actor、非法 option、过期 item 返回稳定错误。
  - admin list 支持 actor/source/status 过滤。

- 前端测试：
  - `/me` 在无问题、有 open、answered 后三种状态下展示正确。
  - `chat.tsx` 提交 clarification answer 后不会重复作为普通消息发送两次。
  - dashboard badge 在 open count 变化后更新。

- 手工验收：
  - 创建一个 scheduled task 缺 channel target 的 fixture，用户在 `/me` 回答后，任务草稿能继续生成 preview。
  - 在 Feishu 私聊收到一个选项问题，回复 `1` 后 item 标记 answered；回复过期 token 不会误应用。
  - 群聊中非触发者回复 token 不会回答别人的问题。

- 指标：
  - clarification open count、平均回答时间、过期率、source breakdown。
  - scheduled task 因缺字段失败率下降。
  - public `/me` 到 `/chat` 的回访率提升。

## 风险与取舍

- **风险：和审批/确认系统概念重叠。**
  取舍：Clarification Queue 只收“需要用户输入的信息问题”；真正的 permission approval、mutation apply、automation approve 仍由对应服务判定，只把入口呈现在同一收件箱。

- **风险：用户被问题轰炸。**
  取舍：第一版只生成 P1 高价值问题；同一 source/ticker/task 做幂等 upsert；低优先级问题进入 digest 或 dashboard，不主动多渠道打扰。

- **风险：回答被误用为投资指令。**
  取舍：schema 明确禁止交易执行语义；回答只能补上下文、偏好、分类或 retry 决策，不能表达买卖下单。

- **风险：旧问题在市场时间敏感场景中过期。**
  取舍：每个 item 必须有 default action；retry/market-sensitive continuation 过期后要求重新确认并重算当前时间。

- **风险：多渠道 token 可能串线。**
  取舍：token 绑定 actor、channel、item、expiry 和 reply context；群聊默认更严格，无法确认 sender 时不应用。

- **不做的边界：** 不在本提案中实现交易审批、不实现通用 workflow engine、不让 UI 直接编辑 company profile 正文、不改变 `ActorIdentity` / `SessionIdentity` 的隔离模型。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，重点比对了 automation intent、mutation ledger、permission broker、interrupted run recovery、investment context intake、evidence review queue、response feedback、user journey replay、daily brief workspace、desktop alert center 和 collaborative research rooms。

- 不重复 `auto_p1_automation_intent_control_plane.md`：该提案解决 cron/automation 变更的 preview、approve 和 audit；本提案解决所有来源产生的“需要用户补充信息”的统一收件箱。Automation intent 可以把待确认问题投到 Clarification Queue，但审批语义仍属于 intent 服务。
- 不重复 `auto_p1_agent-mutation-ledger.md`：mutation ledger 记录状态变更 before/after、确认和撤销；clarification 不记录状态变更事实，只记录用户需要回答的问题及其回答。
- 不重复 `auto_p1_agent-permission-broker.md`：permission broker 判断 runtime action 能不能执行；clarification queue 不授予工具/文件/终端权限，只能承载“是否继续、选哪个、补哪个字段”的人类输入。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：recovery inbox 管已经中断的运行如何发现、通知、重试或失败；clarification queue 可以承载“是否重试”这个用户问题，但不负责中断检测和恢复策略。
- 不重复 `auto_p1_investment_context_intake.md`：context intake 是投资上下文初始化和缺口修复流程；clarification queue 是通用待回答层，context intake 可以作为来源之一创建 missing context 问题。
- 不重复 `auto_p1_evidence_review_queue.md`：evidence review queue 管证据项是否应复盘、更新画像或忽略；clarification queue 只在需要用户对某条证据做简短选择时承载问题，不保存证据生命周期。
- 不重复 `auto_p2_collaborative-research-rooms.md`：research rooms 面向团队研究空间、open questions 和决策记录；clarification queue 面向单 actor 的待回答任务，不引入多人 room 或协作权限。

查重结论：现有 proposal 覆盖了待审批变更、权限、证据复盘、中断恢复和投资上下文初始化，但没有覆盖“跨 chat、自动化、事件引擎、桌面和多渠道的持久化 agent 澄清问题队列”。本主题是新的、可落地的 P1 提案。

## 文档同步说明

本轮只创建 proposal，不开始执行实现，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续进入实现，应按动态计划准入标准新增或复用 `docs/current-plans/agent-clarification-queue.md`，并在新增存储、API、前端入口、多渠道回复语义后同步更新 `docs/repo-map.md`、`docs/invariants.md` 和必要的 handoff。
