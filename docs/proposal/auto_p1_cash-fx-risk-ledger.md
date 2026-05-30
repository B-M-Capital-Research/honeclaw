# Proposal: Cash and FX Risk Ledger for Portfolio Discipline

status: proposed
priority: P1
created_at: 2026-05-30 08:04:02 +0800
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
- `docs/proposal/auto_p1_portfolio-transaction-ledger.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_corporate-action-reconciliation.md`
- `docs/proposal/auto_p1_investment-policy-guardrails.md`
- `docs/proposal/auto_p1_instrument_identity_registry.md`
- `docs/proposal/auto_p2_brokerage-readonly-sync-gateway.md`
- `docs/proposal/auto_p2_portfolio-performance-attribution.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-channels/src/scheduler.rs`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/pages/public-portfolio.tsx`

## 背景与现状

Hone 当前已经把 portfolio 放在多个核心链路中心：Web / desktop / IM 维护持仓，event-engine 用持仓构建 watch pool 和主动推送受众，company portraits 和 public `/portfolio` 展示长期研究主线，未来的 exposure、transaction ledger、performance attribution 和 brokerage sync 也都依赖这层投资上下文。

代码层面，portfolio 仍是一个以证券持仓为主的当前态快照：

- `memory/src/portfolio.rs` 的 `Holding` 包含 `symbol`、`asset_type`、`shares`、`avg_cost`、期权字段、`holding_horizon`、`strategy_notes`、`notes` 和 `tracking_only`，但没有账户基础币种、现金余额、外币现金、融资余额、可用保证金或 FX rate metadata。
- `crates/hone-tools/src/portfolio_tool.rs` 的工具参数同样围绕股票 / 期权持仓 CRUD，`strategy_notes` 可以写“现金担保卖沽”，但工具并不知道用户是否真有对应现金储备。
- `crates/hone-web-api/src/routes/portfolio.rs` 的 summary 目前是 holdings 数、watchlist 数、总股数和更新时间。
- `packages/app/src/components/portfolio-detail.tsx` 在表格里直接用 `Intl.NumberFormat(..., currency: "USD")` 显示 `avg_cost` 和 `shares * avg_cost`，隐含假设所有成本都是美元。
- `crates/hone-event-engine/src/subscription.rs` 只从 holdings/watchlist 提取 symbol watch pool；现金比例、外币暴露、融资约束和 FX 变化不会影响通知优先级。
- `crates/hone-channels/src/scheduler.rs` 已要求引用股价、金价、汇率或商品价格时核实时间戳，说明系统对时间敏感价格口径已有警觉，但 portfolio 侧还没有把 FX rate 当成一等事实。

现有提案已经覆盖许多相邻主题：

- `Portfolio Transaction Ledger` 会记录 buy/sell/cash_dividend/fee/transfer 等流水，并在 schema 中预留 `currency`，但明确把 cash balance 放到后续版本。
- `Corporate Action Reconciliation` 会为分红生成 cash effect，但也明确不新增 portfolio cash balance source of truth。
- `Portfolio Exposure Radar` 和 `Portfolio Performance Attribution` 都需要更可靠的组合权重与收益口径，但在现金流缺失、现金余额缺失、多币种缺失时只能降级。
- `Investment Policy Guardrails` 允许用户声明“只关注美股和现金类资产”等纪律，但没有可机读现金与 FX 状态来执行这些纪律。

这形成了一个产品和架构缺口：Hone 可以知道用户持有哪些证券，却不知道用户保留多少现金、现金在哪个币种、是否有足够现金支持某个期权策略、组合真实基础币种是什么、以及外汇波动是否正在改变用户的风险敞口。对一个强调投资纪律的助手来说，现金不是空白资产，而是风险预算、机会成本、仓位纪律、期权安全边界和业绩基准的一部分。

## 问题或机会

这是 P1。它不直接阻断 P0 级安全事故，但会显著影响 portfolio truth、投资纪律、组合分析质量、期权风险解释、付费用户信任和未来交易流水 / 券商同步的落地价值。

主要问题：

1. **组合没有基础币种。**  
   现在 UI 和很多推导隐含 USD 口径。对持有港股、A 股、美股、现金类基金、外币现金或跨市场 ETF 的用户，`shares * avg_cost` 在没有 currency 字段时不能稳定比较权重、收益或集中度。

2. **现金储备不可见。**  
   Hone 可以记录一只股票或期权，却不能回答“我现在现金比例多少”“这笔操作会不会突破现金纪律”“现金担保卖沽是否真的有现金覆盖”。这会削弱投资政策、期权 lifecycle guard 和 position advice 的可信度。

3. **分红、费用、转入转出没有账户余额承接。**  
   现有公司行动和交易流水提案都能产生 cash effect，但没有 cash account layer 时，这些 effect 只能停留在 ledger 附注，无法进入组合视图、收益归因或通知判断。

4. **FX 变化无法进入风险提醒。**  
   对非美元本位用户，美元资产上涨但本币升值、港币/美元 peg 风险、日元资产汇率波动、人民币现金和美元持仓错配，都可能改变真实收益和风险。当前 event-engine 的 symbol watch pool 无法表达这类非证券标的。

5. **业绩归因会持续低置信度。**  
   `Portfolio Performance Attribution` 已指出现金流缺失会导致报告降级。如果不补现金与 FX 层，未来即使有交易流水，也难以准确处理 idle cash、cash drag、分红再投资、手续费和多币种收益。

6. **用户信任叙事不完整。**  
   严肃投资助手不能只看“买了什么”，还要看“还有多少风险预算未使用、现金是否支持策略、收益是否超过现金/基准、外币风险是否被忽略”。这是从聊天助手走向投资工作台的关键体验。

机会是新增 **Cash and FX Risk Ledger**：一个 actor-scoped、账户级、保守计算的现金与币种层。它不取代 portfolio holdings，也不做交易执行；它为 portfolio、policy、exposure、performance、event-engine 和 agent 回答提供统一的 cash / FX context。

## 方案概述

新增 actor-scoped 的现金与汇率风险层，第一版聚焦四个对象：

1. `CashAccount`
   用户确认过的现金账户或现金桶，例如 `USD brokerage cash`、`HKD cash`、`CNY bank reserve`、`money_market_fund_like_cash`。记录币种、余额、可用/冻结、来源、更新时间和置信度。

2. `CashLedgerEntry`
   现金变动流水，例如 dividend、fee、transfer_in、transfer_out、interest、manual_adjustment、cash_secured_option_reserve、release_reserve。它可以由未来 transaction ledger、corporate action reconciliation 或手工确认生成。

3. `FxRateSnapshot`
   汇率观察值，记录 base currency、quote currency、rate、source、fetched_at、valid_until、payload hash 和 freshness status。它不要求第一版接实时外汇 API，但必须把汇率时间戳和来源作为模型输入边界。

4. `PortfolioCurrencyView`
   面向组合分析的派生视图：base currency、cash by currency、securities cost by currency、estimated total value、cash ratio、unhedged FX exposure、unknown currency amount 和 limitations。

核心原则：

- 现金和 FX 是投资上下文，不是交易执行能力。
- 不从自然语言里静默创建大额现金余额；真实现金余额需要用户确认或导入确认。
- 没有 freshness 的 FX rate 只能产生低置信度分析，不能包装成实时事实。
- 第一版保留现有 `PortfolioStorage` JSON，不做破坏性迁移；新增 ledger 与派生 view 并行。
- 期权策略只做风险解释和 reserve check，不自动判断保证金充足或允许卖出。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加 `现金与币种` 区块：
  - 基础币种：用户可选 USD / CNY / HKD / EUR 等。
  - 现金余额：按币种显示可用、预留、更新时间和来源。
  - 组合口径：以基础币种估算证券成本、现金比例和未知币种金额。
  - FX freshness：例如 `USD/CNY rate fetched 2026-05-30 07:55` 或 `FX stale, cannot estimate base-currency return`。
- 持仓编辑表单增加 currency 字段，默认可由 instrument registry 或用户基础币种推断，但必须可见可改。
- 当用户问“我还能不能加仓”“我的现金够不够覆盖卖沽”“我的真实美元暴露是多少”时，Hone 先说明 cash / FX 口径、缺失项和不确定性，再拆解纪律问题。
- 当现金或 FX 数据缺失时，界面提示用户补充，而不是把缺失当作 0。

### 管理端

- `/users/:actor/portfolio` 增加 cash / FX tab：
  - cash accounts、ledger entries、manual adjustments、pending dividend cash effects。
  - FX snapshots 和 stale rate warnings。
  - securities by currency、cash by currency、unknown-currency holdings。
- Admin 可以帮助用户创建现金账户草稿或修正币种，但大额余额变更要进入 `manual_adjustment` ledger，而不是直接覆盖一行 current state。
- 用户抱怨收益或仓位计算不准时，admin 能快速看到是不是缺 cashflow、缺 FX、持仓币种未知或 FX rate stale。

### 桌面端

- Desktop bundled 模式本地保存 cash / FX ledger，适合不愿上云的高敏感用户。
- Dashboard 可显示紧凑状态：`现金 18% · USD/CNY stale · 2 个持仓币种未知`。
- Remote backend 模式显示现金与汇率数据所在 backend，避免用户误以为桌面本地现金状态已经自动同步。

### 多渠道

- 私聊中支持低风险查询：
  - “我的现金比例是多少？”
  - “卖出这张 put 需要预留多少现金？”
  - “如果美元兑人民币跌 3%，我的组合口径会怎么变？”
- IM 回复只给摘要、口径和最多 3 个缺口；完整 cash ledger 和 FX 表引导回 Web/desktop。
- 群聊默认不暴露个人现金、融资或币种细节；如果用户在群里问个人现金相关问题，应引导私聊。

## 技术方案

### 1. 类型与存储

建议在 `memory` 新增 `cash_fx` 模块，第一版用 SQLite 存账户和流水，按 actor 隔离。若 Cloud PG 迁移已完成，应通过 repository trait 抽象本地 SQLite 与 PG。

```text
cash_accounts (
  account_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  label TEXT NOT NULL,
  currency TEXT NOT NULL,
  balance NUMERIC NOT NULL,
  reserved_balance NUMERIC NOT NULL DEFAULT 0,
  source TEXT NOT NULL,
  confidence TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_at TEXT NOT NULL
)

