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

## Documentation Sync

- Append the completed follow-up to the existing same-day handoff, move this plan to `docs/archive/plans/`, update `docs/archive/index.md`, and remove the active index entry after production verification.
- No architecture decision or repo-map change is required because the existing public finance-calendar route and browser-rendered image flow remain authoritative.

## Risks / Open Questions

- Safari must not receive native page pinch zoom inside the fixed application shell; pinch is interpreted at the image canvas level instead.
- Existing messages contain only the desktop path and must remain readable.
- Upload validation must be applied independently to both image paths before persistence.
