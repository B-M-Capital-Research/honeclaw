# Proposal: Data Licensing and Attribution Boundary for Market, News, and Search Sources

status: proposed
priority: P1
created_at: 2026-06-02T02:07:00+08:00
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
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_external-egress-ledger.md`
- `docs/proposal/auto_p2_signal-source-lab.md`
- `docs/proposal/auto_p2_shareable-investment-briefs.md`
- `config.example.yaml`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-event-engine/src/fmp.rs`
- `crates/hone-event-engine/src/event.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-event-engine/src/pollers/news.rs`
- `crates/hone-event-engine/src/pollers/rss.rs`
- `crates/hone-event-engine/src/pollers/sec_enrichment.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/chat.tsx`
- `skills/market_analysis/SKILL.md`
- `skills/stock_research/SKILL.md`

## 背景与现状

Hone 的产品核心已经从单轮问答延伸到持续投资研究工作台：用户可以在 Web、桌面、Feishu、Telegram、Discord、iMessage 和 OpenAI-compatible API 中提问；event-engine 会主动拉取行情、新闻、财报日历、SEC filing、RSS feed、社交源，并把重要事件推送到用户的多渠道端；skills 也会通过 `data_fetch` 和 `web_search` 补充最新材料。

代码层面已经有很强的“事实来源”和“运行可靠性”基础：

- `crates/hone-tools/src/data_fetch.rs` 通过 FMP 获取 quote、profile、financials、news、gainers/losers、sector performance、ETF holdings、earnings calendar 和 snapshot，并支持多 API key fallback。
- `crates/hone-tools/src/web_search.rs` 通过 Tavily 搜索，并支持 key pool fallback、额度/鉴权错误识别和错误脱敏。
- `crates/hone-event-engine/src/fmp.rs` 是 event-engine 的 FMP HTTP 客户端，负责 key fallback、认证/额度错误判断和响应清洗。
- `crates/hone-event-engine/src/pollers/news.rs` 会把 FMP stock news 分类为 trusted、PR wire、opinion blog、uncertain，并把 legal-ad / transcript 等低信号内容降级。
- `crates/hone-event-engine/src/pollers/rss.rs` 支持 Bloomberg、SpaceNews、STAT News 等 RSS 源，把 feed item 转成 `MarketEvent`，并模拟 FMP payload 以复用后续 collector / router。
- `crates/hone-event-engine/src/pollers/sec_enrichment.rs` 会抓取 SEC.gov filing HTML，按 section-aware 摘抄后交给 LLM 生成长期主线投资者摘要，并在配置中要求部署方设置包含联系邮箱的 SEC User-Agent。
- `MarketEvent` 已包含 `source`、`url`、`occurred_at`、`summary`、`payload`；`EventStore` 会把事件落 SQLite，并可写 JSONL 镜像和 delivery audit。
- `config.example.yaml` 已经记录 FMP 配额风险、RSS feed、global digest full-text fetch、Jina fallback、SEC User-Agent、OpenRouter profile 和 Tavily key 等配置。
- 现有 proposal 已经提出 `Source Provenance and Freshness Registry`、`External Egress Ledger`、`Signal Source Lab`、`Shareable Investment Briefs` 等相邻能力。

这些能力说明 Hone 已经在努力解决“事实是否新鲜、来源是否可信、外部调用是否可解释”的问题。但还有一个独立缺口：Hone 还没有把第三方数据的授权、归因、缓存保留、再分发边界和用户可见 attribution 做成一等产品/架构约束。

当前所有外部事实大多进入统一的事件、工具结果、digest、chat answer 或 notification body。系统能告诉研发“来源字符串是什么”，但还不能稳定回答：

- 某条输出中来自 FMP、RSS、SEC、Tavily、Jina、LLM 摘要或用户上传材料的内容分别占多少。
- 这些内容能否出现在 public Web、IM 推送、分享 brief、support bundle、Hone Cloud API response 或未来 webhook 中。
- 是否必须显示 provider/source attribution、原文 URL、访问时间或免责声明。
- 缓存里能保留完整正文、摘要、hash、metadata 还是只能保留链接。
- 管理员新增 RSS/source 时是否已经记录该来源的授权类别和产品可用范围。
- 开源用户、self-host 用户和 Hone Cloud 服务是否应采用不同的数据授权责任边界。

## 问题或机会

