# ACP Prompt 双超时收口

- title: ACP Prompt 双超时收口
- status: done
- created_at: 2026-04-13
- updated_at: 2026-04-13
- owner: Codex
- related_files:
  - `crates/hone-channels/src/runners/acp_common.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/gemini_acp.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/config.rs`
  - `config.example.yaml`
  - `config.yaml`
- related_docs:
  - `docs/current-plan.md`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/bugs/opencode_acp_prompt_timeout.md`
- related_prs:

## Summary

Agent runtime timeout 已收敛到两档顶层配置：`agent.step_timeout_seconds = 180`、`agent.overall_timeout_seconds = 1200`。`codex_acp`、`gemini_acp`、`opencode_acp` 的 prompt 走 `idle=step / overall=overall`，`gemini_cli` 走 `per_line=step / overall=overall`。

## What Changed

- 在 `acp_common.rs` 新增公共 `wait_for_response_with_timeouts(...)`，按“收到任意 ACP 行就刷新 idle deadline”的语义等待最终 JSON-RPC `result`。
- 用户可配置的 runner timeout 只保留两档：
  - `step timeout`: 180 秒
  - `overall timeout`: 1200 秒
- `codex_acp` / `gemini_acp` / `opencode_acp`：
  - `initialize` / `session/load` / `session/new` / `session/set_model` 走 step timeout
  - `session/prompt` 走 `idle=step + overall=overall`
- `gemini_cli`：
  - `per_line_timeout` 走 step timeout
  - `overall_timeout` 走 overall timeout
- `session/load` 超时时不再直接失败，而是像 load error 一样降级到 `session/new`。
- `config.yaml` 与 `config.example.yaml` 已改为只在 `agent` 顶层暴露两个 timeout 字段。

## Verification

- `cargo fmt --all`
- `cargo test -p hone-core test_agent_runner_timeouts_default_to_step_plus_overall`
- `cargo test -p hone-core test_agent_runner_timeout_override_preserves_explicit_values`
- `cargo test -p hone-channels runners::tests`
- `cargo check -p hone-channels`

## Risks / Follow-ups

- 当前 idle timer 以“收到任意 ACP 行”为进展信号；如果以后需要把“工具心跳”和“真正用户可见进展”区分开，需继续细化语义。
- `codex_cli` / `function_calling` 这类 runner 仍没有统一的硬超时边界；本轮只收敛了 ACP、`gemini_cli` 和多 agent answer 路径。

## Next Entry Point

继续沿 `docs/current-plans/acp-runtime-refactor.md` 收口 ACP runner contract；若要再调 timeout 语义，优先看 `crates/hone-channels/src/runners/acp_common.rs` 和 `config.example.yaml`。
