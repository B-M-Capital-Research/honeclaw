# Proposal: Answer Evidence Cards for User-Visible Trust

status: proposed
priority: P1
created_at: 2026-07-05 21:04:16 +0800
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
- `docs/proposal/auto_p1_factual_snapshot_cache.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `crates/hone-channels/src/run_event.rs`
- `crates/hone-channels/src/response_finalizer.rs`
- `crates/hone-channels/src/prompt_audit.rs`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `memory/src/session.rs`
- `memory/src/llm_audit.rs`
- `packages/app/src/components/chat-view.tsx`
- `packages/app/src/lib/messages.ts`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/notifications.tsx`

## 背景与现状

Hone 的核心承诺是做严肃投资研究助手，而不是只给出流畅文本。README 已经把长期公司画像、持仓监控、定时任务、多渠道提醒和投资纪律作为主产品价值；`docs/invariants.md` 也要求时间敏感分析使用当前时间、区分事实与噪音、不输出买卖指令，并保持 company portraits 的证据和反证条件。

当前代码已经具备不少“证据材料”，但它们主要服务运行时和排障，还没有变成用户可见的回答结构：

- `crates/hone-tools/src/data_fetch.rs` 会拉 FMP 的 quote/profile/news/financials/earnings calendar，并有短 TTL 的内存缓存与 key fallback，但工具结果最终通常只被模型消化成文本。
- `crates/hone-tools/src/web_search.rs` 能做 Tavily 搜索并处理 key fallback，但搜索来源、查询时间、失败/降级状态不会以统一 UI 元数据挂到最终回答上。
- `crates/hone-channels/src/run_event.rs` 的运行事件包含 `Progress`、`StreamDelta`、`ToolStatus`、`StreamThought` 和 `Error`，适合实时流式展示，但没有“本条最终回答引用了哪些证据”的稳定事件类型。
- `crates/hone-channels/src/response_finalizer.rs` 负责清理用户可见文本、恢复工具结果、处理本地图表 `file://` marker、同步 company profiles 到 PG；它能防止明显泄漏和空成功，但不生成回答级证据结构。
- `memory/src/session.rs` 已经持久化版本化会话、tool result messages、summary 和 session ownership；`memory/src/llm_audit.rs` 能保存模型调用审计。但聊天历史 UI 主要读取最终文本，不把工具结果、公司画像、通知事件、快照和缺口组合成可折叠证据卡。
- `packages/app/src/components/chat-view.tsx` 与 `packages/app/src/lib/messages.ts` 目前把 assistant/scheduled 文本交给 Markdown 渲染，并特殊解析本地/OSS 图片 marker；除了图片/附件外，没有结构化的 source/evidence 展示面。
- Public `/portfolio` 已能显示投资主线与只读画像，Notifications/LLM Audit/Task Health 也有运维证据，但它们与某一条用户看到的回答之间缺少稳定回链。

现有提案已经覆盖相邻底座：`Source Provenance and Freshness Registry` 记录外部事实来源与时效；`Factual Snapshot Cache` 保存可复用工具结果；`Investment Output Safety Gate` 决定高风险输出能否送达；`Run Trace Workbench` 面向排障；`Evidence Review Queue` 管理市场证据是否进入长期画像复盘。这些都很重要，但它们仍没有定义一个面向终端用户的产品契约：**每条回答应该如何显示“我依据了什么、没依据什么、哪里需要你确认”。**

## 问题或机会

这是 P1。它不如 P0 安全门禁那样直接阻止错误送达，但会显著提升核心体验、信任、留存、支持效率和未来付费转化。投资助手最容易失去用户信任的时刻，往往不是模型完全失败，而是回答看起来很确定，却没有让用户快速判断依据是否充分。

主要问题：

1. **回答文本把证据压扁了。**  
   模型可能同时参考实时行情、搜索结果、公司画像、持仓、定时任务历史、用户上传附件和旧会话摘要。最终用户只能看到一段 Markdown，很难区分哪些是刚核验的事实、哪些是旧记忆、哪些是推理判断。

