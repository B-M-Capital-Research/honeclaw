# Codex ACP Session Continuity Diagnosis

- title: Codex ACP Session Continuity Diagnosis
- status: done
- created_at: 2026-07-30
- updated_at: 2026-07-30
- owner: Codex
- related_files: `crates/hone-channels/src/runners/codex_acp.rs`, `crates/hone-channels/src/runners/acp_common/protocol.rs`, `crates/hone-channels/src/runners/types.rs`, `crates/hone-channels/src/runners/tests.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `crates/hone-channels/src/agent_session/restore.rs`, `bins/hone-cli/src/onboard.rs`, `bins/hone-discord/src/handlers.rs`, `memory/src/session.rs`, `tests/regression/manual/test_codex_acp_initialize.sh`, `tests/regression/manual/test_codex_acp_event_stream.sh`
- related_docs: `docs/current-plans/acp-runtime-refactor.md`, `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/runbooks/hone-cli-install-and-start.md`, `docs/bugs/archive/web_direct_session_update_prompt_echo_leak.md`
- related_prs: N/A

## Summary

The reported observation was correct for the pre-follow-up implementation. Since commit `be5d7414` on 2026-05-04, `codex_acp` had intentionally created a fresh upstream Codex ACP session for every Hone turn instead of calling `session/load`. That prevented historical `session/update` replay but split native Codex page history into one rollout per message.

The follow-up now implements the requested native behavior: each deterministic Hone logical session maps to one persistent Codex session. The first turn uses `session/new`; every later turn uses `session/resume` for the same mode-marked ID. Hone seeds its durable transcript only once when entering persistent mode, then the Codex harness owns conversation history and automatic compaction.

This remains isolated per actor/session identity rather than one global thread shared by the whole Hone installation. A real two-turn Discord-path probe produced one Hone session, one native Codex session, and two complete native turns; turn two returned the exact sentinel from turn one.

## What Changed

### Diagnosis stage

- The existing ACP runtime plan records the live continuity evidence and the compatibility risks found during diagnosis.
- Temporary `config.yaml` changes used to admit a synthetic Discord admin and run a one-off compatible adapter were restored exactly.

### CLI and compatibility follow-up

- Replaced the unrelated Homebrew `zed-industries/codex-acp 0.11.1` formula with npm `@agentclientprotocol/codex-acp 1.1.7`.
- Installed npm `@openai/codex 0.146.0`; `/opt/homebrew/bin/codex` and `/opt/homebrew/bin/codex-acp` now resolve to the paired npm installation.
- Raised Hone's validated startup floors to Codex `0.146.0` and adapter `1.1.7`.
- Passed the configured base model and reasoning effort as Codex process `-c` overrides before `session/new`, and removed the Codex-specific `session/set_model` call.
- Made ACP initialize version probes use the configured `CODEX_PATH`; lightweight onboarding/doctor checks continue to use the adapter's supported `codex-acp --version`.
- Updated onboarding copy, configuration examples, manual regressions, runbook, technical specification, invariants, decisions, and repo map.

### Persistent native-session follow-up

- Added ACP `session/resume` support and deliberately avoided `session/load`. Adapter source inspection and a real cross-process probe showed that load streams old thread history, while resume returns without historical `session/update` replay.
- Stored `codex_acp_session_mode=persistent_resume_v1` beside the native session ID. Legacy pre-marker IDs are not resumed because they represent one-turn rollouts from the previous implementation.
- On cold start, created one persistent native session and injected Hone's normalized local transcript once for upgrade/runner-switch continuity. Later turns resume the native session and omit restored transcript from the prompt.
- Made resume failure explicit rather than silently falling back to `session/new`, preserving the one-native-session guarantee.
- Marked Codex as relying on native context compaction so AgentSession does not locally compact and replay a context-overflow turn. OpenCode's existing fresh-session behavior remains unchanged.

## Verification

- Controlled Discord probe:
  - Hone session stayed `Actor_discord__direct__acp_5fctx_5fprobe_5f20260730_5fdiscord` for both turns.
  - Turn-one Codex ACP session: `019fb385-8327-7ce1-a5a9-b87bcd27bebc`.
  - Turn-two Codex ACP session: `019fb385-ba48-73c3-a991-f5628bc46736`.
  - Turn one stored `DISCORDCTX-20260730-NATIVE` and returned `已记住。`.
  - Turn two asked for the prior sentinel and returned exactly `DISCORDCTX-20260730-NATIVE`.
- `data/runtime/logs/acp-events.log` showed a new `session/new` in each turn. The second `session/prompt` included a restored JSON transcript containing the first user message and assistant reply.
- The deterministic Hone session JSON contained four ordered rows: `user`, `assistant`, `user`, `assistant`.
- The two matching files under `~/.codex/sessions/2026/07/30/` each contained one native `user_message`, one native `agent_message`, and one task completion, confirming why native Codex history looks split.
- Regression tests:
  - `cargo test -p hone-channels codex_acp_does_not_reuse_remote_session_metadata -- --nocapture`
  - `cargo test -p hone-channels codex_prompt_text_includes_restored_transcript_when_session_is_recreated -- --nocapture`
  - `cargo test -p hone-discord group_session_id_is_channel_scoped -- --nocapture`
- All three tests passed.

Compatibility follow-up verification:

- Installed commands:
  - `/opt/homebrew/bin/codex --version` → `codex-cli 0.146.0`
  - ACP `initialize.agentInfo.version` → `1.1.7`
  - `codex update --help` succeeds and is now the documented update command.
- Automated checks:
  - `cargo test -p hone-channels codex_acp -- --nocapture`
  - `cargo test -p hone-channels configured_codex -- --nocapture`
  - `cargo test -p hone-channels codex_version_matrix -- --nocapture`
  - `cargo test -p hone-core agent_runner_kind_keeps_wire_values_and_probe_mapping -- --nocapture`
  - `cargo test -p hone-cli onboard_runner_kind_config_values_and_probe_requirements -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `cargo check -p hone-cli --tests`
