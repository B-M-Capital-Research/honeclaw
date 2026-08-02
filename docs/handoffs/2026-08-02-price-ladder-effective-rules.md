- title: 用户级价格阶梯与最终生效规则查询交接
- status: done
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/prefs.rs
  - crates/hone-event-engine/src/router/policy.rs
  - crates/hone-event-engine/src/router/dispatch.rs
  - crates/hone-tools/src/notification_prefs_tool.rs
  - crates/hone-tools/src/schedule_view.rs
  - crates/hone-web-api/src/routes/schedule.rs
  - packages/app/src/pages/schedule.tsx
- related_docs:
  - docs/archive/plans/price-ladder-effective-rules.md
  - docs/decisions.md#d-2026-08-02-04-separate-price-candidate-bands-from-actor-notification-ladders
  - docs/invariants.md
  - docs/repo-map.md
- related_prs:
  - none; committed and pushed directly to main

## Summary

价格提醒现在分成稳定的全局候选网格和 actor 级通知阶梯。用户的首次阈值、方向覆盖、重复步长、大仓位例外和系统地板由一个 domain policy 解析；Router 执行、聊天概览、管理 API 和 Web 页面读取同一结果。8% 首次、之后每 4 个百分点提醒的实际候选序列为正负 8/12/16。

## What Changed

- Actor prefs 新增可继承的重复步长，并纳入共享原子 patch、范围校验、存储兼容和 Web 编辑模型。
- PriceAlert 可按 actor 最终阈值双向升/降级；显式价格阈值优先于 `immediate_kinds`，min severity 按最终 severity 判定。
- 重复 band 按 actor/symbol/direction/day 的最终步长和单调新高执行；上涨与下跌独立。
- 最终规则查询展示系统事件引擎与全局 disabled kinds、原始来源、系统地板限制、真实可观测首档、候选网格、重复步长、普通/大仓位差异、每日每类 High cap，以及普通 same-symbol cooldown 对盘中价格 band 的豁免。
- Web 规则卡直接消费服务端 examples，不在前端重算策略；桌面可伸展、窄屏独占一行。

## Verification

- Core 136；event-engine 538 passed/13 ignored；tools 163 passed/1 ignored；Web API 定向 6；Web 343；TypeScript typecheck；workspace all-target check；完整 CI-safe regressions 全部通过。
- 离线 news classifier baseline fixture 43 条（其中 15 个 LLM 项目）加载成功，未调用 live LLM。
- 浏览器连接当前源码的临时后端，验证 8/12/16、cap 8、cooldown 60 及豁免文案；桌面 `scrollWidth == clientWidth == 1280`，控制台 0 error。

## Risks / Follow-ups

- 任意 actor 阈值不会改变 PricePoller 的全局候选网格；非网格阈值只能在下一条候选 band 被观察到。
- 全局每日 High cap 仍可把后续价格档降为摘要；查询已经明确这一点。
- 管理端固定侧栏的全站移动适配仍是独立议题；本次只保证新增规则卡在窄屏内容区可读。
- 已提交并直接推送 `main`；没有部署或 live LLM 验证。临时本地服务已停止，测试数据已移到废纸篓。

## Next Entry Point

执行语义从 `NotificationPrefs::effective_price_alert_policy` 和 `NotificationRouter::dispatch` 开始；用户查询与渠道文本从 `schedule_view::build_overview_with_cron_jobs` / `render_overview` 开始。若后续调整 PricePoller 候选网格，必须同时更新 `PriceAlertPolicyDefaults`、8/4 contract tests 和有效规则示例。
