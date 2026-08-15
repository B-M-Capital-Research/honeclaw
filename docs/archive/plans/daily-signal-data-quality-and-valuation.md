- title: 每日红绿灯数据质量与公司估值重构
- status: done
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/daily_signals.rs
  - crates/hone-web-api/src/routes/company_ratings.rs
  - packages/app/src/lib/types.ts
  - packages/app/src/components/daily-signal-dashboard.tsx
  - packages/app/src/components/company-rating-dashboard.tsx
  - packages/app/src/components/company-rating-dashboard.css
- related_docs:
  - docs/decisions.md
  - docs/handoffs/2026-08-11-daily-signal-data-quality-and-valuation.md

## Goal

先收紧现有三个每日投资面板：宏观增加长期利率、政策利率、就业率与 VIX；AI 删除无法稳定取得一手证据的专项和硬件兑现因子；公司评级不再把演讲期静态估值或通用市盈率档位冒充当日估值，并为通过新鲜数据门槛的估值显示悲观、基准、乐观区间、现价位置和估值时间。

## Completed Scope

- 宏观 FRED 序列加入 10 年期、30 年期国债收益率、联邦基金利率、就业人口比和 VIX，并使用适合利率/波动率的风险方向。
- AI 评分只保留七个可从标准财报稳定验证的公司财务因子；AI 收入、RPO、订单、专项商业化和硬件兑现不再展示、不计分。
- 公司综合分中的估值维度改为可空；只有当日、已复核、来源充分的 Hari 三情景估值才参与评分，否则明确显示“今日不计估值分”。
- 前端支持悲观/基准/乐观/当前值、区间位置、方法、估值时间和数据日期。
- 旧快照在读取时按 v2 边界归一化，不再泄漏旧估值或旧 AI 因子。
- 未启动其它首页功能。

## Validation

- Rust formatting passed.
- Web API: 229 passed, 2 ignored.
- Full Web: 420 passed; focused dashboard contracts: 21 passed.
- TypeScript and production build passed.
- Authenticated local browser acceptance passed for all three dashboards.
