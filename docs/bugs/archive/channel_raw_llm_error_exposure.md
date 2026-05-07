# Bug: 渠道失败分支会把原始 LLM/provider 报错直接发给用户

- **发现时间**: 2026-04-15 21:20 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实回归：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f69970af6b0ef6ce8e233ef0e0cc0bd79`
    - `2026-04-20T22:28:42.874437+08:00` 用户发送：`2`
    - `2026-04-20T22:29:51.998260+08:00` assistant 直接落库：`正在思考中...Falling back from WebSockets to HTTPS transport. unexpected status 403 Forbidden: Unknown error, url: wss://chatgpt.com/backend-api/codex/responses, cf-ray: ... 我把你的“2”解释为...`
    - 这条文本把 WebSocket 回退、`403 Forbidden`、`Unknown error`、`wss://chatgpt.com/backend-api/codex/responses` 以及 `cf-ray` 调试信息直接拼进用户可见答复，而不是收口成产品化错误提示
    - 同会话直到 `2026-04-20T22:36:27.344689+08:00` 用户继续发送 `继续` 后，系统才在 `2026-04-20T22:37:54.546948+08:00` 给出正常的 `WULF` 正式分析，说明上一轮并没有正确完成用户请求
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

## 2026-04-26 修复

- 在 `crates/hone-channels/src/runtime.rs` 扩展 `looks_internal_error_detail(...)`，把 `Falling back from WebSockets to HTTPS transport`、`unexpected status`、`backend-api/codex/responses`、`cf-ray` 等 Codex/OpenAI 传输回退痕迹统一判定为内部错误细节，继续收口成通用失败提示。
- 在 `crates/hone-channels/src/response_finalizer.rs` 增加成功态兜底：如果最终正文混入上述内部传输痕迹，不再把污染文本当作正式回复发送，而是直接降级为现有空回复 fallback，避免“内部错误 + 半成品答案”一并投递给用户。
- 新增回归测试：
  - `crates/hone-channels/src/runtime.rs`：覆盖 transport trace 错误文案收口。
  - `crates/hone-channels/src/agent_session/tests.rs`：覆盖 success=true 但正文混入内部传输痕迹时的 finalizer fallback。

## 2026-04-26 验证

- `cargo test -p hone-channels sanitize_user_visible_output_ -- --nocapture`
- `cargo test -p hone-channels user_visible_error_message_ -- --nocapture`
- `cargo test -p hone-channels finalize_agent_response_suppresses_internal_transport_error_fragments -- --nocapture`

## 期望效果

- 渠道失败分支应向用户返回稳定、产品化的错误提示，例如“处理失败，请稍后再试”或“工具调用失败，已自动重试仍未成功”。
- 底层 provider 名称、`bad_request_error`、`invalid params`、`tool_call_id` 这类内部实现细节不应直接暴露给最终用户。
- 原始错误应保留在日志或受控诊断渠道，用于排障，而不是进入普通对话面板。

## 当前实现效果

- `2026-04-20 22:29` 的最新 Feishu 直聊样本说明，这个缺陷已经以新形态回归：虽然不再是 `bad_request_error: invalid params`，但底层传输回退细节仍会直接拼进用户可见正文。
- 这次暴露出来的不只是泛化的“处理失败”，而是完整的运行时排障细节，包括 `Falling back from WebSockets to HTTPS transport`、`unexpected status 403 Forbidden`、`Unknown error`、具体 `wss://chatgpt.com/backend-api/codex/responses` URL 与 `cf-ray` 追踪信息。
- 同一条 assistant 文本还把这段底层错误和后续的业务答复草稿混在一起，导致用户先看到一段污染文本，随后还要继续追问才能拿到正式的 `WULF` 分析，说明问题已经不只是“错误提示不友好”，而是原始错误直接污染主回复链路。
- 当前 Feishu 直聊失败分支会把 `response.error` 原样截断后直接拼入用户消息。
- 这次真实故障中的错误文本包含 `bad_request_error`、`invalid function arguments json string` 和具体 `tool_call_id`，都属于内部实现细节。
- 最近一小时复现说明泄露文本的具体形态会变化，除了 `invalid function arguments json string` 之外，还会直接把 `tool call result does not follow tool call` 这类协议级报错发给用户。
- 而且这类透传并不要求会话整轮完全失败：`01:10:01` 刚完成一次“表面成功”的回复后，`01:10:05` 与 `01:10:08` 紧接着又落回同类 provider 错误，说明失败链路净化缺口会在同一会话内持续暴露。
- `08:32` 的两条 scheduler 运行进一步证明：不仅直聊失败分支会透传原始错误，scheduler 也会把同样的 `bad_request_error` 直接写进 `response_preview` 并记成 `sent + delivered=1`。
- 该问题不是 `context window exceeds limit` 老缺陷的原样回归；老缺陷只为“上下文超限”补了特判改写，其他 provider 错误仍会直出。
- 同类风险不只存在于 Feishu：共享的 `run_session_with_outbound(...)` 失败路径也会执行 `抱歉，处理失败：{truncate_chars(err, 300)}`，说明这是跨渠道的共性缺口。

