# Proposal: Investment Thread Workbench for Ongoing Research Continuity

status: proposed
priority: P1
created_at: 2026-05-24 14:03:14 +0800
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
- `docs/proposal/auto_p1_session-memory-correction.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `packages/app/src/context/sessions.tsx`
- `packages/app/src/components/session-list.tsx`
- `packages/app/src/components/admin-chat-shell.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/app.tsx`

## 背景与现状

Honeclaw 已经具备多入口、长期投资记忆和主动任务能力，但当前“会话”仍主要按运行时归属来组织：

- `memory/src/session.rs` 以 `Session` 保存消息、actor、session identity、summary、runtime prompt state 和 metadata；JSON 仍是权威，SQLite 是可选索引 / runtime backend。
- `memory/src/session_sqlite.rs` 已把 session、message、metadata、last message preview、last role、message count 等投影进 SQLite，并支持按 channel 查找中断会话。
- `crates/hone-web-api/src/routes/users.rs` 的 `/api/users` 列出所有 session，核心排序和预览来自最后一条 user / assistant 消息。
- `crates/hone-web-api/src/routes/history.rs` 的 `/api/history` 返回压缩边界后的最近消息窗口，并提取附件和本地图像 marker。
- `packages/app/src/components/session-list.tsx` 管理端会话列表支持搜索、channel 过滤、actor/session badge、未读点和最后消息预览。
- `packages/app/src/components/admin-chat-shell.tsx` 在会话顶部提供跳转到当前 actor 的 portfolio、profiles 和 tasks 的按钮。
- Public `/chat` 维护最近历史和当前对话体验，但它仍以线性聊天为核心；用户要回到某个长期议题，主要依赖最近历史、浏览器状态和自己记忆。
- Public `/portfolio` 与公司画像解决“长期投资事实和主线在哪里”，Research Artifact Library 提案解决“报告交付物在哪里”，Session Memory Correction 提案解决“压缩摘要错了怎么纠正”。但这些都没有把多次聊天里反复出现的投资问题组织成一个可持续推进的“议题”。

这导致一个产品空档：Hone 的核心使用场景不是一次性问答，而是围绕某个公司、行业、宏观假设或持仓纪律反复追问、补证据、设任务、复盘结论。当前代码能保存每次会话，却不能把“这几次会话其实都在推进同一个投资议题”作为一等对象。

## 问题或机会

这是 P1 级机会，因为它直接影响核心体验、用户留存、管理端排障和长期研究资产的使用效率。

1. **会话列表回答的是“谁最近说了什么”，不是“哪些投资问题正在推进”。**  
   管理端 `/sessions` 和 public chat 历史适合找最近聊天，但不适合回答“MU 财报后复盘进行到哪一步”“NVDA 估值争议还有哪些未决问题”“上周关于利率路径的结论后来有没有更新”。

2. **长期资产和聊天上下文之间缺少议题桥梁。**  
   公司画像保存稳定主线，research artifact 保存报告，portfolio 保存持仓，cron 保存任务。用户实际会在聊天里产生很多中间状态：待验证假设、反证条件、下一次财报前要问的问题、某个结论是否已写入画像。这些状态既不适合直接写入公司画像，也不应只埋在 session transcript 中。

3. **跨渠道连续性仍偏 actor/session 级。**  
   `ActorIdentity` 和 `SessionIdentity` 的隔离是正确的，但用户会从 Web、桌面、Feishu、Telegram 或 Discord 回到同一个研究议题。Linked User Workspace 解决身份绑定，本提案解决绑定前后都需要的“同一议题如何被识别、继续、归档”。

4. **用户复访缺少明确入口。**  
   Public `/portfolio` 展示主线后，用户下一次打开 Hone 时不一定想继续“空白聊天”。更自然的入口是：继续上次的 MU 复盘、查看待验证假设、把一个已解决议题沉淀到画像、启动下一次提醒。

5. **管理端无法按议题判断价值和卡点。**  
   管理员可以看 session、users、profiles、tasks、research，但很难看到某个 actor 的活跃投资议题：哪些已经有画像支撑、哪些反复聊天但没有沉淀、哪些生成过报告但未 handoff、哪些需要创建 scheduled task。

机会是新增 **Investment Thread Workbench**：一个从 session transcript、company portraits、portfolio、research artifacts 和 tasks 派生的 actor-scoped 议题层。它不替代现有真相源，只把“正在推进的投资问题”产品化。

## 方案概述

新增 `InvestmentThread` 作为轻量、可派生、可手动修正的研究连续性对象。

核心能力：

1. **议题索引**
   - 从会话、附件、profile 引用、research artifact、portfolio symbol 和 cron task 中提取候选 thread。
   - 每个 thread 有标题、scope、相关 tickers、来源 sessions、最近活动、状态和未决问题。

2. **继续推进入口**
   - 用户可以从 public `/chat`、`/portfolio`、admin `/sessions` 或 desktop dashboard 打开某个 thread。
   - 打开后进入普通 chat，但 prompt 中带入 thread compact context，而不是整段历史。

3. **议题状态**
   - `active`：仍在研究或等待事件。
   - `waiting_evidence`：等待财报、公告、价格触发、用户补材料或外部数据。
   - `ready_to_distill`：已有足够结论，可交给公司画像或 mainline distill。
   - `archived`：用户确认暂时结束，保留可搜索索引。

4. **资产连接**
   - Thread 可以关联公司画像、research artifact、document、cron job、portfolio holding 和 session id。
   - 关联是引用关系，不把这些资产复制进 thread。

5. **可解释摘要**
   - Thread summary 只保存议题级状态：当前问题、已确认结论、反证条件、下一步。
   - 不替代 session compact summary，也不成为公司画像真相源。

第一版可以先做只读派生 + 手动 pin / archive，避免引入重型项目管理系统。等派生质量稳定后，再允许 agent 创建 thread draft 或更新 thread state。

## 用户体验变化

### 用户端

- Public `/chat` 历史侧栏增加 `Threads` 视图：
  - `MU 财报后复盘`
  - `NVDA 估值假设`
  - `利率路径对长久期持仓的影响`
  - `本周需要补证据的标的`
- 用户点击 thread 后，聊天 composer 显示当前议题标题和相关资产，不需要翻找旧消息。
- Public `/portfolio` 的每个 ticker 卡片可显示活跃 thread 数和最近一个未决问题，例如“等待 2026Q2 毛利率指引验证”。
- 当用户在聊天里完成阶段性结论，Hone 可以建议：
  - `把本议题沉淀到公司画像`
  - `创建下一次复盘提醒`
  - `归档本议题`
- 用户仍可直接聊天。Thread 是组织层，不强迫所有对话先选择项目。

### 管理端

- `/sessions` 增加 thread lens：按 actor、ticker、status、channel、recent activity 过滤。
- `AdminChatShell` 顶部除 portfolio / profiles / new task 外，增加当前 session 关联的 thread 和 `Open thread`。
- `/users/:actorKey` 可增加 `Threads` tab，展示：
  - 活跃议题
  - 反复聊天但未关联画像的议题
  - 有报告但未 handoff 的议题
  - 已归档但最近又被提到的议题
- 管理员可以合并重复 thread、修正 ticker、归档无价值 thread，但不直接改公司画像正文。

### 桌面端

- Desktop dashboard 显示 `Continue` 区块：最近 3 个活跃投资议题和下一个待办。
- Bundled 模式完全使用本地 session / profile / task 数据；remote 模式显示后端计算时间和来源。
- 桌面托盘可提供“继续最近议题”入口，比打开空白 chat 更贴近工作台体验。

### 多渠道

- 用户在 Feishu / Telegram / Discord 私聊中说“继续 MU 那个问题”，系统可以通过 actor 最近 thread 找到候选并追问确认。
- 群聊 thread 默认属于 group `SessionIdentity`，不泄露个人 direct thread。
- 多渠道回复只输出 thread 摘要和下一步，不把完整历史塞进 IM。

## 技术方案

### 1. 新增 Thread 类型与派生索引

建议先在 `memory` 增加 `investment_thread` 模块，使用 SQLite 存 thread metadata 和可手动修正的状态；候选生成可从现有 session index 派生。

核心表：

```text
investment_threads (
  thread_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  related_symbols_json TEXT NOT NULL,
  summary_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived_at TEXT,
  source_kind TEXT NOT NULL
)

