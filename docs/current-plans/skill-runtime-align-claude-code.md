# Skill Runtime 对齐 Claude Code

- title: Skill Runtime 对齐 Claude Code
- status: in_progress
- created_at: 2026-03-31
- updated_at: 2026-04-16
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/decisions.md`

## Goal

Bring the skill runtime in line with the Claude Code interaction model without regressing the existing Hone agent flow.

## Scope

- Listing disclosure is aligned and now stage-aware: discovery, related-skill hints, direct invoke, `/skill` search, and MCP-side load/list surfaces all hide skills that cannot run in the current stage.
- Full prompt injection on invoke is already aligned.
- Slash/direct invoke and session resume are aligned.
- This round also closed two concrete runtime gaps:
  - `HONE_SKILLS_DIR` is now forwarded into `hone-mcp`, so sandboxed skill loading sees the same system skill root as prompt-time discovery.
  - Skills that require blocked tools such as `cron_job` now disappear from visible surfaces and return an explicit “missing tool in this stage” error if forced through a lower-level path.
- This round also closed a runner-specific visibility breakage on `codex_acp`: `cron_job`'s MCP schema exposed `tags.items` as a bare string instead of a JSON Schema object, so Codex ACP failed to convert the tool even when cron was enabled.
- Remaining gaps: real hook execution, watcher hot reload, and any additional turn-scope enforcement beyond the current stage/tool filtering contract.

## Validation

- `cargo test -p hone-tools`
- `cargo test -p hone-channels handle_tools_list_exposes_cron_job_only_when_allow_cron_is_enabled`
- `cargo test -p hone-channels handle_tools_call_rejects_cron_job_when_stage_allowed_tools_excludes_it`
- `cargo test -p hone-channels resolve_prompt_input_hides_cron_only_skills_when_cron_is_not_allowed`
- `bash tests/regression/ci/test_skill_runtime_stage_consistency.sh`
- `bash tests/regression/manual/test_hone_mcp_skill_dir_env.sh`
- `bash tests/regression/manual/test_hone_mcp_cron_visibility.sh`
- `bash tests/regression/manual/test_codex_acp_session_reload_cron_visibility.sh`

## Documentation Sync

- Keep `docs/current-plan.md` and this file aligned.
- Record concrete subtask outcomes in `docs/handoffs/2026-04-16-skill-runtime-stage-visibility.md` when a release-worthy runtime increment lands.
- If the runtime contract changes, update `docs/decisions.md` or `docs/adr/*.md`.

## Risks / Open Questions

- Hook execution and any stricter turn-scope enforcement can still change behavior across multiple channels.
- The new “visible means usable” contract now depends on stage constraints being plumbed consistently anywhere skills are surfaced; new listing surfaces must reuse the same constraint model.
- ACP adapters can still regress if a tool's MCP `inputSchema` drifts away from strict JSON Schema; runner-visible tool conversion is now part of the practical compatibility surface.
- Remaining work appears to depend on runner / infra changes rather than prompt-only edits.
