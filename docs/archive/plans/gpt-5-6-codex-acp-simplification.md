# GPT-5.6 Codex ACP Runtime Simplification

- title: GPT-5.6 Codex ACP Runtime Simplification
- status: archived
- created_at: 2026-07-13
- updated_at: 2026-07-14
- owner: Codex
- related_files: `Cargo.toml`, `crates/hone-core/src/config/agent.rs`, `crates/hone-channels/src/core/bot_core.rs`, `crates/hone-channels/src/runners/codex_acp.rs`, `crates/hone-channels/src/turn_builder.rs`, `config.example.yaml`, `soul.md`
- related_docs: `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`, `docs/decisions.md`, `docs/invariants.md`, `docs/repo-map.md`
- related_prs: N/A
- verification: workspace check/test, Web typecheck/test, desktop dev check, CI regressions, Codex ACP initialize probe, live GPT-5.6 Sol/xhigh probe
- risks: old runner config values now fail explicitly; full Tauri packaging requires prepared sidecars
- goal_id: `019f5708-14fb-7802-831a-873d2c9e3626`

## Goal

Remove the legacy in-process function-calling and sequential multi-agent paths, converge Codex ACP on the current package/version contract and GPT-5.6 Sol/xhigh defaults, and reduce duplicated runtime prompt material without weakening hard policies.

## Completed Scope

- [x] Verified current official GPT-5.6 model/reasoning contract and current package registry versions.
- [x] Removed `agents/function_calling`, `hone-agent`, and all production callers.
- [x] Removed `multi_agent.rs`, its schema, CLI/Desktop/Web settings, tests, and current product documentation.
- [x] Moved transient heartbeat execution onto the configured unified runner path.
- [x] Made invalid and retired runner values fail explicitly without fallback.
- [x] Rebased onto current `origin/main` while preserving its trusted-host boundary: non-admin native CLI/ACP requests fail closed and use `hone_cloud`.
- [x] Upgraded Codex ACP version floors, package guidance, diagnostics, defaults, and active automations.
- [x] Set the canonical default to `codex_acp`, `gpt-5.6-sol`, and `xhigh`.
- [x] Reduced the persona prompt and stopped injecting a full skill catalog into every static prompt.
- [x] Updated long-lived context documents, architecture assets, runbooks, public copy, and regression coverage.
- [x] Completed formatting, workspace, Web, desktop-dev, CI, ACP initialization, and live-model validation.

## Verification

See `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md` for the complete command/result list and follow-up boundaries.

## Archive Note

The task is complete and no longer belongs in `docs/current-plan.md`. Historical references to the removed runners remain in release notes, bug records, archived plans, and older handoffs only.
