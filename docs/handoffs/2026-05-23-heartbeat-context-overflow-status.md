# Heartbeat Context Overflow Status Boundary

- title: Heartbeat Context Overflow Status Boundary
- status: done
- created_at: 2026-05-23 12:04 CST
- updated_at: 2026-05-23 12:04 CST
- owner: Codex
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/scheduler_heartbeat_context_window_limit_no_recovery.md`
  - `docs/bugs/README.md`
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_docs:
  - `docs/current-plan.md`
- related_prs:
  - N/A

## Summary

Reopened P2 heartbeat context overflow bug is fixed at the scheduler status boundary. The runner may still fail if a heartbeat prompt exceeds the provider context window, but that failure is no longer recorded as a legitimate noop.

## What Changed

- Removed the `ContextOverflowNoop` branch from heartbeat runner error handling.
- Context overflow errors now keep `ScheduledTaskExecution.error`.
- Metadata now includes `failure_kind=context_window_overflow` and `parse_kind=ContextOverflowError`.
- Channel scheduler handlers already map any non-deliverable execution with `error.is_some()` to `execution_failed + skipped_error`, so no channel-specific status rewrite was needed.

## Verification

- `cargo test -p hone-channels heartbeat_context_overflow_error_is_not_classified_as_noop --lib -- --nocapture`
- `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## Risks / Follow-ups

- This fix makes repeated overflow visible and auditable; it does not shrink large heartbeat prompts or add automatic chunking.
- If future samples show frequent `context_window_overflow`, next work should focus on prompt budget metrics, request summarization, or splitting multi-symbol heartbeat jobs.

## Next Entry Point

`docs/bugs/scheduler_heartbeat_context_window_limit_no_recovery.md`
