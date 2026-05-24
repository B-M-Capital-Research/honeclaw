# Proposal: Factual Snapshot Cache for Tool Results and Replayable Research

status: proposed
priority: P1
created_at: 2026-05-24 20:04:42 +0800
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
- `docs/proposal/auto_p2_signal-source-lab.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_prompt-context-budget-inspector.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-tools/src/base.rs`
- `crates/hone-tools/src/registry.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/prompt_audit.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-event-engine/src/fmp.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-event-engine/src/pollers/{price,news,earnings,sec_enrichment}.rs`
- `crates/hone-event-engine/src/global_digest/{fetcher,collector,curator}.rs`
- `memory/src/llm_audit.rs`
- `config.example.yaml`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/pages/notifications.tsx`

## 背景与现状

Hone 的投研回答和主动推送高度依赖“事实工具”：

- `crates/hone-tools/src/data_fetch.rs` 的 `DataFetchTool` 直接调用 FMP，支持 `quote`、`profile`、`snapshot`、`financials`、`news`、`earnings_calendar` 等数据类型，并用 config key pool 做多 key fallback。
- `crates/hone-tools/src/web_search.rs` 的 `WebSearchTool` 直接调用 Tavily，支持多 key fallback、额度/鉴权错误分类和敏感错误脱敏。
- `snapshot` 当前会连续拉 `quote + profile + news`，如果部分失败就在同一个返回 JSON 里放 `errors`；它没有把成功子结果沉淀成可复用事实快照。
- event-engine 也有自己的 FMP client 和 pollers：`PricePoller` 批量拉 quote，`NewsPoller` 拉 FMP stock news，SEC/earnings/global digest 路径也会读取外部事实，再写入 `EventStore` 或 digest 中间态。
- `EventStore` 对 `MarketEvent` 做去重和 JSONL 镜像，但它记录的是已成形事件，不是所有工具请求的原始或半结构化结果。
- `memory/src/llm_audit.rs` 能记录模型调用，`prompt_audit` 能记录 prompt，但外部事实工具的返回结果目前没有同级的“可复用快照”对象。

现有提案已经覆盖相邻问题：`Source Provenance and Freshness Registry` 记录来源、时效和健康；`Signal Source Lab` 管理事件源上线；`Model Route Evaluation Lab` 比较模型路线；`User Journey Replay Lab` 用 fake runner 回放产品旅程；`Usage Entitlement Ledger` 记录用量与成本。但代码仍缺一层非常实际的运行能力：同一 actor、同一 ticker、同一时间窗口里，聊天、定时任务、digest、event-engine 和后续回放是否能共享同一份事实快照，而不是重复打 FMP/Tavily、拿到彼此不一致的“最新事实”。

## 问题或机会

这是 P1 级机会，因为它影响回答一致性、外部 API 成本、回归可复现性和用户对“最新事实”的信任。

1. **同一轮研究可能重复拉相同事实。**  
   一个用户问 NVDA，`snapshot` 拉 quote/profile/news；随后 skill、position advice、company portrait 或 chart 可能再次拉 quote/news。event-engine 或 digest 同时也可能在附近窗口拉同一 ticker。当前没有共享事实快照，成本和延迟都会放大。

2. **“最新”可能在同一体验内漂移。**  
   用户在 Web 看到一个价格，IM 推送又引用另一个价格，后续复盘时 prompt audit 里只剩模型输入文本。没有工具结果快照时，很难说明当时到底依据哪一份 quote/search/news。

3. **失败后的降级不可控。**  
   FMP/Tavily key pool 都能 fallback，但如果 provider 短时间抖动，系统要么再次失败，要么让模型在缺事实下保守回答。对 quote/news/search 这类可短期复用的数据，系统应能在明确标注 stale/fallback 的前提下使用最近快照，而不是每次从零开始。

4. **回放和评估缺少真实工具结果底座。**  
   Model Lab、Journey Replay、Run Trace 都需要复现一次回答或 digest。fake fixture 能验证状态机，但无法复现“当时真实 FMP/Tavily 返回了什么”。保存可脱敏、带 TTL 的事实快照，可以让 bugfix 和模型评估基于同一组事实输入。

5. **来源健康与事实缓存职责不同但需要衔接。**  
   Provenance 解释来源和 freshness；它不一定保存可复用 payload。Cache 需要保存受控 payload、命中策略、过期规则和回放引用。两者如果混在一起，会让来源审计表变成大对象存储；如果完全没有 cache，又会让 provenance 只能指向“曾经访问过”，不能复用事实。

6. **商业化会被外部数据成本卡住。**  
   Public chat、Hone Cloud API、scheduled tasks、global digest 和未来 share brief 都会消耗事实数据源。只靠 LLM token usage 不能控制成本，事实工具也需要可解释的 cache hit / miss / stale fallback 指标。

## 方案概述

新增 **Factual Snapshot Cache**：一个按工具请求归一化、带 freshness policy、可回放、可观测的事实快照层。它位于 `data_fetch` / `web_search` / event-engine poller 与外部 provider 之间，保存有限期、可脱敏的工具结果，并为后续回答、推送、评估和排障提供稳定引用。

核心对象：

- `FactualSnapshot`
  一次成功或可用的事实结果。包含 snapshot id、provider、tool name、normalized request key、subject、payload path/hash、fetched_at、observed_at、expires_at、freshness class、source observation ids、redaction level。

- `SnapshotRequestKey`
  归一化请求键。例如 `fmp:quote:AAPL`、`fmp:snapshot:AAPL:quote_profile_news`、`tavily:search:sha256(query+max_results+depth)`、`fmp:earnings_calendar:2026-05-24..2026-06-07`。

- `SnapshotPolicy`
  每类事实的 TTL 与 stale fallback 规则：quote 几分钟，market profile 几天，financials 数天到数周，news/search 数小时，earnings calendar 日内或按窗口，SEC filing 长期有效但 summary 可过期。

- `SnapshotUse`
  某次 run、task、digest 或 evaluation 使用了哪份 snapshot，是 `fresh_hit`、`stale_fallback`、`miss_fetch_success`、`miss_fetch_failed` 还是 `bypass_cache`。

- `SnapshotReplayBundle`
  把一次 run 或一组 eval 所需的工具结果冻结成可带走的 fixture，供 bug 回放、模型评估和 release confidence 使用。

第一版不做全局“大数据湖”，也不把所有第三方原文无限保存。它只缓存高频、结构化、可明确 TTL 的事实工具结果，并严格限制大小、保留期和敏感字段。

## 用户体验变化

### 用户端

- 聊天回答中可更稳定地说明数据时间：“行情快照获取于 20:02，若需要实时盘中判断请重新刷新。”
- 当 FMP/Tavily 暂时不可用时，Hone 可以明确降级：“实时源暂不可用，本轮只使用 18 分钟前的行情快照，不做最新价结论。”
- Public `/portfolio`、digest 和 chat 对同一 ticker 的短时间引用更一致，减少“刚才说的价格和现在不一样但没有解释”的体验。
- 用户不需要理解 cache，但会看到更清楚的新鲜度和降级文案。

### 管理端

- 在 `Task Health`、`Notifications`、`LLM Audit` 或未来 `Run Trace` 详情中显示本次用到的 factual snapshots：
  - provider、endpoint、subject、fetched_at、fresh/stale、cache hit/miss。
  - fallback 到旧快照的原因。
  - payload hash 与可选脱敏摘要。
- Settings 或诊断页展示最近 24h 的事实工具 cache 指标：FMP/Tavily 命中率、miss 成功率、stale fallback 次数、provider failure 后被 cache 缓冲的次数。
- 管理员可以按 provider/endpoint 清理缓存或导出一组 redacted replay bundle，用于复现问题。

### 桌面端

- Desktop bundled 模式可以离线查看最近缓存快照和“当前事实源是否过期”，尤其适合本地网络或 provider key 不稳定时排障。
- Remote mode 只显示远端 backend 的 cache 状态，不把本地缓存误认为远端事实来源。
- 本地用户可在设置里选择更保守的 cache retention；public/remote 部署默认短 TTL 和容量上限。

### 多渠道

- Feishu / Telegram / Discord 回复只暴露轻量说明，例如“使用 20:02 的行情快照”或“搜索源不可用，未使用旧新闻快照”。
- 主动推送如果用了 stale fallback，文案必须降低确定性，不把旧事实包装成实时触发。
- 群聊不展示 provider key、内部 cache path 或完整 payload，只展示数据时间和是否降级。

## 技术方案

### 1. Snapshot store

建议在 `memory` 新增 `factual_snapshot.rs`，使用 SQLite 存 metadata，payload 写到受控文件目录或压缩 JSON blob。第一版优先 SQLite metadata + 文件 payload，避免把大 JSON 压进单表。

```text
factual_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  tool_name TEXT NOT NULL,
  request_key TEXT NOT NULL,
  subject TEXT,
  payload_sha256 TEXT NOT NULL,
  payload_path TEXT,
  payload_bytes INTEGER NOT NULL,
  status TEXT NOT NULL,
  freshness_class TEXT NOT NULL,
  fetched_at_ts INTEGER NOT NULL,
  observed_at_ts INTEGER NOT NULL,
  expires_at_ts INTEGER NOT NULL,
  source_observation_ids_json TEXT NOT NULL,
  metadata_json TEXT NOT NULL
);

