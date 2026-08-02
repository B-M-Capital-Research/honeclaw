# Codex ACP 工具状态跨渠道投影修复

- title: Codex ACP 工具状态跨渠道投影修复
- status: done
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: shared
- related_files:
  - `crates/hone-channels/src/runners/acp_common/ingest.rs`
  - `crates/hone-channels/src/runners/acp_common/tests.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `crates/hone-channels/src/outbound.rs`
  - `bins/hone-imessage/src/main.rs`
- related_docs:
  - `docs/current-plan.md`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/archive/index.md`
- related_prs: none; local change set is not committed or pushed

## Summary

Codex ACP 1.1.7 把取消的 `hone` MCP 启动 watcher 作为 `mcp_startup.hone` 失败工具事件上报，同时把真正的 MCP 调用和本地 shell 都标成 `kind=execute`。旧投影因此先显示并计数一个并非模型调用的 MCP startup，又把 `mcp.hone.web_search` 错写成“本地命令”。本次修复在共享 ACP/runner 层区分 adapter lifecycle、结构化 MCP 和 shell，确保各渠道消费同一份安全事件。

## What Changed

- `mcp_startup.<server>` 仍写入原始 ACP 日志供排障，但不再产生用户可见 `ToolStatus`，也不进入 pending/restored tool state 或业务工具计数。
- `rawInput.server/tool/arguments` 生成有界 MCP 摘要，例如 `hone/web_search query="..."`。
- shell 的 argv 和字符串两种真实 Codex 负载都收敛为类别摘要，例如 `检查 Git（git status）`、`读取本地内容（pwd）`、`请求接口（curl）`；完整参数、路径、URL 和 secret-like 值不会进入摘要。
- Codex 完成事件缺少 `kind/rawInput` 时，从对应 pending start 恢复显示上下文，使 start/done 使用同一个安全标签。
- Discord、Telegram 使用共享 outbound Full/Compact 路径；飞书的 Full/Compact 卡片 listener 复用相同摘要；群聊会把 MCP 查询和 shell 进一步压成 `正在搜索信息...`、`正在检查 Git...` 等无参数状态。
- iMessage 不新增聊天气泡，只把安全的 tool start 摘要转发到已有控制台 pending 状态。

## Verification

- `cargo test -p hone-channels --lib`: `710 passed`, `1` host-dependent OCR test ignored.
- `cargo test -p hone-imessage`: `3 passed`.
- `cargo check -p hone-discord -p hone-telegram -p hone-feishu -p hone-imessage`: passed.
- Exact changed Rust files were formatted directly with Rustfmt; `git diff --check` passed.
- Isolated live Discord MCP probe: raw startup watcher was logged and suppressed; actual tool emitted `hone/web_search query="..."` and completed successfully.
- Isolated final Discord shell probe: both start and done emitted `读取本地内容（pwd）`; the reply was `LOCAL_PROBE_OK` and the full sandbox path was absent from visible status.
- Source LaunchAgent restarted only after active chats reached zero. It runs Cargo from this repository, Discord re-authenticated, all four local HTTP surfaces returned `200`, and active chats returned to zero.

## Risks / Follow-ups

- Current source config has no non-empty Telegram administrator ID, so Telegram cannot presently route a real Codex ACP turn. Authorization was not changed for testing; shared rendering regressions and channel-bin compilation cover that dormant route.
- Only Web and Discord are enabled in the current source runtime. Telegram, Feishu, and iMessage were compiled and tested but not connected to external accounts or sent canary messages.
- Shell summaries intentionally expose only the first safe command category and optional safe subcommand. Detailed arguments remain in internal ACP logs, not channel progress.
- No ADR or decision update is required: this restores the existing lifecycle/user-visible projection boundary and does not change ownership, routing, or storage authority.

## Next Entry Point

For another mislabeled tool event, start from the raw `session/update` in `data/runtime/logs/acp-events.log`, then inspect `render_codex_tool_status`, `tool_update_with_pending_start`, and `render_compact_tool_status_start`. Preserve raw diagnostics while keeping adapter lifecycle and secret-bearing command details out of channel-visible progress.
