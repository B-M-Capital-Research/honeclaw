# Desktop 启动锁冲突体验优化方案

- title: Desktop 启动锁冲突体验优化方案
- status: archived
- created_at: 2026-03-29
- updated_at: 2026-04-16
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/current-plans/desktop-runtime-startup-locks.md`

## Goal

Turn startup lock conflicts from a hard-to-explain failure into an operator-friendly experience with takeover, recovery, or clear degraded behavior.

## Scope

- This task was intentionally strategy-first and may land before code changes.
- The target outcome was a clear UX plan for automatic takeover, layered recovery, and explainable degradation.
- Archived on 2026-04-16 because it remained conceptual and had not been tightened into concrete acceptance criteria.

## Validation

- Archived without additional execution. Reopen only when a real product/UX decision round starts and acceptance criteria can be written explicitly.

## Documentation Sync

- Archived from the active index on 2026-04-16.
- If the strategy becomes active again, reopen as a concrete proposal or ADR-oriented task.

## Risks / Open Questions

- UX guidance must stay consistent with the actual lock enforcement model.
- A purely conceptual plan without clear acceptance criteria will be hard for the next agent to implement.