investment_thread_links (
  thread_id TEXT NOT NULL,
  link_kind TEXT NOT NULL,
  link_id TEXT NOT NULL,
  label TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY(thread_id, link_kind, link_id)
)

investment_thread_events (
  event_id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL,
  event_kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL
)
```

`source_kind` 第一版包含：

- `derived_from_session`
- `manual_pin`
- `agent_suggested`
- `imported_from_research_artifact`

### 2. Thread Candidate 生成

第一版不需要 LLM 全量重扫，可用规则 + 轻量文本抽取：

- 从 `session_messages.content` 和 `/api/users` preview 提取 ticker-like token，并和 portfolio / company profile 的已知 symbol 交叉验证。
- 从 session metadata 中读取 invoked skills、tool call、attachment marker、research references。
- 对含有 `profile.md`、`company_profiles`、`research`、`report`、`财报`、`估值`、`风险`、`复盘` 等关键词的会话生成候选。
- 同一 actor、同一 primary symbol、近 30 天内相似标题可合并为一个 candidate thread。

后续可引入低频 background distill：

- 每日或用户打开 workbench 时，对活跃 candidate 用 auxiliary model 生成 thread title、open questions 和 next action。
- 结果先进入 `agent_suggested`，用户或 admin 确认后变成 active thread。

### 3. API 设计

Admin：

- `GET /api/admin/threads?actor=&status=&symbol=&q=`
- `GET /api/admin/threads/:thread_id`
- `POST /api/admin/threads/:thread_id/pin`
- `POST /api/admin/threads/:thread_id/archive`
- `POST /api/admin/threads/:thread_id/merge`
- `POST /api/admin/threads/:thread_id/link`

Public：

- `GET /api/public/threads`
- `GET /api/public/threads/:thread_id`
- `POST /api/public/threads/:thread_id/archive`
- `POST /api/public/chat` 可接受可选 `thread_id`，后端把 thread compact context 作为 session-fixed supplement 或 turn input 注入。

兼容策略：

- 没有 thread_id 的聊天保持现状。
- Thread API 只返回当前 public web actor 的数据，不允许用户指定任意 actor。
- Admin 可跨 actor 读取，但仍按现有管理端权限边界。

### 4. Prompt 与 Session 关系

Thread context 应是独立于 session compact summary 的轻量材料：

```text
当前投资议题:
- 标题: MU 财报后复盘
- 相关标的: MU
- 当前问题: 毛利率改善是否来自结构性供需变化，还是短期库存周期
- 已确认: 用户关注长期主线，不需要短线买卖建议
- 下一步: 等待 2026Q2 指引，补充数据中心收入和库存天数
```

注入规则：

- 只在用户显式打开 thread、或当前 session 已绑定 thread 时注入。
- 不把完整 thread event log 注入 prompt。
- 如果相关 company profile 存在，仍以 profile 为长期事实优先；thread 只描述“本议题还在推进什么”。
- 如果 thread 与当前 session actor 不匹配，拒绝注入。

### 5. 前端落点

- `packages/app/src/context/sessions.tsx`：增加 `currentThreadId` 和发送时的 thread 参数，但保持无 thread 的旧路径。
- `packages/app/src/components/session-list.tsx`：加 `Sessions / Threads` segmented control，thread row 展示 title、symbols、status、last activity、source badges。
- `packages/app/src/components/admin-chat-shell.tsx`：显示当前 thread link 与打开 workbench 按钮。
- Public `/chat`：历史侧栏增加 active threads，移动端放入抽屉。
- Public `/portfolio`：ticker 卡片展示 active thread badge。

## 实施步骤

### Phase 1: Read-only Thread Index

- 新增 thread 类型、SQLite schema 和扫描 session index 的候选生成器。
- 只读 API 返回 actor-scoped thread candidates。
- 管理端先展示 candidate 列表和 source sessions，不允许 agent 写入。

### Phase 2: Pin / Archive / Link

- 支持用户或 admin 把 candidate pin 成 active thread。
- 支持 archive、修改标题、修正 symbol、链接 profile / session / research artifact / task。
- Public `/chat` 和 admin shell 可打开 thread。

### Phase 3: Thread-aware Chat

- `POST /chat` 增加可选 `thread_id`。
- turn builder 或 Web API 在进入 `AgentSession::run()` 前注入 compact thread context。
- Chat 回答完成后更新 thread activity 和可选 open question。

### Phase 4: Distill and Handoff

- 对活跃 thread 增加低频 distill：生成 open questions、resolved claims、next action。
- 支持 `ready_to_distill` 到 company portrait / research artifact handoff 的 prompt。
- 与 scheduled task 集成：从 thread 创建复盘提醒或等待证据提醒。

## 验证方式

- 单元测试：
  - ticker / symbol candidate extraction 不把普通大写词误判为股票。
  - 同一 actor 相似 session 合并，同名不同 actor 不合并。
  - group session thread 不泄露 direct actor thread。
  - archive / merge / link 操作幂等。
- Web API 测试：
  - public user 只能读取自己的 threads。
  - admin thread query 能按 actor、status、symbol 过滤。
  - chat 传入非法 thread_id 返回明确错误。
- 前端测试：
  - session list 在 Sessions / Threads 两种视图下都能过滤和选择。
  - public chat 在无 thread 时保持现有行为。
  - portfolio ticker badge 正确处理无 thread、active thread、archived thread。
- 手工验收：
  - 创建三次围绕同一 ticker 的会话，候选 thread 能聚合并显示来源。
  - 打开 thread 后继续聊天，回答能引用 thread 当前问题，但不误称它是长期画像事实。
  - 将 thread archive 后，默认列表不显示，但搜索仍可找到。
- 指标：
  - thread 打开后的二次会话率。
  - thread 到 company portrait handoff 的转化数。
  - 用户从 public `/portfolio` 点击继续议题的比例。
  - 管理端排查“找不到上次讨论”的时间下降。

## 风险与取舍

- **风险：把聊天议题误当长期投资事实。**  
  取舍：thread summary 只描述进行中问题和下一步；长期事实仍以 company portrait、portfolio、research artifact 为优先。

- **风险：自动聚合错误导致串题。**  
  取舍：第一版使用 candidate / pin 流程，不自动把所有候选注入 prompt；只有用户显式打开或确认的 thread 才影响聊天。

- **风险：增加又一个信息架构层。**  
  取舍：thread 只解决“持续议题”，不承载报告全文、附件治理、交易流水、数据导出或运行排障；这些继续由已有模块负责。

- **风险：跨渠道身份未绑定时体验不完整。**  
  取舍：第一版按 actor 工作；Linked User Workspace 落地后再提供 workspace 聚合视图。

- **风险：后台 distill 成本。**  
  取舍：v1 规则生成 + 手动确认；v2 低频 auxiliary distill，且只对活跃 thread 运行。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下全部现有提案，并重点核对以下相邻主题：

- 不重复 `auto_p1_session-memory-correction.md`：该提案处理 session compact summary 可见、纠错和恢复；本提案处理跨多次会话的投资议题组织和继续推进入口。
- 不重复 `auto_p1_research_artifact_library.md`：该提案把深度研究报告变成长期交付物；本提案把聊天中的进行中问题组织成 thread，并可链接报告。
- 不重复 `auto_p1_investment_context_intake.md`：该提案解决新用户投资上下文初始化和缺口修复；本提案解决已有上下文之后如何持续推进议题。
- 不重复 `auto_p1_linked-user-workspace.md`：该提案解决跨渠道真实用户绑定；本提案保持 actor 边界，提供 actor 内和未来 workspace 内都可用的议题层。
- 不重复 `auto_p1_run_trace_workbench.md`：该提案服务运维排障和运行证据；本提案服务用户和管理员围绕投资问题继续研究。
- 不重复 `auto_p1_evidence_review_queue.md` / `auto_p1_mainline-distill-ledger.md`：它们处理证据到画像 / 主线的审阅与更新；本提案处理“证据进入长期资产之前的活跃研究议题容器”。

查重结论：现有提案覆盖了会话记忆纠错、研究报告资产、上下文初始化、跨渠道身份、运行追踪、证据复盘和画像沉淀，但没有覆盖“把多次聊天中反复推进的投资问题组织成可继续、可归档、可 handoff 的议题工作台”。因此本主题是新的、可落地的 P1 产品/架构提案。

## 本轮文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md` 或 `docs/invariants.md`。如果后续实际落地该提案，需新增 current plan，并在引入 thread storage、API、prompt 注入或前端导航时同步更新 repo map；若 thread context 变成长期行为契约，还需补充 invariants 或 decision。
