# Proposal: Subscription Explainability Map for Portfolio-Driven Monitoring

status: proposed
priority: P1
created_at: 2026-07-06 03:03:01 CST
owner: automation

related_files:

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `config.example.yaml`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/router/dispatch.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/engine.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/lib/mainline-context-model.ts`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposal/auto_p1_notification-policy-backtest.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_channel-activation-proof.md`
- `docs/proposal/auto_p1_watchlist-conversion-pipeline.md`

## 背景与现状

Honeclaw 已经有一条比较完整的投资监控链路：用户通过 Web、对话工具或渠道入口维护 portfolio / watchlist；`memory/src/portfolio.rs` 将持仓和关注项按 `ActorIdentity` 存在本地 JSON 或云端 PG；`crates/hone-tools/src/portfolio_tool.rs` 允许 agent 通过 `add` / `watch` / `unwatch` 等动作更新同一份状态；事件引擎在 `crates/hone-event-engine/src/subscription.rs` 里用 `registry_from_portfolios` 从 portfolio 构建 `PortfolioSubscription` 和 `GlobalSubscription`，并由 `SharedRegistry` 每 60 秒热刷新 watch pool。

事件抵达后，`crates/hone-event-engine/src/router/dispatch.rs` 再把订阅命中结果与用户通知偏好叠加：`enabled`、`portfolio_only`、`min_severity`、`allow_kinds`、`blocked_kinds`、`quiet_mode`、`quiet_hours`、价格阈值、same-symbol cooldown、daily cap、digest queue 等规则共同决定一条事件是直推、进 digest、被过滤、被降级，还是因为没有 actor 命中而写入 `no_actor` delivery log。

产品侧目前有几个独立视图：管理端 `/notifications` 可以查通知日志和状态；`/api/notification-prefs` 能读写 actor 偏好；`/api/event-engine/mainline-context` 和 public `/portfolio` 能展示持仓、公司画像和投资主线；用户也可以在对话里自然语言修改组合和偏好。但这些视图还没有合成一个“订阅解释地图”：用户或运维很难直接回答“为什么我会收到 AAPL 的 SEC 事件但没有收到 TSLA 的价格异动”、“这个关注标的是否真的进入 event-engine watch pool”、“某个静音设置是否导致最近 7 天没有推送”、“群聊 portfolio 为什么不会主动推送”。

## 问题或机会

监控产品的信任来自可解释性。Hone 的定位不是泛聊天，而是长期投资纪律助手；如果用户不能确认自己关心的标的是否被监控、哪些规则会阻止推送、错过消息时该去哪里排查，portfolio、通知偏好、事件引擎和多渠道投递会被体验为一组黑箱。

这个问题会影响四条链路：

- 用户端：用户在 public `/portfolio` 看到投资主线和画像，但看不到每个 ticker 的监控状态、最近命中事件、下一次 digest 窗口、被静音原因。
- 管理端：`/notifications` 偏向事后日志查询，缺少按 actor / ticker 展开的“订阅配置 -> watch pool -> 路由规则 -> 最近投递结果”总览。
- 多渠道：Feishu / Discord / Telegram / iMessage / Web push 的渠道目标、直接聊天限制、群聊不推送规则分散在代码和日志里，用户难以知道某个目标是否能接收主动事件。
- 商业化与留存：付费价值之一是“我不会错过持仓关键变化”。若缺少可解释订阅状态，用户配置完成后的安全感不足，也不利于客服或 agent 快速定位误报、漏报、过度打扰。

## 方案概述

新增一个 Subscription Explainability Map：以 actor 为中心，把 portfolio/watchlist、event-engine registry、notification prefs、channel target、delivery log、digest schedule、mainline context 聚合成一张可查询的订阅解释图。

第一阶段只做只读解释面，不新增新的推送策略：

- 管理端新增“Monitoring Map”视图，按 actor 展示所有 monitored tickers、watchlist-only tickers、真实持仓 tickers、global macro/social subscriptions、最近事件和最近过滤原因。
- public `/portfolio` 增加轻量“监控状态”区块，让终端用户知道每个标的是“已监控 / 仅有画像未入组合 / 偏好过滤 / 渠道未就绪 / 最近无事件”。
- 对话侧提供一个只读 tool 或 skill prompt 能回答“为什么没有收到 X”并引用同一 explain API。
- API 返回机器可读 reason codes，而不是只返回中文文案，便于测试、日志、前端和 agent 共用。

## 用户体验变化

用户端：

