# ACP Version-aware Runtime And Revision-bound Source Deployment

- title: ACP Version-aware Runtime And Revision-bound Source Deployment
- status: in_progress
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: shared
- related_files: `crates/hone-channels/src/runners/`, `crates/hone-channels/src/agent_session/emitter.rs`, `crates/hone-channels/src/outbound.rs`, `crates/hone-core/src/build_info.rs`, `crates/hone-web-api/src/routes/meta.rs`, `crates/hone-web-api/src/routes/chat.rs`, `packages/app/src/context/sessions.tsx`, `scripts/deploy_source_runtime.sh`, `tests/fixtures/acp/`, `tests/regression/ci/test_source_runtime_deploy_contract.sh`
- related_docs: `docs/current-plans/acp-runtime-refactor.md`, `docs/adr/0002-agent-runtime-acp-refactor.md`, `docs/decisions.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/runbooks/opencode-setup.md`, `docs/runbooks/source-web-startup.md`
- related_prs: feature branch `codex/acp-versioned-runtime-deploy`; no release or tag

## Summary

ACP adapter identity and adapter version are now separate facts. Codex ACP and OpenCode select their stream dialect from the live connection's `initialize.agentInfo.version`, retain explicit compatibility status, and fail before a session or prompt when the version is older, missing, unparsable, or from an unknown major. Codex's persistent role boundary is unchanged: every `session/prompt`, including after native compaction, contains only the current turn; developer instructions, history, tool calls/results, and compact summaries are never rewrapped as user input.

Source deployment is now a revision-bound state machine rather than a sequence of ad hoc restarts. It builds an immutable Web/Discord/MCP unit, drains active chats, distinguishes a loaded launchd job from a live PID, waits for process locks, verifies startup/readiness/channel login and build provenance, and restores all previously running services on any failure.

## What Changed

- Added adapter-specific typed profiles for codex-acp `1.1.7` and OpenCode `1.18.11`, selected only from the real initialize result. Exact captures are `validated`; only newer releases in the same major use the nearest fixture conservatively as `compatible_newer`.
- Added version-labelled external fixtures with capture dates and protocol-level assertions for Codex execute/raw-output variance and OpenCode thought, split-answer, and detailed usage variance.
- Kept answer, reasoning, tool status, usage, reset, and terminal events distinct. Full direct/admin surfaces may show sanitized reasoning, compact surfaces show a generic analysis signal, and unsuitable/OpenAI-compatible surfaces omit it without changing final answer bytes.
- Added sanitized build/runtime provenance to `/api/meta` and bounded runtime profile files. The output excludes prompts, raw protocol payloads, paths, credentials, and user data.
- Added `scripts/deploy_source_runtime.sh` plus a CI-safe fake-boundary regression. The script refuses dirty, unexpected, unpushed, or unknown-port-owner revisions/topologies by default and commits the `current` symlink only after the full managed runtime is verified.
- A real local preflight exposed the prior `com.honeclaw.source.runtime` supervisor with Web/Discord child processes. The first migration also proved that launchd can reparent those children to PID 1 instead of terminating them. The deployer now executable-verifies each captured orphan, sends TERM with a bounded grace/KILL escalation, atomically installs persistent revision-bound Web/Discord LaunchAgent plists, recoverably disables the legacy plist on success, and restores/bootstrap-verifies the legacy topology on failure.
- Updated manual Codex/OpenCode probes to work from a worktree. The OpenCode probe uses the current read-only MCP tool name and optionally selects an explicit model through `session/set_model`.

## Verification

- Workspace check and workspace tests excluding Apple clients passed.
- Web tests passed `334/334`; Public Community Edge typecheck and tests passed `45/45`.
- Complete CI-safe regressions passed, including direct-job and legacy-supervisor source-deployment success, unknown-owner/dirty/unpushed refusal, persistent plist installation, partial-start failures, and full rollback/bootstrap cases.
- Post-review `hone-channels` library tests passed `715` with one existing host-dependent ignore; the build-info unit contract passed.
- Real Codex CLI `0.146.0` plus codex-acp `1.1.7` initialize/session probing passed.
- Real OpenCode `1.18.11` completed initialize, model selection, a read-only Hone MCP call, streamed reasoning/answer/usage, and `end_turn` with an explicit free probe model.
- The configured default OpenAI OAuth returned `401`; it is recorded as a provider-auth limitation, not a successful default-model probe.
- Exact pushed-revision source deployment and post-deploy `/api/meta`/port/channel/profile canary are still pending.

## Risks / Follow-ups

- A same-major newer adapter is compatible, not validated. Capture a real exchange and add a new version-labelled fixture before promoting its dialect.
- OpenCode's configured default OpenAI OAuth must be repaired independently; the successful explicit free-model probe proves ACP transport/MCP/streaming only.
- Runtime profile files are observational. An empty profile list means no adapter has completed initialize in that runtime; it must never be filled from a runner-name guess.
- Frontend Vite processes remain deliberately outside the managed backend/Discord deployment unit.

## Next Entry Point

Commit and push the reviewed branch, deploy that exact revision with `scripts/deploy_source_runtime.sh`, verify zero active chats, ports `8077/8088`, fresh Discord login, executable paths, `/api/meta.build.git_sha`, and a real Codex turn that writes the expected validated runtime profile. If any stage fails, confirm the script restored every service before attempting another revision.
