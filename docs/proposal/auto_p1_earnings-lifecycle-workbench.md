# Proposal: Earnings Lifecycle Workbench for Pre/Post-Call Discipline

status: proposed
priority: P1
created_at: 2026-05-25 02:01:43 +0800
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
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_temporal-operations-calendar.md`
- `docs/proposal/auto_p1_investment-thread-workbench.md`
- `docs/proposal/auto_p1_company-portrait-health.md`
- `docs/proposal/auto_p1_factual_snapshot_cache.md`
- `crates/hone-event-engine/src/pollers/earnings.rs`
- `crates/hone-event-engine/src/pollers/earnings_surprise.rs`
- `crates/hone-event-engine/src/pollers/earnings_quality.rs`
- `crates/hone-event-engine/src/pollers/news.rs`
- `crates/hone-event-engine/src/unified_digest/sources/synth.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/router/classify.rs`
- `crates/hone-event-engine/src/router/policy.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/events.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `memory/src/company_profile/{mod.rs,types.rs,storage.rs}`
- `memory/src/portfolio.rs`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/notifications-model.ts`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/context/company-profiles.tsx`
- `packages/app/src/lib/api.ts`

## 背景与现状

Hone 的投资助理定位里，财报是最容易改变长期主线、组合风险和用户纪律的高价值节点。当前仓库已经具备不少底层能力：

- `EarningsPoller` 从 FMP earnings calendar 生成稳定的 `EarningsUpcoming` teaser，T-3/T-2/T-1 倒计时由 `UnifiedDigestScheduler` 的 synth source 在 digest slot 现算，避免 poller 漂移。
- `EarningsSurprisePoller` 只把 EPS surprise 当作触发器，必须结合近期 8-K 摘抄和 `EarningsQualityReviewer` 的 LLM 质量判断后才产出用户可见事件，避免 EPS-only 噪音。
- `news.rs` 已把 earnings call transcript 从普通新闻中拆成 `EarningsCallTranscript`，`EventKind`、renderer、notification prefs、router policy 和 global digest collector 都已经认识这个独立 kind。
- 公司画像是 actor sandbox 下的长期研究资产，事件引擎、digest mainline distill 和 public digest context 已经会读取画像和用户持仓来决定个性化推送。
- Web / public / desktop 端已有通知、持仓、公司画像、任务和 chat 页面，但没有一个专门围绕“某一次财报”的产品对象。

结果是，财报相关能力在系统里已经分布很广，却仍以事件流方式暴露：用户会收到“财报将至”“财报发布”“电话会纪要”或 digest 摘要，但看不到一个可持续推进的 lifecycle：财报前要验证什么，发布后哪些指标证实或证伪了原主线，电话会 Q&A 解决了哪些问题，最终有没有更新 company portrait 或投资政策。

## 问题或机会

1. **财报前缺少准备面。**
   Hone 可以知道持仓和 watchlist 的 upcoming earnings，也能推 T-3/T-1 倒计时，但目前没有把 company portrait 中的关键假设、风险、反证条件和用户关注点组织成一张 pre-call checklist。用户仍要临时翻聊天历史、画像和通知。

2. **财报后缺少闭环状态。**
   `EarningsQualityReview` 能把 8-K 摘抄转成质量判断，但它只是事件 payload 的一部分。系统无法稳定展示“这次财报是否已复盘、是否等待 transcript、是否需要更新画像、是否已生成 follow-up 任务”。

3. **电话会纪要是高价值材料，却没有与同一次财报绑定。**
   `EarningsCallTranscript` 已是独立事件 kind，但现状更像“另一条事件”。用户关心的是它补充了这次财报中的哪些未决问题，而不是又收到一条孤立推送。

4. **长期记忆与短期事件之间缺少可解释桥梁。**
   公司画像是长期资产，event-engine 是事实流，chat 是交互层。财报作为季度节奏的天然 review point，应该成为把三者串起来的高信任工作流。

5. **商业和留存价值明确。**
   对投资用户来说，财报季是最强复访理由。一个清楚的财报工作台能把 Hone 从“会提醒我”推进到“帮我守住研究纪律”，对付费转化、长期留存和专业口碑都更直接。

## 方案概述

新增 **Earnings Lifecycle Workbench**：一个 actor-scoped、以 ticker + fiscal period / earnings date 为核心的派生工作台。

