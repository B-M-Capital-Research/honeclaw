# Codex ACP Session Continuity Diagnosis

- title: Codex ACP Session Continuity Diagnosis
- status: done
- created_at: 2026-07-30
- updated_at: 2026-07-31
- owner: Codex
- related_files: `crates/hone-channels/src/runners/codex_acp.rs`, `crates/hone-channels/src/runners/acp_common/protocol.rs`, `crates/hone-channels/src/runners/types.rs`, `crates/hone-channels/src/runners/tests.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `crates/hone-channels/src/agent_session/restore.rs`, `crates/hone-channels/src/turn_builder.rs`, `crates/hone-channels/src/execution.rs`, `crates/hone-channels/src/sandbox.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-web-api/src/routes/public.rs`, `skills/skill_manager/SKILL.md`, `soul.md`, `bins/hone-cli/src/onboard.rs`, `bins/hone-discord/src/handlers.rs`, `memory/src/session.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_codex_acp_initialize.sh`, `tests/regression/manual/test_codex_acp_event_stream.sh`
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

### Static system-prompt lifecycle follow-up

- Corrected the persistent Codex prompt builder, which still appended the complete static system prompt on every resumed turn even though native history was already persistent.
- The first native prompt now sends the static system prompt and any one-time restored transcript. Ordinary resumed prompts omit both.
- Codex ACP 1.1.7 `tool_call` / `tool_call_update` records carrying `_meta.contextCompaction=true` are now recognized as internal harness lifecycle events rather than Hone tool calls. They set one pending system-prompt reseed for the next turn.
- A successful seed/reseed clears `acp_needs_sp_reseed`; a failed seed keeps it pending, and another compaction during the reseed turn keeps it true. Legacy compact text remains a fallback signal.

### Minimal native-turn follow-up

- Trusted Codex ACP Interactive turns now send only `【当前时间】` and `【本轮用户输入】`. The user section keeps normalized attachment/image/PDF/archive content and an explicitly invoked slash-skill task because those are current-turn material.
- Hone no longer injects Session ID/history, compact summaries, channel-routing counters, same-speaker buffers, query-relevant skill hints, entity-loop guidance, or final-answer contracts into those user turns.
- Codex owns retained thread history, MCP/tool lifecycle, image/local-item handling, and compaction. Stable Hone behavior remains in the first/post-compaction system seed.
- OpenCode, scheduled/heartbeat execution, and non-admin strict fallback keep their existing context assembly; the optimization is intentionally Codex-Interactive-specific.

### Native skill discovery follow-up

- Trusted persistent Codex ACP turns now resolve enabled Hone system/custom skills through the shared skill registry and create one symlink per skill under `<actor sandbox>/.agents/skills/`.
- Codex owns metadata discovery and progressive `SKILL.md` loading. Hone no longer adds the MCP `SkillTool` loading instructions to the native Codex system seed, and `discover_skills`, `load_skill`, and `skill_tool` are removed from that turn's MCP allowlist.
- The symlink namespace is limited to `hone__*`. A later turn refreshes changed targets and removes stale Hone-managed symlinks while preserving actor-owned skill entries.
- Live Hone data/action tools still use MCP. OpenCode, transient scheduler/heartbeat work, and strict actor-bound runners retain their existing skill bridge.
- Built-in skill guidance now uses host-neutral skill names. `skill_manager` documents native Codex as the primary Codex path and keeps the legacy bridge as an explicit compatibility path.

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

Static system-prompt lifecycle verification:

- Focused ACP/Codex unit suites passed, including regressions for first-turn/post-compact seed selection, ordinary resumed-turn prompt contents, structured `contextCompaction` handling, and the legacy italic compact notification.
- The full `hone-channels` library suite passed with 705 tests and one host-dependent OCR test ignored; `cargo check -p hone-channels --tests` and the changed-file format check also passed.
- Codex ACP initialize and real MCP tool event-stream manual regressions passed.
- A real two-turn `hone-cli probe` on 2026-07-31 kept one logical/native session and returned `SP-SEED-ONE` then `SP-SEED-TWO`.
- Parsed outbound ACP requests showed:
  - prompt one: static system present, restored transcript absent, then-current Session/current-user context present;
  - prompt two: static system absent, restored transcript absent, then-current Session/current-user context present.
- Persisted session metadata ended with `acp_needs_sp_reseed=false`, proving the initial seed was consumed instead of remaining sticky.
- This was the intermediate lifecycle-only state; the minimal native-turn follow-up below subsequently removed the repeated Session block.

Minimal native-turn verification:

- `cargo test -p hone-channels --lib` passed 708 tests with one host-dependent OCR test ignored.
- `cargo check -p hone-channels --tests` passed.
- A real three-turn `hone-cli probe` used one Hone logical session and native Codex ID `019fb5e4-7796-7410-bfb8-e5ab2fad887c`, returning exactly `MINIMAL-TURN-ONE`, `MINIMAL-TURN-TWO`, and `MINIMAL-TURN-THREE`.
- Parsed outbound ACP prompts showed:
  - prompt one: `prompt_chars=20467`, `has_system=true`;
  - latest resumed prompt: `prompt_chars=103`, `has_system=false`, `has_wrapper=false`;
  - every prompt: current time and current user input present; restored transcript, `【Session 上下文】`, answer contract, entity-loop contract, and related-skill hint absent.
- Session metadata remained `codex_acp_session_mode=persistent_resume_v1`, kept the same native ID, and ended with `acp_needs_sp_reseed=false`.
- A real native-skill CLI probe created 16 Hone-managed symlinks, then Codex read the source `skills/market_analysis/SKILL.md` and returned exactly `原因本轮未完全核验`. Its ACP trace contained native file/command operations and no Hone MCP skill-loading call.
- The same probe's `session/new` payload exposed only live Hone MCP tools (`cron_job`, data/search/state/local-file tools, and administrator tools); `discover_skills`, `load_skill`, and `skill_tool` were absent.
- `codex app-server skills/list` with `forceReload=true` returned all 16 Hone skills and `errors=[]`.
- After merging current `origin/main`, `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` and `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed.
- `bun run test:web` passed all 309 tests; the Public Community Edge Worker passed typecheck and all 45 tests.
- `bash tests/regression/run_ci.sh` passed, including all 44 finance automation contracts. The Web API attachment test and CI contract now assert native image-path handling and reject the obsolete `skill_tool` wording.

