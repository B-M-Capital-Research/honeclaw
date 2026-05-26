# Proposal: Brokerage Read-Only Sync Gateway for Verified Portfolio Truth

status: proposed
priority: P2
created_at: 2026-05-26 08:07:55 +0800
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
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p0_secrets-vault-rotation.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `bins/hone-mcp/src/main.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `skills/portfolio_management/SKILL.md`
- `config.example.yaml`

## 外部参考

- Plaid Investments API documents investment holdings, investment transactions, refresh, and update webhooks: <https://plaid.com/docs/investments/> and <https://plaid.com/docs/api/products/investments/>
- SnapTrade positions itself around brokerage account connectivity for balances, holdings, orders, and transactions: <https://docs.snaptrade.com/> and <https://snaptrade.com/>
- Model Context Protocol treats external data as resources and external capabilities as tools, which is a useful shape for isolating connector reads from agent actions: <https://modelcontextprotocol.io/docs/learn> and <https://modelcontextprotocol.info/docs/concepts/resources/>

## 背景与现状

Hone 的投资上下文目前主要来自用户显式录入、附件识别和 agent 工具写入：

- `memory/src/portfolio.rs` 以 `ActorIdentity` 为边界，把 portfolio 存成 `portfolio_{actor}.json`，`Holding` 记录 symbol、asset type、shares、avg cost、期权字段、horizon、strategy notes 和 `tracking_only`。
- `crates/hone-tools/src/portfolio_tool.rs` 暴露 `view/add/update/remove/watch/unwatch`，`add/update` 会覆盖或插入当前持仓，适合低门槛录入，但不是账户级同步。
- `crates/hone-web-api/src/routes/portfolio.rs` 和 public `/portfolio` 已经把 portfolio 作为用户端和管理端核心资产展示。
- 现有 `Portfolio Transaction Ledger` 提案已经指出当前态覆盖会丢失交易来源，并建议通过 import preview / apply / rebuild 把 CSV、截图、PDF、手工流水转成可确认 ledger。
- `Investment Document Inbox` 覆盖券商 statement / holding screenshot 等文档进入系统的路线，但仍是“用户上传材料”模式，不是持续连接。
- `mcp_bridge.rs` 和 `hone-mcp` 已经有把 Hone tools 暴露给 runner 的基础，说明未来外部数据源可以走受控 connector / resource / tool 边界，而不是让模型直接拿第三方凭证。
- `config.example.yaml` 和近期决策明确要求凭证归 `config.yaml` 或专门安全层管理，不能把 API key 从环境变量隐式读入 runtime。

这形成了一个清晰缺口：Hone 正在把 portfolio 提升为投资纪律、通知和研究记忆的共同输入，但 portfolio 仍主要依赖用户手动维护。对高意向用户来说，手动维护越久越容易和真实券商账户漂移；一旦 shares、成本、期权到期或 watchlist / holding 语义不准，后续 exposure radar、事件通知、公司画像和投资政策 verdict 都会被污染。

业界成熟方向不是让 agent 直接下单，而是通过只读 account connectivity 获取 holdings / transactions，再进入用户确认和审计层。Plaid 和 SnapTrade 这类服务已经把投资账户持仓、交易和更新事件产品化；MCP 方向也强化了“外部资源/工具必须有清晰边界”。Hone 可以借鉴这个方向，但必须保持自己的投资安全边界：只读、最小授权、先预览、可撤销、永不自动交易。

## 问题或机会

这是 P2，而不是 P1/P0：它价值明确，但依赖前置的数据治理和安全能力。没有 transaction ledger、permission broker、secret vault、user data trust center 和 entitlement 边界时，直接接券商 API 会把核心投资数据、隐私、成本和责任面一次性放大。

主要问题：

1. **真实账户与 Hone portfolio 容易漂移。** 用户通过 chat 或 Web 手工维护 shares / avg_cost，一段时间后很难保证和券商账户一致。漂移会影响通知优先级、组合暴露、期权到期提醒和研究上下文。

2. **手工上传无法覆盖持续变化。** 券商 CSV、截图和 statement 适合一次性导入，但无法自动发现新的买卖、分红、拆股、转仓、期权行权或账户断连。

3. **直接同步风险过高。** 投资账户数据敏感，第三方连接可能失败、延迟、字段缺失或提供错误成本。若同步结果直接覆盖 portfolio，会比手工误填更危险，因为用户会以为“这是系统验证过的真相”。

4. **多渠道体验缺少可信回流。** IM 里问“我的真实仓位是多少”时，Hone 只能读本地 portfolio，无法说明数据是否来自手工录入、导入流水、券商只读连接，还是已经断连 30 天。

5. **商业化机会尚未产品化。** 只读账户连接是高信任、高留存能力，但它也有外部服务成本、合规解释和用户信任门槛，应该和 usage entitlement、data export/delete、connector health 一起设计。

机会是新增 **Brokerage Read-Only Sync Gateway**：把券商连接当成一个受控、只读、可撤销的数据源，输出标准化 holdings / transactions snapshot，进入现有或未来 Portfolio Transaction Ledger 的 preview / reconcile 流程，而不是直接改 portfolio。

## 方案概述

新增 actor-scoped `BrokerageSyncGateway`，第一版只做只读数据连接和差异预览：

- `BrokerageConnection`：用户授权的只读连接，记录 provider、account ids、scope、status、last sync、断连原因和 token reference。
- `BrokerageSnapshot`：某次从 provider 拉到的 holdings / balances / transactions 原始快照 hash 和标准化投影。
- `BrokerageReconcileBatch`：把 snapshot 映射到 Hone portfolio / transaction ledger 的候选差异，等待用户确认。
- `BrokerageSecurity`：永不保存下单权限；凭证只存 vault reference；provider token 不进入 prompt、logs、support bundle 或 session transcript。
- `BrokerageConnector` trait：先支持 `mock` / `manual-fixture`，再接一个真实 provider；provider 差异被限制在 adapter 层。

关键原则：

- 只读连接不是 portfolio 真相源。portfolio 真相源仍应由用户确认后的 ledger / current snapshot 表达。
- 同步结果默认进入 preview，不直接覆盖 holdings。
- 任何 provider 数据都带 `as_of`、`provider`、`confidence`、`field_missing`、`normalized_by`，避免把外部字段包装成绝对事实。
- 不做交易下单，不暴露 order / trade tool，不保存 trading scope。
- Public Web / desktop / admin 必须能让用户断开连接、删除同步缓存、导出连接历史摘要。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“账户连接状态”：
  - `未连接`：解释可继续手工维护或上传 statement。
  - `已连接`：显示 provider、账户昵称、最近同步时间、同步是否进入待确认差异。
  - `需要处理`：例如断连、字段缺失、provider 延迟、发现未确认交易。
- 用户发起同步后看到的是差异预览：
  - 新增 / 减少 / 数量变化的持仓。
  - provider 识别到但 Hone 不支持或无法确认的交易。
  - 期权字段缺失、成本为空、symbol 无法映射、疑似拆股/分红/转仓。
- 用户确认后才生成 transaction ledger rows 或 current snapshot adjustment。
- 聊天里回答“我的仓位”时必须说明来源，例如“根据 2026-05-26 08:00 的只读券商快照，仍有 3 条差异未确认”。
- 用户可以一键暂停同步、断开连接、删除缓存，并保留已确认进入 ledger 的历史。

### 管理端

- `/users/:actor/portfolio` 增加 `Connections / Sync` tab：
  - connection status、last success、last failure、pending batches、provider field coverage、token/vault status。
  - 按 actor 查看哪些 portfolio 来自手工录入、文档导入、ledger rebuild、broker snapshot。
  - 对高风险差异给出审核原因，例如“provider quantity 与 local ledger 差异超过 20%”“option expiration missing”“security identity unresolved”。
- Settings 增加 connector capability 和 provider health，不在普通配置响应中泄露 token。
- Admin 可协助用户重跑 sync 或 mark provider issue，但不能替用户静默应用真实账户差异。

### 桌面端

- Desktop bundled 模式可以优先做本地 mock / CSV-to-connector fixture，给用户离线验证同步预览体验。
- 如果真实 provider OAuth 需要浏览器跳转，桌面打开受控 external auth flow，回调到 backend 后只保存 token reference。
- Remote backend 模式必须明确显示“券商数据会同步到远端 Hone backend”，并提供关闭/删除路径。

### 多渠道

- Feishu / Telegram / Discord / iMessage 私聊只展示摘要：
  - “券商快照发现 4 条差异，需到 Web/desktop 确认。”
  - “连接已断开 12 天，本次回答只使用 Hone 本地 portfolio。”
- 群聊默认不展示个人账户连接状态或 holdings 差异。
- 任何“同步后直接更新我的仓位”的 IM 请求都应返回确认引导，不在弱交互渠道批量应用。

## 技术方案

### 1. 类型与存储

建议在 `memory` 新增 `brokerage_sync` 模块，SQLite 存元数据，原始 provider payload 写入 actor-scoped encrypted / redacted artifact 目录或只保存 hash。

```text
brokerage_connections (
  connection_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  provider TEXT NOT NULL,
  provider_item_id_hash TEXT,
  status TEXT NOT NULL,
  scopes_json TEXT NOT NULL,
  token_ref TEXT NOT NULL,
  account_labels_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_sync_at TEXT,
  revoked_at TEXT,
  error_code TEXT,
  error_message TEXT
)

