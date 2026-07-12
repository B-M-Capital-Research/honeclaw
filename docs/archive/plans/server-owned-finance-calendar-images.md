# Server-owned Finance Calendar Images

- title: Server-owned Finance Calendar Images
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
  - `crates/hone-web-api/src/routes/history.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `crates/hone-web-api/src/types.rs`
  - `packages/app/src/components/finance-calendar-message.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/lib/messages.ts`
  - `packages/app/src/lib/public-chat.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/decisions.md#d-2026-07-12-02-persist-finance-calendar-variants-and-select-them-server-side`
  - `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- verification:
  - `cargo test -p hone-web-api --lib`: 100 passed, 2 credentialed tests ignored
  - `bun run test:web`: 216 passed
  - `bun run typecheck:web`: passed
  - `bun run build:web:public`: passed
  - production frontend, origin, and public Worker health checks after deployment
- risks:
  - Legacy messages with only one image marker fall back to that desktop image; they are not regenerated in the client.
  - Device selection is intentionally User-Agent based, so unusual clients without a mobile marker receive the desktop image.

## Goal

Persist desktop and mobile finance-calendar image variants as backend-owned session metadata, select one variant server-side per public request, and make the user client render one stable image URL without fetching calendar data or regenerating a PNG while restoring history.

## Completed Scope

- [x] Persist a required desktop/mobile/month calendar image contract in session metadata.
- [x] Project a single device-selected calendar image through public bootstrap/history, with legacy marker compatibility.
- [x] Remove frontend history lazy rendering, blob replacement, and source upgrade logic.
- [x] Add private immutable caching to authenticated calendar image responses.
- [x] Add regression tests and run full frontend/Web API verification.
- [x] Update decision, repository map, handoff, archived plan, and archive index.

## Documentation Sync

`docs/decisions.md`, `docs/repo-map.md`, the existing finance-calendar handoff, and this archive index were updated. No new runbook is required because deployment mechanics are unchanged.

## Risks / Open Questions

No open implementation item remains. Calendar creation still renders the two artifacts once in the browser before upload; history display and device selection no longer execute that rendering path.
