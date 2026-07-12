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
- Public resource responses expose neither the source author nor internal object-store URI. Inline preview is restricted to passive image formats and PDF, with `nosniff`, sandbox CSP, same-origin isolation, and a 25 MiB bound; other stored MIME types download through an authenticated blob flow.
- Multi-image posts use a compact grid. The image viewer supports explicit zoom/fit, wheel, double-click, touch pinch, drag, Escape/focus handling, and mobile safe areas. Pagination failures stay inline instead of replacing the loaded feed.
- The chat unread state refreshes on a low-frequency timer and when the app regains focus; the mobile quick-action strip scrolls horizontally at narrow widths.

## Verification

- Cloud doctor with schema apply: PostgreSQL and R2 healthy; schema applied.
- Direct database verification: 616 unique content rows, 764 protected-file metadata rows, and 34 stored image resources.
- Rust tests: 218 passed, 2 credentialed tests ignored; workspace check and CI-safe regression suite pass.
- Web tests: 236 passed; TypeScript check and public production Vite build pass.
- Browser QA passed at desktop and `390x844` mobile viewports for timeline cards, 5-image grid, full-screen viewer, zoom state, protected-file cards, unread-dot accessibility, and chat-to-community navigation.
- The source runtime was rebuilt and upgraded from `0.13.0` to `0.14.1`. Cloud doctor reports PostgreSQL and R2 healthy; local, origin, and Worker anonymous community probes now return the expected JSON `401` rather than `404`.

## Risks / Follow-ups

- The source platform protected 764 file attachments. They are intentionally visible as metadata-only cards; preview becomes available only after a legitimate archival source writes their bytes to R2.
- The production Web bundle is published by Cloudflare Pages from `main`; the deployment/asset smoke is tracked in `docs/current-plans/public-community-deployment-qa.md` until the production-branch push is visible.

## Next Entry Point

`crates/hone-web-api/src/routes/public_community.rs`, `packages/app/src/pages/public-community.tsx`, and `packages/app/src/pages/chat.tsx`.
