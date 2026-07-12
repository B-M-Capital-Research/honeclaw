# Public Read-only Community

- title: Public Read-only Community
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - crates/hone-core/src/cloud_runtime.rs
  - crates/hone-web-api/src/routes/public_community.rs
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/archive/plans/public-community-readonly.md
  - docs/repo-map.md
- related_prs: []

## Summary

The public user experience now exposes the user-authorized community archive as `/community`. The same route is used by Web, the remote macOS user shell, and the iOS WebView shell; no local client copy of the archive exists.

## What Changed

- Cloud schema includes `community_spaces`, `community_contents`, ordered `community_content_resources`, and per-user `community_read_states`.
- `/api/public/community` is authenticated, paginated newest-first, and returns one complete post per row. `/seen` records the newest item read.
- `/api/public/community/resources/:resource_id` only streams R2 objects with `access_state=stored`; it refuses source-protected and metadata-only files.
- The chat composer has a community action beside the finance calendar and a server-backed red dot. The timeline always labels the author HONE 官方 and contains no social write/like/comment controls.
- Public resource responses expose neither the source author nor internal object-store URI. Inline preview is restricted to passive image formats and PDF, with `nosniff`, sandbox CSP, same-origin isolation, and a 128 MiB bound aligned with the explicit backfill ceiling; other stored MIME types download through an authenticated blob flow.
- Multi-image posts use a compact grid. The image viewer supports explicit zoom/fit, wheel, double-click, touch pinch, drag, Escape/focus handling, and mobile safe areas. Pagination failures stay inline instead of replacing the loaded feed.
- The chat unread state refreshes on a low-frequency timer and when the app regains focus; the mobile quick-action strip scrolls horizontally at narrow widths.

## Verification

- Cloud doctor with schema apply: PostgreSQL and R2 healthy; schema applied.
- Direct database verification: 616 unique content rows, 764 protected-file metadata rows, and 34 stored image resources.
- Rust tests: 218 passed, 2 credentialed tests ignored; workspace check and CI-safe regression suite pass.
- Web tests: 236 passed; TypeScript check and public production Vite build pass.
- Browser QA passed at desktop and `390x844` mobile viewports for timeline cards, 5-image grid, full-screen viewer, zoom state, protected-file cards, unread-dot accessibility, and chat-to-community navigation.
- The source runtime was rebuilt and upgraded from `0.13.0` to `0.14.1`. Cloud doctor reports PostgreSQL and R2 healthy; local, origin, and Worker anonymous community probes now return the expected JSON `401` rather than `404`.
- Cloudflare Pages published the production bundle from `main`: the root asset changed from `index-DnNGxqeh.js` to `index-D-q3AOum.js`, which references `public-community-BCjQZQha.js` and `public-community-_Vdl-l38.css`. Production `/community` renders the dedicated route and its login boundary.

## Risks / Follow-ups

- The source platform protected 764 file attachments. They are intentionally visible as metadata-only cards; preview becomes available only after a legitimate archival source writes their bytes to R2.
- Large-object streaming/Range support remains a future improvement. The authenticated proxy now accepts every object allowed by the 128 MiB community backfill default so the restored 31–41 MiB PDFs remain retrievable, but it still buffers each response in memory.

## Next Entry Point

`crates/hone-web-api/src/routes/public_community.rs`, `packages/app/src/pages/public-community.tsx`, `packages/app/src/pages/chat.tsx`, and `docs/archive/plans/public-community-deployment-qa.md`.

## 2026-07-12 Asset Fidelity And Navigation Follow-up

### Status

In progress while the resumable historical attachment capture finishes. Code, content reconciliation, original images, service restart, and frontend production deployment are complete.

### Reconciliation and data result

