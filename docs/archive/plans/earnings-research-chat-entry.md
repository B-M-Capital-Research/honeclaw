# Earnings Research Chat Entry

- title: 管理员财报前瞻 / 财报分析聊天入口与 PDF 交付
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/agents/openai.yaml
  - skills/earnings-research/scripts/render_report_pdf.py
  - tests/regression/ci/test_earnings_research_pdf_markdown.sh
  - tests/regression/manual/test_earnings_research_pdf.sh
- related_docs:
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md
  - docs/archive/index.md

## Goal

让 Hone 财报前瞻和财报分析不仅章节匹配旧 Workflow，正文判断顺序和语言也保持一致：结论先行、逻辑连续、近期新闻进入经营假设，不使用普通 AI 问答的时间口径、事实/推断标签、元话术或机械清单。

## Scope

- 已完成：财报前瞻在 `整体分析` 的第一句和 `核心结论` 同步给出唯一的超预期 / 低于预期 / 持平判断；用管理层指引、机构预期、财务假设与近期公司新闻解释判断。
- 已完成：财报分析维持五段报表解读，但去掉重复的“本节结论”、元数据说明与过度防御性 caveat。
- 已完成：渲染前拒绝时间口径开场、事实/推断标签、AI 元话术、普通问答章节和前瞻结论不一致。
- 未调整入口、权限、runner、PDF 品牌或生产部署。

## Validation

- Skill `quick_validate.py` 通过，`agents/openai.yaml` 与触发范围一致。
- CI 正例覆盖自然前瞻与报表分析；反例覆盖时间开场、元话术、错误标题、结论不一致和 Q&A 章节。
- Chromium PDF 手工回归保持 A4、分享页及免责声明布局。
- SNDK FY2026 Q4 前瞻样例按真实数据生成 3 页 A4 PDF，120 DPI 全页视觉检查通过；第一句先判断，近期 NBM 协议和 Kioxia 安排均回接收入、利润或供给假设。

## Documentation Sync

- 更新同日 handoff 与 archive index，并重新归档本计划。
- 本次只收紧 Skill 输出契约，不改变模块边界、信任边界或运行架构，因此无需更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`。

## Risks / Open Questions

- “AI 味”不能完全由语法规则衡量；硬校验只覆盖明确的元话术、判断顺序和新闻逻辑，具体判断仍依赖当轮证据质量。
- 近期新闻必须与收入、利润、毛利率或业务量假设建立因果关系，不能简单罗列标题。
