# Public Mobile Overlay & Calendar Hotfix

- title: Public Mobile Overlay & Calendar Hotfix
- status: archived
- created_at: 2026-07-10
- updated_at: 2026-07-10
- owner: Codex
- related_files:
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/lib/public-chat.ts`
  - `packages/app/src/lib/public-content.ts`
  - `packages/app/src/components/finance-calendar-card.tsx`
  - `packages/app/e2e/public-mobile-overlays.spec.ts`
- related_docs:
  - `docs/decisions.md`
  - `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`

## Goal

Make authenticated public chat usable at iPhone widths by fixing push overlay stacking and density, clearing the unread indicator when the inbox is opened, separating the dot from the bell glyph, and providing a real zoomable finance-calendar preview.

## Result

- Push center and detail overlays now sit above the fixed public nav, use explicit mobile viewport edges, and render denser list/detail cards.
- Opening the push center acknowledges only the latest push known at open time, immediately clears the dot, and preserves later arrivals as unread.
- The unread dot sits outside the bell glyph and has a separate sidebar position.
- The calendar modal is compact on mobile; its preview opens a full-screen, scrollable viewer with fit and zoom controls.
- The calendar viewer renders through a document-body Portal so the fixed public nav cannot intercept its controls.

## Verification

- `bun run typecheck:web`
- `bun run test:web`
- `bun run build:web:public`
- `cd packages/app && ./node_modules/.bin/playwright test e2e/public-mobile-overlays.spec.ts --project=public --reporter=line` (1 passed)
- Isolated PostgreSQL browser fixture was removed; affected production actor remained at 79 total / 79 unread pushes after QA.

## Documentation Sync

- Updated decision `D-2026-07-10-01`, the scheduled-push handoff, and archive index.
- No repo-map update: module boundaries, API routes, and storage authority are unchanged.

## Risks / Follow-ups

- Mobile E2E uses deterministic API mocks to keep it account-independent; production smoke still verifies deployed assets and public API health.
- Opening the inbox is intentionally the read acknowledgement gesture; individual items do not display read state.
