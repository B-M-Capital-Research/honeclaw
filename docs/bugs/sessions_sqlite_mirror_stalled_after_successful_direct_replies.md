# Bug: Feishu 直聊已成功持久化并发送，但 `sessions.sqlite3` 会话镜像停留在前一日下午

- **发现时间**: 2026-04-28 01:05 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 16:04 CST` 复核：`sessions` 与 `session_messages` 的 `MAX(updated_at/last_message_at/imported_at/timestamp)` 仍全部卡在 `2026-04-27T16:54:20+08:00`，最近一小时依旧没有任何新增镜像。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍是 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍是 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T16:01:24.734498+08:00`（`run_id=8908`，`Cerebras IPO与业务进展心跳监控`），说明 sqlite 文件本身仍在接收最新调度结果，而会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 15:40:50.862957Z`：Feishu 直聊 `Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 记录 `step=message.accepted`
    - `2026-04-28 15:42:58.343762Z`：同一 session 记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 15:42:58.343840Z`：同一轮落成 `done ... success=true elapsed_ms=126131 iterations=1 tools=3 ... reply.chars=2801`
    - `2026-04-28 15:43:01.031598Z`：同一消息继续记录 `step=reply.send detail=segments.sent=2/2`
    - 说明到 `15:43` 为止，至少又有 1 条新的 Feishu 成功直聊完整走完执行、持久化与发送，但 sqlite 会话镜像仍没有任何推进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 15:03 CST` 复核：`sessions` 与 `session_messages` 的 `MAX(updated_at/last_message_at/imported_at/timestamp)` 仍全部卡在 `2026-04-27T16:54:20+08:00`，最近一小时依旧没有任何新增镜像。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍是 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍是 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T15:01:20.718842+08:00`（`run_id=8860`，`ORCL 大事件监控`），说明 sqlite 文件本身仍在接收最新调度结果，而会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 14:08:14.947846Z`：Feishu 直聊 `Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 落成 `done ... success=true elapsed_ms=173485 iterations=1 tools=3 ... reply.chars=2985`
    - `2026-04-28 14:15:15.926893Z`：同一 session 紧接着又有一轮成功直聊，继续落成 `done ... success=true elapsed_ms=110457 iterations=1 tools=3 ... reply.chars=2335`
    - 说明到 `14:15` 为止，至少又有 2 条新的 Feishu 成功直聊完整走完执行与发送，但 sqlite 会话镜像仍没有任何推进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 14:01 CST` 复核：`sessions` 与 `session_messages` 的 `MAX(updated_at/last_message_at/imported_at/timestamp)` 仍全部卡在 `2026-04-27T16:54:20+08:00`，最近一小时依旧没有任何新增镜像。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍是 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍是 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T14:01:25.926162+08:00`（`run_id=8812`，`ORCL 大事件监控`），说明 sqlite 文件本身仍在接收最新调度结果，而会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 13:56:05.407`：Feishu 直聊 `Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21` 记录 `step=message.accepted`
    - `2026-04-28 13:59:03.801`：同一 session 记录 `step=session.persist_assistant detail=done`
    - `2026-04-28T13:59:03.800959+08:00`：同一轮落成 `done ... success=true elapsed_ms=177032 iterations=1 tools=8 ... reply.chars=4795`
    - `2026-04-28 13:59:07.507`：同一消息继续记录 `step=reply.send detail=segments.sent=3/3`
    - 说明到 `13:59` 为止，至少又有 1 条新的 Feishu 成功直聊完整走完执行、持久化与发送，但 sqlite 会话镜像仍没有任何推进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 13:01 CST` 复核：`sessions` 与 `session_messages` 的 `MAX(updated_at/last_message_at/imported_at/timestamp)` 仍全部卡在 `2026-04-27T16:54:20+08:00`，最近一小时依旧没有任何新增镜像。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍是 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍是 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T13:01:45.211635+08:00`（`run_id=8764`，`持仓重大事件心跳检测`），说明 sqlite 文件本身仍在接收最新调度结果，而会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 12:30:59.969`：`持仓重大事件心跳检测` 完成 `run_id=8739`，记录 `deliver job_id=j_db12f27f ... parse_kind=JsonTriggered`
    - `2026-04-28T12:31:02.817388+08:00`：同一 run 在 `cron_job_runs` 落成 `completed + sent + delivered=1`
    - `2026-04-28 13:01:45.210`：下一窗口同一 job 又落成 `parse_kind=JsonNoop`，并在 `2026-04-28T13:01:45.211635+08:00` 写入 `cron_job_runs`
    - 说明到 `13:01` 为止，至少又有一轮新的 scheduler 会话结果完整进入 runtime / `cron_job_runs`，但 sqlite 会话镜像仍没有任何推进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 12:02 CST` 复核：`sessions` 与 `session_messages` 的 `MAX(updated_at/last_message_at/imported_at/timestamp)` 仍全部卡在 `2026-04-27T16:54:20+08:00`，最近一小时没有任何新增镜像。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍是 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍是 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T12:02:08.803575+08:00`（`run_id=8716`，`每日公司资讯与分析总结`，`completed + sent + delivered=1`），说明 sqlite 文件本身仍在接收最新调度结果，而会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 12:00:00.471`：Feishu scheduler 直达 session `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 记录新的 `recv` 与 `step=session.persist_user detail=done`
    - `2026-04-28 12:02:03.414`：同一 session 完成 `step=session.persist_assistant detail=done`
    - `2026-04-28 12:02:03.414331Z`：同一轮落成 `done ... success=true elapsed_ms=122928 iterations=1 tools=1(Tool: hone/skill_tool) reply.chars=3979`
    - `2026-04-28T12:02:08.803575+08:00`：同窗 `cron_job_runs.run_id=8716` 记录这轮任务已 `completed + sent + delivered=1`
    - 说明到 `12:02` 为止，至少又有 1 条新的 Feishu 成功会话完整走完执行与发送，但 sqlite 会话镜像仍没有任何推进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 11:01 CST` 复核：`sessions` 与 `session_messages` 最近一小时仍然没有任何新 `updated_at` / `last_message_at` / `timestamp` / `imported_at`。
    - `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍停在 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T11:01:19.005400+08:00`，说明 sqlite 文件本身仍在写，而会话镜像链路已静默停滞超过 18 小时。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 10:51:04.542`：Feishu 直聊 `Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21` 记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 10:51:04.542484Z`：同一轮落成 `done ... success=true elapsed_ms=231555 iterations=1 tools=12 ... reply.chars=5195`
    - `2026-04-28 10:51:08.185`：同一 session 继续记录 `step=reply.send detail=segments.sent=3/3`
    - 说明最近一小时仍有新的 Feishu 成功直聊完整走完持久化与发送，但 sqlite 会话镜像依旧完全没有前进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 10:03 CST` 复核最近一小时增量查询：`sessions` 与 `session_messages` 依旧没有任何 `updated_at` / `last_message_at` / `timestamp` / `imported_at` 落入最近一小时。
    - 同时 `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;` 仍停在 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T10:00:43.429866+08:00`，说明 sqlite 文件仍在承接最新调度结果，而会话镜像链路已静默停滞超过 17 小时。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 09:32:20.693`：Discord 群聊 `Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371` 新增 user turn，记录 `step=session.persist_user detail=done`
    - `2026-04-28 09:32:31.780`：同一 session 完成 `step=session.persist_assistant detail=done`
    - `2026-04-28 09:32:32.094`：同一 session 继续 `step=reply.send detail=segments.sent=1`
    - `2026-04-28 09:33:33.245`：同一 session 第二轮追问 `退订所有消息` 再次完成 `step=session.persist_assistant detail=done`
    - `2026-04-28 09:33:33.245299Z`：同一轮落成 `done ... success=true elapsed_ms=35011 iterations=1 tools=3 ... reply.chars=85`

    - 说明最近一小时不仅 Feishu，会话主链路在 Discord 也有真实成功 turn，但 sqlite 会话镜像依旧完全没有前进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 09:02 CST` 复核 `SELECT MAX(updated_at), MAX(last_message_at), MAX(imported_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 同时 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T09:01:43.651211+08:00`，说明 sqlite 文件仍在承接最新调度结果，而会话镜像链路已静默停滞超过 16 小时。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 09:01:15.398`：`session=Actor_feishu__direct__ou_5f95ab3697246ded86446fcc260e27e1e2` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 09:01:15.398376Z`：同一 session 紧接着落成 `done ... success=true elapsed_ms=74189 iterations=1 tools=6 ... reply.chars=2524`
    - `2026-04-28 09:01:37.083`：`session=Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 09:01:37.083247Z`：同一 session 紧接着落成 `done ... success=true elapsed_ms=95874 iterations=1 tools=none reply.chars=2701`
    - `2026-04-28 09:01:40.527`：`session=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 09:01:40.527698Z`：同一 session 紧接着落成 `done ... success=true elapsed_ms=99312 iterations=1 tools=25(Tool: hone/data_fetch) reply.chars=2586`
    - 说明 `09:01` 这一轮至少 3 条 Feishu 直聊真实会话都完整走完了 `persist_assistant -> success=true`，但 sqlite 会话镜像依旧完全没有前进。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 08:04 CST` 复核 `SELECT MAX(last_message_at), MAX(updated_at), MAX(imported_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 同时 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T08:02:34.000419+08:00`，说明最新一小时 sqlite 文件本身仍可写，而会话镜像链路继续静默停滞超过 15 小时。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 08:00:01.199`：`session=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 已记录新的 scheduler 直达消息 `recv`
    - `2026-04-28 08:02:30.249`：同一 session 已记录 `step=session.persist_assistant detail=done`
    - `2026-04-28 08:02:30.249923Z`：同一 session 紧接着落成 `done ... success=true elapsed_ms=149038 iterations=1 tools=16 ... reply.chars=2939`
    - 说明 `08:00` 这轮 Feishu 直达链路已经完整完成持久化与成功收口，但 sqlite 会话镜像依旧没有前进
  - 最近一小时 ACP 事件：`data/runtime/logs/acp-events.log`
    - `2026-04-28T00:02:32.238585+00:00` 对同一 `sessionId=019dcc3c-631c-7e03-9495-fa75695f36f1` 持续写出 `agent_message_chunk`
    - `2026-04-28T00:02:55.884506+00:00` 到 `00:02:57.488371+00:00` 继续完成 `web_search` MCP 工具调用与 `status=unavailable` 结果回写
    - 说明这轮不仅主链路成功，连 ACP session/update 事件也持续推进；缺口仍集中在 sqlite 镜像没有把新 turn 写进 `sessions` / `session_messages`
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 07:03 CST` 复核 `SELECT MAX(last_message_at), MAX(updated_at), MAX(imported_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034097+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 同时 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;` 仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 但同库 `cron_job_runs` 已继续写到 `2026-04-28T07:00:40.541211+08:00`，说明最新一小时 sqlite 文件本身仍可写，而会话镜像链路继续静默停滞超过 14 小时。
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 04:03 CST` 复核 `SELECT MAX(last_message_at), MAX(updated_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034097+08:00`
    - `2026-04-28 04:03 CST` 复核 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;`，同样仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 同时 `cron_job_runs` 已继续写到 `2026-04-28T04:01:13.314715+08:00`，说明最新一小时 sqlite 文件本身仍可写，但会话镜像链路继续静默停滞。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 04:01:08.855`：`session=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 已记录 `step=session.persist_assistant detail=done`
    - 同一时间点继续记录 `done ... success=true elapsed_ms=67733 iterations=1 tools=7`
    - 这轮说明即使到 04:01 仍有新的 Feishu 直聊/调度直达会话完成最终持久化与成功收口，但 `sessions.sqlite3` 会话镜像依旧没有前进
  - 最近一小时真实会话镜像状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `2026-04-28 03:05 CST` 复核 `SELECT MAX(last_message_at), MAX(updated_at) FROM sessions;`，最新会话镜像仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034097+08:00`
    - `2026-04-28 03:05 CST` 复核 `SELECT MAX(timestamp), MAX(imported_at) FROM session_messages;`，同样仍停在 `2026-04-27T16:54:20.033926+08:00` / `2026-04-27T16:54:20.034386+08:00`
    - 同时 `cron_job_runs` 已继续写到 `2026-04-28T03:00:56.089238+08:00`，说明最新一小时 sqlite 文件本身仍可写，但会话镜像链路继续静默停滞。
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

## 修复与验证

- 2026-04-28: `crates/hone-core/src/config/server.rs` 将 `server.session_sqlite_shadow_write_enabled` 的 serde 默认值改为 `true`，`config.example.yaml` 也同步改为 `true`。
- 2026-04-28: 本机 `config.yaml` 已确认该开关为 `true`，后续成功直聊和 scheduler 会话应重新写入 `sessions.sqlite3` 镜像。
- 2026-04-28: `cargo check -p hone-memory -p hone-scheduler -p hone-tools -p hone-web-api -p hone-event-engine -p hone-channels --tests`

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

- 到 `2026-04-28 16:04 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 16:01:24+08:00`。
- 到 `2026-04-28 15:43` 为止，至少又有 1 条 Feishu 直聊成功完成 `message.accepted -> session.persist_assistant -> success=true -> reply.send`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 15:03 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 15:01:20+08:00`。
- 到 `2026-04-28 14:15` 为止，至少又有 2 条 Feishu 直聊成功完成 `done + success=true`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 14:01 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 14:01:25+08:00`。
- 到 `2026-04-28 13:59` 为止，至少又有 1 条 Feishu 直聊成功完成 `message.accepted -> session.persist_assistant -> success=true -> reply.send`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 13:01 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 13:01:45+08:00`。
- 到 `2026-04-28 12:31` 与 `13:01` 为止，至少又有 1 条 heartbeat scheduler 会话结果分别完成 `deliver` 或 `noop` 收口并写入 `cron_job_runs`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 11:01 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 11:01:19+08:00`。
- 到 `2026-04-28 12:02 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 12:02:08+08:00`。
- 到 `2026-04-28 12:02` 为止，至少又有 1 条 Feishu scheduler 直达会话完成 `recv -> session.persist_assistant -> success=true -> delivered=1`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 10:51` 为止，至少又有 1 条 Feishu 直聊成功完成 `persist_assistant + reply.send + success=true`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 10:03 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 10:00:43+08:00`。
- 到 `2026-04-28 09:33` 为止，至少又有 2 条 Discord 群聊 turn 完成 `persist_user/persist_assistant + reply.send + success=true`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 08:04 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 08:02:34+08:00`。
- 到 `2026-04-28 09:02 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 09:01:43+08:00`。
- 到 `2026-04-28 09:01` 为止，至少又有 3 条 Feishu 直聊成功完成 `persist_assistant + success=true`，但仍没有任何一条进入 sqlite 会话镜像。
- 到 `2026-04-28 08:02` 为止，至少又有 1 条 Feishu scheduler 直达会话完成 `recv -> agent.run -> session.persist_assistant -> success=true`，但仍没有进入 sqlite 会话镜像。
- 到 `2026-04-28 07:03 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，而 `cron_job_runs` 已继续前进到 `2026-04-28 07:00:40+08:00`。
- 到 `2026-04-28 04:03 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，且在 `04:01` 又有新的成功会话完成 `persist_assistant` 之后依然没有前进一步。
- 到 `2026-04-28 03:05 CST` 为止，`data/sessions.sqlite3` 的 `sessions` / `session_messages` 最新时间仍停在 `2026-04-27 16:54:20+08:00`，与上一轮 `02:01` 巡检相比没有前进一步。
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
