# Bug: Feishu scheduler 预写的 `running/pending` 台账不会被终态覆盖，长期残留为悬挂运行中

- **发现时间**: 2026-04-28 17:02 CST
- **Bug Type**: System Error
- **严重等级**: P3
- **状态**: New

## 证据来源

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 19:01 CST` 再次复核，最新 `19:00` heartbeat 窗口继续把 started 行与终态行拆成两批记录：
    - started 行为 `run_id=9029-9040`，`executed_at=2026-04-28T19:00:00.534475+08:00` 到 `19:00:00.543875+08:00`，12 个 job 全部仍是 `execution_status=running`、`message_send_status=pending`
    - 同窗终态行则另起为 `run_id=9041-9052`，在 `19:00:30.019454+08:00` 到 `19:01:48.874222+08:00` 之间全部落成 `noop + skipped_noop`
    - 这意味着仅 heartbeat 窗口内，`18:00`、`18:30`、`19:00` 三个批次已经累计留下 `36` 条未被终态覆盖的 started 行
  - 若按最近一小时全量 scheduler 聚合，坏态已经不只是 heartbeat 局部噪声：
    - `running + pending = 446`
    - `noop + skipped_noop = 278`
    - `execution_failed + skipped_error = 163`
    - `completed + sent = 5`
    - 说明 started 行残留已经成为最近一小时里占比最高的台账状态，而不是边缘偶发
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 18:03 CST` 再次复核，started 残留在最新整点窗口继续累积：
    - 最近一小时聚合结果已变成 `running + pending = 24`、`noop + skipped_noop = 24`、`completed + sent = 1`
    - 说明 `17:30` 与 `18:00` 两个 heartbeat 窗口都各自额外留下 12 条 started 行，而终态行继续另起新记录
  - `18:00` 窗口的 started 行为 `run_id=8981-8992`，对应 job 包含 `ASTS 重大异动心跳监控`、`全天原油价格3小时播报`、`小米30港元破位预警`、`RKLB异动监控`、`持仓重大事件心跳检测`、`CAI破位预警`、`小米破位预警`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`ORCL 大事件监控`
  - 同一批 job 的终态行继续单独存在：
    - `run_id=8993-9003` 在 `18:00:18-18:01:32` 已分别落成 `noop + skipped_noop`
    - `run_id=8985`（`持仓重大事件心跳检测`）的终态还继续延后到 `2026-04-28T18:02:14.065110+08:00` 才新增 `run_id=9004` 对应的 `noop + skipped_noop`
  - 这说明 started 行不是短暂延迟；即使终态已在同一窗口内补齐，原 started 行也不会被覆盖或关闭
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 17:02 CST` 复核最近两小时 Feishu heartbeat 类任务后，发现 12 个 job 都同时存在：
    - 预写的 `execution_status=running`、`message_send_status=pending`
    - 同一 `delivery_key` 对应的终态行（`noop/skipped_noop`、`completed/sent` 或 `execution_failed/skipped_error`）
  - 聚合结果显示，这 12 个 job 在 `15:30`、`16:00`、`16:30`、`17:00` 四个窗口里各自都保留了 4 条 `running/pending` 残留：
    - `持仓重大事件心跳检测`
    - `TEM破位预警`
    - `CAI破位预警`
    - `ORCL 大事件监控`
    - `ASTS 重大异动心跳监控`
    - `Monitor_Watchlist_11`
    - `RKLB异动监控`
    - `全天原油价格3小时播报`
    - `小米30港元破位预警`
    - `Cerebras IPO与业务进展心跳监控`
    - `TEM大事件心跳监控`
    - `小米破位预警`
  - 同一批 job 的终态行并未缺失：
    - 例如 `小米30港元破位预警` 在最近两小时同时存在
      - `run_id=8866` / `8885` / `8910` / `8936`: `running + pending`, `detail_json.phase="started"`
      - `run_id=8878`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:15:30:heartbeat`
      - `run_id=8900`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:16:00:heartbeat`
      - `run_id=8930`: `completed + sent + delivered=1`, `delivery_key=j_654aef9b:2026-04-28:16:30:heartbeat`
      - `run_id=8950`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:17:00:heartbeat`
    - `持仓重大事件心跳检测` 同样在四个窗口都出现“started 行 + 终态行”并存：
      - `run_id=8863` / `8889` / `8919` / `8939`: `running + pending`
      - `run_id=8882`: `noop + skipped_noop`
      - `run_id=8907`: `execution_failed + skipped_error`
      - `run_id=8931`: `noop + skipped_noop`
      - `run_id=8954`: `noop + skipped_noop`
