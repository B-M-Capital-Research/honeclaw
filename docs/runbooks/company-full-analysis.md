# Company full analysis operations

- title: 公司完整分析任务、进度与 PDF
- status: active
- created_at: 2026-09-14
- updated_at: 2026-09-14
- owner: HONE maintainers
- related_files: crates/hone-web-api/src/routes/company_analysis/; skills/earnings-research/scripts/render_report_pdf.py

## Runtime

The public chat administrator tools contain 公司完整分析. Only a company name/ticker is required. The server checks the current database admin flag and creates a UUID; client-supplied workflow/model/task IDs are not accepted on start.

Dependencies: configured PostgreSQL, OpenRouter provider credentials, FMP and search credentials/endpoint, and actor object storage. `agent.company_analysis_model` defaults to `google/gemini-3.1-pro-preview`, preserving the main original model. Credentials use existing provider configuration; do not place them in prompts or source assets.

The shared earnings renderer requires Python 3, Chromium and CJK fonts. Deploy its whole `scripts/vendor/` directory (Mermaid, KaTeX, fonts and licenses), preserving the executable script and the existing membership image path. macOS prefers the installed Playwright headless shell because desktop Chrome can print missing CJK glyphs. Linux uses system Chromium. All rendering scripts/fonts are offline; external HTML resource loads are blocked by CSP.

## API and progress callbacks

All endpoints are authenticated and administrator-only under `/api/public/company-analysis/tasks`:

- `POST {"company":"TEM"}` starts and returns the server task ID.
- `GET` lists the actor's latest 20 tasks.
- `GET /{task_id}` returns status, phase progress, completed stages and PDF readiness.
- `POST /{task_id}/resume` continues failed/expired jobs; running/completed jobs return their current state.
- `GET /{task_id}/pdf` downloads only that actor's completed PDF with private/no-store caching.

Original callbacks at 1/10/25/50/80/100 percent are migrated to native `Run.progress`/checkpoint callbacks, rather than posting to Bamang's external task service. The title/PDF/save stages add 85/90/95 percent. Progress is stage-weighted, not an ETA. 100% requires object persistence plus SHA-256 readback. There is no anonymous externally writable progress callback: a caller cannot forge completion.

`company_analysis_tasks` is created on first authorized use with serialized DDL. A partial unique index allows one running task per actor. Lease renewal is every 20 seconds with a two-minute expiry. Backend interruption becomes `interrupted` after expiry. Resume assigns a new attempt token, keeps the original research date and completed node outputs, and repeats only unfinished technical work. Attempt-specific PDF object names prevent expired workers from replacing the current attempt's PDF. Completed reports remain downloadable after restart.

## Diagnosis and verification

Check server logs by task ID and the actor-scoped task row. Checkpoint `usage` records each node's token counts/model/elapsed time; `outputs` retains original data inputs and completed text, and PDF metadata records the immutable object key/hash/size. Treat those records as actor data; do not dump them to shared logs or include credentials in handoffs.

A failed download must not be worked around by returning an arbitrary local PDF. Verify actor key, configured object-store provider (R2 uses `r2`/S3 signing), hash and service health. A renderer error can be resumed without regenerating completed research. Diagram/formula syntax failures preserve source text with a technical note; they do not trigger content rewriting.

Relevant automated checks: `cargo test -p hone-web-api --lib`, `bun run test:web`, `bun run --cwd packages/app typecheck`, `cd packages/app && bun run test:e2e public-chat-pdf-download.spec.ts --project public`, and `bash tests/regression/ci/test_earnings_research_pdf_markdown.sh`. Rust tests need the PostgreSQL environment in AGENTS.md.

For a real canary use an isolated PostgreSQL database/admin actor, explicit local-only dev login and disabled outbound/event services. Run TEM through the public API, observe actual progress, download the completed PDF, restart/re-download and compare SHA-256, render every page to inspect Chinese/tables/diagrams/assumptions. Assess content as a review, not a mechanical production gate.

## Deployment and rollback

Stage an immutable runtime revision plus the matching frontend and whole renderer asset tree. Company research participates in `/api/runtime/active-chat-runs`; drain before restart. Follow the existing revision/digest-bound runtime deployment procedure. PostgreSQL schema is additive; rollback can restore the previous runtime/frontend/skill symlinks without deleting task records or PDF objects. Completed PDFs are immutable; resumed attempts create a new key. Keep credentials out of artifact manifests.
