---
name: sync-open-source-repo
description: Safely sync selected changes from an internal repository into an open-source mirror or public fork when the two repos may have diverged in history, structure, docs, branding, or release policy. Use when Codex needs to compare an internal repo against a public repo, inspect diffs before syncing, create an isolated sync branch or worktree, port only a targeted subset of changes, avoid force-pushing over upstream work, or summarize sync risks and follow-up actions.
---

# Sync Open Source Repo

## Overview

Use this skill to port changes from an internal repo into a public mirror without accidentally overwriting unrelated upstream work. Prefer isolated comparison, narrow file selection, and branch-based delivery over direct pushes to public `main`.

## Workflow

1. Confirm the target public repo, target branch, and whether the goal is:
   - sync one feature or fix
   - sync a selected file set
   - align a larger slice of the internal repo
2. Inspect divergence before editing anything:
   - compare remote heads
   - check whether the histories share a merge base
   - inspect commit-count and file-level diffs
3. Create an isolated sync workspace:
   - fetch the public branch into a local tracking ref
   - create a detached worktree or a dedicated sync branch from the public branch
   - keep the internal repo’s current branch untouched
4. Port only the intended changes:
   - prefer patching only the relevant files or commit range
   - exclude internal-only docs, secrets, private workflows, branding, and operational runbooks unless the user explicitly wants them public
   - if a full patch fails, reduce scope and apply code/config changes first, then adapt docs separately
5. Validate in the public worktree:
   - run compile/test commands that cover the changed area
   - distinguish pre-existing public-repo failures from failures introduced by the sync
6. Deliver on a public sync branch:
   - commit only the synced subset
   - push to a non-`main` branch unless the user explicitly asks to update `main`
   - summarize what was synced, what was intentionally skipped, and any remaining manual review points

## Operating Rules

- Default to branch sync, not direct push to public `main`.
- Treat the public repo as the source of truth for public-only structure, README tone, branding, and release constraints.
- Prefer targeted sync over “make public look like internal”.
- If histories diverge heavily, compare by file/feature intent, not by raw commit replay.
- If the public repo has unique commits, never overwrite them without explicit user instruction.
- Keep internal planning docs, handoffs, bugs, runbooks, and private assets out of the public sync unless the user explicitly wants them published.

## References

- Read [references/sync-guide.md](references/sync-guide.md) before doing a real sync. It contains the detailed checklist, common pitfalls, and decision rules for worktrees, patching, validation, and push strategy.
