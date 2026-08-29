- title: 每日公司评级、持仓新闻与仓位建议非空数据修复
- status: completed
- created_at: 2026-08-12
- updated_at: 2026-08-12
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/company_ratings.rs`
  - `crates/hone-web-api/src/routes/portfolio_news.rs`
  - `crates/hone-web-api/src/routes/position_management.rs`
  - `packages/app/src/components/company-rating-dashboard.tsx`
  - `packages/app/src/components/portfolio-news-dashboard.tsx`
  - `packages/app/src/components/position-management-dashboard.tsx`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-12-daily-investment-products-data-recovery.md`

## Goal

让三个每日投资产品在已配置真实持仓时稳定给出有内容、可追溯且不伪造的数据结果：公司评级至少有新鲜官方行情与明确研究基线；持仓新闻逐标的显示已覆盖、有重点新闻或未发现重点新闻；仓位管理在缺少完整财务/估值时仍给出低置信度的结构复核动作，而不是把所有持仓清空为“数据不足”。

## Scope

- 诊断 FMP、Tavily、持仓文件、每日 worker、快照与前端过滤链路。
- 为公司行情增加无需用户密钥的 Nasdaq 官方页面 API 降级；保留 FMP 为优先源。
- 不用行情推导缺失财务，不用旧目标价或模拟值补估值。
- 为持仓新闻增加逐标的覆盖状态，明确区分重点新闻、无重点新闻和代码未覆盖。
- 放宽仓位动作的最小证据到“当前行情 + 公司研究基线 + 组合/宏观”，但财务或估值缺失时禁止输出加仓候选并降低置信度。
- 不修改用户持仓，不把 `APPL` 静默改成 `AAPL`；仅明确提示代码无法匹配。

## Validation

- 公司评级解析与 FMP/Nasdaq 降级单元测试。
- 持仓新闻覆盖状态与仓位建议缺财务降级单元测试。
- Web 组件模型/文案测试、TypeScript 检查与生产构建。
- 本地重启后真实刷新：公司评级行情覆盖非零；8 个持仓均有新闻覆盖状态；已覆盖持仓均有仓位动作，错误代码清楚标记。
- `bash scripts/ci/check_fmt_changed.sh`、`git diff --check`。

## Documentation Sync

- 在 `docs/decisions.md` 记录真实数据降级与非空展示边界。
- 完成后新增 handoff、将本计划移至 `docs/archive/plans/`，更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。
- 不改变模块边界，因此无需更新 `docs/repo-map.md`；若实现过程改变长期真相源边界，再同步 `docs/invariants.md`。

## Risks / Open Questions

- Nasdaq 页面 API 不是付费 SLA；失败时必须保留最近成功快照并显示陈旧状态，而不是降为模拟价格。
- 新闻为空可能是真实的“无重大新闻”，产品必须展示覆盖结果而不是制造新闻。
- 未配置 FMP 时财务与估值仍可能不全；本任务保证有可解释内容，不承诺伪造八因子满覆盖。

## Completion

- 2026-08-12 本地实现并验收完成。
- 真实刷新得到 52 家公司、51 份 Nasdaq 行情；8 个持仓均有新闻覆盖状态；7 个持仓有行情支持的仓位动作，错误代码 `APPL` 保持显式数据不足。
- 详见 `docs/handoffs/2026-08-12-daily-investment-products-data-recovery.md` 与 D-2026-08-12-02。
