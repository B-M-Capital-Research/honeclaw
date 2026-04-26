# Bug: Feishu 定时任务在 Answer 阶段返回空/无效回复后，调度台账仍记为 `completed + sent`

- **发现时间**: 2026-04-15 21:08 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 2026-04-26 08:01 最新真实 scheduler 样本：
    - `session_id=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
    - `2026-04-26T08:01:00.036629+08:00` 用户消息：`[定时任务触发] 任务名称：HoneClaw每日使用Tips...`
    - `2026-04-26T08:01:15.998784+08:00` `multi_agent.search.done success=true iterations=1 tool_calls=0`
    - `2026-04-26T08:01:19.622891+08:00` 首轮 answer 再次记录 `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-26T08:01:29.639678+08:00` 第二轮 answer 再次记录 `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-26T08:01:41.331255+08:00` 日志改为 `empty successful response persisted as fallback`，`step=agent.run.fallback ... detail=empty_success_exhausted`
    - `2026-04-26T08:01:41.334034+08:00` assistant 最终落库并发送的仍是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
  - 2026-04-26 08:01 最新调度落库：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6539`，`job_name=HoneClaw每日使用Tips`，`executed_at=2026-04-26T08:01:45+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`
    - `response_preview` 为通用 fallback，而不是用户要求的约 30 字使用技巧；说明虽然零字节正文已被 fallback 止血，但 scheduler 仍把 Answer 失败收口成表面成功投递
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-04-15T20:46:45.280417+08:00` 用户消息：`[定时任务触发] 任务名称：美股盘前AI及高景气产业链推演...`
    - `2026-04-15T20:50:03.111837+08:00` assistant 消息长度为 `0`
  - 最近一小时调度落库：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1789`，`job_id=j_856e457e`，`job_name=美股盘前AI及高景气产业链推演`，`executed_at=2026-04-15T20:50:05.138992+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`response_preview` 长度为 `0`
    - `run_id=1788`，`job_id=j_ac4a9736`，`job_name=美股AI产业链盘前报告`，`executed_at=2026-04-15T20:46:47.478227+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`response_preview` 长度为 `0`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 20:50:03.060` `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-15 20:50:03.060` `empty reply (stop_reason=end_turn), no stderr captured`
    - `2026-04-15 20:50:03.060` `multi_agent.answer.done success=true`
    - `2026-04-15 20:50:03.119` `MsgFlow/feishu ... success=true ... reply.chars=0`
    - `2026-04-15 20:38:15.518` 另一条 Feishu 会话也记录到 `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-15 20:38:18.604` 同轮仍执行 `step=reply.send ... detail=segments.sent=1/1`
  - 相关已知缺陷：
    - `docs/bugs/feishu_direct_empty_reply_false_success.md`
    - `docs/bugs/discord_scheduler_empty_reply_send_failed.md`

## 端到端链路

1. Feishu 用户的定时任务到点触发，调度器把任务正文作为 `[定时任务触发]` 用户消息注入会话。
2. Multi-Agent 搜索阶段正常完成，已经拿到 `data_fetch` / `web_search` 结果。
3. Answer 阶段的 `opencode_acp` 以 `stop_reason=end_turn` 结束，但最终回复为空字符串。
4. 多代理链路仍把本轮记为 `success=true`，消息流继续持久化空 assistant 消息。
5. 调度落库最终写成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`，但用户侧拿到的要么是空白消息，要么只是通用 fallback，真实定时任务内容没有完成。

## 期望效果

- Feishu 定时任务在搜索阶段已有结果后，应产出与任务目标一致的最终回复，或者至少显式失败，而不是落成空字符串或通用 fallback。
- 一旦 Answer 阶段出现 `reply_chars=0` / `empty_success_exhausted`，调度链路应停止把本轮记为成功投递，并把本次运行标记为失败或可重试状态。
- `cron_job_runs` 不应出现“`completed + sent + delivered=1`，但实际只发出空白内容或通用 fallback”的伪成功记录。

## 当前实现效果

