# Proposal: Instrument Identity Registry for Portfolio, Events, and Research Memory

status: proposed
priority: P1
created_at: 2026-05-20 08:03:24 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/types.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/pollers/price.rs`
- `crates/hone-event-engine/src/pollers/analyst_grade.rs`
- `crates/hone-event-engine/src/pollers/corp_action.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `packages/app/src/context/symbol-drawer.tsx`
- `packages/app/src/components/symbol-drawer.tsx`
- `packages/app/src/lib/mainline-context-model.ts`
- `packages/app/src/pages/public-portfolio.tsx`

## 背景与现状

Honeclaw 已经不是单一聊天入口，而是一个跨 Web、桌面、Feishu、Telegram、Discord、iMessage 的投资研究工作台。当前架构已经围绕 `ActorIdentity` 隔离用户数据，围绕 `SessionIdentity` 恢复会话上下文，围绕 actor sandbox 保存公司画像，并通过 event-engine 把持仓、关注列表、公司事件、价格异动、SEC filing、analyst grade、news、macro/social 事件串成主动提醒和 digest。

从代码看，系统里已有多个“证券对象”入口，但它们主要靠字符串 ticker 临时对齐：

- `memory/src/portfolio.rs` 的 `Holding` 使用 `symbol` 作为持仓和 watchlist 的主键之一；旧字段 `ticker` 只是 serde alias。
- `PortfolioTool` 的参数也以 `ticker` 为模型可见输入，`watch/add/update/remove` 直接写入 symbol 字符串。
- `crates/hone-event-engine/src/event.rs` 的 `MarketEvent.symbols: Vec<String>` 是事件和订阅匹配的主要连接字段。
- `crates/hone-event-engine/src/subscription.rs` 从 portfolio 扫描 watch pool，把 holdings 的 `symbol` 转大写后参与事件订阅。
- public/admin digest 视图通过 `global_digest::extract_tickers(&md)` 从 `profile.md` 文本中提取 ticker，再按 `ticker` 查画像。
- `packages/app/src/context/symbol-drawer.tsx` 只做 `trim -> uppercase -> remove $` 的轻量标准化；`components/symbol-drawer.tsx` 通过标题 `includes(sym)` 查公司画像和研究任务。
- `memory/src/company_profile/types.rs` 的 `ProfileMetadata` 已有 `company_name`、`stock_code`、`aliases`，但这些元数据还没有成为跨 portfolio、event-engine、company profile、research task、UI 的统一身份层。

这说明 Hone 已经有足够多的产品面依赖同一个概念：用户说的公司、持仓里的 symbol、事件源返回的 ticker、公司画像里的 stock code、研究任务里的 company name、UI URL 里的 `?symbol=`，应当能被稳定地解析成同一个“投资标的身份”。现在这个身份还不是一等对象。

## 问题或机会

### 1. ticker 字符串不足以承载长期投资工作台的信任要求

Ticker 不是稳定业务实体。它可能存在交易所歧义、ADR/本股映射、点号/连字符差异、退市/换代码、基金/期权/加密资产等资产类型差异，也可能和公司画像标题、公司简称、中文名、alias 不一致。当前把大写 ticker 当作通用连接键，短期能跑通 MVP，但随着 portfolio、event-engine、company portraits、public portfolio 和增长分享能力变多，会放大错配风险。

典型失败模式：

- 用户录入 `BRK.B`，事件源返回 `BRK-B`，画像写成 `Berkshire Hathaway`，UI 无法稳定连起来。
- 用户持有 ADR，但公司画像按境外本股或母公司名称建档，digest 判断“缺画像”。
- 公司改名或换 ticker 后，旧画像、旧交易流水、旧提醒订阅分裂成两套资产。
- 期权 `underlying` 和期权合约 `symbol` 混用，event-engine 可能按合约代码订阅价格事件，而不是按 underlying 订阅公司事件。
- `SymbolDrawer` 用标题 contains 匹配画像，可能把相似 ticker 或公司名误连。

### 2. 现有多个提案都需要一个更稳的标的身份底座

已有提案已经覆盖 portfolio transaction ledger、corporate action reconciliation、portfolio exposure radar、company portrait health、cross-company thesis map、source provenance freshness、investment context intake 等方向。它们都假设 `symbol/ticker/company` 能被可靠关联，但没有单独提出一个轻量、兼容、可逐步启用的 Instrument Identity Registry。