## Risks / Follow-ups

- Native Codex and Hone session identities are now intentionally one-to-one only within a deterministic logical-session scope. Direct actors and channel/group scopes must never share one global Codex session.
- After the cold-start seed, absence of `### Restored Conversation Transcript ###` is expected. Diagnose continuity from the persisted mode/ID metadata, the second-turn `session/resume` request/result, and the single matching native Codex rollout.
- A deleted or corrupted marked native session currently fails the turn. Recovery requires a deliberate metadata repair/reset; automatic `session/new` fallback would silently fragment the Codex page again.
- The diagnosis-stage adapter/config blockers are resolved by the CLI upgrade and the pre-session process configuration change.
- Model identifiers exposed by `session/new` remain adapter-specific (`gpt-5.6-sol[xhigh]` in `1.1.7`). Future adapter upgrades should rerun initialize, event-stream, and two-turn continuity probes before raising the validated floor.
- Future Codex ACP upgrades must also revalidate the structured `contextCompaction` metadata shape. If that event changes, normal resumed turns must remain static-prompt-free while post-compaction recovery stays at-most-once per observed boundary.
- Do not reintroduce Harness-owned history/tool/compaction descriptions into trusted Codex Interactive user input. A new per-turn section needs evidence that it is both genuinely dynamic and unavailable through the native thread or seeded system instructions.
- Do not redirect Codex cwd to the repository to make skills visible. Keep actor isolation and project only enabled source skill directories into `.agents/skills`.
- The native projection depends on directory symlinks (explicitly supported by Codex). Windows hosts may still require symlink permission; the current supported production/validation hosts are macOS and Linux.

## Next Entry Point

- For continuity, prompt lifecycle, and native skills: `crates/hone-channels/src/runners/codex_acp.rs`, `crates/hone-channels/src/runners/acp_common/protocol.rs`, `crates/hone-channels/src/runners/acp_common/ingest.rs`, `crates/hone-channels/src/turn_builder.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/execution.rs`, and `crates/hone-channels/src/sandbox.rs`, especially the mode marker, `session/new`, `session/resume`, `acp_needs_sp_reseed`, `contextCompaction`, the native minimal-turn selector, and `sync_native_codex_skill_links`.
- For persisted mapping/history: the actor's logical session metadata under `data/sessions/`, plus its one native rollout under `~/.codex/sessions/`.
- For future upgrades: start from `docs/runbooks/hone-cli-install-and-start.md`, keep the Codex and adapter npm packages paired, and rerun the listed manual regressions plus a two-turn channel probe.