2. **来源和缺口没有一致的用户体验。**  
   有些回答会自然语言写“根据 FMP”，有些不写；有些工具失败会被模型总结成一句保守话，有些会被忽略。用户无法形成稳定预期，也不容易发现“本轮其实没有拿到最新价格”。

3. **管理端排障证据难回到用户语境。**  
   LLM audit、prompt audit、notifications、task-health 可以帮助工程师排查，但客服或运营要解释一条回答为什么这么说时，需要跨页面拼接。用户也无法自助打开“这条回答的证据”。

4. **多渠道主动推送更需要轻量证据。**  
   Feishu / Telegram / Discord / iMessage 的消息不能塞满内部日志，但主动提醒如果只发结论，会让用户很难判断该不该进入 Web 继续处理。需要短标签和深链，而不是长篇排障。

5. **未来提案需要统一展示出口。**  
   provenance、snapshot、safety verdict、evidence queue、research artifact、company portrait health 都会产生结构化信号。如果没有回答级 Evidence Cards，它们会各自在 UI 中添加局部标签，最终体验继续碎片化。

机会是新增 **Answer Evidence Cards**：一个回答级、用户可见、可折叠的证据摘要层。它不替代底层来源注册、缓存或安全门禁，而是消费这些信号，把每条回答的依据、数据时间、引用资产、缺口和下一步动作组织成稳定 UI。

## 方案概述

为每条 assistant / scheduled / public API answer 生成一个可选的 `AnswerEvidenceEnvelope`，并在 Web、desktop 和多渠道中按不同密度展示。

核心对象：

- `AnswerEvidenceEnvelope`：挂在最终消息或 run metadata 上，包含 answer id、actor/session、created_at、evidence cards、limitations、safety/provenance summary 和可用动作。
- `EvidenceCard`：一张用户可理解的证据卡，例如 `Market quote`、`Company portrait`、`Portfolio context`、`Search result`、`SEC filing`、`Notification event`、`User document`、`Tool failure`。
- `EvidenceCitation`：卡片内的引用项，保留 source label、retrieved_at、observed_at、freshness、title/url 或内部 asset ref。
- `AnswerLimitation`：本条回答明确的边界，例如 `no_realtime_quote`、`search_unavailable`、`profile_stale`、`portfolio_missing_cost`、`old_session_summary_used`、`generated_without_company_portrait`。
- `EvidenceAction`：从证据卡跳转或处理的动作，例如 `open_profile`、`refresh_quote`、`review_evidence`、`update_portfolio`、`view_run_trace`、`open_notification`。

第一版只做轻量、保守、可增量落地：

- 不要求模型自己写 citation JSON；由工具调用、known asset refs、finalizer / session metadata 和 existing route context 合成。
- 不把完整工具 payload 展示给普通用户，只展示摘要、时间、来源和缺口。
- Public chat 和 IM 默认展示 1-3 个最关键证据标签；Web/desktop 可展开完整卡片。
- 没有证据 envelope 的历史消息继续按普通 Markdown 渲染，不做迁移。

## 用户体验变化

### 用户端

- Public `/chat` 的 assistant 气泡下方增加一个紧凑 evidence strip：
  - `行情 21:01`、`画像: MU`、`持仓上下文`、`搜索失败`。
  - 点击后展开证据卡，显示来源、时间、是否过期、相关公司画像/持仓/通知事件。
- 对时间敏感回答，卡片优先显示数据时间和缺口：
  - `FMP quote fetched 2026-07-05 21:01 +0800`
  - `Tavily search unavailable; no latest news used`
  - `Company portrait last updated 2026-06-29`
- 当用户问“你为什么这么判断？”时，Hone 可以引用上一条 answer evidence，而不是重新让模型解释一遍。
- Public `/portfolio` 的投资主线旁可显示“最近支撑证据”：画像、最近 digest/event、最近回答证据卡；但不把运维内部字段暴露出来。

### 管理端

