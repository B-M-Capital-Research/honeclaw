# Proposal: Daily Brief Workspace for Active Investment Review

status: proposed
priority: P1
created_at: 2026-05-29 08:05:25 +0800 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/dashboard.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/notifications.tsx`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-event-engine/src/unified_digest/types.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-event-engine/src/digest/render.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`

## 背景与现状

Hone 当前已经具备多条很有价值但分散的链路：

- Public surface 有 `/chat`、`/me`、`/portfolio`。其中 `/portfolio` 通过 `GET /api/public/digest-context` 展示当前 Web actor 的持仓、投资主线、`mainline_style`、上次蒸馏时间和 sandbox 内公司画像列表，并允许用户手动刷新主线。
- Event engine 已经有 unified digest：`DigestSlot` 支持用户自定义 digest 时刻，scheduler 会合并 per-actor buffer、earnings countdown、global news，做 floor / curation / cap，再通过 `send_digest` 发往多渠道，并把 `digest`、`digest_item`、`global_digest_item` 写入 delivery log。
- 管理端 `/notifications` 通过 `GET /api/admin/notifications` 合并 cron run 和 event-engine delivery log，适合 operator 排查 24 小时推送、失败、跳过和去重情况。
- 管理端 dashboard 更偏本地/桌面控制台：后端连接、channel 运行数、active research、recent sessions、runner 选择和 quick chat。

这些能力说明 Hone 已经不只是聊天入口，而是一个主动投研系统。但用户每天主动打开 Hone 时，仍缺少一个聚合页回答：

1. 今天真正影响我的持仓和主线的事项是什么？
2. 哪些 digest 已经发过、哪些因为 channel / quiet hours / cap 没看到？
3. 哪些事项需要我继续问 agent、更新公司画像、建立任务或调整通知偏好？
4. 当前主线是否新鲜，哪些持仓没有画像或蒸馏失败？

目前这些答案散在 chat、portfolio、IM digest、admin notifications、schedule / tasks 页面里。对于 public 用户，`/portfolio` 更像长期上下文页，不像每日复盘页；对于桌面用户，dashboard 更像系统状态页，不像投资行动页。

## 问题或机会

### 问题

1. **Hone 缺少高频回访首页。**  
   README 的产品叙事是专业投资助手和投资纪律守门人，但 public 登录后最直接的入口仍是 chat 与长期画像。用户读完主线后，再次打开的增量不明显。

2. **推送系统缺少用户可回看的主动阅读面。**  
   Unified digest 会发送到 IM channel，并在 delivery log 留下审计；但如果用户错过推送、channel 离线、quiet hours hold、cap 截断或没有绑定 IM，就没有一个面向用户的“今日摘要和缺口”页。

3. **长期主线和当天事件没有产品级连接。**  
   `mainline_by_ticker` 与 digest item 都存在，但 UI 没有把“这条新闻为什么和我的主线相关 / 证伪 / 无关”作为用户每日阅读结构。用户需要自己在 chat 里追问，容易丢失复盘闭环。

4. **管理端和用户端视角错位。**  
   `/notifications` 对 operator 很有用，但普通用户不该看到 delivery-log 细节。反过来，用户需要的是可读、可操作、低噪音的今日 briefing。

### 机会

新增 **Daily Brief Workspace**：一个面向 end-user 和 desktop 本地用户的“今日投资工作台”，消费已有 digest context、delivery log、company profile inventory、portfolio holdings 和 notification prefs，生成一个只读为主、动作明确的每日视图。它不替代 IM 推送，也不替代公司画像；它把散落链路聚合成用户每天愿意打开的产品主页。

这符合 AI agent 产品的一个成熟方向：从“问答窗口”转向“持续工作台”。用户不需要每次都想 prompt，而是从系统整理好的今日上下文、待判断事项和下一步动作开始。

## 方案概述

第一版做一个 **read-mostly daily brief surface**，不引入新的 agent 工作流引擎：

