# ACP 对齐的 Agent Runtime 全栈重构

- title: ACP 对齐的 Agent Runtime 全栈重构
- status: in_progress
- created_at: 2026-03-17
- updated_at: 2026-08-02
- owner: shared
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-channels/src/runners/acp_common/`
  - `crates/hone-channels/src/core/`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/outbound.rs`
  - `bins/hone-imessage/src/main.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/sandbox.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/attachments/vision.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-channels/src/agent_session/`
  - `memory/src/session.rs`
  - `crates/hone-channels/src/runners/gemini_acp.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-channels/src/runners/acp_common/version.rs`
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-web-api/src/routes/meta.rs`
  - `crates/hone-web-api/src/types.rs`
  - `scripts/deploy_source_runtime.sh`
  - `tests/regression/ci/test_source_runtime_deploy_contract.sh`
  - `docs/runbooks/source-web-startup.md`
  - `config.example.yaml`
  - `config.yaml`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/decisions.md`
  - `docs/bugs/opencode_acp_prompt_timeout.md`
  - `docs/handoffs/2026-04-13-acp-prompt-dual-timeout.md`
  - `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`
  - `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
  - `docs/handoffs/2026-07-29-codex-acp-discord-runtime-recovery.md`
  - `docs/handoffs/2026-08-02-acp-tool-status-projection.md`

## Goal

Finish converging the agent runtime on ACP semantics so channel entrypoints, runners, and frontend streaming behave through one contract.

## Scope

- ACP runners already bridge into Hone MCP.
- `gemini_acp` 的初始化与 usage 信号链路已被完整复盘，但该 runner 现已禁用并输出迁移提示，不再作为收敛目标保留在默认运行路径里。
- Runner timeout config is being converged to two top-level knobs under `agent`: `step_timeout_seconds` and `overall_timeout_seconds`.
- ACP `session/prompt` now uses `idle=step_timeout_seconds` and `overall=overall_timeout_seconds`. Codex continuation no longer uses `session/load`; a marked persistent session must resume successfully and may not silently fall back to `session/new`.
- `codex_acp` transcript is being reworked so intermediate model output is preserved in restorable transcript segments without flattening everything into one assistant blob.
- Common code now only carries generic `message.metadata`; runner-specific transcript fields must stay in each runner / channel implementation instead of being centralized under a shared ACP schema.
- `codex_acp` and `opencode_acp` persist the same normalized local transcript model for audit/restore: top-level history uses `user/assistant` turns, while tool calls/results and progress/final answer live inside assistant `content[]` parts. OpenCode compiles that local history into each fresh ACP session; Codex keeps it out of native prompts and relies on its persistent upstream thread.
- Session storage itself now writes the normalized model directly as `version=4` with `content[] + status` instead of the old flat string `content`; legacy JSON still deserializes for compatibility, but new writes use the breaking on-disk layout.
- `codex_acp` now patches execute-completion `tool_call_update.rawOutput` into persisted `tool_result` parts, so codex execute turns are recorded as `progress -> tool_call -> tool_result -> final` in the same assistant turn instead of falling back to a partial tool-call-only record.
- `codex_cli` reasoning runs are now explicitly covered by the same normalized persistence contract: runner tail messages are normalized into `progress/tool_call/tool_result/final` assistant content parts before storage.
- Historical note: the former `multi-agent` transcript merge work was superseded when that runner was removed on 2026-07-13; it is no longer an active runtime branch.
- Native-persistent Codex treats its own session/compact logic as the source of truth, so Hone skips its SessionCompactor and compact-summary injection on that effective runner. Fresh-session OpenCode remains a Hone-managed replay/compaction path.
- `acp_common` detects compact lifecycle signals and keeps them out of ordinary visible text. Historical Codex reseed metadata is superseded: native Codex compact is now telemetry only, while OpenCode keeps its separately modelled fresh-session replay behavior.
- `gemini_acp` is no longer offered as an active runtime path: factory creation now errors with a migration hint because Gemini ACP does not emit reliable `usage_update` signals and is unsafe for Hone's long-session compact detection model.
- `codex_acp` passes `agent.codex_acp.model` / `variant` to Codex CLI as `CODEX_CONFIG` `model` / `model_reasoning_effort` values before ACP startup and no longer calls adapter-specific `session/set_model`.
- Admin actors routed to `codex_acp` now use the adapter's native image capability directly instead of synchronously compiling and running the optional Apple Vision OCR helper before the first Discord placeholder. Strict public fallback actors retain local OCR pre-extraction. Timed-out OCR helper and Swift compiler children use kill-on-drop so they cannot remain behind as detached work.
- Remaining work is still needed around runner contract coverage and end-to-end runtime behavior alignment.
- 2026-06-24: Fixed ACP child-process lifecycle leaks where `codex_acp` / `opencode_acp` error or timeout paths could leave their stdio `hone-mcp` grandchildren running after the turn exits. ACP CLI children now run in their own process group and are cleaned up through a shared guard that terminates the group before returning from the runner.
- 2026-07-13 follow-up: the simplification task removed the in-process `function_calling` agent and sequential `multi-agent` runner, moved current defaults to Codex ACP with GPT-5.6 Sol / xhigh, and reduced duplicated prompt layers. This plan remains the parent ACP architecture record.
- 2026-07-13 completion: the simplification follow-up is done and archived at `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`; verification and migration details are in `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`.
- 2026-07-29 follow-up: a real Discord admin turn proved that `codex-acp 1.1.7` creates the intended `gpt-5.6-sol[xhigh]` session but rejected Hone's subsequent bare `gpt-5.6-sol` legacy `session/set_model` request before `session/prompt`. The selector is now normalized and covered. A second real image turn then exposed a separate 95.5-second pre-placeholder delay: two optional Apple Vision OCR helper compilation timeouts ran before ACP even started. Codex ACP admin image turns now bypass that redundant pre-extraction and use native image reads; Kimi/Hone session compaction remains unchanged and out of scope.
- 2026-07-30 live continuity diagnosis: Codex ACP still intentionally creates one fresh upstream ACP session per Hone turn. A controlled two-turn Discord probe kept one deterministic Hone session, created two distinct Codex rollout/session IDs, injected the first `user/assistant` pair into the second turn's `### Restored Conversation Transcript ###`, and recovered the exact sentinel. Native Codex session views therefore show separate one-turn rollouts even though semantic conversation continuity is preserved by Hone's local transcript restore. Evidence and local adapter compatibility findings are recorded in `docs/handoffs/2026-07-30-codex-acp-session-continuity-diagnosis.md`.
- 2026-07-30 compatibility follow-up: replaced the unrelated Homebrew `zed-industries/codex-acp 0.11.1` with npm `@agentclientprotocol/codex-acp 1.1.7`, installed npm `@openai/codex 0.146.0`, raised Hone's validated floors, and moved Codex model/reasoning selection into process `-c` overrides before `session/new`. This avoids adapter-specific `session/set_model` model-id formatting and makes version probes use the same `CODEX_PATH` as real turns.
- 2026-07-30 persistent-session follow-up: the preceding fresh-session diagnosis is retained as historical evidence of the old behavior, but its contract is superseded by `D-2026-07-30-01`. Hone now creates one native Codex session for each deterministic Hone logical session, stores a mode-marked native ID, resumes it on every later turn with `session/resume`, and seeds local transcript only once when entering persistent mode. Codex owns history and automatic compaction; a resume failure fails closed instead of silently forking the Codex page.
- 2026-07-31 static-prompt lifecycle follow-up: Codex persistent sessions now send the complete static Hone system prompt only in the first native prompt. Ordinary `session/resume` turns send the freshly assembled runtime input without repeating the static prompt. Codex ACP 1.1.7 `_meta.contextCompaction=true` updates are treated as internal harness lifecycle events and set one pending reseed; the next successful prompt includes the static prompt once and clears the flag. The legacy compact text and usage-drop paths remain fallback detection. OpenCode behavior is unchanged.
- 2026-07-31 minimal native-turn follow-up: trusted Codex ACP Interactive turns now treat the native harness as owner of retained history, tool/MCP lifecycle, and compaction. Their user payload contains only current Beijing time plus normalized current user content, including attachment/image material or an explicit slash-skill task. Hone no longer repeats Session IDs/history, receive-routing metadata, related-skill hints, entity-loop instructions, or final-answer contracts on those turns. OpenCode, scheduled/heartbeat tasks, and strict actor fallback remain unchanged.
- 2026-07-31 native-skill follow-up: trusted persistent Codex ACP workspaces expose each enabled Hone system/custom skill through an individual symlink under `<actor sandbox>/.agents/skills/`. Codex owns skill discovery and progressive `SKILL.md` loading; Hone no longer repeats the MCP `SkillTool` loading contract or exposes skill-loading MCP schemas on that path. MCP remains the transport for live Hone data/action tools, while legacy runners retain the existing skill bridge.
- 2026-08-01 native-turn contract follow-up (done; parent plan remains active): replaced the seed/reseed user-message convention with an explicit runner conversation strategy. Persistent Codex sessions receive Hone instructions through Codex's native `developer_instructions` configuration and every ACP `session/prompt` contains only the canonical current user turn. Native compaction never causes Hone to replay system instructions, local transcript, historical user/assistant messages, tool calls, or tool results. The persistent-session mode and instruction fingerprint form a generation boundary so pre-contract or instruction-mismatched native sessions rotate deliberately instead of being resumed as if clean. OpenCode remains a fresh-session Hone-replay adapter until its own resume/event contract is independently proven.
- 2026-08-02 MCP startup and execute-status follow-up (done; parent plan remains active): `codex-acp 1.1.7` can report a cancelled MCP startup watcher as a synthetic terminal `tool_call` with `toolCallId=mcp_startup.<server>` and `status=failed`. Hone now retains that adapter lifecycle telemetry in the raw ACP log while excluding it from visible progress, pending/restored tool state, and business tool counts. Structured MCP calls and actual shell execution are distinguished from `rawInput`; MCP calls use bounded tool/argument summaries, shell calls use safe categories without arguments, secrets, or full paths, and completion events inherit the same start summary when Codex omits start metadata. Discord, Telegram, and Feishu direct/group projections share the Full/Compact rendering contract, while iMessage forwards safe tool-start state to its console pending view.
- 2026-08-02 version-aware dialect and source-deployment follow-up (local acceptance done; GCE rollout paused): keep the existing Codex/OpenCode conversation ownership split, but replace static observed-version labels with one shared runtime adapter profile selected from actual CLI/adapter probes. Known profiles bind adapter identity, detected version, compatibility status, stream dialect, and fixture provenance; unknown newer versions run only through an explicit conservative compatibility policy with visible telemetry, while older/structurally incompatible versions fail before a turn. Runtime metadata exposes sanitized build provenance and selected profiles. Direct source deployment is one repository-owned state machine with active-turn drain, exact PID/lock release, startup/readiness separation, all-service rollback, immutable commit provenance, and no credential output. A real preflight found the existing `com.honeclaw.source.runtime` parent/child topology, so the state machine now explicitly migrates that supervisor to persistent revision-bound Web/Discord LaunchAgents, rejects unknown port owners, and restores the old plist/job on failure. Exact local implementation revision `b8e183124e21c9d54e6a4449db88f711d16279d3` is deployed and accepted; GCE rollout is deliberately paused by user direction.

