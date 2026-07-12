# Public Read-only Community

- title: Public Read-only Community
- status: archived
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - crates/hone-core/src/cloud_runtime.rs
  - crates/hone-web-api/src/routes/public_community.rs
  - crates/hone-web-api/src/routes/public.rs
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/chat.tsx
  - apps/hone-ios/HONE/NavigationPolicy.swift
- related_docs:
  - docs/repo-map.md
  - docs/handoffs/2026-07-12-public-community-readonly.md

## Goal

Expose the user-authorized archived community as a production-quality, read-only HONE Official timeline on Web/desktop and the iOS user client.

## Scope

- Added idempotent Cloud Postgres community/archive/read-state schema.
- Added authenticated cursor pagination, unread acknowledgement, and object-store preview APIs.
- Added `/community`, media lightbox/file-preview UI, and a chat composer entry beside the finance calendar with a server-backed unread dot.
- Kept public macOS and iOS shells on the same first-party production Web route.

## Validation

- `cargo run -p hone-cli -- cloud doctor --ensure-schema --json`: Cloud PG/R2 healthy and schema applied; archive has 616 unique content rows.
- `cargo test -p hone-core -p hone-web-api`: 216 passed, 2 credentialed tests ignored.
- `PATH=/Users/fengming2/.bun/bin:$PATH bun run test:web`: 231 passed.
- `tsc -p packages/app/tsconfig.json` and a public Vite production build passed.

## Documentation Sync

- Updated `docs/repo-map.md`; created a handoff and archive index entry.

## Risks / Open Questions

- Files explicitly marked source-protected were never downloaded. The UI shows their metadata but enables preview only for bytes that are already in R2.
- This change applies the database migration but does not publish a new web bundle or restart an external production service.
