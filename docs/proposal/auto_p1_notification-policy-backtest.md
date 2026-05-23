# Proposal: Notification Policy Backtest Lab for Event-Engine Strategy Changes

status: proposed
priority: P1
created_at: 2026-05-23 14:02:24 +0800
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
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p2_signal-source-lab.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `crates/hone-event-engine/src/router/{config,dispatch,policy,tests}.rs`
- `crates/hone-event-engine/src/{event,store,prefs,subscription}.rs`
- `crates/hone-event-engine/src/digest/{buffer,curation,render,time_window}.rs`
- `crates/hone-event-engine/src/tests.rs`
- `crates/hone-web-api/src/routes/{notifications,event_engine_admin,notification_prefs}.rs`
- `packages/app/src/pages/{notifications,task-health}.tsx`
- `packages/app/src/pages/notifications-model.ts`
- `packages/app/src/lib/admin-content/notifications.ts`
- `config.example.yaml`
- `tests/regression/ci/`
- `tests/regression/manual/`

## 背景与现状

Hone 的主动通知已经不是简单的「有事件就推」。当前 `hone-event-engine` 在 `NotificationRouter::dispatch` 中做了多层策略判断：全局 kind 禁用、系统级 severity 降级、actor 订阅命中、用户偏好过滤、LLM 不确定新闻仲裁、per-actor price / immediate override、quiet mode、当日 High cap、价格 band advance、同 ticker cooldown、quiet_hours hold，以及 High 直推和 Medium/Low digest 入队。

这些策略都已经落在真实代码里：

- `crates/hone-event-engine/src/router/config.rs` 保存 `high_daily_cap`、`same_symbol_cooldown_minutes`、`price_min_direct_pct`、`price_band_min_advance_pct`、`price_close_direct_enabled`、macro immediate window、news upgrade cap 等关键参数。
- `dispatch.rs` 把 `sent`、`queued`、`filtered`、`capped`、`cooled_down`、`price_low_advance`、`quiet_held`、`failed`、`no_actor` 等结果写入 `delivery_log`。
- `store.rs` 将 `events`、`engine_meta` 和 append-only `delivery_log` 存在 SQLite，并可选写 JSONL 镜像。
- `prefs.rs` 让每个 actor 的通知偏好按 JSON 文件运行时生效，包含 allow/block kinds、source allow/block、digest slots、mainline style、quiet hours 等。
- `router/tests.rs` 已覆盖大量单点规则，例如 high cap、cooldown、price band、macro due window、legal ad 降级、quiet mode、LLM 新闻升级 cap。
- `crates/hone-event-engine/src/tests.rs` 里存在一个忽略的 `live_portfolio_backtest_push`，会用真实持仓、真实 FMP、真实 Telegram 做端到端回测，但它依赖外部账号和本机数据，只适合作为手工 smoke。
- 管理端 `/admin/notifications` 已合并 cron job 发送记录和 event-engine `delivery_log`，能看到真实送达、静音 hold、偏好过滤、digest 排队与失败。

这说明 Hone 已经有「策略执行」和「事后审计」基础，但缺少一个专门面向策略变更的回测实验室。现在如果维护者想把 `price_band_min_advance_pct` 从 2% 调到 3%、降低 `same_symbol_cooldown_minutes`、修改 quiet mode 豁免列表、调整 High cap、让某类 SEC filing 直推，或者改 digest curation 上限，主要只能依赖局部单元测试、少量手工真实回测和上线后的 delivery_log 观察。

## 问题或机会

主动通知是 Hone 投资助手最敏感的信任链路之一。推太少，用户会错过 thesis-changing evidence；推太多，用户会把 Hone 当作噪音源关掉；推送策略变化如果没有历史样本证明，很容易在「降低打扰」和「漏掉关键事件」之间盲调。

当前缺口集中在五类：

