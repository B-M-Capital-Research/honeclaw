# Hourly Bug Audit Automation Update

- title: Hourly Bug Audit Automation Update
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: Codex
- related_files:
  - `.codex/automations/bug/automation.toml`
  - `~/.codex/automations/bug/automation.toml`
- related_docs:
  - `.codex/README.md`
  - `docs/archive/index.md`
- related_prs: N/A

## Summary

Updated the hourly `bug` automation so it no longer relies mainly on recent code commits when doing defect patrol. The new prompt now prioritizes the most recent hour of real session records and runtime logs, explicitly audits output quality and formatting defects, and introduces `P3` for bugs that hurt answer quality but do not break the functional chain.

## What Changed

- Expanded the evidence priority to start from `data/sessions.sqlite3` and recent `data/runtime/logs/*.log` / `data/logs/*.log` before falling back to recent code commits and bug docs.
- Added explicit patrol checks for:
  - AI responses that fail user intent
  - malformed Markdown / table / JSON / card / code block structure
  - leaked internal errors, tool drafts, or reasoning traces
  - poor result consumption after tool calls
  - language / tone / formatting drift
  - degraded but still functional answer quality
- Added `P3` to the severity whitelist and required the automation to explain why a `P3` issue does not break the main functional chain.
- Kept the task boundary unchanged: this automation still only maintains `docs/bugs/` and does not fix product code.
- Synced the repository snapshot `.codex/automations/bug/automation.toml` with the intended live automation behavior.

## Verification

- Compared the existing live automation id `bug` with the repository snapshot before editing.
- Confirmed the session and log evidence paths exist in the workspace:
  - `data/sessions.sqlite3`
  - `data/runtime/logs/`
  - `data/logs/`
- Verified `data/sessions.sqlite3` exposes recent-message timestamps via:
  - `sessions.updated_at`
  - `sessions.last_message_at`
  - `session_messages.timestamp`
  - `session_messages.imported_at`
- Updated the live Codex automation `bug` and re-read `~/.codex/automations/bug/automation.toml` to confirm the new prompt is active.

## Risks / Follow-ups

- After adding `P3`, the patrol may surface more low-severity quality tickets than before; if noise grows, the next tuning point should be stronger repeatability / evidence thresholds rather than removing `P3`.
- This round only changes automation policy and documentation, not runtime code. If the patrol starts finding recurring `P3` quality defects, they should be clustered by root cause instead of opening many near-duplicate docs.
- This task was kept out of `docs/current-plan.md` because it was a one-shot automation policy adjustment, not a cross-session tracked workstream.

## Next Entry Point

- `.codex/automations/bug/automation.toml`