### 2026-08-02 version-aware dialect and deployment acceptance checklist

- [x] Preserve the existing `NativePersistent` Codex and `EphemeralCompiledPrompt` OpenCode conversation inputs; no version refactor may reintroduce system/history/tool replay into Codex prompts.
- [x] Parse Codex CLI and live Codex/OpenCode initialize versions through one shared typed model; retain a bounded normalized semantic version for diagnostics.
- [x] Resolve an adapter profile from actual identity/version, not from runner name alone. The profile includes compatibility status and the exact stream dialect/fixture baseline selected.
- [x] Accept the currently observed Codex `0.146.0` + codex-acp `1.1.7` and OpenCode `1.18.11` profiles, reject versions below validated floors or in unknown majors, and emit a bounded warning for same-major newer versions without claiming they are the observed baseline.
- [x] Require the live initialize identity to match the captured `@agentclientprotocol/codex-acp` or `opencode` value exactly, reject mismatched/missing/unobserved shorthand names before session creation, and apply the unknown-major fail-closed policy to the Codex CLI companion as well as both adapters.
- [x] Keep external protocol fixtures adapter- and version-labelled, with capture date and source boundary. Assertions target protocol roles/events and safe normalized output rather than private implementation details or byte-identical cross-runner rendering.
- [x] Preserve safely mappable answer, reasoning, tool, progress, usage, and terminal detail through shared `RunEvent` projection for Full/Compact channel renderers; unknown fields remain bounded diagnostics rather than user-visible raw JSON.
- [x] Expose application version, Git SHA, build timestamp/profile, detected runner tool versions, selected dialect baseline, and compatibility status through `/api/meta` without paths, credentials, prompts, or user data.
- [x] Expose a closed `build.source` provenance kind and binary hash in `/api/meta`; emit the same bounded build facts at startup and the actual adapter/companion versions, compatibility, and dialect in selection logs.
- [x] Implement a non-interactive direct-source deployment command with explicit `preflight -> drain -> stop -> wait_pid_and_lock -> start -> startup -> ready -> channel_login -> verify` states and one all-service rollback path.
- [x] Deployment refuses a dirty/unpushed or unexpected revision unless explicitly overridden, waits for active chats to reach zero, proves old PIDs exited and locks released, uses configurable startup/readiness deadlines above the observed startup baseline, and restores every previously running service on failure.
- [x] CI-safe deployment regression uses fake launch/service/HTTP boundaries and proves direct-job and legacy-supervisor success, orphaned legacy-child TERM handling, unknown-port-owner refusal, PID/lock waiting, persistent plist installation, crashed-but-loaded jobs, timeout, partial-start failure, and complete rollback/bootstrap without requiring launchd, credentials, or live ports.
- [x] Bootstrap the exact persistent plist used after login, keep its runner `PATH` independent of the invoking Codex turn, reject `.codex/tmp` / `.cache/codex-runtimes` entries before shutdown, and prove the stable command still executes after the external temporary path disappears.
- [x] Run focused Rust tests, complete workspace gates, Web/Edge tests, CI-safe regressions, real local Codex version/initialize probes, and a manual OpenCode probe only when a real binary is available; absence of OpenCode must be reported rather than simulated as live proof.
- [x] Complete the local source-runtime canary from exact implementation commit `b8e183124e21c9d54e6a4449db88f711d16279d3`: zero active chats, matching `/api/meta` provenance, all four listeners healthy, fresh Discord login, stable launchd jobs, real persistent-`PATH` Codex ACP sentinel, and a successful post-deploy Discord turn. GCE rollout remains paused and is not counted as local acceptance.

