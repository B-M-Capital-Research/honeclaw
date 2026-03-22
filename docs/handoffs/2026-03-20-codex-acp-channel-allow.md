# 2026-03-20 Codex ACP 全渠道放行

## 结果

- 已移除 `codex_cli` / `codex_acp` 在 `AgentSession` 入口的 strict actor sandbox 拒绝逻辑，正式渠道不再因为该 guard 直接 fail-fast。
- 已将 `config.yaml` 与 `data/runtime/config_runtime.yaml` 的默认 runner 切到 `codex_acp`。
- 已同步更新长期约束文档，明确这是“已接受 Codex workspace-write 可能越界读取”的有意风险决策，而不是仍受 guard 保护的状态。

## 关键改动

- `crates/hone-channels/src/core.rs`
  - `runner_supports_strict_actor_sandbox()` 现统一返回 `true`
  - `strict_actor_sandbox_guard_message()` 不再对 `codex_cli` / `codex_acp` 返回拒绝文案
- `config.yaml`
- `data/runtime/config_runtime.yaml`
  - `agent.runner` 切为 `codex_acp`
- `docs/invariants.md`
  - 从“禁止在正式渠道使用 codex_*”更新为“允许使用，但风险已接受”

## 风险

- `codex_acp` / `codex_cli` 仍保留既有已知风险：`workspace-write` 可能读取 actor sandbox 外的 repo 文件。
- 这次变更是全渠道生效，影响 Telegram / Discord / Feishu / iMessage / CLI / KB analysis 等统一 runner 链路。
- 若后续需要恢复隔离约束，可优先回滚 `crates/hone-channels/src/core.rs` 的 guard，并把默认 runner 切回 `gemini_acp` 或 `opencode_acp`。

## 验证建议

- `cargo check -p hone-channels -p hone-cli -p hone-telegram`
- `printf 'Reply with exactly: HONE_CODEX_ACP_OK\nquit\n' | cargo run -q -p hone-cli`
- 重启渠道进程后，在真实 Telegram 对话中观察 `dialog.engine=codex_acp` 与首条回复是否成功
