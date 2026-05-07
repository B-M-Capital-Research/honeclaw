# SEC Enrichment OpenRouter Token Cap

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
  - docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md
  - docs/archive/index.md

## Goal

Make SEC filing LLM enrichment use `event_engine.sec_filings.enrichment.max_summary_tokens` as its OpenRouter completion budget, instead of inheriting the global `llm.openrouter.max_tokens` budget. This keeps long filing input context available while avoiding OpenRouter per-request preauthorization failures for 30k output-token budgets.

## Scope

- Add a separate SEC filing enrichment LLM provider path with a capped completion budget.
- Preserve the existing global digest provider and global `llm.openrouter.max_tokens` behavior for other paths.
- Add tests for provider cap selection and EventEngine builder wiring.

## Validation

- Passed: `cargo test -p hone-web-api sec_filings_enrichment --lib`
- Passed: `cargo test -p hone-event-engine sec_filings_enrichment --lib`
- Passed: `cargo check -p hone-web-api`
- Passed: `rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs crates/hone-event-engine/src/engine.rs`
- Not blocking: `cargo fmt --all -- --check` fails on unrelated existing `bins/hone-cli/src/*` formatting drift.

Live LLM smoke confirmed `x-ai/grok-4.1-fast` succeeds with `max_tokens=800` and fails with `max_tokens=30000` under the current OpenRouter key weekly limit.

## Documentation Sync

- Added `docs/bugs/sec_enrichment_openrouter_max_tokens_402.md`.
- Added `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md`.
- Removed the task from `docs/current-plan.md` and archived this plan.
- Updated `docs/archive/index.md`.

## Risks / Open Questions

- This fix only caps SEC filing enrichment output tokens. Global digest and mainline distill still use the global OpenRouter provider unless separately capped later.
- A live SEC filing tick after deployment is still useful as production confirmation.
