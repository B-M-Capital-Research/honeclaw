# ACP 对齐的 Agent Runtime 全栈重构

- title: ACP 对齐的 Agent Runtime 全栈重构
- status: in_progress
- created_at: 2026-03-17
- updated_at: 2026-07-29
- owner: shared
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-channels/src/runners/acp_common/`
  - `crates/hone-channels/src/core/`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/attachments/vision.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-channels/src/agent_session/`
  - `memory/src/session.rs`
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
  - `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`
  - `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
  - `docs/handoffs/2026-07-29-codex-acp-discord-runtime-recovery.md`

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
- Historical note: the former `multi-agent` transcript merge work was superseded when that runner was removed on 2026-07-13; it is no longer an active runtime branch.
- ACP runners now treat their own session/compact logic as the source of truth: Hone skips its auto SessionCompactor for `codex_acp` / `opencode_acp`, and prompt construction suppresses Hone-side compact summaries for self-managed runners.
- `acp_common` now detects codex literal `Context compacted` chunks and opencode usage-drop / markdown-summary compact signatures, drops those leak paths from user-visible output, and sets session metadata so the next turn can reseed the system prompt when needed.
- `gemini_acp` is no longer offered as an active runtime path: factory creation now errors with a migration hint because Gemini ACP does not emit reliable `usage_update` signals and is unsafe for Hone's long-session compact detection model.
- `codex_acp` keeps passing `agent.codex_acp.variant` to Codex CLI as `model_reasoning_effort`, and also normalizes the ACP `session/set_model` selector to the adapter-required `model[effort]` form. Legacy `model/variant` and already-normalized `model[variant]` inputs converge to the same selector instead of sending a bare model id that the adapter rejects.
- Admin actors routed to `codex_acp` now use the adapter's native image capability directly instead of synchronously compiling and running the optional Apple Vision OCR helper before the first Discord placeholder. Strict public fallback actors retain local OCR pre-extraction. Timed-out OCR helper and Swift compiler children use kill-on-drop so they cannot remain behind as detached work.
- Remaining work is still needed around runner contract coverage and end-to-end runtime behavior alignment.
- 2026-06-24: Fixed ACP child-process lifecycle leaks where `codex_acp` / `opencode_acp` error or timeout paths could leave their stdio `hone-mcp` grandchildren running after the turn exits. ACP CLI children now run in their own process group and are cleaned up through a shared guard that terminates the group before returning from the runner.
- 2026-07-13 follow-up: the simplification task removed the in-process `function_calling` agent and sequential `multi-agent` runner, moved current defaults to Codex ACP with GPT-5.6 Sol / xhigh, and reduced duplicated prompt layers. This plan remains the parent ACP architecture record.
- 2026-07-13 completion: the simplification follow-up is done and archived at `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`; verification and migration details are in `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`.
- 2026-07-29 follow-up: a real Discord admin turn proved that `codex-acp 1.1.7` creates the intended `gpt-5.6-sol[xhigh]` session but rejected Hone's subsequent bare `gpt-5.6-sol` legacy `session/set_model` request before `session/prompt`. The selector is now normalized and covered. A second real image turn then exposed a separate 95.5-second pre-placeholder delay: two optional Apple Vision OCR helper compilation timeouts ran before ACP even started. Codex ACP admin image turns now bypass that redundant pre-extraction and use native image reads; Kimi/Hone session compaction remains unchanged and out of scope.

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
- ACP compact detection currently depends on codex literal markers plus opencode usage-drop heuristics; if upstream protocols change those signals, the detection path needs fresh live validation.
- ACP CLI cleanup must account for grandchildren such as `hone-mcp`; killing only the direct ACP process can leave orphaned MCP servers when the upstream CLI does not close them itself.
- Codex ACP model selection is a protocol compatibility boundary: adapter releases may change the accepted selector shape even when `session/new` continues to report the intended model. Keep focused tests for bare, legacy slash, and bracketed config forms, and validate against the minimum supported adapter plus the currently installed release.
