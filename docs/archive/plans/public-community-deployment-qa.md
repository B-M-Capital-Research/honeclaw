# Public Community Deployment And QA

- title: Public Community Deployment And QA
- status: archived
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
  - docs/handoffs/2026-07-12-public-community-readonly.md
  - docs/runbooks/backend-deployment.md

## Goal

Review the public community experience on desktop and mobile, fix production-readiness issues, deploy the current user service, run production smoke checks, and commit/push the complete scoped change set.

## Completed Scope

- Upgraded the source runtime from `0.13.0` to `0.14.1` using the existing detached `hone-runtime` supervisor path.
- Hardened the resource proxy against active same-origin content and internal metadata disclosure, and bounded object reads to 25 MiB.
- Added multi-image grids, controlled zoom/pan, accessible modal focus handling, authenticated downloads, inline pagination recovery, mobile quick-action scrolling, and periodic/focus unread refresh.
- Pushed the production branch and verified Cloudflare Pages switched from `index-DnNGxqeh.js` to `index-D-q3AOum.js`, which references the new community JS/CSS chunks.

## Verification

- `cargo test -p hone-core -p hone-web-api`: 218 passed, 2 credentialed tests ignored.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed.
- `bun run test:web`: 236 passed; Web typecheck and public production build passed.
- `bash tests/regression/run_ci.sh`: passed.
- Cloud doctor: PostgreSQL and R2 healthy, zero local durable dependencies.
- Browser QA: desktop and `390x844` mobile timeline, multi-image grid, viewer zoom/close, protected resource state, unread dot, narrow quick-action strip, and chat-to-community navigation passed.
- Production smoke: `/community` returns the new SPA; local, origin, and Worker anonymous community probes return JSON `401` rather than `404`.

## Documentation Sync

- Updated `docs/repo-map.md`, the community handoff, current-plan index, and archive index.

## Risks

- The source platform protects 764 file attachments. They intentionally remain metadata-only; only 34 already archived JPEG images are currently previewable.
- Large-object streaming/Range support is not needed for the current 1.9 MiB image set; the current proxy fails closed above 25 MiB. Add streaming before enabling large PDF archives.
