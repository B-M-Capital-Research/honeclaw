# Proposal: SEC Filing Diff Ledger for Material Change Review

status: proposed
priority: P1
created_at: 2026-06-09 20:02:36 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_data-licensing-attribution.md`
- `docs/proposal/auto_p1_earnings-lifecycle-workbench.md`
- `docs/proposal/auto_p1_mainline-distill-ledger.md`
- `docs/proposal/auto_p1_corporate-action-reconciliation.md`
- `crates/hone-event-engine/src/pollers/sec_enrichment.rs`
- `crates/hone-event-engine/src/pollers/corp_action.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-event-engine/src/renderer.rs`
- `crates/hone-event-engine/src/pipeline.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/research.tsx`
- `memory/src/company_profile/{types,storage}.rs`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 已经把 SEC filing 纳入主动事件链路：`SecFilingsPoller` 按持仓 / watch pool ticker 拉取 FMP `/v3/sec_filings`，默认关注 `8-K`、`10-Q`、`10-K`、`S-1`、`DEF 14A`，并生成稳定的 `sec:{SYM}:{ACCESSION}` 事件。`EventKind::SecFiling` 会按 form 设置严重度，`EventStore` 用 SQLite 做事件去重和投递审计，renderer 会把 SEC 事件推送到 Web / IM 渠道。

当前 SEC enrichment 已经比简单标题推送更进一步：`crates/hone-event-engine/src/pollers/sec_enrichment.rs` 会抓取 SEC.gov HTML，过滤 XBRL 噪声，优先抽取 MD&A、Risk Factors、Legal Proceedings、8-K 前段叙事和业务 / 资本配置 / 风险关键词窗口，再交给 LLM 生成约 200 字长期主线投资者摘要。代码注释也明确了两个边界：

- enrichment 是非阻塞 best-effort，失败时事件仍保留并回退到基础 summary。
- 目前不做“基于上一期 filing 的 diff”；Risk Factors 只能从本期披露里摘取显式变化或 `no material changes` 口径。

这让 Hone 能“看见 SEC filing”，但还不能稳定回答投资用户更关心的问题：这份 10-Q / 10-K 相比上一期到底变了什么，哪些变化可能证伪或强化长期主线，哪些只是模板文字、会计表格或 routine filing。

## 问题或机会

SEC filing 是投资研究里最接近一手事实的来源之一，但当前产品链路仍偏“通知流”：

1. **摘要无法区分新增变化和重复披露。**  
   LLM 摘要只看到本期精选摘抄，不知道上一期相同 section 的原文。它可能把长期存在的风险、模板诉讼措辞或重复业务描述误写成新信号，也可能漏掉措辞微调背后的风险升级。

2. **事件入库后缺少可复用的 filing 结构化资产。**  
   `MarketEvent.payload.llm_summary` 适合推送，但不适合作为后续研究、回测、画像更新、用户追问和模型评估的长期输入。当前系统没有 actor / symbol 级的 filing section snapshot、hash、period、form family、prior accession 关系和 diff 结果。

3. **用户看到 SEC 推送后仍缺少复盘路径。**  
   `Evidence Review Queue` 可以把 SEC 事件变成待处理项，但它不负责理解 filing 内部变化。本提案补上“这条 SEC evidence 为什么值得复盘”的 filing-specific 判断层。

4. **运营和模型评估缺少 filing 质量样本。**  
   `Model Route Evaluation Lab` 能评估背景模型路线，但 SEC enrichment 目前很难构造稳定的“同一 ticker 相邻 filing 的 material diff”样本。没有 diff ledger，模型升级后只能看摘要文风，难以判断是否更会识别主线变化。

机会是新增 **SEC Filing Diff Ledger**：把 SEC filing 从一次性事件升级为“可比较、可审计、可复盘”的研究资产。第一版只覆盖持仓 / watch pool 中的 10-Q、10-K 和高价值 8-K，不做全市场扫描，不直接改写公司画像，也不提供交易建议。

## 方案概述

新增一个 filing-specific 的派生层，围绕三类对象工作：

- `FilingSnapshot`  
  一份 filing 的稳定记录：symbol、form、accession、filing_date、period_end、source_url、source provider、raw HTML hash、extracted_at、extractor_version、section coverage、payload hash。

- `FilingSectionSnapshot`  
  section 级摘抄：`mdna`、`risk_factors`、`legal_proceedings`、`liquidity_capital_resources`、`business_updates`、`capital_allocation`、`front_narrative` 等。每段存 normalized text hash、excerpt、char count、confidence flags，不保存整份 HTML。

- `FilingDiffCandidate`  
  本期 vs prior comparable filing 的差异候选：prior accession、section id、change type、severity hint、stable / added / removed / materially_changed、deterministic diff summary、LLM materiality verdict、recommended review action。

核心流程：

1. `SecFilingsPoller` 发现新 filing 后继续生成 `MarketEvent`，保持现有推送链路兼容。
2. 新增 filing snapshot extractor，复用 `sec_enrichment.rs` 的 section-aware HTML 清理和 item span 识别逻辑，把可比较 section 落入 ledger。
3. 对 10-Q 比较上一季度或去年同期 10-Q，对 10-K 比较上一年 10-K，对 8-K 只在同 accession / exhibit 内做 front narrative 摘要，不强行找 prior。
4. 先运行 deterministic diff：section hash、相似度、关键词风险窗口、`no material changes` 状态变化、法律 / 监管 / 客户集中 / 流动性 / 债务 / SBC / capex 等 signal windows。
5. 只对 deterministic diff 命中的候选调用 LLM，要求输出“是否 material、影响哪条长期主线、是否需要写入 company portrait event”。
6. 将高置信候选接入通知、Evidence Review Queue、company portrait skill prompt 和管理端 SEC filing view。

## 用户体验变化

### 用户端

- Public `/portfolio` 或未来研究工作台里，SEC 卡片不只显示“公司提交了 10-Q”，而是显示：
  - “Risk Factors: no material changes -> 新增供应链集中风险”
  - “MD&A: backlog / capex / major customer language changed”
  - “Legal Proceedings: 新增监管调查段落”
- 用户点击 SEC 卡片后看到三层信息：本期摘要、相较上一期的关键变化、建议动作。
- 建议动作保持克制：`加入待复盘证据`、`让 Hone 检查是否更新画像`、`标记 routine`、`打开 SEC 原文`。不出现买卖建议。

### 管理端

- `/notifications` 的 SEC 事件详情增加 filing diff 摘要、prior accession、section coverage 和 extractor/model version，方便排查“为什么推这条”。
- 可新增 `/research/sec-filings` 或在现有 research 页面增加 SEC tab：
  - 按 actor、symbol、form、materiality、review status、extract status 过滤。
  - 展示 extraction failure、no prior comparable filing、LLM skipped、review queued 等状态。
  - 支持打开单条 diff candidate，看 section excerpt、prior excerpt、change reason 和后续 evidence queue 处理状态。

### 桌面端

- Desktop 不需要新增 sidecar。bundled / remote 模式都通过 Web API 读取同一 filing ledger。
- 本地通知或 dashboard badge 可以显示“2 条 SEC material changes 待复盘”，点击进入 Web 研究页。

### 多渠道

- Feishu / Telegram / Discord / iMessage 仍发送短消息，不在 IM 中塞长 diff。
- 高价值 SEC filing 的推送正文增加一行原因，例如“相较上一期 10-Q，Risk Factors 新增供应链集中风险段落；已加入待复盘证据。”
- 用户在 IM 中回复“复盘这份 10-Q”时，agent 可以引用 `FilingDiffCandidate`，而不是重新抓取整份 filing。

## 技术方案

### 存储与兼容

新增 filing ledger 存储，建议第一版放在 event-engine store 附近，但保持和 `MarketEvent` 解耦：

- local mode：SQLite 表或独立 `sec_filing_ledger.sqlite3`，避免把大 excerpt 混进 `delivery_log`。
- cloud mode：PG 表，例如 `cloud_sec_filing_snapshots`、`cloud_sec_filing_sections`、`cloud_sec_filing_diffs`。
- JSONL mirror：可选保留 snapshot / diff 的 compact JSONL，用于本地故障恢复和离线评估。

表设计原则：

- accession + form + symbol 是 filing 级稳定键。
- section 只存 normalized excerpt 和 hash，不保存完整 HTML，降低存储、版权和敏感信息风险。
- diff candidate 记录 extractor version、LLM profile/model、prompt hash、source section hashes，保证后续能解释为什么结果变化。
- 老事件没有 snapshot 时保持 degraded 展示，不迁移历史 `events` 表。

### 数据流

1. `events_from_sec_filings` 继续生成 `MarketEvent`，不改变事件 id。
2. `SecFilingsPoller::fetch` 在 enrichment 之前或之后调用 `FilingLedger::upsert_snapshot(event, html_or_fetcher)`。为避免重复 SEC 抓取，第一版可让 `LlmSecFilingSummarizer` 暴露已抓取 HTML / extracted blocks 的内部 helper，或者抽出共享 `sec_filing_extract` 模块。
3. ledger 根据 symbol/form/period 找 prior comparable filing。
4. deterministic diff 生成 `FilingDiffCandidate(status=deterministic_pending_llm | routine | no_prior)`。
5. LLM materiality reviewer 只处理候选，不处理 routine unchanged sections。
6. 若 verdict 为 material 或 counter-thesis risk，router 可把事件 severity hint 上调或把 reason 写入 payload；Evidence Review Queue 可读取 diff candidate 生成待复盘项。

### API 与 UI

建议新增 API：

- `GET /api/sec-filings?actor=&symbol=&form=&status=&limit=`
- `GET /api/sec-filings/:accession`
- `GET /api/sec-filings/:accession/diffs`
- `POST /api/sec-filings/:accession/review-action`

Public API 可只开放 actor 自己的数据：

- `GET /api/public/sec-filings?symbol=&limit=`
- `POST /api/public/sec-filings/:accession/review-action`

UI 第一版只做只读列表 + action，不做复杂 side-by-side 全文 diff。完整 SEC 原文继续用 `url` 跳转 SEC.gov。

### Agent 与 skill 接入

- `company_portrait` skill 接收 filing diff context 时，应明确区分：
  - 原始 filing excerpt
  - deterministic diff
  - LLM materiality verdict
  - 用户选择的动作
- agent 只能建议或执行画像事件追加，不自动覆盖 `profile.md` 主结论。
- 若用户要求“更新画像”，prompt 应要求：如果 diff 不改变 thesis，只追加 routine note 或说明不写入。

## 实施步骤

1. **抽出 SEC section extractor。**  
   从 `sec_enrichment.rs` 中拆出可复用的 HTML 清理、item span、section excerpt helper，保持现有摘要行为不变。

2. **新增 filing ledger 存储。**  
   定义 snapshot、section、diff candidate 类型和 local SQLite 表；cloud mode 预留 PG schema / repository trait。

3. **落 deterministic snapshot + diff。**  
   对 10-Q / 10-K 建立 prior comparable filing 查找和 section hash / similarity / risk keyword diff；先不接 LLM。

4. **接入 LLM materiality reviewer。**  
   只对 deterministic 命中的候选调用背景 profile，记录 prompt hash、model、verdict 和 failure reason。

5. **连接通知与 evidence queue。**  
   让 SEC push 文案使用 diff reason；material diff 自动生成或增强 evidence review item。

6. **增加 Web 只读视图。**  
   管理端先做 SEC filing tab，public `/portfolio` 只显示当前 actor 相关 ticker 的 material changes。

7. **灰度与回滚。**  
   用 feature flag 控制 `event_engine.sec_filing_diff_ledger`，关闭时保留原 SEC enrichment 和推送行为。

## 验证方式

- Rust 单元测试：
  - 使用固定 10-Q / 10-K HTML fixture，验证 section extraction、item span、risk/no-material-change 识别稳定。
  - 构造 prior/current section，验证 hash unchanged、routine wording、material added risk、removed disclosure、legal update 的 diff 分类。
  - 验证无 prior filing、HTML 抽取失败、LLM materiality 失败时事件不丢失且状态 degraded。
- Event-engine 回归：
  - 用 mock FMP + mock SEC HTML + mock LLM 跑 `SecFilingsPoller`，确认 `MarketEvent` id 不变、ledger 写入成功、router 能读到 diff reason。
  - 加入 CI-safe fixture，不依赖真实 SEC / FMP / OpenRouter。
- Web 测试：
  - `bun run test:web` 覆盖 SEC filing list model、diff status label、review action payload。
  - 手工验收移动端 public `/portfolio` 不被长 diff 撑破布局。
- 指标：
  - SEC filing extraction success rate。
  - material diff precision 的人工抽样通过率。
  - SEC evidence queue open -> handled 转化率。
  - LLM skipped ratio 与 token cost。

## 风险与取舍

- **风险：SEC filing 文本很长，存储与版权压力上升。**  
  取舍：只存 section excerpt、hash、URL 和差异摘要，不存完整 HTML；原文查看跳回 SEC.gov。

- **风险：文本 diff 误报 routine wording。**  
  取舍：第一版只把高置信变化推为 material；routine / ambiguous 默认进入 digest 或待复盘，不强行直推。

- **风险：LLM materiality reviewer 增加成本。**  
  取舍：先用 deterministic diff 缩小候选，只对 section hash 改变且命中风险 / 资本配置 / 业务关键词的候选调用模型。

- **风险：和 company portrait 自动更新混淆。**  
  取舍：ledger 只生产 evidence 和建议动作，不直接写画像；画像更新仍通过 agent + `company_portrait` skill，并保留用户确认路径。

- **风险：cloud/local 存储再次分叉。**  
  取舍：从第一版定义 repository trait 和 explicit storage mode；local SQLite 与 cloud PG 表语义一致，避免只做本地文件路径。

- **不做的边界：**  
  不做全市场 SEC 扫描，不做逐字全文 redline，不做法律审查，不做交易建议，不把 DEF 14A 治理细节扩展成独立治理产品，不替代现有 source provenance / evidence queue。

## 与已有提案的差异

- 不重复 `auto_p1_evidence_review_queue.md`：该提案负责把事件变成可处理的研究待办；本提案负责 SEC filing 内部 section snapshot、prior-period diff 和 materiality verdict，为 evidence queue 提供更高质量输入。
- 不重复 `auto_p1_source-provenance-freshness.md`：该提案回答事实来自哪里、何时获取、是否新鲜；本提案回答同一公司相邻 filing 的披露内容发生了什么变化。
- 不重复 `auto_p1_data-licensing-attribution.md`：该提案关注来源标注和使用边界；本提案关注 SEC 原文的结构化比较与复盘动作。
- 不重复 `auto_p1_earnings-lifecycle-workbench.md`：该提案围绕财报发布、transcript 和 post-call lifecycle；本提案覆盖 10-Q / 10-K / 高价值 8-K 的 filing diff，可被 earnings lifecycle 复用但不依赖财报工作台。
- 不重复 `auto_p1_mainline-distill-ledger.md`：该提案保护主线蒸馏的候选版本和回滚；本提案产生 filing-specific evidence，是否影响 mainline 仍由画像 / distill 链路决定。
- 不重复 `auto_p1_corporate-action-reconciliation.md`：该提案处理 split/dividend/ticker-change 等组合真相调整；本提案处理 SEC 披露文本变化，不自动修改 portfolio。

查重结论：`docs/proposal/` 与 `docs/proposals/` 中已有提案多次提到 SEC filing，但主题分别是来源可信、通知策略、通用证据复盘、财报 lifecycle、公司行动和主线蒸馏，没有覆盖“SEC filing section snapshot + prior comparable diff + materiality review + filing-specific复盘入口”的产品/架构层。因此本提案是新的 P1 主题。
