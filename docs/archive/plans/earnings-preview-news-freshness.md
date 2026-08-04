# Earnings Preview News Freshness

- title: 财报前瞻新闻时效与密度增强
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
  - tests/regression/ci/test_earnings_research_pdf_markdown.sh
- related_docs:
  - docs/current-plan.md
  - docs/invariants.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md

## Goal

提高财报前瞻新闻页的时效、密度与信息覆盖：优先财报日前 7–14 天的公司、同业、供应链、需求端和预期修订证据，在公司缺少当月公告时明确扩展到能直接验证当季经营假设的行业事件，而不是用较早产品新闻凑数。

## Scope

- 新闻容量调整为 8–10 条，保持一页紧凑时间线。
- 至少一半事件来自财报日前 14 天；若不足，必须在审计中记录来源缺口。
- 强制覆盖公司/机构预期、同业或供应链、需求端三类中的至少两类，并保持当季影响和指引计入状态。
- 更新 SNDK 样例，优先纳入 7 月下旬至 8 月 4 日的同业财报、云资本开支和一致预期变化。

## Validation

- 更新正反例，覆盖新闻条数、14 日新鲜度和来源类别。
- 运行 Skill、Python、CI 和手工 PDF 回归。
- 逐页检查新版 SNDK 新闻页是否保持单页且无拥挤、截断和链接溢出。

## Documentation Sync

- 更新 `docs/invariants.md`、现有 earnings decision/handoff/archive index；完成后归档本计划。
- 主模块边界不变，无需更新 `docs/repo-map.md`。

## Risks / Open Questions

- 财报日前可能确实没有足够公司级新闻；允许使用直接相关的同业与需求端证据，但必须说明传导逻辑，不能把宏观标题当公司事实。
- 8–10 条仍需控制单条长度，最终页数以 PDF 视觉验收为准。

## Completion

- 新闻时间线已固定为 8–10 条；`preview_audit.report_date` 成为必填字段，渲染器强制至少一半事件位于财报日前 14 天、严格倒序，并覆盖公司/机构预期、同业/供应链、需求端三类信号。
- SNDK 样例更新为 10 条，其中 8 条位于财报日前 14 天，包含 3 条 8 月事件；新增 HBF 首版标准、Seagate 财报与需求、Amazon/Meta/Alphabet AI 基建投入和财报日前预期锁定。
- focused CI、Python 编译、手工 Chromium PDF 回归、四页 A4 渲染与逐页视觉检查均通过；新闻保持独立单页，无截断或新增空白页。