factual_snapshot_uses (
  use_id TEXT PRIMARY KEY,
  snapshot_id TEXT,
  request_key TEXT NOT NULL,
  actor_key TEXT,
  session_id TEXT,
  trace_id TEXT,
  task_id TEXT,
  origin TEXT NOT NULL,
  use_kind TEXT NOT NULL,
  created_at_ts INTEGER NOT NULL,
  detail_json TEXT NOT NULL
);
```

`request_key` 建唯一索引时不要只保留一条记录。建议按 `(request_key, fetched_at_ts)` 保留历史，查询时取未过期最新；这样 run replay 能引用旧 snapshot，日常路径也能拿最新快照。

### 2. Policy registry

在 `hone-core` 或 `memory` 定义事实类型策略：

```rust
pub struct SnapshotPolicy {
    pub request_kind: &'static str,
    pub fresh_ttl_secs: i64,
    pub stale_fallback_ttl_secs: i64,
    pub max_payload_bytes: usize,
    pub allow_stale_for_chat: bool,
    pub allow_stale_for_direct_push: bool,
    pub redact_payload_for_export: bool,
}
```

建议初始策略：

- `fmp.quote`: fresh 2-5 分钟，stale fallback 不超过 20 分钟；主动 price alert 不允许 stale fallback 触发。
- `fmp.profile`: fresh 7 天，stale 30 天。
- `fmp.financials`: fresh 1 天到 7 天，视 endpoint。
- `fmp.news`: fresh 1-3 小时，stale 24 小时；回答必须标注时间。
- `tavily.search`: fresh 1 小时，stale 12 小时；时间敏感宏观问题默认不允许 stale。
- `fmp.earnings_calendar`: fresh 当日，stale 3 天；接近财报日期时要求刷新。

这些策略要和 `docs/invariants.md` 的时间敏感分析约束保持一致：不能因为有 cache 就把旧数据说成“最新”。

### 3. Tool wrapper 而不是重写工具

不要把 cache 逻辑散进每个 tool。建议新增 `CachedTool<T>` 或在 `ToolRegistry` 创建工具时包一层：

1. 根据 tool name + args 生成 `SnapshotRequestKey`。
2. 查找未过期 snapshot。
3. 命中则返回原 payload，并附加 `_hone_snapshot` metadata。
4. 未命中则调用底层 tool。
5. 成功后按 policy 写 snapshot。
6. 失败时根据 policy 决定是否返回 stale fallback，且必须附加 `stale_fallback=true` 与 warning。

返回兼容示例：

```json
{
  "data_type": "quote",
  "ticker": "AAPL",
  "data": [...],
  "_hone_snapshot": {
    "snapshot_id": "snap_...",
    "request_key": "fmp:quote:AAPL",
    "fetched_at": "2026-05-24T20:02:11+08:00",
    "cache_use": "fresh_hit",
    "freshness": "fresh"
  }
}
```

对 `snapshot` 聚合结果，建议同时缓存子请求和聚合结果：

- `fmp:quote:AAPL`
- `fmp:profile:AAPL`
- `fmp:news:AAPL`
- `fmp:snapshot:AAPL`

这样后续单独请求 quote/profile/news 可以复用，不必只能命中整包。

### 4. Event-engine integration

event-engine 不应直接依赖聊天工具 wrapper，但可共享同一 `FactualSnapshotStore` 与 policy：

- `PricePoller` 批量 quote 成功后写 `fmp.quote_batch` snapshot，并可拆分 per-symbol quote snapshot。
- `NewsPoller` 写 `fmp.news` snapshot，同时继续把成形事件写 `EventStore`。
- SEC/earnings/global digest fetcher 对原始响应写 snapshot，对成形事件/摘要继续走现有 store。
- Direct push 决策默认不允许 stale snapshot 触发新的 high-severity 事件；stale 只用于解释、digest 或排障，除非策略显式允许。

这能让 chat 与 event-engine 共享“近期事实”而不互相调用对方内部 API。

### 5. Replay bundle

为模型评估和 bug 复现提供一个只读导出：

```text
snapshot-replay/
  manifest.json
  snapshots/
    snap_...json
  redaction_report.json
