# Runbook: Task Delivery

Last updated: 2026-03-18

## When to Use

- Any development, refactoring, bug-fix, or documentation task of medium-or-greater complexity, or any task that needs cross-turn / parallel tracking

## Standard Flow

1. Read `AGENTS.md`, `docs/repo-map.md`, and `docs/invariants.md` first
2. Form a todo list in the current session first, including at least the goal, the affected files, the verification steps, and the document-sync steps
3. Decide whether the task meets the dynamic-plan entry criteria; only if it does, open and update `docs/current-plan.md`, then create or reuse a matching `docs/current-plans/*.md`
4. Read the relevant entrypoints, implementation, and tests before making changes
5. Run verification that matches the change set
6. If dynamic-plan docs were used in this round, update both the `docs/current-plan.md` index and the matching task plan file before finishing
7. If the task changes long-lived behavior, update `docs/decisions.md` and add an ADR when needed
8. If the task needs handoff, pause-and-resume support, or is medium-or-greater with follow-up risk, update or add a handoff; prefer reusing the original file for the same topic

## Minimum Delivery Requirements

- If code changed, state at least the verification method or why it was not verified
- If behavior changed, the matching document must be updated
- If no code changed, explain why the change only affected docs or workflow
- Before closing the task, confirm that the todo's "verification" and "document sync" items are complete
- When dynamic-plan docs are used, confirm that the index-page and single-task-plan links are correct
- Small one-off execution tasks do not have to be written to `docs/current-plan.md` or `docs/current-plans/*.md`, but the delivery note still needs to mention verification and the document decision
- Handoffs should not be created in bulk for low-value tasks; reuse when possible and skip when unnecessary
