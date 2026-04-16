# Desktop / Runtime 启动锁收口

- title: Desktop / Runtime 启动锁收口
- status: archived
- created_at: 2026-03-29
- updated_at: 2026-04-16
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/runbooks/desktop-dev-runtime.md`

## Goal

Enforce one consistent startup lock model across the desktop main process, bundled backend, and channel listeners so conflicting runtimes are rejected coherently.

## Scope

- Startup locks have been added across the relevant processes.
- The remaining task had devolved into a generic validation placeholder with no current owner or concrete acceptance checklist.
- Archived on 2026-04-16; future work should reopen under a narrower verification or regression task.

## Validation

- Archived without new execution in this cleanup pass. Reopen when there is a specific lock conflict, recovery bug, or acceptance run to capture.

## Documentation Sync

- Archived from the active index on 2026-04-16.
- If lock behavior changes again, open a fresh focused plan and update the matching runbook.

## Risks / Open Questions

- Lock behavior interacts directly with the startup conflict UX strategy.
- A partially enforced lock model can make cleanup and takeover behavior harder to reason about.
