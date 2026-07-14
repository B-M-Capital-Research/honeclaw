# GPT-5.6 Codex ACP Runtime Simplification

- title: GPT-5.6 Codex ACP Runtime Simplification
- status: done
- created_at: 2026-07-13
- updated_at: 2026-07-14
- owner: Codex
- related_files: `Cargo.toml`, `crates/hone-core/src/config/agent.rs`, `crates/hone-channels/src/{execution.rs,turn_builder.rs}`, `crates/hone-channels/src/runners/codex_acp.rs`, `config.example.yaml`, `soul.md`, `bins/hone-cli/`, `bins/hone-desktop/`, `packages/app/`
- related_docs: `docs/decisions.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/technical-spec.md`, `docs/runbooks/hone-cli-install-and-start.md`
- related_prs: N/A

## Summary

Honeclaw no longer ships or selects the in-process `function_calling` agent or sequential `multi-agent` runner. The default local route is Codex ACP with `gpt-5.6-sol` and `xhigh`; heartbeat/transient runs now use the same configured runner preparation path as normal sessions.

The Codex ACP compatibility floor is `@openai/codex >= 0.144.1` plus `@agentclientprotocol/codex-acp >= 1.1.2`. The deprecated `@zed-industries/codex-acp` package was removed from the local global installation. The adapter receives `CODEX_PATH`, so the binary Hone probes is also the binary it executes.

The 2026-07-14 rebase onto current `origin/main` preserved the newer trusted-host boundary: native CLI/ACP runners remain administrator-only. Because the in-process fallback is gone, non-admin native-runner requests now fail closed and should use `hone_cloud`.

## What Changed

- Removed `agents/function_calling/`, its workspace dependency, the function-calling runner wrapper, `multi_agent.rs`, the old schema/UI/CLI fields, and the manual multi-agent regression.
- Old runner strings resolve to `Unknown` and produce an explicit removed-runner error; there is no silent fallback.
- Set Rust, example-config, Desktop/Web, onboarding, documentation, architecture assets, and active Codex automation defaults to `codex_acp` / `gpt-5.6-sol` / `xhigh`.
- Updated Codex ACP version gates and installation guidance to the current Agent Client Protocol package.
- Reduced `soul.md` from 15,532 bytes to 2,952 bytes and made it the persona/work-style layer. Hard finance and company-profile policies stay in Rust.
- Removed the per-turn 4,000-character full skill catalog. The static prefix now contains compact skill-use rules, while up to five query-relevant summaries go in the current turn and misses use `discover_skills`.
- Added regression coverage for prompt budget, skill-prefix stability, defaults, version floors, reasoning args, and explicit retired-runner behavior.

## Verification

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `cargo test --workspace --all-targets --exclude hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`
- `bun run typecheck:web`
- `bun run test:web` — 184 passed
- `bash tests/regression/run_ci.sh`
- `bash tests/regression/manual/test_codex_acp_initialize.sh` — initialize/session-new passed
- Local versions: `codex-cli 0.144.1`; `@agentclientprotocol/codex-acp 1.1.2`
- Live model probe: `codex exec -m gpt-5.6-sol -c model_reasoning_effort=\"xhigh\" ...` reported model `gpt-5.6-sol`, reasoning effort `xhigh`, and returned the expected sentinel.
- Static removal scan confirmed the retired source files are absent and active config/UI/runtime paths no longer expose their schema.

## Risks / Follow-ups

- Existing user configs that still select either retired runner now fail intentionally and must be migrated to a supported runner.
- Deployments serving non-admin actors cannot use a native CLI/ACP runner as a safe fallback; configure `hone_cloud` for those actors or explicitly register the operator as an administrator.
- Full Tauri bundle validation still requires prepared sidecar binaries; the repository's normal CI contract excludes `hone-desktop`, and the desktop code passed its supported dev/IDE check with `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1`.
- Historical releases, bug records, archived plans, and handoffs retain old runner names as evidence.
- Pre-existing event-engine working-tree changes were preserved and were not reformatted or folded into this task.

## Next Entry Point

- Runtime factory: `crates/hone-channels/src/core/bot_core.rs`
- Codex ACP adapter: `crates/hone-channels/src/runners/codex_acp.rs`
- Defaults: `crates/hone-core/src/config/agent.rs` and `config.example.yaml`
- Prompt layering: `soul.md`, `crates/hone-channels/src/prompt.rs`, and `crates/hone-channels/src/turn_builder.rs`
