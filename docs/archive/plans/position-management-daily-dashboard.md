- title: Hari 每日仓位管理建议仪表盘
- status: completed
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/position_management.rs
  - crates/hone-web-api/src/routes/portfolio_news.rs
  - crates/hone-web-api/src/routes/company_ratings.rs
  - crates/hone-web-api/src/routes/daily_signals.rs
  - packages/app/src/components/position-management-dashboard.tsx
  - packages/app/src/components/position-management-dashboard.css
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/decisions.md
  - docs/repo-map.md
  - docs/handoffs/2026-08-11-position-management-daily-dashboard.md

## Goal

实现第 5 个首页 Button“仓位管理建议”：在每日持仓新闻刷新后，结合真实仓位结构、当日公司评级、宏观红绿灯、经验证的当日估值和近 48 小时持仓新闻，按 Hari `LOG-V0003/4/5/6` 生成 actor 隔离的研究级仓位建议。

## Completed Scope

- 组合集中度、主题暴露、最大/前三大持仓和未分配比例。
- 证据门控的五档动作、风险、证伪条件、数据日期和来源。
- actor 隔离 latest/history 快照，持仓变化后旧建议失效。
- 第 5 个 Button、筛选、详情与保存报告后的对话衔接。
- 明确排除券商连接、自动下单和自动调仓。

## Validation

- Position policy 7/7；Web API 242 passed / 2 ignored。
- Web 430/430；focused contracts 23/23；TypeScript 与 production build passed。
- Authenticated local mobile browser acceptance passed with eight real positions and fail-closed evidence states.

## Conclusion

原定五个首页产品入口全部完成。仓位建议只在当前证据满足严格门槛时给动作；本地未配置行情/估值时正确输出数据不足，而不是把演讲基线包装成当前建议。
