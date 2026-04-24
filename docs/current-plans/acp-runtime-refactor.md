# ACP 对齐的 Agent Runtime 全栈重构

- title: ACP 对齐的 Agent Runtime 全栈重构
- status: in_progress
- created_at: 2026-03-17
- updated_at: 2026-04-24
- owner: shared
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-channels/src/runners/acp_common.rs`
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-channels/src/agent_session.rs`
  - `crates/hone-core/src/storage.rs`
  - `crates/hone-channels/src/runners/gemini_acp.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-core/src/config/agent.rs`
  - `config.example.yaml`
  - `config.yaml`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/decisions.md`
  - `docs/bugs/opencode_acp_prompt_timeout.md`
  - `docs/handoffs/2026-04-13-acp-prompt-dual-timeout.md`

## Goal

Finish converging the agent runtime on ACP semantics so channel entrypoints, runners, and frontend streaming behave through one contract.

## Scope

- ACP runners already bridge into Hone MCP.
- `gemini_acp` 的初始化与 usage 信号链路已被完整复盘，但该 runner 现已禁用并输出迁移提示，不再作为收敛目标保留在默认运行路径里。
- Runner timeout config is being converged to two top-level knobs under `agent`: `step_timeout_seconds` and `overall_timeout_seconds`.
- ACP `session/prompt` now uses `idle=step_timeout_seconds` and `overall=overall_timeout_seconds`; `session/load timeout` now falls back to `session/new` instead of directly failing the turn.
- `codex_acp` transcript is being reworked so intermediate model output is preserved in restorable transcript segments without flattening everything into one assistant blob.
- Common code now only carries generic `message.metadata`; runner-specific transcript fields must stay in each runner / channel implementation instead of being centralized under a shared ACP schema.
- `codex_acp` and `opencode_acp` now share the same normalized cross-turn history model: top-level history restores as `user/assistant` turns, while tool calls/results and progress/final answer are represented inside assistant `content[]` parts instead of as runner-specific prompt JSON.
- Session storage itself now writes the normalized model directly as `version=4` with `content[] + status` instead of the old flat string `content`; legacy JSON still deserializes for compatibility, but new writes use the breaking on-disk layout.
- `codex_acp` now patches execute-completion `tool_call_update.rawOutput` into persisted `tool_result` parts, so codex execute turns are recorded as `progress -> tool_call -> tool_result -> final` in the same assistant turn instead of falling back to a partial tool-call-only record.
- `codex_cli` reasoning runs are now explicitly covered by the same normalized persistence contract: runner tail messages are normalized into `progress/tool_call/tool_result/final` assistant content parts before storage.
- `multi-agent` no longer drops intermediate stage transcript at the top-level runner boundary: search-stage and answer-stage `context_messages` are now merged and persisted back into the shared session model instead of storing only the final answer fallback.
- ACP runners now treat their own session/compact logic as the source of truth: Hone skips its auto SessionCompactor for `codex_acp` / `opencode_acp`, and prompt construction suppresses Hone-side compact summaries for self-managed runners.
- `acp_common` now detects codex literal `Context compacted` chunks and opencode usage-drop / markdown-summary compact signatures, drops those leak paths from user-visible output, and sets session metadata so the next turn can reseed the system prompt when needed.
- `gemini_acp` is no longer offered as an active runtime path: factory creation now errors with a migration hint because Gemini ACP does not emit reliable `usage_update` signals and is unsafe for Hone's long-session compact detection model.
- `codex_acp` now treats `agent.codex_acp.variant` as Codex CLI `model_reasoning_effort` instead of appending it to the model id; legacy `model/variant` strings are stripped back to the base model before starting the ACP session.
- Remaining work is still needed around runner contract coverage and end-to-end runtime behavior alignment.

## Validation

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

## Documentation Sync

- Keep this file and `docs/adr/0002-agent-runtime-acp-refactor.md` aligned.
- If the runtime contract changes materially, update `docs/decisions.md`.
- Runner timeout semantics are now configured only through `agent.step_timeout_seconds` and `agent.overall_timeout_seconds`; keep `config.yaml` / `config.example.yaml` and the timeout analysis docs in sync when adjusting those values again.
- If ACP transcript persistence semantics change, update the ACP runtime ADR or `docs/decisions.md` to reflect the new transcript contract.
- Compact leak handling and Gemini ACP disablement must stay aligned with `docs/bugs/session_compact_summary_report_hallucination.md` and `config.example.yaml`.
- If runner-specific transcript metadata is added later, keep it under the owning runner/channel namespace and avoid introducing a shared ACP-wide event schema in `memory` or other common storage helpers.
- If the normalized history model expands again, preserve runner interchangeability: prompt restoration should keep consuming the shared `user/assistant` model rather than any single runner’s raw event stream.

## Risks / Open Questions

- The remaining work spans runners, channel ingress, and Web SSE semantics.
- Partial convergence is risky if one runner path silently diverges from ACP behavior.
- `opencode_acp` and `codex_acp` now consume the same normalized history for prompt restore, but their raw ACP event shapes still differ; raw-session persistence and replay must remain runner-owned.
- Runner-specific transcript metadata can still grow session files; any future expansion should be validated against real session size and restore cost.
- ACP compact detection currently depends on codex literal markers plus opencode usage-drop heuristics; if upstream protocols change those signals, the detection path needs fresh live validation.