```

`manifest.json` 记录 request key、tool args hash、payload hash、fetched_at、policy 和使用场景。默认导出 redacted payload；完整 payload 仅本地 admin 显式选择并遵守 Data Trust / Support Bundle 规则。

Model Lab 可以把 replay bundle 作为 fixed tool result 输入，比较不同模型在相同事实下的回答质量；Journey Replay 可以用它替代 fake data source 的一部分。

### 6. API 与 UI

Admin API：

- `GET /api/factual-snapshots?provider=&tool=&subject=&freshness=&from=&to=`
- `GET /api/factual-snapshots/:id`
- `POST /api/factual-snapshots/purge`
- `POST /api/factual-snapshots/replay-bundle`
- `GET /api/factual-snapshots/stats`

Public API 第一版不开放原始 snapshot 列表，只在 chat/history/digest payload 中返回用户可见的 fetched time 和 freshness warning。

前端落点：

- `LLM Audit` / future `Run Trace` detail：展示工具事实快照。
- `Task Health` / `Notifications` detail：显示推送引用的 snapshot 和是否 stale。
- `Settings` 或 diagnostics：显示 cache size、hit rate、purge action。

## 实施步骤

### Phase 1: 只读 snapshot store 与 `data_fetch` wrapper

- 新增 `FactualSnapshotStore`、policy 类型、request key 规范和单元测试。
- 给 `data_fetch` 增加 wrapper，覆盖 `quote/profile/news/financials/earnings_calendar/snapshot`。
- 返回 `_hone_snapshot` metadata，但保持现有业务 JSON 不变。
- 默认只对 chat/session path 启用，event-engine 暂不接入。

### Phase 2: `web_search` 与诊断可见性

- 接入 Tavily search snapshot，按 query hash 和 max_results 建 key。
- 增加 admin stats API，展示 hit/miss/stale fallback。
- 在 LLM audit 或 prompt audit metadata 中记录本轮 snapshot ids。
- 增加 cache purge 和容量上限，避免长期膨胀。

### Phase 3: Event-engine 共享事实缓存

- 让 price/news/earnings/SEC pollers 写 snapshot metadata。
- Direct push 禁止 stale snapshot 产生新 high-severity 触发；digest 可显示 stale warning。
- Notifications detail 增加 snapshot refs，帮助解释某次推送依据。

### Phase 4: Replay bundle 与评估联动

- 增加 redacted replay bundle 导出。
- Model Lab 支持使用 replay bundle 固定工具结果。
- Journey Replay 可使用 snapshot fixture 验证真实工具结果结构。
- Run Trace 落地后，把 snapshot ids 纳入 trace timeline。

## 验证方式

- Rust 单元测试：
  - request key normalization：ticker 大小写、symbol/ticker alias、earnings date window、Tavily query hash 稳定。
  - policy 判断：fresh hit、expired miss、allowed stale fallback、disallowed stale direct push。
  - payload hash 与 metadata 序列化稳定。
  - `data_fetch(snapshot)` 成功时同时写子 snapshot 与聚合 snapshot。
  - provider 失败时，若 stale fallback 被允许，返回 payload 但带 `_hone_snapshot.cache_use=stale_fallback` 和 warning。
- Web/API 测试：
  - admin stats API 不返回敏感 provider key 或本机绝对 path。
  - purge 只删除过期或指定 policy 范围，不破坏被 replay bundle 引用的 snapshot metadata。
- 回归脚本：
  - 用 mock FMP/Tavily server 连续发两次相同请求，第二次必须 cache hit，且 mock server 请求计数不增加。
  - 模拟 provider 短暂失败，chat path 可使用允许的旧 snapshot，direct price alert path 不触发新推送。
- 手工验收：
  - Public chat 显示数据时间和 stale warning。
  - Admin 能在一次失败回答或通知详情中看到引用的 snapshot id。
  - Model Lab 使用同一 replay bundle 比较两个候选模型时，工具事实输入保持一致。

## 风险与取舍

- **风险：旧数据被误当成实时事实。**  
  取舍：policy 默认保守；time-sensitive prompt 必须标注 fetched_at；direct high-severity push 默认不允许 stale fallback。

- **风险：缓存第三方 payload 增加版权、隐私和存储压力。**  
  取舍：只缓存必要 JSON，不保存完整网页 raw content；设置 TTL、容量上限、payload hash、redacted export；长期 retention 交给 storage lifecycle 提案。

- **风险：cache 与 provenance 职责混淆。**  
  取舍：cache 保存可复用 payload 和命中记录；provenance 保存来源、健康和血缘摘要。两者通过 snapshot/source observation id 互相引用，不合并成一个巨表。

- **风险：工具 wrapper 改变模型看到的 JSON。**  
  取舍：只追加 `_hone_snapshot` metadata，不改现有 `data`、`error`、`errors` 字段；如果某 runner 对额外字段敏感，可通过 config 禁用 metadata 注入但仍记录 use。

- **风险：缓存隐藏 provider 问题。**  
  取舍：stale fallback 必须写 `SnapshotUse` 和 warning，readiness/source health 仍显示 provider 当前失败；cache 不能把长期不可用伪装成 healthy。

- **不做的边界：**
  - 不做实时行情数据库。
  - 不做无限历史行情回测。
  - 不替代 `EventStore`、`Source Provenance`、`LLM Audit` 或 `Run Trace`。
  - 不在第一版缓存用户上传文档全文；文档缓存归 Document Inbox / storage lifecycle。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/`，重点比对了 source provenance、signal source lab、model route evaluation、journey replay、prompt budget、usage entitlement、storage lifecycle、redacted support bundle、run trace、investment document inbox 和 event-engine delivery 相关提案。

