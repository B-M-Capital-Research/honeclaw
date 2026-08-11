# Multi-method Daily Valuation And Rating Integration

- title: 多方法每日估值与公司评级联动
- status: completed
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/valuation_lab.rs`
  - `crates/hone-web-api/src/routes/company_ratings.rs`
  - `packages/app/src/pages/public-valuation-lab.tsx`
  - `packages/app/src/components/company-rating-dashboard.tsx`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md#d-2026-08-11-10-route-daily-valuation-by-business-model-and-feed-ratings-only-from-cross-checked-results`
  - `docs/handoffs/2026-08-11-multi-method-valuation-rating-integration.md`

## Goal

用用户提供的闪迪深度报告所展示的多方法、周期调整和概率情景逻辑替代当前通用 70% DCF / 30% 前瞻倍数模型，并让每日公司评级只消费同日、可复算、通过质量门禁的估值结果。

## Completed Scope

- 按周期制造、盈利成长和收入转型三种商业模式选择主估值法、交叉验证法和权重。
- 周期硬件采用前瞻 P/E、EV/EBIT 和向中周期现金流回归的 DCF；避免以 EV/S 作为已经盈利周期公司的基准方法。
- 悲观、基准、乐观情景展示每种方法的值、权重、假设和 20%/55%/25% 概率，并输出概率加权价值、预期空间和当前股价反向估值。
- 估值结果只有在至少两种方法、日期新鲜、情景有序且方法离散度合格时，才写入每日公司评级。
- 公司评级升级为 `hone-company-rating-v3`，仅接受合格的 `hone-valuation-v2` 计算结果或独立的人审 `hari-invest-v1 / verified` 结果。
- 估值 worker 保持北京时间 19:20 更新，写入后立即刷新公司评级；19:30 独立评级任务继续作为兜底。

## Verification

- `cargo test -p hone-web-api --lib`: 276 passed / 2 ignored。
- 前端相关测试：22/22。
- TypeScript 类型检查、Rust 格式检查、public production build、console binary build 与 `git diff --check` 通过。
- 本地后端重启后生成 `hone-valuation-v2` 与 `hone-company-rating-v3` 快照；本机没有 FMP key 时 52 家全部明确不可用，评级估值为 0 条，没有沿用旧值或模拟值。

## Remaining Operational Step

目标环境需要配置现有 FMP key pool，才能形成真实的每日估值。首个有效批次应人工抽检周期类与非周期类公司各至少一只，再决定是否调整 HONE 默认权重；不得为提高覆盖率而放松缺数与离散度门槛。
