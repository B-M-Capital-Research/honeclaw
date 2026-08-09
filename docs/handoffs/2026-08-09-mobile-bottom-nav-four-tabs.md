# Mobile Bottom Navigation Four Tabs Handoff — 2026-08-09

- title: Restore the visible mobile `我的` tab
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files: `packages/app/src/pages/public-agent-workspace.css`, `packages/app/src/pages/public-chat-style-contract.test.ts`
- related_docs: `docs/archive/plans/mobile-bottom-nav-four-tabs.md`, `docs/archive/index.md`
- related_prs: direct `main` implementation commit `959fca1600af5791118a17912cc944b6b9ca3464`; no PR, release, tag, or backend deployment

## Summary

The mobile workspace rendered four navigation buttons but retained a three-column grid. `我的` therefore wrapped into a second 54px row below the 56px fixed navigation container and was clipped outside the viewport. Commit `959fca1600af5791118a17912cc944b6b9ca3464` changes the grid to four columns and updates the regression contract; Cloudflare Pages is serving the corrected CSS.

## What Changed

- Changed `.agent-workspace-mobile-nav` from `repeat(3,1fr)` to `repeat(4,1fr)`.
- Updated the public chat visual contract to require the `推送` label and four CSS tracks alongside `Agent`, `洞察`, and `我的`.
- No route, authentication, API, backend binary, or production secret changed.

## Verification

- Before: production computed three columns around `124.66px`; `我的` occupied `top=844`, `bottom=898` while the navigation ended at `844`, proving it was invisible.
- After: production computed four equal `93.5px` columns; all four controls occupy `top=790`, `bottom=844` within the navigation and report visible.
- Clicking the unique `我的` bottom-navigation button reached `https://hone-claw.com/me`.
- Focused style contracts: 22 passed; full Web suite: 404 passed; public production build: passed; navigation responsiveness regression: passed.
- GitHub CI `31319035174`, Secret Scan, and Code Quality passed for the implementation commit.
- Cloudflare Pages serves `public-agent-workspace-BnsmYCiP.css`, whose deployed navigation rule contains `grid-template-columns:repeat(4,1fr)`.

## Risks / Follow-ups

- No backend restart was needed; production GCE remains on the previously accepted runtime revision.
- If the mobile product adds or removes tabs later, update the component, grid track count, labels, and this contract together.

## Next Entry Point

For any future mobile navigation change, start at `AgentWorkspaceMobileNav` in `packages/app/src/components/public-agent-workspace.tsx` and the matching media-query rule in `packages/app/src/pages/public-agent-workspace.css`, then repeat a real mobile bounding-rectangle check rather than relying on DOM presence alone.
