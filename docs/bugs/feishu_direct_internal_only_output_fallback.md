# Bug: Feishu direct 短问题失败后外发 internal-only output 占位文案

- 发现时间：2026-07-27 15:03 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：Fixed
- GitHub Issue：无，非 P1

## 修复进展

- **2026-08-20 代码级修复，状态更新为 `Fixed`**：
  - `crates/hone-channels/src/runtime.rs`
    - `looks_internal_error_detail()` 现把 `agent returned internal-only output` 识别为内部错误细节，统一映射到通用用户态失败文案，避免内部占位短语进入用户可见 final。
  - `crates/hone-channels/src/agent_session/tests.rs`
    - 新增 `failed_assistant_persisted_message_hides_internal_only_output_error` 回归，锁定 `AgentFailed + internal-only output` 只能持久化脱敏失败文案。
  - `crates/hone-channels/src/runtime.rs`
    - 新增 `user_visible_error_message_rewrites_internal_only_output_errors` 回归，锁定统一错误映射不会把该内部占位原样返回给用户。
  - 验证：
    - `cargo test -p hone-channels user_visible_error_message_rewrites_internal_only_output_errors --lib -- --nocapture`
    - `cargo test -p hone-channels failed_assistant_persisted_message_hides_internal_only_output_error --lib -- --nocapture`
    - `cargo check -p hone-channels --tests`
  - 说明：
    - 本轮完成代码与回归闭环，但未重启或重建任何运行态服务；live 复核仍待后续自然部署窗口观察。

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
  - `2026-07-27T12:01:28.041110+08:00` 用户短问 `AAOI和海力士财报准确时间`。
  - `2026-07-27T12:01:57.250586+08:00` assistant final 只返回 `agent returned internal-only output`。
  - 同条 assistant `metadata_json` 标记 `error_kind=AgentFailed`、`run_failed=true`，并带同一 Feishu `message_id`。
- 本轮窗口 `2026-07-27 11:01:50-15:03 CST` 对照：
  - `data/sessions.sqlite3` 新增 10 条 user、8 条 assistant、2 条 system compact，覆盖 5 个更新 session。
  - 同窗其它 Feishu direct / scheduler 样本可正常收口，未见全渠道停摆、错投、敏感信息泄露或本机绝对路径外泄。
  - 最近四小时无非文档代码提交；未找到同名已登记缺陷。

## 端到端链路

1. Feishu 用户在直聊旧会话中提出一个明确、低复杂度的财报时间查询。
2. 直聊 runner 进入 agent 执行链路并失败，落库元数据标记 `AgentFailed`。
3. 失败收口没有映射成可理解的用户态错误，也没有保留可用的财报日期答复。
4. 出站层把 `agent returned internal-only output` 作为 final 发送给用户。

## 期望效果

- 对这类短问题，系统应直接回答 AAOI 与海力士的准确财报时间；若上游执行失败，也应返回脱敏、可理解、可重试的用户态失败说明。
- 用户可见 final 不应包含 `internal-only output` 这类内部占位短语。
- 失败元数据应保留给日志和台账，不能替代最终回复正文。

## 当前实现效果

- 用户没有得到任何财报时间信息。
- final 只包含内部占位文案，既不可执行，也无法解释失败原因。
- 本轮不是空消息伪成功：消息有非空正文，但正文是内部失败占位符；也不是既有 `agent_owned_finance_persistent_tool_error` 裸 key，而是新的 `internal-only output` 失败变体。

## 用户影响

- 用户明确提出的财报时间查询完全未完成，需要重新提问或换会话绕行。
- 该问题影响 Feishu direct 主问答链路和错误边界，因此定级为功能性 `P2`。
- 未升级为 P1 的原因：本轮只确认单个 Feishu direct 短问失败，同窗其它 direct / scheduler 有正常 assistant 收口，未见全渠道不可用、跨用户错投、数据破坏或敏感信息泄露。

## 根因判断

- 直接症状是 `AgentFailed` 后的用户可见错误映射缺失：内部占位短语被当作 final 文本发送。
- 该样本发生在历史较长的 Feishu direct session 中，但本轮未见 `context_window_exceeded`、`/compact` 或 `<absolute-path>/compact` 文案，因此不直接归入已关闭的 compact recovery 缺陷。
- 与空回复伪成功缺陷相邻但不同：本轮不是 `reply_chars=0` 或 `planning_sentence_suppressed` 后成功发送通用 fallback，而是失败态直接暴露内部占位文案。

## 下一步建议

- 在 direct 失败收口层把 `agent returned internal-only output` 识别为内部错误细节，统一映射成脱敏、可理解的用户态失败说明。
- 追加回归：Feishu direct `AgentFailed + internal-only output` 不得原样进入 final。
- 若上下文恢复或 Answer 阶段能拿到部分工具证据，应优先恢复为具体业务答案，而不是只发送失败占位。
