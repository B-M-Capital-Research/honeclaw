# Proposal: Portfolio Performance Attribution and Benchmark Review

status: proposed
priority: P2
created_at: 2026-05-29 02:04:10 +0800
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
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_signal-outcome-calibration.md`
- `docs/proposal/auto_p2_brokerage-readonly-sync-gateway.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-event-engine/src/tests.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/lib/types.ts`

## 背景与现状

Honeclaw 现在已经有一个清晰的投资工作台骨架：用户可以在 Web、桌面和 IM 里维护 portfolio，系统用 company portraits、mainline distill、event engine、scheduled tasks 和多渠道通知帮助用户跟踪持仓与关注列表。仓库中的关键事实包括：

- `memory/src/portfolio.rs` 以 `ActorIdentity` 隔离 portfolio JSON，`Holding` 记录 `symbol`、`asset_type`、`shares`、`avg_cost`、期权字段、`holding_horizon`、`strategy_notes`、`notes` 和 `tracking_only`。
- `crates/hone-tools/src/portfolio_tool.rs` 暴露 `view/add/update/remove/watch/unwatch`，适合让 agent 维护当前持仓和 watchlist，但本质仍是当前态快照。
- `crates/hone-web-api/src/routes/portfolio.rs` 的 summary 目前只包含 holdings 数、watchlist 数、总股数和更新时间。
- Public `/portfolio` 当前重点展示整体投资风格、各持仓投资主线和只读公司画像，不展示组合回报、相对基准、贡献来源或错失成本。
- `crates/hone-tools/src/data_fetch.rs` 已能获取 quote、profile、financials、news、sector performance、ETF holdings、earnings calendar 等市场数据，为未来的价格序列与基准抓取提供入口。
- `crates/hone-event-engine/src/tests.rs` 里已有盘前快照测试样例会手工组装 `P&L vs 成本`，说明产品已有“用户关心盈亏变化”的自然需求，但它还没有成为稳定的架构层。

现有提案已经补齐了不少前置拼图：`Portfolio Transaction Ledger` 处理交易流水与导入确认，`Portfolio Exposure Radar` 处理暴露和质量 flag，`Trade Discipline Journal` 处理操作意图与纪律复盘，`Signal Outcome Calibration Ledger` 处理判断后来是否被事实验证。这些仍没有直接回答一个高意向投资用户每天都会问的问题：我的组合表现到底来自哪里，和我本来应该比较的基准相比，Hone 是否帮助我保持了纪律。

## 问题或机会

这是 P2。它对长期留存、付费感知和投资复盘价值很强，但需要可靠的 portfolio 快照历史、交易流水或至少周期性持仓快照作为前置，否则容易把不完整数据包装成精确业绩。

当前缺口主要有五类：

1. **用户看不到组合层面的真实结果。**  
   Hone 能解释持仓主线、事件和风险，但没有一等视图展示组合在一周、一月、一季内的收益、回撤、波动和贡献来源。用户只能把每个 ticker 的回答拼起来，无法形成整体复盘。

2. **没有基准，盈亏容易被误读。**  
   如果 NVDA 盈利 8%，但同期香港科技或半导体基准涨 15%，这不是同一种结论。缺少 SPY、QQQ、行业 ETF、自定义基准或现金收益率对比时，用户容易把 beta 当 alpha，把运气当纪律。

3. **当前 portfolio 快照无法回答“谁贡献了收益”。**  
   `shares * avg_cost` 能估算成本口径 P&L，但无法稳定处理现金流、分批交易、部分卖出、期权开平仓、股息、拆股和转仓。即使第一版只做估算，也应该明确数据置信度和限制。

4. **Hone 的产品价值难以闭环。**  
   投资纪律产品不应只证明“我提醒过你”，还应帮助用户复盘“哪些提醒、哪些主线、哪些不操作带来了结果”。否则 signal calibration、trade journal 和 notification backtest 都停留在过程质量，缺少组合结果的用户语言。

5. **管理端缺少 high-intent 用户的健康指标。**  
   当前 admin 能看用户、sessions、portfolio、notifications、LLM audit，但无法识别哪些用户已经进入长期复盘阶段、哪些用户数据不足以做绩效结论、哪些用户可能因为看不到结果闭环而流失。

机会是新增 **Portfolio Performance Attribution and Benchmark Review**：把组合快照、交易流水、市场价格和用户自选基准汇成一个保守、可解释、明确标注置信度的复盘层。它不是荐股系统，也不是税务级报表，而是让 Hone 从“研究与提醒助手”进化为“能复盘投资纪律是否有效的工作台”。

## 方案概述

新增 actor-scoped 的 performance review 层，第一版以只读派生数据为主，不改变 portfolio 真相源。

核心对象：

- `PortfolioPerformanceSnapshot`：某个 actor 在某个日期或时间窗口的组合估值快照，包含 holdings、价格来源、基准价格、估值口径和缺失项。
- `PortfolioReturnSeries`：按日或按周生成的组合收益序列，第一版可从 portfolio 快照和价格缓存估算，后续由 transaction ledger 重建。
- `BenchmarkProfile`：用户或系统为 actor 选择的比较基准，例如 `SPY`、`QQQ`、`VT`、`SOXX`、`cash` 或自定义多基准权重。
- `PerformanceAttributionReport`：一个窗口内的收益、基准收益、超额收益、最大回撤、贡献前后若干项、未覆盖资产、置信度和解释文本。
- `ReviewPromptCandidate`：当某个窗口出现明显偏离时生成的复盘建议，例如“组合跑输 QQQ 主要来自两只未建画像的仓位”。

第一版目标：

1. 建立 benchmark profile 与估值快照类型。
2. 对股票持仓做保守的成本口径和市场价口径收益估算。
3. 对期权、缺成本、缺价格、watchlist、现金流缺失等场景明确降级为 `low_confidence`。
4. 在 public `/portfolio` 和 admin portfolio detail 展示“近 30 天复盘摘要”，而不是完整机构级绩效系统。
5. 给 agent 一个只读工具或 API，回答“我这段时间为什么跑赢/跑输基准”。

明确不做：

- 不承诺税务报表、FIFO/LIFO、wash sale 或券商对账精度。
- 不根据业绩自动推荐买卖。
- 不把短期相对收益作为主线正确性的唯一标准。
- 不在没有足够快照/价格/交易流水时假装可以算精确 alpha。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“业绩复盘”区块：
  - 默认显示近 30 天组合估算收益、基准收益、差值和置信度。
  - 显示贡献最大的 3 个持仓和拖累最大的 3 个持仓。
  - 显示数据限制：例如 `2 个持仓缺价格`、`期权未做 delta 调整`、`现金流缺失，收益为估算`。
  - 提供“用 QQQ / SPY / SOXX / 自定义基准比较”的轻量选择。
- 当用户问“我最近跑赢了吗”时，Hone 先给出基准和数据口径，再解释贡献，而不是只罗列涨跌。
- 当数据不足时，Hone 应明确说“现在只能做成本口径估算”，并引导用户补充交易流水或确认 portfolio，而不是输出精确百分比。
- 复盘文案保持投资纪律导向：重点回答“哪些假设贡献了结果、哪些数据不足、下次应该复核什么”，不转成短线交易建议。

### 管理端

- `/users/:actor/portfolio` 增加 `Performance` tab：
  - actor 的 benchmark profile、快照覆盖率、价格覆盖率、最近生成时间。
  - 窗口选择：7d、30d、quarter-to-date、自定义。
  - holdings contribution 表：symbol、asset type、weight、return、contribution、confidence、missing reason。
  - 数据质量提示：缺交易流水、缺基准、缺价格、期权估值不足、watchlist 排除。
- Admin dashboard 可以显示进入复盘阶段的用户数、近 30 天有可用 performance snapshot 的 actor 数，以及 `low_confidence` 占比。
- 支持把 performance report deep link 到 trade journal、signal calibration、company portrait，便于排查“判断正确但仓位没贡献”或“仓位贡献来自 beta 而非主线”。

### 桌面端

- Desktop bundled 模式使用本地 portfolio、价格缓存和快照，不依赖云端。
- Remote backend 模式显示 report 的后端来源和生成时间，避免用户误以为本地 portfolio 已经同步。
- Tray 或 dashboard 可显示一句克制摘要：`30d vs QQQ: -1.8pp, low confidence: 2 missing prices`。

### 多渠道

- Feishu / Telegram / Discord 私聊支持简短查询：
  - “这个月组合相对 QQQ 表现怎么样？”
  - “最近拖累最大的是谁？”
  - “这次跑赢是靠 alpha 还是行业 beta？”
- IM 回复只给摘要、基准、贡献前后 3 项和置信度；完整表格引导回 Web/desktop。
- 群聊默认不展开个人 portfolio performance，除非未来有明确 group workspace 权限。

## 技术方案

### 1. 数据模型

建议在 `memory` 新增 `portfolio_performance` 模块，第一版使用 SQLite 或按 actor 的 JSONL 快照。若 Cloud PG 迁移已落地，应抽象 repository trait，避免再次绑定本地文件。

建议类型：

```rust
pub struct BenchmarkProfile {
    pub actor: ActorIdentity,
    pub benchmark_id: String,
    pub components: Vec<BenchmarkComponent>,
    pub default_window_days: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct PortfolioPerformanceSnapshot {
    pub actor: ActorIdentity,
    pub as_of: DateTime<Utc>,
    pub valuation_basis: ValuationBasis,
    pub positions: Vec<PerformancePosition>,
    pub benchmark_values: Vec<BenchmarkValue>,
    pub source_versions: PerformanceSourceVersions,
    pub limitations: Vec<PerformanceLimitation>,
}

pub struct PerformanceAttributionReport {
    pub actor: ActorIdentity,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub portfolio_return_pct: Option<f64>,
    pub benchmark_return_pct: Option<f64>,
    pub excess_return_pct: Option<f64>,
    pub max_drawdown_pct: Option<f64>,
    pub contributors: Vec<PositionContribution>,
    pub detractors: Vec<PositionContribution>,
    pub confidence: PerformanceConfidence,
    pub limitations: Vec<PerformanceLimitation>,
}
```

`ValuationBasis` 建议包含：

- `cost_basis_estimate`
- `market_price_estimate`
- `ledger_rebuilt`
- `broker_snapshot`

`PerformanceLimitation` 建议包含：

- `missing_price`
- `missing_cost_basis`
- `cashflow_unknown`
- `option_not_delta_adjusted`
- `corporate_action_unreconciled`
- `watchlist_excluded`
- `benchmark_missing`
- `insufficient_history`

### 2. 快照生成

第一版快照可以从现有 portfolio 当前态和行情工具生成：

- 股票持仓：`shares * latest_price` 作为 market value；缺 latest price 时用 `shares * avg_cost` 并标记 limitation。
- 期权持仓：只使用用户记录的 `quantity * avg_cost * multiplier` 做成本口径，不尝试精确 Greeks；若缺 expiration/strike/type，直接标记不可估。
- watchlist：默认不进入真实组合收益，只作为“错失观察”后续扩展。
- 现金、股息、手续费：除非 transaction ledger 提供，否则第一版不纳入收益，只在 limitation 中声明。

生成时机：

- 用户打开 `/portfolio` 时按需生成短 TTL report。
- 每日 digest 或 scheduled review 可生成 daily snapshot。
- transaction ledger 或 portfolio 更新后标记 cache stale。

### 3. 基准 profile

Benchmark profile 需要可解释、低门槛：

- 默认：美股用户可选 `SPY` 或 `QQQ`，但必须让用户确认，不把单一基准强行当真相。
- 行业集中用户可添加 `SOXX`、`SMH`、`XLK`、`XLE` 等自定义 ticker。
- 支持多基准权重，例如 `60% QQQ + 40% cash`，用于更贴近用户真实策略。
- 每个 report 都显示使用的 benchmark 和价格时间戳。

### 4. API 与工具

新增只读 API：

- `GET /api/portfolio/performance?channel=&user_id=&channel_scope=&window=30d&benchmark=`
- `GET /api/public/portfolio/performance?window=30d&benchmark=`
- `PUT /api/portfolio/benchmark-profile`
- `PUT /api/public/portfolio/benchmark-profile`

新增 agent 工具可选命名为 `portfolio_performance`：

- `action=summary`
- `action=attribution`
- `action=set_benchmark`
- `action=list_benchmarks`

工具输出必须包含 `confidence` 和 `limitations`，并在 prompt 侧要求模型不得把低置信度估算包装成精确业绩。

### 5. 与现有模块集成

- `memory/src/portfolio.rs`：继续作为 current snapshot 输入，不在性能层直接改写。
- `auto_p1_portfolio-transaction-ledger.md` 落地后：performance report 优先使用 ledger-rebuilt holdings 和 cashflow，提升 confidence。
- `auto_p1_portfolio-exposure-radar.md`：复用 weight、quality flag、option limitation；exposure 说明风险结构，performance 说明结果贡献。
- `auto_p1_trade_discipline_journal.md`：把某次操作或不操作与后续 attribution report 关联，但不把短期跑输自动判定为纪律错误。
- `auto_p1_signal-outcome-calibration.md`：对关键判断增加 performance observation，例如“判断被事实支持，但仓位权重太小，对组合贡献有限”。
- `crates/hone-tools/src/data_fetch.rs`：复用 quote / sector performance / ETF data 能力；若需要历史价格，应新增明确的 `historical_price` data type，而不是滥用实时 quote。
- `crates/hone-event-engine/src/tests.rs` 中已有 P&L 快照样例可作为第一批回归 fixture 灵感，但正式实现应移到独立模块和测试。

## 实施步骤

### Phase 1: Conservative report prototype

- 定义 performance types 和 deterministic calculation helpers。
- 从 `PortfolioStorage` 读取当前 holdings，调用价格 provider 或 fixture 生成单日估值。
- 实现 `BenchmarkProfile` 的本地存储与默认选择。
- 新增单元测试覆盖普通股票、缺价格、缺成本、watchlist 排除、期权 limitation。
- 输出 `PerformanceAttributionReport`，但先不接 UI。

### Phase 2: Web/Admin read-only surface

- 增加 admin 和 public performance API。
- 在 `packages/app/src/lib/types.ts` / `api.ts` 增加类型和 client。
- Public `/portfolio` 增加 30d summary，数据不足时优先显示缺口。
- Admin portfolio detail 增加 Performance tab 和 confidence/limitation 表。

### Phase 3: Scheduled snapshot and benchmark review

- 增加每日或每周 snapshot 生成，不阻塞主聊天链路。
- 支持 benchmark profile 编辑。
- 生成“本周复盘候选”供 scheduled task 或 digest 使用。
- 对 price provider 失败、基准缺失、数据太短做稳定错误码。

### Phase 4: Ledger-backed attribution

- 若 portfolio transaction ledger 已落地，接入 cashflow-aware return。
- 支持 realized / unrealized 的解释性拆分，但仍不承诺税务级口径。
- 把 performance report deep link 到 trade journal、signal calibration 和 company portrait。

## 验证方式

- Rust 单元测试：
  - 股票 market value / return / contribution 计算。
  - 基准收益和 excess return 计算。
  - 缺价格、缺成本、期权字段不完整、watchlist 排除时的 `limitations`。
  - `low_confidence` 不输出精确 attribution。
- Web/API 测试：
  - public 和 admin API 在无 portfolio、无 benchmark、价格失败、正常 report 四种状态下返回稳定 schema。
  - 前端模型测试覆盖数据不足、低置信度、正常贡献榜。
- 回归脚本：
  - 构造 actor portfolio fixture 和 mock price series，生成 30d report，断言 report 文件或 API response 包含 benchmark、confidence、limitations。
- 手工验收：
  - 用户端能看懂“估算收益 vs 基准”和“不足以计算”的区别。
  - IM 查询不会泄露完整持仓表，只返回摘要和限制。
  - 管理端能定位一个用户为何 performance report 是 low confidence。
- 指标：
  - 有可用 benchmark profile 的活跃 portfolio actor 占比。
  - performance report 生成成功率。
  - low-confidence limitation 分布。
  - 用户从 report deep link 进入 trade journal / company portrait / chat 的比例。

## 风险与取舍

- **风险：把估算误读成精确收益。**  
  取舍：所有 report 必须带 `valuation_basis`、`confidence`、`limitations`；低置信度时 UI 优先显示缺口，不突出百分比。

- **风险：短期业绩绑架长期投资主线。**  
  取舍：文案和 prompt 明确 performance 是复盘输入，不是买卖信号；短期跑输不能自动推翻长期 thesis。

- **风险：历史价格和公司行动处理复杂。**  
  取舍：P2 第一版只覆盖保守估算；split/dividend 等交给 corporate action reconciliation 和 transaction ledger 后续提升置信度。

- **风险：多基准选择带来认知负担。**  
  取舍：默认只推荐 2 到 3 个常见基准，并允许用户跳过；自定义多基准放到高级设置。

- **风险：与 exposure、ledger、calibration 提案边界混淆。**  
  取舍：本提案只回答“结果和贡献”；portfolio truth、风险暴露、判断是否被事实验证分别属于现有提案。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下所有现有提案，并重点比对了 portfolio、notification、research、calibration、billing、growth、runtime 和 desktop 相关主题。

- 不重复 `auto_p1_portfolio-transaction-ledger.md`：该提案解决交易流水、导入 preview 和 portfolio 当前态重建；本提案消费 snapshot / ledger，生成收益、基准和贡献复盘。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：exposure radar 说明组合风险结构和数据质量；performance attribution 说明某个时间窗口内真实或估算结果来自哪里。
- 不重复 `auto_p1_trade_discipline_journal.md`：trade journal 记录操作意图和纪律复盘；本提案只提供结果层证据，不能单独判定某次操作正确或错误。
- 不重复 `auto_p1_signal-outcome-calibration.md`：signal calibration 复盘判断是否被后续事实支持；performance attribution 复盘组合收益与基准差异。一个判断可以被事实支持但组合贡献很小，两个结论需要分开。
- 不重复 `auto_p2_brokerage-readonly-sync-gateway.md`：brokerage gateway 提供只读账户数据来源；本提案不接券商，且可以先用本地 portfolio 和 mock/historical prices 落地。
- 不重复 `auto_p1_corporate-action-reconciliation.md`：corporate action reconciliation 处理 split/dividend 对 portfolio truth 的调整；本提案只在缺少这些调整时标记 performance limitation。

查重结论：现有提案没有覆盖“actor 级 portfolio performance snapshot + benchmark profile + attribution report + 低置信度限制展示”的产品/架构层。因此本主题是新的、可落地的 P2 提案。