## 用户影响

- 这是功能性缺陷，不是单纯表达问题。用户本轮任务失败，同时还看到了不该暴露的底层报错与协议细节。
- `2026-04-20 22:29` 的回归样本表明，即使后续模型还能继续产出部分业务文本，只要原始传输报错被混入最终回复，用户侧看到的仍是被污染的正式答复，产品边界已经失守。
- 这会损害用户对系统稳定性和专业性的信任，也会暴露内部实现形态，如 `tool_call_id` 与 provider 参数校验细节。
- 问题出现在用户主对话链路，且任何未被专门改写的上游错误都可能命中，因此定级为 `P1`。
- 之所以不是 `P3`，是因为这不只是“文案不好”，而是失败链路的错误边界失守，直接把内部错误透传给用户。

## 根因判断

- 最新回归说明，`user_visible_error_message(...)` 虽然覆盖了 `bad_request_error`、`invalid params`、`tool_call_id` 等既有模式，但并没有覆盖 `Falling back from WebSockets to HTTPS transport`、`unexpected status 403 Forbidden`、`Unknown error`、`cf-ray` 这类 OpenAI/Codex 传输回退细节。
- 同时，当前链路没有阻断“内部错误片段 + 后续答复草稿”被拼接成一条用户可见消息的情况，导致即便不是纯失败文案，也会把底层传输排障文本夹带进最终回复。
- `HoneError::Llm` 当前默认把上游错误包装成 `LLM 错误: {0}`，没有区分“日志可见文案”和“用户可见文案”。
- `AgentSession` 失败时直接把 `err.to_string()` 填入 `response.error`。
- Feishu 处理器和共享 outbound 适配器都把 `response.error` 直接拼进最终回复，缺少统一的用户态错误净化层。
- 当前只对 `context window exceeds limit` 做了特定友好化改写，其它 provider 错误没有经过同类产品化处理。

## 修复情况（2026-04-16，现已回归）

- 已在 `crates/hone-channels/src/runtime.rs` 增加共享 `user_visible_error_message(...)`，统一把超时错误映射为超时提示，把 `bad_request_error`、`invalid params`、`tool_call_id`、`session/prompt` 等内部/provider 细节收口为稳定的用户可见文案。
- 已把该 helper 接入以下用户可见失败分支：
  - `crates/hone-channels/src/outbound.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `bins/hone-feishu/src/handler.rs`
  - `bins/hone-discord/src/handlers.rs`
  - `bins/hone-imessage/src/main.rs`
- 原始错误仍保留在日志与运行诊断路径里用于排障，但不再直接拼进用户回复，也不再作为 scheduler 失败投递正文下发给用户。
- 但 `2026-04-20 22:29` 的 Feishu 真实会话说明，传输回退类报错仍未被这套净化规则覆盖，因此本缺陷不能继续维持 `Fixed`。

## 回归验证

- `cargo test -p hone-channels user_visible_error_message_ -- --nocapture`
- `cargo check -p hone-feishu -p hone-discord -p hone-imessage -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/runtime.rs crates/hone-channels/src/outbound.rs crates/hone-channels/src/scheduler.rs bins/hone-feishu/src/handler.rs bins/hone-discord/src/handlers.rs bins/hone-imessage/src/main.rs`

## 2026-04-23 巡检结论

- 本轮开始时工作区存在一组未提交的代码/版本号/release note 改动，其中包含疑似针对 Codex WebSocket/HTTPS 传输残留的 sanitizer 补丁；该补丁不属于本次缺陷台账维护边界，已按自动化规则恢复。
- 恢复后 `crates/hone-channels/src/runtime.rs` 中没有 `Falling back from WebSockets`、`backend-api/codex/responses`、`cf-ray`、`unexpected status 403 Forbidden` 等识别规则，不能用未提交脏代码支撑本缺陷 `Fixed`。
- 最近一小时未再观察到同类用户可见外泄样本，但已知 2026-04-20 22:29 真实会话证据仍成立；状态维持 `New`，等待正式代码修复后再切到 `Fixed`。
