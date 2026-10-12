# HONE prompt revisions (2026-10-11)

`../original/` stays a verbatim snapshot of the Dify branch; its SHA-256 values are pinned by a unit test. Revisions are layered here instead of editing the snapshot, so the original text and the delta stay reviewable separately.

- `valuation.system.md` / `valuation.user.md` replace the `全跑完-美` node (chapter 5). They keep the original role, method list, `## 5. 估值分析` / 5.1 / 5.2 / 5.3 format, quoted assumptions, plain-text formulas and the no-trading-advice rule, and add the shared HONE valuation rules. `{{HONE_VALUATION_RULES}}` is filled from the `## Valuation rules — shared HONE revision` block in `skills/earnings-research/SKILL.md`, the single source used by all three report types.
- The valuation node also receives the raw quote/profile/financial JSON and the consensus/price-target JSON, so price date, share count, cash, debt, cash flow and SBC can be checked directly instead of only through the formatter node's table.
- Every other node keeps its original system/user prompt. `workflow.rs::node_addendum` adds one short system note per node (time scope, separating disclosed facts from inference, not inventing missing history/quotes/numbers). Chapter structure and order are unchanged.
- Placeholders use `{{node.variable}}` and are substituted in a single pass; substituted source text is never re-read as a template.

Source of the revision: the user-supplied "HONE 三类报告完整 Prompt — 估值逻辑修订与使用说明" (2026-10-11). Its single-run full-report prompts were not pasted into single-chapter nodes, as that document itself advises.
