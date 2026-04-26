# Hourly Bug Audit Automation Update

- title: Hourly Bug Audit Automation Update
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-26
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

### 2026-04-26 Update

- Added a `P1 GitHub issue` rule to both the repository snapshot and the live `bug` automation.
- When the patrol newly records or confirms an active `P1`, it must also create one desensitized GitHub issue with `gh issue create`.
- If `gh` is not on PATH, the automation should try `~/.local/bin/gh`.
- Required issue metadata:
  - title prefix: `[P1][hone-scanner]`
  - body fields: `Reporter: hone-scanner`, `Severity: P1`, `Status`, `Bug doc`, and `CC: @chet-zzz @Finn-Fengming`
- The issue body must stay brief and avoid session text, private user content, account identifiers, phone numbers, tokens, absolute local paths, long log excerpts, or internal prompts.
- Duplicate issues are disallowed; existing issue links should be preserved in the bug doc or index.
- If `gh` is unavailable, unauthenticated, unauthorized, or issue creation fails, the automation should record a short sanitized pending/failure note in the bug doc instead of leaking raw details.

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
- 2026-04-26: Updated the live Codex automation through the app automation tool and verified both `.codex/automations/bug/automation.toml` and `~/.codex/automations/bug/automation.toml` contain `P1 GitHub issue`, `hone-scanner`, `@chet-zzz`, `@Finn-Fengming`, and `gh issue create`.

## Risks / Follow-ups

- After adding `P3`, the patrol may surface more low-severity quality tickets than before; if noise grows, the next tuning point should be stronger repeatability / evidence thresholds rather than removing `P3`.
- This round only changes automation policy and documentation, not runtime code. If the patrol starts finding recurring `P3` quality defects, they should be clustered by root cause instead of opening many near-duplicate docs.
- GitHub issue authorship is still controlled by the authenticated `gh` account. The automation records `Reporter: hone-scanner` in the issue content, but a real `hone-scanner` GitHub author requires `gh auth` to use that account or token.
- This task was kept out of `docs/current-plan.md` because it was a one-shot automation policy adjustment, not a cross-session tracked workstream.

## Next Entry Point

- `.codex/automations/bug/automation.toml`
