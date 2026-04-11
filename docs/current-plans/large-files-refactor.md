# 大文件物理拆分重构

- title: 大文件物理拆分重构
- status: in_progress
- created_at: 2026-03-22
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/repo-map.md`

## Goal

Reduce change risk and editing friction by physically splitting oversized files along stable module boundaries.

## Scope

- The task is active because it changes structure across multiple modules.
- The expected output is smaller files with clearer boundaries and no behavior regressions.

## Validation

- Pending. Each split should record compile/test coverage for the affected modules here.

## Documentation Sync

- Update `docs/repo-map.md` if module boundaries change materially.
- Keep `docs/current-plan.md` aligned with the active scope.

## Risks / Open Questions

- Physical splits can create hidden import or ownership regressions if boundaries are guessed rather than derived.
- This work should prefer incremental proofs over one-shot file explosions.
