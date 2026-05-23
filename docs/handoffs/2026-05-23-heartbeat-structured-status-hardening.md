# Heartbeat Structured Status Hardening

- title: Heartbeat Structured Status Hardening
- status: done
- created_at: 2026-05-23 12:13 CST
- updated_at: 2026-05-23 12:13 CST
- owner: Codex
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/scheduler_heartbeat_unknown_status_silent_skip.md`
  - `docs/bugs/README.md`
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_docs:
  - `docs/current-plan.md`
- related_prs:
  - N/A

## Summary

The remaining active heartbeat structured-status degradation item is fixed with parser and prompt hardening. The change widens deterministic compatibility for common nonstandard status values and internal-only no-op reasoning while preserving the boundary that arbitrary free text is not sent as a triggered alert.

## What Changed

- JSON status aliases such as `not_triggered`, `not met`, and `skip` now normalize to noop.
- JSON status aliases such as `condition_met`, `trigger`, and `alert` can deliver only when a usable `message` is present.
- Complete `<think>...</think>` outputs that explicitly say the condition is not met now normalize to `PlainTextNoop`.
- Heartbeat prompt now forbids tool/task/profile configuration fragments, including `set_immediate_kinds` and `cron_job`, as final output.

## Verification

- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`
- `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## Risks / Follow-ups

- This does not make arbitrary natural-language trigger descriptions deliverable; that remains intentionally blocked unless a structured status/message can be recovered.
- If future samples continue producing unrelated configuration prose, next work should focus on runner/tool routing constraints rather than adding one-off parser phrases.

## Next Entry Point

`docs/bugs/scheduler_heartbeat_unknown_status_silent_skip.md`