### 2026-08-02 MCP startup status acceptance checklist

- [x] Recognize adapter-generated `mcp_startup.<server>` lifecycle calls without relying only on the display title.
- [x] Keep synthetic startup failures out of user-visible `ToolStatus`, pending tool state, restored tool transcript, and tool-call counts.
- [x] Preserve ordinary MCP business tool start/completion events unchanged.
- [x] Add a regression fixture matching the observed Codex ACP 1.1.7 cancelled-startup payload.
- [x] Render observed `rawInput.server/tool/arguments` MCP calls as bounded business-tool summaries instead of `本地命令`.
- [x] Render real shell calls with a safe executable/category summary while proving command arguments and secret-like values stay hidden.
- [x] Keep start/done summaries stable when Codex completion events omit `kind` and `rawInput`.
- [x] Apply the Full/Compact projection to Discord, Telegram, and Feishu, and forward the safe start summary to iMessage console state.
- [x] Run focused and full tests, changed-file formatting, diff validation, then deploy through the existing source runtime and verify fresh MCP/shell turns.

### 2026-08-01 acceptance checklist

- [x] Define runner conversation strategies so native persistence, structured replay, and single-prompt compilation are not inferred from one ambiguous `manages_own_context` boolean.
- [x] Provision Codex developer instructions through the adapter-supported configuration boundary and bind the native session to an instruction fingerprint.
- [x] Make every Codex `session/prompt` current-turn-only on new, resumed, and post-compaction turns; remove local transcript/system reseeding from that transport.
- [x] Rotate legacy `persistent_resume_v1` and instruction-mismatched session metadata to a new native generation without deleting the old Codex task.
- [x] Validate observable JSON-RPC requests against an ACP boundary double, including a compact notification between turns; avoid tests coupled only to private prompt-builder implementation.
- [x] Model Codex ACP and OpenCode ACP as versioned stream dialects. Preserve every safely available visible/progress/reasoning/tool/usage lifecycle detail without requiring byte-identical cross-runner channel output, and label fixtures with the adapter versions they were observed from.
- [x] Probe the installed real Codex ACP and OpenCode ACP entrypoints without changing provider authentication or exposing credentials.
- [x] Run the repository gates, synchronize ADR/decision/invariant/repo-map/handoff/archive evidence, and commit only the reviewed task files.

