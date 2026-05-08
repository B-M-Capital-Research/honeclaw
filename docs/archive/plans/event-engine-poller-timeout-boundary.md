# Event Engine Poller Timeout Boundary

- title: Event Engine Poller Timeout Boundary
- status: archived
- created_at: 2026-05-09
- updated_at: 2026-05-09
- owner: bug-2 automation
- related_files:
  - `crates/hone-event-engine/src/spawner.rs`
  - `docs/bugs/archive/event_engine_poller_cadence_stall_without_restart.md`
  - `docs/bugs/README.md`
- related_docs:
  - `docs/archive/index.md`

## Goal

Close the historical P2 event-engine cadence bug where one stuck `poll().await` / `run_once().await` could suppress `poller ok` output and event creation for much longer than the configured interval.

## Scope

- Add a schedule-aware timeout around each unified event-source poller tick.
- Keep the existing next-tick retry model: timeout is recorded as a failed run and the loop proceeds to the next scheduled tick.
- Do not add source-specific network special cases or production-log dependent behavior.

## Validation

- `cargo test -p hone-event-engine spawner::tests --lib -- --nocapture`
- `cargo test -p hone-event-engine pollers::earnings_surprise::tests::quality_review_applies_successful_earnings_event --lib -- --nocapture`
- `cargo test -p hone-event-engine --lib`
- `cargo check -p hone-event-engine --tests`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-event-engine/src/spawner.rs crates/hone-event-engine/src/pollers/earnings_surprise.rs`

## Documentation Sync

- Update the poller cadence bug document to `Fixed`.
- Update `docs/bugs/README.md` historical rows, including two stale rows whose entry docs were already `Closed`.
- Add this archived plan to `docs/archive/index.md`.

## Risks / Open Questions

- The timeout bounds are intentionally conservative: fixed-interval pollers get `2x` interval clamped to `30s..300s`; cron-aligned pollers get `300s`.
- This does not replay historical missed events; it prevents future indefinite cadence stalls from one stuck tick.