- Sessions 页面每条 assistant/scheduled message 可展开 `Evidence` tab：
  - 本轮工具/来源摘要、引用公司画像、portfolio 状态、search/news/quote 时间、limitations、safety verdict 摘要。
  - 能跳转到 LLM Audit、Run Trace、Notification event、Company Profile、Portfolio。
- Notifications 页面可把一条推送与它的 answer evidence 关联，帮助排查“这次提醒依据了哪些事件、是否用了 stale fallback”。
- LLM Audit 页面可反向显示“哪些用户可见回答消费了这条审计/工具结果”。
- Support 场景中，operator 可以把 evidence envelope 的脱敏摘要加入 redacted support bundle，而不需要复制完整 prompt 或 tool payload。

### 桌面端

- Desktop bundled/remote 复用 Web 证据卡组件。
- Dashboard 可显示最近回答的证据健康：`2 条回答缺实时行情`、`1 条使用过期画像`。
- Remote backend 模式明确显示 evidence 来自远端 backend，不把本地桌面缓存误认为事实来源。

### 多渠道

- IM 消息默认只增加短标签，不展开长卡：
  - `依据：FMP 21:01 / MU 画像 / 无最新搜索`
  - `依据不足：行情源失败，本轮不做最新价判断`
- 如果渠道支持卡片或按钮，提供 `Open evidence` 链接到 Web/desktop；不支持则保持短文本。
- 群聊中不显示个人持仓、成本、现金或私有画像细节；只显示公共事实来源或引导私聊查看个人证据。

## 技术方案

### 1. 数据模型

建议先在 session message metadata 中保存 envelope，后续再抽成独立 store。这样历史读取和 UI 展示可以随消息一起走，不需要第一版重构所有审计表。

```rust
pub struct AnswerEvidenceEnvelope {
    pub answer_id: String,
    pub actor: ActorIdentity,
    pub session_id: String,
    pub message_index: Option<u32>,
    pub created_at: String,
    pub cards: Vec<EvidenceCard>,
    pub limitations: Vec<AnswerLimitation>,
    pub safety_summary: Option<SafetySummaryRef>,
    pub run_trace_ref: Option<String>,
}

pub struct EvidenceCard {
    pub card_id: String,
    pub kind: EvidenceKind,
    pub title: String,
    pub summary: String,
    pub freshness: EvidenceFreshness,
    pub citations: Vec<EvidenceCitation>,
    pub actions: Vec<EvidenceAction>,
    pub visibility: EvidenceVisibility,
}
```

`EvidenceVisibility` 至少区分：

- `public_safe`
- `current_actor_private`
- `operator_only`
- `redacted_summary_only`

这样 Web public、admin、desktop、IM 可以按同一 envelope 做不同展示，不需要每个 surface 自己判断隐私。

### 2. Envelope 生成路径

第一版可以从四类已知材料合成，不依赖模型配合：

1. **工具调用结果**  
   从 `AgentResponse.tool_calls_made`、tool result messages、未来 `_hone_provenance` / `_hone_snapshot` 字段提取 provider、subject、retrieved_at、freshness 和 failure summary。

2. **长期资产引用**  
   从当前 actor 的 company profile sync、prompt context、`company_portrait` skill invocation metadata、portfolio tool result 中提取 asset ref。只展示摘要和更新时间，不暴露 sandbox path。

3. **运行/送达上下文**  
   对 scheduled output、notification、event-engine push，带上 job id、event id、digest window、delivery decision summary。若没有事件 ref，只显示 `scheduled task context`。

4. **Finalizer 与 safety 信号**  
   `response_finalizer` 已知道本地图表、fallback reason、sanitized empty、planning sentence suppression。后续 safety gate 落地后，把 verdict reason code 接入 `safety_summary`。

生成时机建议：

- `AgentSession::run()` finalizer 之后、message persistence 之前：生成 direct chat answer evidence。
- scheduler / event-engine sink 渲染后、写会话或发 channel 前：生成 scheduled / push evidence。
- public `/api/public/v1/chat/completions`：返回 OpenAI-compatible 主体时可在 `metadata.hone_evidence` 中带最小摘要；默认不破坏兼容客户端。

