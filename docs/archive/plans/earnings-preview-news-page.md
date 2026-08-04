# Earnings Preview News Page

- title: 财报前瞻独立近期新闻页
- status: archived
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
  - tests/regression/ci/test_earnings_research_pdf_markdown.sh
  - output/pdf/SNDK-FY2026-Q4-earnings-preview-with-news-2026-08-04-c6d2db81.pdf
- related_docs:
  - docs/invariants.md
  - docs/decisions.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md
  - docs/archive/index.md

## Goal

在所有财报前瞻中恢复旧 Workflow 的独立近期新闻时间线，并让 PDF 从新页开始展示一整页可扫读的“日期、事件、当季影响、是否计入指引”，同时保持新闻与财务预测的因果关系。

## Scope

- 已完成：前瞻固定章节新增 `## 1.3 近期新闻`，位于机构对比之后。
- 已完成：新闻按绝对日期倒序排列，限定 4–8 条，每条说明经营影响、影响财季和指引计入状态并带来源。
- 已完成：PDF 将该章节强制从新页开始；未改变财报分析模式、入口、权限、品牌或分享页。
- 已完成：重新生成 SNDK FY2026 Q4 样例和 PDF。

## Validation

- CI 正反例覆盖缺少新闻页、条数不足、缺少日期/影响/计入状态和分页 CSS 标记。
- Skill 校验、Python 编译、PDF CI 与手工回归通过。
- 新版 SNDK PDF 为 4 页 A4；第 3 页完整独立承载 6 条新闻，第 4 页为分享页。全部页面 120 DPI 逐页检查通过，无溢出、空白页、截断或乱码。

## Documentation Sync

- 更新 `docs/invariants.md`、现有 earnings decision、同日 handoff 和 archive index，本计划归档。
- 章节和主数据流职责未变，因此无需更新 `docs/repo-map.md` 或新增 ADR。

## Risks / Open Questions

- 结构校验能阻止新闻页缺失和字段不完整，但不能自动判断一条新闻是否真正重要；Skill 仍要求剔除纯股价波动和无法连接经营变量的标题。
- 4–8 条与紧凑格式通常能控制在一页，最终仍以逐页 PDF 视觉检查为准。
