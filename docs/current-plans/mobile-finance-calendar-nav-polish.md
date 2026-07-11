# Mobile Finance Calendar And Navigation Polish

- title: Mobile Finance Calendar And Navigation Polish
- status: in_progress
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

- Add a fixed-ratio calendar loading/error state, full-screen zoom, download, Web Share, and long-press fallback.
- Remove duplicate generic actions from calendar messages while preserving ordinary assistant images.
- Align the mobile header shell and notification/menu controls and refine the menu typography.
- Commit and push `main`, wait for the connected Pages deployment, and verify production asset/version and 390px behavior.
- Keep backend APIs, persistence, Feishu, and other channels unchanged.

## Validation

- `bun run typecheck:web`, `bun run test:web`, and `bun run build:web:public`.
- 390 x 844 local QA for header/menu geometry and production QA after Pages switches.
- Verify production HTML references the new public bundle and `/`, `/chat`, `/roadmap`, and public auth API health.

## Documentation Sync

- Add a deployment-complete handoff, archive this plan, update `docs/archive/index.md`, and remove the active entry after production verification.
- No repo-map, architecture, or decision update is expected because boundaries and APIs do not change.

## Risks / Open Questions

- Cloudflare Pages deployment timing is external; do not claim completion until the production asset changes.
- iOS Safari download support varies, so retain direct download, file-based Web Share, and long-press saving fallbacks.