- `2026-04-26 08:01` 的 `HoneClaw每日使用Tips` 最新样本说明，用户侧零字节正文虽然已被非空 fallback 止血，但 Answer 阶段的空成功根因仍活跃，且 scheduler 仍把本轮记成 `completed + sent + delivered=1`。
- `j_856e457e` 等旧样本可见原始坏态：任务正文存在、搜索工具已执行、最终 assistant 仍为零字节。
- `sidecar.log` / `web.log` 明确识别到 `empty reply` 与 `empty successful response persisted as fallback`，但 `multi_agent.answer.done`、`MsgFlow/feishu done` 和调度落库仍全部走成功路径。
- 与 `discord_scheduler_empty_reply_send_failed` 不同，Feishu scheduler 这条链路在最新样本里甚至把通用 fallback 也记成了任务完成，进一步掩盖“日报/提示未按要求生成”的真实失败。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量波动。用户预期收到的是具体定时播报内容，但最新样本只拿到通用失败 fallback，任务仍等同未完成。
- 调度台账同时把本轮记为已执行、已发送、已送达，会误导人工和后续 agent 以为任务正常工作。
- 最新样本说明问题已经从“空白消息直接外发”演变为“空成功被 fallback 遮蔽后仍计成功”；这仍直接破坏定时任务主功能链路，因此继续维持 `P1`。
- 之所以不是 `P3`，是因为问题并非“内容不够好”，而是定时任务没有交付用户要求的结果，只是由产品化 fallback 暂时掩盖。

## 根因判断

- `opencode_acp` 已能识别 `reply_chars=0` 和 `empty reply`，但当前返回值没有把这类情况提升为硬失败。
- 多代理封装层继续把空结果记为 `answer.done success=true`，导致 Feishu scheduler 上层无法区分“正常完成”和“零字节完成”。
- Feishu 调度落库与发送侧只看流程是否走完，没有校验最终正文是否真正满足任务目标，于是把空结果或 fallback 都记成 `completed + sent`。
- 该问题与 `feishu_direct_empty_reply_false_success` 共享底层空回复判定缺口，但这里是独立的 scheduler 投递链路，影响范围和错误台账形态不同，需单独跟踪。

## 修复情况（2026-04-16）

- 已通过 `crates/hone-channels/src/agent_session.rs` 的共享空成功判定修复收口：
  - 搜索阶段遗留的 `tool_calls_made` 不再让空 answer 被视为有效成功
  - 重试耗尽后会返回非空兜底文案，而不是继续让 scheduler 发送零字节正文
- 因为 Feishu scheduler 复用同一 `AgentSession` / multi-agent 成功判定链路，`response_preview` 与最终投递内容不再出现空字符串的 `completed + sent + delivered=1` 伪成功。

## 修复结论复核（2026-04-26 08:01 CST）

- 最新真实 scheduler 样本 `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 已证明，上述“已修复”结论不能成立为彻底收口：
  - `HoneClaw每日使用Tips` 连续两次 answer 都以 `reply_chars=0` 结束；
  - 链路最终只靠 `EMPTY_SUCCESS_FALLBACK_MESSAGE` 对用户止血；
  - `cron_job_runs.run_id=6539` 仍被记为 `completed + sent + delivered=1`。
- 这说明 2026-04-16 的修补只解决了“不要发送零字节正文”，没有解决“scheduler 把空成功 fallback 误记为任务成功完成”。
- 因此本单状态从 `Fixed` 调整回 `Fixing`：用户侧空白消息症状已缓解，但 scheduler 主功能链路仍会在真实生产任务中退化成通用 fallback，并被台账伪装成成功。

## 下一步建议

- 把 scheduler 成功判定从“最终正文非空”进一步收紧到“最终正文不是统一 fallback，且满足任务最小产出要求”；至少对 `empty_success_exhausted` 这类收口不要继续记 `completed + sent`。
- 继续把 `reply_chars=0`、`empty successful response persisted as fallback`、`cron_job_runs.response_preview=通用 fallback` 的组合视为当前主监控信号，而不只盯零字节消息。
- 回归至少覆盖 `HoneClaw每日使用Tips` 这类无工具、短文本 scheduler 任务：出现空 answer 时要么自动补出合格 tip，要么明确记失败并重试，而不是向用户发送通用失败文案。

## 回归验证

- `cargo test -p hone-channels should_return_runner_result_ -- --nocapture`
- `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session.rs`
