# Bug: Feishu 图片附件会向用户发送内部 skill transcript，并夹带未清洗的中间协议

- **发现时间**: 2026-04-16 01:10 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16T00:01:41.401902+08:00` 用户先上传单张图片附件
    - `2026-04-16T00:05:53.111901+08:00` assistant 落库消息不是纯文本答案，而是同一条消息内混入 `progress`、`tool_call`、`tool_result`、`final`
    - 该 assistant 内容直接包含 `<think>`、`skill_tool(skill_name=\"image_understanding\")` 的参数、完整 skill prompt 展开结果，以及最终图片分析正文
    - `2026-04-16T00:05:57.926067+08:00` 到 `2026-04-16T00:07:25.811515+08:00` 用户继续发送“我给你四个截图你帮我记录下我的持仓情况”与后续 3 张图片，但系统未能回到正常持仓识别链路
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 00:05:54.042` `step=reply.send ... detail=segments.sent=1/1`
    - `2026-04-16 00:05:55.267` 同会话随即出现 `multi_agent.search.done success=false`
    - `2026-04-16 00:05:55.268` 错误为 `bad_request_error: invalid params, tool call result does not follow tool call (2013)`
    - `2026-04-16 00:05:57.918`、`00:06:00.133`、`00:06:48.002`、`00:07:21.540`、`00:07:25.806` 同一会话连续重复相同失败
  - 相关代码位置：
    - `crates/hone-channels/src/outbound.rs:151-160`
    - `crates/hone-channels/src/agent_session.rs:1466-1531`
    - `crates/hone-channels/src/runners/opencode_acp.rs:511-516`
  - 相关历史缺陷：
    - `docs/bugs/multi_agent_internal_output_leak.md`
    - `docs/bugs/channel_raw_llm_error_exposure.md`

## 端到端链路

1. Feishu 直聊用户先上传一张图片附件，系统按默认附件策略触发图片理解流程。
2. 用户侧随后实际收到一条 assistant 回复；该回复不是纯净答案，而是把 `progress`、`tool_call`、`tool_result` 和最终正文一起发了出去。
3. 落库消息显示 `tool_result` 中还携带了完整 `image_understanding` skill prompt 展开内容，包括工具说明、路径和内部 reminder。
4. 用户继续补充“我给你四个截图你帮我记录下我的持仓情况”及剩余图片后，链路没有正确进入新的持仓识别任务，而是在 `00:05:55` 到 `00:07:28` 连续多次因 `tool call result does not follow tool call` 失败。
5. 最终用户既看到了不该暴露的内部协议文本，也没拿到本轮“记录四张持仓截图”的正常结果。

## 期望效果

- 图片附件链路对用户只应发送最终可见答案，不应把 `<think>`、`tool_call`、`tool_result`、skill prompt 展开文本或内部 reminder 发给用户。
- 即便附件触发了默认技能，用户侧也只能看到经过净化后的最终结论，不能看到技能装载过程和工具参数。
- 当用户继续补发说明与多张附件时，会话应平滑进入新的任务目标，而不是在内部协议污染后持续报错。

## 当前实现效果

- 当前真实会话已证明：Feishu 图片附件链路会把内部协议片段和最终正文混在同一条 assistant 消息里落库并发给用户。
- 泄露内容不只是一小段 `<think>`，还包含完整 `skill_tool` 调用参数、`tool_result` 结构体和 `image_understanding` 技能全文展开。
- 同一时间窗内，用户补充“帮我记录四个截图里的持仓情况”后，链路连续 5 次失败，没有产出新的正常回答。
- 这说明问题不只是“格式不够好”，而是用户可见输出边界失守，并且后续主任务也被打断。

## 用户影响

- 这是功能性缺陷，不是单纯表达质量问题。用户看到了系统内部协议与技能中间稿，同时未能完成“记录持仓截图”的实际任务。
- 问题发生在 Feishu 直聊主链路，并且涉及用户上传的图片与本地路径、工具参数、技能内部说明等敏感实现细节，因此定级为 `P1`。
- 之所以不是 `P3`，是因为它已经影响到主功能链路完成度、错误边界和内部实现暴露，而不是仅仅“答案不够好”。

## 根因判断

- 当前用户可见输出净化对常规文本答案有效，但这条链路表明：图片附件触发的 skill transcript / tool transcript 仍可能在某个发送路径上被当作正式 assistant 内容发送出去。
- `run_session_with_outbound(...)` 在 `response.success` 时会直接发送 `response.content`，说明如果上游把混合了协议片段的内容标成成功，这一层不会再次阻断。
- `opencode_acp` 侧虽然已注明不能回放旧会话 chunk，但本轮现象说明附件/技能链路仍存在“内部 chunk 或 transcript 被拼进最终回复”的独立缺口。
- 同轮随后反复出现 `tool call result does not follow tool call`，说明协议污染很可能进一步破坏了后续消息序列，导致会话无法恢复到正常的图片处理路径。

## 下一步建议

- 排查图片附件默认技能链路在 Feishu 直聊中的最终出站文本拼装，确认 `progress`、`tool_call`、`tool_result` 为什么仍能穿透到 `response.content`。
- 为图片/技能链路补一条回归测试，覆盖“上传图片附件时最终发送内容不得包含 `<think>` / `tool_call` / `tool_result` / skill prompt 展开文本”。
- 排查同会话后续连续触发 `tool call result does not follow tool call` 的消息序列构造，确认是否由这次泄露的 transcript 污染了历史上下文。
- 修复时联动复核 `docs/bugs/channel_raw_llm_error_exposure.md`，避免只拦住 transcript 泄露但继续把底层 provider 报错直发给用户。
