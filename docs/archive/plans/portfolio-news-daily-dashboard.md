- title: 持仓重点新闻分析每日仪表盘
- status: completed
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/portfolio_news.rs
  - crates/hone-web-api/src/routes/mod.rs
  - crates/hone-web-api/src/lib.rs
  - packages/app/src/lib/api.ts
  - packages/app/src/lib/types.ts
  - packages/app/src/components/portfolio-news-dashboard.tsx
  - packages/app/src/components/portfolio-news-dashboard.css
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/repo-map.md
  - docs/decisions.md
  - docs/handoffs/2026-08-11-portfolio-news-daily-dashboard.md

## Goal

按照产品优化顺序实现第 4 个首页 Button“持仓重点新闻分析”：北京时间每天 20:00 读取每位用户自己的真实持仓，拉取近 48 小时可信新闻，去噪后用 HONE 配置的模型生成影响方向、期限、核心逻辑影响和关注动作，并保存用户隔离的每日快照供网页快速读取。

## Completed Scope

- `PortfolioStorage` actor 隔离与云/本地兼容列表。
- FMP `NewsPoller`、可信来源分类、噪声过滤和稳定新闻 ID。
- 模型只分析新闻；仓位和成本留在 HONE，权重只参与本地排序。
- actor 隔离 latest/history 快照、完整状态与下次更新时间。
- 第 4 个 Button、摘要计数、筛选、来源链接和发送到对话。
- 明确未实现第 5 个“仓位管理”。

## Validation

- Portfolio-news Rust 6/6；Web API 235 passed / 2 ignored。
- Web 425/425；TypeScript 与 production build passed。
- authenticated local browser no-portfolio acceptance passed。

## Conclusion

功能在本地完成并符合失败关闭、隐私和 actor 隔离边界。生产数据取决于现有 FMP 与 digest model 配置；没有配置时必须继续显示真实降级状态。
