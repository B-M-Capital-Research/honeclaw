# Event Engine Push Quality Full Fix

- title: Event Engine Push Quality Full Fix
- status: in_progress
- created_at: 2026-04-23
- updated_at: 2026-04-23
- owner: Codex
- related_files:
  - `crates/hone-event-engine/src/pollers/price.rs`
  - `crates/hone-event-engine/src/pollers/macro_events.rs`
  - `crates/hone-event-engine/src/pollers/`
  - `crates/hone-event-engine/src/router.rs`
  - `crates/hone-event-engine/src/digest.rs`
  - `crates/hone-event-engine/src/event.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/news_classifier.rs`
  - `crates/hone-event-engine/src/sinks/`
  - `crates/hone-event-engine/src/pollers/news.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `crates/hone-core/src/config/event_engine.rs`
  - `config.example.yaml`
  - `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
  - `tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
  - `.agents/skills/event-engine-baseline-testing/SKILL.md`
- related_docs:
  - `docs/bugs/event_engine_high_macro_events_unrouted.md`
  - `docs/bugs/event_engine_social_source_decode_failures.md`
  - `docs/bugs/event_engine_window_convergence_upgrade_burst.md`
  - `/Users/bytedance/.codex/automations/event-engine/memory.md`

## Goal

Close the full 24-item event-engine push-quality backlog without overfitting to one noisy Telegram batch. Each fix must be reproduced with mock data first, implemented generically, and covered by regression tests. Live LLM API calls may be used only for manual classifier validation; CI proof must use deterministic mocks.

## Scope

- Event modeling: timestamp semantics, event identity, stale data, macro calendar due windows, earnings preview lead time.
- Routing: immediate versus digest, severity gating, per-category cap/cooldown, decision reasons.
- Digest: curation, dedupe, topic memory, source quality, portfolio/watchlist relevance, digest window policy.
- Preferences: portfolio-first defaults, quiet mode, source/channel controls, price thresholds by direction and exposure.
- Observability: structured logs, delivery status semantics, buffer/flush evidence, poller degraded state, classifier fallback markers.

## Execution Rules

- Keep a single master checklist here; do not rely on chat-only todos.
- Do not mark an item `done` until code, mock regression, and targeted verification pass.
- Prefer one theme per commit, but allow tightly coupled items to share a commit when rollback remains clear.
- Before moving to the next theme, update this plan with status and verification.
- If an item needs product judgment rather than code only, implement the conservative default and leave the configurable surface explicit.

## Master Checklist

| # | Item | Status | Next proof |
|---|---|---|---|
| 1 | Price quote timestamp and stale protection | done_uncommitted | `price::tests` and `hone-event-engine --lib` |
| 2 | Closing move versus intraday alert | done_uncommitted | close quote + router override tests |
| 3 | Personal price threshold sensitivity | done_uncommitted | system floor, large-position, directional threshold tests |
| 4 | Social source digest noise | done_uncommitted | social cap, source allow/block, source-quality digest score tests |
| 5 | Repeated digest news | done_uncommitted | title dedupe, similar-title clustering, recent topic memory tests |
| 6 | Window convergence over-upgrade | done_uncommitted | hard-signal correlation and per-symbol/per-tick cap tests |
| 7 | Macro high rule gating | done_uncommitted | FMP impact + country + event type fixtures |
| 8 | Macro high immediate due window | done_uncommitted | future macro digest; near-window macro immediate test |
| 9 | Legal ad/news demotion | done_uncommitted | poller template + router hard-demotion tests |
| 10 | High news delivery evidence | done_uncommitted | no_actor/filter/cap/cooldown/fail plus digest_item logs |
| 11 | Digest value ranking | done_uncommitted | source quality and high-value event score tests |
| 12 | Digest window policy | done_uncommitted | min-gap duplicate-window suppression test |
| 13 | Pushed topic memory | done_uncommitted | 24h similar-topic suppression test |
| 14 | Social source/channel preferences | done_uncommitted | source allow/block preference tests |
| 15 | Rich sink/digest logs | done_uncommitted | event_id/source/symbols/item_ids/status logging paths |
| 16 | Digest buffer/flush evidence policy | done_uncommitted | buffer path/rotated flushed timestamp logs |
| 17 | Delivery `sent` semantics | done_uncommitted | dryrun status cannot equal sent test |
| 18 | Poller decode/fetch degraded logs | done_uncommitted | poller/source/url_class/degraded log fields |
| 19 | LLM classifier fallback | done_uncommitted | provider error and unparseable response tests |
| 20 | High cap/cooldown by category | done_uncommitted | price/news/filing/earnings/macro category bucket store tests |
| 21 | Portfolio-first default | done_uncommitted | symbol news remains portfolio/watchlist; global social/macro digest-gated |
| 22 | Quiet mode preset | done_uncommitted | quiet mode demotes news but keeps SEC immediate test |
| 23 | Directional/exposure price thresholds | done_uncommitted | up/down threshold and large-position tests |
| 24 | Earnings preview lead time and transcript split | done_uncommitted | far preview Low digest + transcript dedicated-kind tests |

## Validation

- Targeted unit tests per item under the touched Rust module.
- `cargo fmt --all -- --check`.
- `cargo test -p hone-event-engine --lib`.
- `cargo test -p hone-core --lib` when config changes.
- Add or extend regression fixtures only with mock data; live LLM validation must stay manual/non-blocking.

Latest validation:

- 2026-04-23: `cargo fmt --all -- --check`
- 2026-04-23: `cargo test -p hone-event-engine --lib` → 205 passed, 13 ignored
- 2026-04-23: `cargo test -p hone-core --lib` → 52 passed
- 2026-04-23: `cargo check -p hone-web-api`
- 2026-04-23: targeted tests for macro due window, source preferences, topic memory, min-gap, dryrun status, category budgets, directional price thresholds, quiet mode, legal demotion, and convergence caps
- 2026-04-23: live OpenRouter/FMP manual eval with `x-ai/grok-4.1-fast`: 120 FMP portfolio-news items, 12 uncertain-source items after source demotion, 12/12 parseable, 4 yes / 8 no, all routed to digest Medium/Low (no immediate sink)
- 2026-04-23: `earnings_call_transcript` split added; transcript fixtures now become `EventKind::EarningsCallTranscript` and can be allow/block/immediate/disabled independently
- 2026-04-23: saved the live news set as `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`; added an offline fixture stability unit test and a manual OpenRouter drift script. The manual script intentionally uses a title-only baseline to avoid storing FMP article bodies; the fixture keeps the original live-with-text answer separately where it differed.
- 2026-04-23: added repository skill `.agents/skills/event-engine-baseline-testing` to document the event-engine test matrix, live LLM baseline rerun commands, and rules for adding new baseline samples
- 2026-04-23: screened non-Google/non-OpenAI/non-Anthropic OpenRouter models for news classifier use. `amazon/nova-lite-v1` won on the saved baseline: 5 passes x 12 items = 60 calls, 0 drift, 0 parse errors, reported cost `$0.002563`, avg latency 1.44s, p95 2.44s. Updated event-engine default/recommended classifier model to `amazon/nova-lite-v1`.

## Documentation Sync

- This plan is now the active cross-session tracker because the task spans multiple modules, behavior changes, and more than one turn.
- Update `docs/current-plan.md` while active.
- When complete, add a handoff with final behavior, verification, rollback notes, and archive this plan through `docs/archive/index.md`.
- Bug-specific docs under `docs/bugs/` should only be updated when closing or replacing an existing anomaly report.

## Risks / Open Questions

- Some preference defaults affect user-facing notification volume; use conservative digest-first behavior where uncertain.
- Current branch is `main` but has unrelated staged changes and is behind `origin/main`; commits must avoid mixing staged anomaly docs with implementation batches.
- Current implementation writes per-item `digest_item` delivery rows and uses recent delivered item events for 24h topic suppression.