cash_ledger_entries (
  entry_id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  entry_type TEXT NOT NULL,
  currency TEXT NOT NULL,
  amount NUMERIC NOT NULL,
  related_symbol TEXT,
  related_event_id TEXT,
  related_transaction_id TEXT,
  status TEXT NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL,
  effective_at TEXT NOT NULL
)

fx_rate_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  base_currency TEXT NOT NULL,
  quote_currency TEXT NOT NULL,
  rate NUMERIC NOT NULL,
  source TEXT NOT NULL,
  fetched_at TEXT NOT NULL,
  valid_until TEXT,
  payload_hash TEXT,
  freshness TEXT NOT NULL,
  metadata_json TEXT
)
```

需要在 `Holding` 增加可选 `currency` 字段或通过 instrument registry 派生。兼容策略：

- 旧 portfolio JSON 缺 currency 时，按 `InstrumentIdentityRegistry` 推断；无法推断则标记 `unknown_currency`。
- UI 显示旧持仓时不强制迁移，但保存时写入 currency。
- `avg_cost` 保持原币种单价，不把历史数据悄悄换算成基础币种。

### 2. 派生 view

新增 deterministic 计算 helper：

```rust
pub struct PortfolioCurrencyView {
    pub actor: ActorIdentity,
    pub base_currency: String,
    pub generated_at: DateTime<Utc>,
    pub cash_by_currency: Vec<CashBucket>,
    pub securities_by_currency: Vec<SecuritiesCurrencyBucket>,
    pub estimated_total_base_value: Option<Decimal>,
    pub cash_ratio_base: Option<Decimal>,
    pub fx_exposures: Vec<FxExposure>,
    pub limitations: Vec<CurrencyViewLimitation>,
}
```

`CurrencyViewLimitation` 至少包含：

- `unknown_holding_currency`
- `missing_fx_rate`
- `stale_fx_rate`
- `missing_cash_account`
- `cash_balance_unconfirmed`
- `option_reserve_estimate_only`
- `negative_cash_or_margin_unmodeled`

这个 view 可以被 exposure radar、performance attribution、policy verdict 和 chat tool 复用。

### 3. API 与工具

新增 Web API：

- `GET /api/portfolio/cash-fx?channel=&user_id=&channel_scope=`
- `POST /api/portfolio/cash-accounts`
- `PUT /api/portfolio/cash-accounts/:id`
- `POST /api/portfolio/cash-ledger/manual-adjustment`
- `GET /api/portfolio/currency-view?base_currency=`
- `GET /api/public/portfolio/currency-view`

新增 agent 工具可命名为 `cash_fx`：

- `action=view`
- `action=add_cash_account`
- `action=manual_adjustment`
- `action=reserve_for_option`
- `action=release_reserve`
- `action=currency_view`

工具写入必须带 reason/note；大额 manual adjustment 可接入未来 `Agent Mutation Ledger` 或 `Permission Broker`。

### 4. 与现有模块集成

- `Portfolio Transaction Ledger`：交易确认后写 securities transaction，同时产生现金 debit/credit entry。
- `Corporate Action Reconciliation`：cash dividend candidate 确认后写 `cash_ledger_entries(entry_type=dividend)`，不再只是 UI 附注。
- `Portfolio Exposure Radar`：用 cash ratio、unknown currency 和 FX exposure 增加质量 flag。
- `Portfolio Performance Attribution`：用 cash ledger 处理 cashflow，减少 `cashflow_unknown` 限制；多币种收益必须引用 `FxRateSnapshot`。
- `Investment Policy Guardrails`：支持 `min_cash_ratio_pct`、`max_unhedged_fx_exposure_pct`、`cash_secured_option_requires_reserve` 等 rule。
- `Instrument Identity Registry`：为证券默认交易币种提供推断来源，但不替代用户确认。
- `Event Engine`：第一版不需要全天候 FX polling；可以从用户配置的 currency pairs 建立低频 watch pool，未来再进入 source provenance/freshness registry。

## 实施步骤

### Phase 1: 数据模型与只读 view

- 在 `memory` 新增 cash / FX 类型、SQLite schema 和 repository trait。
- 为 `Holding` 增加可选 `currency` 字段，并保证旧 JSON 反序列化兼容。
- 实现 `PortfolioCurrencyView` 的纯计算 helper。
- 添加单元测试覆盖单币种、缺 currency、缺 FX、stale FX、多现金账户和负现金限制。

### Phase 2: Web/Admin 最小表面

- 新增 admin API 和 public read-only currency view API。
- 在 admin portfolio detail 增加 cash accounts 与 currency view。
- 在 public `/portfolio` 增加现金比例、基础币种、FX freshness 和缺口提示。
- 持仓编辑表单展示 currency 字段；旧数据保存时写入。

### Phase 3: Agent 工具与策略联动

- 新增 `cash_fx` 工具，只允许 actor-scoped 读写。
- 将期权 reserve check 接入 position / portfolio prompt，但不做保证金承诺。
- 为 investment policy verdict 提供 cash / FX context。
- IM 私聊支持只读查询和小额/明确确认的手工调整入口。

### Phase 4: 交易流水与事件联动

- 交易流水 apply 时生成现金 debit/credit。
- 分红 reconciliation apply 时生成 dividend cash entry。
- Performance attribution 优先读取 cash ledger，减少 cashflow limitation。
- 为 FX rate source 接入 source freshness 记录，形成可审计价格口径。

## 验证方式

- Rust unit tests：
  - 旧 portfolio JSON 缺 `currency` 可正常读取。
  - `PortfolioCurrencyView` 对缺 FX/stale FX/unknown currency 产生 limitation，不输出伪精确总值。
  - cash ledger manual adjustment 正确更新账户余额并保留 entry。
  - reserve/release 不允许 reserved balance 超过余额，除非显式标记 margin/unmodeled。
- API tests：
  - admin 按 actor 读取 cash/fx 视图。
  - public 端只能读取当前登录 actor 的 currency view。
  - 大额或缺 note 的 manual adjustment 被拒绝或进入 confirm path。
- Frontend tests：
  - currency view 空状态、缺 FX、stale FX、多币种现金、unknown currency badge 渲染正确。
  - `formatMoney` 不再固定 USD，而是按 holding/account currency 显示。
- Manual acceptance：
  - 建立 USD + CNY cash accounts，加入 USD 股票和 HKD 股票，查看基础币种 CNY 的估算与 limitation。
  - 创建 cash-secured put reserve，确认 cash available 下降、release 后恢复。
  - 分红事件确认后能在 cash ledger 看到 dividend entry。

## 风险与取舍

- **风险：用户把现金余额当成券商级精确账户。**  
  取舍：第一版所有余额都带 source/confidence/updated_at；手工余额不是券商对账结果，UI 必须清楚标注。

- **风险：FX 数据 stale 导致错误收益判断。**  
  取舍：没有 fresh FX snapshot 时不输出基础币种精确收益，只输出缺口和原币种视图。

- **风险：期权 reserve 被误解为保证金计算。**  
  取舍：只支持现金担保估算和用户纪律检查，不做 broker margin、SPAN、组合保证金或下单建议。

- **风险：与 transaction ledger 重叠。**  
  取舍：transaction ledger 记录证券/账户事件；cash_fx ledger 负责现金账户余额、预留、FX 口径和派生 view。前者可以驱动后者，但不替代后者。

- **风险：多币种引入大量 UI 复杂度。**  
  取舍：第一版只在 portfolio 关键位置显示 currency 和 limitation，不把所有页面改成会计系统。

- **明确不做：**
  - 不接真实银行或券商资金划转。
  - 不做税务、融资利息或保证金合规计算。
  - 不自动换汇、下单或建议用户把现金换成某资产。
  - 不把缺失现金当作 0 参与投资纪律判断。

## 与已有提案的差异

查重范围：已检查 `docs/proposal/` 与 `docs/proposals/` 下现有提案文件名、标题和相关摘要，并额外搜索 `cash`、`currency`、`fx`、`avg_cost`、`cost_basis`、`现金`、`汇率` 等关键词。

- 与 `auto_p1_portfolio-transaction-ledger.md` 不重复：该提案解决交易流水、导入预览和当前持仓重建；本提案解决现金账户余额、现金预留、基础币种、FX freshness 和 currency view。交易流水可以产生 cash entry，但不是 cash / FX view 本身。
- 与 `auto_p1_portfolio-exposure-radar.md` 不重复：exposure radar 关注证券暴露、质量 flag 和场景 guardrail；本提案提供 cash ratio、cash reserve、unhedged FX exposure 和 unknown currency 作为其输入。
- 与 `auto_p1_corporate-action-reconciliation.md` 不重复：公司行动提案处理 split/dividend 等事件是否需要调账；本提案把确认后的 dividend cash effect 落到可查询现金账本。
- 与 `auto_p2_portfolio-performance-attribution.md` 不重复：performance 提案处理收益与基准归因；本提案补足 cashflow 和 FX 口径，使 performance 不再长期依赖 `cashflow_unknown` 降级。
- 与 `auto_p1_investment-policy-guardrails.md` 不重复：policy guardrails 定义用户纪律和 verdict；本提案提供执行现金比例、现金担保和 FX 暴露规则所需的数据层。
- 与 `auto_p1_instrument_identity_registry.md` 不重复：instrument registry 解决证券身份、交易所和默认币种；本提案解决 actor 账户里的现金余额、基础币种、FX rate 和 portfolio currency view。
- 与 `auto_p2_brokerage-readonly-sync-gateway.md` 不重复：brokerage sync 是未来外部只读连接；本提案可在无券商连接时手工维护，也可作为券商现金快照进入 Hone 后的标准落点。
