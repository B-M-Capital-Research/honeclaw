# Bug: Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效

- **发现时间**: 2026-04-15 14:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近一小时同一任务持续异常：
    - `run_id=1775`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-15T19:01:17.700484+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
    - `run_id=1772`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-15T18:31:18.669982+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
    - `run_id=1768`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-15T18:01:20.663318+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
    - `run_id=1763`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-15T17:00:25.067028+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
    - `run_id=1760`，`executed_at=2026-04-15T16:30:23.531089+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
    - `run_id=1756`，`executed_at=2026-04-15T16:03:37.828145+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 19:01:17.699` `parse_kind=JsonUnknownStatus`
    - `2026-04-15 18:31:18.669` `parse_kind=JsonUnknownStatus`
    - `2026-04-15 18:01:20.663` `parse_kind=JsonUnknownStatus`
    - `2026-04-15 16:03:37.827` `parse_kind=JsonUnknownStatus`
    - `2026-04-15 17:00:25.066` `parse_kind=JsonUnknownStatus`
  - 对比同一小时其他 heartbeat 任务：
    - `j_38745baf`、`j_654aef9b` 在 `18:31` 与 `19:01` 同时段仍为 `JsonNoop -> noop / skipped_noop`
  - 24 小时聚合：
    - `j_ab7e8fb1` 共运行 57 次，其中 27 次为 `JsonUnknownStatus`
  - 生命周期聚合：
    - `j_ab7e8fb1` 自 `2026-04-04T21:30:31.191391+08:00` 起累计运行 454 次，仅 3 次 `completed` / `delivered`

## 端到端链路

1. 用户创建 heartbeat / watchlist 监控任务，预期在满足条件时收到提醒，在不满足条件时收到可判定的 `noop`。
2. 调度器按计划执行 heartbeat 任务，模型需要返回符合约定的结构化状态。
3. 当前 `Monitor_Watchlist_11` 在最近一小时连续返回无法被解析器识别的结果，数据库记录为 `parse_kind=JsonUnknownStatus`。
4. 调度器没有把这类解析失败升级为错误，也没有回退到人工可见告警，而是直接按 `noop` 处理并 `skipped_noop`。
5. 结果是用户侧既没有收到提醒，也不知道本次检测其实没有被正确解析。

## 期望效果

- heartbeat 任务应稳定返回可解析的结构化状态，至少能明确区分 `triggered`、`noop`、`error`。
- 当模型输出不符合约定时，调度器不应静默吞掉，而应记录可追踪错误并进入可观测状态。
- 对“监控提醒”类任务，解析失败至少应进入 `execution_failed` 或运维可见告警，而不是伪装成正常 `noop`。

## 当前实现效果

- 最近一小时内，`Monitor_Watchlist_11` 在 `18:01`、`18:31`、`19:01` 三轮连续落到 `noop / skipped_noop`，说明该缺陷不是早前单次波动，而是在当前巡检窗口内稳定复现。
- `web.log` 在 `18:31:18.669` 与 `19:01:17.699` 仍直接记录 `parse_kind=JsonUnknownStatus`，证明解析异常没有消失，只是继续被静默吞到 `noop` 分支。
- 同一时间窗内其他 heartbeat 任务没有出现相同行为，说明问题不是“heartbeat 全局都无结果”，而是该链路的输出契约或解析兼容性异常。
- 数据库没有保存可供人工直接复核的最终文本预览，导致一旦进入 `JsonUnknownStatus`，排障信息同时丢失。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户依赖 heartbeat 监控来发现触发条件，一旦解析失败被静默当成 `noop`，就可能漏掉本应触发的提醒。
- 问题影响的是“自动监控是否按约工作”，会直接破坏任务可信度，因此定级为 `P2`，而不是只影响表达质量的 `P3`。
- 由于系统对用户和运维都没有显式失败信号，这类问题更容易长期潜伏。

## 根因判断

- heartbeat 输出协议对模型返回格式过于脆弱，出现非标准 JSON 或状态枚举漂移时，会落入 `JsonUnknownStatus`。
- 调度器把“无法识别状态”错误地归并进 `noop` 路径，造成功能性失败被静默吞掉。
- 现有落库字段只保留 `parse_kind` 与字符数，没有把原始响应片段保留下来，进一步放大了排障盲区。

## 下一步建议

- 把 `JsonUnknownStatus` 从 `noop` 分支中拆出来，至少升级为 `execution_failed` 或独立的可观测状态。
- 为 heartbeat 运行记录保留受控长度的原始响应摘要，方便定位是 JSON 包装漂移、字段名变化还是模型回了自然语言。
- 复核 `Monitor_Watchlist_11` 的 prompt / parser 契约，确认最近从 `JsonNoop` 漂移到 `JsonUnknownStatus` 的具体时间点和触发条件。
