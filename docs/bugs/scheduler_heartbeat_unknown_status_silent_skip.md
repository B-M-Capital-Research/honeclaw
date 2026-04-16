# Bug: Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效

- **发现时间**: 2026-04-15 14:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近一小时同一任务持续异常：
    - `run_id=1855`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:00:32.184986+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1849`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T09:30:22.379738+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - 上一轮最近一小时巡检中的同一任务也持续异常：`run_id=1830 (01:31, JsonUnknownStatus)`、`1826 (01:01, JsonNoop)`、`1822 (00:30, JsonNoop)`
    - 往前回溯同一任务仍可见连续多次 `JsonUnknownStatus`：`run_id=1813 (23:00)`、`1806 (22:00)`、`1791 (21:00)`、`1787 (20:31)`、`1781 (20:01)`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 10:00:32.184` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 10:00:32.184` 同轮 `raw_preview` 直接列出 11 只股票的“当前价格 vs 触发价”，但仍未返回合法状态 JSON
    - `2026-04-16 09:30:22.379` 同一任务上一轮仍为 `parse_kind=JsonUnknownStatus`
    - `2026-04-16 01:31:19.950` 与 `01:01:16.495` 说明上一轮巡检窗口里也一直在 `JsonUnknownStatus / JsonNoop` 之间漂移
  - 对比同一小时其他 heartbeat 任务：
    - `j_38745baf`（`全天原油价格3小时播报`）在 `run_id=1847`（`09:30:04`）也短暂出现 `JsonUnknownStatus`，`run_id=1853`（`10:00:10`）又恢复为 `JsonNoop`
    - `j_654aef9b`（`小米30港元破位预警`）在 `10:00:10` 仍为 `JsonNoop -> noop / skipped_noop`
  - 24 小时聚合：
    - `j_ab7e8fb1` 共运行 59 次，其中 29 次为 `JsonUnknownStatus`
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

- 最近一小时内，`Monitor_Watchlist_11` 在 `09:30` 与 `10:00` 两个窗口连续落回 `JsonUnknownStatus`，说明该缺陷在当前活跃时段仍持续存在，而不是只在凌晨偶发。
- `web.log` 在 `2026-04-16 10:00:32.184` 明确记录 `parse_kind=JsonUnknownStatus`；同一轮 `raw_preview` 已经枚举 11 只股票的实时价格和触发价，但因为没有落成合法状态 JSON，最终仍被当成 `noop / skipped_noop` 静默吞掉。
- 对照 `run_id=1849`（`09:30`）和更早的 `01:31` 记录可以看到，这条任务已经连续多轮维持同一症状，并没有随着时间窗切换自然恢复。
- 这也说明问题不只是“偶发返回乱码”，而是模型已经完成了业务判断，却在最后结构化封装一步失配，监控链路因此丢失了本该可追踪的判定结果。
- 数据库没有保存可供人工直接复核的最终文本预览，导致一旦进入 `JsonUnknownStatus`，排障信息同时丢失。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户依赖 heartbeat 监控来发现触发条件，一旦解析失败被静默当成 `noop`，就可能漏掉本应触发的提醒。
- 问题影响的是“自动监控是否按约工作”，会直接破坏任务可信度，因此定级为 `P2`，而不是只影响表达质量的 `P3`。
- 由于系统对用户和运维都没有显式失败信号，这类问题更容易长期潜伏。

## 根因判断

- heartbeat 输出协议对模型返回格式过于脆弱，出现非标准 JSON 或状态枚举漂移时，会落入 `JsonUnknownStatus`。
- 最近一小时同一任务在相邻轮次间会在 `JsonNoop` 与 `JsonUnknownStatus` 之间来回抖动，说明除了单个任务 prompt 外，解析器对“先给分析过程、最后未严格收口到状态 JSON”的输出也缺少足够稳健的兼容或强制约束。
- 调度器把“无法识别状态”错误地归并进 `noop` 路径，造成功能性失败被静默吞掉。
- 现有落库字段只保留 `parse_kind` 与字符数，没有把原始响应片段保留下来，进一步放大了排障盲区。

## 下一步建议

- 把 `JsonUnknownStatus` 从 `noop` 分支中拆出来，至少升级为 `execution_failed` 或独立的可观测状态。
- 针对 heartbeat 的 `noop` 合法输出补一条更宽松的解析回归，覆盖“模型先输出解释性自然语言或 `<think>`，末尾再给 `{\"status\":\"noop\"}`”的场景，避免同一条链路在相邻轮次间随机漂移。
- 为 heartbeat 运行记录保留受控长度的原始响应摘要，方便定位是 JSON 包装漂移、字段名变化还是模型回了自然语言。
- 复核 `Monitor_Watchlist_11` 的 prompt / parser 契约，确认最近从 `JsonNoop` 漂移到 `JsonUnknownStatus` 的具体时间点和触发条件。
