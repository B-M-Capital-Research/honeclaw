# Mobile Finance Calendar And Navigation Polish

- title: Mobile Finance Calendar And Navigation Polish
- status: archived
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `packages/app/src/components/finance-calendar-message.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-site.css`
- related_docs:
  - `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

## Goal

Correct the production iOS full-screen calendar regression where native page zoom and component zoom combine, hide the fixed controls, and leave the user trapped in an oversized calendar crop.

## Scope

- Remove native page-level pinch zoom from the calendar viewer and restore the chat-wide gesture guard.
- Replace binary 210vw tap zoom with explicit controlled fit/125/150/200 percent levels.
- Keep header, zoom controls, close, save, and share chrome fixed while only the image canvas pans.
- Preserve the loading, retry, download/share, mobile navigation, backend, and channel behavior already shipped.
- Commit, push, wait for Cloudflare Pages, and verify the new production chat chunk.

## Validation

- Add pure zoom-level regression coverage.
- Run `bun run typecheck:web`, `bun run test:web`, and `bun run build:web:public`.
- Verify fit and zoom DOM/CSS contracts at 390 x 844, then verify production asset switch and route health.

Completed with 207 passing frontend tests, successful typecheck/public build, 390 x 844 local and production overflow checks, and production asset verification for `index-D4wSdzNX.js` / `chat-ByxolQgf.js`.

## Documentation Sync

- Append the production follow-up to the existing same-day handoff, archive this plan again after deployment, and update the existing archive index entry.
- No repo-map, architecture, decision, or backend runbook update is needed.

## Risks / Open Questions

- Do not rely on native Safari page zoom inside a fixed application shell.
- Keep long-press saving available on the fitted image while preventing the viewer itself from scaling the browser viewport.
- An authenticated physical-iPhone save/share smoke remains useful, but the reported mixed browser/component zoom path is removed from production.