- Real ACP checks:
  - `tests/regression/manual/test_codex_acp_initialize.sh` passed.
  - `tests/regression/manual/test_codex_acp_event_stream.sh` passed with one completed `hone/skill_tool` call and terminal `stopReason=end_turn`.
- Upgraded Discord probe:
  - Hone session remained `Actor_discord__direct__acp_5fupgrade_5fdiscord_5f20260730`.
  - Upstream ACP sessions were `019fb3a7-de65-7d52-a9c9-2106b93e2209` and `019fb3a8-2137-7ae0-b43e-d933751a2a22`.
  - Both `session/new` responses selected `gpt-5.6-sol[xhigh]`; no `session/set_model` request occurred.
  - The second ACP prompt contained restored history and returned exactly `UPGRADED-DISCORD-CTX-20260730`.
  - The temporary synthetic Discord admin entry was restored after the probe.

Persistent native-session verification:

- A raw cross-process `session/resume` probe resumed `019fb3a8-2137-7ae0-b43e-d933751a2a22`; no historical `session/update` was emitted before the response.
- Real Discord actor/session: `Actor_discord__direct__acp_5fpersistent_5fdiscord_5f20260730`.
- Native Codex session stayed `019fb3c2-f2f7-7140-8140-7520409d79be` for both turns.
- Codex Desktop's own task list indexed that same ID as one local task, so the thread is page-visible rather than only present as a rollout file.
- ACP log counts were exactly one `session/new`, one `session/resume`, and two `session/prompt`. Neither prompt contained a restored-transcript block for this brand-new actor.
- The single native rollout file under `~/.codex/sessions/2026/07/31/` contained two `user_message`, two `agent_message`, and two task completions. The outputs were `已记住。` and `ONE-CODEX-SESSION-20260730`.
- Focused automated checks passed:
  - `cargo test -p hone-channels codex_acp_reuses_only_persistent_remote_session_metadata -- --nocapture`
  - `cargo test -p hone-channels codex_prompt_text_includes_restored_transcript_when_persistent_session_is_initialized -- --nocapture`
  - `cargo test -p hone-channels self_managed_runner_context_overflow_is_not_locally_compacted_or_retried -- --nocapture`
- Full post-rebase package verification passed: `cargo test -p hone-channels --lib` → 699 passed, 1 host-dependent OCR test ignored; `cargo check -p hone-channels --tests` and `cargo check -p hone-cli --tests` both passed.
- The Codex initialize and real tool event-stream manual regressions were rerun after the persistent-session change and both passed.
- The temporary synthetic Discord administrator entry was restored exactly after the probe.

## Risks / Follow-ups

- Native Codex and Hone session identities are now intentionally one-to-one only within a deterministic logical-session scope. Direct actors and channel/group scopes must never share one global Codex session.
- After the cold-start seed, absence of `### Restored Conversation Transcript ###` is expected. Diagnose continuity from the persisted mode/ID metadata, the second-turn `session/resume` request/result, and the single matching native Codex rollout.
- A deleted or corrupted marked native session currently fails the turn. Recovery requires a deliberate metadata repair/reset; automatic `session/new` fallback would silently fragment the Codex page again.
- The diagnosis-stage adapter/config blockers are resolved by the CLI upgrade and the pre-session process configuration change.
- Model identifiers exposed by `session/new` remain adapter-specific (`gpt-5.6-sol[xhigh]` in `1.1.7`). Future adapter upgrades should rerun initialize, event-stream, and two-turn continuity probes before raising the validated floor.

## Next Entry Point

- For continuity behavior: `crates/hone-channels/src/runners/codex_acp.rs` and `crates/hone-channels/src/runners/acp_common/protocol.rs`, especially the mode marker, `session/new`, and `session/resume`.
- For persisted mapping/history: the actor's logical session metadata under `data/sessions/`, plus its one native rollout under `~/.codex/sessions/`.
- For future upgrades: start from `docs/runbooks/hone-cli-install-and-start.md`, keep the Codex and adapter npm packages paired, and rerun the listed manual regressions plus a two-turn channel probe.
