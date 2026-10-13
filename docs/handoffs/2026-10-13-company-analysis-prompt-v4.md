# Company full analysis Prompt V4

- title: 公司完整分析改用 Prompt V4（全章节）与可复用的真实 canary
- status: done (code + local verification); real canary pending
- created_at: 2026-10-13
- updated_at: 2026-10-13
- owner: Claude (user request: "Replace the Deep Analysis version with this one and see how it works")
- related_files: crates/hone-web-api/src/routes/company_analysis/{v4.rs,workflow.rs,mod.rs,revisions/v4/prompt.md,revisions/README.md}; packages/app/src/components/company-analysis-dialog.tsx; tests/regression/manual/test_company_analysis_canary.sh; skills/earnings-research/SKILL.md (note only)
- related_docs: docs/decisions.md#d-2026-10-13-01-company-full-analysis-adopts-prompt-v4-across-all-chapters; docs/runbooks/company-full-analysis.md; docs/handoffs/2026-10-11-report-reliability-and-valuation-prompts.md
- related_prs: none (uncommitted)

## Summary

The user-supplied 《HONE 公司完整分析通用 Prompt V4》 replaces the company-analysis prompts for all five chapters. The copyable prompt is stored verbatim (`revisions/v4/prompt.md`, hash-pinned) and `v4.rs` maps it onto the staged task instead of running one huge call: each node gets V4's role paragraph and shared research chain (一 任务输入, 二 Ontology 研究链, 三 AI 影响, 十 复核), the method/model/valuation sections its chapter needs, only its own chapter outline, and a short segmentation note (its slice, consistency with earlier chapters, the materials actually provided, heading/formula format).

Pipeline: data (5 original + 2 new V4 searches, quote/profile/financials, consensus, shares) → original financial formatter → 公司简介 ∥ 产品与商业模式 (2.1–2.4, then optional illustration) ∥ 管理团队 → 竞争与价值传导 (2.5–2.8) → 财务分析 (4.1–4.6 forward model) → 估值分析 (5.1–5.6) → 标题与摘要 ∥ 附录 (both optional) → assembly → PDF. Later chapters read the earlier ones, so the forward model and valuation use the same mapping and numbers.

Removed: the 2026-10-11 company valuation revision files and per-node notes; company analysis no longer reads the `SKILL.md` valuation rules (earnings modes still do). Node persistence now goes through `workflow::Checkpoints`, which lets the manual canary run the production pipeline code without a task row or OSS. A task whose `_report` already exists goes straight to rendering on resume.

## Verification

- `cargo test -p hone-web-api --lib`: 403 passed / 4 ignored (canary and prompt dump are the new ignored ones). New tests: verbatim/sectioned prompt hash, per-node framework and slice (no file-delivery paragraph, no foreign outline items), missing-input wiring error, chapter order with optional gaps, head fallback, end-to-end DAG with a node-keyed fake model (inputs flow 2.1–2.4 → 2.5–2.8 → 4 → 5 → summary/appendix, 10 model calls, resume with `_report` makes none), optional-node degradation and chapter-failure checkpointing.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`, rustfmt, `bun run test:web` 669/669, typecheck, earnings renderer regression, hone-channels earnings/turn-builder tests 34/34.
- Composed node prompts were dumped (`company_analysis_prompt_dump`) and read; two scope-text conflicts in the title/summary and appendix nodes were fixed before finishing.
- Not done: no real model run — this machine has no OpenRouter/FMP/search keys. Use `tests/regression/manual/test_company_analysis_canary.sh` with a credentialed config.

## Risks / Follow-ups

- Run at least two real canaries of different business types (e.g. MU for a cycle, and a software or operator name) and review: chapter consistency, the 2.8 driver table → 4.6 model → 5.x valuation chain, core range/median/price position, reverse valuation, length, tokens, cost and wall time. Fix findings in prompt/retrieval, not with gates.
- Longer sequential chain and longer chapters mean slower, costlier runs than the 2026-10-11 version.
- V4 asks hosts to remove conflicting rules; the earnings `SKILL.md` block still says software/platforms default to an economic cash-flow approach. Decide whether earnings modes should adopt V4's framework.
- Deploying this changes production company reports immediately for administrators; tasks started under the old prompts regenerate their chapters under V4 if resumed.

## Next Entry Point

`crates/hone-web-api/src/routes/company_analysis/revisions/README.md`, then `v4.rs` (prompt mapping) and `workflow.rs::generate_report` (DAG). Canary: `docs/runbooks/company-full-analysis.md`.
