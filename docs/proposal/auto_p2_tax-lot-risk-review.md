# Proposal: Tax-Lot Risk Review for Pre-Sell Discipline

status: proposed
priority: P2
created_at: 2026-05-31 02:04:58 +0800
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
- `docs/proposal/auto_p2_brokerage-readonly-sync-gateway.md`
- `docs/proposal/auto_p2_portfolio-performance-attribution.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_corporate-action-reconciliation.md`
- `memory/src/portfolio.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/public-portfolio-model.ts`
- `crates/hone-channels/src/response_finalizer.rs`
- `crates/hone-channels/src/runners/multi_agent.rs`

## 背景与现状

Hone 的产品定位已经从“聊天问答”走向“投资纪律工作台”：README 强调它要帮助用户跟踪持仓、抵抗情绪化交易，并通过 Web、桌面和 IM 多渠道进入日常研究流程。当前仓库里 portfolio 相关能力已经有清晰基础：

- `memory/src/portfolio.rs` 以 `ActorIdentity` 隔离 `Portfolio`，每个 `Holding` 记录 `symbol`、`asset_type`、`shares`、`avg_cost`、期权字段、`holding_horizon`、`strategy_notes`、`notes` 和 `tracking_only`。
- `crates/hone-web-api/src/routes/portfolio.rs` 提供 admin portfolio CRUD，但 summary 仍主要是持仓数、关注数、总股数和更新时间。
- Public `/portfolio` 目前重点展示投资主线和公司画像，帮助用户确认“我关注什么、长期 thesis 是什么”，但不展示卖出前的税务敏感信息。
- `crates/hone-channels/src/runners/multi_agent.rs` 已经在 prompt 中要求涉及本地 portfolio/watchlist 状态时优先使用本地状态工具，说明卖出、减仓、止损、换仓这类问题天然应绑定用户本地账户上下文。
- `crates/hone-channels/src/response_finalizer.rs` 会从 portfolio 工具结果恢复持仓确认文案，当前能解释 shares / avg_cost / notes，但不能解释成本批次或卖出后果。

现有提案已经补齐相邻层：

- `Portfolio Transaction Ledger` 设计了交易流水、import preview 和简化 `PortfolioLot`，但明确“不做税务申报级 lot accounting，不承诺 FIFO/LIFO、wash sale 或 realized P&L”。
- `Brokerage Read-Only Sync Gateway` 设计了券商快照差异预览，但不做税务计算。
- `Portfolio Performance Attribution` 设计组合收益和基准复盘，也明确不承诺税务报表或 wash sale 精度。
- `Trade Discipline Journal` 记录用户的操作意图、证据和复盘，但没有把税务敏感因素作为卖出前 checklist 的结构化输入。

因此当前缺口不是“马上做报税系统”，而是：当用户准备卖出、减仓、止损、换仓或清理亏损仓时，Hone 无法提醒这次动作可能涉及持有期、成本批次、亏损抵扣窗口、相近标的再买入、已实现收益口径和记录保留等问题。对一个定位为“纪律助手”的产品，这会直接影响用户信任。

## 问题或机会

这是 P2。它不是核心可用性 P0，也不应早于 transaction ledger 强行落地；但一旦 Hone 开始承接真实 portfolio、交易流水和高意向用户复盘，税务敏感卖出提醒会显著提升专业感、付费价值和风险边界。

主要问题：

1. **`holding_horizon` 只有意图标签，不是税务或成本批次事实。**  
   现在 `Holding` 可以标注 long_term / short_term，但没有 acquisition date、lot quantity、lot cost、broker account、import source、adjustment reason。模型容易把用户的长期策略标签误读成实际持有期。

2. **用户最需要纪律的时刻常常是卖出或换仓。**  
   买入前 Hone 可以要求 thesis、反证和仓位纪律；卖出前还应提醒“这笔卖出对应哪些 lot、是否会触发已实现损益、是否与计划内再买入冲突、是否需要先咨询税务专业人士”。没有结构化 review，回答只能停留在泛泛风险提示。

