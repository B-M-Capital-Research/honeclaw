# Public Web Visual Architecture Refactor

- title: Public Web Visual Architecture Refactor
- status: archived
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

Completed in `5b7b1d67`. The chat page stylesheet moved from a 2,069-line JSX string into `public-chat.css`; foundation, shared public polish, and calendar composition now have independent ownership. The dense 15-event fixture produced a 1500 x 2668 PNG from the exact 750 x 1334 card at deterministic 2x scale, then rendered at 390 x 693.67 with no horizontal overflow, clipped labels, or category-background mismatch. All 209 frontend tests, typecheck, and the public build passed.

Production switched to `index-DbfrdfV3.js`, `chat-CJ_LPzbz.js`, `chat-u_ejMPXz.css`, and `public-nav-x7Fh5Iof.css`. The production chunk contains both v3 migration contracts and the signal-calendar marker. `/`, `/chat`, `/roadmap`, and `/me` return 200; auth returns the expected 401 JSON. A 390 x 844 browser check loaded both CSS layers, reported no console errors or horizontal overflow, and measured the nav at 370 x 60 with 10px side insets. Runtime PID `9767`, local backends, Feishu, Discord, and the console process are healthy.

iOS follow-up: a real Safari-generated v3 artifact still clips the lower halves of Chinese agenda titles and the top signal line even though desktop Chromium and DOM geometry pass. The mobile artifact must stop using html2canvas entirely. New sends and lazy legacy upgrades will share one Canvas 2D renderer and use a `mobile-v4` marker so existing v3 artifacts are replaced in view.

The v4 implementation now paints the complete 1500 x 2668 PNG through Canvas 2D at a 750 x 1334 logical coordinate system. New sends and in-view upgrades both call `renderFinanceCalendarMobilePng`; only the desktop card still uses html2canvas. A dense 13-event fixture rendered at 390 x 693.67 with all six agenda titles, including the long FOMC row, fully visible and no horizontal overflow. The shared agenda and canvas wrapping contracts have regression coverage; all frontend tests, typecheck, and the public build pass.

Shipped in `a3e0dbaa`. All 211 frontend tests, typecheck, and the public build passed. Production switched to `index-C6T9yKIo.js` / `chat-VvOemH_a.js`; the chunk contains two v4 migration contracts, 24 direct `fillText` calls, and the Canvas error guard. Core routes return 200, auth returns the expected 401 JSON, the 390 x 844 page has no overflow or browser errors, and runtime PID `9767`, both local backends, Feishu, Discord, and the console process remain healthy.

## Documentation Sync

- Update `docs/repo-map.md` for the new public visual ownership boundaries.
- Record the CSS/rendering ownership decision in `docs/decisions.md`.
- Append production evidence to the existing same-day calendar handoff.
- Archive this plan and update `docs/archive/index.md` after production verification.

## Risks / Open Questions

- Existing v2 calendar images must be rebuilt lazily rather than mutating conversation history.
- html2canvas must not depend on fractional scaling or unsupported variable font weights.
- Public-site pages and the authenticated chat share brand primitives but must retain independent layout behavior.
