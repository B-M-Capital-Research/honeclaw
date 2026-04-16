# Bug: 渠道失败分支会把原始 LLM/provider 报错直接发给用户

- **发现时间**: 2026-04-15 21:20 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-15T20:58:51.471633+08:00` 用户消息：`最近存储系列的还能买吗`
    - `2026-04-15T21:00:30.520799+08:00` 用户追问：`咋回事`
    - 说明失败发生后，用户没有拿到正常回答，只能立即追问原因；失败文案没有作为正常 assistant 消息入库
  - 最近一小时再次复现：
    - 同一 `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16T00:05:57.926067+08:00` 用户发送“我给你四个截图你帮我记录下我的持仓情况 也就是一鸣的持仓情况 并遵守保密义务哈”
    - `2026-04-16T00:06:45.464163+08:00` 用户追问：`咋出错了`
    - `2026-04-16T00:07:19.161517+08:00`、`00:07:21.549084+08:00`、`00:07:25.811515+08:00` 用户继续补发图片附件，但本轮仍未获得正常答复
    - `2026-04-16T01:10:05.517752+08:00` 与 `2026-04-16T01:10:08.492007+08:00` 同会话在上一轮“成功回复”后又紧接着再次失败，说明原始 provider 报错暴露并不只发生在 `00:05-00:07` 那一轮
  - 最近运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 20:58:58.152` `MsgFlow/feishu failed ... error="LLM 错误: bad_request_error: invalid params, invalid function arguments json string, tool_call_id: call_function_v0wwk1qhh65v_1 (2013)"`
    - 同一会话随后在 `21:00:30` 重试，最终于 `21:04:05` 恢复出正常回答
    - `2026-04-16 00:05:57.918` `MsgFlow/feishu failed ... error="LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)"`
    - `2026-04-16 00:06:00.133`、`00:06:48.002`、`00:07:21.540`、`00:07:25.806` 同会话持续复现相同错误，说明并非单次抖动
    - `2026-04-16 01:10:05.509` 与 `01:10:08.485` 同会话再次记录 `MsgFlow/feishu failed ... error="LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)"`
  - 2026-04-16 08:32 scheduler 再次复现：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1839`，`job_name=每日美股收盘与持仓早报`，`executed_at=2026-04-16T08:32:01.996117+08:00`
    - `execution_status=execution_failed`，`message_send_status=sent`，`delivered=1`
    - `response_preview` 与 `error_message` 都直接等于 `LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)`
    - `run_id=1840`，`job_name=每日持仓分析早报`，`executed_at=2026-04-16T08:32:05.416666+08:00`，同样以原始 provider 报错作为已发送内容落库
  - 更完整的渠道日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-15T12:58:58.152238Z` 同样记录 `error="LLM 错误: bad_request_error: invalid params, invalid function arguments json string, tool_call_id: call_function_v0wwk1qhh65v_1 (2013)"`
    - `2026-04-15T16:05:57.918Z` 到 `2026-04-15T16:07:25.806Z` 同样反复记录 `error="LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)"`
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
- 最近一小时复现说明泄露文本的具体形态会变化，除了 `invalid function arguments json string` 之外，还会直接把 `tool call result does not follow tool call` 这类协议级报错发给用户。
- 而且这类透传并不要求会话整轮完全失败：`01:10:01` 刚完成一次“表面成功”的回复后，`01:10:05` 与 `01:10:08` 紧接着又落回同类 provider 错误，说明失败链路净化缺口会在同一会话内持续暴露。
- `08:32` 的两条 scheduler 运行进一步证明：不仅直聊失败分支会透传原始错误，scheduler 也会把同样的 `bad_request_error` 直接写进 `response_preview` 并记成 `sent + delivered=1`。
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

## 修复情况（2026-04-16）

- 已在 `crates/hone-channels/src/runtime.rs` 增加共享 `user_visible_error_message(...)`，统一把超时错误映射为超时提示，把 `bad_request_error`、`invalid params`、`tool_call_id`、`session/prompt` 等内部/provider 细节收口为稳定的用户可见文案。
- 已把该 helper 接入以下用户可见失败分支：
  - `crates/hone-channels/src/outbound.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `bins/hone-feishu/src/handler.rs`
  - `bins/hone-discord/src/handlers.rs`
  - `bins/hone-imessage/src/main.rs`
- 原始错误仍保留在日志与运行诊断路径里用于排障，但不再直接拼进用户回复，也不再作为 scheduler 失败投递正文下发给用户。

## 回归验证

- `cargo test -p hone-channels user_visible_error_message_ -- --nocapture`
- `cargo check -p hone-feishu -p hone-discord -p hone-imessage -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/runtime.rs crates/hone-channels/src/outbound.rs crates/hone-channels/src/scheduler.rs bins/hone-feishu/src/handler.rs bins/hone-discord/src/handlers.rs bins/hone-imessage/src/main.rs`
