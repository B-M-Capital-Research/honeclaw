# Bug Index And Automation Doc Mode

- title: Bug Index And Automation Doc Mode
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: Codex
- related_files:
  - `docs/bugs/README.md`
  - `.codex/automations/bug/automation.toml`
  - `.codex/automations/bug-2/automation.toml`
  - `~/.codex/automations/bug/automation.toml`
  - `~/.codex/automations/bug-2/automation.toml`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`
- related_prs: N/A

## Summary

Added a bug navigation page at `docs/bugs/README.md` so collaborators can see the current backlog, fixed items, and historical analysis records before deciding what to repair. Updated both bug-related automations so they treat this page as the primary navigation/index and keep it synchronized whenever a bug document changes status or repair conclusion.

## What Changed

- Added `docs/bugs/README.md` with:
  - usage rules
  - backlog counts
  - an active-bugs table
  - a fixed/closed table
  - a historical-analysis / partial-mitigation table
- Updated the hourly `bug` automation to:
  - read the bug index first
  - treat `docs/bugs/README.md` as the navigation source of truth
  - update the index whenever bug docs change
- Updated the `bug-2` repair automation to:
  - choose candidates from the index active queue first
  - reconcile index/doc drift before fixing
  - update both the bug document and index row after a fix
- Kept this change outside `docs/current-plan.md` because it is a one-shot workflow and documentation upgrade, not a long-running tracked implementation stream.

## Verification

- Enumerated every current file under `docs/bugs/` and classified them into active backlog, fixed/closed, or historical-analysis buckets.
- Verified current active repair candidates in the new index all map to underlying bug documents with `New` status.
- Updated both repository automation snapshots for `bug` and `bug-2`.
- Updated the live Codex automations `bug` and `bug-2`, then re-read their `~/.codex/automations/*/automation.toml` files to confirm the new index-maintenance workflow is active.

## Risks / Follow-ups

- `docs/bugs/` still contains a few legacy analysis documents whose metadata is not yet normalized to the standard bug card schema; the new index preserves them in a separate section instead of forcing a bulk rewrite in this round.
- If future work splits one bug into several more precise bug cards, both automations must update `docs/bugs/README.md` in the same change set or the index will drift.
- If the backlog grows quickly, the next improvement should be adding `owner` or `last_checked` columns to the index rather than moving state back into ad-hoc prose.

## Next Entry Point

- `docs/bugs/README.md`