如果继续在每个提案里各自实现 symbol normalization，会出现新的隐性耦合：同一个标的在 portfolio 是 `symbol`，在 portrait 是 `stock_code`，在 event 是 `symbols[]`，在 research task 是 `company_name`，在 UI 是 URL query。后续越多增长和商业化入口依赖这些数据，越难解释“为什么这条提醒没命中我的持仓”或“为什么这个画像没有出现在我的 portfolio”。

### 3. 产品上可以把“建档/录入/订阅/复盘”变成更可信的闭环

投资助手的核心体验不是回答一次问题，而是长期维护用户的投资上下文。标的身份一旦稳定，Hone 可以在用户输入 `Apple`、`AAPL`、`苹果`、`NASDAQ:AAPL`、期权合约、截图 OCR、研究报告标题时给出明确确认：

- “识别为 Apple Inc. / NASDAQ:AAPL / USD / 股票，是否加入 watchlist？”
- “这份画像的 stock code 为空，但可能对应 MSFT。确认后会绑定到现有持仓。”
- “这条 corporate action 是 ticker change，是否把旧 symbol 的画像和历史事件迁移到新 symbol？”

这会直接提升用户端信任、管理端排障效率、event-engine 命中准确性和后续商业化体验。

## 方案概述

新增一个轻量的 `Instrument Identity Registry`，第一版不替换现有 portfolio JSON、company profile Markdown 或 event-engine schema，而是在其旁边建立可查询、可解释、可回滚的标的身份层。

核心目标：

1. 给每个投资标的分配稳定 `instrument_id`，例如 `inst_us_equity_aapl` 或 UUID/ULID。
2. 保留用户熟悉的 ticker 展示，但内部跨模块关联优先使用 `instrument_id + canonical_symbol + aliases`。
3. 提供统一解析 API：输入 symbol/company/alias/URL/OCR hint，返回候选 instrument、置信度、冲突原因和需要用户确认的字段。
4. 让 portfolio、event-engine、company profile、research task、SymbolDrawer 和 public portfolio 逐步读取同一 identity projection，而不是各自猜。
5. 支持兼容迁移：旧数据继续可读，缺 identity 的记录在读取或用户确认时逐步补齐。

第一版建议定位为 P1 基建，不做大型金融 master data 平台，不引入付费数据源强依赖，也不阻塞当前事件引擎运行。

## 用户体验变化

### 用户端 Web / Public Portfolio

- 添加持仓或 watchlist 时，输入框从纯 ticker 变成“标的搜索/解析”：用户输入 `AAPL`、`Apple` 或 `苹果` 后，显示候选卡片：公司名、交易所、canonical symbol、资产类型、币种、数据源和置信度。
- 若解析结果唯一且低风险，可以自动接受；若存在多市场、多资产类型、ADR、本股/母公司歧义，则要求用户确认。
- `/portfolio` 中每个 ticker 的 mainline、画像、digest context 使用同一个 instrument 连接，避免“持仓有 AAPL，但画像被识别为 Apple Inc. 所以不显示”的断裂。
- 当画像缺 stock code 或 alias 时，显示“可绑定到现有标的”的修复入口，引导用户通过 chat 或确认动作补齐。

### 管理端

- 新增 Instrument 列表/详情只读页面或 tab：按 actor/workspace 看到 portfolio holdings、company portraits、mainline entries、event subscriptions、research tasks 对同一个 instrument 的关联状态。
- 在用户详情页显示 identity health：未解析 symbol、低置信度绑定、一个 symbol 对多个画像、一个画像被多个 symbol 引用、期权 underlying 缺失、事件源 symbol 无法映射等。
- event-engine 管理页可以按 instrument 而不只是 raw ticker 过滤，排查“该 actor 为什么收到/没收到某事件”。

### 桌面端

- Desktop bundled 模式复用同一 Web UI 和 backend API，不需要单独实现。
- 在 channel/settings 或 doctor/status 中显示 identity registry 是否可读、索引是否需要重建、最近解析失败数。
- 本地离线用户仍可使用内置启发式解析；外部数据源不可用时只降低置信度，不阻断手动确认。

### 多渠道接入

- IM 中用户发“关注苹果”或“把 BRK.B 加入 watchlist”时，agent 可以先调用解析能力；高风险歧义时返回一条短确认，而不是直接写 portfolio。
- 群聊默认不把个人 actor 的 identity 绑定暴露给群成员；只有触发用户或明确 group actor 才能确认写入。
- 通知文案使用用户确认过的 display name 和 canonical symbol，减少不同渠道同一标的显示不一致。

## 技术方案

