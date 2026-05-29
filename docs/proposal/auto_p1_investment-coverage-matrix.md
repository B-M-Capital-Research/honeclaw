# Proposal: Investment Coverage Matrix for Actor Readiness

status: proposed
priority: P1
created_at: 2026-05-30 02:05:03 +0800
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
- `docs/proposal/auto_p1_company-portrait-health.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_watchlist-conversion-pipeline.md`
- `docs/proposal/auto_p1_context-return-links.md`
- `docs/proposal/auto_p1_investment-thread-workbench.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `memory/src/portfolio.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/session.rs`
- `memory/src/web_auth.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/context/company-profiles.tsx`
- `packages/app/src/context/tasks.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/symbol-drawer.tsx`
- `skills/company_portrait/SKILL.md`
- `skills/scheduled_task/SKILL.md`
- `skills/stock_research/SKILL.md`

## 背景与现状

Honeclaw 已经从单点聊天扩展成一组围绕 actor 的投资工作台能力：

- `memory/src/portfolio.rs` 用 actor-scoped JSON 保存真实持仓和 `tracking_only` watchlist，同一个 API 层已经能按 actor 列出、读取、创建、更新、删除持仓。
- `memory/src/company_profile/` 与 `crates/hone-web-api/src/routes/company_profiles.rs` 提供 actor sandbox 内公司画像的列表、详情、导入、导出和删除；`docs/invariants.md` 明确公司画像是用户可见长期研究记忆的唯一主入口。
- `memory/src/cron_job/mod.rs` 与 `crates/hone-web-api/src/routes/cron.rs` 支持 actor 维度定时任务、channel target、执行历史和 heartbeat 类任务。
- `crates/hone-web-api/src/routes/public_digest.rs` 会为 public Web 用户读取 portfolio symbols、mainline prefs 和 actor sandbox 中已有画像，组成 `/portfolio` 投资上下文。
- `crates/hone-tools/src/notification_prefs_tool.rs` 已让用户在自己的渠道内管理推送偏好，且 overview 需要聚合 cron + digest 默认配置。
- 管理端 `packages/app/src/pages/users.tsx` 已把同一个 actor 的 portfolio、profiles、mainline、sessions、research 放在 tab 下；`SymbolDrawer` 也能把一个 symbol 关联到画像、研究任务、会话和 watchlist 操作。

这些能力已经能回答“某个 actor 有哪些资产”和“某个 symbol 在某个页面里有什么信息”。但系统还缺少一个更基础的产品问题：**这个 actor 的每个投资对象是否已经具备可持续跟踪的最小覆盖闭环？**

今天用户或管理员需要在 portfolio、company profiles、mainline、tasks、notifications、sessions、research 多个页面之间切换，才能判断：

- 持仓是否有公司画像。
- watchlist 是否只是一个 ticker，还是已有研究理由和复核任务。
- 某个公司画像是否已经被纳入 public digest 的 mainline。
- 某个重要持仓是否有 scheduled task 或 event-engine 推送路径。
- 某个多渠道用户是否已经把目标渠道和 quiet hours 配好。
- 某个用户反复聊过的公司是否还没有沉淀到画像或任务。

这不是单个缺字段问题，而是产品架构层缺少一个 actor/symbol 粒度的 **coverage readiness** 读模型。

## 问题或机会

这是 P1 级机会。它不直接改变投资建议边界，也不替代画像健康、组合暴露、deep link 或 playbook；但它能显著提升用户首次激活、管理员排障、推送准确性和后续 agent 自动化落地效率。

1. **用户不知道自己的投资上下文是否“装好”。**  
   Public `/portfolio` 能展示持仓、主线和画像，但缺少“还差什么才能让 Hone 持续跟踪这个标的”的明确状态。用户看见空画像、缺主线或没有任务时，需要自己猜下一步。

2. **管理员缺少 actor 级 readiness 总览。**  
   `/users/:actor` 已经有多个 tab，但它们是并列资产列表。排查“为什么这个用户没有收到有价值提醒”时，需要人工联查 portfolio、profiles、cron jobs、notification prefs、sessions 和 research tasks。

3. **agent 工作流难以判断补齐优先级。**  
   `company_portrait`、`scheduled_task`、`stock_research` 都是强能力，但当用户说“帮我把组合接入 Hone”时，系统没有一个确定性矩阵告诉 agent 先补哪些持仓画像、哪些 watchlist 过于松散、哪些任务缺 channel target。

4. **增长体验缺少可见进度条。**  
   对新用户来说，Hone 的价值来自“我已经把核心持仓接入一个长期研究系统”。如果没有 coverage 分数或缺口列表，用户无法感知从空白聊天到可复用工作台的进度。

5. **多渠道投递与研究资产的闭环关系不透明。**  
   Channel 设置、cron target、notification prefs、digest slots 和 company portraits 分散在不同模块。系统无法一眼告诉用户：AAPL 只有 Web 可见，没有 Telegram 推送；NVDA 有画像但没有复核任务；MU 有任务但最近 5 次执行失败。

机会是新增 **Investment Coverage Matrix**：一个 actor-scoped、symbol-indexed、只读优先的覆盖矩阵，把现有资产投影成最小闭环状态，并为用户、管理员和 agent 生成下一步补齐建议。

## 方案概述

新增 `InvestmentCoverageMatrix` 作为派生读模型。它不改变 portfolio、company profile、cron job、notification prefs 或 session 的真相源，只把它们按 actor + symbol 汇总成可解释的覆盖状态。

核心对象：

1. `CoverageSubject`
   - `symbol`
   - `subject_kind`: `holding`、`watchlist`、`profile_only`、`session_mentioned`、`task_only`
   - `portfolio_state`: 是否真实持仓、watchlist、缺成本、缺 horizon、缺 notes

2. `CoveragePillar`
   - `portfolio`: 是否存在 portfolio/watchlist 记录
   - `profile`: 是否有公司画像
   - `mainline`: 是否有 per-ticker mainline 或最近跳过原因
   - `automation`: 是否有相关 cron job、heartbeat、review task 或 playbook 来源
   - `delivery`: 是否有可用 channel target、notification prefs、digest slot、最近成功投递
   - `conversation`: 是否有最近会话或研究任务证据

3. `CoverageGap`
   - `missing_profile`
   - `missing_mainline`
   - `no_review_task`
   - `notification_disabled`
   - `target_missing`
   - `watchlist_no_reason`
   - `profile_without_portfolio_subject`
   - `task_without_symbol_anchor`
   - `session_mentions_not_distilled`

4. `CoverageReadiness`
   - `ready`: 最小闭环已具备。
   - `partial`: 可回答但跟踪链不完整。
   - `setup_needed`: 关键资产缺失。
   - `stale_or_broken`: 有资产但最近失败、跳过或无目标渠道。

5. `CoverageNextAction`
   - 低副作用补齐建议，例如 `create_profile_prompt`、`distill_mainline`、`create_review_task_draft`、`fix_channel_target`、`open_notification_prefs`、`convert_watchlist_reason`。

第一版只做确定性规则和只读展示；所有写入仍通过现有 chat/agent、任务 API、设置页或画像导入导出完成。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加紧凑的“覆盖状态”区块：
  - `6 个标的中 3 个已具备长期跟踪闭环`
  - `2 个持仓缺公司画像`
  - `1 个 watchlist 缺关注理由`
  - `1 个画像有主线但没有复核任务`
- 每个 ticker 卡片显示 coverage badges：`有画像`、`有主线`、`有任务`、`可推送`、`缺下一步`。
- 用户点击缺口后进入 chat draft，而不是直接静默改写。例如：“请为 MU 建立公司画像，并根据我的持仓说明投资主线和证伪条件。”
- 新用户可以把 coverage 当成 onboarding progress：先补 portfolio，再补核心画像，再开复核任务，再确认推送。

### 管理端

- `/users/:actor` 新增 `Coverage` tab，或在当前 actor header 下增加 summary strip：
  - 持仓覆盖率、watchlist 覆盖率、画像覆盖率、mainline 覆盖率、自动化覆盖率、投递覆盖率。
  - 按 severity 排序的缺口列表，支持筛选 `holdings only`、`watchlist`、`missing delivery`、`stale task`。
- `PortfolioDetail` 和 `CompanyProfileDetail` 可以在行级显示相同 coverage badges，减少管理员跨 tab 查找。
- 任务详情显示“本任务覆盖哪些 symbol / actor”，反向暴露 orphan task：任务存在但无法映射到任何持仓、关注或画像。
- 当用户反馈“推送不准/没推送”时，管理员先看 coverage matrix，再决定是补数据、改偏好、修 channel、还是调查 event-engine。

### 桌面端

- Desktop dashboard 显示一个可操作 readiness 摘要：“本地工作台 7 个投资对象，4 个 ready，3 个需补齐。”
- Bundled 模式全部使用本地 storage、actor sandbox 和 runtime config；remote 模式展示后端生成时间，避免用户误以为本地数据已经同步到云端。
- 托盘或首页可以把 top 3 gaps 做成待办入口，但不引入新的 desktop-only 存储。

### 多渠道

- 用户在 Feishu / Telegram / Discord 私聊中问“我的 Hone 还差什么没配好”时，agent 调用 coverage summary，只返回最重要的 3 个缺口和下一步。
- 群聊默认不暴露个人 coverage；只有 group actor 或明确绑定的 workspace scope 才能显示共享 coverage。
- 多渠道不发送完整矩阵，避免噪音和隐私泄露；完整视图引导回 Web/desktop。

## 技术方案

### 1. 派生 coverage evaluator

建议先在 `crates/hone-web-api` 增加纯读 evaluator，后续若需要 agent/tool 复用，再下沉到 `memory` 或独立 crate。第一版输入均来自已有存储：

- `PortfolioStorage::load(actor)`：真实持仓、watchlist、notes、horizon、updated_at。
- `company_profile_storage.for_actor(actor).list_profiles_raw()`：画像 id、ticker、updated_at、raw summary。
- `FilePrefsStorage::load(actor)`：per-ticker mainline、distill skipped、quiet hours、digest slots、enabled 状态。
- `CronJobStorage::list_jobs(actor)` 与 execution records：相关任务、target、enabled、最近结果。
- `SessionStorage` / `/api/users` 投影：最近会话中提到的 symbol，用于发现“聊过但未沉淀”的候选。
- `Research` context 中的本地任务索引：已启动 deep research 但没有画像或任务承接的对象。

第一版 symbol 匹配只使用保守规则：

- portfolio symbol 精确大写匹配。
- company profile ticker 从 metadata 或 public digest scan 已有 ticker 字段读取。
- cron task 仅匹配明确出现在 `task_prompt`、name 或 tags 中的大写 ticker，避免把普通英文单词误判为股票。
- session mention 只作为低置信 `session_mentioned` subject，不自动生成 ready 状态。

### 2. Coverage schema

示例响应：

```json
{
  "actor": { "channel": "web", "user_id": "u_123" },
  "generated_at": "2026-05-30T02:05:03+08:00",
  "summary": {
    "subjects": 8,
    "ready": 3,
    "partial": 2,
    "setup_needed": 2,
    "stale_or_broken": 1
  },
  "rows": [
    {
      "symbol": "MU",
      "subject_kind": "holding",
      "readiness": "partial",
      "pillars": {
        "portfolio": "present",
        "profile": "missing",
        "mainline": "missing",
        "automation": "missing",
        "delivery": "unknown",
        "conversation": "recent"
      },
      "gaps": ["missing_profile", "missing_mainline", "no_review_task"],
      "next_actions": ["create_profile_prompt", "distill_mainline", "create_review_task_draft"]
    }
  ],
  "limitations": ["symbol matching is exact-only in v1", "no market data required"]
}
```

### 3. API

新增只读路由：

- `GET /api/coverage?channel=&user_id=&channel_scope=`
- `GET /api/public/coverage`
- `GET /api/coverage/summary`

权限边界：

- Admin route 沿用 `require_actor`，未来可接 operator access audit。
- Public route 必须从 `hone_web_session` 推导 `channel=web` actor，不接受 actor query。
- 多渠道 tool 只允许读取当前 actor 的 coverage summary，不允许帮别人查询。

### 4. Agent/tool 集成

新增轻量 tool：`investment_coverage_tool`，只读返回：

- `get_summary`
- `list_gaps`
- `get_symbol(symbol)`
- `draft_next_action(symbol, action)`

`draft_next_action` 只生成 prompt draft 或任务草稿，不直接写画像、不直接启用渠道、不直接创建高风险自动化。实际写入仍经 `company_portrait`、`scheduled_task`、settings UI 或已有 API。

Agent 使用规则：

- 用户问“我还缺什么”时先读取 coverage summary。
- 用户要求“一次性帮我配好”时，先列 preview 和确认项，再逐项执行低风险补齐。
- 涉及真实持仓、通知、任务启用时，保留明确确认，不把 coverage 变成隐式交易或自动投递权限。

### 5. 前端落点

- `packages/app/src/lib/types.ts`：新增 `InvestmentCoverageMatrix` 类型。
- `packages/app/src/lib/api.ts`：新增 admin/public coverage client。
- `packages/app/src/pages/users.tsx`：新增 `coverage` tab 或 header summary。
- `packages/app/src/pages/public-portfolio.tsx`：读取 public coverage，渲染 readiness summary 和 ticker badges。
- `packages/app/src/components/symbol-drawer.tsx`：在 symbol drawer 内显示该 symbol 的 coverage row 和 next actions。
- `packages/app/src/context/portfolio.tsx` / `company-profiles.tsx` 不直接维护 coverage 状态，避免各 context 重复聚合；coverage 由独立 resource 拉取派生 API。

### 6. 兼容和迁移

无需迁移旧数据。旧用户第一次打开 coverage 时由后端即时计算：

- 缺 portfolio 时只显示 profile/session/task 维度，不创建空 portfolio。
- 缺 company profile 时标记 gap，不自动建档。
- 缺 notification prefs 时使用默认投影并标记 `delivery_unknown`，不自动启用推送。
- 缺 cron execution history 时只显示任务配置，不判定最近失败。

## 实施步骤

### Phase 1: Admin read model

1. 定义 coverage DTO、readiness、gap、pillar 枚举。
2. 实现 actor-scoped evaluator，聚合 portfolio、company profiles、prefs、cron jobs。
3. 增加 `GET /api/coverage`。
4. 为 evaluator 写单元测试：持仓缺画像、画像无持仓、watchlist 缺理由、任务缺 target、prefs disabled。
5. 在 `/users/:actor` 增加 coverage summary 与缺口表。

### Phase 2: Public readiness

1. 增加 `GET /api/public/coverage`，actor 从 public session 推导。
2. 在 `/portfolio` 增加 readiness summary、ticker badges 和 chat draft next actions。
3. 对空 portfolio / 未登录 / 缺画像做明确 fallback。
4. 增加前端 model tests，覆盖 summary 排序和 badge 显示。

### Phase 3: Agent and multi-channel summary

1. 增加只读 `investment_coverage_tool`。
2. 在 prompt 或 skill guidance 中要求“帮我配置 Hone / 还差什么”先查 coverage。
3. 在 Feishu / Telegram / Discord 私聊中允许返回 top gaps。
4. 增加 manual regression：一个本地 actor 含 portfolio/profile/task/prefs，覆盖 tool 输出和 Web 页面。

### Phase 4: Orphan and regression loops

1. 增加 task-only、profile-only、session-mentioned subjects，暴露未沉淀资产。
2. 把 execution history 接入 `stale_or_broken`，标记最近失败或 target_missing。
3. 与未来 playbook launcher / context return links 集成，把 next action 从文本升级为受控入口。

## 验证方式

- Rust 单元测试：
  - `holding + profile + mainline + enabled task` 应为 `ready`。
  - `holding` 缺 profile/mainline/task 应生成对应 gaps。
  - `tracking_only` watchlist 缺 notes 应生成 `watchlist_no_reason`，但不当成真实持仓暴露。
  - cron job 缺 `channel_target` 或 disabled 应影响 `automation` / `delivery` pillar。
  - profile-only 和 task-only 对象不应被误判为真实持仓。
- 前端测试：
  - coverage summary rows 按 severity 和 subject_kind 排序稳定。
  - public `/portfolio` 在缺 coverage API capability 时优雅降级。
  - symbol badge 不遮挡既有画像和 mainline 展示。
- 手工验收：
  - 创建一个 actor，分别添加持仓、watchlist、画像、mainline prefs、cron task，确认矩阵逐步从 setup_needed 到 ready。
  - public 登录用户只能看到自己的 coverage。
  - 私聊查询只返回当前 actor top gaps，群聊不泄露 direct actor coverage。
- 指标：
  - 新用户从登录到第一个 `ready` subject 的时间。
  - ready subjects / total holdings 的比例。
  - coverage gap 点击到 chat/task/settings 的转化。
  - 用户反馈“没有收到提醒”时，能否用 coverage 在 1 分钟内定位缺口类型。

## 风险与取舍

- **风险：矩阵变成又一个要维护的真相源。**  
  取舍：v1 只做即时派生，不落库存储；所有数据仍来自 portfolio、profile、prefs、cron、session。

- **风险：symbol 匹配误判。**  
  取舍：v1 只做精确 ticker 匹配和低置信 session mention，不用 LLM 猜公司名；需要更强识别时依赖 Instrument Identity Registry 类前置能力。

- **风险：coverage 分数被用户误解成投资质量评分。**  
  取舍：避免“好/坏股票”语言，只说“跟踪闭环是否完整”；不输出买卖建议。

- **风险：UI 过重，打断现有 portfolio/mainline 体验。**  
  取舍：public 端默认只显示摘要和 badge；完整表放 admin 或折叠面板。

- **风险：next actions 绕过确认。**  
  取舍：所有写入动作都以 prompt draft、task draft 或跳转开始，关键状态变化仍需用户确认。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 下全部 `auto_p*.md` 提案，以及历史目录 `docs/proposals/desktop-bundled-runtime-startup-ux.md`、`docs/proposals/skill-runtime-multi-agent-alignment.md`。

- 不同于 `auto_p1_company-portrait-health.md`：该提案评估单个公司画像文档质量和复审 cadence；本提案跨 portfolio/profile/mainline/task/delivery/session 聚合 actor + symbol 的覆盖闭环。
- 不同于 `auto_p1_portfolio-exposure-radar.md`：该提案关注组合集中度、成本权重、期权窗口和风险 guardrails；本提案不计算风险暴露，只判断跟踪系统是否配置完整。
- 不同于 `auto_p1_watchlist-conversion-pipeline.md`：该提案把 watchlist 升级为研究漏斗状态机；本提案把 watchlist 作为矩阵中的一种 subject，并检查它是否有后续研究与投递覆盖。
- 不同于 `auto_p1_context-return-links.md`：该提案建立跨 surface deep link 契约；本提案生成缺口和 readiness，后续可消费 deep link，但不解决 URL 路由本身。
- 不同于 `auto_p1_investment-thread-workbench.md`：该提案把多次会话组织成持续议题；本提案把会话只作为 coverage 的一个 evidence pillar，重点仍是 actor 投资对象是否具备长期跟踪闭环。
- 不同于 `auto_p1_investment_playbook_launcher.md`：该提案提供标准工作流启动器；本提案告诉用户和 agent 哪些对象需要启动什么补齐动作，可作为 playbook 的输入。
- 不同于 `auto_p1_runtime_readiness_matrix.md`：该提案判断模型、provider、sidecar、渠道等运行能力是否可用；本提案判断某个 actor 的投资对象是否完成 portfolio/profile/mainline/task/delivery/conversation 覆盖。

因此本提案的新主题是 **投资对象覆盖就绪度矩阵**：用现有数据源形成可执行、可验证的 setup/readiness 视图，减少用户和管理员在多个页面之间人工联查。
