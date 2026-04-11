# Desktop / Runtime 启动锁收口

- title: Desktop / Runtime 启动锁收口
- status: in_progress
- created_at: 2026-03-29
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/runbooks/desktop-dev-runtime.md`

## Goal

Enforce one consistent startup lock model across the desktop main process, bundled backend, and channel listeners so conflicting runtimes are rejected coherently.

## Scope

- Startup locks have been added across the relevant processes.
- The remaining task is to finish validating and polishing the whole-lock behavior as one system.

## Validation

- Pending. Record lock conflict, recovery, and rejection-path verification here.

## Documentation Sync

- Keep `docs/current-plan.md` aligned.
- If operator behavior changes, update the matching runbook.

## Risks / Open Questions

- Lock behavior interacts directly with the startup conflict UX strategy.
- A partially enforced lock model can make cleanup and takeover behavior harder to reason about.
