# Mobile Bottom Navigation Four Tabs

- title: Restore all four mobile bottom-navigation tabs
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-agent-workspace.css`
  - `packages/app/src/pages/public-chat-style-contract.test.ts`
- related_docs:
  - `docs/handoffs/2026-08-09-mobile-bottom-nav-four-tabs.md`
  - `docs/archive/index.md`

## Goal

Restore the visible `Agent / 推送 / 洞察 / 我的` four-tab mobile navigation on every authenticated public workspace page.

## Scope

- Correct the mobile grid from the stale three-column layout to four columns.
- Add a regression contract that keeps four rendered buttons and four CSS tracks in sync.
- Verify the public build and a real mobile viewport, then publish the frontend fix.

## Validation

- Focused public chat/workspace style contracts passed: 22/22.
- Full Web unit suite passed: 404/404; the public production build completed successfully.
- Navigation responsiveness regression passed, including its Rust and Web subsets.
- GitHub CI `31319035174`, Secret Scan, and Code Quality passed for implementation commit `959fca1600af5791118a17912cc944b6b9ca3464`.
- Production at a `390 × 844` viewport reports four equal `93.5px` columns; every tab has `top=790`, `bottom=844`, and remains inside the fixed navigation bounds.
- The production `我的` control navigates successfully to `/me`.

## Documentation Sync

- Archived this plan, removed it from `docs/current-plan.md`, wrote the production handoff, and added an archive index entry.
- No architecture decision or runbook update is required because this repairs the existing four-tab UI contract without changing routes or deployment procedures.

## Risks / Open Questions

- None open for this defect. The fourth button already existed and no data/API behavior changed.
- Future tab additions must update both rendered controls and the CSS track count; the regression now locks the current four-tab contract.