### 1. 新增数据模型

建议在 `memory` 中新增 `instrument_identity.rs` 或子模块，初期可以使用 SQLite，也可以先用 actor-scoped JSON。考虑到查询和冲突检测，SQLite 更合适：

```sql
CREATE TABLE instruments (
  instrument_id TEXT PRIMARY KEY,
  asset_type TEXT NOT NULL,
  canonical_symbol TEXT NOT NULL,
  exchange TEXT,
  mic TEXT,
  currency TEXT,
  country TEXT,
  company_name TEXT,
  display_name TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE instrument_aliases (
  instrument_id TEXT NOT NULL,
  alias TEXT NOT NULL,
  alias_kind TEXT NOT NULL,
  source TEXT NOT NULL,
  confidence REAL NOT NULL DEFAULT 1.0,
  created_at TEXT NOT NULL,
  PRIMARY KEY (instrument_id, alias, alias_kind)
);

CREATE TABLE actor_instrument_bindings (
  actor_key TEXT NOT NULL,
  raw_value TEXT NOT NULL,
  instrument_id TEXT NOT NULL,
  binding_source TEXT NOT NULL,
  confidence REAL NOT NULL,
  confirmed_by_user INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (actor_key, raw_value)
);
```

第一版字段保持小而稳：

- `instrument_id`: 内部稳定 id，不要求用户可见。
- `canonical_symbol`: 当前展示和 event source 使用的主 symbol。
- `asset_type`: `stock`、`option`、`fund`、`crypto` 等，先支持现有 portfolio 里的 `stock/option`。
- `aliases`: ticker 变体、公司英文名、中文名、旧 ticker、profile title、OCR/导入别名。
- `actor_instrument_bindings`: 记录某个 actor 曾确认 raw 输入到 instrument 的映射，避免全局别名误伤个人上下文。

### 2. 统一解析服务

新增纯逻辑解析层，例如：

```rust
pub struct InstrumentResolveInput {
    pub actor: Option<ActorIdentity>,
    pub raw: String,
    pub asset_type_hint: Option<String>,
    pub context: ResolveContext,
}

pub struct InstrumentCandidate {
    pub instrument_id: String,
    pub canonical_symbol: String,
    pub display_name: String,
    pub asset_type: String,
    pub confidence: f32,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub requires_confirmation: bool,
}
```

解析顺序建议：

1. 精确命中 actor confirmed binding。
2. 精确命中 canonical symbol / alias。
3. 从 portfolio、company profile frontmatter、profile title、research task company name、event history 中生成本地候选。
4. 可选使用已有市场数据源做弱验证，例如 FMP quote/profile 是否存在；外部源失败只记录 warning。
5. 多候选或低置信度时返回 `requires_confirmation=true`。

### 3. 渐进接入 portfolio

- `PortfolioStorage` 保持旧 JSON schema 可读写，第一版只在 `Holding` 增加可选 `instrument_id`，不破坏现有文件。
- `portfolio_tool` 在 `add/update/watch` 前调用 resolver。若唯一候选高置信度，写入 `instrument_id` 与规范化 `symbol`；若需要确认，返回候选列表，让 agent 询问用户。
- `remove/unwatch` 同时支持 raw symbol 和 instrument id 查找；避免用户输入旧 ticker 后删不掉已迁移记录。
- 对期权，`Holding.underlying` 优先绑定 equity instrument，期权合约本身可以后续再建 derivative instrument。

### 4. 渐进接入 company profile

- `ProfileMetadata.stock_code` 继续保留，新增可选 `instrument_id` 或通过 sidecar index 记录 profile 到 instrument 的绑定。
- `company_profile` 读取时生成 `ProfileIdentityProjection`：profile id、company name、stock code、aliases、候选 instrument、health warning。
- public/admin digest 不再只靠 `extract_tickers(&md)`；优先读 frontmatter/sidecar identity，再回退文本提取。
- 画像导入 preview 增加 identity conflict：导入包里的 `AAPL` 是否绑定到本 actor 现有 Apple instrument，是否会创建重复画像。

### 5. 渐进接入 event-engine

- `MarketEvent.symbols` 暂时不改，新增解析投影：`resolved_instruments: Vec<instrument_id>` 可以存在 store projection 或 router 内部上下文，不一定第一版写回原事件。
- `SubscriptionRegistry` 从 portfolio 构建 watch pool 时同时生成 `symbol -> instrument_id` 映射；命中事件时先 raw symbol 兼容，再按 resolved instrument 去重。
- 对 ticker change / split / merger 这类事件，输出候选 identity mutation，交给 corporate action reconciliation 或人工确认，而不是直接改历史数据。
- `same_symbol_cooldown` 后续可升级为 `same_instrument_cooldown`，避免 symbol 变体绕过冷却。

