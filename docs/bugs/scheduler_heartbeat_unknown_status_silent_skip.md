# Bug: Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效

- **发现时间**: 2026-04-15 14:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近一小时同一任务持续异常：
    - `run_id=1865`，`job_id=j_c1c1be63`，`job_name=存储板块加仓信号监控`，`executed_at=2026-04-16T11:00:31.512294+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `2026-04-16T11:30:28.836+08:00`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，运行日志再次记录 `parse_kind=JsonUnknownStatus`，且同轮仍被记为“心跳任务未命中，本轮不发送”
    - `run_id=1860`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:30:32.268455+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1855`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:00:32.184986+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1849`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T09:30:22.379738+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - 上一轮最近一小时巡检中的同一任务也持续异常：`run_id=1830 (01:31, JsonUnknownStatus)`、`1826 (01:01, JsonNoop)`、`1822 (00:30, JsonNoop)`
    - 往前回溯同一任务仍可见连续多次 `JsonUnknownStatus`：`run_id=1813 (23:00)`、`1806 (22:00)`、`1791 (21:00)`、`1787 (20:31)`、`1781 (20:01)`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 10:30:32.267` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:00:31.512` `job_id=j_c1c1be63` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:30:28.836` `job_id=j_ab7e8fb1` 再次出现 `parse_kind=JsonUnknownStatus`，随后仍打印 `心跳任务未命中，本轮不发送`
    - `2026-04-16 10:00:32.184` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 10:00:32.184` 同轮 `raw_preview` 直接列出 11 只股票的“当前价格 vs 触发价”，但仍未返回合法状态 JSON
    - `2026-04-16 09:30:22.379` 同一任务上一轮仍为 `parse_kind=JsonUnknownStatus`
    - `2026-04-16 01:31:19.950` 与 `01:01:16.495` 说明上一轮巡检窗口里也一直在 `JsonUnknownStatus / JsonNoop` 之间漂移
  - 对比同一小时其他 heartbeat 任务：
    - `j_38745baf`（`全天原油价格3小时播报`）在 `run_id=1847`（`09:30:04`）也短暂出现 `JsonUnknownStatus`，`run_id=1853`（`10:00:10`）又恢复为 `JsonNoop`
    - `j_654aef9b`（`小米30港元破位预警`）在 `10:00:10` 仍为 `JsonNoop -> noop / skipped_noop`
    - `j_ab7e8fb1` 在 `run_id=1864`（`11:00:24`）短暂恢复为 `JsonNoop`，但同一窗口另一条 heartbeat `j_c1c1be63` 又落回 `JsonUnknownStatus`，说明缺陷并未整体收口，只是不同任务间漂移
  - 24 小时聚合：
    - `j_ab7e8fb1` 共运行 59 次，其中 29 次为 `JsonUnknownStatus`
  - 生命周期聚合：
    - `j_ab7e8fb1` 自 `2026-04-04T21:30:31.191391+08:00` 起累计运行 454 次，仅 3 次 `completed` / `delivered`

## 端到端链路

1. 用户创建 heartbeat / watchlist 监控任务，预期在满足条件时收到提醒，在不满足条件时收到可判定的 `noop`。
2. 调度器按计划执行 heartbeat 任务，模型需要返回符合约定的结构化状态。
3. 当前 `Monitor_Watchlist_11` 在最近一小时连续返回无法被解析器识别的结果，数据库记录为 `parse_kind=JsonUnknownStatus`。
4. 调度器仍然没有把这类解析失败稳定升级为错误，也没有回退到人工可见告警，而是直接按 `noop` 处理并 `skipped_noop`。
5. 结果是用户侧既没有收到提醒，也不知道本次检测其实没有被正确解析。

## 期望效果

- heartbeat 任务应稳定返回可解析的结构化状态，至少能明确区分 `triggered`、`noop`、`error`。
- 当模型输出不符合约定时，调度器不应静默吞掉，而应记录可追踪错误并进入可观测状态。
- 对“监控提醒”类任务，解析失败至少应进入 `execution_failed` 或运维可见告警，而不是伪装成正常 `noop`。

## 当前实现效果

- 最近一小时内，`Monitor_Watchlist_11` 在 `10:00`、`10:30` 与 `11:30` 三个窗口连续落回 `JsonUnknownStatus`；到 `11:00` 虽短暂恢复为 `JsonNoop`，但另一条 heartbeat `存储板块加仓信号监控` 又在 `11:00:31` 落回 `JsonUnknownStatus`，说明该缺陷仍在当前活跃时段持续出现，而不是只在凌晨偶发。
- `web.log` 在 `2026-04-16 10:00:32.184` 明确记录 `parse_kind=JsonUnknownStatus`；同一轮 `raw_preview` 已经枚举 11 只股票的实时价格和触发价，但因为没有落成合法状态 JSON，最终仍被当成 `noop / skipped_noop` 静默吞掉。
- 对照 `11:30` 的最新日志、`run_id=1860`（`10:30`）、`1855`（`10:00`）、`1849`（`09:30`）和更早的 `01:31` 记录可以看到，这类 heartbeat 已经连续多轮维持同一症状，并没有随着时间窗切换自然恢复。
- 这也说明问题不只是“偶发返回乱码”，而是模型已经完成了业务判断，却在最后结构化封装一步失配，监控链路因此丢失了本该可追踪的判定结果。
- 数据库没有保存可供人工直接复核的最终文本预览，导致一旦进入 `JsonUnknownStatus`，排障信息同时丢失。
- 由于 `run_id=1865` 仍是 `parse_kind=JsonUnknownStatus + execution_status=noop`，当前线上行为与“已升级为 `execution_failed + skipped_error`”的修复结论不一致，说明此前修复尚未生效到当前运行实例，或存在未覆盖的分支。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户依赖 heartbeat 监控来发现触发条件，一旦解析失败被静默当成 `noop`，就可能漏掉本应触发的提醒。
- 问题影响的是“自动监控是否按约工作”，会直接破坏任务可信度，因此定级为 `P2`，而不是只影响表达质量的 `P3`。
- 由于系统对用户和运维都没有显式失败信号，这类问题更容易长期潜伏。

## 根因判断

- heartbeat 输出协议对模型返回格式过于脆弱，出现非标准 JSON 或状态枚举漂移时，会落入 `JsonUnknownStatus`。
- 最近一小时同一任务在相邻轮次间会在 `JsonNoop` 与 `JsonUnknownStatus` 之间来回抖动，说明除了单个任务 prompt 外，解析器对“先给分析过程、最后未严格收口到状态 JSON”的输出也缺少足够稳健的兼容或强制约束。
- 调度器把“无法识别状态”错误地归并进 `noop` 路径，造成功能性失败被静默吞掉。
- 现有落库字段只保留 `parse_kind` 与字符数，没有把原始响应片段保留下来，进一步放大了排障盲区。

## 修复情况（2026-04-16，待重新验证）

- `crates/hone-channels/src/scheduler.rs` 已把 heartbeat 的解析失败从静默 `noop` 分支中拆出：
  - `JsonUnknownStatus` 现在会返回 `error`，由各渠道 scheduler 落库为 `execution_failed + skipped_error`
  - `JsonMalformed` 也同步升级为失败，不再继续伪装成正常 `noop`
- 同一修复里补上了受控长度的 `raw_preview` 留存：
  - heartbeat detail 现在会把原始响应摘要写进 `detail_json.raw_preview`
  - 后续可以直接区分是“未知 status 枚举”“非法 JSON”还是“正常 noop”
- 这样一来，监控类 heartbeat 任务在模型已经跑完但结构化收口失败时，不会再被后台静默吞掉。
- 但最近一小时真实运行的 `run_id=1860` 与 `1865` 仍然落成 `noop + skipped_noop`，因此当前不能继续维持 `Fixed` 结论；本单需要重新回到活跃缺陷队列，直到线上落库结果与预期修复行为一致。

## 回归验证

- `cargo test -p hone-channels heartbeat_unknown_json_status_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_malformed_json_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_ -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## 后续建议

- 先核对当前运行实例是否已部署包含上述修复的 scheduler 版本；如果代码已合入但运行结果仍为 `noop + skipped_noop`，应优先排查是否存在旧二进制、分支遗漏或另一条未覆盖的 heartbeat 执行路径。
- 如果后面还观察到 heartbeat 在 `JsonNoop` 和 `JsonUnknownStatus` 之间抖动，可以继续收紧 prompt / parser 契约，让模型在末尾 JSON 收口更稳定。
- 如需更强的运维可观测性，可以再把 `parse_kind` 聚合到状态页或告警面板，而不只停留在 `cron_job_runs.detail_json`。
