# Proposal: Corporate Action Reconciliation for Portfolio Truth

status: proposed
priority: P1
created_at: 2026-05-15 08:02:15 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/pollers/corp_action.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/router/policy.rs`
- `crates/hone-event-engine/src/store.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/pages/notifications-model.ts`
- `packages/app/src/lib/admin-content/notifications.ts`
- `skills/portfolio_management/SKILL.md`
- `skills/scheduled_task/SKILL.md`

## 背景与现状

Hone 已经把 portfolio 做成多个核心链路的输入：

- `memory/src/portfolio.rs` 按 `ActorIdentity` 保存持仓和关注列表，单条 `Holding` 记录 `symbol`、`asset_type`、`shares`、`avg_cost`、期权字段、持有期限、策略备注和 `tracking_only`。
- `crates/hone-tools/src/portfolio_tool.rs` 与 `skills/portfolio_management/SKILL.md` 支持 agent 通过 `view/add/update/remove/watch/unwatch` 维护用户持仓和 watchlist。
- `crates/hone-web-api/src/routes/portfolio.rs` 提供管理端持仓 CRUD，并把 summary 简化为持仓数、关注数、总股数和更新时间。
- `crates/hone-event-engine/src/subscription.rs` 从 portfolio 构建订阅池，持仓和 watchlist 都会进入主动推送命中范围。
- `crates/hone-event-engine/src/pollers/corp_action.rs` 已经接入 FMP 的 stock split calendar、stock dividend calendar 和 SEC filings，生成稳定 id 的 `split:{SYM}:{DATE}`、`div:{SYM}:{EXDATE}`、`sec:{SYM}:{ACCESSION}` 事件。
- `crates/hone-event-engine/src/event.rs` 已把 `Dividend` 与 `Split` 作为事实性事件处理，`shelf_life=None`，说明系统认为这类事件跨日仍有价值。
- 通知模型和前端文案已经显示 `dividend`、`split` 事件类型，event router policy 也把它们归入 `corp_action`。

这些能力说明 Hone 能发现公司行动，也能把它们推给相关 actor。但 portfolio 本身仍是静态快照：事件不会形成待确认调账项，拆股不会生成建议的股数/成本调整，现金分红不会形成收益或现金记录，ticker 变更、合并、spin-off 等复杂事件也没有一个可治理的 pending 状态。对一个以投资纪律和长期记忆为核心的产品来说，组合真相源如果长期停留在“用户最初录入的 shares/avg_cost”，后续 exposure radar、event personalization、公司画像复盘和用户信任都会被拖累。

## 问题或机会

当前缺口不是“没有公司行动推送”，而是“公司行动没有进入 portfolio truth 的闭环”。

1. **组合数据会自然过期。**  
   股票拆股后，如果用户没有手动更新 shares 和 avg_cost，portfolio 会继续以旧股数和旧成本参与推送、summary 和未来的 exposure 计算。分红、特别分红、配股、ticker 变更等事件也会让用户真实账户和 Hone 本地状态逐渐偏离。

2. **事实事件和状态变更之间没有确认层。**  
   Event engine 可以告诉用户“某标的发生 split/dividend”，但不能表达“这件事可能需要调整你的本地持仓记录”。直接自动改 portfolio 又有风险：用户可能只是在 watchlist、可能已经卖出、可能持有期权而非正股，或者券商实际处理存在时差。

3. **portfolio 只有当前态，没有变更解释。**  
   当前 `PortfolioStorage` 保存单份 JSON，`portfolio_tool` 的 update/add/remove 会覆盖状态。后续如果用户问“为什么我的 AAPL 股数变了”或“上次分红算进去了吗”，系统缺少调账 ledger 和来源 event id。

4. **通知和研究链路无法区分“已处理”和“待确认”。**  
   `Dividend` / `Split` 现在只是消息事件。若没有 reconciliation 状态，系统无法在 public `/portfolio`、admin `/users/:actor/portfolio`、IM 私聊或 scheduled briefing 中提示“这条公司行动还没确认是否影响你的持仓”。

5. **商业和信任价值被浪费。**  
   投资助手的高价值体验不只是回答问题，而是帮用户保持投资记录可信。确认式调账可以成为用户回到 Web/desktop 的理由，也能减少“为什么 Hone 推送不准/持仓不准”的支持成本。

这是 P1：它不属于输出安全门那种立即阻止错误建议的 P0，但会显著提升 portfolio 可信度、自动化质量、用户留存和未来组合分析能力。第一版可以只做拆股/现金分红的待确认 reconciliation，不需要接券商 API，也不做自动交易。

## 方案概述

新增 actor-scoped 的 **Corporate Action Reconciliation** 层，把公司行动事件转成“可解释、可确认、可回滚的 portfolio 调整建议”。

核心原则：

- 事件不直接改 portfolio。`MarketEvent` 只生成 reconciliation candidate。
- 用户确认后才写入 `PortfolioStorage`，并保留 adjustment ledger。
- watchlist 默认只提示事实，不生成持仓调账建议。
- 正股、期权、现金分红、特殊事件分开处理，避免用一个公式假装精确。
- 第一版只覆盖 deterministic 规则：split ratio、cash dividend per share、symbol match、holding exists、tracking_only、effective date。
- 复杂事件如 spin-off、merger、rights offering、ticker change 先进入 `manual_review`，不自动计算。

建议对象：

- `CorporateActionCandidate`：由 event-engine 事件派生，包含 event id、actor、symbol、kind、source、ex-date/effective-date、ratio/dividend amount、matched holding、suggested status。
- `PortfolioAdjustmentDraft`：可应用的持仓变更草案，包含 before/after shares、before/after avg_cost、cash amount estimate、rounding note、confidence、reason。
- `PortfolioAdjustmentLedger`：已应用或拒绝的调账记录，保存 actor、event id、holding key、operator、applied_at、before snapshot、after snapshot、rollback hint。
- `ReconciliationState`：`pending`、`applied`、`ignored`、`manual_review`、`not_applicable`、`superseded`。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加轻量 `Corporate Actions` 区块：
  - `待确认`：拆股、分红或复杂事件可能影响真实持仓。
  - `已处理`：最近完成的调账，显示来源和结果摘要。
  - `不适用`：watchlist 命中、用户无持仓、或用户明确忽略。
- 对拆股显示可理解的 before/after：
  - `AAPL 4:1 split candidate: 10 shares @ 180 -> 40 shares @ 45`
  - 标注“请与券商账户核对后确认”。
- 对现金分红显示估算：
  - `MSFT dividend 0.75/share, estimated cash 75.00 before tax for 100 shares`
  - 不自动改 `avg_cost`，默认记录为 cash event 或收益备注。
- 用户可执行：
  - `确认应用`
  - `忽略此事件`
  - `我已卖出/数量不同，去更新持仓`
  - `稍后提醒我`
- `/chat` 中用户问“拆股后帮我更新一下”时，agent 先读取 pending candidate，再调用受控工具应用，不从自然语言里重新猜 ratio。

### 管理端

- `/users/:actor/portfolio` 增加 reconciliation panel：
  - pending candidates by symbol/kind/effective date。
  - matched holding、tracking-only、confidence、blocking reason。
  - apply/ignore/manual review 操作入口。
- `/notifications` 的公司行动详情可以展示 reconciliation 状态：`portfolio_adjustment=pending/applied/not_applicable`。
- 管理员可以查看 ledger，回答“某用户持仓为什么变更”。
- 对高风险操作，如 split 应用到大仓位或期权标的，要求二次确认。

### 桌面端

- Desktop bundled/remote 复用同一 Web panel。
- Dashboard 显示“有 N 个公司行动待核对”，点击进入 portfolio。
- 本地单用户模式可以允许更低摩擦的确认；remote/public 模式保守默认，只在用户确认后应用。

### 多渠道

- Feishu / Telegram / Discord 的公司行动推送增加一句短状态：
  - `这可能影响你记录的 AAPL 持仓，已加入待确认。`
  - `这是关注标的，不会调整持仓。`
- 私聊支持 `/portfolio actions` 或自然语言“有哪些公司行动待处理”。
- 群聊不主动暴露个人持仓状态，只保留事实事件或提示去私聊处理。

## 技术方案

### 1. 新增 reconciliation 存储

建议在 `memory` 新增 `portfolio_reconciliation` 模块。第一版用 SQLite 存候选和 ledger，避免继续扩展单个 portfolio JSON：

```text
corporate_action_candidates (
  candidate_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  event_id TEXT NOT NULL,
  event_kind TEXT NOT NULL,
  symbol TEXT NOT NULL,
  source TEXT NOT NULL,
  effective_date TEXT,
  payload_json TEXT NOT NULL,
  matched_holding_key TEXT,
  state TEXT NOT NULL,
  confidence TEXT NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(actor_channel, actor_user_id, actor_scope, event_id)
)