3. **交易流水和绩效复盘如果没有 tax-sensitive 标记，后续解释会失真。**  
   Performance attribution 可以估算收益，但用户关心的真实结果往往是税后、扣费、分批实现后的结果。即使 Hone 不计算税额，也应知道哪些 report 是 tax-blind，哪些卖出决策需要单独标记。

4. **多渠道更容易发生低上下文操作请求。**  
   用户在 Telegram/Feishu/Discord/iMessage 里说“帮我看看要不要卖掉一半”时，IM 不适合展示完整批次表；Web/desktop 更适合做 pre-sell review。缺少 review id，跨 surface 无法接力。

5. **安全和商业边界需要产品化，而不是只靠免责声明。**  
   如果 Hone 完全不碰税务敏感因素，专业投资工作台价值不完整；如果直接计算应纳税额，又会进入高风险法律/税务建议。更好的边界是做“税务敏感风险检查与记录”，不做“税务申报或个性化税务建议”。

机会是新增 **Tax-Lot Risk Review**：在用户表达卖出、减仓、换仓、清仓、止损、税损收割、滚动期权等意图时，Hone 生成一份保守的 pre-sell review，列出数据缺口、可能敏感的 lot、需要用户确认的问题和建议保留的记录；只有在有足够 ledger/lot 数据时才给出估算区间，并始终标注非税务建议。

## 方案概述

新增 actor-scoped 的 `TaxLotRiskReview` 派生层。它依赖 portfolio 当前态，优先复用未来 `PortfolioTransactionLedger` 和简化 `PortfolioLot`；在前置 ledger 未完成前，只允许生成 `insufficient_data` review，不伪造精确批次。

核心对象：

- `TaxProfileHint`：用户自选或 admin 配置的粗粒度提示，例如 jurisdiction、tax year start/end、默认 lot selection preference、是否显示税务敏感提醒。v1 不内置具体税率，也不替代税务顾问。
- `TaxLotCandidate`：由交易流水、券商只读快照或用户手工录入得到的批次候选，包含 symbol、asset_type、quantity_remaining、acquired_at、cost_basis、source、confidence、broker_account_alias。
- `PreSellIntent`：用户表达的卖出/减仓/换仓意图，包含 symbol、quantity 或 percentage、reason、target action、source_session_id、channel 和 created_at。
- `TaxLotRiskReview`：一次 review 的结果，包含 data_quality、selected_lots_preview、realized_gain_estimate_basis、holding_period_flags、replacement_purchase_flags、missing_fields、disclaimers、next_actions。
- `TaxReviewDecision`：用户对 review 的确认记录，例如 `viewed_only`、`decided_not_to_sell`、`will_consult_tax_pro`、`proceeded_outside_hone`、`recorded_manual_outcome`。它不表示 Hone 建议或批准交易。

第一版目标：

1. 在聊天和 Web/desktop 中识别卖出/减仓/换仓意图，生成 `PreSellIntent`。
2. 如果没有 transaction ledger / lot 数据，明确显示“无法判断税务批次，仅能提醒需要补齐哪些字段”。
3. 如果有 lot 候选，只做风险标记和解释性预览，不计算最终税额。
4. 把 review 与 trade discipline journal、portfolio transaction ledger、performance report 通过 id 关联。
5. 让 IM 只返回摘要和安全提醒，完整 lot 表和用户确认动作只在 Web/desktop/admin 中完成。

明确不做：

- 不生成税表，不计算应纳税额，不提供个性化税务建议。
- 不替用户选择 FIFO/LIFO/specific identification，也不默认推荐某个 lot 卖出。
- 不执行交易，不连接券商下单。
- 不把 tax profile 作为全球通用法律规则库；v1 只保留用户自填配置和保守 reason code。

## 用户体验变化

### 用户端

