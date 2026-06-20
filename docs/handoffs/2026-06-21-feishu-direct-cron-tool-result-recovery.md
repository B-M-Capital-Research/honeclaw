- title: Feishu direct cron tool result recovery
- status: done
- created_at: 2026-06-21 03:06 CST
- updated_at: 2026-06-21 03:06 CST
- owner: Codex bug-2 automation
- related_files:
  - crates/hone-channels/src/response_finalizer.rs
  - crates/hone-channels/src/agent_session/tests.rs
  - docs/bugs/feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md
  - docs/bugs/README.md
- related_docs:
  - docs/bugs/feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md
  - docs/bugs/README.md
- related_prs:
  - N/A

## Summary

Feishu direct 的定时任务治理回复继续往确定性收口推进：当 `cron_job` 工具其实已经返回结果，但模型最终回复退化成过渡句，或被共享净化层统一压成“定时任务管理暂时不可用，请稍后再试”时，`response_finalizer` 现在会优先恢复真实工具结果，而不是把这类 turn 留在“工具未暴露”假象里。

## What Changed

- `crates/hone-channels/src/response_finalizer.rs`
  - 新增 `cron_job(action="list")` 的工具结果恢复，直接输出用户态任务列表摘要。
  - 新增 `cron_job(action="remove")` 且 `needs_confirmation=true` 的确认文案恢复，避免用户只看到通用失败提示。
  - 对 `cron_job(action="add"/"update")` 保留既有恢复，并把恢复时机扩展到“净化后为空”与“净化后只剩通用 cron 不可用提示”两类场景。
- `crates/hone-channels/src/agent_session/tests.rs`
  - 新增 `finalize_agent_response_recovers_cron_job_list_from_tool_result`
  - 新增 `finalize_agent_response_recovers_cron_job_remove_confirmation_from_tool_result`
  - 新增 `finalize_agent_response_recovers_cron_job_result_after_sanitization_strips_internal_copy`

## Verification

- `cargo test -p hone-channels finalize_agent_response_recovers_cron_job_ --lib -- --nocapture`
- `cargo test -p hone-channels finalize_agent_response_recovers_portfolio_confirmation --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## Risks / Follow-ups

- 这次修复的是“工具结果已存在但最终回复退化”的确定性收口，不证明 Feishu direct live 运行态已经不再出现“根本没有调用 `cron_job`”的新样本。
- 如果后续部署当前代码后，Feishu direct 仍然把用户的列任务/建任务请求收口成“工具没暴露/暂时不可用”，应继续沿 MCP tools list、runner stage allowed tools、真实 `ToolCallMade` 记录排查是否存在未调用工具的独立根因。

## Next Entry Point

- `docs/bugs/feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md`
