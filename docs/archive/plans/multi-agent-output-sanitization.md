# Plan

- title: Multi-Agent Output Sanitization And Leak Repair
- status: done
- created_at: 2026-04-13T22:22:05+0800
- updated_at: 2026-04-13T23:45:00+0800
- owner: codex
- related_files:
  - crates/hone-channels/src/runners/multi_agent.rs
  - crates/hone-channels/src/agent_session.rs
  - crates/hone-channels/src/runtime.rs
  - crates/hone-channels/src/session_compactor.rs
  - bins/hone-feishu/src/handler.rs
  - bins/hone-feishu/src/listener.rs
  - crates/hone-channels/src/outbound.rs
- related_docs:
  - AGENTS.md
  - docs/current-plan.md
  - docs/invariants.md
  - docs/repo-map.md

## Goal

修复 multi-agent / channel 链路会把 `<think>`、`<tool_call>`、`[TOOL_CALL]` 等内部工作稿或伪工具调用正文透传给用户的问题，并阻断其继续污染 compact summary 与后续会话。

## Scope

- 为 multi-agent 搜索阶段增加“内部工作稿 / 伪工具调用正文”识别，避免误判为可直接回复用户
- 在统一运行时层补充可复用的输出净化逻辑，要求在渠道将 `<think>` 转换为引用前先清理不应暴露的片段
- 阻断 compact summary 将内部工作稿落盘回会话
- 为关键格式补充自动化测试，并在修复后重启本地系统验证

## Validation

- 已完成：
  - `cargo test -p hone-channels`
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels sanitize_user_visible_output -- --nocapture`
  - `cargo test -p hone-channels restore_context_sanitizes_polluted_assistant_history -- --nocapture`
  - `cargo test -p hone-channels internal_search_note_does_not_skip_answer_stage -- --nocapture`
- 未执行：
  - 本地运行时重启验证（本回合聚焦代码与自动化回归，未启动整套 listener/runtime 进程）

## Documentation Sync

- 该任务满足动态计划准入标准：跨模块、影响行为、需要验证与重启，因此落盘到 `docs/current-plan.md` 与本计划页
- 若本回合完成修复，归档本计划到 `docs/archive/plans/`，并更新 `docs/archive/index.md`
- 如需后续接力或保留风险，再新增 handoff；若验证与风险已在归档中可完整表达，则可不单独新增 handoff

## Risks / Open Questions

- 已收口的泄漏面：
  - multi-agent 搜索阶段不再把带内部工作稿的 zero-tool 搜索笔记直接当最终答案返回
  - `AgentSession` 在持久化 assistant 内容前会统一净化；只有内部协议残渣的回复会被直接判失败
  - `restore_context` / `session_compactor` 会跳过或净化污染 assistant / compact summary 内容，阻断历史污染继续回灌 prompt
  - Feishu / Telegram / Discord / iMessage 用户可见输出已切到隐藏 `<think>`，Feishu / iMessage 流式 formatter 也会吞掉 `<tool_call>` / `<tool_result>` / `<tool_use>`
- 剩余注意事项：
  - 本轮自动化已覆盖核心 Rust 路径，但还没做一轮真实 Feishu 会话的手工回放