这是 P1 级问题。它不像 secret 泄露或错误投资建议那样是即时 P0，但会显著影响公开产品可信度、商业化、分享增长、团队版、自托管部署、support/debug 和后续合规审查。

主要风险和机会集中在六类链路：

1. **用户看到的是 Hone 的答案，但数据责任来自多个上游。**  
   当一条 digest 同时包含 FMP quote、RSS 摘要、SEC filing 摘抄、Tavily 搜索结果和 LLM 归纳时，用户很难知道哪些是官方披露、哪些是聚合新闻、哪些是模型摘要、哪些只是搜索候选。投资产品必须让事实来源和使用边界可解释。

2. **source provenance 解决“哪里来”，不解决“能怎么用”。**  
   `Source Provenance and Freshness Registry` 可以记录 provider、endpoint、fetched_at、fallback、freshness；但授权边界还需要额外字段：允许展示摘要还是全文、允许缓存多久、是否允许公共分享、是否必须附 attribution、是否允许进入训练/评测样本、是否仅限用户自带 key 的 self-host 场景。

3. **share / webhook / support / public API 会放大再分发风险。**  
   Hone 已有或已有提案覆盖 shareable briefs、webhook、redacted support bundle、Hone Cloud API。如果没有统一的 data license policy，这些能力容易把原本只适合“用户自己看”的第三方内容转成“可转发给别人或机器消费”的内容。

4. **RSS 与 full-text 抓取的产品边界比普通 API 更模糊。**  
   `rss.rs` 当前把 RSS item 的 title/link/summary 转成 `MarketEvent`，global digest 还可以启用 full-text fetch 和 Jina fallback。工程上这是合理的补源策略，但产品上需要把“feed 摘要、抓取正文、LLM 摘要、原文链接”区分清楚，避免把抓取文本当作 Hone 自有内容长期保存或大段再分发。

5. **开源仓库和托管服务的责任边界不同。**  
   README 已说明部分专业估值工具和 proprietary workflows 不在公开仓库中。类似地，外部数据授权也可能因部署方式不同而不同：self-host 用户用自己的 FMP/Tavily/Jina key 时，责任更偏部署方；Hone Cloud 或 public Web 用服务方 key 时，产品需要更严格的 attribution、scope 和 retention。

6. **管理端缺少“数据来源授权配置”的操作面。**  
   `Signal Source Lab` 可以帮助新增/试运行 RSS 或 Telegram source，但如果没有 source license metadata，管理员只能判断噪音和技术健康，不能判断“这个来源是否可以进入 public digest、是否能进入 share brief、是否必须显示 attribution”。

机会是：Hone 不需要先实现完整法律合规模块，也不需要在代码里写死各 provider 条款。第一版只要建立一个部署方可配置、产品可消费的 **Data Licensing and Attribution Boundary**，就能让后续的 provenance、egress、brief、webhook、support 和 source lab 都有同一套“可怎么用”的判断依据。

## 方案概述

新增一个轻量的 **Data License Policy Registry**，用于描述外部数据源在 Hone 内的使用边界。它不替代 source freshness、egress ledger、event store 或 run trace，而是给每个 source/provider/endpoint 绑定可机读的授权与归因策略。

核心目标：

- 给每个外部数据源定义 `license_scope`、`allowed_surfaces`、`retention_policy`、`attribution_policy`、`redistribution_policy` 和 `operator_review_status`。
- 让 event-engine、tool result、digest renderer、public API、share brief、support bundle、webhook 等在输出前能查询“这段材料是否允许出现在当前 surface”。
- 在用户可见输出中提供稳定的、短而清晰的 attribution label，例如 `FMP quote, fetched 08:28`、`SEC filing excerpt`、`RSS summary from Bloomberg feed`、`Search result via Tavily`。
- 对不确定授权的来源默认使用保守策略：只展示短摘要和原文链接，不缓存全文，不进入 share / webhook / public API 的长期可携带 artifact。
- 明确 self-host 与 hosted/cloud 的责任边界：默认配置给出保守模板，部署方可以在 admin UI 或 config 中确认自己有权启用更宽的使用范围。

第一版应聚焦“元数据与执行前检查”，不做自动法律判断。系统只表达和执行部署方已经确认的策略。

## 用户体验变化

### 用户端

