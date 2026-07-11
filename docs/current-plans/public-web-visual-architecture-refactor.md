# Public Web Visual Architecture Refactor

- title: Public Web Visual Architecture Refactor
- status: in_progress
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-chat.css`
  - `packages/app/src/pages/public-foundation.css`
  - `packages/app/src/components/finance-calendar-mobile-card.tsx`
  - `packages/app/src/components/finance-calendar-mobile-card.css`
  - `packages/app/src/lib/finance-calendar.ts`
- related_docs:
  - `docs/current-plan.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

## Goal

Replace the public user's accumulated page-local overrides with a coherent HONE visual foundation, and rebuild the mobile finance calendar so typography, labels, grid geometry, messages, navigation, and dialogs remain precise across iPhone and desktop rendering.

## Scope

- Extract the chat page's embedded stylesheet into an owned CSS module.
- Introduce public-surface design tokens for type, color, spacing, radius, elevation, and controls.
- Normalize the user chat shell, navigation, message cards, composer, and modal geometry.
- Rebuild the portrait calendar with explicit line boxes, standard font weights, deterministic integer-scale capture, and a v3 migration marker.
- Keep API, persistence, desktop calendar, Feishu, and other channels unchanged.

## Validation

- Add regression coverage for the v3 image migration contract.
- Run frontend tests, typecheck, and public production build.
- Render dense and sparse calendar fixtures and inspect at source size plus 390 x 844 mobile size.
- Verify production assets, core routes, mobile overflow, and runtime health.

## Documentation Sync

- Update `docs/repo-map.md` for the new public visual ownership boundaries.
- Record the CSS/rendering ownership decision in `docs/decisions.md`.
- Append production evidence to the existing same-day calendar handoff.
- Archive this plan and update `docs/archive/index.md` after production verification.

## Risks / Open Questions

- Existing v2 calendar images must be rebuilt lazily rather than mutating conversation history.
- html2canvas must not depend on fractional scaling or unsupported variable font weights.
- Public-site pages and the authenticated chat share brand primitives but must retain independent layout behavior.