### 6. API 与前端

建议新增 backend API：

- `GET /api/instruments?actor=&query=&status=`
- `GET /api/instruments/{instrument_id}`
- `POST /api/instruments/resolve`
- `POST /api/instruments/bind`
- `GET /api/users/{actor}/instrument-health`

前端第一阶段只需要：

- portfolio 添加/关注入口调用 resolve。
- SymbolDrawer 改为接收 `instrument_id` 或 raw symbol；若只有 raw symbol，先 resolve 再展示候选。
- 用户详情页和 public portfolio 展示 identity warning，不做复杂编辑器。

### 7. 兼容与迁移策略

- 不做一次性全量强迁移。启动或管理端操作时可按需 rebuild identity index。
- 旧 portfolio 没有 `instrument_id` 时继续工作；读取时返回 `identity_status=unresolved`。
- 旧 company profile 没有 frontmatter 或 stock code 时继续工作；进入 health warning 和可确认绑定流程。
- 所有自动绑定都必须可解释：记录 source、confidence、reasons。
- 对低置信度、多候选、影响 existing portfolio/profile 的动作，要求用户确认。

## 实施步骤

### Phase 0: 设计契约与夹具

1. 定义 `Instrument`, `InstrumentAlias`, `ActorInstrumentBinding`, `InstrumentResolveInput`, `InstrumentCandidate`。
2. 增加 resolver 纯逻辑测试夹具：大小写、`$AAPL`、`BRK.B/BRK-B`、company name、中文 alias、期权 underlying、多候选、无候选。
3. 明确 identity 不替代 `ActorIdentity` / `SessionIdentity`，只描述投资标的。

### Phase 1: 本地 registry 与 portfolio 接入

1. 在 `memory` 增加 registry 存储。
2. 从现有 portfolio 和 company profile 生成初始候选，不写回原文件。
3. `portfolio_tool` 的 `watch/add/update/remove` 接入 resolve 结果。
4. `Holding` 增加可选 `instrument_id`，保存时保持旧字段兼容。

### Phase 2: profile/digest/SymbolDrawer 接入

1. company profile 列表 API 返回 identity projection 和 warning。
2. public/admin digest 读取 profile ticker 时优先使用 identity projection，回退文本提取。
3. `SymbolDrawer` 从 raw symbol 查询升级为 instrument-aware 查询；标题 contains 只作为 fallback。
4. UI 增加“绑定/确认标的”的轻量动作，不做完整 master-data 编辑器。

### Phase 3: event-engine projection

1. subscription registry 维护 `canonical_symbol -> instrument_id` 映射。
2. router 记录 unresolved symbols 和 instrument match reason，供 admin 排障。
3. price/news/analyst/corp action 事件按 raw symbol 兼容，同时支持 instrument 去重和 cooldown 观测。
4. 为 ticker change / merger / spin-off 输出 identity mutation candidate，交给后续 corporate action 流程处理。

### Phase 4: 可观测性与治理

1. 增加 identity health 页面或 users tab。
2. `hone-cli doctor/status` 报告 identity registry 可读性、未解析数量、最近冲突。
3. 增加 manual rebuild 命令或 admin action。
4. 与 backup/export proposal 对齐，把 registry 纳入用户数据备份和删除范围。

## 验证方式

### 自动化测试

- `memory` 单元测试：
  - alias/canonical symbol 大小写归一。
  - actor binding 优先级高于全局 alias。
  - 多候选返回 `requires_confirmation`。
  - 旧 portfolio JSON 读取后不会丢字段，新增 `instrument_id` 可选。
- `portfolio_tool` 测试：
  - watch/add/update 能写入高置信度 instrument。
  - 低置信度输入不直接落盘，而是返回候选确认。
  - remove 支持旧 symbol 和 canonical symbol。
- `company_profile` 测试：
  - profile frontmatter stock_code/aliases 生成 identity projection。
  - legacy profile 无 frontmatter 仍可展示，状态为 unresolved 或 inferred。
- event-engine 测试：
  - `BRK.B` / `BRK-B` 变体能命中同一 watch pool projection。
  - same instrument cooldown 不重复推送。
  - unresolved event symbol 被记录但不阻断 raw symbol 兼容路径。
