# ACP Native-Turn Contract And Versioned Stream Dialects

- title: ACP Native-Turn Contract And Versioned Stream Dialects
- status: done
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: shared
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `crates/hone-channels/src/mcp_bridge.rs`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/runbooks/hone-cli-install-and-start.md`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/decisions.md#d-2026-08-01-01-separate-conversation-ownership-from-acp-stream-dialects`
  - `docs/invariants.md`
  - `docs/runbooks/hone-cli-install-and-start.md`
  - `docs/runbooks/opencode-setup.md`
- related_prs: none

## Summary

Codex ACP now has one enforceable role boundary: Hone instructions enter Codex as native developer instructions, while every ACP user prompt contains only the current turn. Native compaction never triggers a seed/reseed. Other runners retain explicitly named replay/compiled strategies, and Codex/OpenCode streaming is modelled as separate versioned adapter dialects.

## What Changed

- Replaced ambiguous context booleans with `AgentConversationStrategy` and a matching typed `RunnerConversationInput` prepared after actual runner selection.
- Kept conversation ownership, versioned stream dialect, and `NativeSkillProjection::CodexWorkspace` as independent typed capabilities, preventing future native runners from inheriting Codex-specific skill/MCP behavior.
- Changed Codex process setup to the adapter-supported `CODEX_CONFIG`, including `developer_instructions`, model/effort, safety settings, and extra overrides.
- Added the `native_turn_v2` session generation plus an instruction fingerprint. Exact matches resume; legacy v1 or mismatched instructions rotate to a new native generation without deleting the old Codex task.
- Deleted Codex system-prompt seed/reseed, pending-reseed metadata, and local transcript/tool serialization from `session/prompt`. Compact remains internal lifecycle telemetry.
- Added versioned `codex-acp 1.1.7` and OpenCode `1.18.11` stream dialects. OpenCode now retains detailed `used/size/cost/currency` usage in addition to its existing answer, thought, and tool mapping.
- Added an executable fake ACP boundary regression that captures real JSON-RPC requests across new/resume/compact/instruction-rotation, plus a fixture based on a real OpenCode `1.18.11` stream.

## Verification

- Focused Codex boundary regression passed.
- Focused OpenCode stream fixture passed.
- `cargo check -p hone-channels --tests` passed.
- Real OpenCode ACP returned `OPENCODE_ACP_OK` through `initialize -> session/new -> session/prompt`.
- Real source-built Hone Codex ACP ran two turns in native session `019fbe23-f728-7a72-a2fb-6ed9260d5e31`; turn two resumed the same session and returned its exact sentinel. Rollout inspection found two Hone current-turn user payloads, with no system/history/tool replay or cross-turn marker.
- Workspace check and tests passed for all targets except the two repository-declared desktop exclusions. `hone-channels` passed 704 tests with one host-dependent OCR test ignored.
- Web passed 334 tests, Public Community Edge passed typecheck plus 45 tests, and every CI-safe regression script passed. Frozen Bun dependencies had to be installed in this new worktree before the JS gates could run.

## Risks / Follow-ups

- Codex itself may add adapter-owned environment/plugin messages to its rollout. The contract guarantees Hone's outbound `session/prompt`, not suppression of separate messages created inside the external harness.
- Adapter event shapes are external and version-sensitive. A version-floor change must rerun the real probes and update the explicitly labelled fixtures only after observation.
- OpenCode remains a fresh-session Hone-replay adapter; do not adopt Codex resume semantics until OpenCode persistence and replay behavior are independently validated.
- The official OpenCode installer added `~/.opencode/bin` to the local shell path. The machine's existing OpenAI provider refresh token was stale during validation; an isolated existing OpenRouter route completed without changing or exposing provider credentials.

## Next Entry Point

Use `docs/current-plans/acp-runtime-refactor.md` for remaining parent-runtime work. For this contract, start with the two version-labelled tests in `crates/hone-channels/src/runners/tests.rs` and rerun the corresponding real adapter probe before changing an adapter floor.