- Public 新增 `/brief`，desktop/admin 可在 `/dashboard` 增加同构的 “Today” 区块或跳转入口。
- 后端新增 actor-scoped `BriefWorkspaceService`，聚合：
  - `/api/public/digest-context` 已有的 mainline / holdings / profile inventory。
  - event-engine delivery log 中最近 24 到 72 小时的 `digest`、`digest_item`、`global_digest_item`。
  - cron execution history 中用户任务的完成 / 失败 / 未发送记录。
  - notification prefs 中的 digest slots、quiet hours、timezone。
- 输出结构化 `DailyBrief`，按 “Needs attention / Delivered / Missed or hidden / Context health / Suggested actions” 分组。
- UI 默认不暴露 operator 术语，只展示：
  - 今日重点 3 到 7 条。
  - 每条为何出现：持仓相关、主线证伪、财报临近、宏观 floor、任务结果、错过推送。
  - 下一步动作：打开 chat 追问、查看公司画像、刷新主线、调整通知时间、创建 evidence review draft。

## 用户体验变化

### 用户端

- 登录后 `/me` 和 `/portfolio` 增加 “今日简报”入口；未来可以把 `/brief` 设为登录后的默认 next step。
- `/brief` 顶部显示一句克制摘要：例如 “今天有 2 条可能影响你的 NVDA / TSM 主线，1 条 digest 已送达，1 条因勿扰时段未推送。”
- 中部以列表展示今日事项，每条有：
  - ticker / event kind / 发生时间 / 来源类型。
  - 与投资主线关系：`aligned`、`counter`、`neutral`、`unknown`。
  - 状态：已推送、未推送、被合并、被数量上限隐藏、任务失败。
  - 操作：继续问 Hone、查看画像、刷新主线、调整通知。
- 空状态不说“暂无内容”，而是给出下一步：添加持仓、建立公司画像、开启 digest slot 或绑定一个渠道。

### 管理端

- `/dashboard` 增加 “Today for selected actor” 或入口卡片，帮助 operator 代用户诊断为什么今日没有价值回访。
- `/notifications` 保持排障表格，不改成用户视图；Daily Brief 可以深链到对应 filtered notification record 供管理员排查。
- Settings 的 invite 用户列表可以显示最近 brief 是否有可读内容，作为 activation evidence。

### 桌面端

- 本地 dashboard / tray 可显示 “今日 3 条待看” 和最近一次 brief 更新时间。
- 桌面 bundled 模式下仍只通过本地 backend 聚合，不新增 sidecar。
- 如果 channel 未配置，brief 仍可工作，成为未绑定 IM 用户的主要主动阅读入口。

### 多渠道

- IM digest 继续负责主动送达；Daily Brief 负责回看、解释和补动作。
- IM 中的 digest 可以追加短 link：`查看今日简报`。第一版 link 只指向 public `/brief` 或 desktop local URL，不在 IM 内承载复杂交互。
- 对 Feishu / Telegram / Discord 的消息格式无新增要求；它们仍消费现有 `DigestPayload`。

## 技术方案

### 1. DailyBrief 数据模型

新增结构化投影，不改变 event-engine 原始表：

```rust
pub struct DailyBrief {
    pub actor: ActorIdentity,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub timezone: String,
    pub generated_at: DateTime<Utc>,
    pub headline: String,
    pub sections: Vec<DailyBriefSection>,
    pub context_health: BriefContextHealth,
}

pub struct DailyBriefItem {
    pub id: String,
    pub source: BriefItemSource,
    pub title: String,
    pub summary: Option<String>,
    pub symbols: Vec<String>,
    pub relation: Option<MainlineRelation>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub delivery_state: BriefDeliveryState,
    pub reason: BriefReason,
    pub actions: Vec<BriefAction>,
}
```

`DailyBrief` 是 derived view。第一版不新建长期真相源，只允许可选缓存最近一次 JSON 到 runtime cache，方便页面快速打开；缓存可随 runtime reset 删除。

### 2. 聚合来源

