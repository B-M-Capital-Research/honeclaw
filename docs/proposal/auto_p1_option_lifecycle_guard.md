# Proposal: Option Lifecycle Guard for Expiration and Assignment Discipline

status: proposed
priority: P1
created_at: 2026-05-25 20:06:13 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_corporate-action-reconciliation.md`
- `docs/proposal/auto_p1_portfolio-transaction-ledger.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `memory/src/portfolio.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-event-engine/src/pollers/price.rs`
- `crates/hone-event-engine/src/pollers/earnings.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `packages/app/src/lib/types.ts`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Hone 的 portfolio 已经不只是股票清单：

- `memory/src/portfolio.rs` 的 `Holding` 已有 `asset_type`、`underlying`、`option_type`、`strike_price`、`expiration_date`、`contract_multiplier`、`holding_horizon`、`strategy_notes` 和 `tracking_only` 字段，并有单元测试覆盖 option holding 与负成本 option 场景。
- `crates/hone-web-api/src/routes/portfolio.rs` 接受 `asset_type=option`，会标准化 `option_type` 的 `c/p -> call/put`，并能在 update 路径保留 `strike_price`、`expiration_date`、`contract_multiplier` 等字段。
- `packages/app/src/lib/types.ts` 已暴露 option 字段，但 `packages/app/src/context/portfolio.tsx` 和 `packages/app/src/components/portfolio-detail.tsx` 当前主要按股票/关注列表体验组织表单与摘要；编辑现有记录时只回填 symbol、shares、avg_cost、horizon、strategy_notes、notes、tracking_only。
- Event engine 已有 price、earnings、SEC / corporate action 等事实事件和 subscription/watch pool，但没有把 option 的到期日、价内/价外状态、财报窗口、指派风险或 roll-review 变成一等派生状态。
- 现有 proposal 已覆盖组合暴露、公司行动调账、交易流水、投资纪律日志，但它们都没有专门处理“期权仓位从建仓到到期前复核，再到到期/行权/指派后的状态闭环”。

这说明仓库已经为 option position 留了数据模型入口，但产品层还没有把期权作为高风险生命周期对象处理。对强调投资纪律的助手来说，期权不是普通 ticker：同一标的价格不变时，时间流逝本身就会改变风险；财报、除权、波动、到期日、行权价和合约乘数都可能让“看起来只是一个持仓”变成必须复核的事项。

## 问题或机会

这是 P1：它直接影响核心投资纪律、用户信任和多渠道主动提醒质量，但可以先以只读派生状态、复核任务和确认式处理落地，不需要接券商 API，也不提供交易指令。

主要缺口：

1. **option 元数据存在，但没有生命周期状态。**  
   系统知道 expiration、strike、underlying，却没有 `days_to_expiry`、`moneyness_band`、`earnings_before_expiry`、`assignment_risk`、`review_due`、`expired_unresolved` 等可执行状态。

2. **近到期期权对用户是高优先级复核点。**  
   普通价格波动可以进入 digest，但 30/14/7/1 天到期窗口更像“必须确认原计划是否仍成立”。如果 Hone 不主动提示，用户可能把短期期权当成普通长期持仓遗忘。

3. **事件重要性缺少 option 语义。**  
   Price alert、earnings upcoming、SEC filing 命中 underlying 时，持有正股和持有近到期期权的紧急程度不同。当前 event engine 主要基于 symbol 命中和 severity，不知道这个 actor 的风险来自 option timebox。

4. **到期后状态容易污染 portfolio truth。**  
   到期、行权、被指派、平仓、roll 到新合约都是不同结果。直接自动改 portfolio 风险很高，但完全不建 pending 状态会让过期期权长期留在 portfolio，影响 exposure、通知和 agent 回答。

5. **商业化与留存价值明确。**  
   对期权用户来说，“到期前纪律复核”和“到期后记录清理”是高频刚需，比泛泛聊天更能体现 Hone 是投资工作台而非普通聊天机器人。

## 方案概述

新增 **Option Lifecycle Guard**：围绕 actor-scoped portfolio 中的 option holding 生成只读 lifecycle snapshot、复核提醒、到期后确认项和多渠道摘要。

核心原则：

- 不自动给买卖/roll 建议，只提醒复核原 thesis、风险边界和需要确认的状态。
- 不自动改 portfolio；到期后生成 pending resolution，由用户确认 `expired worthless`、`exercised/assigned`、`closed`、`rolled` 或 `still open / data wrong`。
- 不把 option 估值伪装成精确风控；v1 只用 deterministic 字段和已有 price/earnings 事件做分层。
- 群聊不主动暴露个人期权仓位；私聊、public portfolio、desktop/admin 才展示 actor 私有状态。

核心对象：

- `OptionPositionKey`：稳定标识合约，优先由 `underlying + option_type + strike_price + expiration_date` 组成；保留 `symbol` 兼容当前 portfolio。
- `OptionLifecycleSnapshot`：每个 actor 当前 option 状态集合，包含窗口、数据质量、事件关联和建议复核动作。
- `OptionLifecycleFlag`：`missing_underlying`、`missing_expiration`、`missing_strike`、`expiry_30d/14d/7d/1d`、`expired_unresolved`、`earnings_before_expiry`、`near_money`、`deep_itm`、`deep_otm` 等。
- `OptionReviewItem`：需要用户确认的复核项，例如“7 天内到期，且财报在到期前”。
- `OptionResolutionDraft`：到期后待确认结果，确认后才更新 portfolio 或写入交易/纪律记录。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加 `Options` 区块：
  - 最近 30 天到期合约数量。
  - 缺关键字段的合约数量。
  - 本周必须复核的 3 个事项。
  - 已过期但未确认状态的合约。
- 每个 option row 显示简洁 badge：`7d expiry`、`earnings before expiry`、`missing strike`、`expired unresolved`。
- 用户点击某个 item 后进入 chat 或详情页，Hone 只帮助复核：
  - 原始 thesis 是什么。
  - 当前触发了哪些事实事件。
  - 哪些字段缺失影响判断。
  - 用户需要自己确认的结果是什么。
- 到期后，用户可以选择状态：已归零、已行权/指派、已平仓、已 roll、录入错误。系统根据选择进入受控后续流程。

### 管理端

- `/users/:actor/portfolio` 增加 option lifecycle panel：
  - 合约字段完整性。
  - 到期窗口分布。
  - 和 price/earnings 事件关联的 review items。
  - 到期后未处理队列。
- Admin 能快速判断“推送不准”是否来自 option 元数据缺失、portfolio 未清理或用户误把 option 当 stock 录入。
- 对高风险状态变更只提供确认入口，不让 operator 在没有用户授权时替用户决定交易结果。

### 桌面端

- Desktop dashboard 显示轻量状态：`2 options need review this week`、`1 expired position unresolved`。
- Bundled 模式完全基于本地 portfolio JSON 和 event store 派生，适合个人工作台。
- Remote 模式展示后端 snapshot 的生成时间和数据来源，避免用户误判本地与云端状态已经同步。

### 多渠道

- 私聊可以主动推送低泄露摘要：`本周有 2 个期权持仓需要复核，1 个缺到期日。`
- Feishu / Telegram / Discord 回复中只列最多 3 条，完整处理引导到 Web/desktop。
- 群聊只在用户明确私有触发并通过权限检查后提示去私聊处理，不暴露合约明细。

## 技术方案

### 1. 派生 lifecycle snapshot

第一版不修改 `PortfolioStorage` 的真相源结构，在 `memory` 或 `crates/hone-web-api` 增加纯派生模块：

```rust
pub struct OptionLifecycleSnapshot {
    pub actor: ActorIdentity,
    pub generated_at: DateTime<Utc>,
    pub positions: Vec<OptionLifecyclePosition>,
    pub review_items: Vec<OptionReviewItem>,
    pub unresolved_resolutions: Vec<OptionResolutionDraft>,
    pub quality_flags: Vec<OptionLifecycleFlag>,
}
```

输入：

- `PortfolioStorage::load(actor)`：读取 option holding。
- price quote：可复用已有 `PricePoller` 的 quote fetch 或事件库最近 price event；不可用时降级。
- earnings teaser：读取 event store 中 upcoming earnings 或复用 digest scheduler 现算窗口。
- notification prefs：用于决定 direct vs digest，只影响提醒方式，不影响 flag 生成。

v1 规则：

- `asset_type != option` 的 holding 不进入 snapshot。
- `expiration_date` 可解析时计算 `days_to_expiry`；不可解析则生成 quality flag。
- `strike_price` 与 underlying latest price 都可用时计算粗粒度 moneyness：
  - call: price / strike - 1
  - put: strike / price - 1
  - 只输出 `near_money`、`itm`、`otm` 等 band，不输出交易建议。
- expiration 在 30/14/7/1 天窗口内生成 review item。
- 已过期且仍在 portfolio 中存在时生成 `expired_unresolved` resolution draft。
- 若 upcoming earnings date <= expiration_date，生成 `earnings_before_expiry` review item。

### 2. 到期后 resolution queue

新增轻量 SQLite 存储，或作为未来 transaction ledger 的前置表：

```text
option_resolution_drafts (
  draft_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  option_key TEXT NOT NULL,
  symbol TEXT NOT NULL,
  underlying TEXT,
  expiration_date TEXT,
  state TEXT NOT NULL,
  reason TEXT,
  before_holding_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  resolved_at TEXT,
  resolution_kind TEXT
)
```

状态：

- `pending_review`
- `snoozed`
- `resolved_expired_worthless`
- `resolved_exercised_or_assigned`
- `resolved_closed`
- `resolved_rolled`
- `dismissed_bad_data`

确认行为：

- `expired_worthless`：可建议删除旧 option holding，并写入 ledger。
- `exercised_or_assigned`：不直接猜正股 shares/cost，进入确认式 portfolio adjustment draft。
- `closed`：可删除或标记 archived，后续由 transaction ledger 记录成交。
- `rolled`：要求用户录入新合约字段，旧合约 resolution 与新 holding 关联。
- `bad_data`：引导补字段，不改仓位。

### 3. API 与权限

Admin API：

- `GET /api/portfolio/options/lifecycle?channel=&user_id=&channel_scope=`
- `POST /api/portfolio/options/resolutions/:draft_id/resolve`
- `POST /api/portfolio/options/resolutions/:draft_id/snooze`

Public API：

- `GET /api/public/portfolio/options/lifecycle`
- `POST /api/public/portfolio/options/resolutions/:draft_id/resolve`

public route 只能使用当前 cookie session 推导 actor，不能接受任意 actor query。所有响应脱敏，不返回 actor sandbox path、内部 event store path 或完整 admin metadata。

### 4. Event-engine 集成

第一阶段只做 Web/API snapshot。

第二阶段让 event engine 在 actor-specific delivery 前读取 compact option hints：

- price alert 命中 underlying 且存在 7 天内到期 option：允许提升到 direct review reminder，但仍走 notification prefs 和 quiet mode。
- earnings upcoming 命中 underlying 且 earnings 在 expiration 前：生成 review reason。
- SEC/corp action 命中 underlying：只加 explain reason，不自动进入 option resolution。
- 已过期 unresolved：定时 digest 中提示一次，避免每天重复轰炸。

这条集成不改变 `MarketEvent` 的事实属性，只在 actor delivery 层增加 option-aware personalization。

### 5. UI 落点

- `packages/app/src/components/portfolio-detail.tsx`：
  - form 增加 option 字段编辑入口。
  - summary 下展示 lifecycle panel。
  - 持仓行展示 option badge。
- `packages/app/src/pages/public-portfolio.tsx`：
  - 展示面向用户的 option review list。
  - 移动端压缩为 3 条最高优先级 item。
- `packages/app/src/lib/types.ts` / `packages/app/src/lib/api.ts`：
  - 增加 lifecycle snapshot、review item、resolution draft 类型和 client。
- 多渠道回复复用一个 shared renderer，避免 Feishu/Telegram/Discord 文案各自发散。

## 实施步骤

### Phase 1: Snapshot and UI proof

- 定义 option lifecycle 类型和 deterministic flag 规则。
- 从 portfolio JSON 生成 snapshot，不接实时行情时也能输出到期与字段完整性。
- Admin `/users/:actor/portfolio` 展示 option panel。
- Public `/portfolio` 展示 compact option review list。
- 增加单元测试覆盖 expiration parsing、missing metadata、expired unresolved、moneyness 降级。

### Phase 2: Resolution drafts

- 增加 option resolution draft 存储和 API。
- 到期后生成 `expired_unresolved` draft。
- 支持用户确认 expired/closed/rolled/bad_data。
- 确认操作写入受控 ledger；需要 portfolio 变更时走现有 `PortfolioStorage` API。

### Phase 3: Event-aware reminders

- 关联 upcoming earnings 与 price alert。
- 在 notification delivery explain reason 中加入 option lifecycle hints。
- 私聊支持“本周有哪些期权要复核”。
- quiet mode、digest slots、portfolio_only 偏好必须继续生效。

### Phase 4: Discipline loop

- 和 `trade_discipline_journal` 或 future transaction ledger 打通，把用户确认结果沉淀为复盘素材。
- 对 repeated snooze、过期未处理、缺字段长期不补等行为生成低频提醒。
- 后续若接券商导入，只把 broker status 当作确认来源，不让它绕过用户确认。

## 验证方式

- Rust unit tests：
  - option key 生成稳定。
  - expiration_date 解析与 30/14/7/1 天窗口正确。
  - 缺 underlying/strike/expiration 时输出 quality flags。
  - 过期 option 生成唯一 unresolved draft，不重复创建。
  - moneyness 在缺 price 时明确降级，不 panic。
- API tests：
  - admin query 可按 actor 查看 snapshot。
  - public API 只返回当前 session actor 的 option lifecycle。
  - resolution 操作不可跨 actor。
- Frontend tests：
  - option row badges、compact list、missing metadata 文案。
  - form 能保存并回填 option 字段。
- Manual regression：
  - 构造一个含股票、watchlist、近到期期权、已过期期权的 portfolio JSON。
  - 验证 admin/public/desktop surfaces 展示一致。
  - 验证 Feishu/Telegram/Discord 私聊摘要不泄露群聊或其它 actor 数据。
- Product metrics：
  - 近到期期权 review item 打开率。
  - expired unresolved 平均关闭时间。
  - option 字段完整率。
  - 因 option 元数据缺失造成的通知误报/漏报投诉下降。

## 风险与取舍

- 风险：用户把 review reminder 当成交易建议。  
  取舍：所有文案只说复核、确认状态、补数据和核对券商，不说买/卖/roll。
- 风险：moneyness 粗略计算误导。  
  取舍：v1 只输出 band 和 limitation，不输出 Greeks、盈亏预测或胜率。
- 风险：自动删除过期期权会破坏记录。  
  取舍：永远先生成 resolution draft，用户确认后才改 portfolio。
- 风险：期权数据格式不标准。  
  取舍：第一版以现有字段为准，不强制支持 OCC symbol parser；后续可增加 parser 作为 import helper。
- 风险：通知过多。  
  取舍：30/14 天默认 digest，7/1 天可 direct；已过期 unresolved 做低频提醒并支持 snooze。
- 不做边界：不接券商交易、不给 roll 建议、不计算 Greeks、不自动行权/指派、不把 option lifecycle 作为买卖信号。

## 与已有提案的差异

- 与 `auto_p1_portfolio-exposure-radar.md` 不重复：exposure radar 关注组合层面的集中度、质量 flags 和场景 guardrails；本提案关注单个 option 合约从近到期、事件复核到到期后确认的生命周期闭环。
- 与 `auto_p1_corporate-action-reconciliation.md` 不重复：corporate action 处理 split/dividend 等公司行动如何影响正股 portfolio truth；本提案处理 option 自身的 expiration、exercise/assignment、roll/close resolution。
- 与 `auto_p1_portfolio-transaction-ledger.md` 不重复：transaction ledger 记录交易流水和导入预览；本提案可以把 resolution 结果交给 ledger，但核心是到期前提醒与到期后状态确认。
- 与 `auto_p1_trade_discipline_journal.md` 不重复：discipline journal 记录用户决策复盘；本提案生成期权特有的复核触发器和确认队列。
- 与 `auto_p1_notification-policy-backtest.md` / `auto_p1_end-user-notification-control.md` 不重复：它们管理通知策略和用户偏好；本提案提供新的 option-aware reason 和 review item，仍服从既有通知偏好。

查重结论：`docs/proposal/` 与 `docs/proposals/` 中已有 proposal 覆盖了 portfolio exposure、corporate actions、transaction ledger、trade discipline、notification policy、event delivery 和 user journey，但没有一篇把 option holding 的到期窗口、事件关联、到期后 resolution 和多渠道私有提醒作为独立产品/架构链路。该主题建立在当前已存在但未充分产品化的 option 字段上，范围清晰，可分阶段落地。

## 文档同步

本轮只新增 proposal，不开始实施，因此不更新 `docs/current-plan.md`，也不归档计划页。若后续实际落地本提案，应新增或复用 `docs/current-plans/option-lifecycle-guard.md`，并在引入 option lifecycle 存储、public/admin API、event-engine delivery hints 或 resolution ledger 时同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`，必要时补充“期权提醒不是交易建议”的长期约束。
