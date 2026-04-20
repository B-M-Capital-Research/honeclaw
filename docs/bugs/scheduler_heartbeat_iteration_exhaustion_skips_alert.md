# Bug: Heartbeat 重大事件监控触发 `已达最大迭代次数 6` 后整轮跳过，用户收不到应发提醒

- **发现时间**: 2026-04-20 06:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3548`，`job_id=j_38745baf`，`job_name=全天原油价格3小时播报`，`executed_at=2026-04-20T18:00:32.340199+08:00`
    - 本轮再次落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`delivered=0`
    - `error_message=已达最大迭代次数 6`
    - 对比同任务前后窗口：
      - `run_id=3530`，`executed_at=2026-04-20T17:30:07.907846+08:00`，仍是 `noop + skipped_noop`
      - `run_id=3549`，`executed_at=2026-04-20T18:30:07.814055+08:00`，又恢复为 `noop + skipped_noop`
      - 这说明最新真实窗口里，heartbeat 不是一直稳定失败，而是会在正常 `noop` 与 `已达最大迭代次数 6` 的执行失败之间抖动；用户无法从行为上区分“本轮没触发”还是“本轮根本没跑完”
  - `data/runtime/logs/sidecar.log`
    - `2026-04-20 18:00` 同批 heartbeat 已启动；`cron_job_runs` 最终把 `全天原油价格3小时播报` 记成 `execution_failed + skipped_error`
    - 同一半小时窗口里其它任务既有 `noop + skipped_noop`，也有 `JsonUnknownStatus + execution_failed`，说明这不是整批 scheduler 宕掉，而是原油 heartbeat 本轮单独撞到 `max_iterations=6`
    - 最新样本再次证明：一旦 heartbeat 在推理阶段触顶，当前链路仍然只会静默跳过，不会给用户态任何失败说明或降级提醒
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3291`，`job_id=j_fc7749ca`，`job_name=ASTS 重大异动心跳监控`，`executed_at=2026-04-20T06:01:44.164566+08:00`
    - 本轮落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`delivered=0`
    - `error_message=已达最大迭代次数 6`
    - 对比同一任务前两个窗口：
      - `run_id=3270`，`executed_at=2026-04-20T05:01:30.466350+08:00`，仍是 `completed + sent + delivered=1`
      - `run_id=3281`，`executed_at=2026-04-20T05:31:19.225338+08:00`，仍是 `completed + sent + delivered=1`
    - 这说明 ASTS heartbeat 在连续两轮围绕同一 `BlueBird 7` 旧事件反复送达后，`06:01` 这一轮已经进一步退化成直接执行失败，用户侧本轮完全收不到提醒。
  - `data/runtime/logs/web.log`
    - `2026-04-20 06:00:59.684` 记录 `job_id=j_fc7749ca job=ASTS 重大异动心跳监控` 启动
    - `2026-04-20 06:01:44.162` 记录 `run_finish ... success=false error="已达最大迭代次数 6"`
    - `2026-04-20 06:01:44.163` 紧接着记录 `runner_error ... error="已达最大迭代次数 6"`
    - 同一失败窗口之后没有新的 `deliver` 日志，随后直接落成 `Feishu 心跳任务未命中，本轮不发送`
    - 对比上一窗口：
      - `2026-04-20 05:31:17.675` 仍记录 `parse_kind=JsonTriggered`，并实际执行 `deliver`
      - `2026-04-20 05:01:29.559` 也仍记录 `parse_kind=JsonTriggered`，并实际执行 `deliver`
    - 这说明 `06:01` 的坏态不是“同一旧事件被继续重报”，而是链路在 search / reasoning 阶段直接耗尽迭代预算，连结构化收口都没有完成。
  - 历史同类 heartbeat 证据：
    - `cron_job_runs.run_id=1429`，`job_name=全天原油价格3小时播报`，`executed_at=2026-04-12T18:00:46.520085+08:00`，同样落成 `execution_failed + skipped_error`，`error_message=已达最大迭代次数 6`
    - `cron_job_runs.run_id=442`，`job_name=全天原油价格3小时播报`，`executed_at=2026-04-07T09:01:34.791427+08:00`，也同样是 `execution_failed + skipped_error`，`error_message=已达最大迭代次数 6`
    - `data/runtime/logs/web.log` 对应保留了 `2026-04-07 09:01:34.790` 与 `2026-04-12 18:00:46.513` 的 `HeartbeatDiag runner_error ... 已达最大迭代次数 6`
    - 这说明“heartbeat 任务达到最大迭代次数后直接跳过、没有用户态降级”并不是 ASTS 单任务特例，而是 heartbeat/function-calling 链路的独立历史根因。

## 端到端链路