- `FilePrefsStorage::load(actor)`：读取 timezone、digest slots、mainline、last distilled time、skipped tickers。
- `PortfolioStorage::load(actor)`：读取 holdings，用于判断缺失画像和 symbol coverage。
- actor sandbox `company_profiles/`：沿用 `public_digest.rs` 的 scan profile 逻辑，判断画像存在、大小、ticker。
- `EventStore::list_recent_delivery_logs`：读取 `digest` / `digest_item` / `global_digest_item`，按 actor 和窗口过滤。
- `CronJobStorage::list_recent_executions`：读取任务执行历史和发送状态。
- 可选：从 `DigestPayload` 或 delivery `body_preview` 回填标题；中期应让 delivery log 记录结构化 payload summary，避免只从文本反推。

### 3. API 设计

Public:

- `GET /api/public/brief?window_hours=24`
- actor 由 HttpOnly session 推导，复用 `public_digest.rs::require_public_actor` 模式。
- 返回 `DailyBrief`，不包含 sandbox 绝对路径、raw prompt、raw delivery body。

Admin:

- `GET /api/admin/brief?channel=web&user_id=...&window_hours=24`
- 只给 operator 视角使用，可额外返回 `debug_refs`，例如 delivery log id / cron run id。

Desktop:

- 复用 admin backend API；前端根据 `backend.state.isDesktop` 控制入口。

### 4. 排序和分组规则

第一版不依赖 LLM 重新总结，避免把 daily brief 变成新的高成本路径：

- Priority 1：`MainlineRelation::Counter`、High severity digest fallback、earnings T-3/T-1、SEC filing enrichment。
- Priority 2：用户持仓相关的 delivered digest item 和 failed / missed scheduled task。
- Priority 3：global macro floor 和 neutral global picks。
- Priority 4：context health warnings，例如缺画像、主线超过 N 天未蒸馏、digest slot 关闭、channel 无送达记录。

当需要生成一句 `headline` 时，先使用 deterministic template；后续可接辅助 LLM，但必须落入 LLM audit，并受 usage entitlement / cost policy 管理。

### 5. 前端落点

- `packages/app/src/pages/public-brief.tsx`：public daily brief 页面。
- `packages/app/src/lib/api.ts`：新增 `getPublicDailyBrief`。
- `packages/app/src/app.tsx`：public route 增加 `/brief`。
- `packages/app/src/pages/public-me.tsx`、`public-portfolio.tsx`：增加入口，不改变现有 portfolio context 逻辑。
- `packages/app/src/pages/dashboard.tsx`：desktop/admin 增加 Today block。

UI 采用紧凑投资工作台，不做营销式 hero。列表密度应接近 `/notifications` 的可扫描性，但文案面向用户，不暴露 `delivery_log.status` 这类内部术语。

## 实施步骤

### Phase 1: 只读聚合 API

- 在 `crates/hone-web-api` 增加 `routes/brief.rs` 和纯函数 model。
- 复用 `public_digest.rs` 的 actor 推导与 profile scan。
- 从 prefs、portfolio、EventStore、CronJobStorage 构造 `DailyBrief`。
- 覆盖 public actor、admin actor、空 portfolio、无 event store、digest disabled、channel failed 等场景的单元测试。

### Phase 2: Public `/brief` 页面

- 新增 public route 和 API client。
- 页面先展示 headline、attention list、context health 和 suggested actions。
- 给 `/me`、`/portfolio`、`/chat` 增加一致入口。
- 空状态引导到添加持仓、创建画像、开启 digest 或进入 chat。

### Phase 3: Desktop/Admin Today block

- 在 dashboard 上接入同一个模型，默认展示本地 `ME_SESSION_ID` / selected actor 的 brief。
- 从 brief item 链接到 `/notifications`、`/users/:actor/portfolio`、`/sessions/:actor` 或 `/research`。
- 增加 refresh 和最近更新时间，不默认自动触发 LLM。

### Phase 4: Action handoff

- 将 actions 标准化为 deep link 或 prefilled chat prompt：
  - `ask_hone_about_event`
  - `view_profile`
  - `refresh_mainline`
  - `adjust_notification_prefs`
  - `create_evidence_review_draft`
- 如果 `auto_p1_evidence_review_queue.md` 后续落地，可让 counter-thesis item 直接创建 evidence review draft。

## 验证方式

