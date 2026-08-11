# Daily Valuation Lab

- title: HONE 每日估值实验室与评级联动
- status: done_locally
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/valuation_lab.rs`
  - `crates/hone-web-api/src/routes/company_ratings.rs`
  - `packages/app/src/pages/public-valuation-lab.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-11-daily-valuation-lab.md`

## Goal

把“今日不计估值分”的结构性缺口变成一个可见、可复算、可失败关闭的每日估值产品：展示悲观/基准/乐观值、当前价格位置、反向 DCF 隐含增长、交叉验证、输入来源和数据日期，并让通过完整证据门槛的估值参与公司评级与仓位管理。

## Delivered Scope

- 独立 `/valuation-lab` 页面与对话首页入口，覆盖现有 52 家研究公司。
- 19:20 北京时间自动读取 FMP 当日行情、季度现金流、资产负债表和分析师估计。
- HONE 自有三情景 DCF、forward EPS 倍数交叉验证、反向 DCF 与完整来源/日期/假设审计。
- 缺少核心输入、负自由现金流、数据过期、交叉验证分歧过大或模型不收敛时明确失败关闭。
- 只有合格结果写入公司评级估值契约，随后由 19:30 公司评级任务独立校验并使用。
- HONE 数值方法与 Hari 定性投资框架明确分层，不把模型折现率和倍数归因给老王。

## Validation

- `cargo test -p hone-web-api --lib --no-fail-fast`: 268 passed, 2 ignored.
- 估值单元测试：5 passed。
- `bun run typecheck:web`: passed.
- `bun run test:web`: 444 passed.
- `bun run build:web:public`: passed.
- `cargo fmt --all -- --check`: passed.
- Authenticated API smoke and desktop browser acceptance passed. With no local FMP key the real snapshot retained 52 unavailable rows and zero fabricated values.

## Follow-up

- 在部署环境通过现有 HONE FMP 配置注入合法凭据，然后观察首个完整市场日的覆盖率、交叉验证差异和反向 DCF 分布。
- 按商业模式增加银行、保险、REIT 或尚未产生正自由现金流公司的专用模型；在此之前保持无估值。
- 人工抽查首批 eligible 公司后再考虑调整折现率、终值增长和倍数上限，所有变更必须版本化。