- 当用户在 `/chat` 说“我想卖掉一半 TSLA”“要不要税损收割这只”“把 NVDA 换成 QQQ”时，Hone 不直接给买卖结论，而是先生成 pre-sell review：
  - 当前 portfolio 是否有该 symbol。
  - 是否有可用交易流水或 lot 数据。
  - 本次卖出可能涉及哪些数据缺口。
  - 哪些问题应在交易前确认，例如 acquisition date、成本批次、近期相近买入、手续费、账户类型。
  - 明确提示“这不是税务建议，税务处理请咨询专业人士”。
- Public `/portfolio` 增加 “Pre-sell checks” 入口：
  - 对每个真实持仓显示 `tax data: missing / partial / reviewable`。
  - 有卖出意图时生成 review card，展示 lot 数据质量和下一步。
  - 用户可以标记“仅查看”“暂不卖出”“已咨询专业人士”“已在外部执行并记录结果”。
- 如果数据不足，界面优先引导补充 transaction ledger 或导入券商流水，而不是输出精确百分比。
- 如果用户通过 IM 发起，回复只给摘要和 deep link / short review id，不在群聊或公共 channel 展示完整持仓批次。

### 管理端

- Admin portfolio 详情增加 `Tax risk` 或 `Pre-sell reviews` tab：
  - 按 actor 查看最近 review、缺失字段、review 状态、关联 session / journal / transaction batch。
  - 标记高风险数据状态，例如 lot 缺 acquisition date、quantity 与 current holding 不一致、同一 symbol 多账户来源混杂。
  - 支持为用户补录粗粒度 `TaxProfileHint`，但必须写 audit log，并避免保存敏感税号等身份信息。
- 管理端可以查看 aggregate readiness：有真实持仓 actor 中，多少具备 lot 数据、多少只能做 insufficient-data review。

### 桌面端

- Desktop bundled 模式适合本地导入券商 CSV/XLSX 并生成 lot candidates；review 页面复用 Web API，但文件默认留在本机数据目录。
- Remote backend 模式必须明确提示文件会上传到远端，且 review 只保存必要结构化字段，不保存原始税务文件，除非用户显式纳入 artifact/data trust 管理。
- 菜单或 dashboard 可以显示“有 2 个卖出前检查数据不足”，帮助用户在真正交易前补齐信息。

### 多渠道

- 直聊 channel 可创建 pre-sell review 摘要，但完整 lot 表只通过 Web/desktop 查看。
- 群聊中识别卖出或税务敏感问题时，只给通用提醒并引导私聊，不读取或展示个人 portfolio。
- Cron / scheduled task 可以在用户设定复盘后提醒“上次卖出前 review 尚未记录最终结果”，但不能自动判断用户是否应该交易。

## 技术方案

### 1. 派生存储与数据边界

新增 `memory/src/tax_lot_review.rs` 或放在未来 portfolio ledger 模块下，保持 actor-scoped：

```text
TaxProfileHint {
  actor,
  jurisdiction_hint,
  tax_year_label,
  lot_selection_preference_hint,
  enabled,
  updated_at,
  updated_by,
}

TaxLotRiskReview {
  id,
  actor,
  symbol,
  asset_type,
  intent,
  data_quality,
  lot_candidates[],
  flags[],
  missing_fields[],
  limitations[],
  related_session_id,
  related_journal_entry_id,
  related_transaction_batch_id,
  created_at,
  status,
}
```

本地模式可先用 JSON 或 SQLite；cloud mode 后续跟随 PG durable store。它不是 portfolio truth source，只是 review/read model。

### 2. Lot candidate 来源

v1 来源按可信度排序：

1. Confirmed `PortfolioTransactionLedger` 和用户确认的 import rows。
2. Brokerage read-only snapshot 中带 acquisition/cost 字段的 lot 或 tax lot 导出。
3. 用户手工录入的 opening lots。
4. Legacy `Holding.avg_cost` 只能生成 `aggregate_cost_only`，不可拆成具体 lot。

当只有 legacy holding 时，review 必须是 `insufficient_lot_data`，但可以列出需要补的字段。

### 3. Risk reason code

不要把税法逻辑硬编码成黑箱建议。第一版只输出可解释 reason code：

