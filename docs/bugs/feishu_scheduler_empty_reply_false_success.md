# Bug: Feishu 定时任务在 Answer 阶段返回空回复后，调度台账仍记为 `completed + sent`

- **发现时间**: 2026-04-15 21:08 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
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
5. 调度落库最终写成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`，但 `response_preview` 为空，用户侧实际收到的是空白定时消息。

## 期望效果

- Feishu 定时任务在搜索阶段已有结果后，应产出非空最终回复，或者至少显式失败，而不是返回空字符串。
- 一旦 Answer 阶段出现 `reply_chars=0`，调度链路应停止投递，并把本次运行标记为失败或可重试状态。
- `cron_job_runs` 不应出现“`completed + sent + delivered=1`，但 `response_preview` 为空”的伪成功记录。

## 当前实现效果

- 最近一小时内至少两条 Feishu 定时任务运行已经落成 `completed + sent`，但 `response_preview` 长度都为 `0`。
- 其中 `j_856e457e` 可在真实会话中看到：任务正文存在、搜索工具已执行、最终 assistant 仍为零字节。
- `web.log` 明确识别到 `empty reply`，但 `multi_agent.answer.done`、`MsgFlow/feishu done` 和调度落库仍全部走成功路径。
- 与 `discord_scheduler_empty_reply_send_failed` 不同，Feishu scheduler 这条链路甚至把空回复记成了 `sent` 与 `delivered=1`，可观测性更差。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量波动。用户预期收到一条完整定时播报，但实际收到的是空消息，任务等同未完成。
- 调度台账同时把本轮记为已执行、已发送、已送达，会误导人工和后续 agent 以为任务正常工作。
- 最近一小时已在两条 Feishu 定时任务上复现，不再是单个边缘样本，因此定级为 `P1`。
- 之所以不是 `P3`，是因为问题并非“内容不够好”，而是最终根本没有可消费内容，直接破坏定时任务主功能链路。

## 根因判断

- `opencode_acp` 已能识别 `reply_chars=0` 和 `empty reply`，但当前返回值没有把这类情况提升为硬失败。
- 多代理封装层继续把空结果记为 `answer.done success=true`，导致 Feishu scheduler 上层无法区分“正常完成”和“零字节完成”。
- Feishu 调度落库与发送侧只看流程是否走完，没有校验最终正文非空，于是把空结果记成 `completed + sent`。
- 该问题与 `feishu_direct_empty_reply_false_success` 共享底层空回复判定缺口，但这里是独立的 scheduler 投递链路，影响范围和错误台账形态不同，需单独跟踪。

## 修复情况（2026-04-16）

- 已通过 `crates/hone-channels/src/agent_session.rs` 的共享空成功判定修复收口：
  - 搜索阶段遗留的 `tool_calls_made` 不再让空 answer 被视为有效成功
  - 重试耗尽后会返回非空兜底文案，而不是继续让 scheduler 发送零字节正文
- 因为 Feishu scheduler 复用同一 `AgentSession` / multi-agent 成功判定链路，`response_preview` 与最终投递内容不再出现空字符串的 `completed + sent + delivered=1` 伪成功。

## 回归验证

- `cargo test -p hone-channels should_return_runner_result_ -- --nocapture`
- `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session.rs`
