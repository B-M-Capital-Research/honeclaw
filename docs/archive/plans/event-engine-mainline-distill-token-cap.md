# Event Engine Mainline Distill Token Cap

- title: Event Engine Mainline Distill Token Cap
- status: archived
- created_at: 2026-05-09
- updated_at: 2026-05-09
- owner: bug-2 automation
- related_files:
  - `crates/hone-web-api/src/lib.rs`
  - `docs/bugs/event_engine_mainline_distill_openrouter_402.md`
  - `docs/bugs/README.md`
- related_docs:
  - `docs/archive/index.md`

## Goal

Fix the P2 bug where event-engine mainline distill reused the global OpenRouter completion budget and could trigger HTTP 402 for a short 1-2 sentence background-summary task.

## Scope

- Add a dedicated mainline distill LLM provider in `hone-web-api`.
- Cap mainline distill completion tokens at `1200`.
- Keep global digest curator provider behavior unchanged.
- Do not add one-off handling for a single OpenRouter balance window.

## Validation

- `cargo test -p hone-web-api mainline_distill_uses_short_completion_budget --lib -- --nocapture`
- `cargo check -p hone-web-api --tests`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs`

## Documentation Sync

- Mark `docs/bugs/event_engine_mainline_distill_openrouter_402.md` as `Fixed`.
- Move the bug out of `docs/bugs/README.md` active queue and add it to the fixed table.
- Add this archived plan to `docs/archive/index.md`.

## Risks / Open Questions

- The fix bounds completion budget only. If OpenRouter later rejects on input prompt budget, add profile markdown excerpting or retry with smaller section-aware input.
- Failure classification for skipped tickers can still be improved separately.
