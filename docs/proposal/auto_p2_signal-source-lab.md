# Proposal: Signal Source Lab for Event-Engine Source Lifecycle

status: proposed
priority: P2
created_at: 2026-05-17 14:03:08 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_temporal-operations-calendar.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `config.example.yaml`
- `crates/hone-core/src/config/event_engine.rs`
- `crates/hone-core/src/config/mutation.rs`
- `crates/hone-event-engine/src/source.rs`
- `crates/hone-event-engine/src/spawner.rs`
- `crates/hone-event-engine/src/pollers/rss.rs`
- `crates/hone-event-engine/src/pollers/social/telegram_channel.rs`
- `crates/hone-event-engine/src/router/{classify,dispatch,policy,stats}.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `packages/app/src/pages/{notifications,task-health,settings}.tsx`
- `packages/app/src/lib/admin-content/{notifications,task-health,settings}.ts`

## 背景与现状

Hone 的事件引擎已经从单一 FMP 拉取扩展到更多外部信号：

- `EventSource` trait 统一了 FMP、RSS、Telegram 公开频道等来源的 `name()`、`schedule()` 和 `poll()`。
- `spawn_event_source` 负责冷启动拉取、FixedInterval / CronAligned 调度、poll timeout、失败观测和重试。
- `RssNewsPoller` 会把 RSS 2.0 feed 转成 `MarketEvent`，并通过 `source_class="trusted"`、title 级 ticker alias 和 FMP-like payload 复用 global digest / router 链路。
- `TelegramChannelPoller` 通过 `https://t.me/s/<handle>` 抓公开频道预览，产出 `EventKind::SocialPost`，默认 `source_class="uncertain"`，交给 LLM news classifier 判断重要性。
- `config.example.yaml` 已经有 `event_engine.sources.rss_feeds` 与 `telegram_channels` 配置说明。
- `event_engine_admin.rs` 已提供 RSS feed list/create/update/delete API，并通过 `apply_overlay_mutations` 写到 `<config>.overrides.yaml`；响应会标出 `needs_restart=true`，因为 scheduler/RSS 子树不是热生效。
- 管理端已有 notifications、task-health、settings、logs 等页面，但还没有一个专门面向“新增/评估/启用/回滚事件源”的工作台。

这说明底层接入能力已经存在，但事件源的产品生命周期仍然偏运维手工：管理员要知道 feed URL 或 Telegram handle，直接写入配置，重启后再从事件表、通知日志和模型判断结果里观察效果。对投资助手来说，新增一个来源不只是“能不能抓到”，还要回答它是否高信噪、是否会轰炸用户、是否与持仓/画像相关、是否应进入 immediate、digest 还是仅候选池。

## 问题或机会

当前缺口不是来源健康，而是来源上线前后的治理：

1. **写配置前缺少可见样本。**  
   RSS CRUD 只能校验 handle、URL 和 interval；Telegram poller 也有解析逻辑，但没有 admin API 让操作员先拉一次、看最近样本、确认 ticker 抽取、source class 和可能事件数量。

2. **新增来源的噪音成本不可预估。**  
   一个 RSS feed 或 Telegram 频道可能每小时产出几十条低价值内容。当前只能重启后让 router、classifier、digest 真实消耗成本，再从 logs 和 notifications 里人工判断。

3. **配置改动与 runtime 生效之间存在断点。**  
   `event_engine_admin` 清楚返回 `needs_restart=true`，但管理端缺少“待生效变更 / 当前运行中来源 / 重启后将启用来源”的差异视图。用户可能以为保存后立即生效。

4. **RSS 和 Telegram 来源没有统一的 lifecycle 状态。**  
   `rss_feeds` 有 CRUD；`telegram_channels` 在配置中存在，poller 也已实现，但当前没有同等 API 和 UI 管理面。未来新增 SEC/RSS/social/source catalog 时会继续扩散。

5. **来源上线缺少回滚和试运行语义。**  
   如果新增来源导致 digest 噪音、LLM classifier 成本上涨或大量 uncertain social post，管理员需要手动删配置、重启、再观察。没有 “trial source”、“dry run only”、“promote to active” 的渐进路径。

这个主题适合 P2：它不会像 output safety、delivery decision 或 runtime readiness 那样直接决定核心可用性，但能显著提高事件源扩展质量，降低噪音、成本和运维误判。随着 Hone 从个人工具走向持续监控服务，事件源治理会成为增长和可靠性之间的重要产品层。

## 方案概述

新增 **Signal Source Lab**：一个面向管理员和高级本地用户的事件源生命周期工作台，用于发现、测试、预览、试运行、启用、停用和回滚 RSS / Telegram / 未来信号源。