- 在 `/portfolio` 的每个 ticker 卡片旁显示监控状态：`active`、`watchlist_only`、`profile_only`、`blocked_by_prefs`、`channel_not_ready`、`group_push_unsupported`、`no_recent_events`。
- 展开后显示“监控依据”：来自真实持仓、watchlist、global macro/social，还是仅来自公司画像。
- 显示“下一次可能到达”：即时推送条件、下一次 digest slot、quiet hours 结束时间。
- 当用户问“为什么 MU 没提醒我”时，agent 可以返回结构化解释：portfolio 是否包含 MU、registry 是否命中、最近事件有没有被 `blocked_kinds` / cooldown / quiet hours / digest cap 处理。

管理端：

- 在用户详情或 notifications 页面增加 Monitoring Map 入口。
- 支持按 actor、ticker、reason code 过滤，直接跳到最近 delivery log。
- 对配置问题给出修复入口：去 portfolio 增加 watch、去通知偏好取消 block、去渠道设置完成 activation proof、或提示“群聊主动推送按现有产品规则不支持”。

桌面端：

- 桌面 bundled 模式复用同一 Web UI 和 API；在本地用户排查“为什么没推送”时不需要翻 `data/` 目录。
- 桌面 tray / channel status 后续可以链接到 Monitoring Map，但本提案 v1 不要求新增 native UI。

多渠道：

- 解释面需要明确区分 actor 身份、session 身份和 channel target。对于同一手机号 / open_id / Discord user，在不同渠道下要展示各自的订阅和投递能力。
- 群聊 portfolio 被 `registry_from_portfolios` 跳过时，应显示为产品规则导致的 `group_push_unsupported`，不是误判为“没有配置”。

## 技术方案

新增 read-only 聚合层，建议放在 `crates/hone-web-api`，核心逻辑抽到 `hone-event-engine` 或新的轻量 helper，避免前端拼接多个 API 后各自解释规则。

建议 API：

- `GET /api/event-engine/subscription-map?channel=&user_id=&channel_scope=`
- `GET /api/public/subscription-map`
- 可选：`GET /api/event-engine/subscription-map/ticker?channel=&user_id=&channel_scope=&ticker=`

返回结构示例：

```json
{
  "actor": {
    "channel": "web",
    "user_id": "phone_hash_or_id",
    "channel_scope": null
  },
  "registry": {
    "direct_actor": true,
    "subscribed_symbols": ["AAPL", "MU"],
    "global_kinds": ["macro_event", "social_post"],
    "last_refresh_hint": "registry refreshes every 60s"
  },
  "prefs": {
    "enabled": true,
    "portfolio_only": false,
    "min_severity": "Low",
    "blocked_kinds": [],
    "quiet_mode": false,
    "quiet_hours_active": false,
    "digest_slots": [{"time": "08:30", "label": "盘前摘要"}]
  },
  "tickers": [
    {
      "symbol": "AAPL",
      "source": ["holding"],
      "monitoring_status": "active",
      "reason_codes": ["portfolio_subscription_active"],
      "recent_delivery": {
        "last_event_at": "2026-07-05T12:00:00Z",
        "last_status": "queued",
        "last_route": "digest"
      }
    }
  ]
}
```

数据来源：

- portfolio / watchlist：`PortfolioStorage::load`，保留 `tracking_only` 区分真实持仓与关注项。
- registry 规则：复用 `registry_from_portfolios` 的同等判断，尤其是 direct actor 限制、symbol normalization、global macro/social subscription。不要让 API 自己发明一套不同订阅逻辑。
- notification prefs：复用 `FilePrefsStorage` / cloud prefs，读取 `NotificationPrefs::default()` 后的有效状态。
- quiet hours：复用 `EffectiveTz` 或同等 helper 判断当前是否 active。
- delivery log：从 `EventStore` 读取 actor + symbol + kind 的最近投递记录，包括 `sent`、`queued`、`filtered`、`capped`、`cooled_down`、`quiet_held`、`no_actor` 等已有状态。
- channel readiness：v1 只读现有 channel settings / `/api/channels` 能力，不新增 activation 流程；如果渠道状态不可靠，返回 `unknown` 而不是误报为 ready。

兼容策略：

- API 只读，不改变现有 dispatch、prefs、portfolio 写入语义。
- local / cloud 模式都必须通过既有 storage façade 获取 portfolio 和 prefs，避免破坏 `cloud.mode=local|cloud|auto` 的显式权威边界。
- v1 不把公司画像自动加入 watch pool；如果一个 ticker 只有 profile 没有 portfolio/watchlist，应解释为 `profile_only_not_monitored`，并引导用户加入关注或持仓。
- reason code 必须稳定，前端展示文案可以本地化，但测试和 agent 输出不依赖自然语言。

## 实施步骤