- 前端 model tests：
  - SymbolDrawer resolve 状态、候选选择、无候选、已绑定状态。
  - public portfolio 使用 instrument 连接 mainline/profile，而不是仅 ticker set。

### 手工验收

- 给一个 actor 添加 `AAPL` watchlist，创建 Apple 画像，刷新 public portfolio，确认 portfolio/mainline/profile/SymbolDrawer 全部指向同一 instrument。
- 输入 `BRK.B`，确认系统展示 `BRK.B/BRK-B` 兼容提示，不创建重复 watchlist。
- 导入一个缺 stock_code 的画像包，确认 preview 显示可绑定候选而不是静默当作无 ticker。
- 模拟 event-engine 收到 raw symbol 变体，确认该 actor 的订阅命中且 admin 能看到 match reason。

### 指标

- unresolved holdings count。
- unresolved profile count。
- event symbols unresolved rate。
- duplicate profile by instrument count。
- resolve confirmation acceptance rate。
- identity-related notification miss reports。

## 风险与取舍

- 风险：过早引入复杂 master data 会拖慢主线。取舍：第一版只做本地 registry、alias、binding、projection，不做完整证券数据库。
- 风险：自动解析错误会污染 portfolio 和画像。取舍：低置信度、多候选、跨资产类型、影响已有数据的动作必须用户确认。
- 风险：引入 `instrument_id` 后与现有 `symbol` 双写不一致。取舍：`symbol` 继续作为展示和外部 API 兼容字段，`instrument_id` 是增强字段；后台 health 负责发现不一致。
- 风险：外部数据源不可用。取舍：解析服务必须能离线工作；外部源只提升置信度或补充元数据。
- 风险：现有提案已经有 portfolio/corporate action/portrait health 工作，可能范围交叉。取舍：本提案不处理交易流水、不处理公司行动调账、不定义画像内容质量，只提供被这些功能复用的身份解析底座。
- 不做：不接入券商账户，不自动交易，不把公司画像变成 UI 编辑器，不替换 `ActorIdentity` / `SessionIdentity`，不要求所有旧数据一次性迁移。

## 与已有提案的差异

查重范围：已检查 `docs/proposal/` 与 `docs/proposals/` 下现有提案文件名、标题、相关摘要，并额外搜索 `entity`、`instrument`、`ticker`、`symbol`、`canonical`、`alias`、`exchange`、`currency` 等关键词。

- 不重复 `auto_p1_portfolio-transaction-ledger.md`：交易流水解决“仓位为什么变成这样”；本提案解决“这条仓位到底指向哪个投资标的”。
- 不重复 `auto_p1_corporate-action-reconciliation.md`：公司行动提案处理 split/dividend/ticker change 的调账候选；本提案提供 ticker change 前后识别同一 instrument 的身份底座。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：暴露雷达关注组合风险维度；本提案关注组合、画像、事件和 UI 是否指向同一标的。
- 不重复 `auto_p1_company-portrait-health.md`：画像健康检查缺 ticker、缺 refs、过期复审等内容质量；本提案把 profile 的 `stock_code/aliases/title` 连接到可复用 instrument registry。
- 不重复 `auto_p1_cross-company-thesis-map.md`：跨公司主线地图关注多个公司之间的行业假设；本提案先解决单个公司/证券身份在各模块的稳定连接。
- 不重复 `auto_p1_source-provenance-freshness.md`：source provenance 记录数据源请求与新鲜度；本提案记录输入字符串和投资标的身份的解析、确认与冲突。
- 不重复 `auto_p1_linked-user-workspace.md`：linked workspace 解决同一真实用户跨 actor 的资产空间；本提案在 actor/workspace 内部解决投资标的身份。
- 不重复 `auto_p1_investment_context_intake.md`：context intake 关注用户建仓和补齐上下文流程；本提案为这些流程提供标准化解析和确认 API。

差异结论：已有提案已经覆盖很多上层投资工作台能力，但缺少一个跨 portfolio、event-engine、company profile、research task 和 SymbolDrawer 的 instrument identity 基座。本提案小而基础，能降低后续多个提案的重复实现和错配风险。

## 文档同步说明

本轮只新增 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。如果后续执行本提案，应按动态计划准入标准新增或复用 `docs/current-plans/instrument-identity-registry.md`，并在新增 registry 模块、API、前端入口、event-engine projection 或 portfolio schema 可选字段时同步更新 `docs/repo-map.md`、`docs/invariants.md`、必要的 decision/ADR、备份/删除相关文档以及对应测试说明。
