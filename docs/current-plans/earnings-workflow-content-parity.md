# Earnings Workflow 内容一致性与新闻深度修复

- title: Earnings Workflow 内容一致性与新闻深度修复
- status: in_progress
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `skills/earnings-research/SKILL.md`
  - `skills/earnings-research/scripts/render_report_pdf.py`
  - `tests/regression/ci/test_earnings_research_pdf_markdown.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-05-gemini-earnings-workflows.md`

## Goal

修复已能稳定生成 PDF 的财报前瞻在内容层面偏离原 Dify Workflow 的问题：核心结论必须按原 prompt 先明确超出/低于/持平；机构比较必须逐家带日期、评级、目标价和相对指引/独立预测的判断；近期新闻必须逐条展开短期、本季、长期及产品/竞争力传导，不能用弱聚合源或文章观点替代公司事实。

## Scope

- 以线上 Dify `V2-财报前瞻` LLM 节点和“公司近期新闻时间线分析模块”LLM 节点的真实 prompt 为基线，不以二手总结替代。
- 修复 forecast-consensus 恰好落在 tolerance 边界时被二进制浮点误判为 beat/miss 的问题。
- 把 `1.2.1` 恢复为原 prompt 要求的明确结论开头，并加强 `1.2.2` 的增长/毛利率/盈利假设和 `1.2.3` 的逐机构对比。
- 每条新闻保持一个自然段，但至少覆盖事件事实、当季传导、短期/长期判断、产品或竞争力影响、指引计入状态与后续验证点；优先 IR/监管/电话会/真实机构来源。
- 增加结构/语义回归，部署精确 revision/skill 后重新生成 AAOI，检查正文、PDF、下载与刷新持久化。

## Verification

- renderer 单元/回归覆盖 tolerance 边界必须 `与分析师持平`，边界外才可 beat/miss。
- regression fixture 拒绝短新闻段、无逐家日期的机构对比、把 publisher/columnist 写成出具评级机构，以及把合同总额直接当本季收入。
- Skill validator、changed rustfmt（如有 Rust）、相关 renderer/CI 回归通过。
- 生产 AAOI：结论算术正确；`1.2.1` 首句明确 call；机构逐家有日期和差异理由；八至十条新闻均有短期/长期/竞争力传导；PDF 可下载且刷新后仍存在。

## Documentation Sync

- 更新 `docs/invariants.md` 与 `docs/decisions.md`，记录原 Workflow 内容优先级、浮点边界和新闻深度门禁。
- 完成后更新同主题 handoff 或新增内容修复 handoff，把本计划归档到 `docs/archive/plans/`，更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks

- 更深新闻段落可能把新闻页扩展到两页；可以接受，但不得缩短事实/传导链或压小字号来硬塞一页。
- 真实机构单季预测经常不披露；必须如实写未披露并解释评级逻辑，不得用聚合共识补造机构数字。
- 内容门禁只能验证结构和审计一致性，不能把弱来源变成强来源；证据选择仍必须优先公司/监管/真实机构材料并在生产样片中人工复核。
