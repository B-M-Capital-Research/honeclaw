# HONE prompt revisions

`../original/` stays a verbatim snapshot of the Dify branch; its SHA-256 values are pinned by a unit test. Revisions live here instead of editing the snapshot.

## Current: 公司完整分析 Prompt V4 (2026-10-13)

- `v4/prompt.md` is the verbatim “可直接复制的完整 Prompt” from the user-supplied 《HONE 公司完整分析通用 Prompt — 所有公司采用 Ontology 框架 AI 价值与估值合理性增强版 V4》 (role paragraph, sections 一 to 十 and the chapter outline 1 to 5). Its SHA-256 is pinned by `v4::tests::the_v4_prompt_is_kept_verbatim_and_fully_sectioned`; replace the file (and the hash) to adopt a later version.
- `../v4.rs` maps it onto the staged task without rewriting it. Every content node gets the role paragraph and the shared research chain (一 任务输入, 二 Ontology 研究链, 三 AI 影响, 十 复核); chapter 2 adds 四 方法路径; chapter 4 adds 四/五/七; chapter 5 adds 四 to 八. Each node also gets only its own slice of the chapter outline and a short “HONE 分段生成说明” (its slice, consistency with earlier chapters, the materials actually provided — no Ontology records, old models or calculation tools — and heading/formula format). The file-delivery paragraph of 十 is left to the host.
- Nodes and order: 公司简介 ∥ 产品与商业模式 (2.1–2.4) ∥ 管理团队 → 竞争与价值传导 (2.5–2.8, reads 2.1–2.4) → 财务分析 (4.1–4.6 with the forward model, reads chapters 1–2) → 估值分析 (5.1–5.6, reads chapters 1, 2, 4 and the raw data) → 标题与摘要 ∥ 附录. The original 公司财务指标获取 formatter and the 产品分析附图 illustration (fed the V4 product section) are kept; the illustration, title/summary and appendix are optional and never withhold the report.
- Two searches were added for V4's demand/supply/value-capture chain (products, billing units and customers; capacity, competitors, customer self-build and AI impact), after the five original query templates.

## Superseded

- 2026-10-11 valuation revision (`valuation.system.md` / `valuation.user.md`, the per-node notes and the shared `SKILL.md` valuation rules for chapter 5). V4 replaces it for company analysis; the earnings modes still use the `SKILL.md` rules. See `docs/decisions.md` D-2026-10-13-01.