## Validation

- 2026-08-02 version-aware dialect and source-deployment follow-up:
  - `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed.
  - `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed; the only warnings were the existing Web API dead-code warning and upstream future-compatibility notice.
  - `bun run test:web` passed `334/334`; Public Community Edge typecheck and `45/45` tests passed.
  - `bash tests/regression/run_ci.sh` passed, including the new source-runtime deployment contract for success, dirty/unpushed refusal, crashed-but-loaded jobs, Web/Discord readiness failures, and all-service rollback.
  - Post-review `cargo test -p hone-channels --lib` passed `715`, with one existing host-dependent OCR test ignored; `cargo test -p hone-core build_info` passed.
  - The installed Codex CLI `0.146.0` and codex-acp `1.1.7` completed a real initialize/session exchange. The installed OpenCode `1.18.11` completed initialize, `session/set_model`, one read-only Hone MCP call, reasoning/answer/usage streaming, and `end_turn` with an explicit free probe model.
  - OpenCode's configured default OpenAI OAuth independently returned `401`; this remains an external provider-auth problem and is not counted as default-model success.
  - Exact implementation commit `b8e183124e21c9d54e6a4449db88f711d16279d3` was pushed, fast-forwarded into `main`, and deployed through the repository state machine. Both launchd jobs remained at `runs=1` with stable PIDs; historical lock errors did not grow; ports `8077`, `8088`, `3000`, and `3001` remained bound; `/api/meta` reported the exact revision; Discord logged in; and active chats returned to `0`.
  - Under the exact persistent Discord plist `PATH`, `codex --version` returned `0.146.0`, `codex-acp --version` returned `1.1.7`, and a real `hone-cli` ACP probe returned exactly `LOCAL_PERSISTENT_PATH_OK` with `tool_calls=0`. The first subsequent real Discord turn resumed its persistent ACP session, selected the validated `codex-acp 1.1.7` dialect, completed successfully, and sent one reply without the previous command-not-found failure.
  - GCE rollout was explicitly paused after local recovery and is not part of this local acceptance result.
  - Completion audit follow-up added live initialize identity matching, bounded invalid-version diagnostics, Codex CLI unknown-major rejection, closed build-source provenance, startup build logging, and companion version/status logging. The complete workspace check/test gate passed (`hone-channels` `716 passed`, one host-dependent ignore), Web remained `334/334`, Edge remained `45/45`, and the complete CI-safe regression suite passed. An exact pushed-revision local redeploy is the remaining acceptance step for these observability additions.
  - The first completion-audit canary failed closed before session creation because the fixture used an invented `codex-acp` shorthand while the real `1.1.7` initialize boundary reports `@agentclientprotocol/codex-acp`. The captured fixture, protocol double, selector unit contract, decision, and handoff now use the exact observed package identity and explicitly reject the stale alias; this external-boundary correction must pass the focused test and a new exact-revision canary before local closure.

