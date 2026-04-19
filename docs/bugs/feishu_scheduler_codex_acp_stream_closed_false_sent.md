# Bug: Feishu 每日动态监控遇到 `codex acp stream closed before response` 后台账仍记为 sent

- **发现时间**: 2026-04-20 01:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
    - `2026-04-20T00:02:39.878403+08:00` 调度触发 `RKLB 每日动态监控`
    - 巡检时同一 session 在该时间点之后没有新增任何 `role=assistant` 正文；最近一条 assistant 仍停留在上一轮 `TEM 每日动态监控` 的 `2026-04-20T00:02:39.875867+08:00`
    - 说明本轮 RKLB 任务没有形成用户可见正常答复
  - 最近一小时调度台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3168`
    - `job_name=RKLB 每日动态监控`
    - `executed_at=2026-04-20T00:02:42.128538+08:00`
    - `execution_status=execution_failed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview=codex acp stream closed before response`
    - `error_message=codex acp stream closed before response`
    - 这说明台账把底层 runner 错误文本直接当作“已发送”结果记账
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-20 00:02:39.879` `recv ... input.preview="[定时任务触发] 任务名称：RKLB 每日动态监控..."`
    - `2026-04-20 00:02:40.001` `step=agent.run ... detail=start`
    - `2026-04-20 00:02:41.117` `WARN  [MsgFlow/feishu] runner.error ... message="codex acp stream closed before response"`
    - `2026-04-20 00:02:41.117` `ERROR [MsgFlow/feishu] failed ... error="codex acp stream closed before response"`
    - 同轮没有出现 `session.persist_assistant`、`reply.send` 或 `done ... success=true`
    - 说明真实链路是在 runner 提前断流后直接失败退出，而不是完成了一次受控错误发送
  - 相关已有缺陷对照：
    - [`feishu_direct_cron_job_iteration_exhaustion_no_reply.md`](./feishu_direct_cron_job_iteration_exhaustion_no_reply.md) 关注的是 search 触顶 `已达最大迭代次数 8` 后无回复/错误外泄。
    - 本次样本的失败根因是 `codex acp stream closed before response`，且发生在单条 `RKLB 每日动态监控` 上，属于新的 runner 断流路径，而非同一 search 触顶根因。

## 端到端链路

1. Feishu 直达定时任务触发 `RKLB 每日动态监控`。
2. agent 刚进入 `agent.run`，`codex acp` 就在正式答复前断流退出。
3. 运行日志将本轮记为 `failed`，且没有产生 assistant 正文或发送日志。
4. 但调度台账却把同一轮记成 `execution_failed + sent + delivered=1`，并把底层错误字符串写入 `response_preview`。

## 期望效果

- 若 `codex acp` 在正式答复前断流，应生成明确的用户态失败文案并准确记录发送结果。
- 若实际上没有发送任何可见消息，台账应保持 `send_failed` 或 `skipped_error`，而不是记为 `sent + delivered=1`。
- 底层 runner 错误文本不应直接成为用户侧“已发送内容”的唯一摘要。

## 当前实现效果

- 最近一小时的 `RKLB 每日动态监控` 在 `agent.run` 启动约 1 秒后就以 `codex acp stream closed before response` 失败。
- 同轮真实会话没有新增 assistant，日志也没有 `reply.send`。
- 但 `cron_job_runs` 仍把本轮记为 `sent + delivered=1`，并把内部错误文本写进 `response_preview`。
- 这说明当前链路存在“真实未答复，但账本显示已送达”的失真。

## 用户影响

- 这是功能性缺陷。用户设定的定时监控在这一轮没有得到正常结果。
- 台账还会误导后续排查，仿佛本轮已经送达成功，降低人工发现故障的概率。
- 之所以定级为 `P2`，是因为当前证据集中在单条监控任务与单次真实窗口，尚未证明所有 Feishu 定时监控都会断流，但它已经直接影响任务完成与账本可信度。

## 根因判断

- 上游根因是 `codex acp` 在正式响应前异常断流退出。
- 下游还有第二层记账缺陷：scheduler/direct task 台账没有根据真实发送动作校正 `message_send_status`，而是把错误字符串也视作可发送结果。
- 该问题与“search 触顶”“空回复伪成功”不同；这里甚至没有进入正常答复持久化阶段。

## 下一步建议

- 为 `codex acp stream closed before response` 这类 runner 断流接入统一的用户态失败兜底。
- 在写入 `cron_job_runs` 前校验本轮是否真的发生 `reply.send` 或存在 assistant 正文；若没有，应禁止记为 `sent + delivered=1`。
- 增加回归：模拟 runner 在 `agent.run` 后立刻断流，验证 Feishu 定时监控不会再出现“无 assistant、无 reply.send，但台账显示已送达”的坏态。
