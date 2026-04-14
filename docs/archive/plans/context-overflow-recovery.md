# Plan

- title: Context Overflow Recovery And User-Facing Error Hygiene
- status: done
- created_at: 2026-04-14T14:42:09+0800
- updated_at: 2026-04-14T14:58:00+0800
- owner: codex
- related_files:
  - crates/hone-channels/src/agent_session.rs
  - crates/hone-channels/src/session_compactor.rs
  - crates/hone-channels/src/runners/gemini_cli.rs
  - bins/hone-feishu/src/handler.rs
  - bins/hone-imessage/src/main.rs
- related_docs:
  - AGENTS.md
  - docs/current-plan.md
  - docs/invariants.md

## Goal

修复会话上下文超限时把底层 provider 原始错误直接透传给用户的问题；优先自动压缩/裁剪上下文并重试一次，若仍失败则返回合理、非底层实现细节的用户提示。

## Scope

- 为 `AgentSession` 增加上下文超限识别与恢复性重试
- 在重试前强制 compact 当前 session，并用新上下文重新准备 execution request
- 将上下文超限最终失败改写为稳定的用户可见错误文案，不再暴露 `bad_request_error`、`invalid params` 等 provider 字样
- 为恢复成功与恢复失败两条路径补自动化测试

## Validation

- 已完成：
  - `cargo test -p hone-channels`
  - `cargo test -p hone-channels context_overflow_auto_compacts_and_retries_successfully -- --nocapture`
  - `cargo test -p hone-channels context_overflow_failure_is_rewritten_to_friendly_message -- --nocapture`

## Documentation Sync

- 该任务满足动态计划准入标准：影响运行时行为且涉及多模块恢复路径，因此落盘到 `docs/current-plan.md` 与本计划页
- 若本回合完成，移出活跃索引、归档计划并更新 `docs/archive/index.md`

## Risks / Open Questions

- 已收口：
  - `AgentSession` 现在会识别 `context window exceeds limit` / `maximum context length` / `too many tokens` 等常见超限报错
  - 命中后会强制 compact 当前 session，并在同一 turn 内重新准备 execution 后自动重试一次
  - 若自动恢复后仍失败，最终用户只会看到稳定友好的提示，不再看到 `bad_request_error` / `invalid params` 之类底层 provider 报错
- 剩余注意事项：
  - 当前自动恢复只做一次强制 compact；若未来某些 runner 需要“缩短 recent window 再试”之类更细粒度策略，可在此基础上继续扩展
