# Handoff: SEC Enrichment OpenRouter Token Cap

- title: SEC Enrichment OpenRouter Token Cap
- status: done
- created_at: 2026-05-07
- updated_at: 2026-05-07
- owner: Codex
- related_files:
  - crates/hone-web-api/src/lib.rs
  - crates/hone-event-engine/src/engine.rs
  - crates/hone-event-engine/src/pollers/sec_enrichment.rs
  - docs/bugs/sec_enrichment_openrouter_max_tokens_402.md
- related_docs:
  - docs/archive/plans/sec-enrichment-openrouter-token-cap.md
  - docs/archive/plans/sec-enrichment-section-excerpts.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

SEC filing enrichment is now wired through its own OpenRouter provider with a completion budget capped by `event_engine.sec_filings.enrichment.max_summary_tokens`. This fixes the observed `HTTP 402` where a short filing summary request inherited the global `llm.openrouter.max_tokens` budget and OpenRouter preauthorized it as a 30k-output-token request.

Follow-up on the same day fixed the second `HTTP 402` mode: full 10-Q prompt input could still exceed the current key's prompt budget even after output tokens were capped. SEC enrichment now sends selected filing excerpts, not the entire cleaned filing text.

## What Changed

- Added `EventEngine::with_sec_filings_enrichment_provider(...)`.
- Kept global digest provider fallback for non-web-api embedders that only pass `with_global_digest_provider(...)`.
- Added `build_sec_filings_enrichment_provider(...)` in `hone-web-api`, using `OpenRouterProvider::from_config_with_max_tokens(...)`.
- Added tests for token-cap selection and separate provider wiring.
- Added `extract_filing_llm_context(...)` in `sec_enrichment`, which drops hidden inline XBRL/header noise and selects MD&A, strategic/capital/risk windows, Risk Factors, legal proceedings, or front-loaded 8-K exhibit narratives before the LLM call.
- Updated the SEC enrichment prompt/user message to say the input is selected excerpts rather than a full filing.

## Verification

- `cargo test -p hone-web-api sec_filings_enrichment --lib`
- `cargo test -p hone-event-engine sec_filings_enrichment --lib`
- `cargo check -p hone-web-api`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs crates/hone-event-engine/src/engine.rs`
- `cargo test -p hone-event-engine sec_enrichment --lib`

`cargo fmt --all -- --check` still fails on unrelated existing formatting drift in `bins/hone-cli/src/*`, `crates/hone-core/src/quiet.rs`, `crates/hone-event-engine/src/global_digest/fetcher.rs`, and `crates/hone-event-engine/src/router/policy.rs`; this handoff does not change those files.

Live OpenRouter smoke before the fix confirmed the boundary: `x-ai/grok-4.1-fast` succeeds with `max_tokens=800` and fails with `max_tokens=30000` under the current key limit.

Follow-up real-data POC used TEM/AMD/COHR 10-Q and TEM 8-K. TEM 10-Q selected excerpts succeeded live on `x-ai/grok-4.1-fast` with 3,170 prompt tokens, 798 completion tokens, and reported cost about `$0.0010`; the earlier full-input failure was `54381 > 6713` prompt tokens.

## Risks / Follow-ups

- This fix covers both SEC enrichment output-budget and input-budget failures. Global digest / mainline distill still use the global OpenRouter provider and may need their own output caps or semantic input reducers if they hit the same provider preauthorization boundary.
- S-1 and DEF 14A use the generic front narrative plus keyword-window path; they were not included in the 2026-05-07 real-data POC and should get their own samples if quality issues appear.
- A live SEC filing tick after restart is the best production confirmation, because local tests do not call SEC.gov or OpenRouter.

## Next Entry Point

Start from `crates/hone-event-engine/src/pollers/sec_enrichment.rs` for SEC filing input selection quality. Start from `crates/hone-web-api/src/lib.rs` if another event-engine LLM path needs a dedicated provider cap. Start from `docs/bugs/sec_enrichment_openrouter_max_tokens_402.md` for the observed production failure evidence.