1. 设计 `SubscriptionExplainStatus` / `TickerMonitoringStatus` 类型，列出稳定 reason codes。
2. 在 event-engine 或 web-api 增加只读 explain helper，复用 `PortfolioStorage`、`NotificationPrefs`、`registry_from_portfolios` 等现有逻辑。
3. 在 `EventStore` 增加按 actor + symbol 查询最近 delivery records 的窄接口；若现有接口足够则只封装查询，不改 schema。
4. 增加 admin API `/api/event-engine/subscription-map` 和 public API `/api/public/subscription-map`。
5. 管理端在 notifications 或 users 详情页增加 Monitoring Map 视图，支持 actor / ticker drilldown。
6. Public `/portfolio` 增加每个 ticker 的监控状态和轻量展开解释。
7. 给 agent 增加只读工具或 skill 指南，回答“为什么没提醒我 / 我监控了哪些标的”时走 explain API。
8. 增加 regression tests：holding -> subscribed、watchlist -> subscribed、profile-only -> not monitored、group actor -> unsupported、blocked kind -> filtered explanation、quiet hours -> delayed explanation。

## 验证方式

- Rust 单元测试：
  - `PortfolioStorage` 中真实持仓和 `tracking_only` 关注项都能进入 explain map。
  - group `ActorIdentity` 返回 `group_push_unsupported`，且不污染 watch pool。
  - `NotificationPrefs` 的 `enabled=false`、`portfolio_only`、`blocked_kinds`、`allow_kinds`、`quiet_hours` 能产生稳定 reason code。
- Rust API 测试或集成测试：
  - 构造 portfolio + prefs + delivery log，验证 `/api/event-engine/subscription-map` 输出包含 ticker 状态、prefs 摘要和最近 delivery。
  - public API 只能读取当前登录 actor，不允许 query 任意 actor。
- 前端测试：
  - monitoring status model 能把 reason code 映射为稳定标签和 CTA。
  - public portfolio 在 API 缺失或返回 `unknown` 时保持现有主线/画像展示可用。
- 手工验收：
  - 新建 watchlist ticker，等待 registry refresh 后 Monitoring Map 显示 active。
  - 设置 `blocked_kinds=["price_alert"]` 后，ticker 仍显示被监控，但价格提醒路径标记为 blocked by prefs。
  - 清空渠道或禁用 channel 后，解释面显示渠道不可达，而不是误导用户为未监控。
- 指标：
  - “为什么没收到提醒”类客服/issue 数量下降。
  - public `/portfolio` 到 watchlist/holding 补全动作的转化率提升。
  - 通知误报/漏报排查平均时间下降。

## 风险与取舍

- 解释层如果复制 dispatch 规则，容易和真实路由漂移；因此必须复用现有类型和 helper，必要时先把 router 中可纯函数化的判断下沉，而不是在 API 里重写一遍。
- 过度展示内部规则可能让普通用户困惑；public 面只显示简洁状态和下一步，完整 reason code 留给管理端和 agent。
- delivery log 只代表历史，不代表未来一定推送。UI 文案应区分“最近一次状态”和“当前配置推断”。
- 不在 v1 做自动修复和策略变更，避免把 explain map 变成第二套通知控制台；写操作继续走 portfolio tool、prefs API、渠道设置和已有提案覆盖的控制面。
- 不把公司画像直接视为订阅源。画像是长期研究记忆，不应因为用户读过一家公司就自动开始推送，除非用户明确加入关注或持仓。

## 与已有提案的差异

- `auto_p1_end-user-notification-control.md` 关注用户如何配置通知偏好；本提案关注配置之后如何解释“哪些标的正在被监控、为什么某条事件会或不会送达”。
- `auto_p1_notification-policy-backtest.md` 关注策略变更前的历史回放评估；本提案关注实时 actor/ticker 订阅状态和最近 delivery 解释。
- `auto_p1_delivery_decision_loop.md` 关注通知决策闭环和质量反馈；本提案更窄，聚焦 portfolio/watchlist 到 subscription registry 的可见性。
- `auto_p1_channel-activation-proof.md` 关注渠道启用时的投递证明；本提案只读取渠道 readiness 作为 explain map 的一个输入，不负责渠道激活流程。
- `auto_p1_watchlist-conversion-pipeline.md` 关注如何把关注标的转成研究/持仓机会；本提案不推动转化，而是说明 watchlist 是否已经参与主动事件监控。
- `auto_p1_run_trace_workbench.md` 和 `auto_p1_tool-contract-replay-harness.md` 偏 agent 执行和工具可靠性；本提案面向事件引擎订阅/通知链路，不复盘 agent turn。
- `auto_p1_investment-coverage-matrix.md` 关注 actor 投资准备度；本提案关注推送订阅状态，不评价研究覆盖质量。

查重范围已覆盖 `docs/proposal/` 与 `docs/proposals/` 的现有提案标题和主题。本提案选择的“订阅解释地图”不是新的推送策略、不是偏好控制台、不是渠道激活，也不是通知日志回放，而是把已经存在的 portfolio、registry、prefs、delivery log 串成可解释产品面。
