- title: 用户级价格阶梯与最终生效规则查询
- status: archived
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/prefs.rs
  - crates/hone-event-engine/src/router/config.rs
  - crates/hone-event-engine/src/router/policy.rs
  - crates/hone-event-engine/src/router/dispatch.rs
  - crates/hone-event-engine/src/router/tests.rs
  - crates/hone-tools/src/notification_prefs_tool.rs
  - crates/hone-tools/src/schedule_view.rs
  - crates/hone-channels/src/core/bot_core.rs
  - crates/hone-web-api/src/routes/schedule.rs
  - packages/app/src/pages/schedule.tsx
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/handoffs/2026-08-02-price-ladder-effective-rules.md

## Goal

让用户设置的首次上涨/下跌阈值与重复提醒步长共同形成真正参与路由决策的价格阶梯，并让 `notification_prefs.get_overview` 与管理端 schedule API 返回同一份最终生效规则，而不是只展示无法解释实际行为的原始覆盖字段。

## Completed Scope

- 建立共享 `EffectivePriceAlertPolicy`，集中解析系统候选网格、用户通用/方向阈值、系统最小直推地板、大仓位例外、重复步长与继承来源。
- 新增 actor 级 `price_realert_step_pct_override`、单项继承 action 和原子 `update_delivery_controls` 写入。
- 修复用户阈值高于系统 High 阈值时较低档仍即时推送，以及 `immediate_kinds=price_alert` 绕过显式阈值的问题。
- Router 按 actor/symbol/direction/day 的最终步长判断重复提醒；min severity 按 actor 最终 severity 过滤，而不是复用共享事件的原始 severity。
- `notification_prefs.get_overview`、管理端 schedule API 和 Web 页面共用服务端最终策略与示例，并解释系统事件引擎总开关、全局 disabled kinds、每日每类 High 上限、普通同标的冷却和盘中价格阶梯的冷却豁免。
- PricePoller 的外部报价采样和系统候选 band 协议保持不变；非网格阈值向上落到下一条真实候选档，并在查询结果中明确展示。

## Verification

- `cargo test -p hone-core --lib`: 136 passed。
- `cargo test -p hone-event-engine --lib`: 538 passed, 13 ignored；包含 8/4 双向阶梯、有效 severity 过滤、每日 High 上限和普通 cooldown 豁免回归。
- `cargo test -p hone-tools --lib`: 163 passed, 1 ignored；包含结构化 effective policy、全局执行 gate 和各渠道 `display_text` 的 8/12/16 解释。
- `cargo test -p hone-web-api --lib routes::schedule::tests` 与 `routes::notification_prefs::tests`: 各 3 passed。
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed；仅保留既有 Web API dead-code warning。
- `bun run typecheck:web`: passed；`bun run test:web`: 343 passed。
- `bash tests/regression/run_ci.sh`: passed；44/44 finance contracts 与全部 CI-safe 回归通过。
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`: 43-item 离线 fixture、15 个 LLM 项目加载成功；未运行 live LLM。
- 修改 Rust 文件的显式 `rustfmt --check` 与 `git diff --check`: passed。
- 本地真实临时后端 + 当前 Vite 浏览器验收：`web / ladder-demo` 显示上涨/下跌 8/12/16、候选网格 6/2、每日 cap 8、普通 cooldown 60 分钟且盘中价格阶梯豁免；桌面无水平溢出、无浏览器错误，新规则卡在窄屏独占一行。随后新增的系统开关/全局 kind 警告由 schedule/tool 回归和 Web typecheck/tests 覆盖。

## Documentation Sync

- `docs/decisions.md` 新增 D-2026-08-02-04。
- `docs/invariants.md` 固化执行/解释同源和跨切面约束。
- `docs/repo-map.md` 记录 PricePoller 候选层、actor 路由层和统一概览层边界。
- `skills/notification_preferences/SKILL.md` 改为原子设置价格阈值/步长并直接转发最终规则概览。
- 已新增 handoff、更新归档索引并从活跃任务索引移除。

## Risks / Follow-ups

- 系统候选 band 仍由全局 `price_alert_high_pct + price_realert_step_pct` 产生；用户阈值不是候选网格整数点时，首次实际提醒向上落到下一条候选档。
- 用户阈值低于系统最小直推地板时，非大仓位仍使用系统地板；大仓位保留已有敏感阈值语义。
- 收盘价与盘前盘后价格事件继续遵守现有 direct/quiet 策略；本次 actor 阶梯只改变盘中 `price_band:*` 的即时提醒节奏。
- 管理端外壳在 390px 仍使用既有固定侧栏；新价格卡已在剩余内容区自适应，但全站管理端移动导航不在本任务范围。
- 本变更已提交并直接推送 `main`，未部署。