portfolio_adjustment_ledger (
  adjustment_id TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  symbol TEXT NOT NULL,
  adjustment_kind TEXT NOT NULL,
  before_json TEXT NOT NULL,
  after_json TEXT,
  cash_effect_json TEXT,
  decision TEXT NOT NULL,
  decided_by TEXT NOT NULL,
  decided_at TEXT NOT NULL,
  note TEXT
)
```

`before_json` 保存被调整前的 `Holding` 快照；`after_json` 保存应用后的持仓。这样即使 portfolio 后续继续被用户编辑，ledger 仍能解释当时的决定。

### 2. 从 event-engine 派生候选

在 event delivery 或 store 写入后增加一个轻量 hook：

1. 只处理 `EventKind::Split`、`EventKind::Dividend`，以及后续可识别的 ticker change / merger SEC filing。
2. 使用 `SubscriptionRegistry::resolve(event)` 找到命中 actor。
3. 对每个 actor 读取 `PortfolioStorage::load(actor)`。
4. 如果 symbol 只在 watchlist，写 `not_applicable` 或 `watch_only` reason。
5. 如果无持仓，写 `not_applicable`，用于后续排查“为什么没生成调账”。
6. 如果有真实正股持仓，生成 `pending` draft。
7. 如果是期权持仓、缺 shares/avg_cost、ratio 无法解析或事件复杂，生成 `manual_review`。

拆股 draft 规则：

- 输入：`split ratio` 或 poller payload 里的 numerator/denominator。
- `after.shares = before.shares * ratio`
- `after.avg_cost = before.avg_cost / ratio`
- 保留 rounding note，不处理零碎股现金替代。

分红 draft 规则：

- 不改 `shares` 和 `avg_cost`。
- 生成 `cash_effect = before.shares * dividend_per_share`。
- 第一版只记录 ledger 与 UI 提示，不新增 portfolio cash balance source of truth。

### 3. 受控应用 API 与工具

新增 API：

- `GET /api/portfolio/reconciliation?channel=&user_id=&channel_scope=&state=`
- `POST /api/portfolio/reconciliation/{candidate_id}/apply`
- `POST /api/portfolio/reconciliation/{candidate_id}/ignore`
- `POST /api/portfolio/reconciliation/{candidate_id}/snooze`

工具层建议新增到 `portfolio` 工具，而不是新开一个完全独立工具：

- `portfolio(action="list_corporate_actions")`
- `portfolio(action="apply_corporate_action", candidate_id="...")`
- `portfolio(action="ignore_corporate_action", candidate_id="...", note="...")`

这样 `portfolio_management` skill 能保持单一入口。高风险 apply 需要先 `view` 当前 portfolio，并在 tool 内校验 candidate 的 `before_json` 与当前 holding 是否仍匹配；不匹配则拒绝，转 `manual_review`，避免覆盖用户近期手动更新。

### 4. 前端和多渠道展示

- `packages/app/src/context/portfolio.tsx` 读取 reconciliation summary。
- Public `/portfolio` 和 admin portfolio detail 共用同一组派生状态 model。
- `packages/app/src/pages/notifications-model.ts` 对 `dividend/split` detail 增加 reconciliation label。
- 多渠道回复由 event renderer 或 delivery layer 附加短状态，不在消息里泄露完整 shares/avg_cost，除非是私聊且用户已授权查看 portfolio。

### 5. 兼容与迁移

- 不迁移旧 portfolio JSON，不改变 `Holding` schema。
- 旧公司行动事件不自动回填候选；可在 admin 提供“从最近 N 天 event store 生成候选”的手工 backfill。
- `PortfolioStorage` 继续是真实持仓当前态；reconciliation 只是解释和受控变更 ledger。
- 后续若 `auto_p1_agent-mutation-ledger.md` 落地，可把 apply/ignore 决策同步成更通用的 mutation event，但本提案不依赖它。

## 实施步骤

### Phase 1: Candidate model and deterministic drafts

- 在 `memory` 新增 candidate/ledger 类型、SQLite 表、CRUD 和单元测试。
- 从 `MarketEvent` payload 解析 split ratio 与 dividend amount。
- 对 portfolio 中真实正股、watchlist、期权、缺字段分别生成 `pending/not_applicable/manual_review`。
- 增加去重约束，保证同一 actor/event 只生成一次候选。

### Phase 2: Apply/ignore API and portfolio tool extension

- 增加 Web API 列表、apply、ignore、snooze。
- 扩展 `portfolio_tool` 支持列出和处理公司行动候选。
- apply 前校验 current holding 与 candidate before snapshot，失败则不写 portfolio。
- ledger 保存 before/after/cash effect/decision。

### Phase 3: Product surfaces

- Public `/portfolio` 增加待确认区块和详情 drawer。
- Admin portfolio detail 增加 reconciliation panel。
- Notifications detail 显示 candidate state。
- IM 私聊推送附加一行“是否已加入待确认”状态，并支持查询 pending。

### Phase 4: Backfill and advanced events

- 管理端增加最近 N 天 event store backfill。
- 支持 ticker change、merger、spin-off 的 `manual_review` 模板。
- 与 document inbox 的 broker statement 导入联动，用用户上传的券商记录核对候选是否已由券商处理。

## 验证方式

- 单元测试：
  - split ratio 2:1、4:1 正确生成 shares/avg_cost draft。
  - dividend 正确生成 cash effect 且不改 avg_cost。
  - watchlist 命中不生成 apply draft。
  - option holding 命中转 `manual_review`。
  - candidate 去重按 actor + event id 生效。
  - apply 时 current holding 与 before snapshot 不一致会拒绝。
- API 测试：
  - 创建 portfolio 后注入 split event，能查到 pending candidate。
  - apply 后 portfolio 当前态更新，ledger 可查，candidate 变 `applied`。
  - ignore 后 portfolio 不变，candidate 变 `ignored`。
- 前端模型测试：
  - pending/applied/manual_review/not_applicable 显示文案稳定。
  - public 与 admin summary 不泄露群聊或其它 actor 的持仓数据。
- 手工验收：
  - 对一个持有 AAPL 的测试 actor 注入 4:1 split，Web 和 IM 都能提示待确认。
  - 对 watchlist-only actor 注入同一事件，只显示事实，不提供 apply。
  - 对分红事件显示估算现金影响，不改变持仓成本。
- 指标：
  - pending corporate actions 数量、平均确认时长、apply/ignore/manual_review 比例。
  - portfolio 因 split/dividend 触发的手动 update 数是否下降。
  - 公司行动通知后的 `/portfolio` 回访率。

## 风险与取舍

- **风险：自动调整持仓可能误导。**  
  取舍：第一版默认只生成 candidate，必须用户或管理员确认后应用；复杂事件进入 `manual_review`。

- **风险：拆股比例或分红金额来自单一 provider，可能有错误。**  
  取舍：保留 event source 和 payload；与 `source-provenance-freshness` 后续联动。第一版 UI 明确要求用户与券商核对。

- **风险：portfolio 没有 tax lots 和 cash balance，分红处理不完整。**  
  取舍：不在本提案引入完整会计系统。现金分红只作为 ledger/cash effect 记录，未来再决定是否新增 cash account。

- **风险：用户已经手动更新持仓，再 apply 会覆盖新状态。**  
  取舍：apply 必须校验 before snapshot，发现差异时拒绝并转 manual review。

- **风险：事件回填过多造成噪音。**  
  取舍：默认只处理新事件；历史 backfill 必须管理员手动触发并限制时间窗口。

- **不做边界：**
  - 不接券商交易 API。
  - 不做交易、税务或自动现金管理建议。
  - 不把 watchlist 当作真实持仓调整。
  - 不直接修改 company portrait；公司行动是否改变长期 thesis 仍交给 evidence review / company portrait 流程。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 全部自动提案和历史 `docs/proposals/`。

- 与 `auto_p1_portfolio-exposure-radar.md` 不重复：exposure radar 生成组合风险派生视图；本提案治理 portfolio 当前态如何因公司行动被确认式更新。
- 与 `auto_p1_investment_context_intake.md` 不重复：context intake 解决新用户如何补齐初始持仓、画像和偏好；本提案解决持仓建立后的公司行动调账。
- 与 `auto_p1_investment_document_inbox.md` 不重复：document inbox 处理用户上传材料资产化；本提案由 event-engine 公司行动事件驱动，可后续用券商文件交叉校验，但第一版不依赖上传材料。
- 与 `auto_p1_agent-mutation-ledger.md` 不重复：mutation ledger 是通用状态变更审计；本提案定义投资组合公司行动候选、计算规则、确认 UX 和 portfolio-specific apply 校验。
- 与 `auto_p1_source-provenance-freshness.md` 不重复：source provenance 记录金融事实来源和时效；本提案使用这些事实生成 portfolio reconciliation，并保留来源以便核对。
- 与 `auto_p1_evidence_review_queue.md` 不重复：evidence review 处理 thesis 是否需要更新；本提案处理 shares/avg_cost/cash effect 这类组合记录是否需要调整。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 和 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不重复：两者分别关注桌面启动体验和 skill runtime，对 portfolio reconciliation 没有覆盖。
