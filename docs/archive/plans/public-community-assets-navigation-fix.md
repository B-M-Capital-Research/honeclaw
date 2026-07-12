# Public Community Assets And Navigation Fix

- title: Public Community Assets And Navigation Fix
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-13
- owner: Codex
- related_files:
  - bins/hone-cli/src/cloud.rs
  - crates/hone-core/src/cloud_runtime.rs
  - crates/hone-web-api/src/routes/public_community.rs
  - packages/app/src/components/public-nav.tsx
  - packages/app/src/lib/api.ts
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/public-community.css
  - packages/app/src/pages/public-chat.css
- related_docs:
  - docs/runbooks/backend-deployment.md
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
- Completed the authorized historical file library capture. The final archive contains 651 stored originals, 113 source-confirmed protected rows, and one unresolved metadata row (`resource_id=834`) across all 765 file resources. The stored originals total 2,614,811,800 bytes: 642 PDF, 7 DOCX, 1 PPTX, and 1 XLSX.
- Promoted the 651 originals through ten idempotent logical batches. Every local file passed magic, byte-size, and SHA-256 verification; all nine OOXML containers passed ZIP and internal-structure checks; every apply read the immutable object back before its PostgreSQL update and completed with zero final conflicts. The second batch's first attempt safely held PG at zero updates after one transient upload failure, then succeeded 104/104 on retry.
- Resolved the duplicate Starbucks source positions without aliasing bytes onto the ambiguous protected row: resource 951 remains the old protected shadow, while source position 571 is resource 1016 and is stored. One source `.xls` was independently downloaded twice with identical bytes and verified as a valid OOXML workbook; the backfill permits only this narrow `.xls` metadata → XLSX magic alias, and the client corrects its download name to `.xlsx`.
- Added content-hash resource versions, strong ETags, R2 SHA verification, and immutable/private cache behavior so a previously cached thumbnail cannot survive a source-quality replacement. The authenticated proxy ceiling now matches the 128 MiB backfill ceiling, covering the restored 31–41 MiB PDFs.
- Shipped the compact desktop navigation and mobile Home / Chat / Community / Me tab bar, including Community unread-dot projection in chat and safe-area spacing.
- Rebuilt and restarted the backend origin, pushed commits `879e9722`, `af3cb605`, and `7ab36682` to `main`, and observed Cloudflare Pages publish `assets/index-BB8Wrwbl.js`. Local/origin/Worker anonymous auth and community probes return JSON `401` as expected.
- Validation passed: 242 Web tests, TypeScript check, public production build, 118 core tests, 106 Web API tests with two credentialed tests ignored, 67 CLI tests, all 11 CI-safe regression scripts, complete 651-file data audit, and desktop/mobile browser QA with no horizontal overflow.