brokerage_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  connection_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  as_of TEXT NOT NULL,
  raw_payload_sha256 TEXT NOT NULL,
  normalized_json TEXT NOT NULL,
  field_warnings_json TEXT NOT NULL,
  created_at TEXT NOT NULL
)

brokerage_reconcile_batches (
  batch_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  connection_id TEXT NOT NULL,
  snapshot_id TEXT NOT NULL,
  status TEXT NOT NULL,
  diff_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  applied_at TEXT
)
```

### 2. Connector trait

在 `crates/hone-integrations` 或新 crate 中定义 provider adapter：

```rust
#[async_trait]
pub trait BrokerageConnector {
    async fn start_link(&self, actor: &ActorIdentity) -> HoneResult<LinkSession>;
    async fn exchange_callback(&self, callback: LinkCallback) -> HoneResult<TokenReference>;
    async fn fetch_snapshot(&self, connection: &BrokerageConnection) -> HoneResult<BrokerageSnapshot>;
    async fn revoke(&self, connection: &BrokerageConnection) -> HoneResult<()>;
}
```

第一阶段只实现 `mock` connector：

- 从 fixture JSON 读取 holdings / transactions。
- 覆盖断连、字段缺失、重复交易、symbol unresolved、provider delayed 等情况。
- 作为后续真实 provider 的 contract test。

真实 provider 接入时必须满足：

- 只请求 holdings / transactions / account metadata 等 read-only scope。
- token 只写 vault reference，不写 `config.yaml` 明文字段。
- 所有 provider webhook 只创建 snapshot / reconcile batch，不直接 apply。
- provider outage 或 rate limit 只影响 sync status，不阻塞本地 portfolio 页面。

### 3. Reconcile 到 portfolio ledger

本提案依赖 `auto_p1_portfolio-transaction-ledger.md` 的方向。若 ledger 尚未落地，第一版可以只生成只读 diff，不应用。

推荐映射：

- provider holdings -> `PositionCandidate`
- provider transactions -> `PortfolioTransactionCandidate`
- missing/unknown security -> `InstrumentResolutionNeeded`
- split/dividend/transfer -> `CorporateActionOrAdjustmentCandidate`
- provider-only account -> `AccountScopeCandidate`

应用策略：

- 逐行确认后写入 transaction ledger。
- 如果只有 holdings 无 transactions，则生成 `brokerage_snapshot_adjustment`，并标记解释力低于 transaction-backed ledger。
- 任何自动匹配都必须能展示 before / after / reason code。
- 同一 provider transaction id hash 去重，避免重复应用。

### 4. API 与前端

Admin API：

- `GET /api/brokerage/connections?actor=...`
- `POST /api/brokerage/connections/start-link`
- `POST /api/brokerage/connections/callback`
- `POST /api/brokerage/connections/:id/sync`
- `POST /api/brokerage/connections/:id/revoke`
- `GET /api/brokerage/reconcile-batches?actor=...`
- `GET /api/brokerage/reconcile-batches/:id`
- `POST /api/brokerage/reconcile-batches/:id/apply`
- `POST /api/brokerage/reconcile-batches/:id/dismiss`

Public API mirrors only current actor:

- `GET /api/public/brokerage/status`
- `POST /api/public/brokerage/start-link`
- `POST /api/public/brokerage/sync`
- `GET /api/public/brokerage/reconcile-batches`
- `POST /api/public/brokerage/reconcile-batches/:id/apply`
- `POST /api/public/brokerage/revoke`

Frontend落点：

- `packages/app/src/lib/brokerage-sync.ts`
- `packages/app/src/context/brokerage-sync.tsx`
- `packages/app/src/components/brokerage-sync-panel.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`

### 5. 安全、隐私与权限

- 接入 `Agent Permission Broker`：agent 读取 broker snapshot 或触发 sync 都要有 `ReadBrokerageData` permission event。
- 接入 `Secrets Vault`：provider token 用 `token_ref`，不放 prompt、session、logs、config diff。
- 接入 `User Data Trust Center`：用户能导出 connection metadata、snapshots summary、reconcile history，并能删除未应用 raw/snapshot cache。
- 接入 `Usage Entitlement Ledger`：真实 provider sync 有外部成本，按 connection / sync / historical range 计量。
- 接入 `Agent Mutation Ledger`：apply reconcile batch 产生 mutation record，支持撤销或反向 adjustment。

## 实施步骤

### Phase 1: Connector contract 和 mock sync

- 定义 `BrokerageConnector` trait、normalized snapshot schema 和 reconcile diff schema。
- 新增 `mock` connector fixture，覆盖 holdings、transactions、断连、字段缺失和重复交易。
- 增加只读 admin/public status API，先不做真实 OAuth。
- 在 portfolio 页面展示 mock connection 状态和只读 diff。

### Phase 2: Reconcile preview

- 生成 `BrokerageReconcileBatch`，把 provider snapshot 映射为 candidate rows。
- 若 Portfolio Transaction Ledger 尚未实现，只允许 dismiss / export，不允许 apply。
- 若 ledger 已实现，支持逐行 apply，生成 transaction candidates 和 mutation record。
- 增加重复 provider transaction id hash 的去重测试。

### Phase 3: 一个真实 provider 的 read-only adapter

- 在 vault / permission / data trust 基础就绪后接入一个 provider。
- 只启用 holdings / transactions / account metadata read-only scope。
- Webhook 或 refresh 只创建 snapshot + pending reconcile batch。
- provider errors 显示为 health 状态，不覆盖本地 portfolio。

### Phase 4: 多渠道和商业化包装

- IM 回复接入 connection staleness / pending diff 摘要。
- Public `/me` 或 `/portfolio` 展示 sync health 和 entitlement usage。
- Admin 增加 provider health / cost / failure rate dashboard。
- 文档和 onboarding 明确“只读同步，不提供交易执行”。

## 验证方式

- 单元测试：
  - normalized snapshot schema 能表达股票、ETF、期权、现金、未知 security、缺成本、缺交易日期。
  - provider transaction id hash 去重稳定，不泄露原始 id。
  - disconnected / rate-limited / stale connection 不会修改 portfolio。
  - raw payload token / account secrets 不进入 serialized public/admin DTO。
- API 测试：
  - public API 只能访问当前 session actor 的 connection 和 batch。
  - admin API actor filter 不越权返回其它 actor 的 raw payload。
  - revoke 后 sync 返回明确状态，旧 token_ref 不再可用。
- 前端测试：
  - portfolio 页面能展示未连接、已连接、断连、pending diff、field warning。
  - apply 按钮在 ledger 未启用时禁用，并显示前置依赖。
- 回归脚本：
  - `tests/regression/ci/test_brokerage_mock_sync.sh` 使用 mock fixture 完成 start-link -> sync -> list batch -> dismiss，全程无外部账号。
  - 真实 provider 联通性只放 `tests/regression/manual/`，不进入 CI。
- 手工验收：
  - 使用 mock fixture 创建 3 个差异，确认 IM 只返回摘要，Web/desktop 展示完整 diff。
  - 删除连接后，未应用 snapshot cache 可删除，已应用 ledger rows 保留来源引用。

## 风险与取舍

- **风险：投资账户连接带来高敏数据责任。** 取舍：P2 排期，必须等 vault、data trust、permission 和 ledger 基础成熟后再接真实 provider。
- **风险：provider 数据不准或延迟。** 取舍：同步结果永远先进入 preview，字段缺失和 `as_of` 必须可见，不能作为绝对真相直接覆盖 portfolio。
- **风险：用户误以为 Hone 可以交易。** 取舍：产品文案、scope、API 和 tool naming 全部使用 read-only / sync / reconcile，不出现 order / trade / execute。
- **风险：OAuth/provider 成本和地区覆盖不可控。** 取舍：先做 mock contract 和 adapter interface，真实 provider 只作为可插拔实现，手工导入仍是主路径。
- **风险：和 transaction ledger 重叠。** 取舍：本提案只负责连接、快照和 diff；ledger 负责被用户确认后的账户级真相和重建。
- **风险：多渠道泄露账户信息。** 取舍：IM 只发状态摘要和 batch id，完整明细只在 Web/desktop authenticated surface 展示。

## 与已有提案的差异

- 不重复 `auto_p1_portfolio-transaction-ledger.md`：该提案解决手工/附件/导入流水如何成为可确认 ledger；本提案解决只读券商连接如何产生 snapshot 和 reconcile batch，且明确不直接 apply。
- 不重复 `auto_p1_investment_document_inbox.md`：Document Inbox 管用户上传的 statement / screenshot / PDF；本提案管授权连接和持续 sync。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：Exposure Radar 消费可信 portfolio 生成风险视图；本提案提高 portfolio 输入的新鲜度和来源解释力。
- 不重复 `auto_p1_agent-permission-broker.md`：Permission Broker 裁决 agent 能否读取或触发同步；本提案定义 brokerage connector 数据域。
- 不重复 `auto_p0_secrets-vault-rotation.md`：Secret Vault 管 token 安全存储和轮换；本提案只保存 `token_ref` 并依赖 vault。
- 不重复 `auto_p1_user-data-trust-center.md`：Data Trust 管导出/删除/隐私清单；本提案提供需要纳入清单的 brokerage connection / snapshot / reconcile 数据。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：Usage Entitlement 计量外部成本；本提案产生可计量 sync event。

查重结论：现有提案已经覆盖 portfolio ledger、文档导入、组合暴露、权限、密钥和数据信任，但没有单独设计“只读券商连接 -> 标准化快照 -> 差异预览 -> 用户确认”的产品/架构层。本提案补的是高信任 portfolio 数据入口，且明确把交易执行、自动覆盖和无确认同步排除在边界外。
