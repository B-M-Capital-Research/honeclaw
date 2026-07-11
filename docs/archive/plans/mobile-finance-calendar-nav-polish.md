# Mobile Finance Calendar And Navigation Polish

- title: Mobile Finance Calendar And Navigation Polish
- status: archived
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/components/finance-calendar-message.tsx`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/lib/finance-calendar.ts`
  - `packages/app/src/lib/public-content.ts`
- related_docs:
  - `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

## Goal

Restore the verified mobile calendar/navigation fixes on top of the latest unrelated docs commit, publish them to main and Cloudflare Pages, and prove the production bundle is active.

## Scope

- Added a fixed-ratio calendar loading/error state, full-screen zoom, download, Web Share, and long-press fallback.
- Removed duplicate generic actions from calendar messages while preserving ordinary assistant images.
- Aligned the mobile header shell and notification/menu controls and refined the menu typography.
- Committed and pushed `main`, waited for the connected Pages deployment, and verified the production asset and 390px behavior.
- Kept backend APIs, persistence, Feishu, and other channels unchanged.

## Validation

- `bun run typecheck:web` passed.
- `bun run test:web` passed: 206 tests, 0 failures, 556 assertions.
- `bun run build:web:public` passed with only the existing large-chunk warning.
- Production switched to `assets/index-CAwvfWGR.js` and `assets/chat-CP613W7-.js`; the chat chunk contains the calendar loading/retry contracts.
- Production `/`, `/chat`, and `/roadmap` returned 200; `/api/public/auth/me` returned the expected JSON 401 for an unauthenticated request.
- Production 390 x 844 QA confirmed a 370 x 60 header, 42 x 42 menu geometry, no horizontal overflow, 54px menu rows, and the Avenir Next / PingFang display stack.

## Documentation Sync

- Added the completion handoff, archived this plan, updated `docs/archive/index.md`, and removed the task from the active index.
- No repo-map, architecture, decision, or backend runbook update was needed because boundaries and APIs are unchanged.

## Risks / Open Questions

- Mobile Safari download behavior varies by iOS version, so the production UI retains direct download, file-based Web Share, and long-press saving fallbacks.
- Authenticated production actor data was intentionally not mutated during automated QA; the reported user's next calendar generation is the final device-level smoke.
