# Report quick-nav reliability and shared valuation prompts

- title: 三类报告（公司完整分析 / 财报分析 / 财报前瞻）可靠交付与 2026-10-11 估值 Prompt 修订
- status: done (deployed 2026-10-12); complete real-model report canaries remain unverified
- created_at: 2026-10-11
- updated_at: 2026-10-12
- owner: Claude (user-requested review)
- related_files: skills/earnings-research/SKILL.md; skills/earnings-research/scripts/render_report_pdf.py; crates/hone-web-api/src/routes/company_analysis/{workflow.rs,mod.rs,store.rs,revisions/}; crates/hone-web-api/src/routes/mod.rs; crates/hone-channels/src/{agent_session/earnings_salvage.rs,agent_session/core.rs,tool_trace.rs,earnings_materials.rs,turn_builder.rs}; crates/hone-tools/src/skill_tool.rs; crates/hone-llm/src/resolver.rs; packages/app/src/{components/company-analysis-dialog.tsx,lib/company-analysis.ts,lib/public-chat.ts,lib/api.ts,pages/chat.tsx}
- related_docs: docs/decisions.md#d-2026-10-11-02-report-quick-nav-reliability-and-shared-valuation-rules; docs/invariants.md; docs/runbooks/company-full-analysis.md; crates/hone-web-api/src/routes/company_analysis/revisions/README.md
- related_prs: none; application commit `1ca6197c5c9efb1fa4affe6b7a458fd25cacb8fd` on main

## Summary

Users often ended up with no report. The main causes and fixes:

| Path | Cause of "no report" | Fix |
|---|---|---|
| 财报前瞻/分析 | Host discarded a finished report whenever the PDF step did not complete (renderer failure, uncertain script state, timeout after a successful render) | Host closes from the report submitted to the renderer: reuse completed PDF → host render → Markdown with "PDF 未生成" note. Prose never submitted to the renderer still fails closed |
| 财报前瞻/分析 | Whole report passed as one argv string; Linux caps one argument at 128 KiB (~43k CJK chars) | `script_payload` goes through a private temp JSON file (`--input`) |
| 财报前瞻/分析 | Fresh-retry draft read from top-level `report_markdown`, real calls nest it in `script_payload` | Shared extractor reads the nested shape (string or object) |
| 财报前瞻/分析 | Model re-typed the full report after rendering, burning the time budget the host then discards | SKILL.md + system override: reply with one confirmation line after success |
| 财报前瞻/分析 | Unbounded prefetch (`links`, 8 × 60k pages); `mode:`/`company:` parsed from any line incl. appended context | Drop `links`, 200k total page budget, 45 s per page; parse only the server parameter block |
| 公司完整分析 | 120 s *total* HTTP deadline also bounds the streamed body; long sections cut mid-stream; `Length` at 16k tokens failed 3 identical times | Long-generation provider (900 s, 32k tokens), 240 s idle / 720 s per call, continuation on `Length`, 4 attempts with backoff |
| 公司完整分析 | Transient DB error in heartbeat/checkpoint write killed the run; diagram/title node failure killed the run | Retried writes, tolerant heartbeat, one automatic continuation from checkpoints, optional diagram/title |
| 公司完整分析 | PDF-only delivery; a render failure hid a finished report | `GET /tasks/{id}/markdown` + `markdown_ready`; dialog offers Markdown download, shows the latest (incl. failed) task on open, opens during a chat reply |
| PDF renderer | Diagram layout missing its signal failed both attempts | Script-free last attempt prints diagram/formula source as text (warning) |
| Chat UI | 403/400/503 before the stream was treated as a disconnect and erased by history restore; PDF card relied on one history fetch | Show the server's refusal on the card; history restore retries after normal completion; HTML responses rejected as downloads |

Prompts: the user-supplied “HONE 三类报告完整 Prompt — 估值逻辑修订与使用说明” (2026-10-11) is integrated the way its own usage notes recommend for a staged pipeline — valuation rules injected at each mode's valuation position, original prompts/stage order kept, full single-run prompts not pasted into single-chapter nodes. One rules block (`## Valuation rules — shared HONE revision` in `SKILL.md`) is shared by all three; company analysis reads it at compile time. Non-valuation chapters only gained short fact-discipline notes from the same document (consensus definition, quote verification, missing market-reaction wording, industry-number scope, news-window coverage, no fabricated "previous target"). Nothing removed; the company snapshot stays verbatim and hash-pinned.

## Verification

Local, macOS, PostgreSQL 16 on 127.0.0.1:5432 (`honeclaw_test`; `HONE_POSTGRES_*` exported with a non-empty placeholder password — an empty one breaks fallback after an mcp_bridge test clears `DATABASE_URL`).

