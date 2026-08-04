# Earnings Preview Expectation Model

- title: 财报前瞻预期判断流程全面校正
- status: archived
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
  - tests/regression/ci/test_earnings_research_pdf_markdown.sh
  - tests/regression/manual/test_earnings_research_pdf.sh
  - output/pdf/SNDK-FY2026-Q4-earnings-preview-new-process-2026-08-04-34dfabff.pdf
- related_docs:
  - docs/invariants.md
  - docs/decisions.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md
  - docs/archive/index.md

## Goal

把财报前瞻从“管理层指引与单点一致预期的静态比较”升级为通用、可审计的预期判断流程：锁定同一财季和时间截面，回测管理层指引偏差，补齐电话会与演示材料，判断催化剂是否已计入指引，形成独立收入/利润预测，并保证量化预测与超出、持平、低于结论闭环。

## Scope

- 已完成：收紧 `earnings-research` 的前瞻证据、建模、冲突处理和判断门槛，不添加公司特例。
- 已完成：新增私有 `preview_audit`，渲染前机械核验财季、预期来源、历史指引、催化剂计入状态、经营桥接、独立预测、中性带、方向结论和正文展示值。
- 已完成：新增正反例回归，覆盖缺少审计、预测与判断冲突、单一预期源不披露限制、历史样本不足不披露、缺少指引计入判断和正文数字不一致。
- 已完成：按新流程重新生成 SNDK FY2026 Q4 前瞻及带品牌 PDF；未修改入口、权限、runner 或文件代理。

## Validation

- `quick_validate.py skills/earnings-research` 通过。
- `python3 -m py_compile skills/earnings-research/scripts/render_report_pdf.py` 通过。
- `tests/regression/ci/test_earnings_research_pdf_markdown.sh` 通过。
- `tests/regression/manual/test_earnings_research_pdf.sh` 通过，A4、分享页和免责声明保持不变。
- 新 SNDK PDF 为 3 页 A4；全部页面 120 DPI 渲染和逐页视觉检查通过，无截断、重叠、乱码、错误断页或分享图异常。

## Documentation Sync

- 更新 `docs/invariants.md` 和现有 earnings workflow decision，固化预期判断和渲染审计边界。
- 更新同日 earnings handoff 与 `docs/archive/index.md`，本计划归档。
- 入口与模块边界未变，因此无需更新 `docs/repo-map.md`。

## Risks / Open Questions

- 多来源预期仍可能延迟或口径不一致；流程通过时间截面、区间和中性带约束，而不是声称存在唯一正确数字。
- 历史超指引只能作为管理层偏差先验，独立预测仍必须由当季价格、销量、组合、成本和催化剂计入状态支持。
- 渲染器能阻止数值与结论自相矛盾，但不能替代证据质量判断或保证预测结果正确。
