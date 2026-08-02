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
- A post-deploy Discord turn exposed a second real boundary: `launchctl submit` had started the canary without the `PATH` stored in the plist, so the later Codex version probe failed with `ENOENT` even though ports and Discord login were healthy. Managed jobs now bootstrap the exact persisted plist, its stable allowlisted `PATH` rejects Codex turn-local/cache entries, and runner commands must pass a version probe before shutdown. The fake-boundary regression deletes/rejects the external temporary path rather than mirroring a private shell function.
- The completion audit found that the live selector trusted `agentInfo.version` but had not yet verified `agentInfo.name`, Codex CLI unknown majors were accepted by a floor-only check, and build provenance lacked an explicit source kind/startup log. Initialize now requires the exact captured `@agentclientprotocol/codex-acp`/`opencode` identity before session creation, every version axis fails closed on unknown majors, invalid external version text is not echoed, and build/runtime logs expose only bounded Git SHA/timestamp/profile/source/hash plus actual adapter/companion version/status/dialect facts. The first exact canary caught a stale shorthand in the Codex fixture; the fixture and boundary double now retain the actual package identity instead of teaching the implementation an alias.
- Updated manual Codex/OpenCode probes to work from a worktree. The OpenCode probe uses the current read-only MCP tool name and optionally selects an explicit model through `session/set_model`.

## Verification

- Workspace check and workspace tests excluding Apple clients passed.
- Web tests passed `334/334`; Public Community Edge typecheck and tests passed `45/45`.
- Complete CI-safe regressions passed, including direct-job and legacy-supervisor source-deployment success, unknown-owner/dirty/unpushed refusal, persistent plist installation, partial-start failures, and full rollback/bootstrap cases.
- Post-review `hone-channels` library tests passed `715` with one existing host-dependent ignore; the build-info unit contract passed.
- Real Codex CLI `0.146.0` plus codex-acp `1.1.7` initialize/session probing passed.
- Real OpenCode `1.18.11` completed initialize, model selection, a read-only Hone MCP call, streamed reasoning/answer/usage, and `end_turn` with an explicit free probe model.
- The configured default OpenAI OAuth returned `401`; it is recorded as a provider-auth limitation, not a successful default-model probe.
- Exact local implementation commit `b8e183124e21c9d54e6a4449db88f711d16279d3` was pushed, merged into `main`, and deployed through the source-runtime state machine. The Web and Discord launchd jobs each remained at `runs=1` with stable PIDs; the apparent startup-lock errors were historical and their log did not change during observation.
- Ports `8077`, `8088`, `3000`, and `3001` remained bound, `/api/meta` reported the exact implementation revision, Discord completed a fresh login, and active chats returned to `0`.
- The exact Discord plist `PATH` resolved Codex CLI `0.146.0` and codex-acp `1.1.7`. A real `hone-cli` ACP canary returned exactly `LOCAL_PERSISTENT_PATH_OK` with `tool_calls=0`, and the next real Discord turn selected the validated `codex-acp 1.1.7` dialect, completed successfully, and sent one reply.
- GCE rollout is explicitly paused by user direction. It is not part of the completed local acceptance and no further remote deployment should be attempted without a new instruction.
- Completion-audit gates passed: full workspace check/test, Web `334/334`, Edge `45/45`, complete CI-safe regressions, identity/version focused tests, and external `/api/meta` build-source deployment-contract cases. Exact pushed-revision local redeployment remains before this follow-up can be marked fully done.

## Risks / Follow-ups

- A same-major newer adapter is compatible, not validated. Capture a real exchange and add a new version-labelled fixture before promoting its dialect.
- OpenCode's configured default OpenAI OAuth must be repaired independently; the successful explicit free-model probe proves ACP transport/MCP/streaming only.
- Runtime profile files are observational. An empty profile list means no adapter has completed initialize in that runtime; it must never be filled from a runner-name guess.
- Frontend Vite processes remain deliberately outside the managed backend/Discord deployment unit.

## Next Entry Point

Local recovery is complete. Resume only on a new explicit request to deploy to GCE: first re-discover the remote runtime/revision and active-chat state, then deploy the exact reviewed revision through the same state-machine contract and retain rollback evidence if any stage fails.
