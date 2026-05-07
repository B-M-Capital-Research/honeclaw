# Handoff: SEC Enrichment OpenRouter Token Cap

- title: SEC Enrichment OpenRouter Token Cap
- status: done
- created_at: 2026-05-07
- updated_at: 2026-05-07
- owner: Codex
- related_files:
  - crates/hone-web-api/src/lib.rs
  - crates/hone-event-engine/src/engine.rs
  - docs/bugs/sec_enrichment_openrouter_max_tokens_402.md
- related_docs:
  - docs/archive/plans/sec-enrichment-openrouter-token-cap.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

SEC filing enrichment is now wired through its own OpenRouter provider with a completion budget capped by `event_engine.sec_filings.enrichment.max_summary_tokens`. This fixes the observed `HTTP 402` where a short filing summary request inherited the global `llm.openrouter.max_tokens` budget and OpenRouter preauthorized it as a 30k-output-token request.

## What Changed

- Added `EventEngine::with_sec_filings_enrichment_provider(...)`.
- Kept global digest provider fallback for non-web-api embedders that only pass `with_global_digest_provider(...)`.
- Added `build_sec_filings_enrichment_provider(...)` in `hone-web-api`, using `OpenRouterProvider::from_config_with_max_tokens(...)`.
- Added tests for token-cap selection and separate provider wiring.

## Verification

- `cargo test -p hone-web-api sec_filings_enrichment --lib`
- `cargo test -p hone-event-engine sec_filings_enrichment --lib`
- `cargo check -p hone-web-api`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs crates/hone-event-engine/src/engine.rs`

`cargo fmt --all -- --check` still fails on unrelated existing formatting drift in `bins/hone-cli/src/*`; this handoff does not change those files.

Live OpenRouter smoke before the fix confirmed the boundary: `x-ai/grok-4.1-fast` succeeds with `max_tokens=800` and fails with `max_tokens=30000` under the current key limit.

## Risks / Follow-ups

- This fix only caps SEC filing enrichment. Global digest / mainline distill still use the global OpenRouter provider and may need their own output caps if they hit the same provider preauthorization boundary.
- A live SEC filing tick after restart is the best production confirmation, because local tests do not call SEC.gov or OpenRouter.

## Next Entry Point

Start from `crates/hone-web-api/src/lib.rs` if another event-engine LLM path needs a dedicated provider cap. Start from `docs/bugs/sec_enrichment_openrouter_max_tokens_402.md` for the observed production failure evidence.