第一版不创建新的财报事实真相源，而是从 EventStore、portfolio/watchlist、company profiles、notification prefs、sessions 和 cron execution history 派生 `EarningsLifecycle` read model。它把当前已分散的事件组织成以下阶段：

- `scheduled`：已发现 upcoming earnings teaser。
- `preparing`：进入 T-3/T-1 窗口，生成 pre-call checklist。
- `released`：已收到 earnings release / surprise / 8-K quality review。
- `transcript_pending`：财报已发布但电话会纪要尚未出现，或用户标记需等待管理层 Q&A。
- `transcript_ready`：出现 `EarningsCallTranscript` 事件。
- `review_required`：质量判断、transcript 或价格反应提示主线可能变化。
- `profile_update_pending`：已有 agent 草稿或 review action，等待用户确认是否写入画像。
- `closed`：已完成复盘，记录结论、证据 refs 和下一次 follow-up。

核心原则：

- 不直接让 UI 修改 company portrait；画像更新仍通过 agent 原生文件操作或已有导入/导出边界完成。
- 不替代通知、日程或 evidence queue；workbench 只是把同一次财报相关对象串成可执行面。
- 不把财报变成交易建议。文案和 prompt 仍遵守 `docs/invariants.md` 中的金融约束：关注长期主线、风险和证伪条件，不提供买卖指令。

## 用户体验变化

### 用户端 / Public Web

- `/portfolio` 或 `/me` 增加“Upcoming earnings”区块，只显示当前登录 actor 的持仓 / watchlist。
- 每个 ticker 展示：
  - 财报日期、T-3/T-1 状态、是否已有 release / transcript。
  - 基于 company portrait 的 3-5 个 pre-call questions，例如“毛利率改善是否持续”“订单能见度是否支持当前主线”。
  - `Awaiting transcript` / `Review required` / `Profile updated` 状态。
- 用户可以一键打开 chat，带入结构化 prompt：“复盘 NVDA 2026Q1 财报，重点检查这些 pre-call questions，不要给交易建议。”
- 财报结束后，用户看到一张 post-call card：关键证据、与旧主线的差异、是否建议更新画像、下一步提醒。

### 管理端

- 新增 `/earnings` 或在 notifications / research 下增加 tab：
  - 按 actor、ticker、日期、状态筛选 lifecycle。
  - 看到该生命周期引用的 raw events、digest item、quality review、transcript event、company profile refs 和最近相关 session。
  - 支持管理员触发“生成 pre-call checklist”“生成 post-call review draft”“标记无需复盘”等非破坏动作。
- 运营可以识别哪些用户财报季最活跃，哪些 ticker 缺少画像或 pre-call questions，从而做定向激活。

### 桌面端

- Tray / dashboard 展示最近 3 个财报节点和本地/远端 backend 状态差异：
  - 本地 bundled 模式可以提示“可读取本地 sandbox 画像并生成 checklist”。
  - remote 模式只显示服务端已有画像和事件，不暗示本地文件可见。
- 桌面通知点击后 deep link 到 lifecycle detail，而不是只打开普通 chat。

### 多渠道

- Feishu / Telegram / Discord 的财报倒计时和 release 推送追加短 deep-link 或短命令：
  - `/earnings NVDA` 查看本次财报状态。
  - “回复 `复盘` 让 Hone 基于本次财报和画像生成 review draft。”
- 对不支持 deep link 的通道，消息内展示状态摘要和下一步命令，不泄露本地绝对路径。

## 技术方案

### 1. Read model

新增派生类型：

```rust
pub struct EarningsLifecycle {
    pub lifecycle_id: String,
    pub actor: ActorIdentity,
    pub symbol: String,
    pub earnings_date: DateTime<Utc>,
    pub fiscal_period: Option<String>,
    pub status: EarningsLifecycleStatus,
    pub event_refs: Vec<String>,
    pub profile_ref: Option<String>,
    pub pre_call_questions: Vec<EarningsQuestion>,
    pub post_call_findings: Vec<EarningsFinding>,
    pub transcript_status: TranscriptStatus,
    pub review_required_reasons: Vec<String>,
    pub updated_at: DateTime<Utc>,
}
```

`lifecycle_id` 可先用 `earnings_lifecycle:{actor_key}:{symbol}:{earnings_date}`，后续若接入更可靠 fiscal period，再增加兼容映射表。

v1 可以完全按需派生，不落新表：

