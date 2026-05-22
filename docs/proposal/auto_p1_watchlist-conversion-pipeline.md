# Proposal: Watchlist Conversion Pipeline for Pre-Position Research

status: proposed
priority: P1
created_at: 2026-05-22 08:03:07 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_portfolio-transaction-ledger.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/global_digest/mainline_cron.rs`
- `crates/hone-event-engine/src/global_digest/audience.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/lib/types.ts`
- `packages/app/src/lib/api.ts`
- `skills/portfolio_management/SKILL.md`
- `skills/stock_research/SKILL.md`
- `skills/company_portrait/SKILL.md`
- `skills/scheduled_task/SKILL.md`

## 背景与现状

Hone 当前已经把 watchlist 纳入核心投资上下文，而不是普通收藏夹：

- `memory/src/portfolio.rs` 的 `Holding` 用 `tracking_only: Some(true)` 表示关注标的，仍存放在同一个 actor-scoped portfolio JSON 中；`upsert_watch` 创建 0 股、0 成本的关注项，`promote_to_holding` 能把关注项升级为真实持仓。
- `skills/portfolio_management/SKILL.md` 明确写出“关注与持仓同级触发 registry，自动进入白名单”，并要求用户没说明 shares 时走 `watch` 而不是写 0 股持仓。
- `crates/hone-event-engine/src/subscription.rs` 从 portfolio 构建订阅时会包含 `tracking_only` symbol，测试也锁定“关注与持仓同级推送”的契约。
- `crates/hone-event-engine/src/global_digest/mainline_cron.rs`、`global_digest/audience.rs` 和 `unified_digest/scheduler.rs` 都会把 portfolio 里的 symbol 纳入 mainline、digest audience 或 actor focus symbols；目前它们基本不区分真实持仓与 watchlist。
- `crates/hone-web-api/src/routes/portfolio.rs`、`packages/app/src/context/portfolio.tsx` 和 `packages/app/src/components/portfolio-detail.tsx` 已在 Admin 端把 holdings 与 watchlist 分开展示，但 watchlist 只有 symbol、notes、strategy notes 和删除/编辑动作。
- Public `/portfolio` 的 `DigestContext` 类型只有 `holdings: string[]`，页面文案是“各持仓投资主线”，但实际数据来源可能包含 watchlist symbol，用户难以理解哪些是已持有、哪些只是候选。

这说明 watchlist 已经进入推送、主线蒸馏、用户上下文和管理端，但产品语义仍停留在“被动订阅 ticker”。对于强调投资纪律的个人研究助手，关注标的更应该是一条 pre-position research pipeline：为什么关注、现在卡在哪、什么证据会让它升级为持仓、什么条件会让它出清或放弃。

AI agent 产品正在从“开放聊天”转向“可恢复、可审核、可持续推进的工作队列”。Hone 已有 multi-channel、cron、company portrait、event-engine 和 actor sandbox，正适合把 watchlist 从静态清单升级为投资研究漏斗，而不是继续让用户靠 notes 和自由聊天记住下一步。

## 问题或机会

这是 P1 级机会：它不会像输出安全门那样直接阻止高危错误，但会显著提升核心体验、研究连续性、推送相关性和持仓转化质量。

当前缺口集中在六点：

1. **关注原因没有结构化。**  
   `strategy_notes` / `notes` 可以写自由文本，但系统不知道用户关注一个 ticker 是因为估值、产品周期、财报、价格区间、同行对比、宏观主题、还是朋友推荐。event-engine 和 agent 只能把它当普通 symbol。

2. **watchlist 没有状态机。**  
   一个关注项可能处在 `idea`、`researching`、`waiting_for_price`、`waiting_for_event`、`needs_profile`、`ready_for_review`、`rejected`、`promoted` 等状态。当前只有 `tracking_only=true`，无法表达“下一步是什么”。

3. **关注与持仓在下游语义混杂。**  
   订阅层故意把 watchlist 纳入推送，这是对的；但 public portfolio、mainline distill 和 digest focus 需要知道“这是候选标的，不是资金暴露”。否则用户会觉得 Hone 把观察对象说成了真实组合。

4. **缺少从关注到持仓的纪律闭环。**  
   `promote_to_holding` 能技术上升级，但没有要求用户确认“为什么现在从关注变成持仓、是否满足原先条件、是否有反证”。这会削弱 Hone “ruthless defender of investment discipline”的定位。

5. **watchlist 无法驱动定期复核。**  
   `scheduled_task` 能建立提醒，但 watchlist 本身没有 `next_review_at`、`watch_until`、`catalyst_date` 或 `stale_after_days`。关注项越多，越容易变成噪音订阅池。

6. **Admin 和用户端都难以判断关注池质量。**  
   Admin 可以看到 watchlist 数量，但看不到哪些关注项缺公司画像、缺关注原因、长期未复核、已经触发条件却未处理，或者被推送打扰但没有投资价值。

机会是新增一个 **Watchlist Conversion Pipeline**：在不改交易执行边界、不绕过 agent-mediated portrait 约束的前提下，为 watchlist 增加研究状态、触发条件、复核节奏和升级/放弃审计，让它成为 Hone 的增长与留存入口。

## 方案概述

第一版不需要重写 portfolio 真相源。建议在现有 `Holding` 的 watchlist 记录上增加一个向后兼容的 `watchlist_meta` 派生/内嵌模型，或者先以并行 actor-scoped JSON 保存，再逐步并入 portfolio：

```rust
pub struct WatchlistMeta {
    pub symbol: String,
    pub status: WatchlistStatus,
    pub thesis_reason: Option<String>,
    pub research_question: Option<String>,
    pub trigger_conditions: Vec<WatchTriggerCondition>,
    pub disqualifiers: Vec<String>,
    pub next_review_at: Option<DateTime<Utc>>,
    pub catalyst_date: Option<DateTime<Utc>>,
    pub linked_profile_id: Option<String>,
    pub linked_tasks: Vec<String>,
    pub promoted_from_watchlist_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub last_event_matched_at: Option<DateTime<Utc>>,
}
```

推荐状态：

- `idea`: 刚加入，只知道 symbol 和粗略原因。
- `needs_profile`: 需要建立或修复 company portrait。
- `researching`: 正在收集证据，尚未形成进入条件。
- `waiting_for_price`: 等待价格/估值区间。
- `waiting_for_event`: 等待财报、产品、监管、宏观或公司行动。
- `ready_for_review`: 条件可能已触发，需要用户复核。
- `rejected`: 用户或 agent 明确放弃，默认不再推送。
- `promoted`: 已升级为真实持仓，保留历史原因供后续复盘。

关键原则：

- Watchlist 仍可进入推送订阅，但事件文案必须标明“关注标的”而不是“持仓”。
- 升级为持仓必须显式确认，记录原关注理由、触发证据和用户确认，不做自动买入暗示。
- UI 可以维护结构化 meta，但 company portrait 正文仍遵守现有约束：创建和内容更新由 agent 原生文件能力处理。
- 第一版只做单 actor；跨渠道共享等待 linked workspace 类提案落地。

## 用户体验变化

### 用户端

- Public `/portfolio` 把“各持仓投资主线”拆成两块：
  - `真实持仓`: 已持有 symbol 的投资主线、画像和风险。
  - `关注研究池`: watchlist symbol 的关注原因、状态、下一次复核、是否缺画像。
- 用户可以在 `/chat` 里说：
  - “关注 TEM，等财报后看客户留存和毛利率。”
  - “如果 NVDA 回到 900 以下提醒我复核，但不要说买入。”
  - “把 RKLB 从关注升级为持仓，我买了 20 股，原因是上次等的发射合同落地了。”
  - “这个标的证伪了，移出研究池。”
- 每个关注项展示最多三个行动入口：`补画像`、`设置复核`、`升级/放弃复核`。行动入口回到 chat 或打开 confirmation flow，而不是直接静默修改关键状态。
- 关注项长期未复核时，页面提示“关注已陈旧”，引导用户删除、更新 thesis、或创建复核任务。

### 管理端

- `/users/:actor/portfolio` 的 watchlist 表从普通表格升级为 pipeline 列表：
  - 状态、关注原因、等待条件、下一次复核、画像覆盖、最近命中事件、相关任务。
  - 快速筛选：`needs_profile`、`stale`、`ready_for_review`、`no_reason`、`too_noisy`。
- Admin 可看到 watchlist quality summary：
  - 关注数、缺原因数、缺画像数、30 天未复核数、触发后未处理数、已升级数。
- 若用户反馈推送太吵，Admin 可以区分是 notification prefs 问题、event-engine routing 问题，还是 watchlist 里有过期/无条件的候选标的。

### 桌面端

- Desktop dashboard 增加紧凑的 watchlist pipeline 状态：
  - “2 个关注项需要补画像 / 1 个价格条件已触发 / 3 个 30 天未复核”。
- 本地 bundled 模式完全使用本地 portfolio 和 actor sandbox；remote 模式显示后端来源和更新时间。
- Desktop 快速捕捉可以把当前浏览的 ticker 或用户输入先加入 `idea`，后续由 chat 补结构化条件。

### 多渠道

- Feishu / Telegram / Discord 私聊支持低摩擦命令：
  - “关注 XX，原因是...”
  - “我的关注池有什么该复核的？”
  - “把 XX 标记为放弃，原因是...”
- 群聊默认不暴露个人 watchlist 状态；只有明确共享 group session 且用户确认时，才创建 group-scoped 关注项。
- 多渠道推送里若命中 watchlist，消息前缀应区分：`关注标的触发`、`持仓触发`、`复核提醒`，避免把候选标的包装成真实仓位。

## 技术方案

### 1. 数据模型与兼容策略

两种落地路径：

- **保守路径：新增 `watchlist_meta` 独立存储。**  
  放在 `memory/src/watchlist_pipeline.rs` 或 `memory/src/portfolio_watchlist.rs`，按 `ActorIdentity` 存储 `watchlist_pipeline_{actor}.json`。优点是对现有 `Portfolio` JSON 无 breaking change；缺点是读取时需要 join。

- **收敛路径：扩展 `Holding`。**  
  在 `memory/src/portfolio.rs` 的 `Holding` 增加 `watchlist_meta: Option<WatchlistMeta>`，仅 `tracking_only=true` 时使用。serde default 保证旧 JSON 兼容。优点是单一 portfolio 文件更直观；缺点是字段较多，会让持仓快照承担更多 workflow 状态。

建议 Phase 1 使用独立存储，Phase 3 再评估是否合并。无论哪种路径，`tracking_only` 仍是是否真实持仓的最小兼容字段。

### 2. API 与工具

- 新增 Admin/Public-safe API：
  - `GET /api/watchlist-pipeline?actor=...`
  - `POST /api/watchlist-pipeline/items`
  - `PATCH /api/watchlist-pipeline/items/{symbol}`
  - `POST /api/watchlist-pipeline/items/{symbol}/review`
  - Public 端从 session 推导 actor，不接受任意 user_id。
- 扩展 `portfolio` tool 或新增 `watchlist_pipeline` tool：
  - 第一版推荐扩展 `portfolio(action="watch")` 的参数，支持 `thesis_reason`、`status`、`trigger_conditions`、`next_review_at`。
  - 若参数继续增多，再拆出 `watchlist_pipeline` tool，避免 `portfolio` tool 过载。
- `portfolio(action="add")` 从 watchlist 升级为 holding 时，读取 meta 并返回 `promoted_from_watchlist`、`watchlist_reason`、`trigger_conditions_matched`，让 finalizer 可以要求用户看到升级原因。

### 3. Event-engine 与 digest 集成

- `registry_from_portfolios` 继续把 watchlist symbol 加入订阅，保持现有“关注与持仓同级推送”契约。
- Router / digest 在 actor 命中 watchlist 时附加 `portfolio_relation=watchlist|holding`，供文案和优先级使用。
- `unified_digest::scheduler::actor_focus_symbols` 继续收集 watchlist，但 digest 生成时把 watchlist 分为“候选研究项”，避免用户误以为它是组合风险暴露。
- 事件命中 watchlist 的 trigger condition 时，把该项标为 `ready_for_review`，但不自动升级持仓。
- `mainline_distill` 对 watchlist symbol 可继续蒸馏，但 public UI 显示为“关注研究主线”；若缺画像，则状态进入 `needs_profile`。

### 4. 前端与产品面

- `packages/app/src/lib/types.ts` 增加 `WatchlistPipelineItem`、`WatchlistStatus`、`WatchTriggerCondition`。
- `packages/app/src/context/portfolio.tsx` 可继续提供 holdings/watchlist；新增 `useWatchlistPipeline` 或在 portfolio context 中并行读取 meta。
- `packages/app/src/components/portfolio-detail.tsx` 的 watchlist table 增加状态列、下一次复核、触发条件和质量 badge。
- `packages/app/src/pages/public-portfolio.tsx` 的 `DigestContext` 需要区分真实 holdings 与 watchlist tickers，避免页面标题和空态误导。

### 5. 与 company portrait / scheduled task 的联动

- `company_portrait` skill 可在创建画像后回填 `linked_profile_id` 或状态 `researching`。
- `scheduled_task` skill 可根据 `next_review_at` / `catalyst_date` 创建一次性或周期性复核任务，并在任务完成后更新 `last_reviewed_at`。
- 不允许 UI 直接编辑 profile 正文；UI 只显示 profile coverage 与“去 chat 更新画像”的入口。

## 实施步骤

### Phase 1: Model and Read-only Surfacing

- 新增 watchlist pipeline 类型和 actor-scoped 存储，或先实现纯派生 read model。
- 从现有 `PortfolioStorage` 中扫描 `tracking_only=true` 项，生成默认 `idea` 状态。
- Admin portfolio watchlist 表增加状态、缺原因、缺画像、未复核等只读 badge。
- Public `/portfolio` 区分 holdings 与 watchlist，修正文案和 summary。

### Phase 2: Agent and UI Mutation

- 扩展 `portfolio` tool 的 `watch` / `unwatch` / `add` 返回值，支持关注原因、复核日期和升级原因。
- 新增 Web/Admin API 写入 watchlist meta。
- Chat 中支持自然语言创建/更新关注项，回复必须总结关注原因和下一次复核。
- 升级为持仓时要求明确用户确认，并记录 `promoted_from_watchlist_at`。

### Phase 3: Event and Automation Loop

- Event-engine 对 watchlist 命中增加 relation metadata，推送文案区分关注/持仓。
- 命中 trigger condition 时标记 `ready_for_review`，并生成可选复核任务。
- Digest 加入“关注池复核”小节，默认只显示最高价值 3 项，避免噪音。

### Phase 4: Quality Metrics and Growth Loop

- Admin 增加 watchlist conversion metrics：关注新增数、复核完成率、升级率、放弃率、陈旧率。
- Public 端把“建立关注研究池”作为比真实建仓更低门槛的 activation step。
- 后续可把高质量 watchlist pipeline 接入 shareable briefs 或 collaborative rooms，但 v1 不做社交传播。

## 验证方式

### 自动化测试

- `memory` 单元测试：
  - 旧 portfolio JSON 没有 watchlist meta 时仍能加载。
  - `tracking_only=true` 项能生成默认 pipeline item。
  - 升级为 holding 后保留 promotion audit。
- `hone-tools` 测试：
  - `portfolio(action="watch")` 可写入关注原因和复核日期。
  - `portfolio(action="add")` 从 watchlist 升级时返回 promotion metadata。
  - `unwatch` 不误删真实持仓。
- `hone-web-api` 测试：
  - Public API 只能操作当前 session actor。
  - Admin API 可读取指定 actor。
  - 旧 portfolio summary 兼容。
- `hone-event-engine` 测试：
  - watchlist 仍进入 subscription watch pool。
  - watchlist 命中事件会带 relation metadata。
  - digest 不把 watchlist 误标为 holding exposure。
- `bun run test:web`：
  - public portfolio 页面区分 holdings/watchlist。
  - Admin watchlist pipeline 状态、空态和筛选逻辑稳定。

### 手工验收

- 在 Web chat 中说“关注 TEM，等下一次财报后看客户留存”，确认 portfolio watchlist 和 pipeline 状态都出现。
- 在 Admin 端看到 TEM 处于 `waiting_for_event`，有原因和下一次复核提示。
- 模拟事件命中 TEM，确认推送为“关注标的触发”，不是“持仓触发”。
- 将 TEM 升级为真实持仓，确认原关注原因被记录，并从 watchlist 移到 holdings。
- 将一个长期未复核 watchlist 项标记为 rejected，确认不再进入主动推送订阅或至少默认降到 digest/静默。

### 指标

- Watchlist items with reason rate。
- Watchlist items with next review rate。
- Stale watchlist ratio。
- Watchlist to holding promotion rate。
- Watchlist event noise complaints。
- Public `/portfolio` activation from empty state to first watchlist item。

## 风险与取舍

- **风险：watchlist 字段膨胀成另一个 portfolio 系统。**  
  取舍：v1 只做状态、原因、条件和复核，不做交易流水、成本、税务或真实仓位。

- **风险：推送变多。**  
  取舍：watchlist 虽同级进入订阅，但 relation metadata 应允许 digest/router 对关注项更保守；用户可把过期项标为 rejected 或降低复核频率。

- **风险：与 company portrait 直接编辑边界冲突。**  
  取舍：pipeline 只记录状态和链接，不直接编辑 portrait 正文；画像仍由 agent 文件能力维护。

- **风险：用户把 watchlist 条件理解为买入建议。**  
  取舍：文案必须坚持“复核条件 / 研究条件”，不输出“到价买入”式指令；升级为持仓必须由用户明确陈述已成交或明确确认。

- **风险：与已有 notification preferences 重叠。**  
  取舍：notification prefs 控制全局通知策略；watchlist pipeline 控制单个候选标的的研究状态和复核原因。

- **不做的边界：**  
  不接券商、不自动交易、不自动把关注升级为持仓、不做跨 actor 共享、不把 rejected 项物理删除为唯一行为。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案。结论：本提案不重复，差异如下：

- 不重复 `auto_p1_portfolio-exposure-radar.md`：exposure radar 处理真实组合风险、权重、数据质量和 scenario guardrails；本提案处理未建仓前的 watchlist 研究状态、复核条件和升级/放弃闭环。
- 不重复 `auto_p1_portfolio-transaction-ledger.md`：transaction ledger 处理账户级买卖流水、导入确认和当前持仓重建；本提案不记录交易事实，只记录候选标的为什么被关注以及何时复核。
- 不重复 `auto_p1_investment_context_intake.md`：context intake 解决新用户如何补齐 portfolio/profile/prefs/task；本提案假设 watchlist 已经成为日常使用对象，解决关注项的长期管理和转化。
- 不重复 `auto_p1_investment_playbook_launcher.md`：playbook launcher 是标准研究工作流入口；本提案是 watchlist 这个投资对象的状态模型和产品面。Playbook 后续可创建或推进 pipeline item，但不是同一层。
- 不重复 `auto_p1_evidence_review_queue.md`：evidence queue 处理外部证据是否进入画像复盘；本提案处理 watchlist item 的状态、条件、复核与 promotion。事件命中可进入 evidence queue，但 pipeline 负责候选标的生命周期。
- 不重复 `auto_p1_trade_discipline_journal.md`：trade journal 记录交易意图、纪律检查和复盘；本提案记录未交易前的关注理由和复核条件。只有升级为持仓时才与 journal 产生交集。
- 不重复 `auto_p1_delivery_decision_loop.md` / `auto_p1_end-user-notification-control.md`：它们关注通知为什么发送和用户如何控制通知；本提案为通知提供“关注 vs 持仓、等待什么条件”的上下文输入。
- 不重复 `auto_p1_company-portrait-health.md` / `auto_p1_cross-company-thesis-map.md`：它们关注画像质量和跨公司主线一致性；本提案只引用画像覆盖状态，不改变画像真相源。

差异结论：现有提案覆盖了 portfolio 风险、交易流水、上下文初始化、证据复盘、通知决策和研究工作流，但尚未覆盖“关注标的作为 pre-position 研究漏斗”的生命周期。这个主题直接贴合 Hone 的核心价值：在买入前帮助用户保持纪律，而不是只在买入后做组合追踪。

## 文档同步说明

本轮只新增 proposal，不开始实施，不修改业务代码、测试、模块边界、运行流程或长期规则，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。

若后续开始实施，应按动态计划准入标准新增或复用 `docs/current-plans/watchlist-conversion-pipeline.md`，并在落地存储、API、event-engine relation metadata、frontend 数据流或 tool schema 时同步更新 `docs/repo-map.md`、必要的 `docs/decisions.md` 和对应 handoff。
