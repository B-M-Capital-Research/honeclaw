# Plan

- title: Feishu 直聊 placeholder 假启动后续修复
- status: in_progress
- created_at: 2026-04-16 14:12 CST
- updated_at: 2026-04-16 14:18 CST
- owner: Codex
- related_files:
  - bins/hone-feishu/src/handler.rs
  - bins/hone-feishu/src/types.rs
  - data/runtime/config_runtime.yaml
  - docs/bugs/README.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md
- related_docs:
  - docs/current-plan.md
  - docs/bugs/README.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md

## Goal

修复 Feishu 直聊消息在发送 placeholder 后未进入主链路的问题，恢复受影响用户可用性；同时把 `+8613871396421` 对应 Feishu 用户补入管理员配置，并完成日志级回归验证。

## Scope

- 排查 `process_incoming_message` 在 placeholder 前后和 `session.run()` 之前的静默中断点
- 修复 Feishu 文本/空输入/异常兜底路径，避免“placeholder 假启动”
- 将 `+8613871396421` 对应 Feishu 身份加入当前运行配置管理员名单
- 更新相关 bug 文档与导航页

## Validation

- 检查 Feishu 运行日志，确认新消息不再只停留在 `reply.placeholder`
- 至少验证出现 `session.persist_user` / `recv` / `agent.prepare` / `agent.run` 或显式失败兜底
- 检查当前运行配置中管理员项已生效
- 已完成：
  - `cargo test -p hone-feishu actionable_user_input_detects_empty_payload -- --nocapture`
  - `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`
  - `cargo build --release -p hone-feishu`
  - 重启当前 `hone-release` 进程组并确认 Feishu 渠道重新连上 stream
- 待补：
  - 下一条真实 Feishu 用户消息的端到端验证

## Documentation Sync

- 更新 `docs/current-plan.md` 活跃索引
- 视修复结果更新 `docs/bugs/README.md` 与 `docs/bugs/feishu_direct_placeholder_without_agent_run.md`
- 完成后将计划移出活跃索引并按需要归档

## Risks / Open Questions

- 最新“喂喂喂”“1”两条消息未成功落库，现有证据只能定位到 placeholder 后静默中断，仍需通过代码路径与新日志进一步缩小范围
- 若最终确认存在第二个独立根因，可能需要从当前 bug 拆分成新文档