核心对象：

- `SignalSourceCandidate`  
  尚未启用的来源草稿，包含 source_kind、handle、URL/telegram handle、interval、默认 source_class、备注、创建者和测试结果。

- `SourceProbeResult`  
  单次抓取测试结果：HTTP 状态、解析出的 item/post 数量、样本标题、occurred_at、URL、symbols、payload class、失败原因、耗时、是否可能触发 classifier。

- `SourceImpactPreview`  
  把样本事件送入轻量模拟：按当前 subscriptions / portfolio / prefs / router policy 估算 `would_insert`、`would_classify`、`would_digest`、`would_direct_send`、`would_filter`，但不写入正式事件表、不投递消息。

- `SourceLifecycleState`  
  `draft`、`tested`、`trial_dry_run`、`active_pending_restart`、`active`、`paused_pending_restart`、`paused`、`failed_probe`、`archived`。

- `SourceChangePlan`  
  配置 diff 和生效计划：将写入哪些 `event_engine.sources.*` overlay、是否需要重启、影响哪些 poller task、如何回滚。

第一版不做自动来源推荐，也不接外部目录服务。它只把已有 RSS/Telegram 接入能力变成可操作的 source lifecycle。

## 用户体验变化

### 用户端

- 普通 public 用户不直接看到 Signal Source Lab。
- 事件卡片、digest 或未来 provenance label 可在后台启用后显示更清晰的来源名，例如 `rss:bloomberg_markets` 或 `telegram.watcherguru`。
- 如果管理员把一个来源先放在 trial/digest-only，用户会少收到未经验证的新来源噪音。

### 管理端

- 新增 `Signal Sources` 页面或在 Settings/Event Engine 下新增 tab：
  - 当前 active 来源：FMP 内置 pollers、RSS feeds、Telegram channels。
  - Draft / trial 来源：最近 probe、样本、impact preview、待重启状态。
  - 新增来源向导：输入 RSS URL 或 Telegram handle，先 `Probe`，再 `Preview impact`，最后 `Enable`.
- Probe 后直接展示样本：
  - title/text preview、URL、发布时间、抽取到的 ticker/cashtag。
  - `trusted` / `uncertain` / `legal_ad_template` 等 payload 关键信号。
  - 预计是否需要 LLM classifier，以及可能进入 digest 的数量。
- 保存配置后，页面显示 `pending restart`，并列出“当前运行中配置”和“重启后配置”的差异。
- 对上线后噪音偏高的来源，可以一键 `Pause` 或 `Rollback to previous source set`，生成明确 overlay diff。

### 桌面端

- Desktop bundled 模式复用同一管理页。
- 当用户保存来源后，桌面端可以提示“事件源改动需要重启后台进程”，并调用已有 backend lifecycle 重启能力或引导用户手动重启。
- Remote desktop mode 只展示远端 backend 的来源状态，不尝试本地重启。

### 多渠道

- 新来源默认不应直接进入 IM immediate 推送。建议第一版启用为 digest/trial 或按现有 router policy 评估后再放开。
- 对 Telegram/social 来源，群聊消息不暴露后台测试细节；只在推送文本里保留短来源标签。
- 若新增来源造成 channel send failed 或 filtered 激增，仍由 notifications / delivery decision 视图承接排障。

## 技术方案

### 1. Source probe API

在 `crates/hone-web-api/src/routes/event_engine_admin.rs` 增加只读/试运行接口：

- `POST /api/event-engine/sources/probe`
- `POST /api/event-engine/sources/impact-preview`
- `GET /api/event-engine/sources/runtime`
- `GET /api/event-engine/sources/pending-diff`

`probe` 输入：

```json
{
  "kind": "rss",
  "handle": "bloomberg_markets",
  "url": "https://...",
  "interval_secs": 1800
}
```

或：

```json
{
  "kind": "telegram_channel",
  "handle": "watcherguru",
  "extract_cashtags": true,
  "interval_secs": 900
}
```

实现上直接复用 `RssNewsPoller::poll()`、`parse_rss_2`、`TelegramChannelPoller::new()` 和 `parse_telegram_preview`，但 probe 不调用 `EventStore::insert_event`，不进入 router，不投递。

### 2. Impact preview

Impact preview 不需要第一版做到完全等价真实 dispatch。建议分两层：

- `static_preview`：样本数、symbols、source_class、severity、是否会触发 LLM classifier、是否可进入 global digest candidate pool。
- `router_preview`：可选读取最近 actor/subscription/prefs，调用纯函数或新增 dry-run wrapper，估算 would_send / would_digest / would_filter。若 router 当前没有可复用纯函数，第一版先只返回 static preview，并明确标为 approximate。