- 从 `EventStore::list_upcoming_earnings` 和近期 events 找 `EarningsUpcoming`、`EarningsReleased`、`EarningsCallTranscript`。
- 用 `SharedRegistry` / portfolio actor 列表确定 actor 是否关注该 symbol。
- 从 company profile storage 读取 matching ticker / title / aliases。
- 从 payload 中读取 `earnings_quality_review`、`earnings_quality_context_url` 等字段。

v2 再考虑增加轻量 cache 表，保存人工状态和生成结果：

```sql
CREATE TABLE earnings_lifecycle_notes (
  lifecycle_id TEXT PRIMARY KEY,
  actor_key TEXT NOT NULL,
  symbol TEXT NOT NULL,
  earnings_date TEXT NOT NULL,
  status_override TEXT,
  pre_call_questions_json TEXT NOT NULL DEFAULT '[]',
  post_call_findings_json TEXT NOT NULL DEFAULT '[]',
  profile_update_state TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 2. API

新增 routes：

- `GET /api/earnings-lifecycle?actor=...&from=&to=&status=`
- `GET /api/earnings-lifecycle/:id`
- `POST /api/earnings-lifecycle/:id/pre-call-draft`
- `POST /api/earnings-lifecycle/:id/post-call-draft`
- `POST /api/earnings-lifecycle/:id/close`
- `GET /api/public/earnings-lifecycle?window=30d`
- `POST /api/public/earnings-lifecycle/:id/chat`

草稿接口不直接写画像，只生成 session prompt / draft artifact，交给 agent 和用户确认。

### 3. Agent workflow

新增一个轻量内置 prompt 或扩展 `company_portrait` skill 的引用材料：

- Pre-call：
  - 读取当前 company profile。
  - 提取 3-5 个本次财报应验证的问题。
  - 明确旧主线、风险、反证条件和用户持仓敏感点。
- Post-call：
  - 输入 release event、quality review、transcript event、相关 price reaction 和旧 profile 摘要。
  - 输出“主线保持 / 需要观察 / 需要更新画像”的草稿。
  - 若建议更新画像，生成 agent-mediated file edit plan，而不是 UI 直接写。

### 4. Event matching

匹配规则要保守：

- `EarningsUpcoming` 的 `occurred_at.date` 是 lifecycle anchor。
- `EarningsReleased` 先按 symbol + `occurred_at` 附近窗口匹配最近 upcoming；没有 upcoming 时创建 release-only lifecycle。
- `EarningsCallTranscript` 按 symbol + release 后 0-10 天窗口匹配；无法匹配时仍显示为 transcript-only item，避免丢证据。
- 同一 symbol 多个 upcoming date 时，优先最近未关闭 lifecycle。

### 5. UI state model

前端新增纯 model helper，避免页面堆逻辑：

- `packages/app/src/pages/earnings-lifecycle-model.ts`
- 状态排序：`review_required` > `released` > `transcript_pending` > `preparing` > `scheduled` > `closed`
- Badge 映射复用 notifications event kind 文案，但 detail 页面展示 lifecycle 阶段而不是 raw event kind。

## 实施步骤

1. **派生模型与 API skeleton**
   - 在 `crates/hone-web-api` 增加 earnings lifecycle route。
   - 从 EventStore + portfolio registry + company profile storage 拼出只读列表。
   - 不落表、不写 agent prompt，先验证 lifecycle grouping 正确。

2. **前端只读工作台**
   - 管理端新增 `/earnings` tab。
   - Public portfolio 增加 upcoming / review required 模块。
   - 展示 raw event refs、profile presence、transcript status 和 quality review summary。

3. **Pre-call checklist draft**
   - 增加 server endpoint，调用现有 runner transient task 或返回 chat deeplink prompt。
   - 从 company profile 派生 questions，不生成买卖建议。
   - 将 draft 作为 session message 或 artifact 引用，便于后续追踪。

4. **Post-call review draft**
   - 合并 earnings release、quality review、transcript、price reaction 和旧画像。
   - 输出 profile update proposal，由用户在 chat 中确认。
   - 需要和未来 evidence review queue / mutation ledger 对接时，只写 adapter，不把本 proposal 阻塞在那些系统落地上。

5. **多渠道入口**
   - 在 earnings 相关推送 renderer 中追加短命令或 deep link。
   - 确认 Feishu / Telegram / Discord / Web 的消息长度、路径脱敏和 fallback 文案。

6. **状态缓存与关闭动作**
   - 若只读模型稳定，再加 `earnings_lifecycle_notes` 缓存人工状态、draft refs 和 close 状态。
   - 加导出 / 支持包中的脱敏摘要，避免 support bundle 暴露完整 transcript。

## 验证方式

- 单元测试：
  - 给定 upcoming + release + transcript fixtures，lifecycle grouping 产出同一个 `lifecycle_id`。
  - release-only / transcript-only 场景不会丢事件。
  - 多个同 symbol upcoming date 时匹配最近未关闭 lifecycle。
  - `pre_call_questions` 在无 company profile 时返回明确 `profile_missing` 状态。
- API 测试：
  - 管理端 actor query 只能返回指定 actor 命中的 holdings/watchlist。
  - Public endpoint 只返回当前登录 web actor 的数据。
  - 草稿接口不直接修改 `profile.md`。
- 前端测试：
  - model helper 对状态排序、badge 文案、空态和 deep link 生成做测试。
  - mobile viewport 下 upcoming card 不遮挡日期、ticker 和 action。
- 回归脚本：
  - CI-safe fixture：插入 AAPL upcoming、release、transcript，验证 `/api/earnings-lifecycle` JSON shape。
  - 手工回归：用真实 FMP/SEC 配置跑一只持仓 ticker 的 end-to-end read model。
- 产品指标：
  - 财报节点打开率、pre-call draft 启动率、post-call review 完成率。
  - 财报后 company portrait 更新率。
  - 财报季 public/desktop 留存和通知点击回流。

## 风险与取舍

- 风险：财报匹配可能因为 fiscal period 缺失而误配。
  取舍：v1 使用 symbol + date window，并把低置信匹配显示为 `needs_review`，不自动写长期记忆。
- 风险：工作台与 evidence review queue、investment thread、temporal calendar 概念重叠。
  取舍：本提案只处理“单次财报生命周期”。跨主题议题、通用证据队列和时间日历仍由对应系统承担。
- 风险：Pre-call checklist 可能诱导短线押注。
  取舍：prompt 固定为长期主线验证，不给交易建议，不展示“买入/卖出”按钮。
- 风险：LLM 生成的 post-call review 可能过度解读。
  取舍：必须附 raw event refs 和旧 profile refs；没有 release/transcript 证据时只能给 `insufficient_evidence`。
- 风险：UI 增加复杂度。
  取舍：先在 portfolio 和 admin research 下做一个窄入口，不扩成新的全局导航中心。
- 风险：Hosted public 和本地 desktop 文件可见性不同。
  取舍：所有草稿接口都通过服务端已知 actor storage 派生；desktop 只额外提示本地可见性，不让前端直接读本地 sandbox。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点核对了包含 earnings、财报、transcript、calendar、evidence、thread、portrait、snapshot、notification 的主题。

- 不重复 `auto_p1_evidence_review_queue.md`：该提案把财报、SEC、counter-thesis 等事件送入通用复盘队列；本提案围绕单次财报建立 pre-call、release、transcript、post-call、profile update 的 lifecycle 工作台。
- 不重复 `auto_p1_temporal-operations-calendar.md`：calendar 解决“未来会发生什么、何时触发”；本提案解决“这一场财报从准备到复盘进行到哪一步”。
- 不重复 `auto_p1_investment-thread-workbench.md`：thread workbench 管理长期投资议题；本提案是季度财报这个高频、结构化、可自动匹配的专用工作流。
- 不重复 `auto_p1_company-portrait-health.md`：portrait health 检查画像本身是否过期或缺证据；本提案使用财报节点驱动画像复盘和更新草稿。
- 不重复 `auto_p1_factual_snapshot_cache.md` / `auto_p1_source-provenance-freshness.md`：它们处理工具结果、来源和事实可回放；本提案消费这些证据来组织用户体验和执行状态。
- 不重复 `auto_p1_notification-policy-backtest.md` / `auto_p1_delivery_decision_loop.md`：它们关注推送策略是否正确；本提案关注用户点击财报推送后进入的持续研究工作流。
- 不重复 `auto_p1_investment_playbook_launcher.md`：playbook launcher 是通用研究流程启动器；本提案是由真实 earnings event 自动挂载的产品对象和状态机。

差异结论：现有提案已经覆盖财报事件的通知、复盘队列、时间可见性、长期议题和事实证据，但还没有把同一次财报组织成一个端到端 lifecycle。这个主题直接服务 Hone 的核心承诺：在最容易情绪化和信息密集的财报季，帮助用户用既有主线、证据和复盘纪律做判断。
