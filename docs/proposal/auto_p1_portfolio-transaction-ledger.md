# Proposal: Portfolio Transaction Ledger and Import Preview

status: proposed
priority: P1
created_at: 2026-05-16 20:03:30 CST
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
- `docs/proposal/auto_p1_corporate-action-reconciliation.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `skills/portfolio_management/SKILL.md`
- `skills/image_understanding/SKILL.md`
- `skills/position_advice/SKILL.md`

## 背景与现状

Hone 的 portfolio 已经是多条核心链路的共同输入：

- `memory/src/portfolio.rs` 以 `ActorIdentity` 为边界，把 `Portfolio` 存成 `portfolio_{actor}.json`，每个 `Holding` 记录 `symbol`、`asset_type`、`shares`、`avg_cost`、期权字段、持有期限、策略备注、`notes` 和 `tracking_only`。
- `PortfolioStorage::upsert_holding`、`promote_to_holding`、`remove_holding` 支持当前态 CRUD；`list_all` 通过扫描 JSON 文件给管理端列出有持仓的 actor。
- `crates/hone-tools/src/portfolio_tool.rs` 暴露 `view/add/update/remove/watch/unwatch`，支持批量 `holdings`，但 `add/update` 的语义仍是“覆盖或插入当前持仓”。
- `crates/hone-web-api/src/routes/portfolio.rs` 提供管理端 portfolio CRUD；summary 目前是 holdings 数、watchlist 数、总股数和更新时间。
- `packages/app/src/context/portfolio.tsx` 和 public `/portfolio` 读取当前态列表，支持编辑持仓和关注列表。
- `skills/portfolio_management/SKILL.md` 已把“真实持仓 vs watchlist”作为重要心智模型，并要求 ticker 验证；`skills/image_understanding/SKILL.md` 可以从截图识别持仓后引导用户调用 portfolio 工具。
- `crates/hone-channels/src/attachments/ingest.rs` 对表格附件已经有“优先调用 `portfolio_management` 提取结构化字段”的提示，说明用户上传券商表格或持仓截图是被预期的入口。

这些能力足以让用户“告诉 Hone 我现在持有什么”。但 portfolio 仍是一个可覆盖快照：系统不知道这 100 股来自哪几笔买入、是否已经部分卖出、成本价为何变化、用户上传的券商 CSV 里哪些行被应用、上一次 agent 或管理端修改前后差异是什么，也不能从一份交易流水重新构建当前持仓。

对投资纪律产品来说，这个缺口会越来越明显。当前态适合初版 onboarding，却不足以支撑长期可信的组合记忆、成本口径、绩效复盘、导入纠错和未来商业化。

## 问题或机会

当前缺口不是“portfolio 没有字段”，而是“portfolio 没有账户级变更历史和导入确认层”。

1. **当前态覆盖会丢失来源。**  
   `portfolio_tool` 的 `add/update` 会直接替换同一 `symbol + asset_type` 的持仓。后续如果用户问“为什么我的 NVDA 成本变成 820”或“这 20 股是我哪天买的”，系统只能看到最终 JSON。

2. **批量导入缺少 preview/apply 语义。**  
   用户上传券商持仓截图、CSV、XLSX 或交易记录时，agent 可以识别字段并调用 `portfolio`，但没有稳定的 import batch、行级状态、置信度、冲突项、跳过原因或可回滚记录。错误提取一旦写入，就和手动编辑混在一起。

3. **成本价和持仓变化不可重建。**  
   当前 `avg_cost` 是用户给出的单值，无法表达分批买入、部分卖出、期权开平仓、手续费、现金替代、红利再投资或多券商来源。后续 exposure radar 只能用这个单值估算，无法知道它是否可靠。

4. **现有提案需要更稳的 portfolio truth。**  
   `Portfolio Exposure Radar` 需要高质量持仓输入；`Corporate Action Reconciliation` 会生成调账 ledger；`Trade Discipline Journal` 记录操作意图和复盘；`Investment Document Inbox` 能接收券商材料。但这些都没有定义“实际账户流水如何进入并重建 portfolio 当前态”。

5. **多渠道录入体验缺少安全刹车。**  
   IM 私聊里发一张截图很方便，但直接写当前态风险较高。Web/desktop 更适合做行级确认。没有 import preview，Hone 很难把“轻量上传”和“严肃确认”分层。

6. **商业化和信任叙事受限。**  
   对高意向投资用户来说，可信 portfolio 不是 ticker 清单，而是“我能导入、核对、追溯、导出、纠错”。这是从玩具聊天走向个人投资工作台的基础体验。

这是 P1。它不如 P0 输出安全那样直接防止危险回答，但会显著提升核心数据可信度、长期留存、后续组合分析质量和支持排障效率。第一版可以完全本地、手工确认、无券商 API，不涉及交易执行。

## 方案概述

新增 actor-scoped 的 **Portfolio Transaction Ledger**，让 portfolio 当前态从“唯一真相源”调整为“可由交易流水和确认导入派生的当前视图”。第一版仍保留 `PortfolioStorage` JSON，避免大迁移；新增 ledger 和 import preview 作为并行层，逐步成为更可信的 portfolio truth。

核心对象：

- `PortfolioTransaction`：用户确认过的一笔账户级变动，例如 buy、sell、short_sell、cover、option_open、option_close、cash_dividend、fee、transfer_in、transfer_out、manual_adjustment。
- `PortfolioLot`：可选的持仓批次视图，第一版可以只做简化 lot，用于解释成本来源；不必一次性实现完整税务 lot。
- `PortfolioImportBatch`：一次从截图、CSV、XLSX、PDF 或手工粘贴中解析出的导入批次。
- `PortfolioImportRow`：导入批次里的行级候选，包含原始字段、标准化交易、置信度、冲突、状态和错误原因。
- `PortfolioSnapshot`：由 ledger 重建出的当前持仓快照，与现有 `Portfolio` JSON 对比，用于验证和渐进迁移。
- `PortfolioRebuildReport`：解释“ledger 当前态”和“legacy JSON 当前态”的差异。

第一版目标：

1. 支持从导入材料生成 preview，不直接写 portfolio。
2. 用户确认行级 apply 后写 transaction ledger。
3. 从 ledger 计算当前 holdings，并与旧 `PortfolioStorage` 双写或对比。
4. 保留手动编辑路径，但把它记录成 `manual_adjustment` transaction。
5. 给 Web/desktop/admin 一个“最近导入、待确认、差异、重建结果”的产品面。

明确不做：

- 不接券商 OAuth 或自动同步。
- 不做税务申报级 lot accounting。
- 不做交易下单。
- 不把 trade discipline journal 变成实际成交记录；用户仍需明确确认 transaction。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加 `导入 / 流水` 入口：
  - 上传 CSV/XLSX/PDF/截图或粘贴交易流水。
  - Hone 生成 preview：识别到的 ticker、方向、数量、价格、日期、费用、账户来源和置信度。
  - 用户逐行确认、跳过、编辑或标记为 watchlist。
  - 应用后展示“已从 N 行流水重建当前持仓”，并提示仍需核对券商账户。
- portfolio 页面保留当前持仓视图，但每个持仓旁增加来源摘要：
  - `由 3 笔交易组成`
  - `手动调整`
  - `与 legacy snapshot 不一致`
  - `成本缺少交易日期`
- 用户从聊天里发截图时，回复不再默认直接写入持仓，而是：
  - “已解析出 5 行候选，已创建导入预览。”
  - 提供短 batch id，引导到 Web/desktop 确认。
  - 对低风险 watchlist 可以仍允许快速确认，但真实持仓默认需要 preview。
- 用户问“我的 AAPL 成本怎么来的”，agent 可以读取 transaction ledger，解释分批来源和最近一次调整，而不是只复述 `avg_cost`。

### 管理端

- 在 actor portfolio 详情中增加 `Transactions / Imports` tab：
  - 最近 import batches、待确认行、失败原因、应用人、应用时间。
  - ledger 重建出的 holdings 与 legacy JSON holdings 的 diff。
  - 每个持仓的 transaction trail。
- 管理员可以帮用户回滚某个 import batch：生成反向 `manual_adjustment` 或把 batch 标记为 reversed，而不是直接删除历史。
- 支持筛选高风险导入：
  - 解析置信度低。
  - 同一 ticker 数量与现有持仓差异过大。
  - 缺交易日期或价格。
  - 期权合约字段不完整。
  - CSV 与截图重复导入疑似冲突。

### 桌面端

- Desktop bundled 复用同一导入预览页面，本地文件处理更自然：拖入券商导出的 CSV/XLSX，确认后写本机 ledger。
- Remote backend 模式清楚显示“文件将上传到远端服务端处理”，并沿用 document inbox / data trust 的删除和导出边界。
- Dashboard 可显示“2 个导入批次待确认 / 1 个持仓与流水重建不一致”。

### 多渠道

- Feishu / Telegram / Discord 私聊中，附件解析只创建 import batch，不在群聊或低确认场景直接覆盖真实持仓。
- 用户可说“应用刚才那份导入里的 AAPL 和 MSFT，跳过期权行”，agent 调用受控工具更新行状态。
- 群聊中如果出现持仓截图或交易记录，默认只做隐私提醒并引导私聊，不创建个人 transaction ledger。
- IM 回复只展示摘要和 batch id；完整行级数据引导回 Web/desktop，避免泄露敏感账户明细。

## 技术方案

### 1. 新增 ledger 存储

建议在 `memory` 新增 `portfolio_ledger` 模块，使用 SQLite 存结构化交易与导入批次。现有 `PortfolioStorage` JSON 保留，第一阶段作为 legacy current snapshot。

建议表：

```text
portfolio_transactions (
  transaction_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  symbol TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  action_kind TEXT NOT NULL,
  quantity REAL NOT NULL,
  price REAL,
  fee REAL,
  currency TEXT,
  trade_date TEXT,
  settle_date TEXT,
  underlying TEXT,
  option_type TEXT,
  strike_price REAL,
  expiration_date TEXT,
  contract_multiplier REAL,
  source_kind TEXT NOT NULL,
  source_ref TEXT,
  import_batch_id TEXT,
  import_row_id TEXT,
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  reversed_by TEXT
)

