# Event-engine Push Quality Hardening

- title: Event-engine Push Quality Hardening
- status: done
- created_at: 2026-05-08
- updated_at: 2026-05-08
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/pollers/analyst_grade.rs
  - crates/hone-event-engine/src/router/dispatch.rs
  - crates/hone-event-engine/src/store.rs
  - crates/hone-event-engine/src/pollers/rss.rs
  - crates/hone-event-engine/src/pollers/news.rs
  - crates/hone-event-engine/src/digest/tests.rs
  - docs/event-review/2026-04-30.md
  - docs/event-review/2026-05-07.md
- related_docs:
  - docs/archive/index.md
  - docs/handoffs/2026-04-23-event-engine-push-quality.md

## Goal

Reduce recurring event-engine push quality issues found in daily reviews:

- same-source analyst-grade fanout creating multiple immediate pushes for one article;
- generic Zacks/opinion stock templates crowding user-visible digests or sinks;
- high-value RSS stories missing actor routing because trusted feeds often lack ticker symbols.

## Scope

- Added deterministic routing protection for analyst-grade rows that share ticker and source article URL.
- Added precision-first RSS title entity linking for a small company alias set proven by real `no_actor` samples, with linked rows kept at digest severity rather than immediate.
- Locked existing Zacks generic demotion behavior with regression tests.
- Did not add LLM calls or broad summary/body-based entity matching.

## Validation

- `cargo test -p hone-event-engine --lib` passed, 475 passed / 13 ignored.
- `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib` passed.
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh` passed, fixture 43 items / 15 LLM items loaded.
- `rustfmt --edition 2024 --check crates/hone-event-engine/src/digest/tests.rs crates/hone-event-engine/src/event.rs crates/hone-event-engine/src/pollers/analyst_grade.rs crates/hone-event-engine/src/pollers/news.rs crates/hone-event-engine/src/pollers/rss.rs crates/hone-event-engine/src/router/dispatch.rs crates/hone-event-engine/src/router/tests.rs crates/hone-event-engine/src/store.rs` passed.
- `cargo fmt --all -- --check` was attempted after scope cleanup and fails on out-of-scope existing formatting debt in `bins/hone-cli`, `crates/hone-core/src/quiet.rs`, `crates/hone-event-engine/src/global_digest/fetcher.rs`, and `crates/hone-event-engine/src/router/policy.rs`; those formatter-only diffs were not kept in this task.

## Documentation Sync

- Removed this task from `docs/current-plan.md` after completion.
- Appended the 2026-05-08 follow-up to `docs/handoffs/2026-04-23-event-engine-push-quality.md`.
- Added this archived plan to `docs/archive/index.md`.

## Risks / Open Questions

- RSS title aliasing is intentionally conservative. Summary-only or URL-only hits remain unlinked to avoid broad market false positives.
- Analyst fanout suppression relies on same source article URL being persisted in `payload.newsURL` or `event.url`; rows without a URL still use the existing firm-aware cooldown behavior.