- Public chat、portfolio、digest 和 notification 中，时间敏感或外部事实密集的回答可显示简洁来源：
  - `行情: FMP, 08:28 获取`
  - `来源: SEC 8-K 原文摘要`
  - `新闻: RSS 摘要 + 原文链接`
  - `搜索: Tavily results, query rewritten at 08:31`
- 当某个内容不能被分享或导出时，界面不要只报错，而是解释为数据边界：
  - `此摘要包含仅供当前用户查看的第三方新闻摘录，分享版将只保留结论、来源名和原文链接。`
- Shareable brief 或未来 webhook 默认不携带长篇新闻正文、搜索结果正文或抓取全文，只携带 Hone 自己的分析、简短 citation、URL 和 fetched_at。
- 用户导出数据包时，manifest 能标注哪些条目来自第三方 source、哪些是 Hone 生成摘要、哪些因授权策略只导出 metadata。

### 管理端

- Settings 或未来 Signal Source Lab 增加 `Data Sources & Licensing` 区块：
  - FMP、Tavily、SEC、RSS feeds、Jina、Telegram social sources、future providers 的策略列表。
  - 每个 source 显示授权状态：`default_safe`、`operator_confirmed`、`restricted`、`disabled_for_public_surfaces`、`needs_review`。
  - 管理员新增 RSS feed 时必须选择用途范围：`internal_digest_only`、`public_user_digest`、`shareable_summary_allowed`、`metadata_only`。
  - 对 hosted/cloud 部署，默认更保守；对 self-host，UI 清楚提示“使用你自己的 provider key 和 source 配置时，你负责确认授权范围”。
- Notifications 详情抽屉展示 attribution 和 license flags：
  - `source=FMP stock_news`
  - `cache=summary_only`
  - `redistribution=private_user_surface_only`
  - `share/export=metadata_only`
- Support bundle 生成时可以自动移除或降级受限第三方正文，只保留 event id、source label、URL、hash、timestamp 和 Hone 生成的非侵权摘要。

### 桌面端

- Desktop bundled 模式默认走 self-host/local policy，提示用户外部数据 key 由本机配置控制。
- Desktop remote mode 显示远端 backend 的 policy snapshot，避免用户误以为远端托管数据和本机 self-host 数据有同样边界。
- 诊断页可把“数据源未确认授权”列为功能降级原因，而不是等输出阶段才失败。

### 多渠道

- Feishu/Telegram/Discord/iMessage 的消息空间有限，应使用短 attribution：
  - `SEC filing · 10-Q · link`
  - `FMP quote · 08:28`
  - `RSS Bloomberg · summary only`
- 群聊默认不发送含个人私有数据或受限第三方长摘录的内容；如果需要，走 permission/clarification。
- 主动推送里的 source label 应稳定，方便用户建立“这是官方 filing / 行情 / 新闻摘要 / 搜索候选”的心智。

## 技术方案

### 1. 新增 Data License Policy 类型

建议在 `hone-core` 或 `memory` 定义稳定类型，先由 Web API / event-engine / tools 消费：

```rust
pub struct DataLicensePolicy {
    pub source_key: String,
    pub provider: String,
    pub endpoint_or_feed: Option<String>,
    pub license_scope: LicenseScope,
    pub allowed_surfaces: Vec<OutputSurface>,
    pub attribution: AttributionPolicy,
    pub retention: RetentionPolicy,
    pub redistribution: RedistributionPolicy,
    pub operator_review_status: ReviewStatus,
    pub updated_at: String,
    pub updated_by: Option<String>,
}
```

示例枚举：

- `LicenseScope`: `user_provided_key`, `hosted_service_key`, `public_open_data`, `rss_summary`, `search_result`, `unknown_restricted`
- `OutputSurface`: `private_chat`, `private_digest`, `admin_debug`, `public_chat`, `public_api`, `share_brief`, `webhook`, `support_bundle`, `model_eval_fixture`
- `AttributionPolicy`: `none_required`, `source_label`, `source_label_and_url`, `provider_and_fetched_at`, `custom_text`
- `RetentionPolicy`: `metadata_only`, `summary_only`, `short_excerpt_with_ttl`, `full_payload_with_ttl`, `no_persistent_cache`
- `RedistributionPolicy`: `private_only`, `summary_with_link`, `metadata_only`, `blocked`
- `ReviewStatus`: `default_safe`, `operator_confirmed`, `needs_review`, `restricted`, `disabled`

