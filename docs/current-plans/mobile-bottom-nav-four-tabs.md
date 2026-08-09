# Mobile Bottom Navigation Four Tabs

- title: Restore all four mobile bottom-navigation tabs
- status: in_progress
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-agent-workspace.css`
  - `packages/app/src/pages/public-chat-style-contract.test.ts`
- related_docs:
  - `docs/current-plan.md`

## Goal

Restore the visible `Agent / 推送 / 洞察 / 我的` four-tab mobile navigation on every authenticated public workspace page.

## Scope

- Correct the mobile grid from the stale three-column layout to four columns.
- Add a regression contract that keeps four rendered buttons and four CSS tracks in sync.
- Verify the public build and a real mobile viewport, then publish the frontend fix.

## Validation

- Run the focused public chat/workspace style contract tests.
- Run the full Web unit suite and public production build.
- Verify all four production tab rectangles are inside the fixed navigation bounds at a mobile viewport.

## Documentation Sync

- Keep this plan and `docs/current-plan.md` current during the fix.
- On completion, archive this plan and add a concise production handoff/archive entry; no architecture decision is required because this only repairs an existing UI contract.

## Risks / Open Questions

- The fourth button already exists in the DOM; the regression must check layout columns, not only labels, so the same invisible-overflow failure cannot recur.
- Cloudflare Pages deployment must be verified by the deployed asset hash and bounding rectangles, not by HTTP `200` alone.
