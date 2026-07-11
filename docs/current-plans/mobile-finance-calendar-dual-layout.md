# Mobile Finance Calendar Dual Layout And Gestures

- title: Mobile Finance Calendar Dual Layout And Gestures
- status: in_progress
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `packages/app/src/components/finance-calendar-card.tsx`
  - `packages/app/src/components/finance-calendar-message.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/lib/api.ts`
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
- related_docs:
  - `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

## Goal

Make generated finance calendars genuinely usable on iPhone: preserve the existing desktop image, add a dedicated portrait mobile image, enable bounded two-finger canvas zoom and one-finger panning without scaling the application shell, and restore the native image long-press menu.

Production follow-up: replace layout-driven width/scroll zoom with GPU transform gestures, remove clipped/ellipsized agenda text, and lazily rebuild a portrait artifact for legacy messages that only contain the desktop image.

Design follow-up: replace the compressed desktop-dashboard aesthetic with an editorial mobile market brief, using a stronger HONE cover, intentional month scan, category-aware key-date timeline, complete text, and balanced use of the full portrait canvas.

## Scope

- Render and upload desktop and mobile calendar PNG files from one payload.
- Persist both validated upload paths in one backward-compatible assistant message.
- Select the portrait image on mobile and the existing image on desktop while old one-image messages continue to work.
- Implement custom bounded pinch zoom inside the fixed viewer, retain scroll panning, and allow iOS long-press on the actual image.
- Deploy the public client and web API changes, then verify production assets and routes.

## Validation

- Add frontend helper/API coverage and Rust route/message contract coverage.
- Run focused Rust tests, `bun run typecheck:web`, `bun run test:web`, and `bun run build:web:public`.
- Perform 390 x 844 browser QA and production route/asset checks.

Completed with 207 passing frontend tests, 7 focused web API tests, successful typecheck/public build, a rendered 390 x 844 portrait-card review, and production verification of `index-BcPNNntX.js` / `chat-DFgZWxOf.js`. The restarted runtime exposes the new backend contract and both public/origin health probes return the expected unauthenticated 401 JSON.

The screenshot-driven follow-up shipped in `a4af378d`: 209 frontend tests, typecheck, and the public build passed. Real-component QA proved exact 342 x 610 contain sizing inside a 390 x 844 viewer, stable 100/125/150/200/250/300 percent transform-only zoom, full agenda text, and automatic conversion of a synthetic 1080 x 1350 legacy image into a 0.562-ratio mobile blob. Production switched to `index-C-0scCea.js` / `chat-qOMG9Bni.js`; core routes, auth proxy, runtime, tunnel, and UI sessions are healthy.

## Documentation Sync

- Append the completed follow-up to the existing same-day handoff, move this plan to `docs/archive/plans/`, update `docs/archive/index.md`, and remove the active index entry after production verification.
- No architecture decision or repo-map change is required because the existing public finance-calendar route and browser-rendered image flow remain authoritative.

## Risks / Open Questions

- Safari must not receive native page pinch zoom inside the fixed application shell; pinch is interpreted at the image canvas level instead.
- Existing messages contain only the desktop path and must remain readable.
- Upload validation must be applied independently to both image paths before persistence.
- Native long-press behavior still depends on iOS Safari itself; the application no longer suppresses touch callout or single-touch image handling.
- Legacy mobile reconstruction uses the current actor/month calendar payload, so it intentionally upgrades readability rather than preserving a pixel-identical historical desktop artifact.