1. **单元测试能证明规则，但不能证明组合策略的历史影响。**  
   `router/tests.rs` 能锁住一个价格 band 或 cooldown 分支，但不能回答「过去 30 天在当前持仓和偏好下，改这个参数会多推多少、少推多少、哪些事件会从 immediate 变 digest、哪些 High 会被 quiet hold」。

2. **delivery decision 解释的是单条结果，不是变更前评估。**  
   `auto_p1_delivery_decision_loop.md` 关注用户和管理员理解一条事件为什么 sent / queued / filtered，并从记录生成偏好 patch。本提案关注在策略或偏好真正生效前，用历史事件和 actor 状态做反事实评估，避免上线后才发现噪音或漏推。

3. **真实回测存在，但不是可沉淀的产品机制。**  
   `live_portfolio_backtest_push` 很适合维护者验证真实外部链路，但它需要 Telegram、FMP 和本机 portfolio；它不会保存结构化反事实差异，也不能进入 CI 或管理端灰度流程。

4. **新增数据源和模型路线都会改变通知压力。**  
   Signal Source Lab 可以预览新 RSS / Telegram source 样本，Model Route Evaluation Lab 可以评估 LLM route 质量，但二者最终都可能改变 event volume、severity 和 classifier 结果。Hone 需要一个共用的通知策略回测层来回答「这些变化对用户收件箱意味着什么」。

5. **商业和留存需要可量化的打扰预算。**  
   主动通知价值应该体现在「关键证据及时到达，噪音被压住」。如果没有策略回测，管理端很难为不同用户类型、套餐、持仓规模或渠道建立稳定的 notification budget。

因此本提案建议新增 **Notification Policy Backtest Lab**：用历史 `MarketEvent`、actor subscription、portfolio-derived registry、`NotificationPrefs` 和候选 router 参数，在隔离环境里重放策略，输出 sent / digest / filtered / capped / held / failed 的反事实差异报告。

## 方案概述

新增一个离线、无真实投递的通知策略回测层。它不替换 `NotificationRouter`，而是用同一套 router 规则在 dry-run sink 和临时 digest/store 中重放历史事件，对比 baseline 策略和 candidate 策略的结果。

核心对象：

- `PolicyBacktestScenario`  
  描述回测窗口、actor 范围、事件来源、baseline 配置、candidate 配置、prefs snapshot、portfolio/subscription snapshot、是否启用 LLM classifier fixture。

- `PolicyBacktestRun`  
  一次实际回测记录，包含 scenario id、输入样本数量、actor 数、策略 diff、开始/结束时间、运行状态和报告文件路径。

- `PolicyBacktestDecision`  
  单个 `(event_id, actor)` 在 baseline 与 candidate 下的结果对比，例如 `sent -> queued`、`queued -> sent`、`filtered -> queued`、`sent -> quiet_held`。

- `PolicyImpactSummary`  
  聚合指标：直推数量变化、digest 数量变化、filtered 数量变化、High 直推保留率、价格事件直推变化、关键 filing / earnings 保留率、每 actor 每日打扰峰值、top changed symbols/sources/kinds。

- `PolicyRiskFlag`  
  可机器判定的风险提示，例如「High earnings sent 减少超过 20%」「单 actor 单日直推超过 8 条」「price alert 全部从 sent 变 queued」「quiet_hours hold 过期 drop 增多」。

第一版目标保守：只做离线对比和报告，不自动改生产配置，不真实投递，不调用真实外部数据源。

## 用户体验变化

### 用户端

- 普通用户不会直接看到回测实验室，但会获得更稳的通知体验：策略调整前可以证明不会明显漏掉财报、SEC、重大价格 band 或持仓主线相关事件。
- 未来如果开放 per-user notification budget，用户端可以看到「基于过去 30 天估算：开启 quiet mode 后即时推送约减少 35%，财报/SEC 仍即时保留」这类可理解的预览。
- 当用户在偏好页调整阈值时，可先看到近 7 天反事实摘要，而不是保存后等待真实市场事件验证。

