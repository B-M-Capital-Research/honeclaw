# Desktop 启动锁冲突体验优化方案

- title: Desktop 启动锁冲突体验优化方案
- status: in_progress
- created_at: 2026-03-29
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/current-plans/desktop-runtime-startup-locks.md`

## Goal

Turn startup lock conflicts from a hard-to-explain failure into an operator-friendly experience with takeover, recovery, or clear degraded behavior.

## Scope

- This task is intentionally strategy-first and may land before code changes.
- The target outcome is a clear UX plan for automatic takeover, layered recovery, and explainable degradation.

## Validation

- Pending. The next update should capture acceptance criteria for the user-visible startup experience.

## Documentation Sync

- Keep `docs/current-plan.md` aligned.
- If the strategy becomes a long-lived product rule, update `docs/decisions.md` or a dedicated ADR.

## Risks / Open Questions

- UX guidance must stay consistent with the actual lock enforcement model.
- A purely conceptual plan without clear acceptance criteria will be hard for the next agent to implement.
