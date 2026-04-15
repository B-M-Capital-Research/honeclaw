# Bug: 渠道失败分支会把原始 LLM/provider 报错直接发给用户

- **发现时间**: 2026-04-15 21:20 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-15T20:58:51.471633+08:00` 用户消息：`最近存储系列的还能买吗`
    - `2026-04-15T21:00:30.520799+08:00` 用户追问：`咋回事`
    - 说明失败发生后，用户没有拿到正常回答，只能立即追问原因；失败文案没有作为正常 assistant 消息入库
  - 最近运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 20:58:58.152` `MsgFlow/feishu failed ... error="LLM 错误: bad_request_error: invalid params, invalid function arguments json string, tool_call_id: call_function_v0wwk1qhh65v_1 (2013)"`
    - 同一会话随后在 `21:00:30` 重试，最终于 `21:04:05` 恢复出正常回答
  - 更完整的渠道日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-15T12:58:58.152238Z` 同样记录 `error="LLM 错误: bad_request_error: invalid params, invalid function arguments json string, tool_call_id: call_function_v0wwk1qhh65v_1 (2013)"`
  - 代码证据：
    - `bins/hone-feishu/src/handler.rs:372-389`
    - `crates/hone-channels/src/outbound.rs:140-150`
    - `crates/hone-core/src/error.rs:9-12`
  - 相关历史缺陷：
    - `docs/bugs/context_overflow_recovery_gap.md`

## 端到端链路

1. 用户在 Feishu 直聊发起正常问题，请求继续分析“存储系列”是否还能买。
2. Multi-Agent 搜索阶段在约 6.5 秒内失败，底层返回 `bad_request_error: invalid params, invalid function arguments json string ...`。
3. `AgentSession` 把原始 `err.to_string()` 作为 `response.error` 往上抛。
4. Feishu 失败分支直接执行 `format!("抱歉，处理出错了: {}", truncated)`，把截断后的原始错误拼进给用户的展示文本。
5. 用户侧看到的不是产品化错误提示，而是底层 provider/协议细节；随后只能追问“咋回事”。

## 期望效果

- 渠道失败分支应向用户返回稳定、产品化的错误提示，例如“处理失败，请稍后再试”或“工具调用失败，已自动重试仍未成功”。
- 底层 provider 名称、`bad_request_error`、`invalid params`、`tool_call_id` 这类内部实现细节不应直接暴露给最终用户。
- 原始错误应保留在日志或受控诊断渠道，用于排障，而不是进入普通对话面板。

## 当前实现效果

- 当前 Feishu 直聊失败分支会把 `response.error` 原样截断后直接拼入用户消息。
- 这次真实故障中的错误文本包含 `bad_request_error`、`invalid function arguments json string` 和具体 `tool_call_id`，都属于内部实现细节。
- 该问题不是 `context window exceeds limit` 老缺陷的原样回归；老缺陷只为“上下文超限”补了特判改写，其他 provider 错误仍会直出。
- 同类风险不只存在于 Feishu：共享的 `run_session_with_outbound(...)` 失败路径也会执行 `抱歉，处理失败：{truncate_chars(err, 300)}`，说明这是跨渠道的共性缺口。

## 用户影响

- 这是功能性缺陷，不是单纯表达问题。用户本轮任务失败，同时还看到了不该暴露的底层报错与协议细节。
- 这会损害用户对系统稳定性和专业性的信任，也会暴露内部实现形态，如 `tool_call_id` 与 provider 参数校验细节。
- 问题出现在用户主对话链路，且任何未被专门改写的上游错误都可能命中，因此定级为 `P1`。
- 之所以不是 `P3`，是因为这不只是“文案不好”，而是失败链路的错误边界失守，直接把内部错误透传给用户。

## 根因判断

- `HoneError::Llm` 当前默认把上游错误包装成 `LLM 错误: {0}`，没有区分“日志可见文案”和“用户可见文案”。
- `AgentSession` 失败时直接把 `err.to_string()` 填入 `response.error`。
- Feishu 处理器和共享 outbound 适配器都把 `response.error` 直接拼进最终回复，缺少统一的用户态错误净化层。
- 当前只对 `context window exceeds limit` 做了特定友好化改写，其它 provider 错误没有经过同类产品化处理。

## 下一步建议

- 为用户可见错误增加统一净化层，把 `bad_request_error`、`tool_call_id`、`invalid params`、HTTP/ACP 原始报错等全部映射为产品化提示。
- 保留原始错误到日志、诊断事件或受控调试字段，不要继续复用 `response.error` 作为终端展示文本。
- 在 `AgentSession` 或统一 outbound 层补一条回归测试，覆盖“provider 返回任意 `bad_request_error` 时，用户文案不得包含 `bad_request_error` / `tool_call_id` / `invalid params`”。
- 修复时一并复核 Feishu handler 的失败分支和共享 `run_session_with_outbound(...)`，避免只补一个入口导致其它渠道继续泄露。
