- title: Feishu Direct Busy Placeholder Gap
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - bins/hone-feishu/src/handler.rs
  - docs/bugs/README.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md
- related_docs:
  - docs/archive/plans/feishu-direct-busy-placeholder-gap.md
  - docs/runbooks/desktop-release-app-runtime.md
- related_prs:
  - N/A

## Summary

本轮定位到最新 `yoyoyo` 对应的 Feishu 私聊失败不是 Tavily / MiniMax / answer provider 报错，而是同一 direct session 已有旧任务处理中时，入口仍然先发送 placeholder，新消息随后卡在更深层 session run lock 之前，导致用户只看到“正在处理中”却没有真正进入 agent 主链路。

## What Changed

- 在 `bins/hone-feishu/src/handler.rs` 中把 `SessionLockRegistry` 的 busy 检查前移到 placeholder 之前。
- Feishu 私聊现在也会在 session 冲突时返回明确 busy 提示，并记录 `direct.busy` 日志。
- 只有真正拿到处理权的消息，才会继续 ingest attachment、发送 placeholder 并进入 `session.run()`。
- 更新 bug 台账，将该缺陷登记并在修复后切换到 `Fixed`。

## Verification

- `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`
- 运行日志复核：此前 `reply.placeholder` 后缺失 `session.persist_user` / `recv` / `agent.run` 的根因可由入口期 busy 缺口解释

## Risks / Follow-ups

- 该修复解决的是“placeholder 假启动”与入口反馈错误，不直接证明深层 `session.run()` 为何会在个别 direct session 长时间占锁。
- 如果后续还观察到某些 direct session 长时间频繁触发 `direct.busy`，需要再单独追踪持锁过久的下游根因。

## Next Entry Point

- `docs/bugs/README.md`
