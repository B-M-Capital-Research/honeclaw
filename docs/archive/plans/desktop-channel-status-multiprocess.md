# Desktop 渠道监听状态与多进程 PID 对齐

- title: Desktop 渠道监听状态与多进程 PID 对齐
- status: archived
- created_at: 2026-03-28
- updated_at: 2026-04-16
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/runbooks/desktop-dev-runtime.md`

## Goal

Align desktop channel status reporting with the real multi-process runtime so operators can see and clean up redundant processes safely.

## Scope

- Heartbeat now reports the primary path from the backend.
- `/api/channels` now aggregates multi-process status and exposes PIDs.
- The desktop badge dropdown already has a quick action to clean extra processes.
- Archived on 2026-04-16 because the remaining text was only a generic validation tail; any future work should reopen as a narrower bug or validation task instead of staying in the active backlog indefinitely.

## Validation

- Archived without additional execution in this cleanup pass. Reopen with a concrete repro or acceptance checklist if multi-process status regresses.

## Documentation Sync

- Archived from the active index on 2026-04-16.
- Future follow-up should add a focused handoff or bug entry rather than restoring this stale plan verbatim.

## Risks / Open Questions

- Multi-process aggregation must stay consistent with startup locks and bundled runtime behavior.
- Operator-facing cleanup must avoid killing the primary healthy process by mistake.