- 2026-08-02 MCP startup and cross-channel tool-status follow-up:
  - Exact Codex ACP 1.1.7 cancelled-startup, structured MCP, argv/string shell, secret-redaction, and start-to-completion metadata-loss regressions passed.
  - `cargo test -p hone-channels --lib` (`710 passed`, `1` host-dependent OCR test ignored).
  - `cargo test -p hone-imessage` (`3 passed`).
  - `cargo check -p hone-discord -p hone-telegram -p hone-feishu -p hone-imessage` passed; the only emitted future-compatibility warning is from upstream `proc-macro-error2`.
  - Exact changed Rust files were formatted with `rustfmt --edition 2024 --config skip_children=true`; `git diff --check` passed.
  - A real isolated Discord Codex ACP MCP probe suppressed the observed `mcp_startup.hone` lifecycle event and emitted `hone/web_search query="..."` for the actual business call.
  - A final isolated Discord shell probe emitted the same safe `读取本地内容（pwd）` label for start and completion, returned `LOCAL_PROBE_OK`, and made one real `pwd` tool call without exposing its sandbox path.
  - Telegram's current source config has no non-empty administrator ID, so a live Telegram Codex ACP turn is not routable without changing channel authorization. No authorization was changed; shared Full/Compact regressions and all four channel-bin compile checks provide the non-credentialed cross-channel proof.
  - The source LaunchAgent restarted at zero active chats with repository Cargo provenance. PID `83979` supervises source-built `hone-console-page` and `hone-discord`; Discord re-authenticated, ports `8077`, `8088`, `3000`, and `3001` returned HTTP `200`, and active chats returned to `0`.

