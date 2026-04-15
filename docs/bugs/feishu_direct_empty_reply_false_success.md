# Bug: Feishu 直聊会话在 Multi-Agent Answer 阶段返回空回复后，链路仍记成功并发送空消息

- **发现时间**: 2026-04-15 18:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
    - `2026-04-15T17:45:15.399804+08:00` 用户消息要求比较 `AT&T`、`T-Mobile`、`VSAT`、`IRDM`、`GSAT`、`ASTS` 在 2025 年财报中的手机通信收入，并输出投资报告格式的表格和文字分析
    - `2026-04-15T17:49:05.656906+08:00` 到 `2026-04-15T17:49:05.706807+08:00` 连续 12 次 `data_fetch` 工具成功返回
    - `2026-04-15T17:49:05.708643+08:00` assistant 消息长度为 `0`
    - 该空 assistant 消息仍落库了真实 `message_id=om_x100b52c1aca3f51cc3d6e91f9c1817a`
  - 最近一小时运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-15T09:48:56.045195Z` `multi_agent.search.done success=true iterations=3 tool_calls=12`
    - `2026-04-15T09:49:05.651091Z` `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-15T09:49:05.651145Z` `empty reply (stop_reason=end_turn), no stderr captured`
    - `2026-04-15T09:49:05.651283Z` `multi_agent.answer.done success=true`
    - `2026-04-15T09:49:05.712429Z` `MsgFlow/feishu ... success=true ... reply.chars=0`
    - `2026-04-15T09:49:08.365202Z` `step=reply.send ... detail=segments.sent=1/1`
  - 会话汇总记录：`data/sessions.sqlite3` -> `sessions`
    - `updated_at=2026-04-15T17:49:05.708644+08:00`
    - `last_message_at=2026-04-15T17:49:05.708643+08:00`
    - `last_message_preview` 为空
  - 对比同一时间窗异常样式：
    - 最近 90 分钟内仅发现两条空 assistant 消息，另一条是已单独登记的 `discord_scheduler_empty_reply_send_failed`
    - 本次是 Feishu 直聊、非 scheduler、用户主动提问链路，影响范围与已有 Discord 定时任务缺陷不同

## 端到端链路

1. Feishu 用户在直聊中发起投研分析请求，要求输出表格和成文报告。
2. Multi-Agent 搜索阶段完成 12 次 `data_fetch`，上下文材料已经齐备。
3. Answer 阶段的 `opencode_acp` 以 `stop_reason=end_turn` 结束，但最终回复为空字符串。
4. 多代理链路仍把本轮 Answer 记为 `success=true`，随后主消息流继续持久化空 assistant 消息。
5. Feishu 发送链路继续执行 `reply.send segments.sent=1/1`，用户侧实际收到的是空消息，本轮任务等同未完成。

## 期望效果

- 用户主动提问的直聊会话在工具阶段成功后，应产出非空最终答复，至少满足基本可读性与结构要求。
- 一旦 Answer 阶段返回空字符串，链路应中止发送并把本轮执行明确标记为失败，而不是继续写入成功状态。
- 日志和落库记录应保留足够的错误摘要，避免出现“已成功发送但没有任何内容”的伪成功。

## 当前实现效果

- 真实会话已经证明：Feishu 直聊在搜索结果齐备的前提下，仍可能产出零字节 assistant 消息。
- `opencode_acp` 日志明确识别到 `empty reply`，但 `multi_agent.answer.done`、`MsgFlow/feishu done` 和 `reply.send` 仍全部走成功路径。
- 数据库最终同时留下“有真实消息 ID”和“assistant 内容为空”这两个互相矛盾的结果，说明空消息并未被链路拦截。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量波动。用户明确要求的投研报告完全没有返回，任务实际失败。
- 问题发生在 Feishu 直聊主链路，而不是边缘后台任务，直接影响用户能否完成一次正常问答，因此定级为 `P1`。
- 该问题不属于 `P3` 质量类问题，因为它不是“答得不够好”，而是最终根本没有可消费内容。

## 根因判断

- `opencode_acp` 能识别 `reply_chars=0` 和 `empty reply`，但当前没有把这类结果升级为硬失败。
- 多代理封装层把空回复继续当作 `answer.done success=true`，导致上层消息流无法区分“正常完成”和“零字节完成”。
- Feishu 发送侧只看分段流程是否跑完，没有拦截空正文，因此把空 assistant 消息照常投递。

## 下一步建议

- 把 `reply_chars=0` / `empty reply` 统一升级为 Answer 阶段失败，禁止继续进入 `success=true` 和 `reply.send`。
- 为直聊消息流补充“空正文不可发送”的最终保护，至少在发送前兜底失败并回填错误摘要。
- 复核 `multi_agent` 对 `opencode_acp` 返回值的成功判定逻辑，确保 Feishu 直聊与 Discord scheduler 不再各自静默吞掉同类空回复。