- `missing_acquisition_date`
- `aggregate_cost_only`
- `partial_lot_coverage`
- `quantity_mismatch`
- `recent_replacement_purchase_unknown`
- `multiple_accounts_same_symbol`
- `option_contract_incomplete`
- `corporate_action_unreconciled`
- `tax_profile_missing`
- `not_tax_advice`

后续如果要支持地区规则，也应通过版本化 ruleset 和 legal review 接入，而不是散落在 prompt 里。

### 4. API 和权限

建议 API：

- `POST /api/portfolio/tax-reviews/preview`
- `GET /api/portfolio/tax-reviews?channel=...&user_id=...`
- `GET /api/portfolio/tax-reviews/{id}`
- `POST /api/portfolio/tax-reviews/{id}/decision`
- `PUT /api/portfolio/tax-profile`

Public API 只能访问当前 cookie actor；admin API 可指定 actor；群聊 channel 不允许创建带个人 lot 的 review。

### 5. Agent / skill 集成

- 在 portfolio / position-advice / scheduled-task 相关 skill 中增加 pre-sell review 调用建议：卖出、减仓、止损、换仓、税损、roll option 等意图先生成 review。
- `multi_agent` search stage 遇到卖出意图时优先读取本地 portfolio / review 状态；如果用户问具体税务规则且需要最新法条，应拒绝替代专业意见，或要求用户确认 jurisdiction 后仅做高层风险解释。
- `response_finalizer` 可以恢复 review 创建确认文案，例如“已创建卖出前检查，当前缺少 acquisition date 和 lot quantity，先不要把这个结果当作税务建议”。

### 6. 与现有提案的依赖关系

- 依赖 `Portfolio Transaction Ledger` 提供可靠 lot candidate；没有它时只做数据缺口提示。
- 可消费 `Brokerage Read-Only Sync Gateway` 的只读 tax lot 导出，但不把 provider 数据静默应用。
- 可关联 `Trade Discipline Journal`，让“为什么卖”与“卖出前税务敏感检查”在同一决策记录里。
- 可为 `Performance Attribution` 标注某个 realized result 是否 tax-blind。

## 实施步骤

### Phase 1: Insufficient-data review MVP

- 新增 review 类型和本地存储。
- 从 portfolio 当前态生成只读数据质量检查。
- API 支持创建 review preview，但没有 lot 时只返回 missing fields。
- Public `/portfolio` 和 admin portfolio detail 显示 tax data readiness。

### Phase 2: Ledger-backed lot candidates

- 接入 confirmed transaction ledger / import batch。
- 支持 opening lot 手工录入和 lot candidate preview。
- 生成 reason code、limitations 和 selected-lots preview。
- 增加前端 review card 和 decision 状态。

### Phase 3: Agent and channel workflow

- 对卖出、减仓、换仓、税损、期权 roll 等 intent 触发 review。
- IM 只返回摘要和 deep link；Web/desktop 展示完整 review。
- Trade discipline journal 可关联 review id。

### Phase 4: Cloud and commercial readiness

- Cloud mode PG store。
- Admin aggregate readiness dashboard。
- 与 entitlement/billing 结合：高级 plan 可开放更多 review 历史、导入批次和自定义规则提示。
- 建立人工法律/税务 review 入口，但不在产品内承诺自动税务建议。

## 验证方式

- 单元测试：
  - Legacy holding 只有 `avg_cost` 时返回 `aggregate_cost_only` 和 `missing_acquisition_date`。
  - transaction lots 数量不足时返回 `partial_lot_coverage`。
  - 多账户同 symbol 生成 `multiple_accounts_same_symbol`。
  - option holding 缺 strike / expiration / multiplier 时返回 `option_contract_incomplete`。
- API 测试：
  - Public 用户只能创建和查看自己的 review。
  - Admin 可按 actor 查看 review。
  - 群聊 actor 不返回个人 lot 明细。
  - decision 更新不修改 portfolio 当前态。
- 前端测试：
  - Public portfolio 覆盖 missing / partial / reviewable 三种 readiness。
  - Review card 在数据不足时优先显示缺口而不是收益数字。