第一版可以本地 JSON / SQLite 存储，cloud mode 后续放 PG。默认策略从 config seed 生成，部署方通过 admin API 覆盖。

### 2. 为 SourceObservation / MarketEvent 附 policy snapshot

当 `data_fetch`、`web_search`、FMP poller、RSS poller、SEC enrichment、global digest full-text fetch 产生外部事实时，附上轻量 policy snapshot：

```json
{
  "source_policy": {
    "source_key": "rss:bloomberg_markets",
    "attribution": "source_label_and_url",
    "retention": "summary_only",
    "redistribution": "summary_with_link",
    "review_status": "operator_confirmed"
  }
}
```

`MarketEvent.payload` 可以先携带 snapshot，避免历史事件因策略变动而失去当时判断；查询时也可结合最新 policy 给出“当前 policy 已更改”的提示。

### 3. 输出前增加 surface check

在以下路径增加轻量检查，不要求第一版阻断所有旧路径：

- digest / notification renderer：长正文输出前检查 `private_digest` / channel surface 是否允许。
- public chat / public API response：若工具结果包含受限 source，输出只保留 Hone analysis + short citation。
- share brief / support bundle / webhook：默认走 `share_brief` / `support_bundle` / `webhook` surface，受限 source 降级到 metadata-only。
- admin debug：允许看更多，但必须显示 policy flags 和 operator audit。

第一版遇到 unknown source 的默认策略：

- private chat/digest：允许短摘要 + URL。
- public API/share/webhook/support bundle：metadata only，除非 operator 确认。
- model eval fixture：不允许保留第三方正文，只保留自造/脱敏 fixture 或 hash/URL。

### 4. 管理端与配置

新增 API：

- `GET /api/data-source-policies`
- `PUT /api/data-source-policies/:source_key`
- `POST /api/data-source-policies/preview-surface`
- `POST /api/data-source-policies/reset-defaults`

Config seed 可放在 `config.example.yaml` 的新 section，或由代码默认生成：

```yaml
data_sources:
  policies:
    fmp:
      attribution: provider_and_fetched_at
      retention: summary_only
      redistribution: private_only
    tavily:
      attribution: provider_and_fetched_at
      retention: metadata_only
      redistribution: summary_with_link
    sec:
      attribution: source_label_and_url
      retention: short_excerpt_with_ttl
      redistribution: summary_with_link
```

注意：默认配置只是产品策略模板，不应在仓库中声称某个 provider 的法律条款。最终授权状态由部署方确认。

### 5. 与现有提案联动

- `Source Provenance and Freshness Registry` 负责记录事实来源、时间、fallback 和 freshness；本提案补充“允许怎么展示、缓存和再分发”。
- `External Egress Ledger` 负责记录数据离开 Hone 的边界；本提案为 egress 前的 surface allow/deny 和 data-class 降级提供规则。
- `Signal Source Lab` 负责 source probe / trial / lifecycle；本提案要求新增 source 时必须带 license policy 或标记 `needs_review`。
- `Shareable Investment Briefs`、`Webhook Delivery Gateway`、`Redacted Support Bundle` 在生成 artifact 时应调用同一 policy check，避免各自写一套过滤规则。

## 实施步骤

### Phase 1: Policy Registry 和默认保守策略

- 新增 `DataLicensePolicy` 类型和本地存储。
- 为 FMP、Tavily、SEC、RSS、Jina、Telegram social source 生成默认策略。
- 在 admin API 暴露 policy list/update/reset。
- 增加单元测试：unknown source 默认 restricted；private digest 可短摘要；share/webhook/support bundle 默认为 metadata-only。

### Phase 2: Event 和 Tool 结果携带 policy snapshot

- `data_fetch` 和 `web_search` 返回结构化 metadata，附 `source_key` 和 policy snapshot。
- FMP event-engine pollers、RSS poller、SEC enrichment 写入 `MarketEvent.payload.source_policy`。
- `EventStore` 查询返回 policy snapshot，notifications detail 展示 flags。
- 增加回归样本：FMP quote、RSS event、SEC filing、Tavily search 各一条。

### Phase 3: Surface-aware 输出降级

- digest / notification renderer 增加受限 source 的输出降级：长正文转 short summary + URL。
- public chat/API、share brief、support bundle、webhook 生成器调用统一 `check_data_source_surface(...)`。
- 支持输出解释：当正文被降级，用户看到的是“基于数据边界已隐藏长摘录”，不是沉默丢内容。

