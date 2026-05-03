# Bug: Web 直聊 `session/update` 把 skill prompt、工具原始回显与绝对路径作为 `rawOutput` 外发

- **发现时间**: 2026-05-02 20:03 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#30](https://github.com/B-M-Capital-Research/honeclaw/issues/30)

## 证据来源

- 最近一小时真实会话：
  - `data/sessions/Actor_web__direct__web-user-e05f5e5f74a3.json`
  - `updated_at=2026-05-02T20:02:07.383458+08:00`
  - 同一 session 在最近窗口内继续正常写入最终 assistant 正文，说明业务回答主链路仍能收口
- 最近一小时运行日志：
  - `data/runtime/logs/acp-events.log`
  - `2026-05-02T12:00:06.975323+00:00` 同一 Web session `019dca1a-9c4c-74e2-bebf-66d97c78e6b7` 收到 `session/update -> tool_call_update`
  - 该更新的 `rawOutput` 直接展开整段 `Scheduled Task Management` skill 内容，包含：
    - `【Invoked Skill Context】`
    - `Base directory for this skill: /Users/fengming2/Desktop/honeclaw/skills/scheduled_task`
    - 完整 tool guide、严格规则、event-driven reminder linkage
  - 同窗内还出现多条 `tool_call_update.rawOutput` / `tool_call_update.content`，直接外发结构化工具结果，例如：
    - `2026-05-02T12:00:06.988264+00:00` 创建定时任务返回完整 `job` JSON，含 `channel_target`、`task_prompt`、`job id`
    - 同一轮还把 `rawOutput` 形式的 skill prompt 与工具结果一起发到客户端事件流
- 旧样本对照：
  - 同一 `acp-events.log` 中，`2026-04-26T14:06:47.918884+00:00` 与 `2026-04-26T14:07:14.870164+00:00` 也曾对同一 Web actor 发送 `tool_call_update.rawOutput`，内容分别是 `Scheduled Task Management` 与 `Stock Research` 的完整 skill prompt
  - 说明这不是一次性日志污染，而是 Web `tool_call_update/rawOutput` 通道长期把内部回显当成可下发事件
- 与已修缺陷区分：
  - [`web_direct_session_update_prompt_echo_leak.md`](./web_direct_session_update_prompt_echo_leak.md) 记录的是 `agent_message_chunk` 把 `### System Instructions ###` prompt 包当成正文 chunk 外发
  - 本单复现的是 `tool_call_update.rawOutput` 继续下发 skill prompt、工具返回和绝对路径，事件类型与泄漏边界不同

## 端到端链路

1. Web 用户在最近一小时内继续使用同一 direct session 发起研究与定时任务请求。
2. ACP 桥接层持续向客户端发送 `session/update` 事件。
3. 最终 answer 能正常收口到 session JSON，但中途 `tool_call_update` 事件把 `rawOutput` 原样透传。
4. 透传内容不只包括工具结果 JSON，还包括 skill prompt 原文、`/Users/.../skills/...` 绝对路径，以及命令/工具执行回显。
5. 结果是：即使最终落库正文看起来正常，实时 Web 事件流仍把内部实现细节公开给前端/用户。

## 期望效果

- Web `session/update` 只能下发对用户有意义的进度摘要，不应直接暴露 `rawOutput`。
- skill prompt、工具协议、结构化 tool result、绝对路径、命令回显应保留在内部诊断层，不应进入用户可见事件流。
- 即使需要显示“工具已完成”，也应只给简短状态，而不是把原始 payload 整段透传。

## 当前实现效果

- 最近一小时同一 Web session 的 `tool_call_update.rawOutput` 直接携带完整 skill prompt 与工具返回。
- 泄漏内容同时包含：
  - skill 基础目录绝对路径
  - 内部工具说明与严格规则
  - 定时任务结构化结果（`channel_target`、`task_prompt`、`job id`）
  - 原始工具/命令输出包装结构
- 同一 session JSON 最终仍能只保留正常 final，说明当前问题集中在“实时外发边界”，而不是最终 answer 落库污染。

## 用户影响

- 这是功能性安全边界缺陷，不是单纯格式退化。
- 一旦前端渲染或调试面板展示这些 `tool_call_update` 事件，用户会直接看到内部 skill prompt、工具结构和本机绝对路径。
- 该问题同时泄露运行实现细节和用户任务配置细节，且在最近一小时真实 Web 会话里持续复现，因此定级为 `P1`。

## 根因判断

- 现有修复只覆盖了 `agent_message_chunk` 的 prompt echo 过滤，没有覆盖 `tool_call_update.rawOutput` 这一独立事件通道。
- `rawOutput` 当前仍按“可直接透传的调试字段”进入 Web `session/update`，缺少用户态净化与字段级裁剪。
- 从旧样本看，问题不是某个单一 skill 的异常，而是 Web 侧对 `tool_call_update/rawOutput` 的统一下发策略缺口。

## 修复进展（2026-05-03 18:20 CST）

- 已在 `crates/hone-channels/src/agent_session/emitter.rs` 收口共享用户态事件出站：
  - `RunEvent::ToolStatus` 的 `tool` / `message` / `reasoning` 现在统一先做路径相对化，再过 `sanitize_user_visible_output`
  - 命中 `【Invoked Skill Context】`、`Base directory for this skill:`、`### System Instructions ###`、`turn-0 可用技能索引` 等内部 prompt 标记时，用户态字段直接抑制，不再继续外发
  - 若 `tool_call_update` 文本本体是结构化 JSON / array payload，也会在用户态事件层被直接抑制；内部 transcript 与 `acp-events.log` 仍保留原始证据，便于恢复与排障
- 这次修复与已完成的 `agent_message_chunk` prompt-echo 过滤形成互补：
  - `agent_message_chunk` 继续在 ACP ingest 层拦截内部 prompt 正文
  - `tool_call_update` / `ToolStatus` 则在 session emitter 层补齐用户态净化，避免 raw payload 经 `tool_call` SSE 或渠道进度事件漏出
- 新增回归测试：
  - `session_event_emitter_relativizes_user_visible_paths`
  - `session_event_emitter_suppresses_internal_tool_status_payloads`

## 当前验证（2026-05-03 18:20 CST）

- 已通过：
  - `cargo test -p hone-channels session_event_emitter_ -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session/emitter.rs crates/hone-channels/src/agent_session/tests.rs`

## 当前结论

- 该缺陷的可控代码路径已补齐并有自动化回归覆盖，可从活跃队列移到 `Fixed`。
- 本轮没有重启现有 Web 服务，因此未做 live SSE 复流；如需运行态复核，可在不重启生产实例的前提下，用隔离会话再次触发 `skill_tool` / `cron_job`，确认前端只看到简短工具状态，而不再看到 skill prompt、绝对路径或 `job` JSON。