- `cargo test --lib`: hone-channels 887 passed / 1 ignored; hone-web-api 402 / 2 ignored; hone-tools 235 / 2 ignored; hone-llm 42. New tests cover: continuation on `Length`, incomplete/filtered streams, single-pass revision binding, snapshot hashes, valuation node inputs, optional diagram, Markdown readiness (PG), failure info, nested renderer payload, salvage ranking, host render contract, completed-PDF-then-timeout salvage through `AgentSession`, payload file transport (>128 KiB report, temp file removed), parameter-block parsing, prefetch bounds.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passes; rustfmt clean on every changed Rust file.
- Regressions: `test_earnings_research_pdf_markdown.sh` (now incl. script-free fallback), `test_skill_runtime_stage_consistency.sh`, `test_skill_runtime_tools.sh` pass.
- Web: `bun run test:web` 669/669; `bun run --cwd packages/app typecheck`; Playwright `public-chat-pdf-download.spec.ts --project public` 6/6 (adds Markdown download and failed-PDF-with-Markdown cases).
- Real Chromium smoke: a Mermaid report rendered normally (no warnings); the script-free variant printed a valid PDF with the diagram source.
- Not done: no real model run (no production credentials used); no deployment.

## Risks / Follow-ups

- Run real canaries before relying on prompt quality: one 公司完整分析, one 财报分析, one 财报前瞻 (TEM or a cyclical name such as MU to exercise normalization). Review valuation tables, scenario logic, reverse valuation and length/cost/latency. Treat findings as prompt/retrieval work, never as new gates.
- Longer valuation sections raise output tokens, cost and duration. The company model must accept `max_tokens=32000` (Gemini 3.1 Pro, Claude Opus/Sonnet do). Production earnings runs use the OpenCode route model from config (handoff 2026-09-13 notes `anthropic/claude-opus-4.8`).
- Salvaged earnings output is the model's last submitted draft; when the host renders it, it is the same Markdown the model asked to render. A Markdown-only closure is labelled at the top of the answer.
- Deploy the binary and the `skills/earnings-research/` tree from the same revision: company analysis compiles the valuation rules from `SKILL.md`, and the renderer gained the `--input`/script-free behaviour the new `skill_tool` transport relies on (`--input` already existed).
- Not addressed (from the review): earnings runs are in-memory and not resumable after a backend restart; only the first OpenRouter key is used for the OpenCode route; chat polling stops after four failed restores; other streaming paths still use the interactive 120 s total HTTP deadline.

## Next Entry Point

`docs/decisions.md` D-2026-10-11-02, then `crates/hone-channels/src/agent_session/earnings_salvage.rs` (earnings closure) and `crates/hone-web-api/src/routes/company_analysis/workflow.rs` (company transport/revisions). Prompt text: `skills/earnings-research/SKILL.md` and `crates/hone-web-api/src/routes/company_analysis/revisions/`.

## Production rollout — 2026-10-12

- status: done / deployed. This section supersedes the earlier “no deployment” status; full real-content canaries remain a stated limitation.
- Runtime, exact-revision earnings skill/vendor/share-image tree and public frontend deployed as `1ca6197c5c9efb1fa4affe6b7a458fd25cacb8fd`. Full release provenance, test results, public chunk hashes, health acceptance and rollback are retained in `docs/handoffs/2026-10-11-public-site-research-insights-email-login.md` and the archived public-site plan.
- Readiness fixes tightened the host-render salvage directory permissions, restored a per-stream-chunk size bound, isolated the secret-environment test process and made payload cleanup check its own file. The existing production query/bridge repair was retained.
- Final workspace: 2,909 Rust tests passed; 97 additional PostgreSQL tests passed; Web669, Worker45, auth E2E3 and PDF/Markdown E2E6 passed. Linux compile/Rust/PG checks passed; overall CI retains only the unchanged ten finance static baseline failures described in the other handoff.
- The deployed Linux renderer ran as the service user under `env -i` using the repository-shaped release tree. A Chinese technical sample with table, formula and Mermaid rendered with no warnings; both PDF pages were inspected, including the resolved membership share image. This was a renderer smoke, not a generated investment report.
- Real authenticated connectivity probes accepted `max_tokens=32000` for `google/gemini-3.1-pro-preview` and configured `anthropic/claude-opus-4.8`; both returned the requested model and finished normally. Combined reported cost was $0.002586. No model credentials were copied into release assets or logged.
- Full 公司完整分析 / 财报分析 / 财报前瞻 model runs, valuation quality/cost/latency review and their real artifact persistence/redownload canaries have not been rerun for the revised prompts. Continue with the isolated-database/admin-actor procedure in `docs/runbooks/company-full-analysis.md`; do not interpret transport/renderer checks as content-quality acceptance.