1. Heartbeat 调度按时启动 `ASTS 重大异动心跳监控`。
2. 任务进入 `function_calling` runner，继续围绕 `BlueBird 7` 事件进行检索和推理。
3. 本轮在完成最终结构化结果前耗尽 `max_iterations=6`，runner 直接返回 `已达最大迭代次数 6`。
4. scheduler 仅把本轮记成 `execution_failed + skipped_error`，随后跳过投递。
5. 用户侧既收不到最终提醒，也没有收到可理解的失败提示，只看到这条监控在本轮静默失效。

## 期望效果

- Heartbeat 任务即便在 search / reasoning 阶段耗尽迭代，也应输出稳定的用户态降级结果，而不是整轮静默跳过。
- 对已经在前一轮识别过的事件，链路应能复用已有判断或快速收口，避免在同一旧事件上额外消耗迭代预算。
- `cron_job_runs.detail_json` 至少应记录本轮 `iterations`、`tool_calls`、失败阶段等诊断信息，便于区分“解析失败”“传输失败”和“迭代耗尽”。

## 当前实现效果

- `2026-04-20 18:00` 的 `全天原油价格3小时播报` 再次把这条缺陷带回最近一小时真实窗口：前一轮 `17:30` 还是正常 `noop`，`18:00` 直接退化成 `已达最大迭代次数 6 + skipped_error`，到 `18:30` 又回到 `noop`。这说明 heartbeat 触顶失败仍在生产活跃，只是故障对象从早晨的 `ASTS` 再次漂移回历史老问题任务 `原油播报`。
- `ASTS 重大异动心跳监控` 在 `05:01`、`05:31` 两轮还会重复送达同一 `BlueBird 7` 旧事件，但到 `06:01` 已经直接退化成 `已达最大迭代次数 6` 的执行失败。
- 最新这轮失败没有像 `JsonUnknownStatus` 那样留下 `parse_kind`、`raw_preview` 或 `deliver_preview`，`detail_json` 只剩 `heartbeat_model`，说明 heartbeat 迭代耗尽时当前台账几乎没有可用于快速定位的执行细节。
- 历史上 `全天原油价格3小时播报` 已至少两次出现同样的 `已达最大迭代次数 6 + skipped_error`，证明 heartbeat 链路早就存在“达到上限后直接静默失败”的公共缺口。
- 之所以定级为 `P2`，是因为这已经影响功能链路而不是单纯质量波动: 用户依赖 heartbeat 自动提醒来捕获事件，但本轮任务直接失败且完全没有送达，监控能力实际中断。

## 用户影响

- 用户会把这类监控理解为“持续运行并在有结果时提醒”，但最新样本显示它会在关键窗口直接静默失败。
- 对 ASTS 这类事件密集型监控而言，前一轮还在重复送达，下一轮就突然彻底失声，会让用户无法判断是“无新增事件”还是“系统本轮根本没跑完”。
- 问题影响的是自动提醒主链路，因此不是单纯的内容质量或措辞问题。

## 根因判断

- `2026-04-20 18:00` 的 `全天原油价格3小时播报` 新样本说明，这个根因并不依赖 ASTS 那种“旧事件反复消费”的复杂上下文；即使是时间型 heartbeat，也仍可能在本轮推理中直接撞到 `max_iterations=6` 后静默失败。
- heartbeat/function-calling 链路缺少对 `max_iterations` 触顶的专门恢复与降级处理，高概率仍沿用“直接失败并跳过发送”的默认分支。
- 从 ASTS 最新样本看，同一旧事件已经先触发“跨窗口重复送达”，随后又拖到 `已达最大迭代次数 6`，说明链路既缺少增量判断，也缺少预算控制，最终把本可快速收口的 heartbeat 任务拖成失败。
- 该问题与 `JsonUnknownStatus` 不是同一根因：本轮没有结构化解析失败日志，而是 runner 在更早阶段就直接耗尽迭代并退出。
- 该问题也不同于直聊/定时汇总里常见的 `已达最大迭代次数 8`。heartbeat 当前使用的是 `max_iterations=6`，且失败后没有用户态兜底文本，影响形态更接近“提醒静默消失”。

## 下一步建议

- 为 heartbeat 链路补专门的“达到最大迭代次数”失败兜底，至少把本轮失败显式记录为可区分的状态，并输出用户可理解的失败说明或内部重试。
- 在 heartbeat 台账里补记 `iterations`、`tool_calls`、失败阶段与关键查询摘要，避免后续再次只能看到 `heartbeat_model`。
- 为 `ASTS 重大异动心跳监控` 与 `全天原油价格3小时播报` 增加回归样本，覆盖“旧事件重复检索后触顶”和“时间型 heartbeat 触顶”两类场景。
