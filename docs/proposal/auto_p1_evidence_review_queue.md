# Proposal: Evidence Review Queue for Thesis Updates

status: proposed
priority: P1
created_at: 2026-05-01 11:03:17 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `crates/hone-event-engine/src/router/dispatch.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-event-engine/src/unified_digest/types.rs`
- `crates/hone-event-engine/src/global_digest/thesis_distill.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/user-thesis-view.tsx`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/users.tsx`
- `skills/company_portrait/SKILL.md`
- `memory/src/company_profile/types.rs`

## 背景与现状

Honeclaw 已经形成了比较完整的投资研究资产链路：

- 公司画像以 actor sandbox 下的 `company_profiles/<profile_id>/profile.md` 和 `events/*.md` 为长期研究资产，`company_portrait` skill 要求把 thesis、证据、证伪条件和事件变化沉淀下来。
- `global_digest/thesis_distill.rs` 会只读公司画像，把每个持仓的长期 thesis 和整体投资风格蒸馏到 `NotificationPrefs`，供全球新闻 digest 和个性化过滤使用。
- `UnifiedDigestScheduler` 已经把 buffered 事件、财报倒计时、全球新闻、floor item、用户 thesis relation 和 curation 串成统一 digest。
- event router 会根据订阅、偏好、quiet mode、cap、cooldown 等规则把事件发送、排队、过滤或降级，并把结果写入 `delivery_log`。
- 用户端 `/portfolio` 能看到 thesis、整体风格和只读公司画像；管理端 `/users/:actor/thesis` 能代 actor 查看和触发蒸馏；管理端 `/notifications` 能排查 cron 与 event-engine 推送日志。

但目前这些能力仍缺少一个关键产品层：当一条事件可能改变长期判断时，系统只能“推送一次”或“进入 digest”，不能把它变成一个可复盘、可处理、可沉淀的研究待办。用户看到新闻、财报、SEC filing 或价格异动后，仍需要自己记得回到 chat，让 agent 更新画像；如果忘了，下一轮 thesis distill 仍读旧画像，digest 个性化也会继续基于旧 thesis 工作。

这与 Hone 的核心定位有冲突：Hone 不是资讯流，而是投资纪律和长期判断的维护系统。主动通知只解决“看到事件”，还没有解决“证据是否改变了我的 thesis，以及是否已经把这个变化写回长期记忆”。

## 问题或机会

当前缺口主要影响四条链路：

- 用户端：用户很难从一次性通知进入“这条证据是否需要更新我的公司画像”的复盘动作。长期资产维护依赖用户临时发起对话，路径不可发现。
- 管理端：管理员可以看到推送日志和 thesis 蒸馏状态，但无法看到“哪些 actor 的哪些 ticker 有待复盘证据、积压多久、是否已经处理”。
- 多渠道：Feishu / Telegram / Discord / iMessage 收到事件后，渠道消息天然易丢失；没有一个 Web/desktop 中央队列承接“稍后处理”。
- 系统可信度：如果 thesis distill 反复读到过期画像，global digest 的 personalize 结果会逐渐偏离真实持仓逻辑，用户会觉得 Hone 只会推消息，不会维护判断。

机会是：仓库已有事件存储、delivery log、company profile event 文档、thesis distill、public/admin thesis 视图和 notification log。无需重构 runner 或直接开放画像编辑器，就可以增加一个“证据复盘队列”，把事件引擎和长期研究记忆连接起来。

## 方案概述

新增一个 actor-scoped 的 Evidence Review Queue，用来记录“可能需要复盘的证据项”。每个证据项不是普通通知，而是一个可处理对象：

- 证据来源：event-engine `events` / `delivery_log` / unified digest selected item / 用户手动从 chat 标记。
- 关联对象：actor、ticker、event id、source、url、occurred_at、origin、当前 delivery status。
- 研究语义：candidate relation，例如 `confirms_thesis`、`counters_thesis`、`requires_update`、`noise_but_noted`、`unknown`。
- 处理状态：`open`、`snoozed`、`dismissed`、`sent_to_agent`、`profile_updated`。
- 用户决策：接受为 thesis-changing、驳回为噪音、稍后提醒、交给 agent 更新画像、加入事件备忘。

第一版应保持 conservative：

- 不让 UI 直接编辑 `profile.md` 或 `events/*.md`。
- 不自动改写公司画像。
- “交给 agent 更新画像”只创建一条带上下文的 agent prompt 或 chat draft，由现有 `company_portrait` skill 通过 runner 原生文件能力完成更新。
- 已处理结果可以写入 queue 自身和 delivery log 旁路审计，但长期画像仍以 Markdown 为源。

## 用户体验变化

### 用户端

在 `/portfolio` 或未来的用户工作台中增加“待复盘证据”区块：

- 顶部按 ticker 聚合显示 open 项数量、最旧积压时间、是否存在 counter-thesis 信号。
- 每条证据卡片展示标题、来源、时间、事件类型、命中的当前 thesis 摘要，以及“为什么值得复盘”的短理由。
- 提供四个主动作：`交给 Hone 更新画像`、`标记为噪音`、`稍后提醒`、`查看原始事件/画像`。
- 用户点击“交给 Hone 更新画像”后进入 chat，预填上下文：“请基于这条证据复盘 <ticker> 的公司画像；如果 thesis 未改变，只追加事件或说明不更新。”用户确认发送后由 agent 处理。

### 管理端

在 `/users/:actor/thesis` 或新增子 tab 中展示 Evidence Review Queue：

- 支持按 actor、ticker、status、event kind、age 过滤。
- 展示 open 项积压、平均处理时长、已转 agent 但未完成的数量。
- 能看到每个 ticker 最近一次 thesis distill 时间、画像 `last_reviewed_at`、open evidence 数量，帮助判断“蒸馏结果是否可能过期”。

### 桌面端

桌面不需要新 runtime 能力，但可以在侧边导航或 dashboard 上显示本地 backend 聚合出的 badge：

- “3 条证据待复盘”
- 点击进入 Web console 对应用户或当前 workspace 的证据队列。
- 在 bundled mode 下复用现有 backend API，不引入额外 sidecar。

### 多渠道

IM 通道收到高价值事件或 digest 时，不强行塞复杂交互，只提供轻量入口：

- 对 High 或 `ThesisRelation::Counter` 事件附一句短提示：“已加入待复盘证据，可在 Web/desktop 处理。”
- 用户在 IM 中回复“稍后复盘 / 这个不是噪音 / 更新画像”时，agent 可以调用队列工具定位最近相关 evidence，再走 `company_portrait` skill。

## 技术方案

### 1. 新增 evidence review 存储

建议在 event-engine SQLite 附近新增一个小表，或在 `memory` 下新增 actor-scoped SQLite/JSON store。优先放在 event-engine DB 中，因为 evidence 的主键和来源高度依赖 `events` / `delivery_log`：

```text
evidence_review_items (
  id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  ticker TEXT NOT NULL,
  event_id TEXT,
  source TEXT NOT NULL,
  url TEXT,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  event_kind TEXT,
  occurred_at_ts INTEGER NOT NULL,
  relation TEXT NOT NULL,
  reason TEXT,
  status TEXT NOT NULL,
  created_at_ts INTEGER NOT NULL,
  updated_at_ts INTEGER NOT NULL,
  snoozed_until_ts INTEGER,
  agent_session_id TEXT,
  profile_id TEXT,
  profile_event_id TEXT
)
```

`id` 可用 `evidence:{actor_hash}:{event_id}:{ticker}`，保证同一 actor/ticker/event 幂等。事件不存在时允许 `event_id=NULL`，用于 chat 手动标记或历史导入。

### 2. 从事件链路生成候选项

生成入口分三类：

- Router immediate：对 High 的 `EarningsReleased`、`EarningsCallTranscript`、`SecFiling`、可信来源 `NewsCritical`、大幅价格 band 生成候选。
- Unified digest：对 pass2 personalize 标记为 `ThesisRelation::Counter` 的 item 默认生成 open evidence；`Aligned` 可只在高严重度或用户开启时生成。
- 用户主动：通过 chat/tool 把最近事件或 URL 标记成 evidence，适合用户在 IM 中说“这个帮我留着复盘”。

第一版不需要复杂模型。可以用规则建立 `relation` 初值：

- `ThesisRelation::Counter` -> `counters_thesis`
- 财报 / SEC / earnings call -> `requires_update`
- 价格 band -> `unknown`，默认只在大仓位或超过用户阈值时生成
- `filtered` / `omitted` 事件默认不生成，避免队列变成垃圾桶；用户在 missed events 中主动选择才生成

### 3. API 与权限边界

新增 admin 与 public 两套 API：

- `GET /api/public/evidence-review`
- `POST /api/public/evidence-review/:id/action`
- `GET /api/evidence-review?channel=&user_id=&channel_scope=`
- `POST /api/evidence-review/:id/action`

public API 的 actor 必须来自 HttpOnly session，沿用 `public_digest.rs` 的 `require_public_actor` 模式。admin API 走 `require_actor` 或现有 actor query 解析。

action 不直接写画像，只更新 queue 或创建 agent draft：

- `dismiss`: `status=dismissed`
- `snooze`: 设置 `snoozed_until_ts`
- `send_to_agent`: 创建带 evidence payload 的 chat draft / transient prompt，并写 `agent_session_id`
- `mark_profile_updated`: 只有当能关联到新增 profile event 或 `profile.updated_at > item.created_at` 时才允许

### 4. Agent/skill 集成

新增一个轻量 tool，例如 `evidence_review`：

- list open items for current actor/ticker
- mark item status
- attach current session id

当用户在 chat 中要求更新画像时，turn builder 不需要特殊分支；tool 返回 evidence context 后，模型调用 `company_portrait` skill，仍按 `skills/company_portrait/SKILL.md` 的规则读写 Markdown。

关键约束：

- queue 只保存事件摘要和 URL，不保存 runner sandbox 绝对路径。
- 跨 actor 读取必须等 workspace proposal 落地后再扩展；第一版只读当前 actor。
- 不改变 `ActorIdentity` / `SessionIdentity` 语义。

### 5. Thesis distill 的 stale signal

`thesis_distill` 当前只读画像并写 prefs。可以先不改蒸馏逻辑，只在 UI 层展示 stale signal：

- 若某 ticker 存在 open `requires_update/counters_thesis` evidence，标记“thesis 可能待复盘”。
- 若 `profile.last_reviewed_at` 或 profile mtime 晚于 evidence `created_at`，可提示“画像已在证据后更新”。
- 下一阶段再让 distill 读取 queue 摘要，把 open counter evidence 作为风险提示输出到 skipped / stale metadata，而不是直接改变 thesis。

## 实施步骤

### Phase 1: 只读队列与候选生成

- 在 event-engine store 增加 evidence review 表、类型和幂等 upsert/list/update 方法。
- 在 router immediate 和 unified digest scheduler 的最小分支生成候选，只覆盖财报、SEC、counter-thesis global items。
- 增加 admin/public list API。
- 在管理端 thesis 视图显示 open evidence 列表和 stale badge。

### Phase 2: 用户动作闭环

- 增加 action API：dismiss、snooze、send_to_agent。
- 用户端 `/portfolio` 增加证据队列卡片与详情抽屉。
- chat 中支持从 evidence id 构造一条带上下文的画像复盘 prompt。
- 增加 `evidence_review` tool，让 IM 用户可查询和处理自己的 open evidence。

### Phase 3: Profile update 关联

- 当 agent 通过 `company_portrait` skill 追加 event 或更新 profile 后，允许工具把相关 evidence 标记为 `profile_updated`。
- 管理端展示 evidence -> profile event 的反向链接。
- 在 thesis distill UI 中显示“open counter evidence count”，提示当前 thesis 是否需要人工复盘后再信任。

### Phase 4: 指标与灰度

- 加入队列指标：生成数、dismiss 率、send_to_agent 率、profile_updated 率、平均处理时长。
- 默认只对直接 actor 和持仓 ticker 开启；watchlist 与 global macro evidence 作为后续开关。
- 根据 dismiss 率调低过度生成的 event kind，避免制造新噪音。

## 验证方式

- 单元测试：
  - evidence id 对同一 actor/ticker/event 幂等。
  - public actor 只能读取自己的 evidence。
  - action 状态机拒绝非法跳转，例如 dismissed 后不能直接 profile_updated。
  - `ThesisRelation::Counter` digest item 能生成 `counters_thesis` evidence。
- 集成/回归：
  - 构造一个持仓 actor、company profile、counter-thesis news，跑 unified digest 后能在 API 中看到 open evidence。
  - 触发 `send_to_agent` 后，chat draft/prompt 包含 event title、url、summary、当前 thesis 摘要和明确的 company_portrait 指令。
  - profile 更新后，关联 evidence 可被标记为 `profile_updated`，并在用户端不再显示为 open。
- 前端验收：
  - public `/portfolio` 在移动和桌面视口展示 evidence 卡片，不遮挡现有 thesis/profile 内容。
  - admin `/users/:actor/thesis` 能按 ticker/status 过滤。
  - 桌面 bundled mode 下通过同一 Web API 看到 badge，不需要额外进程。
- 指标：
  - evidence open 超过 7 天的比例下降。
  - counter-thesis evidence 的 agent 处理率和 profile_updated 率可观测。
  - 用户端从通知/digest 到 profile 更新的转化路径可追踪。

## 风险与取舍

- 风险：生成过多 evidence，用户把队列也视为噪音。取舍：第一版只覆盖财报、SEC、财报电话会和 digest counter-thesis，不把所有新闻都入队。
- 风险：用户误以为 evidence 自动更新了长期画像。取舍：状态文案必须区分 `sent_to_agent` 和 `profile_updated`，且 UI 不直接编辑 Markdown。
- 风险：把价格异动写成长期判断，违反公司画像约束。取舍：价格 band 默认只作为 `unknown` 或手动复盘入口，不自动标记 thesis-changing。
- 风险：新增状态表扩大 event-engine store 职责。取舍：evidence 与 event/delivery 高度相关，先靠 SQLite 事务和幂等 id 保持简单；若后续 workspace 抽象落地，再考虑迁到 memory 的 workspace-level store。
- 风险：agent 更新画像失败后 queue 状态不准确。取舍：`send_to_agent` 只代表已提交处理，不代表完成；完成必须由工具或 profile mtime/event link 显式确认。
- 不做：不重写 notification prefs、不替代 delivery decision loop、不做 UI 直接画像编辑器、不做跨 actor/workspace evidence 共享、不让模型自动改写 thesis distill 输出。

## 与已有提案的差异

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案解释“为什么推/没推、如何调偏好”；本提案处理“这条证据是否应该改变长期 thesis，以及是否已写回公司画像”。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案解决同一真实用户跨渠道资产归属；本提案第一版严格 actor-scoped，只做单 actor 内证据复盘。
- 与 `auto_p1_run_trace_workbench.md` 不重复：该提案面向 agent run/debug trace；本提案面向投资研究证据生命周期，不追踪 runner 内部执行。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案解决桌面启动/sidecar ownership；本提案只复用现有 desktop Web surface 展示 badge。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案解决 skill runtime 与 multi-agent 执行语义；本提案只新增一个可选 evidence tool，并继续通过既有 `company_portrait` skill 完成画像更新。

查重结论：现有提案覆盖通知解释、跨渠道身份、运行排障、桌面启动、skill runtime，但没有覆盖“市场证据 -> 人工复盘 -> agent 更新公司画像 -> thesis distill 可信输入”的闭环。因此本主题是新的、可落地的 P1 产品/架构提案。
