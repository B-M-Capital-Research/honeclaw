# Bug: Feishu 直聊在工具链尚未结束时提前持久化短答，导致用户只收到过渡性半成品回复

- 发现时间：2026-04-16 16:12 CST
- Bug Type：Business Error
- 严重等级：P3
- 状态：New

## 证据来源

- 会话库：
  - `data/sessions.sqlite3`
  - session_id：`Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
- 会话快照：
  - `data/sessions/Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15.json`
- 运行日志：
  - `data/runtime/logs/web.log`

## 端到端链路

1. 2026-04-16 16:00:09 CST，Feishu 直聊用户发送：“你帮我做一下组合风险的评估，然后再做一下核心持仓的评估”。
2. 日志显示这条消息正常经过：
   - `step=message.accepted`
   - `step=reply.placeholder`
   - `step=session.persist_user`
   - `step=agent.prepare`
   - `step=agent.run`
3. 同一轮里执行了大量工具调用：
   - 多次 `ToolRegistry tool_execute_start/tool_execute_success name=data_fetch`
   - 两次 `tool_execute_success name=web_search`
   - `16:00:26.505` 还出现一次 `runner.stage=acp.tool_failed`
4. 但在 `2026-04-16 16:01:05.377`，系统已经先写入并完成：
   - `step=session.persist_assistant detail=done`
   - `done ... success=true ... reply.chars=55`
5. 对应落库的最终 assistant 文本只有一句：
   - “美股行情已经拿到。港股代码格式在底层数据里没直接回出，我改用搜索补腾讯和小米的现价，同时补核心持仓新闻和催化。”
6. 关键异常在于：`session.persist_assistant` 与 `done` 之后的同一时间点，日志仍在启动新工具：
   - `2026-04-16 16:01:05.369` `runner.tool ... Tool: hone/web_search status=start`
7. 用户最终只收到这句过渡性说明，没有继续收到本应完成的“组合风险评估 + 核心持仓评估”正式分析。

## 期望效果

- 当用户明确要求“组合风险评估 + 核心持仓评估”时，系统应在工具完成后输出完整分析，而不是只返回“我正在补数据”的过程性说明。
- 只有在 answer 阶段真正结束且不再继续调工具时，才应执行 `session.persist_assistant`、`done` 与 `reply.send`。
- 若工具失败导致无法完成完整分析，也应给出明确的失败说明或降级结论，而不是伪装成已完成的短答。

## 当前实现效果

- 该轮会话主链路没有中断：消息被接收、持久化、执行并送达。
- 但最终送达内容只是一个过程性过渡句，既没有组合风险评估，也没有核心持仓评估。
- 日志还显示 assistant 已经被持久化并标记 `done` 后，新的 `hone/web_search` 工具才刚开始，说明 answer 收口时序与工具执行状态不一致。
- 这会让用户误以为系统已经完成回答，实际却只拿到半成品。

## 用户影响

- 这是质量性 bug。用户收到了回复，但回复没有完成其明确提出的双重分析任务。
- 之所以定级为 `P3`，是因为消息投递、会话持久化和基础工具链均已工作，没有出现错投、无回复、数据损坏或系统级失败。
- 问题主要体现在 answer 结果的完整性和收口质量，而不是主功能链路完全不可用。

## 根因判断

- 高概率是 answer 阶段的“最终可见文本”在工具链尚未真正收敛时被过早认定为最终结果，导致过程性草稿提前落库并发送。
- `16:01:05` 的日志顺序显示 `session.persist_assistant/done` 与新的 `hone/web_search start` 交错出现，这意味着：
  - 要么 runner/adapter 过早消费了一个中间文本块并把它视为 final；
  - 要么存在工具失败后的提前收口分支，没有等待后续补充分析完成。
- 目前证据更偏向 answer 收口时序异常，而不是单纯模型“答得短”；因为日志里还能看到持久化完成后继续启动工具。

## 下一步建议

- 优先排查 Feishu 直聊 answer 出站链路如何判定“final 可发送文本”，确认是否会把中间进度句提前视为最终答复。
- 对 `session.persist_assistant` 与 `reply.send` 增加约束：若仍存在未完成工具调用或新的 tool start 事件，不应提前结束本轮回答。
- 为这类“短答但 success=true”场景补质量巡检信号，例如：
  - 用户请求明显需要分析型长答
  - 但 `reply.chars` 极短
  - 且 `done` 前后仍有工具事件
- 回归验证时应覆盖：
  - 用户请求组合评估 + 个股评估
  - answer 阶段包含多次 `data_fetch/web_search`
  - 断言最终发送内容不是过程性说明句，且发送完成后不再出现新的工具启动
