# Skill Runtime 对齐 Claude Code

- title: Skill Runtime 对齐 Claude Code
- status: in_progress
- created_at: 2026-03-31
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/decisions.md`

## Goal

Bring the skill runtime in line with the Claude Code interaction model without regressing the existing Hone agent flow.

## Scope

- Listing disclosure is already aligned.
- Full prompt injection on invoke is already aligned.
- Slash/direct invoke and session resume are already aligned.
- Remaining gaps: real hook execution, turn-scope tool enforcement, watcher hot reload.

## Validation

- Pending. The next active implementation update should record concrete verification commands and outcomes here.

## Documentation Sync

- Keep `docs/current-plan.md` and this file aligned.
- If the runtime contract changes, update `docs/decisions.md` or `docs/adr/*.md`.

## Risks / Open Questions

- Hook execution and tool enforcement can change behavior across multiple channels.
- Remaining work appears to depend on runner / infra changes rather than prompt-only edits.
