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
- Original media may materially increase R2 bandwidth; keep browser lazy loading now and evaluate derived timeline thumbnails separately without degrading the full viewer.

## Progress

- Reconciled the complete visible source timeline: 649 topics and 617 contiguous file positions. The existing archive matched 616 positions; one missing duplicate-name file post plus 32 non-file/image/Q&A posts were inserted in one transaction. The post-reconcile dry-run reports no missing content.
- Replaced all 34 archived source thumbnails and added the 19 omitted source images with verified original-resolution bytes and immutable full-SHA R2 keys. All 53 images pass local magic/size/SHA validation and R2 read-back verification.
- Backfilled the two screenshot reproductions (`英伟达路演-译文.pdf` and the original high-resolution `image-5`) plus the independently downloadable Starbucks duplicate. The remaining historical file library is being processed in four resumable source-UI partitions; source-protected rows remain metadata-only.
- Added content-hash resource versions, strong ETags, R2 SHA verification, and immutable/private cache behavior so a previously cached thumbnail cannot survive a source-quality replacement. The authenticated proxy ceiling now matches the 128 MiB backfill ceiling, covering the restored 31–41 MiB PDFs.
- Shipped the compact desktop navigation and mobile Home / Chat / Community / Me tab bar, including Community unread-dot projection in chat and safe-area spacing.
- Rebuilt and restarted the backend origin, pushed commit `879e9722` to `main`, and observed Cloudflare Pages publish `assets/index-B7KkT0YR.js`. Local/origin/Worker anonymous auth and community probes return JSON `401` as expected.
- Validation passed: 241 Web tests, TypeScript check, public production build, 118 core tests, 106 Web API tests with two credentialed tests ignored, 66 CLI tests, all 11 CI-safe regression scripts, and desktop/mobile browser QA with no horizontal overflow.