- 单元测试：
  - `DailyBriefService` 在无 event store / 空 delivery log 时返回健康空状态。
  - 读取 `digest_item`、`global_digest_item`、cron run 后按优先级分组。
  - `MainlineRelation::Counter` item 进入 attention section。
  - 缺公司画像、主线过期、digest slots disabled 能生成 context health warning。
  - public API 不返回 sandbox 绝对路径、raw prompt 或 raw delivery body。
- 前端测试：
  - public `/brief` 能渲染 loading、unauthorized、empty、with items、with health warnings。
  - action URL / prefill 不破坏 public surface routing。
- 手工验收：
  - 构造一个 web actor，建立两只持仓和一个公司画像，跑一次 global digest / event delivery fixture，确认 `/brief` 显示今日事项。
  - 模拟 channel disabled 或 quiet hours，确认 brief 用用户语言解释“未推送但可回看”。
  - 桌面 bundled 模式打开 dashboard，确认 Today block 不依赖外部 IM channel。
- 指标：
  - public login 后访问 `/brief` 的比例。
  - brief item 点击进入 chat / profile / refresh mainline 的比例。
  - IM digest link 回到 `/brief` 的比例。
  - 有 brief 内容用户的 7 日回访率。

## 风险与取舍

- **风险：与 notification / evidence review / portfolio 页面重复。**  
  取舍：Daily Brief 只做每日聚合阅读和动作入口；notification 保持 operator 排障，evidence review 负责把证据变成可处理待办，portfolio 负责长期上下文。

- **风险：delivery log 目前更偏审计，结构化摘要不足。**  
  取舍：第一版从现有 event ids、status、body preview 和 digest payload 投影，必要时只展示保守标题；后续再把结构化 digest payload summary 写入 audit。

- **风险：用户把 brief 当成投资建议。**  
  取舍：页面文案必须延续 `prompt.rs` 的金融约束，强调“影响主线的证据和待核查事项”，不输出买卖建议或自动交易动作。

- **风险：每日页面引入额外 LLM 成本。**  
  取舍：第一版完全 deterministic；只有未来 headline polish 才可选用辅助 LLM，并必须受 audit / entitlement 控制。

- **不做边界：**
  - 不新增通用任务队列。
  - 不改变 digest delivery policy。
  - 不把 `/notifications` 暴露给 public 用户。
  - 不直接修改公司画像，所有画像更新仍通过 agent 文件操作。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点检索了 `brief`、`digest`、`today`、`dashboard`、`portfolio`、`notification`、`evidence`、`timeline`、`daily` 等关键词。

- 不重复 `auto_p1_delivery_decision_loop.md`：该提案解释一次通知为什么发或不发，并用反馈调整 delivery policy；本提案把已发生和未送达的事项组织成用户每日阅读页，不改变推送决策。
- 不重复 `auto_p1_evidence_review_queue.md`：该提案把可能改变 thesis 的事件变成可处理的 review 待办；本提案包含所有今日可读事项，只有部分 item 可后续升级成 evidence review。
- 不重复 `auto_p1_temporal-operations-calendar.md`：该提案展开未来任务、digest slot、quiet hours 和冲突风险；本提案看过去 24 到 72 小时实际发生了什么，以及用户现在该看什么。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：该提案生成组合暴露和情景风险视图；本提案以 digest / delivery / profile health 为每日入口，可以链接到 exposure radar 但不计算风险暴露。
- 不重复 `auto_p1_context-return-links.md`：该提案定义从通知或回答回到上下文的 deep link；本提案定义 deep link 落到哪里以及页面上如何聚合今日事项。
- 不重复 `auto_p2_email-report-bridge.md`：该提案新增邮件作为低打扰 outbound 报告通道；本提案新增 Web/Desktop inbound 阅读工作台，不新增 email sink。
- 不重复 `auto_p1_privacy-preserving-product-events.md`：该提案记录 feature adoption 事件；本提案是用户可见的产品页面，可作为未来 product event 的消费对象。

## 文档同步说明

本轮只新增 proposal，不实际执行该架构改造，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若未来开始实现 Daily Brief Workspace，应创建或复用 `docs/current-plans/daily-brief-workspace.md`，并在影响 public/admin/desktop 路由和 event delivery projection 后同步更新 repo map。