- 最近一小时真实运行语义对照：
  - `run_id=8930`（`小米30港元破位预警`）在 `2026-04-28T16:30:40.310894+08:00` 已成功落成 `completed + sent + delivered=1`
  - 但同一 delivery window 的 `run_id=8910` 仍永久保留 `running + pending`
  - 说明这不是“终态没写出来”，而是“started 行没有在收口时被更新或关闭”
- 已检索相关缺陷文档：
  - [`feishu_scheduler_run_stuck_without_cron_job_run.md`](./feishu_scheduler_run_stuck_without_cron_job_run.md)
  - [`scheduler_heartbeat_unknown_status_silent_skip.md`](./scheduler_heartbeat_unknown_status_silent_skip.md)
  - 当前坏态不同于旧的“整轮卡住且无台账”或“heartbeat 输出契约漂移”：
    - 现在台账有 started 行
    - 终态也能写出
    - 但 started 行没有被同一 run 的终态覆盖

## 端到端链路

1. Feishu scheduler 到点触发 heartbeat / 报警任务。
2. 触发入口先写一条 `cron_job_runs` started 行：`execution_status=running`、`message_send_status=pending`、`detail_json.phase=started`。
3. 任务继续执行，并在几十秒后写出真正终态：`noop`、`completed` 或 `execution_failed`。
4. 实际数据库里，终态不是覆盖 started 行，而是新增了第二条 run 记录。
5. 结果是同一个 delivery window 同时被记录成“仍在运行”和“已经收口”。

## 期望效果

- 同一个 `delivery_key` 在调度台账里应只有一个最终可判定状态。
- started 行应在任务收口时被更新为终态，或至少被明确标记为 superseded/closed。
- 巡检查询 `execution_status=running` 时，不应把已经 `noop`、`completed`、`execution_failed` 的历史窗口误判成仍在执行。

## 当前实现效果

- 最近一小时 `cron_job_runs` 汇总到 `2026-04-28 19:01 CST` 时，仅 heartbeat 三个窗口就已累计 `36` 条 `running + pending` 残留；按最近一小时全量 scheduler 聚合则进一步放大到 `446` 条 `running + pending`。
- `19:00` 新窗再次证明坏态不是历史遗留脏数据：`run_id=9029-9040` 的 started 行刚写出不到两分钟，同窗终态就已经另起为 `9041-9052`，started 行仍不会被覆盖。
- 最近一小时 `cron_job_runs` 汇总到 `2026-04-28 18:03 CST` 已进一步扩大成 `24` 条 `running + pending` 残留，对应两个窗口各 `12` 条 started 行全部没有收口。
- 最近一小时 `cron_job_runs` 汇总里，`running + pending` 反而是最多的组合，达到 24 条；同窗真正终态只有 23 条。
- 对同一个 job / 同一个 `delivery_key`，数据库会同时出现 `running/pending` 与终态行。
- 这会把活跃运行中的数量系统性抬高，并让人工巡检难以分辨“真卡住”与“已经完成但 started 行未清理”。

## 用户影响

- 当前证据里，受影响窗口的大部分 heartbeat 仍能正常收口为 `noop`，个别窗口还能成功 `completed + sent`，因此暂未看到用户因这条缺陷直接收不到消息。
- 影响主要集中在调度台账正确性、巡检判断和后续故障排查：真实已经结束的窗口会长期伪装成“仍在运行”。
- 因此这次定级为 `P3`：它没有直接打断主功能链路，但会显著污染缺陷台账与运维判断，且会掩盖真正的悬挂 run。

## 根因判断

- 高概率是 2026-04-26 为止血“无台账”问题加入的 started 行写入逻辑，只负责 insert，不负责在终态阶段按 `delivery_key` 回写同一条记录。
- 终态链路当前更像是“另起一条 run”，而不是“完成 started 行”。
- 这与 [`feishu_scheduler_run_stuck_without_cron_job_run.md`](./feishu_scheduler_run_stuck_without_cron_job_run.md) 是相关但独立的后续问题：旧问题是没有 started 行；当前问题是 started 行写出来后没有被 finalize。

## 下一步建议

- 按 `delivery_key` 或等价唯一键，把 started 行与终态行收敛成同一条 `cron_job_runs` 记录。
- 若历史兼容必须保留多行，也应在 started 行补一个明确的 closed/superseded 标记，并让默认巡检查询过滤掉已终结的 started 行。
- 增加一个只读巡检规则：同一 `delivery_key` 若同时存在 `phase=started` 和终态行，则记录为台账异常，而不是普通运行中。