### 管理端

- 新增 `Policy Backtest` 页面或放入 `Notifications` 的高级 tab：
  - 选择时间窗口、actor、symbols、sources、kinds。
  - 选择候选改动：High cap、cooldown、price band advance、price close direct、macro window、disabled kinds、quiet mode 默认、digest slots。
  - 点击 `Run backtest` 后查看影响摘要、风险 flags 和逐条 diff。
- 在 `NotificationPreferencesCard` 保存高影响变更前，可展示一个 lightweight dry-run preview。
- 在 event-engine settings 修改全局策略时，页面要求至少运行一次最近 7/30 天回测，或者显式记录「跳过回测」原因。
- Backtest report 可作为 release note、handoff 或提案实施阶段的证据留存。

### 桌面端

- Desktop bundled 模式复用 Web 管理端能力，不新增本地 sidecar。
- 对本地单用户，默认只回放自己的 actor 数据，避免误读其它 actor 的打扰预算。
- Remote desktop 模式只调用远端 backend 的 backtest API，不读取本机数据目录。

### 多渠道

- Feishu / Telegram / Discord / iMessage 不需要新增协议能力。
- 回测报告需要按 channel target 聚合：同一个 actor 在 Feishu 的失败风险、Telegram 的消息长度/格式风险、digest-only 变化要分开呈现。
- group actor 默认单独统计，不能和个人 direct actor 混在一个打扰预算里。

## 技术方案

### 1. 建立 dry-run router 执行路径

第一版应尽量复用 `NotificationRouter::dispatch`，避免复制路由规则：

- 使用临时 `EventStore` 和 `DigestBuffer`。
- 使用 `DryRunSink`，其 `send()` 只记录 body preview、format、actor 和 success status，不调用真实 channel。
- 使用固定 `PrefsProvider`，从回测 scenario 中加载 actor prefs snapshot。
- 使用 `SharedRegistry` snapshot，来源可以是当前 portfolio directory 或 scenario fixture。
- 对 LLM 新闻升级，默认使用 fixture classifier；缺 fixture 时标记 `classifier_skipped`，避免回测消耗真实模型额度。需要 live classifier 的回测只能进入 manual lane。

关键约束：回测不得写生产 `delivery_log`、不得触发真实 digest fire、不得修改 `NotificationPrefs`、不得触发 channel sink。

### 2. 支持历史事件输入

输入来源按优先级分层：

- `events` SQLite：从 `EventStore` 查询窗口内事件，保留 kind、severity、symbols、occurred_at、source、payload。
- JSONL 镜像：当 SQLite 损坏或需要跨机器复现时，从 `events.jsonl` 读取。
- Fixture：放在 `tests/fixtures/event_engine/policy_backtest/*.jsonl`，用于 CI-safe 样本。
- Manual live：真实 FMP / Telegram / RSS 采集仍保留在 `tests/regression/manual/`，但只作为生成样本或手工 smoke，不是默认 backtest source。

第一版应避免直接回放 `delivery_log` 作为输入；`delivery_log` 是 baseline 观测对象，不是事件事实源。

### 3. Baseline 与 candidate 对比

回测需要跑两次：

1. Baseline：当前 effective config + 当前 prefs snapshot。
2. Candidate：只应用 scenario 中声明的策略 patch。

比较粒度是 `(actor_key, event_id)`：

- baseline decision: `sent | queued | filtered | capped | cooled_down | price_low_advance | quiet_held | failed | no_actor | none`
- candidate decision: 同上
- severity before/after
- delivery channel
- body hash/body preview
- digest bucket
- risk tags

需要注意有些结果依赖「此前已 sent」状态，例如 daily cap、cooldown、price band max。这正是回测需要按时间顺序重放的原因，不能只做单事件纯函数判断。

### 4. 新增 API 与 CLI

建议先加 CLI/脚本，再做管理端 API：