- Agent 回归：
  - 用户问“卖掉一半 AAPL 会怎样”时先创建 pre-sell review 或解释缺少数据，不输出确定性税务建议。
  - 用户问普通公司研究时不触发 tax review。
- 手工验收：
  - 从 Web 创建 review，IM 能用 review id 查看摘要但不泄露 lot 表。
  - Desktop remote 模式上传文件前显示远端处理提示。
- 指标：
  - 有真实持仓 actor 中 tax readiness 分布。
  - pre-sell review 创建数、decision 完成率。
  - `insufficient_data` 占比随 transaction ledger adoption 下降。
  - 因 tax-sensitive 问题触发专业人士咨询/暂缓交易的用户自报数量。

## 风险与取舍

- **风险：用户误以为 Hone 在提供税务建议。**  
  取舍：所有文案使用“风险检查 / 数据缺口 / 记录提醒”，不使用“应纳税额”“最优税务方案”“建议卖出某 lot”等措辞；需要明确提示咨询税务专业人士。

- **风险：地区税法差异和更新频率高。**  
  取舍：v1 不内置税率或地区法条，只做 lot 数据质量和通用敏感点提示；后续 ruleset 必须版本化并经过人工 review。

- **风险：没有 ledger 时体验像空检查。**  
  取舍：这正是 P2 而非 P1；Phase 1 的价值是把数据缺口显性化，并引导用户导入 transaction ledger。

- **风险：存储更敏感的账户和交易数据。**  
  取舍：actor-scoped；不保存税号等身份信息；原始文件纳入 artifact/data trust 管理；群聊禁用个人 lot 展示。

- **风险：agent 可能把 review reason code 扩写成具体税务结论。**  
  取舍：prompt 和 finalizer 双层约束；测试覆盖“不得输出税额/不得推荐 lot selection”的回归样例。

- **风险：与 performance 或 trade journal 边界混淆。**  
  取舍：本提案只做卖出前税务敏感检查；performance 回答结果贡献，journal 记录决策理由，ledger 记录账户流水。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点比对了 portfolio、brokerage、performance、trade discipline、corporate action、cash/fx、options、safety 和 billing 相关主题。

- 不重复 `auto_p1_portfolio-transaction-ledger.md`：该提案解决交易流水、import preview 和 portfolio 当前态重建，并明确不做税务级 lot accounting；本提案在它之上定义“卖出前税务敏感风险检查”的只读 review 层。
- 不重复 `auto_p2_portfolio-performance-attribution.md`：performance 关注窗口收益、基准和贡献，本提案关注卖出/减仓动作发生前的 lot 数据质量、已实现损益敏感点和记录边界。
- 不重复 `auto_p1_trade_discipline_journal.md`：journal 记录用户为什么想交易、证据和复盘计划；本提案提供其中一个高价值 checklist：税务敏感字段和 lot readiness。
- 不重复 `auto_p2_brokerage-readonly-sync-gateway.md`：brokerage sync 只读拉取券商快照并做差异预览；本提案可以消费其中的 tax lot 导出，但不负责 provider 连接。
- 不重复 `auto_p1_corporate-action-reconciliation.md`：corporate action 处理拆股、分红等账户事件；本提案把未完成的公司行动对 lot/review 造成的风险显式标出。
- 不重复 `auto_p1_option_lifecycle_guard.md`：option guard 关注到期、roll、assignment 等生命周期提醒；本提案只在期权卖出/平仓/roll 前提示 tax-lot 数据完整性和记录边界。

差异结论：现有 proposal 已多次明确“暂不做税务级 lot accounting”，但尚未给出一个不越界、可落地、面向用户卖出前纪律场景的税务敏感 review 层。本提案填补的是“承认税务重要性，但只做数据质量、风险提示和决策记录”的产品架构边界。

## 文档同步说明

本轮只创建 proposal，不修改业务代码、测试代码、运行配置或长期架构决策；`docs/current-plan.md` 不更新，因为尚未开始执行该提案，也没有进入活跃任务跟踪。