### 3. Session 与 API 兼容

- `memory/src/session.rs` 增加可选 message metadata 字段，例如 `evidence: Option<AnswerEvidenceEnvelope>`。
- 旧 session JSON / SQLite 行缺 evidence 时正常读取。
- `GET /api/history` 和 `/api/public/history` 默认返回 evidence summary；如果体积超限，可只返回 card list 和 lazy detail endpoint。
- 新增可选详情 API：
  - `GET /api/answers/:answer_id/evidence`
  - `GET /api/public/answers/:answer_id/evidence`
- Public API 必须基于当前 cookie/API key actor 校验 answer ownership；admin API 才能按 actor/session 查。

### 4. 前端组件

新增共享组件：

- `packages/app/src/components/answer-evidence.tsx`
- `packages/app/src/lib/answer-evidence.ts`
- `packages/app/src/lib/answer-evidence.test.ts`

展示规则：

- Message bubble 下方显示最多 3 个 evidence chips，优先级：freshness problem > market quote/search > company portrait > portfolio > run trace。
- 展开后按 card 分组，使用稳定 icon/label，不显示内部 tool name 作为主标题。
- 缺口用 neutral/warning 文案，不把“工具失败”包装成技术错误堆栈。
- Admin 视图可显示更多 refs，例如 audit id、trace id、event id；public 视图隐藏。

### 5. 与现有/未来提案衔接

- `Source Provenance and Freshness Registry`：提供更准确的 `EvidenceCitation` 和 `freshness`。
- `Factual Snapshot Cache`：提供 snapshot id、cache hit/stale fallback 和 replay bundle ref。
- `Investment Output Safety Gate`：提供 safety verdict summary 和用户态降级说明。
- `Run Trace Workbench`：从 evidence card 跳转到完整排障 trace。
- `Evidence Review Queue`：从 event/news evidence card 发起 `review_evidence`。
- `Research Artifact Library`：研究报告完成后可作为 evidence card 被回答引用。

## 实施步骤

### Phase 1: Message-level evidence envelope

- 定义 `AnswerEvidenceEnvelope` / `EvidenceCard` 类型，先放在 `hone-core` 或 `hone-channels` 可共享模块。
- 扩展 session message metadata，保持旧数据兼容。
- 从 `AgentResponse.tool_calls_made`、finalizer outcome、portfolio/company profile known refs 中生成基础 cards。
- `GET /api/history` 返回 evidence summary。

### Phase 2: Web/admin evidence UI

- 新增 `AnswerEvidence` 前端组件。
- Admin sessions 页面展示完整 evidence drawer。
- Public `/chat` 展示 evidence chips 和简化 drawer。
- 增加前端测试覆盖 visibility、freshness label、缺口文案和旧消息无 evidence 的降级。

### Phase 3: Scheduled/push and multi-channel

- scheduler / event-engine push 写会话时附 evidence envelope。
- Feishu / Telegram / Discord / iMessage 发送短 evidence label。
- Notifications 页面关联 evidence，支持从推送记录跳到回答证据。

### Phase 4: Provenance/snapshot/safety 接入

- 当 provenance registry 或 factual snapshot cache 落地后，把 observation/snapshot refs 合入 cards。
- 当 safety gate 落地后，把 verdict/limitation 合入 envelope。
- 增加 support bundle 脱敏导出。

## 验证方式

- Rust 单元测试：
  - 旧 session message 缺 evidence 仍可反序列化。
  - tool call result 能生成 market/search/tool-failure evidence card。
  - finalizer fallback reason 能生成 limitation，不暴露内部路径。
  - visibility 规则禁止 public actor 读取其它 actor 的 evidence。
- Web API 测试：
  - public history 只返回当前 actor 的 public/current_actor_private cards。
  - admin history 可看到 operator-only summary，但不返回 secret/token/raw payload。
  - OpenAI-compatible API 不因 `metadata.hone_evidence` 破坏基础响应格式。