portfolio_import_batches (
  batch_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_channel TEXT NOT NULL,
  source_session_id TEXT,
  source_document_id TEXT,
  source_filename TEXT,
  parser_version TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

portfolio_import_rows (
  row_id TEXT PRIMARY KEY,
  batch_id TEXT NOT NULL,
  row_index INTEGER NOT NULL,
  raw_json TEXT NOT NULL,
  normalized_json TEXT,
  confidence REAL NOT NULL,
  status TEXT NOT NULL,
  conflict_json TEXT,
  error_message TEXT,
  applied_transaction_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

`status` 建议包含：

- batch：`preview`、`partially_applied`、`applied`、`cancelled`、`reversed`、`failed`
- row：`candidate`、`needs_edit`、`applied`、`skipped`、`duplicate`、`rejected`

### 2. Ledger 到当前持仓的重建规则

第一版实现简化但明确的确定性规则：

- buy / transfer_in：增加数量；成本按 weighted average 更新。
- sell / transfer_out：减少数量；不计算已实现收益税务口径，只保留 transaction。
- manual_adjustment：直接设定目标数量 / 成本，但必须记录 before snapshot 和原因。
- option_open / option_close：按 `asset_type=option` 和完整合约 key 归并；缺 expiration/strike/type 时不允许 apply。
- cash_dividend / fee：先进入 ledger，不改变 `shares`；是否影响现金余额放到后续版本。

重建输出：

```rust
pub struct PortfolioRebuildReport {
    pub actor: ActorIdentity,
    pub generated_at: DateTime<Utc>,
    pub holdings: Vec<RebuiltHolding>,
    pub warnings: Vec<PortfolioLedgerWarning>,
    pub legacy_diff: Vec<PortfolioHoldingDiff>,
}
```

如果 ledger 为空，继续读取 legacy JSON。若 ledger 存在但 legacy JSON 不一致，第一版不自动覆盖；UI 显示 diff，让用户或 admin 执行 reconcile。

### 3. 导入解析与 preview

导入路径应复用 `Investment Document Inbox` 的文档登记能力；在该提案未落地前，也可以在 portfolio route 内先接一个最小 batch API。

解析来源：

- CSV/XLSX：读取 header，支持常见列名映射：symbol/ticker、action、buy/sell、quantity/shares/contracts、price/cost、date、fee、account。
- 截图：继续由 image understanding / multimodal runner 提取候选，但只写入 import rows，不直接写 transactions。
- PDF statement：先存 preview；复杂 PDF 第一版标记 `needs_edit`，不做盲写。
- 手工粘贴：允许用户粘贴表格文本生成 rows。

冲突检查：

- 同一 batch 内重复行。
- 与已应用 transaction 在 `source_hash`、date、symbol、quantity、price 上疑似重复。
- 应用后数量为负。
- 期权合约 key 不完整。
- `tracking_only` watchlist 与真实 transaction 冲突。

### 4. API 与工具

新增后端 API：

- `POST /api/portfolio/imports/preview`
- `GET /api/portfolio/imports?channel=&user_id=&channel_scope=`
- `GET /api/portfolio/imports/:batch_id`
- `PATCH /api/portfolio/imports/:batch_id/rows/:row_id`
- `POST /api/portfolio/imports/:batch_id/apply`
- `POST /api/portfolio/imports/:batch_id/cancel`
- `GET /api/portfolio/transactions?channel=&user_id=&channel_scope=&symbol=`
- `GET /api/portfolio/rebuild?channel=&user_id=&channel_scope=`

Public 端提供同构但只允许当前登录 actor：

- `POST /api/public/portfolio/imports/preview`
- `GET /api/public/portfolio/imports`
- `POST /api/public/portfolio/imports/:batch_id/apply`
- `GET /api/public/portfolio/transactions`

工具层可以先扩展现有 `portfolio` 工具，避免 skill 入口碎片化：

- `portfolio(action="preview_import", source_ref=..., rows=[...])`
- `portfolio(action="list_imports")`
- `portfolio(action="apply_import", batch_id=..., row_ids=[...])`
- `portfolio(action="list_transactions", ticker=...)`
- `portfolio(action="rebuild_snapshot")`

写入真实持仓时，工具内部执行：

1. apply rows -> 写 `portfolio_transactions`
2. rebuild current holdings
3. 可选双写 `PortfolioStorage::save`
4. 返回 diff 和 warnings

### 5. 兼容与迁移

- 不删除 `memory/src/portfolio.rs`；它继续服务现有 API、event-engine subscription 和 UI。
- 新建 ledger 后，手动 `add/update/remove/watch/unwatch` 继续可用，但真实持仓变更应同时写 `manual_adjustment` transaction。
- 旧 JSON 首次进入 ledger 页面时，可生成一个 `opening_balance` import batch：把当前 holdings 作为期初余额候选，用户确认后 ledger 才开始成为解释来源。
- watchlist 仍保存在 portfolio 当前态；第一版不把 watchlist 强行变成 transaction。
- 后续 `Corporate Action Reconciliation` 可把 split/dividend apply 写成 `corporate_action_adjustment` transaction，而不是另起一套不可合并的调账历史。
- 后续 `Agent Mutation Ledger` 可记录“谁应用了 batch”，但 transaction ledger 自身仍保留金融语义。

## 实施步骤

### Phase 1: Ledger and rebuild core

- 在 `memory` 新增 `portfolio_ledger` 类型、SQLite 表、CRUD 和单元测试。
- 实现 buy/sell/manual_adjustment/opening_balance 的 weighted-average rebuild。
- 增加 legacy JSON diff 报告。
- 手动 portfolio 写入路径先双写 `manual_adjustment`，但保持旧行为不变。

### Phase 2: Import preview API

- 新增 admin/public import preview/list/detail/apply/cancel API。
- 支持手工 rows 和 CSV/XLSX header mapping；截图/PDF 先通过 agent 生成 rows。
- 行级状态支持 edit/skip/apply。
- apply 前校验 actor、重复行、负数量、期权 key、legacy diff。

### Phase 3: Product surfaces

- Public `/portfolio` 增加导入入口、待确认批次、transaction trail 和重建 diff。
- Admin actor portfolio 增加 imports/transactions tab。
- Desktop 复用同一页面，补充本地文件导入说明。
- IM 附件解析改为创建 batch preview，不直接覆盖真实持仓。

### Phase 4: Ledger as preferred portfolio truth

- 在配置或能力协商中标出 `portfolio_ledger_enabled`。
- Event-engine subscription、exposure radar 和 position advice 优先读取 ledger rebuild snapshot；无 ledger 时回退 legacy JSON。
- 将 corporate-action adjustment、document inbox broker statement、trade journal executed_by_user 后续衔接到同一 ledger。

## 验证方式

- Rust unit tests：
  - opening balance + buy + sell 后重建数量和 weighted avg cost 正确。
  - sell 超过当前数量被拒绝或生成 warning，不产生负 holdings。
  - option transaction 缺 expiration/strike/type 时不能 apply。
  - manual adjustment 保存 before/after，并能解释 legacy diff。
  - 同一 import row 不能重复 apply。
- API tests：
  - public route 只能访问当前 web actor 的 batches/transactions。
  - admin route 必须显式 actor 查询。
  - apply batch 后 transactions、rebuild snapshot、legacy portfolio 双写结果一致。
  - cancel batch 不修改 portfolio。
- Frontend tests：
  - import rows 排序稳定，`candidate/needs_edit/applied/skipped` 状态显示正确。
  - rebuild diff 能区分 quantity diff、avg_cost diff、missing in legacy、missing in ledger。
  - 空 ledger、只有 opening balance、部分应用、应用失败都有明确空态或错误态。
- 手工验收：
  - 上传一份小型 CSV，确认两行、跳过一行，portfolio 当前态符合预期。
  - 在 IM 发送持仓截图，只创建 preview，不直接覆盖持仓。
  - 用户问“成本怎么来的”，agent 能引用 transaction trail。
- 指标：
  - import preview -> applied 转化率。
  - 被跳过或编辑的行比例。
  - legacy diff 数量随时间下降。
  - 因 portfolio 数据错误导致的支持/反馈减少。

## 风险与取舍

- 风险：ledger 与 legacy JSON 双写期间出现不一致。  
  取舍：第一版显式展示 rebuild diff，不静默切换 truth source；只有确认过 opening balance 或 import apply 后才让 ledger 成为解释来源。

- 风险：用户误以为 Hone 已连接券商实时账户。  
  取舍：所有导入文案都标注“手工导入 / 用户确认 / 请与券商账户核对”，不使用“同步”一词，除非未来真的接 broker API。

- 风险：交易流水会增加敏感数据面。  
  取舍：严格 actor-scoped；public 只读当前 cookie actor；群聊不创建个人 ledger；导出和删除应与 data trust center 对齐。

- 风险：完整 lot accounting 复杂度很高。  
  取舍：P1 第一版只做 weighted average 和解释型 transaction trail，不承诺税务、FIFO/LIFO、wash sale 或 realized P&L。

- 风险：agent 从截图提取错误并误写。  
  取舍：截图/PDF 只进入 preview，真实 apply 需要用户或 admin 明确确认；低置信度行默认 `needs_edit`。

- 风险：与 existing proposals 重叠。  
  取舍：本提案只定义“实际持仓如何由确认交易和导入流水重建”，不做文档收件箱、公司行动规则、交易纪律审查或组合暴露分析。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案。结论：本提案不重复，重点差异如下：

- 不重复 `auto_p1_investment_document_inbox.md`：document inbox 解决用户上传材料的统一登记、解析和复盘交接；本提案解决从券商材料或手工 rows 生成可确认交易流水，并由流水重建 portfolio 当前态。
- 不重复 `auto_p1_investment_context_intake.md`：context intake 解决新用户补 portfolio/profile/prefs/task 缺口；本提案假设用户已有或正在导入真实账户变动，关注长期可追溯的 transaction truth。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：exposure radar 是只读派生风险视图；本提案是 portfolio 数据底座，提供更可靠的数量、成本和来源。
- 不重复 `auto_p1_corporate-action-reconciliation.md`：corporate-action reconciliation 处理 split/dividend 等市场事件导致的调账候选；本提案处理用户账户交易、导入批次和普通手工调整。后续公司行动 apply 可以写入本 ledger。
- 不重复 `auto_p1_trade_discipline_journal.md`：trade journal 记录用户操作前的理由、纪律检查和复盘；本提案记录用户确认的账户级事实变动，不评价交易理由。
- 不重复 `auto_p1_agent-mutation-ledger.md`：mutation ledger 是通用状态变更审计；本提案有金融领域语义，包括 buy/sell、数量、成本、期权 key、导入行和 portfolio rebuild。
- 不重复 `auto_p1_user-data-trust-center.md`：data trust center 处理导出、删除、隐私控制；本提案会产生敏感数据，需要与其对齐，但不替代用户数据权利产品面。

差异结论：现有提案已经覆盖“怎样收材料”“怎样看组合风险”“怎样处理公司行动”“怎样记录投资决策理由”。尚未覆盖的是 portfolio 最底层的账户流水、导入确认和可重建当前态。这个主题直接提升 Hone 作为长期投资工作台的可信度，是 P1 级底座提案。

## 文档同步说明

本轮只新增 proposal，不开始实施，不修改业务代码、测试、模块边界、运行流程或长期规则，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续开始实施，应按动态计划准入标准新增或复用 `docs/current-plans/portfolio-transaction-ledger.md`，并在落地 API、存储、工具或 event-engine 读取路径时同步更新 `docs/repo-map.md` 与必要的决策记录。
