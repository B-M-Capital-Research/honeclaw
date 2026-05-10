# Proposal: Portfolio Exposure Radar and Scenario Guardrails

status: proposed
priority: P1
created_at: 2026-05-10 11:02:43 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/current-plan.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/context/portfolio.tsx`

## 背景与现状

Hone 的公开定位是投资研究助理，而不是交易执行工具。当前仓库已经具备多块关键基础：

- `memory/src/portfolio.rs` 以 actor 为边界保存 portfolio JSON，支持真实持仓、watchlist、股票、期权、`shares`、`avg_cost`、`holding_horizon`、`strategy_notes` 和 `tracking_only`。
- `crates/hone-tools/src/portfolio_tool.rs` 允许 agent 通过工具查看、添加、更新、删除持仓和关注列表；Admin UI 也能在 `/users/:actor/portfolio` 手工维护这些数据。
- `crates/hone-event-engine/src/subscription.rs` 会从 portfolio 构建订阅和 price watch pool，但当前订阅语义主要是“这个事件是否命中某个 symbol”，不是“这个事件对组合暴露有多重要”。
- `crates/hone-event-engine/src/prefs.rs` 已有 `large_position_weight_pct`、价格阈值、quiet mode、digest slots、per-ticker mainline 和 global style 等偏好字段，说明系统已经开始考虑“不同持仓权重应触发不同推送策略”，但缺少可信的组合级暴露输入。
- Public `/portfolio` 和 Admin `UserMainlineView` 能展示持仓 ticker、系统蒸馏投资主线、公司画像覆盖和刷新状态。它们回答了“我关注哪些公司、每家公司主线是什么”，但还不能回答“我的组合是否过度集中在同一因子、同一宏观假设或即将到期的期权事件上”。
- Admin `PortfolioDetail` 的 summary 目前主要是持仓数、关注数、总股数和更新时间；没有 concentration、成本基准、期权到期、策略标签、长期/短期混合、缺失价格等质量信号。

这说明 Hone 已有数据入口和展示入口，但 portfolio 还停留在“标的清单”层。对于一个强调投资纪律的 AI agent，下一层应是组合暴露和场景 guardrail：在不提供买卖指令的前提下，帮助用户看清风险集中在哪里、哪些前置数据不可靠、哪些自动化和通知应该更敏感。

## 问题或机会

这是 P1 级机会。它不会像输出安全门那样直接阻止错误送达，但会显著提升核心体验、通知相关性、留存和付费感知。

当前缺口集中在五点：

1. **组合风险没有一等视图。**
   用户能看到每个 ticker 的投资主线，但看不到组合层面的集中度、长期/短期冲突、期权到期窗口、watch-only 与真实持仓的区别、成本基准缺失和数据质量问题。

2. **事件重要性仍过度依赖 symbol 命中。**
   Event engine 的 `PortfolioSubscription` 只知道某事件命中了哪些 symbol。即使 notification prefs 有 `large_position_weight_pct` 字段，系统也没有稳定的 exposure snapshot 来告诉 router 某个 symbol 对该 actor 是大仓位、观察标的、短期事件仓、期权杠杆仓，还是成本未知的低质量记录。

3. **主线和持仓规模没有闭环。**
   Mainline distill 能生成 per-ticker thesis，但没有指出“用户组合的主要假设其实集中在 AI capex、利率久期、美元流动性、半导体周期或单一监管风险”。这会让 digest personalization 看起来聪明，但组合纪律感不够强。

4. **Public 用户缺少能每日回来的理由。**
   `/portfolio` 展示长期记忆很有价值，但如果用户已经读过主线，下一次访问的增量不明显。组合暴露雷达可以成为每日检查入口：今天哪些暴露变化、哪些风险进入观察窗口、哪些信息缺口影响判断。

5. **Admin 难以快速判断某个 actor 的投资上下文质量。**
   运维者现在需要分别看 portfolio、profiles、mainline、notifications、sessions。若某个用户抱怨推送不准，admin 缺少一个组合级摘要判断问题来自数据缺口、集中度过高、watchlist 被误当持仓、期权元数据不完整，还是通知偏好过松。

机会是新增一个 **Portfolio Exposure Radar**：基于现有 portfolio、company portraits、mainline prefs、notification prefs 和可选行情数据生成 actor-scoped、可解释、只读的组合暴露快照。第一版不接券商、不做交易建议、不改 portfolio 真相源，只生成派生风险面和 guardrail hints。

## 方案概述

新增一层 actor-scoped 派生模型：`PortfolioExposureSnapshot`。

它回答四个问题：

1. **我真正暴露在哪里？**
   按真实持仓和 watchlist 分开，基于 `shares * avg_cost` 先计算成本口径的初始权重；后续可接入行情后生成 market value 权重。期权用 `contract_multiplier`、underlying、strike、expiration 标记为衍生品暴露，不在 v1 里伪装成精确 delta。

2. **哪些数据让判断不可靠？**
   标记 `missing_cost_basis`、`zero_shares_on_holding`、`option_missing_expiration`、`unknown_horizon`、`watchlist_without_notes`、`profile_missing`、`mainline_stale`、`distill_skipped`、`price_unavailable` 等质量项。

3. **组合有哪些集中风险和场景 guardrails？**
   v1 先做确定性规则：单一 ticker 成本权重过高、同一 strategy_notes/tag 重复过高、短期仓位与长期画像混杂、期权 30/14/7 天到期、多个持仓共享相同主线关键词但缺反证条件。v2 再由 agent 从 company portraits 中抽取行业、宏观因子和 thesis tags。

4. **这些暴露如何影响通知和 agent 回答？**
   Exposure snapshot 可为 event-engine router、global digest personalize 和 chat prompt 提供低成本上下文：某事件命中大仓位时解释优先级更高；仅 watchlist 命中时默认放入 digest；期权到期临近时提醒用户复核假设，但不建议具体买卖。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“组合暴露雷达”区块，显示：
  - Top concentration：前 3 个真实持仓成本权重和是否超过阈值。
  - Data quality：影响判断的缺口数量和最重要 3 项。
  - Scenario guardrails：例如“半导体周期相关假设集中”“2 个期权持仓 14 天内到期”“3 个持仓没有长期/短期标记”。
  - Suggested next review：引导用户去 chat 里补充成本、horizon、策略注释或公司画像，不直接给交易动作。
- 每个 ticker 卡片旁显示轻量 badge：`large_position`、`watch_only`、`needs_profile`、`option_expiring`、`thesis_stale`。
- 用户询问“今天我应该重点看什么”时，agent 可以先引用 exposure radar，再解释为什么某些事件更值得看。

### 管理端

- `/users/:actor/portfolio` 在 summary 下增加 exposure panel：
  - 成本口径总额、可计算权重覆盖率、无法计算权重的记录数。
  - 大仓位、期权到期、缺画像、蒸馏失败、watchlist 误用等 risk cards。
  - Deep link 到对应 portfolio 行、company profile、mainline refresh 和 notification prefs。
- `/users/:actor/mainline` 可显示“组合级 thesis tags”与 profile 覆盖情况，帮助 admin 判断主线蒸馏是否缺上下文。
- 后续可在 `/notifications` 的 delivery detail 中展示“本次事件为什么对该 actor 是 high/medium”：来自 severity、prefs、large position 或期权窗口。

### 桌面端

- 本地 desktop dashboard 可显示一个紧凑状态：“组合暴露 3 项需复核 / 1 个到期窗口 / 2 个画像缺失”。
- bundled 模式完全使用本地 portfolio JSON、prefs 和 actor sandbox，不依赖云端。
- remote backend 模式显示 exposure snapshot 的生成时间和后端来源，避免用户误以为本地数据已同步。

### 多渠道

- 在 Feishu / Telegram / Discord 私聊中支持低风险问法：
  - “我的组合暴露有哪些缺口？”
  - “哪些持仓需要补画像？”
  - “本周有哪些到期或复核点？”
- 多渠道回复只发送 summary 和最多 3 个行动项；完整列表引导回 Web/desktop。
- 群聊不主动暴露个人组合数据，延续 `ActorIdentity` 与 `SessionIdentity` 隔离约束。

## 技术方案

### 1. 新增派生 exposure 模块

建议新模块放在 `memory` 或 `hone-event-engine` 之间需要谨慎。第一版更适合放在 `crates/hone-web-api` 或新建 `crates/hone-core` 纯类型 + `memory` 派生读取层：

```rust
pub struct PortfolioExposureSnapshot {
    pub actor: ActorIdentity,
    pub generated_at: DateTime<Utc>,
    pub valuation_basis: ExposureValuationBasis,
    pub totals: ExposureTotals,
    pub positions: Vec<PositionExposure>,
    pub concentration_flags: Vec<ExposureFlag>,
    pub quality_flags: Vec<ExposureFlag>,
    pub scenario_tags: Vec<ScenarioExposure>,
    pub notification_hints: Vec<NotificationHint>,
}
```

v1 输入：

- `PortfolioStorage::load(actor)`：真实持仓、watchlist、成本、股数、horizon、期权字段、notes。
- `NotificationPrefs`：mainline、distill metadata、quiet/digest/large position config。
- `company_profiles` scan：只读 profile 列表、ticker 覆盖、更新时间。

v1 不要求实时行情。权重使用成本口径：

- 股票：`shares * avg_cost`
- 期权：`quantity * avg_cost * contract_multiplier.unwrap_or(100.0)`，并标记 `valuation_basis=cost_basis_only`。
- 缺成本或数量时不参与分母，产生 quality flag。

v2 可接行情：

- 复用已有 FMP / data fetch 能力获得最新价，生成 `market_value_estimate`。
- 若行情失败，保留成本口径，不阻塞页面。
- 不用行情结果生成买卖建议，只用于风险权重和通知敏感度。

### 2. Exposure flag 规则

先做确定性规则，避免把组合风险判断全部交给 LLM：

- `single_position_concentration`：单一真实持仓成本权重超过 25% / 40% 两档。
- `uncalculable_weight`：真实持仓缺少 shares 或 avg_cost，导致权重不可计算。
- `watchlist_only`：关注标的不计入真实仓位，但可进入 research/digest。
- `option_expiry_window`：期权 30/14/7 天内到期；缺 expiration 也标记。
- `horizon_mismatch`：短期仓位却只有长期画像，或长期仓位缺 company profile。
- `mainline_missing_or_stale`：持仓没有 per-ticker mainline、蒸馏跳过或超过阈值未刷新。
- `strategy_cluster`：多个持仓 `strategy_notes` 中出现相同关键词，如 AI、semiconductor、rates、oil、earnings。

这些 flag 不改变 portfolio 原数据，只作为派生视图和 prompt context。

### 3. API 与缓存

新增只读 API：

- `GET /api/portfolio/exposure?channel=&user_id=&channel_scope=`
- `GET /api/public/portfolio/exposure`

响应中包含：

- `snapshot`
- `generated_at`
- `source_versions`：portfolio updated_at、prefs updated_at 或 mtime、profile mtimes、行情时间戳。
- `limitations`：例如 `cost_basis_only`、`missing_prices`、`options_not_delta_adjusted`。

缓存策略：

- 第一版按请求即时计算即可，数据量小。
- 若后续接行情，可使用短 TTL 缓存，key 为 actor storage key + portfolio updated_at + prefs mtime + profile max mtime。
- 不写入 portfolio JSON，避免派生结果污染真相源。

### 4. Agent 与 event-engine 集成

第一阶段只在 Web API 和 UI 展示。

第二阶段把 snapshot 用作两类上下文：

- Chat prompt：当用户问组合风险、今日重点、某事件是否重要时，turn builder 或 tool 层可提供 compact exposure summary。
- Event engine：router 处理 actor-specific event 时读取 lightweight exposure hints：
  - `large_position` 可配合 `large_position_weight_pct` 降低重要事件进入 digest 的概率。
  - `watch_only` 默认不升 high，除非事件 severity 本身 high。
  - `option_expiry_window` 对 earnings / price alert / SEC filing 增加解释性 reason，但仍经过 safety gate。

这不是新推荐引擎，只是让已有通知和回答更懂用户组合结构。

### 5. UI 落点

- `packages/app/src/components/portfolio-detail.tsx`：
  - summary 下增加 exposure panel。
  - 每行持仓显示 exposure badges。
- `packages/app/src/pages/public-portfolio.tsx`：
  - 在整体投资风格和 per-ticker mainline 之间展示 exposure radar。
  - 移动端只保留 3 个最重要 flag。
- `packages/app/src/components/user-mainline-view.tsx`：
  - mainline tab 可显示 scenario tags 和 profile coverage。
- `packages/app/src/lib/types.ts` 与 `packages/app/src/lib/api.ts`：
  - 增加 snapshot 类型和 API client。

## 实施步骤

### Phase 1: Cost-basis exposure snapshot

- 定义 `PortfolioExposureSnapshot`、`PositionExposure`、`ExposureFlag` 类型。
- 从 `PortfolioStorage`、`NotificationPrefs`、profile scan 生成只读 snapshot。
- 新增 admin/public exposure API。
- 添加 Rust 单元测试覆盖股票、watchlist、期权、缺成本、缺画像、mainline skipped。

### Phase 2: Admin 与 Public UI

- Admin portfolio summary 增加 exposure panel 和 row badges。
- Public `/portfolio` 增加轻量 exposure radar。
- 所有 flag 文案强调“复核/补充上下文”，不使用“买/卖/加仓/减仓”命令式措辞。
- 添加前端数据转换测试和基本响应式手工验收。

### Phase 3: Agent-readable compact context

- 增加只读 tool 或 execution context helper，让 agent 在用户明确问组合风险时读取 compact exposure summary。
- 限制输出边界：只能解释暴露、缺口、复核优先级，不给交易指令。
- 与 `Investment Output Safety Gate` 提案兼容：若回答涉及仓位动作，仍走 safety gate。

### Phase 4: Notification hints and optional market value

- Event engine router 读取 lightweight hints，增强 delivery reason 和 digest priority。
- 可选接入行情价格，生成 market-value estimate；行情不可用时保留 cost-basis-only。
- 在 notification detail 中展示 exposure reason，帮助用户和 admin 理解为什么这条事件被提升或降级。

## 验证方式

- Rust unit tests：
  - 两个股票持仓分别为 70% / 30%，应产生 `single_position_concentration`。
  - watchlist 记录不进入真实仓位分母，但生成 `watch_only` position exposure。
  - 期权缺 `expiration_date` 生成 quality flag；14 天内到期生成 expiry flag。
  - `avg_cost=0` 或 `shares=0` 的真实持仓不参与权重并生成 quality flag。
  - 有持仓但缺 company profile 或 mainline skipped 时生成 profile/mainline flag。
- API tests：
  - admin endpoint 只能按显式 actor 查询；public endpoint 只能返回当前 web session actor。
  - response 包含 `generated_at`、`valuation_basis`、`limitations` 和 source metadata。
- Frontend tests：
  - exposure flags 排序稳定，移动端 summary 不丢失最高严重度 flag。
  - watchlist 与真实持仓 badge 显示不同。
- 手工验收：
  - Admin `/users/:actor/portfolio` 能从 exposure card deep link 到对应持仓行和 mainline/profile 页面。
  - Public `/portfolio` 在空 portfolio、仅 watchlist、完整 portfolio、缺成本、期权到期场景下都有清晰空态或 warning。
  - 多渠道询问“我的组合暴露有哪些缺口”时，回答只总结风险和待补信息，不给买卖建议。
- 指标：
  - Public `/portfolio` 回访率。
  - 用户补全 avg_cost / horizon / profile 的转化率。
  - 被 exposure hints 提升/降级的通知比例。
  - 用户对“为什么收到这条推送”的反馈下降。

## 风险与取舍

- **风险：用户把 exposure flag 当成交易建议。**
  取舍：文案只使用“复核、补充、关注、解释优先级”，禁止输出“应买/应卖/应加仓/应减仓”。
- **风险：成本口径权重不等于真实市值权重。**
  取舍：v1 明确标记 `cost_basis_only`；行情接入前不声称是实时组合净值。
- **风险：期权风险被过度简化。**
  取舍：v1 只标记成本、名义字段、到期窗口和元数据缺口，不计算 delta/gamma，不伪装成专业风控系统。
- **风险：与 context intake 或 thesis map 重叠。**
  取舍：本提案不解决初始化流程、不生成跨公司主线地图，只把已存在的 portfolio 和 mainline 转成组合级暴露视图。
- **风险：LLM 抽取 scenario tags 可能不稳定。**
  取舍：v1 使用确定性 rules 和现有字段；LLM tagging 只作为 v2 派生辅助，结果必须保留 source refs。
- **风险：event-engine 读取 exposure 增加路由复杂度。**
  取舍：先只做 UI/API；router 集成放到 Phase 4，并且只消费 compact hints，不直接依赖完整 snapshot。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案。结论：本提案不重复，重点差异如下：

- 不重复 `auto_p1_investment_context_intake.md`：intake 解决新用户或缺口用户如何建立 portfolio/profile/prefs/task；本提案假设 portfolio 已存在，解决组合级暴露、集中度和复核 guardrails。
- 不重复 `auto_p1_cross-company-thesis-map.md`：thesis map 处理多个公司画像之间的共享主线和差异变量；本提案处理真实持仓规模、watchlist、期权、成本口径和通知敏感度。
- 不重复 `auto_p1_trade_discipline_journal.md`：journal 处理具体交易意图的记录和复盘；本提案不记录交易动作，只展示当前组合暴露和数据缺口。
- 不重复 `auto_p1_evidence_review_queue.md`：evidence queue 处理市场事件是否应该进入画像复盘；本提案处理 portfolio 当前结构如何影响事件优先级和用户理解。
- 不重复 `auto_p1_runtime_readiness_matrix.md` / `auto_p1_temporal-operations-calendar.md`：它们分别关注运行能力可用性和未来自动化时间线；本提案关注投资组合本身的风险面。
- 不重复 `auto_p1_delivery_decision_loop.md`：delivery loop 解释通知送达决策；本提案为通知决策提供组合暴露输入，但不是通知审计面本身。
- 不重复 `auto_p0_investment_output_safety_gate.md`：safety gate 拦截危险输出；本提案在回答和推送前提供更好的组合上下文，不能替代安全门。

差异结论：现有提案已经覆盖安全、证据、自动化、运行、权益、数据权利、研究资产和跨公司主线，但还没有把 portfolio 从“ticker 清单”提升为“组合暴露雷达”。这个主题直接服务 Hone 的核心承诺：帮助用户保持投资纪律，先看清自己暴露在哪里，再讨论市场事件是否重要。

## 文档同步说明

本轮只新增 proposal，不开始实现，不改变模块边界、入口、长期规则或运行流程，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续开始实施本提案，应按动态计划准入标准新增或复用 `docs/current-plans/portfolio-exposure-radar.md`，并在实现涉及 API / event-engine / frontend 数据流时同步更新 `docs/repo-map.md`。
