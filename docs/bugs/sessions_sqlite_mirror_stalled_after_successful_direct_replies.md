# Bug: Feishu 直聊已成功持久化并发送，但 `sessions.sqlite3` 会话镜像停留在前一日下午

- **发现时间**: 2026-04-28 01:05 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 02:01 CST` 复核 `SELECT MAX(updated_at), MAX(imported_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `2026-04-28 02:01 CST` 复核 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;`，同样停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 在 sqlite 中最新 `updated_at` 仍是 `2026-04-25T14:58:40.220819+08:00`，没有任何 2026-04-28 00:36、00:39、00:45 的新 user/assistant turn
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 00:37:14.355`：同一 session 的消息 `om_x100b51cd46bb3ca0c42e2d3b53c9d81` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 00:37:17.499`：同一消息继续记录 `step=reply.send ... segments.sent=2/2`
    - `2026-04-28 00:39:51.599`：消息 `om_x100b51cd5cb5d900c457c84783e5659` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 00:39:52.551`：同一消息继续记录 `step=reply.send ... segments.sent=1/1`
    - `2026-04-28 00:45:41.120`：消息 `om_x100b51cd6740c0a0c110494fb2f0cb7` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 00:45:41.967`：同一消息继续记录 `step=reply.send ... segments.sent=1/1`
    - 三轮都以 `done ... success=true` 收口，说明真实直聊主链路并未失败
  - 最近一小时 ACP 事件：`data/runtime/logs/acp-events.log`
    - `2026-04-27T16:45:38-16:45:40Z` 持续写出 `session/update agent_message_chunk`
    - `2026-04-27T16:45:41.110999+00:00` 对同一 `sessionId=019dcf06-7784-7d83-b840-461b3d122744` 返回 `stopReason=end_turn`
    - 说明 00:45 这轮不仅发送链路成功，ACP answer 也完成了最终收口
  - 对照同库其它表：`data/sessions.sqlite3` -> `cron_job_runs`
    - `01:30` 窗口 `run_id=8200-8210`、`02:00` 窗口 `run_id=8222-8232` 仍持续写入同一个 sqlite 文件
    - 例如 `run_id=8232` 的 `executed_at=2026-04-28T02:00:41.592070+08:00` 已存在，说明 sqlite 文件本身并未整体停写，停滞集中在 `sessions` / `session_messages` 镜像链路

## 端到端链路

1. Feishu 用户在直聊中发起真实提问，消息被正常接受并写入运行链路。
2. runner 正常完成工具调用与 answer 收口，`sidecar.log` 记录 `session.persist_assistant detail=done`。
3. 出站链路继续成功分段发送，`reply.send` 记录 `segments.sent`。
4. 按预期，这轮 user / assistant turn 应同步进入 `data/sessions.sqlite3` 的 `sessions` 与 `session_messages`。
5. 实际上 sqlite 会话镜像仍停在前一日下午，导致最近成功会话在巡检和任何依赖该镜像的功能里完全不可见。

## 期望效果

- 成功直聊在 `session.persist_assistant` 与 `reply.send` 之后，应在可接受延迟内同步更新 `data/sessions.sqlite3` 的 `sessions` 与 `session_messages`。
- `sessions.updated_at`、`last_message_at`、`imported_at` 不应长期落后于真实成功会话。
- 若镜像链路落后，应有明确的 lag 指标或失败诊断，而不是静默停在旧时间点。

## 当前实现效果

- 到 `2026-04-28 02:01 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`。
- 同一时间窗内，至少 3 条 Feishu 直聊已经完成 `persist_assistant + reply.send + success=true`，但都没有进入 sqlite 会话镜像。
- `cron_job_runs` 仍在 `01:30`、`02:00` 持续正常更新，说明不是整个 sqlite 数据面停摆，而是会话镜像专用链路失效或卡住。

## 用户影响

- 这是功能性缺陷，不是单纯观测偏差。任何依赖 `data/sessions.sqlite3` 的巡检、历史检索、会话审计、质量回溯或衍生功能都会读到过期状态。
- 当前证据显示用户仍能收到最终回复，因此它还不是“主回复链路中断”的 `P1`；但会话镜像整体停更会直接误导缺陷巡检与下游状态判断，因此定级为 `P2`。
- 若后续有产品功能直接依赖 sqlite 会话镜像进行恢复、列表展示或二次加工，这条缺陷会进一步放大为用户可见的数据陈旧问题。

## 根因判断

- 高概率是 `sessions` / `session_messages` 的异步镜像或导入链路在 `2026-04-27 16:54` 后停滞，而不是 Feishu 直聊主执行链路失败。
- `cron_job_runs` 仍能持续写入同一个 sqlite 文件，说明数据库文件可写、进程也未整体失活；问题更像是“会话镜像专属 writer / importer / flush 路径卡住”。
- 这与 [`web_scheduler_codex_acp_unfinished_tool_send_failed.md`](./web_scheduler_codex_acp_unfinished_tool_send_failed.md) 不同：那条缺陷是单轮 scheduler 既未落库也未送达；本条证据里 direct 会话已经成功送达，但 sqlite 镜像整体没有跟上。

## 下一步建议

- 先排查 `sessions` / `session_messages` 的 writer、importer、checkpoint 或队列消费者是否在 `2026-04-27 16:54` 后卡住。
- 增加“会话镜像最新时间 vs 实际成功 `reply.send` 时间”的 lag 监控，避免再次只能靠人工巡检发现。
- 对成功会话补一条只读诊断，确认真实会话源文件是否已更新、是否只是 sqlite 镜像缺失，而不是更早的持久化链路已分叉。
