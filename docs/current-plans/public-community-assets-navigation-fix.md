# Public Community Assets And Navigation Fix

- title: Public Community Assets And Navigation Fix
- status: in_progress
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - crates/hone-core/src/cloud_runtime.rs
  - crates/hone-web-api/src/routes/public_community.rs
  - packages/app/src/components/public-nav.tsx
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/public-community.css
  - packages/app/src/pages/public-chat.css
- related_docs:
  - docs/archive/plans/public-community-readonly.md
  - docs/archive/plans/public-community-deployment-qa.md
  - docs/handoffs/2026-07-12-public-community-readonly.md

## Goal

Restore every legitimately accessible original community asset, replace thumbnail-quality previews with source-resolution media, and simplify the public desktop/mobile navigation while making Community a first-class tab.

## Scope

- Audit source-page, database, and R2 metadata for missing files and low-resolution images without bypassing source access controls.
- Backfill original files/images that the authorized browser session can retrieve through supported source flows.
- Update resource serving and the community viewer only where the current data/transport loses source quality.
- Add Community to the shared public navigation and reduce visual/action density across desktop and mobile.

## Validation

- Database/R2 asset counts, dimensions, byte sizes, hashes, and access-state verification.
- Rust tests/checks plus Web tests, typecheck, and public production build.
- Desktop and narrow mobile browser QA for navigation, timeline thumbnails, full-resolution viewer, protected resources, and community entry.
- Production API/asset/runtime smoke after deployment.

## Documentation Sync

- Update this plan, the existing community handoff, `docs/repo-map.md` if navigation/data flow changes, and `docs/archive/index.md` on completion.

## Risks

- Source-protected resources must remain unavailable unless the authenticated source UI/API legitimately exposes their bytes.
- Existing worktree changes belong to the user; stage only this task's scoped files.
- Original media may materially increase R2 bandwidth; preserve lazy thumbnail loading while using original bytes only in the full viewer when possible.