### Phase 4: Source Lab 与 Cloud/Self-host 分层

- Signal Source Lab 新增 source policy 步骤，新增 RSS / Telegram source 默认 `needs_review`。
- Cloud/hosted deployment 可把未确认 source 禁止进入 public/share/webhook surface。
- Self-host/local deployment 允许 operator 显式确认更宽策略，并在 UI 中标注由部署方负责。

## 验证方式

- 静态验证：
  - policy 文件或表能列出 FMP、Tavily、SEC、RSS、Jina、Telegram social source 的默认策略。
  - proposal 所述 source keys 能映射到 `MarketEvent.source`、tool provider 或 config source。
- 单元测试：
  - unknown source 在 public/share/webhook/support bundle surface 下被降级为 metadata-only。
  - private chat/digest 对 `summary_with_link` source 允许短摘要和 URL。
  - operator-confirmed policy 能放宽指定 surface，但不会影响无关 source。
  - policy snapshot 在 event 入库后保留当时状态。
- 集成/回归测试：
  - 构造 FMP quote、RSS news、SEC filing summary、Tavily search result，验证 renderer 输出包含正确 attribution。
  - 构造 share/support/webhook 输出，验证不会包含受限 third-party long text。
  - 修改 source policy 后，notifications detail 和 admin policy UI 能显示新旧状态差异。
- 手工验收：
  - 管理员新增 RSS feed 时必须选择或确认 policy。
  - Public 用户分享 brief 时，受限新闻正文被替换为 Hone 摘要、source label、URL 和 fetched_at。
  - Desktop remote 用户能看到远端 backend 的 data source policy 状态。
- 指标：
  - `unknown_source_policy_count`
  - `restricted_surface_downgrade_count`
  - `source_policy_needs_review_count`
  - `attribution_missing_block_count`
  - `third_party_long_text_export_block_count`

## 风险与取舍

- **不是法律意见。** 该 registry 只让产品执行部署方声明的策略，不自动解释或替代 provider 条款审查。
- **会增加输出复杂度。** 过多 attribution 会让 IM 消息变臃肿；必须按 surface 使用短 label，而不是把完整 policy 暴露给用户。
- **旧事件可能缺 policy snapshot。** 迁移期需要默认 `manual_or_legacy` / `unknown_restricted`，避免误认为旧数据已确认授权。
- **过度保守会降低产品体验。** 如果默认把所有 RSS/search 内容都 metadata-only，digest 质量会下降；需要允许 operator 对已确认 source 放宽到 short summary。
- **不能和 freshness 混为一谈。** 一条数据可以很新鲜但不允许 public share，也可以允许 share 但已经过期；两个判断需要独立。
- **不应把 provider 条款硬编码进开源仓库。** 默认策略应是保守模板，真实授权范围由部署环境确认。

## 与已有提案的差异

- 与 `auto_p1_source-provenance-freshness.md` 不重复：该提案解决事实来源、时间、新鲜度、fallback 和 provider health；本提案解决同一事实在不同 surface 中的授权、归因、缓存和再分发边界。
- 与 `auto_p1_external-egress-ledger.md` 不重复：该提案记录用户数据离开 Hone 的第三方边界；本提案在输出前决定外部来源数据能否进入某个边界，以及需要如何降级。
- 与 `auto_p2_signal-source-lab.md` 不重复：该提案管理事件源的 probe、trial、启停和噪音影响；本提案要求每个 source 在启用前带上可执行的 license policy。
- 与 `auto_p2_shareable-investment-briefs.md` 不重复：该提案定义可分享 brief 的增长产品形态；本提案提供 brief 生成时过滤第三方来源正文和保留 attribution 的底层规则。
- 与 `auto_p1_user-data-trust-center.md` 不重复：该提案关注用户自己的数据导出/删除/隐私控制；本提案关注 Hone 引入的第三方数据在用户输出和 artifact 中的使用边界。

查重结论：`docs/proposal/` 和 `docs/proposals/` 中已有来源可信度、外发账本、source lab、share brief、support bundle 等相邻主题，但没有一篇专门把第三方市场/新闻/搜索数据的授权、归因、缓存和再分发策略抽象为产品架构边界。本提案覆盖的是一个新的可执行治理层。
