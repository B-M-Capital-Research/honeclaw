# Bug: Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效

- **发现时间**: 2026-04-15 14:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近一小时同一任务持续异常：
    - `run_id=1889`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T14:00:27.633510+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1865`，`job_id=j_c1c1be63`，`job_name=存储板块加仓信号监控`，`executed_at=2026-04-16T11:00:31.512294+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `2026-04-16T11:30:28.836+08:00`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，运行日志再次记录 `parse_kind=JsonUnknownStatus`，且同轮仍被记为“心跳任务未命中，本轮不发送”
    - `run_id=1860`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:30:32.268455+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1855`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:00:32.184986+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1849`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T09:30:22.379738+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - 上一轮最近一小时巡检中的同一任务也持续异常：`run_id=1830 (01:31, JsonUnknownStatus)`、`1826 (01:01, JsonNoop)`、`1822 (00:30, JsonNoop)`
    - 往前回溯同一任务仍可见连续多次 `JsonUnknownStatus`：`run_id=1813 (23:00)`、`1806 (22:00)`、`1791 (21:00)`、`1787 (20:31)`、`1781 (20:01)`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 14:00:27.632` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 14:00:27.632` 同轮 `raw_preview` 仍直接输出 11 只股票的“当前价格 vs 触发价”分析，但末尾没有合法状态 JSON
    - `2026-04-16 14:00:27.632` 同轮已不再打印“心跳任务未命中，本轮不发送”，而是升级为 `parse failure escalated`
    - `2026-04-16 10:30:32.267` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:00:31.512` `job_id=j_c1c1be63` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:30:28.836` `job_id=j_ab7e8fb1` 再次出现 `parse_kind=JsonUnknownStatus`，随后仍打印 `心跳任务未命中，本轮不发送`
    - `2026-04-16 10:00:32.184` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 10:00:32.184` 同轮 `raw_preview` 直接列出 11 只股票的“当前价格 vs 触发价”，但仍未返回合法状态 JSON
    - `2026-04-16 09:30:22.379` 同一任务上一轮仍为 `parse_kind=JsonUnknownStatus`
    - `2026-04-16 01:31:19.950` 与 `01:01:16.495` 说明上一轮巡检窗口里也一直在 `JsonUnknownStatus / JsonNoop` 之间漂移
  - 最近半小时新增样本：
      - `2026-04-16 16:31:18.288` `job_id=j_ab7e8fb1` 再次记录 `parse_kind=JsonUnknownStatus`，并升级为 `execution_failed + skipped_error`
      - `2026-04-16 17:01:30.375` 同一任务又恢复为 `JsonNoop + skipped_noop`
      - `2026-04-16 17:01:47.317` `job_id=j_977ac60c`（`AAOI_动态监控`）也新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:16.099` `job_id=j_654aef9b`（`小米30港元破位预警`）新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:28.801` `job_id=j_ab7e8fb1` 再次记录 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:42.919` `job_id=j_c1c1be63`（`存储板块加仓信号监控`）同样新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 18:01:42.443` `job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`）恢复为 `JsonNoop + skipped_noop`
      - `2026-04-16 18:01:48.288` `job_id=j_977ac60c`（`AAOI_动态监控`）仍落成 `JsonUnknownStatus + execution_failed`
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
4. 当前线上已经有一部分实例把这类解析失败升级为 `execution_failed + skipped_error`，但模型输出本身仍然频繁落入 `JsonUnknownStatus`。
5. 结果是用户侧仍然收不到提醒；区别只是从“静默伪装成 noop”变成了“后台记失败但仍未恢复功能”。

## 期望效果

- heartbeat 任务应稳定返回可解析的结构化状态，至少能明确区分 `triggered`、`noop`、`error`。
- 当模型输出不符合约定时，调度器不应静默吞掉，而应记录可追踪错误并进入可观测状态。
- 对“监控提醒”类任务，解析失败至少应进入 `execution_failed` 或运维可见告警，而不是伪装成正常 `noop`。

## 当前实现效果

- `Monitor_Watchlist_11` 在 `2026-04-16 14:00:27` 再次落回 `JsonUnknownStatus`；与 `10:00`、`10:30`、`11:30` 一样，模型已经完成逐项价格判断，但末尾仍未收口成合法状态 JSON，说明问题仍在当前活跃时段持续出现。
- `web.log` 在 `2026-04-16 10:00:32.184` 明确记录 `parse_kind=JsonUnknownStatus`；同一轮 `raw_preview` 已经枚举 11 只股票的实时价格和触发价，但因为没有落成合法状态 JSON，最终仍被当成 `noop / skipped_noop` 静默吞掉。
- 到 `14:00` 这一轮，线上行为已从 `noop / skipped_noop` 变成 `execution_failed / skipped_error`，说明“不要静默吞掉未知状态”的修复开始在当前实例生效；但结构化收口本身仍未修好，所以该缺陷不能关闭。
- 到 `16:31` 这一轮，`Monitor_Watchlist_11` 又一次落成 `JsonUnknownStatus + execution_failed`；到 `17:01` 它短暂恢复为 `JsonNoop + skipped_noop`，说明问题不是线性修复，而是在相邻轮次之间抖动。
- `17:01:47` 的样本显示 `AAOI_动态监控` 也开始产出 `JsonUnknownStatus + execution_failed`；到 `17:31`，同类问题又继续扩散到 `小米30港元破位预警` 与 `存储板块加仓信号监控`。
- 到 `18:01` 这一轮，同批 heartbeat 已出现明显分化：`Monitor_Watchlist_11` 与 `小米30港元破位预警` 都恢复为 `JsonNoop + skipped_noop`，但 `AAOI_动态监控` 仍继续落成 `JsonUnknownStatus + execution_failed`。
- 最新日志还显示 `AAOI_动态监控` 在 `parse failure escalated` 之后仍打印 `心跳任务未命中，本轮不发送`，说明“未知状态已升级为失败”和“渠道日志仍按 noop 口径描述”之间还存在观测口径不一致。
- 这说明当前并不是单个 watchlist prompt 失配，而是 heartbeat 输出协议在多条不同模板的监控任务上都可能失去稳定收口。
- 对照 `11:30` 的最新日志、`run_id=1860`（`10:30`）、`1855`（`10:00`）、`1849`（`09:30`）和更早的 `01:31` 记录可以看到，这类 heartbeat 已经连续多轮维持同一症状，并没有随着时间窗切换自然恢复。
- 这也说明问题不只是“偶发返回乱码”，而是模型已经完成了业务判断，却在最后结构化封装一步失配，监控链路因此丢失了本该可追踪的判定结果。
- 数据库没有保存可供人工直接复核的最终文本预览，导致一旦进入 `JsonUnknownStatus`，排障信息同时丢失。
- 由于最近样本已经同时出现 `parse_kind=JsonUnknownStatus + execution_failed` 与下一轮自动恢复为 `JsonNoop`，当前缺陷的状态应理解为“部分止血但强烈抖动”：错误不再总是伪装成 noop，但 heartbeat 仍会在不同任务、不同轮次上失去稳定的结构化收口。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户依赖 heartbeat 监控来发现触发条件，一旦解析失败被静默当成 `noop`，就可能漏掉本应触发的提醒。
- 即便 14:00 这一轮已升级为 `execution_failed + skipped_error`，用户侧仍旧拿不到监控结果，所以功能损失没有消失，只是可观测性有所改善。
- 问题影响的是“自动监控是否按约工作”，会直接破坏任务可信度，因此定级为 `P2`，而不是只影响表达质量的 `P3`。
- 由于系统对用户和运维都没有显式失败信号，这类问题更容易长期潜伏。

## 根因判断

- heartbeat 输出协议对模型返回格式过于脆弱，出现非标准 JSON 或状态枚举漂移时，会落入 `JsonUnknownStatus`。
- 最近一小时同一任务在相邻轮次间会在 `JsonNoop` 与 `JsonUnknownStatus` 之间来回抖动，说明除了单个任务 prompt 外，解析器对“先给分析过程、最后未严格收口到状态 JSON”的输出也缺少足够稳健的兼容或强制约束。
- 最近一小时新增的 `AAOI_动态监控`、`小米30港元破位预警` 与 `存储板块加仓信号监控` 样本说明这不是 `Monitor_Watchlist_11` 单任务 prompt 特例，而是 heartbeat 输出协议对多条不同模板的监控任务都不够稳健。
- `18:01` 批次里同组任务已经出现“部分恢复、单点残留”的抖动形态，进一步说明问题不只是某个固定任务模板写坏，而是 heartbeat 输出协议缺少稳定的最终收口约束。
- 调度器曾把“无法识别状态”错误地归并进 `noop` 路径，造成功能性失败被静默吞掉；而 14:00 的运行结果表明，这一收口正在部分修正，但尚未彻底消除所有旧路径或旧实例。
- 渠道侧日志仍沿用“心跳任务未命中，本轮不发送”的 noop 文案，说明即使数据库台账已按 `execution_failed` 落账，部分运行日志和可观测口径还没有完全跟上新的失败语义。
- 现有落库字段只保留 `parse_kind` 与字符数，没有把原始响应片段保留下来，进一步放大了排障盲区。

## 修复情况（2026-04-16，待重新验证）

- `crates/hone-channels/src/scheduler.rs` 已把 heartbeat 的解析失败从静默 `noop` 分支中拆出：
  - `JsonUnknownStatus` 现在会返回 `error`，由各渠道 scheduler 落库为 `execution_failed + skipped_error`
  - `JsonMalformed` 也同步升级为失败，不再继续伪装成正常 `noop`
- 同一修复里补上了受控长度的 `raw_preview` 留存：
  - heartbeat detail 现在会把原始响应摘要写进 `detail_json.raw_preview`
  - 后续可以直接区分是“未知 status 枚举”“非法 JSON”还是“正常 noop”
- 这样一来，监控类 heartbeat 任务在模型已经跑完但结构化收口失败时，不会再被后台静默吞掉。
- 到 `2026-04-16 14:00` 的 `run_id=1889`，未知状态已经落成 `execution_failed + skipped_error`，说明这部分止血开始在线上生效。
- 但更早轮次的 `run_id=1860` 与 `1865` 仍落成 `noop + skipped_noop`，而且 `14:00` 这一轮依然持续产出 `JsonUnknownStatus`；因此本单只能更新为“状态有变化但问题仍活跃”，不能关闭或降级。

## 回归验证

- `cargo test -p hone-channels heartbeat_unknown_json_status_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_malformed_json_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_ -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## 后续建议

- 先核对当前运行实例是否都已部署包含上述修复的 scheduler 版本；14:00 这轮已经看到 `execution_failed + skipped_error`，但更早轮次仍是 `noop + skipped_noop`，说明版本或路径可能处于混合态。
- 如果后面还观察到 heartbeat 在 `JsonNoop` 和 `JsonUnknownStatus` 之间抖动，可以继续收紧 prompt / parser 契约，让模型在末尾 JSON 收口更稳定。
- 如需更强的运维可观测性，可以再把 `parse_kind` 聚合到状态页或告警面板，而不只停留在 `cron_job_runs.detail_json`。