- 前端测试：
  - `bun run test:web` 覆盖 evidence chip 排序、展开/收起、fresh/stale/failure labels、移动端换行。
  - 旧消息无 evidence 时 UI 不显示空容器。
- 手工验收：
  - Web chat 问一个需要 quote + company portrait 的问题，回答下方能看到行情时间、画像引用和缺口。
  - FMP key 缺失时，回答显示“未使用最新行情”证据缺口，而不是只在日志里失败。
  - 定时任务推送在 Notifications 里能打开对应 evidence。
  - IM 回复只显示短标签，不泄露个人持仓细节或 sandbox path。
- 指标：
  - 用户点击 evidence drawer 的比例。
  - “没有引用证据/事实来源不清楚”类反馈下降。
  - support 排查单条回答依据的平均时间下降。

## 风险与取舍

- 风险：证据卡过多，干扰聊天阅读。  
  取舍：默认只显示 1-3 个 chips，详情折叠；普通闲聊或低风险回答可以不显示。

- 风险：用户把 evidence card 误解为完整审计或保证正确。  
  取舍：卡片只表达“本轮用到的依据和限制”，不宣称事实绝对正确；对 stale/missing 明确标注。

- 风险：暴露私有资产或内部排障细节。  
  取舍：所有 card 带 visibility，public/IM 只显示当前 actor 安全摘要；admin/operator-only refs 不下发给 public API。

- 风险：过早依赖尚未落地的 provenance/snapshot/safety 提案。  
  取舍：第一版从现有 tool calls、finalizer outcome、known asset refs 合成；后续底座落地后再增强。

- 风险：模型生成文本和 evidence metadata 不一致。  
  取舍：evidence 由系统生成而不是完全信任模型；若文本声称“最新行情”但 evidence 显示 no realtime quote，未来交给 safety gate 拦截或降级。

- 不做的边界：不把完整第三方原文或工具 payload 展示给普通用户；不做通用文献管理器；不替代 research artifact library；不在第一版强制所有回答必须有 citations。

## 与已有提案的差异

本轮查重覆盖 `docs/proposal/` 与历史 `docs/proposals/`，并重点检索了 `source`、`provenance`、`freshness`、`snapshot`、`evidence`、`citation`、`answer`、`run trace`、`safety`、`research artifact` 等相关主题。

- 不重复 `auto_p1_source-provenance-freshness.md`：该提案记录外部事实进入系统时的 provider、endpoint、fallback 和 health；本提案定义最终回答如何把这些事实变成用户可见证据卡。
- 不重复 `auto_p1_factual_snapshot_cache.md`：snapshot cache 保存和复用工具 payload；Answer Evidence Cards 只引用 snapshot 摘要和时间，不承担缓存策略。
- 不重复 `auto_p0_investment_output_safety_gate.md`：safety gate 决定是否允许/降级/拦截输出；evidence cards 解释已经送达或可查看回答的依据与限制。
- 不重复 `auto_p1_run_trace_workbench.md`：run trace 面向工程排障全链路；evidence cards 是用户态、回答级、经过脱敏和折叠的信任层。
- 不重复 `auto_p1_evidence_review_queue.md`：evidence review queue 管理市场证据是否应更新长期画像；answer evidence cards 管理单条回答引用了哪些依据，可把某张市场证据卡转入 review queue。
- 不重复 `auto_p1_research_artifact_library.md`：artifact library 保存深度研究交付物；本提案允许回答引用 artifact，但不定义报告存储。

差异结论：已有提案已经覆盖来源记录、事实缓存、安全门禁、运行排障和研究证据生命周期，但缺少一层“每条用户可见回答的证据呈现契约”。本提案小而可落地，可以作为这些底座的共同前端出口，也能在底座完全落地前先改善用户信任体验。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md` 或运行配置。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/answer-evidence-cards.md`，并在改变 session message schema、Web/API history payload、channel outbound 文案或 answer metadata 合约时同步更新 repo map、invariants、相关 runbook 与必要 ADR。