- 不重复 `auto_p1_source-provenance-freshness.md`：该提案回答“事实来自哪里、是否新鲜、provider 是否健康”；本提案回答“可否复用同一份事实 payload、如何命中/降级/回放”。
- 不重复 `auto_p2_signal-source-lab.md`：Signal Source Lab 管理 RSS/Telegram 等事件源上线前 probe 和 trial；本提案管理已执行工具请求的结果快照。
- 不重复 `auto_p1_model-route-evaluation-lab.md`：Model Lab 评估模型输出质量；本提案提供固定事实输入，让 Model Lab 的比较更可复现。
- 不重复 `auto_p1_user-journey-replay-lab.md`：Journey Replay 验证产品状态机；本提案提供真实工具结果 replay bundle，可被 Replay Lab 消费。
- 不重复 `auto_p1_prompt-context-budget-inspector.md`：Prompt Budget 观测上下文大小；本提案减少重复事实调用并让工具结果可引用。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：Entitlement 记录权益和用量；本提案可为其提供事实工具 cache hit/miss 成本信号，但不定义商业 plan。

本轮只创建 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续执行本提案，应按动态计划准入标准新增或复用 `docs/current-plans/factual-snapshot-cache.md`，并在改变工具 registry、event-engine poller、admin API 或缓存 retention 规则时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