关键是不写正式事件表、不触发 sink、不消耗真实推送配额。LLM classifier preview 默认关闭；需要管理员手动打开并显示成本警告。

### 3. 来源配置统一模型

保留 `event_engine.sources.rss_feeds` 和 `telegram_channels` 现有配置结构，不引入第二套真相源。Signal Source Lab 的 draft/trial 状态可以放在 runtime 辅助文件或 SQLite：

```text
signal_source_candidates (
  candidate_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  handle TEXT NOT NULL,
  config_json TEXT NOT NULL,
  lifecycle_state TEXT NOT NULL,
  last_probe_json TEXT,
  last_impact_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

真正启用时仍通过 `apply_overlay_mutations` 写入 canonical overlay：

- RSS：`event_engine.sources.rss_feeds`
- Telegram：`event_engine.sources.telegram_channels`

这样不破坏 `config.yaml` 作为长期用户配置源，也保留现有重启语义。

### 4. Pending restart 与 runtime source view

当前 API 写 RSS 后返回 `needs_restart=true`，但没有 runtime diff。建议新增：

- `configured_sources`: base + overlay 合并后的来源列表。
- `running_sources`: web-api 启动时实际 spawn 的 source names，可由 `spawn_event_source` 注册到内存 registry，或从 task observer 最近 `poller.*` 记录推断。
- `pending_changes`: configured 与 running 的差异。

这能在 UI 里明确显示：

- `rss:foo` 已保存但未运行。
- `telegram.bar` 已从配置删除但旧进程仍运行到下次重启。
- interval 已修改但当前 poller 仍按旧 interval 运行。

### 5. Trial / dry-run 模式

如果要让 trial 来源持续运行但不投递，可新增配置层：

```yaml
event_engine:
  sources:
    source_modes:
      rss:foo: dry_run
      telegram.watcherguru: digest_only
