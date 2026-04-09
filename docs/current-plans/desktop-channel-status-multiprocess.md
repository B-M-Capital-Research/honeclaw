# Desktop 渠道监听状态与多进程 PID 对齐

- title: Desktop 渠道监听状态与多进程 PID 对齐
- status: in_progress
- created_at: 2026-03-28
- updated_at: 2026-04-09
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

## Validation

- Pending. The next active update should capture end-to-end verification across backend status, desktop UI, and process cleanup behavior.

## Documentation Sync

- Keep `docs/current-plan.md` aligned.
- Update the relevant runbook if operational cleanup steps change.

## Risks / Open Questions

- Multi-process aggregation must stay consistent with startup locks and bundled runtime behavior.
- Operator-facing cleanup must avoid killing the primary healthy process by mistake.