- 2026-08-01 native-turn v2 follow-up:
  - `cargo test -p hone-channels codex_acp_1_1_7_boundary_keeps_every_prompt_current_turn_only -- --nocapture`
  - `cargo test -p hone-channels opencode_1_18_11_stream_preserves_available_reasoning_answer_and_usage -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - Real OpenCode `1.18.11` ACP `initialize -> session/new -> session/prompt` returned `OPENCODE_ACP_OK`; the observed stream contained object-shaped thought/message chunks plus detailed usage.
  - Real source-built `hone-cli` ran two Codex ACP turns against native ID `019fbe23-f728-7a72-a2fb-6ed9260d5e31`; the second process used `session/resume`, both sentinels returned exactly, and rollout inspection found two Hone current-turn user payloads with no system/history/tool replay or cross-turn marker.
  - The external fake codex-acp adapter injected `_meta.contextCompaction=true` between turns and proved that the next prompt remained exact current-turn-only with no reseed metadata.
  - `bash scripts/ci/check_fmt_changed.sh` (no staged Rust paths at this pre-commit point; exact changed Rust files were formatted directly with rustfmt)
  - `git diff --check`
  - `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - `bun run test:web` (`334 passed` after installing the frozen root dependencies in the new worktree)
  - `cd workers/public-community-edge && bun run typecheck && bun run test` (`45 passed` after installing that worker's frozen dependencies)
  - `bash tests/regression/run_ci.sh`
  - Final post-review `cargo check -p hone-channels --tests` and `cargo test -p hone-channels --lib` (`704 passed`, `1` host-dependent OCR test ignored)

- 2026-04-13:
  - `cargo test -p hone-core test_agent_runner_timeouts_default_to_step_plus_overall test_agent_runner_timeout_override_preserves_explicit_values`
  - `cargo test -p hone-channels runners::tests`
  - `cargo check -p hone-channels`
- 2026-04-15:
  - `cargo run -q -p hone-cli -- --config config.yaml probe --channel telegram --user-id acp_probe_user --group --scope 'chat:-1009000000000' --query '详细分析一下FLNC现在的价位以及潜力'`
  - `cargo run -q -p hone-cli -- --config config.yaml probe --channel telegram --user-id acp_probe_fresh --group --scope 'chat:acp-probe-fresh-20260415' --query '详细分析一下FLNC现在的价位以及潜力'`
  - `cargo test -p hone-channels --lib`
  - `cargo test -p hone-channels --lib -- --test-threads=1`
  - `cargo test -p hone-memory --lib`
  - `cargo check --workspace --all-targets --exclude hone-desktop`
  - `cargo run -q -p hone-cli -- --config config.yaml probe --channel telegram --user-id acp_probe_short2 --group --scope 'chat:acp-probe-short2-20260415' --query '先告诉我你会检查本地 版本，然后执行 --version，最后只输出一行 VERSION=<结果>。'`
  - `cargo run -q -p hone-cli -- --config config.yaml probe --channel telegram --user-id acp_storage_probe2 --group --scope 'chat:acp-storage-20260415-215524' --show-events true --query '先告诉我你会检查本地 版本，然后执行 --version，最后只输出一行 VERSION=<结果>。'`
  - `cargo run -q -p hone-cli -- --config data/runtime/config_runtime_opencode.yaml probe --channel telegram --user-id acp_storage_probe2 --group --scope 'chat:acp-storage-20260415-215524' --show-events true --query '上一轮你拿到的 VERSION 是什么？不要重新执行命令，不要调用工具，只输出一行 SAME=<结果>。'`
  - verified persisted session JSON: `data/runtime/data/sessions/Session_telegram__group__chat_3aacp-storage-20260415-215524.json`
  - bare `codex-acp` JSON-RPC probe with `initialize/session/new/session/prompt` and explicit `mcpServers: []`
- 2026-04-23:
  - `cargo test -p hone-channels --lib`
  - `cargo test -p hone-web-api --lib`
  - `bun run test:web`
  - `cargo check --workspace --all-targets --exclude hone-desktop`
  - `cargo test --workspace --all-targets --exclude hone-desktop`
  - `bash tests/regression/run_ci.sh`
- 2026-04-24:
  - `cargo test -p hone-channels configured_codex`
  - `cargo test -p hone-channels codex_acp_effective_args`
- 2026-06-24:
  - `rustfmt --edition 2024 --config skip_children=true crates/hone-channels/src/runners/acp_common/process.rs crates/hone-channels/src/runners/acp_common/mod.rs crates/hone-channels/src/runners/codex_acp.rs crates/hone-channels/src/runners/opencode_acp.rs`
  - `cargo test -p hone-channels acp_child_guard_terminates_grandchild_process_group -- --nocapture`
  - `cargo test -p hone-channels codex_acp -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `pgrep -fl 'hone-mcp' || true` returned no `hone-mcp` processes after cleanup and tests
  - Note: repository-wide `cargo fmt --check` still reports pre-existing formatting diffs in `crates/hone-channels/src/runtime.rs` and `crates/hone-core/src/cloud_runtime.rs`; those files were not changed by this task.
  - Note: one unrelated Clash Verge `<defunct>` process remains because killing its parent `verge-mihomo` returned `operation not permitted`; it is outside the Hone process tree.
- 2026-07-29:
  - live Discord evidence: ACP `initialize` and `session/new` succeeded with adapter `1.1.7` and current model `gpt-5.6-sol[xhigh]`
  - failing request: legacy `session/set_model` sent `gpt-5.6-sol` and returned `Unsupported format of modelId: gpt-5.6-sol. Expected: modelId[effort].`
  - fixed request: `session/set_model` sent `gpt-5.6-sol[xhigh]`, returned success, and was followed by `session/prompt`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/attachments/ingest.rs crates/hone-channels/src/attachments/vision.rs crates/hone-channels/src/runners/codex_acp.rs crates/hone-channels/src/runners/tests.rs`
  - `cargo test -p hone-channels attachments::ingest::tests --lib` (`25 passed`)
  - `cargo test -p hone-channels configured_codex_model_id --lib` (`5 passed`)
  - `cargo test -p hone-channels codex_acp --lib` (`5 passed`, including the native-image routing regression)
  - `cargo test -p hone-channels --lib` (`695 passed`, `1 ignored`)
  - `cargo check -p hone-channels --tests`
  - `cargo build --bin hone-cli --bin hone-console-page --bin hone-discord --bin hone-mcp`
  - admin-scoped no-side-effect ACP probe returned `ACP_MODEL_OK` with `tool_calls=0`; ACP event trace showed successful `initialize -> session/new -> session/set_model(gpt-5.6-sol[xhigh]) -> session/prompt -> final`
  - real Discord image turn: Discord snowflake timestamp `2026-07-29 00:36:37.913 +08:00`; pre-fix placeholder creation at `00:38:13.282` proved a 95.4-second attachment preprocessing delay; ACP then completed successfully in `296318ms` with native reads of both images, 19 tool calls, and a 671-character final reply sent at `00:43:11`
  - source launchd service gracefully restarted after the attachment fix (`runs=6`, PID `94657` at validation); Discord re-logged as `Hone-TEST`, ports `8077` and `8088` were listening, stderr remained empty, and no `swiftc` / `hone-image-ocr` process remained
  - first post-deployment Discord follow-up was sent at `00:46:28.111`; the bot placeholder was created at `00:46:29.232` (`1.121s` later), ACP completed `session/set_model(gpt-5.6-sol[xhigh]) -> session/prompt`, and the turn finished successfully in `112473ms` after 53 portfolio/data/search tool calls with a 680-character answer
  - Discord API readback confirmed the same placeholder was edited to the final answer at `00:48:22.320` and no processing-placeholder text remained
- 2026-07-30:
  - Real `codex_acp` two-turn probe through `channel=discord`: one Hone session, two fresh Codex ACP sessions, exact prior-turn sentinel recovered on turn two.
  - ACP JSONL inspection confirmed the second `session/prompt` contained the prior `user/assistant` pair in `### Restored Conversation Transcript ###`.
  - Native Codex rollout inspection confirmed each upstream ACP session contains one native `user_message -> agent_message` turn, matching the intentional fresh-session contract.
  - `cargo test -p hone-channels codex_acp_does_not_reuse_remote_session_metadata -- --nocapture`
  - `cargo test -p hone-channels codex_prompt_text_includes_restored_transcript_when_session_is_recreated -- --nocapture`
  - `cargo test -p hone-discord group_session_id_is_channel_scoped -- --nocapture`
  - No business code changed. Temporary `config.yaml` overrides used for the live probe were restored exactly.
- 2026-07-30 compatibility follow-up:
  - `codex --version` reported `codex-cli 0.146.0`; ACP initialize reported adapter `1.1.7`.
  - `cargo test -p hone-channels codex_acp -- --nocapture`
  - `cargo test -p hone-channels configured_codex -- --nocapture`
  - `cargo test -p hone-channels codex_version_matrix -- --nocapture`
  - `cargo test -p hone-core agent_runner_kind_keeps_wire_values_and_probe_mapping -- --nocapture`
  - `cargo test -p hone-cli onboard_runner_kind_config_values_and_probe_requirements -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `cargo check -p hone-cli --tests`
  - `bash tests/regression/manual/test_codex_acp_initialize.sh`
  - `bash tests/regression/manual/test_codex_acp_event_stream.sh`
  - Repeated the two-turn Discord probe with the installed binaries. Both turns used `gpt-5.6-sol[xhigh]`, no `session/set_model` call occurred, and turn two returned `UPGRADED-DISCORD-CTX-20260730`.
  - Temporary Discord admin configuration was restored exactly after the probe.
- 2026-07-30 persistent-session follow-up:
  - Raw cross-process ACP probe resumed an existing Codex ID with `session/resume`; the adapter emitted no historical `session/update` between request and response.
  - Real Discord two-turn probe kept one Hone logical session and one native Codex ID `019fb3c2-f2f7-7140-8140-7520409d79be`.
  - Codex Desktop's task index listed that same native ID as one local task.
  - ACP evidence: one `session/new`, one `session/resume`, two `session/prompt`; neither current prompt contained a restored-transcript block.
  - Native Codex rollout evidence: one rollout file, two `user_message`, two `agent_message`, two completed tasks; turn two returned `ONE-CODEX-SESSION-20260730`.
  - `cargo test -p hone-channels codex_acp_reuses_only_persistent_remote_session_metadata -- --nocapture`
  - `cargo test -p hone-channels codex_prompt_text_includes_restored_transcript_when_persistent_session_is_initialized -- --nocapture`
  - `cargo test -p hone-channels self_managed_runner_context_overflow_is_not_locally_compacted_or_retried -- --nocapture`
  - `cargo test -p hone-channels --lib` passed 699 tests with one host-dependent OCR test ignored after rebasing onto current `origin/main`.
  - `cargo check -p hone-channels --tests` and `cargo check -p hone-cli --tests`
  - `bash tests/regression/manual/test_codex_acp_initialize.sh`
  - `bash tests/regression/manual/test_codex_acp_event_stream.sh`
  - Temporary Discord administrator configuration was restored exactly after the probe.
- 2026-07-31 static-prompt lifecycle follow-up:
  - `cargo test -p hone-channels runners::acp_common::tests --lib -- --nocapture`
  - `cargo test -p hone-channels runners::tests::codex --lib -- --nocapture`
  - `cargo test -p hone-channels --lib` (`705 passed`, `1` host-dependent OCR test ignored)
  - `cargo check -p hone-channels --tests`
  - `bash scripts/ci/check_fmt_changed.sh`
  - `bash tests/regression/manual/test_codex_acp_initialize.sh`
  - `bash tests/regression/manual/test_codex_acp_event_stream.sh`
  - At this intermediate subphase, a real CLI two-turn probe used one Hone logical session and one native Codex ID. The ACP log showed prompt one with `has_system=true`, prompt two with `has_system=false`, no restored transcript in either prompt, and then-current Session/current-user context in both. Both turns returned their exact requested sentinel, and session metadata ended with `acp_needs_sp_reseed=false`; the following minimal-turn subphase removed the repeated Session block.
- 2026-07-31 minimal native-turn follow-up:
  - `cargo test -p hone-channels --lib` (`708 passed`, `1` host-dependent OCR test ignored)
  - `cargo check -p hone-channels --tests`
  - Focused regressions covered the minimal payload shape, trusted Codex vs strict fallback routing, and the Codex-specific boundary versus OpenCode.
  - A real three-turn CLI probe kept native ID `019fb5e4-7796-7410-bfb8-e5ab2fad887c` and returned `MINIMAL-TURN-ONE` / `MINIMAL-TURN-TWO` / `MINIMAL-TURN-THREE`. Parsed ACP requests showed the first system seed and no resumed system seed; all prompts omitted restored transcript, Session context, answer/entity contracts, and related-skill hints. The latest resumed prompt was a raw 103-character payload with no generic wrapper and contained only `【当前时间】` plus `【本轮用户输入】`.
- 2026-07-31 native-skill follow-up:
  - Focused symlink, native prompt, attachment, and skill-runtime tests passed.
  - A real `hone-cli probe` created 16 managed links and made Codex natively read `skills/market_analysis/SKILL.md`; it returned exactly `原因本轮未完全核验` without a Hone MCP skill-loading call.
  - The outbound `session/new` MCP allowlist omitted `discover_skills`, `load_skill`, and `skill_tool`.
  - `codex app-server skills/list` with `forceReload=true` returned all 16 Hone skills with `errors=[]`.
  - The bundled skill-creator validator could not start because its local Python environment lacks `PyYAML`; Codex's own parser/listing and the real ACP activation provide the runtime validation for this host.
  - After merging the latest `origin/main`, the full default gates passed: changed-file formatting, workspace check/test excluding Apple clients, all 309 Web tests, Edge Worker typecheck plus 45 tests, and the complete CI-safe regression suite including all 44 finance automation contracts.
  - The post-merge Web API and finance-contract assertions were updated to require native image-path handling and the absence of the obsolete `skill_tool` instruction.

## Documentation Sync

- Keep this file and `docs/adr/0002-agent-runtime-acp-refactor.md` aligned.
- If the runtime contract changes materially, update `docs/decisions.md`.
- Runner timeout semantics are now configured only through `agent.step_timeout_seconds` and `agent.overall_timeout_seconds`; keep `config.yaml` / `config.example.yaml` and the timeout analysis docs in sync when adjusting those values again.
- If ACP transcript persistence semantics change, update the ACP runtime ADR or `docs/decisions.md` to reflect the new transcript contract.
- Compact leak handling and Gemini ACP disablement must stay aligned with `docs/bugs/session_compact_summary_report_hallucination.md` and `config.example.yaml`.
- If runner-specific transcript metadata is added later, keep it under the owning runner/channel namespace and avoid introducing a shared ACP-wide event schema in `memory` or other common storage helpers.
- If the normalized history model expands again, preserve runner interchangeability: prompt restoration should keep consuming the shared `user/assistant` model rather than any single runner’s raw event stream.
- If ACP process lifecycle semantics change, keep `docs/repo-map.md` aligned because it documents the MCP bridge and runner process boundaries.

## Risks / Open Questions

- The remaining work spans runners, channel ingress, and Web SSE semantics.
- Partial convergence is risky if one runner path silently diverges from ACP behavior.
- `opencode_acp` and `codex_acp` now consume the same normalized history for prompt restore, but their raw ACP event shapes still differ; raw-session persistence and replay must remain runner-owned.
- Runner-specific transcript metadata can still grow session files; any future expansion should be validated against real session size and restore cost.
- ACP compact detection uses Codex ACP's structured `_meta.contextCompaction=true` signal first, retains legacy Codex compact text as a fallback, and still uses usage-drop/summary-boundary heuristics for OpenCode; future adapter upgrades must revalidate those event shapes.
- ACP CLI cleanup must account for grandchildren such as `hone-mcp`; killing only the direct ACP process can leave orphaned MCP servers when the upstream CLI does not close them itself.
- Codex ACP's model list and session/model-id representation remain adapter-version-specific. Hone now avoids that dependency by supplying the base model and reasoning effort through the adapter-supported `CODEX_CONFIG` before `session/new`; future adapter upgrades must rerun initialize, event-stream, and two-turn continuity probes.
- A persistent native Codex session is scoped to one deterministic Hone logical session, not globally to the whole Hone installation. Any future session-key change must preserve actor/channel isolation.
- Deleting or invalidating a marked native Codex session currently makes resume fail explicitly. Recovery must remain an operator-visible mapping repair; do not add an automatic `session/new` fallback that silently fragments page history.
