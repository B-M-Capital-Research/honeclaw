# Bug: Discord 定时任务在 Answer 阶段返回空/无效回复后，仍被记为成功执行

- **发现时间**: 2026-04-15 17:10 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixing
- **证据来源**:
  - 2026-04-26 09:30-09:33 最新真实 scheduler 样本：
    - `session_id=Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371`
    - `2026-04-26T09:30:00+08:00` 用户消息：`[定时任务触发] 任务名称：每日美股降息概率推送...`
    - `2026-04-26T01:30:24.202004Z` `skill_tool fed_rate_cut_analysis` 已成功执行；`2026-04-26T01:30:49.805258Z` search 记录 `success=true iterations=2 tool_calls=1`
    - `2026-04-26T01:33:05.153879Z` 日志最终记录 `empty successful response persisted as fallback`
    - `2026-04-26T09:33:05.165637+08:00` assistant 最终落库并发送的却是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - 同轮 `MsgFlow/discord done ... success=true ... tools=7(data_fetch,skill_tool,web_search) reply.chars=35`
  - 2026-04-26 09:33 最新调度落库：
    - `run_id=6578`
    - `job_name=每日美股降息概率推送`
    - `executed_at=2026-04-26T09:33:06+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview=这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_discord__direct__483641214445551626`
    - `2026-04-15T16:03:00.978691+08:00` 用户消息：`[定时任务触发] 任务名称：美伊局势午间汇总。请执行以下指令：查询并汇总美伊战争最新进度，重点关注高层缓和信号及石油设施受损情况`
    - `2026-04-15T16:04:47.348239+08:00` 与 `2026-04-15T16:04:47.352346+08:00` 已有两次 `web_search` 工具成功返回
    - `2026-04-15T16:04:47.356221+08:00` assistant 消息长度为 `0`
  - 最近一小时调度落库：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1757`
    - `job_id=j_c9cc9573`
    - `job_name=美伊局势午间汇总`
    - `executed_at=2026-04-15T16:04:47.361039+08:00`
    - `execution_status=completed`
    - `message_send_status=send_failed`
    - `delivered=0`
    - `response_preview` 长度为 `0`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 16:03:51.447` `multi_agent.search.done success=true iterations=2 tool_calls=2`
    - `2026-04-15 16:04:47.336` `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-15 16:04:47.336` `empty reply (stop_reason=end_turn), no stderr captured`
    - `2026-04-15 16:04:47.360` `MsgFlow/discord ... success=true ... reply.chars=0`
  - 对比同一任务历史：
    - 同一 `job_id=j_c9cc9573` 自 `2026-04-05` 以来累计 `11` 次运行，其中 `10` 次 `sent`、本次首次出现 `send_failed`
  - 代码证据：
    - `crates/hone-channels/src/runners/opencode_acp.rs:621-627`
    - `bins/hone-discord/src/scheduler.rs:122-148`
    - `crates/hone-web-api/src/routes/events.rs:139-236`

## 端到端链路

1. Discord 用户的定时任务 `美伊局势午间汇总` 到点触发。
2. 多代理搜索阶段正常完成，`web_search` 已返回两份外部信息。
3. Answer 阶段的 `opencode_acp` 以 `stop_reason=end_turn` 结束，但最终回复内容为空字符串。
4. 会话链路仍把这次执行记为 `success=true` 并持久化空 assistant 消息。
5. 调度落库可能写成 `execution_status=completed`，但用户实际收到的不是任务报告，而是通用 fallback，或者在更早形态下直接 `send_failed` 未送达。

## 期望效果

- 定时任务在搜索阶段已有结果后，Answer 阶段应产出非空最终回复，或者至少明确返回错误而不是空字符串。
- 一旦最终回复为空，链路应把本次执行标记为失败，并记录可用于排障的错误信息。
- 调度台账不应出现“执行完成但实际零字节、零送达”的伪成功记录。

## 当前实现效果

- 最新样本说明，Discord scheduler 当前不再只表现为“空消息后 send_failed”；它已经演变为“空成功被 fallback 止血后仍记 `completed + sent + delivered=1`”。
- `opencode_acp` 日志明确识别到 `empty reply`，但多代理链路和消息流仍继续以 `success=true` 收尾。
- `run_id=6578` 最终把通用 fallback 写进 `response_preview` 并记为成功送达，掩盖了“降息概率推送并未按要求生成”的真实失败。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量波动。用户预期收到的是降息概率分析报告，但最新样本只拿到通用 fallback。
- 问题同时破坏了“调度是否按时执行”的可信度与可观测性，因为运行台账把它记为 `completed`、`sent`、`delivered=1`，容易误导排障。
- 当前证据只覆盖单个 Discord 任务、单次运行，尚未证明存在跨渠道或跨用户大面积扩散，因此定级为 `P2`，而不是 `P1`。

## 根因判断

- `opencode_acp` 已识别到空回复异常，但当前链路没有把“`reply_chars=0`”提升为硬失败。
- Discord scheduler 现在会把空成功止血成非空 fallback，因此不再触发旧的 `send_failed` 保护，但也因此更容易把失败误记成成功送达。
- 结果是 Answer 阶段的空结果、消息流的 `success=true`、调度落库的 `completed + sent` 三者继续割裂，只是用户侧症状从“完全没收到”漂移成“收到通用 fallback”。

## 修复情况（2026-04-16）

- 已通过 `crates/hone-channels/src/agent_session.rs` 的共享空成功判定修复收口：
  - “正文为空但保留搜索阶段工具调用”的结果不再被视为有效成功
  - 重试耗尽后会降级成非空兜底文案，因此 Discord scheduler 不再落到 `response_preview=''` 且 `send_failed` 的空回复静默漏发
- 该修复和 `feishu_direct_empty_reply_false_success.md`、`feishu_scheduler_empty_reply_false_success.md` 共享同一底层根因和回归证明。

## 2026-04-26 状态回退结论

- 最新 `run_id=6578` 证明 Discord scheduler 这条链路已经回归，但表现形态从 `send_failed + 空 response_preview` 变成了 `sent + delivered=1 + 通用 fallback`。
- 这不是新的独立缺陷，而是同一 Answer 空成功根因在 Discord scheduler 上的最新用户可见形态，因此复用原单并把状态从 `Fixed` 调整回 `Fixing`。

## 回归验证

- `cargo test -p hone-channels should_return_runner_result_ -- --nocapture`
- `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session.rs`
