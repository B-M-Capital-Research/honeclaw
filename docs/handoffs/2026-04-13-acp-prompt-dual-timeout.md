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

ACP `session/prompt` 从固定 300 秒总超时改为双超时：`idle timeout = 300s`，`overall timeout = 1200s`。适用 runner 为 `codex_acp`、`gemini_acp`、`opencode_acp`。

## What Changed

- 在 `acp_common.rs` 新增公共 `wait_for_response_with_timeouts(...)`，按“收到任意 ACP 行就刷新 idle deadline”的语义等待最终 JSON-RPC `result`。
- `session/prompt` 超时现在区分：
  - `idle timeout`: 连续 300 秒无任何 ACP 输出
  - `overall timeout`: 整轮超过 1200 秒
- `set_acp_session_model(...)` 仍保留单次请求式等待，但默认使用 `min(idle, overall)`，避免被 20 分钟 overall timeout 放大。
- 三个 ACP 配置新增 `request_idle_timeout_seconds`，默认值为 300；`request_timeout_seconds` 默认值调整为 1200。
- 增补了配置默认值与兼容覆盖的单测，并同步更新示例配置和问题分析文档状态。

## Verification

- `rtk cargo fmt --all`
- `rtk cargo test -p hone-core test_acp_prompt_timeouts_default_to_idle_plus_longer_overall`
- `rtk cargo test -p hone-core test_acp_prompt_timeout_override_preserves_explicit_overall_value`
- `rtk cargo test -p hone-channels runners::tests`
- `rtk cargo check -p hone-channels`

## Risks / Follow-ups

- 当前 idle timer 以“收到任意 ACP 行”为进展信号；如果以后需要把“工具心跳”和“真正用户可见进展”区分开，需继续细化语义。

## Next Entry Point

继续沿 `docs/current-plans/acp-runtime-refactor.md` 收口 ACP runner contract；若要再调 timeout 语义，优先看 `crates/hone-channels/src/runners/acp_common.rs` 和 `config.example.yaml`。
