# Windows 桌面端打包可用性

- title: Windows 桌面端打包可用性
- status: in_progress
- created_at: 2026-03-28
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/runbooks/desktop-dev-runtime.md`

## Goal

Make Windows desktop packaging verifiable in a real Windows environment after the sidecar preparation flow switched to a cross-platform script.

## Scope

- Cross-platform sidecar preparation is already in place.
- Remaining work is real packaging validation on a Windows environment with Rust and Bun available.

## Validation

- Pending real Windows packaging validation.

## Documentation Sync

- Keep `docs/current-plan.md` aligned with packaging readiness.
- If setup requirements change, update the matching runbook.

## Risks / Open Questions

- The current blocker is environment availability rather than code structure.
- Packaging validation may surface Windows-only path, bundling, or sidecar issues.