```shell
cargo run -p hone-cli -- event-engine backtest-policy \
  --from 2026-05-01T00:00:00Z \
  --to 2026-05-23T00:00:00Z \
  --actor web::::u_demo \
  --candidate tests/fixtures/event_engine/policy_backtest/candidate.json \
  --out data/runtime/policy-backtests/run-001
```

后端 API：

- `POST /api/event-engine/policy-backtests`
- `GET /api/event-engine/policy-backtests`
- `GET /api/event-engine/policy-backtests/:id`
- `GET /api/event-engine/policy-backtests/:id/diff`

Public API 暂不开放。普通用户的偏好预览可以后续通过 actor-scoped endpoint 单独设计。

### 5. 报告格式

每次 run 输出：

- `summary.json`
- `diff.csv`
- `changed_decisions.jsonl`
- `report.md`

`summary.json` 用于前端渲染，`report.md` 用于 handoff / release 留存。最小字段：

- scenario metadata
- event_count / actor_count / decision_count
- baseline counts
- candidate counts
- deltas by status / severity / kind / source / symbol / actor
- risk flags
- skipped reasons
- verification notes

### 6. CI-safe fixture 与长期回归

新增少量脱敏、合成或公开样本 fixture：

- price band 连续跨档：验证 advance 调整不会漏掉大行情。
- same symbol 多来源 burst：验证 cooldown 对新闻、SEC、财报的组合影响。
- quiet hours + exempt kinds：验证财报/SEC/盘中 price band 仍保留。
- legal ad / noisy PR wire：验证降噪策略仍进入 digest 或 filtered。
- macro due window：验证远期宏观事件不会误直推。

CI 只跑小样本和 mock classifier，目标是证明 backtest harness 和报告格式稳定；大窗口历史回放进入 manual/release preflight。

## 实施步骤

### Phase 1: 离线回测骨架

- 在 `hone-event-engine` 增加 `policy_backtest` 模块，定义 scenario、decision、summary、risk flag 类型。
- 实现 `DryRunSink`、临时 store/digest、snapshot prefs provider。
- 支持从 JSONL fixture 读取 `MarketEvent` 并按 `occurred_at` 排序重放。
- 增加 3-5 个小型 CI fixture 和单元测试，证明 baseline/candidate diff 可生成。

### Phase 2: 接入真实本地历史

- 支持从 production event SQLite 只读查询时间窗口。
- 支持从 actor portfolio / subscription registry 构造 actor 命中 snapshot。
- 支持从 current prefs dir 读取 prefs snapshot，并把 snapshot hash 写入 report。
- 增加 CLI 或 regression script：`tests/regression/manual/test_event_engine_policy_backtest.sh`。

### Phase 3: 管理端工作台

- 新增 Web API 创建和读取 backtest run。
- 在 `/notifications` 或 event-engine settings 下增加 backtest tab。
- 策略编辑保存前展示最近 7 天 preview，并给高风险变更加确认。
- 将 report 链接接入 `task-health` 或 release checklist。

### Phase 4: 灰度与产品化

- 将 backtest risk flags 作为策略发布门禁：高风险需要手动确认或先灰度。
- 支持 per-actor / per-workspace notification budget preview。
- 与 Delivery Decision Loop 合并：单条决策解释解决「为什么」，Backtest Lab 解决「改了会怎样」。
- 与 Signal Source Lab 合并：新增来源启用前自动跑一次 source impact + policy backtest。

## 验证方式

- Rust 单元测试：
  - JSONL fixture 能解析并按 `occurred_at` 排序。
  - dry-run sink 不调用真实 outbound，但会生成 `sent` 决策。
  - baseline/candidate 对同一事件窗口生成稳定 diff。
  - daily cap、cooldown、price band advance、quiet hours 等依赖历史状态的规则在回测中按顺序生效。
  - missing classifier fixture 时不会调用真实 LLM，并在 report 中写 `classifier_skipped`。

