# ACP 对齐的 Agent Runtime 全栈重构

- title: ACP 对齐的 Agent Runtime 全栈重构
- status: in_progress
- created_at: 2026-03-17
- updated_at: 2026-04-13
- owner: shared
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-channels/src/runners/acp_common.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
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
- `gemini_acp initialize timeout` has been diagnosed and fixed.
- Runner timeout config is being converged to two top-level knobs under `agent`: `step_timeout_seconds` and `overall_timeout_seconds`.
- ACP `session/prompt` now uses `idle=step_timeout_seconds` and `overall=overall_timeout_seconds`; `session/load timeout` now falls back to `session/new` instead of directly failing the turn.
- Remaining work is still needed around runner contract coverage and end-to-end runtime behavior alignment.

## Validation

- 2026-04-13:
  - `rtk cargo test -p hone-core test_agent_runner_timeouts_default_to_step_plus_overall test_agent_runner_timeout_override_preserves_explicit_values`
  - `rtk cargo test -p hone-channels runners::tests`
  - `rtk cargo check -p hone-channels`

## Documentation Sync

- Keep this file and `docs/adr/0002-agent-runtime-acp-refactor.md` aligned.
- If the runtime contract changes materially, update `docs/decisions.md`.
- Runner timeout semantics are now configured only through `agent.step_timeout_seconds` and `agent.overall_timeout_seconds`; keep `config.yaml` / `config.example.yaml` and the timeout analysis docs in sync when adjusting those values again.

## Risks / Open Questions

- The remaining work spans runners, channel ingress, and Web SSE semantics.
- Partial convergence is risky if one runner path silently diverges from ACP behavior.