```

第一版可以不落这个配置，先用 manual probe + impact preview；第二阶段再做持续 dry-run。若实现 dry-run，需要在 `pipeline::run_once` 或 router 前插入 mode policy：

- `dry_run`: poll + parse + task run 记录，不 insert event，不 dispatch。
- `record_only`: insert event，但不 dispatch。
- `digest_only`: 允许进入 digest buffer，禁止 immediate sink。
- `active`: 当前行为。

### 6. 前端工作台

新增前端模块：

- `packages/app/src/lib/signal-sources.ts`
- `packages/app/src/pages/signal-sources.tsx` 或 Settings 子页
- `packages/app/src/lib/admin-content/signal-sources.ts`

UI 结构：

- Source table：kind、handle、mode、interval、last probe、running/pending 状态。
- Probe drawer：输入、样本、错误、impact preview。
- Change plan：将写入 overlay 的 diff、needs restart、rollback action。

### 7. 兼容策略

- 旧配置里的 RSS feeds 和 Telegram channels 自动显示为 `active` 或 `active_pending_restart`，不要求迁移。
- 旧事件没有 source lifecycle metadata 时仍按 `source` 字符串展示。
- 如果 Telegram web preview 被改版，probe 应返回 `failed_probe`，不影响已有 active source 的下一 tick 重试行为。
- 不改变 `source_class`、router、classifier、digest 的现有判定；Signal Source Lab 只是上线前/上线中的控制面。

## 实施步骤

### Phase 1: Manual probe and sample preview

1. 在 Web API 增加 `sources/probe`，支持 RSS 和 Telegram。
2. 抽出 probe response 类型，覆盖 HTTP 失败、parse 失败、空 feed、正常样本。
3. 管理端新增最小 Source Lab 页面：输入来源、运行 probe、查看样本。
4. 不写配置，不改变 runtime 行为。

### Phase 2: Enable plan and pending restart view

1. 把现有 RSS CRUD 包装成 `SourceChangePlan`，保留原 API 兼容。
2. 为 Telegram channel 增加对等 CRUD 或 source-generic upsert。
3. 增加 configured vs running diff，明确 `needs_restart`。
4. UI 支持 enable / pause / delete，并展示 rollback diff。

### Phase 3: Impact preview

1. 增加 static impact preview：样本数、symbols、severity、source_class、classifier 需求。
2. 若 router 逻辑可安全复用，再加 approximate router preview。
3. 将 preview 结果保存到 candidate，用于后续审计。
4. 对高噪音来源给出 guardrail，例如“先 digest-only 试运行”。

### Phase 4: Trial modes

1. 增加 source mode policy：dry_run / record_only / digest_only / active。
2. 让 `pipeline::run_once` 或 router 分支尊重 source mode。
3. notifications/task-health 展示 dry-run 统计。
4. 达到质量阈值后允许从 trial promote 到 active。

## 验证方式

- 单元测试：
  - RSS probe 能解析正常 RSS、空 RSS、非法 URL、非 2xx、非法 XML。
  - Telegram probe 能解析 fixture HTML、跳过空文本 post、抽取 cashtag、处理 HTTP 失败。
  - `SourceChangePlan` 对 create/update/delete 生成正确 overlay diff 和 rollback diff。
  - configured/running diff 能识别新增、删除、interval 修改和未重启状态。
- Web API 测试：
  - `sources/probe` 不写入 `events` 表，不触发 router/sink。
  - RSS/Telegram invalid handle 返回 400，不产生候选。
  - source config 写入仍通过 `apply_overlay_mutations`，不直接覆盖 `config.yaml`。
- 前端验证：
  - Probe drawer 在 success/empty/error 三类状态下不溢出。
  - Pending restart badge 在 enable/delete 后出现，重启后消失。
  - 旧后端缺少 source lab API 时 Settings graceful degrade。
- 手工验收：
  - 用一个有效 RSS URL probe，能看到样本但事件表无新增记录。
  - 启用该 RSS 后页面提示需要重启；重启后 source 出现在 running list。
  - 用 Telegram 公开频道 probe，能看到 `source_class=uncertain` 和是否抽取 cashtag。
  - 删除或 pause 来源后可看到 rollback plan。
- 指标：
  - 新增来源后 7 日内 filtered/noise 比例下降。
  - 新来源导致的 send failed / digest flood 支持问题下降。
  - 管理员能在不读日志的情况下判断“保存了但未重启”。

## 风险与取舍

- 风险：Impact preview 被误解为真实投递保证。取舍：第一版标明 approximate，不执行真实 LLM classifier 和 sink。
- 风险：Probe 抓外部站点会引入网络抖动。取舍：probe 是显式管理员动作，设置短 timeout，并把失败留在 candidate 而不是阻塞后台。
- 风险：新增 lifecycle 存储与 config source of truth 冲突。取舍：candidate/trial 只是工作台状态；active truth 仍是 `config.yaml` + overlay。
- 风险：Telegram web preview DOM 可能变化。取舍：probe 失败只影响新来源验证；现有 poller 保持失败重试，不把该来源提升为核心依赖。
- 风险：持续 dry-run 会增加拉取成本。取舍：Phase 1-3 先做手动 probe；Phase 4 再引入 mode，并默认低频或需要显式开启。
- 不做：不建设公开来源市场，不自动推荐媒体源，不绕过现有 router/prefs，不改变 source provenance/freshness 的提案职责，不把普通 public 用户暴露到来源运维页面。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 与历史 `docs/proposals/`：

- 不重复 `auto_p1_source-provenance-freshness.md`：该提案记录事实进入系统后的来源、时效、fallback 和健康；本提案关注来源进入系统前的 probe、preview、trial、enable、pause 和 pending restart 生命周期。
- 不重复 `auto_p1_delivery_decision_loop.md`：该提案解释某个 actor 为什么收到、进入 digest 或被过滤；本提案在事件产生前评估一个 source 可能制造多少事件和噪音。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：该提案判断当前部署的 runner、模型、渠道和能力是否 ready；本提案判断外部信号源配置和运行中 poller 是否一致、是否可安全上线。
- 不重复 `auto_p1_temporal-operations-calendar.md`：该提案关注自动化/推送时间窗口和运行节奏；本提案只把 source interval / pending restart 作为来源生命周期的一部分。
- 不重复 `auto_p1_multichannel-render-preview.md`：该提案验证同一输出在多渠道中的渲染；本提案验证上游事件源的样本质量和可能投递影响。
- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该历史提案关注 skill runtime 和 multi-agent 执行语义；本提案不改变 skill 注入或 runner 阶段。

查重结论：现有 proposal 已覆盖来源健康、投递解释、runtime readiness、时间运营和多渠道渲染，但没有覆盖“RSS/Telegram/未来事件源从草稿到试运行再到启用/回滚”的产品和架构控制面。本提案是新的 P2 机会，适合后续在事件源继续扩展时执行。

## 本轮文档同步说明

本轮只创建 proposal，不开始实施，不修改业务代码、测试、运行配置或 `docs/current-plan.md`。若后续执行本提案，预计需要同步更新 `docs/repo-map.md`、`docs/invariants.md`，并视实现范围补充事件源运维 runbook。