- 回归脚本：
  - 新增 `tests/regression/ci/test_event_engine_policy_backtest.sh`，只跑 fixture、mock prefs、mock registry、mock classifier。
  - 新增 `tests/regression/manual/test_event_engine_policy_backtest_local_history.sh`，读取本机 `data/runtime` / event DB，默认只输出 report，不投递。

- Web/API 验证：
  - admin-only API 能创建和读取 backtest run。
  - actor filter 不会越权读取其它 actor 的 prefs/portfolio。
  - report 大小、diff pagination 和长窗口超时有明确错误。

- 前端验证：
  - `bun run test:web` 覆盖 backtest summary -> table/chart model、risk flag 文案、diff filters。
  - 手工检查 `/notifications` backtest tab 在桌面和移动视口不溢出。

- 产品指标：
  - 每次 event-engine 策略变更前是否生成 backtest report。
  - 上线后 sent/queued/filtered 实际分布与 backtest 预测偏差。
  - 用户关闭通知或反馈漏推的比例是否下降。

## 风险与取舍

- 风险：回测结果被误解为未来保证。取舍：明确标注「基于历史事件和当前 prefs/portfolio 的反事实估算」，不预测未来事件。
- 风险：复用 `NotificationRouter::dispatch` 仍可能不完全等价生产，因为 LLM classifier、sink format、运行时当前时间和外部状态不同。取舍：第一版默认 mock classifier，固定 clock，并在 report 中列出 skipped / approximated 项。
- 风险：读取生产历史和 prefs 可能暴露用户数据。取舍：admin-only，本地生成，report 默认只存 event id、source、kind、symbol、body hash/preview，不存完整私密正文。
- 风险：长窗口大 actor 回测耗时。取舍：提供窗口、actor、symbol、source filters；默认 7 天，长窗口进入后台 job。
- 风险：为了回测而复制 router 规则会产生漂移。取舍：强制复用 `NotificationRouter::dispatch` 和同一 policy helpers，只在 dry-run 边界替换 store/sink/prefs/classifier。
- 不做：不自动修改生产 config，不接真实支付或套餐，不替代 Delivery Decision Loop，不把回测结果当作投资建议，不把真实 channel 发送纳入 CI。

## 与已有提案的差异

查重范围：

- `docs/proposal/`
- `docs/proposals/`
- 关键代码中的 `backtest` / `replay` / `dry-run` 相关命中

差异结论：

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案解释已经发生的单条投递决策，并从具体事件生成偏好调优建议；本提案在策略生效前对历史事件做 baseline/candidate 反事实回测。
- 与 `auto_p1_user-journey-replay-lab.md` 不重复：该提案是跨 public/admin/desktop/channel 的产品旅程回放；本提案聚焦 event-engine notification policy 的时间顺序重放、打扰预算和策略 diff。
- 与 `auto_p2_signal-source-lab.md` 不重复：该提案治理 RSS/Telegram 等事件源上线、probe 和 impact preview；本提案评估任意事件源或策略变化最终对 actor 通知决策的影响。
- 与 `auto_p1_source-provenance-freshness.md` 不重复：该提案记录外部事实来源和新鲜度；本提案假设事件事实已存在，回测路由/偏好/cap/cooldown/quiet/digest 的产品影响。
- 与 `auto_p1_model-route-evaluation-lab.md` 不重复：该提案评估 LLM route 质量；本提案只把 classifier 结果作为可 fixture 化输入或跳过项，重点是通知策略组合影响。
- 与代码里的 `live_portfolio_backtest_push` 不重复：该忽略测试是真持仓、真 FMP、真 Telegram 的手工链路 smoke；本提案要沉淀 CI-safe fixture、生产历史只读回放、结构化 diff 和管理端策略发布证据。

本提案补的是 event-engine 从「规则正确」到「策略变更可评估」之间的产品架构缺口，优先级为 P1，因为它直接影响主动通知可信度、打扰控制、策略发布安全和用户留存。