- A full authorized source-UI audit found 649 visible topics, 617 file-backed topics, 765 file items, and 53 source images. The first importer had 616 content rows, 764 file-resource rows, and 34 thumbnail image rows; it had omitted 32 non-file topics, 19 images, and one of two visually identical Starbucks PDF topics.
- `hone-cli cloud community-contents` now validates the entire contiguous source manifest and writes missing posts/resources in one PG transaction. The apply inserted exactly 33 content rows and 20 resource rows. The follow-up dry-run reports 617/617 existing file positions, 32 non-file topics already present by deterministic key, and `would_insert=0`.
- Seven source Q&A topics that the first body selector represented as empty were re-read from their visible question/answer containers before reconciliation, so the archive contains complete readable cards rather than blank placeholders.
- All 53 source images now use original captured bytes: 34 replacements plus 19 newly restored images. They were validated locally by magic, byte size, SHA-256, and dimensions; uploaded under immutable full-SHA keys; read back from R2; then atomically linked in PG. The screenshot reproduction `resource_id=7` changed from a 380×204 thumbnail to the 3142×1684 original.
- The screenshot PDF plus five additional early reproductions and the missing Starbucks duplicate were legitimately downloaded through the visible source UI, validated, uploaded, and promoted. Remaining source-protected resources are not bypassed and stay metadata-only.
- The first historical attachment batch promoted 193 resources: 186 immutable R2 uploads, 7 idempotent reuses, 186 PG updates, and no conflicts. A later 104-object batch correctly held PG at zero updates after one transient upload failure; its idempotent retry read back all 104 objects, atomically updated all 104 PG rows, and completed with no conflicts.
- Resource 295 is source-named `.xls`, but two independent visible-UI downloads produced the same 19,566-byte SHA and a valid OOXML package (`PK`, `[Content_Types].xml`, and `xl/`). The safety validator accepts only this narrow source `.xls` → verified XLSX alias; the Web client changes the downloaded filename to `.xlsx` while preserving source metadata.

### Runtime and interaction result

- Community API projection exposes only a short SHA-derived resource version. Resource responses reject stale versions, verify R2 bytes against the full PG SHA, emit strong ETags, cache correct versioned bytes as private immutable, and force unversioned legacy URLs to revalidate.
- Desktop public navigation now keeps Home / Community / Blog / Chat visible and moves Roadmap, account, social/contact, and language actions into More. Mobile uses a safe-area-aware Home / Chat / Community / Me tab bar; the hamburger contains only secondary destinations.
- Authenticated desktop chat uses one navigation surface and has Community in the sidebar. Mobile composer/content padding clears the new tab bar. Community unread state continues to drive the chat quick action, sidebar, and the shared navigation dot where the state is available.
- Backend source runtime was rebuilt and restarted on the current `0.14.1` codebase; no release tag was created. Commit `879e9722` was pushed to `main`, and Cloudflare Pages published `assets/index-B7KkT0YR.js`.

### Verification

- Data: complete-source reconciliation dry-run (`would_insert=0`); 19-image and prior 34-image manifests re-read from R2 with no conflicts; immutable-object/PG updates are idempotent.
- Rust: 118 `hone-core` tests; 106 `hone-web-api` tests passed with two live-credential tests ignored; 66 `hone-cli` tests; package checks and changed-file formatting passed.
- Web: 241 tests, TypeScript check, and public production build passed.
- Regression: all 11 scripts under `tests/regression/ci/` passed.
- Browser: desktop and 390×844 mobile navigation/menu/tab QA passed locally; production bundle and four mobile tabs were verified after publish; no horizontal overflow was present. Anonymous local/origin/Worker auth and community probes return JSON `401`.

### Rollback and remaining risk

- `raw_metadata.community_asset_backfill` retains every promoted resource's previous SHA, size, OSS URI, and access state; old immutable objects remain available. Revert PG fields from this audit record or a PG snapshot before deleting any object.
- The current resource proxy still buffers objects, with a 128 MiB ceiling aligned to the backfill command. Large-object Range/streaming and separate derived timeline thumbnails remain follow-ups.
- Historical attachment capture is resumable and partitions by source library index. A row is final only as downloaded, visibly App-only protected, or unresolved after source-UI retry; signed source URLs are never logged or persisted.
