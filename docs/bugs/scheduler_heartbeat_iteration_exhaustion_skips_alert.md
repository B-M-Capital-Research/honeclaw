# Bug: Heartbeat 重大事件监控触发 `已达最大迭代次数 6` 后整轮跳过，用户收不到应发提醒

- **发现时间**: 2026-04-20 06:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=7931`，`job_id=j_671d3cd3`，`job_name=小米破位预警`，`executed_at=2026-04-27T20:00:23.688655+08:00`
    - 本轮再次落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`should_deliver=0`、`delivered=0`
    - `error_message=max_iterations_exceeded:6`
    - 对比同任务紧邻窗口：
      - `run_id=7906`，`executed_at=2026-04-27T19:30:11.861359+08:00`，同一 job 仍是另一类 heartbeat 结构化失败：`heartbeat 输出不是结构化 JSON，任务已标记失败`
      - `run_id=7929`，`executed_at=2026-04-27T20:00:11.297995+08:00`，同一用户另一条阈值 heartbeat `小米30港元破位预警` 也继续落成 `heartbeat 输出不是结构化 JSON，任务已标记失败`
    - 这说明最近一小时内，同一批 heartbeat 任务仍会在 `max_iterations_exceeded:6 + skipped_error` 与 `PlainTextSuppressed` 类结构化失败之间交替出现；用户侧依然无法区分“条件未命中”还是“这一轮根本没跑完”
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-27 20:00:23.688` `cron_job_runs` 对应台账最终落成 `小米破位预警 -> max_iterations_exceeded:6`
    - 同窗 `20:00:11.296` 的 `小米30港元破位预警` 与 `20:00:09.199` 的 `CAI破位预警` 仍在输出“高于触发价/条件未触发”的自然语言 noop，被收口成另一类 heartbeat 非结构化失败
    - 说明这次 20:00 失败不是整批 scheduler 停摆，而是 heartbeat/function-calling 链路在同一批次里继续混出“迭代耗尽”和“结构化坏态”两种失败形态
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=7642`，`job_id=j_671d3cd3`，`job_name=小米破位预警`，`executed_at=2026-04-27T13:30:21.145372+08:00`
    - 本轮再次落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`should_deliver=0`、`delivered=0`
    - `error_message=max_iterations_exceeded:6`
    - 对比同任务紧邻窗口：
      - `run_id=7619`，`executed_at=2026-04-27T13:00:15.526996+08:00`，仍是 `noop + skipped_noop`
      - `run_id=7665`，`executed_at=2026-04-27T14:00:23.959272+08:00`，又漂移成 `heartbeat 输出不是结构化 JSON，任务已标记失败`
    - 这说明最近一小时内同一 heartbeat job 仍会在正常 `noop`、`max_iterations_exceeded:6 + skipped_error` 与下一窗的结构化失败之间摆动；用户侧依然无法区分“条件未命中”还是“这一轮根本没跑完”
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-27 13:30:21.146` 连续记录：
      - `run_finish ... success=false error="max_iterations_exceeded:6"`
      - `runner_error ... error="max_iterations_exceeded:6"`
      - 随后直接 `心跳任务未命中，本轮不发送: job=小米破位预警`
    - 同窗 `13:30:21.143-13:30:21.144` 还先连续出现 Tavily `usage limit` 告警，但最终 `web_search` 仍回落成 `tool_execute_success`；说明当前主问题不是独立检索中断，而是 heartbeat/function-calling 链路自身再次撞到迭代上限
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6693`，`job_id=j_671d3cd3`，`job_name=小米破位预警`，`executed_at=2026-04-26T15:00:45.699117+08:00`
    - 本轮再次落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`should_deliver=0`、`delivered=0`
    - `error_message=max_iterations_exceeded:6`
    - 对比同任务紧邻窗口：
      - `run_id=6678`，`executed_at=2026-04-26T14:30:11.313085+08:00`，仍是 `noop + skipped_noop`
      - 同一 `15:00` 批次其余 heartbeat 大多继续是 `noop + skipped_noop`，说明不是整批 scheduler 停摆，而是同一 job 再次单独撞到迭代上限
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-26 15:00:45.697-15:00:45.698` 连续记录：
      - `run_finish ... success=false error="max_iterations_exceeded:6"`
      - `runner_error ... error="max_iterations_exceeded:6"`
      - 随后直接 `心跳任务未命中，本轮不发送: job=小米破位预警`
    - 同一时间窗前后，`RKLB异动监控`、`TEM大事件心跳监控`、`ASTS 重大异动心跳监控` 与 `持仓重大事件心跳检测` 仍在 `PlainTextSuppressed` / `JsonEmptyStatus` 坏态下收口为 `noop`；这进一步说明 `max_iterations=6` 仍会和 heartbeat 结构化坏态交替出现
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6416`，`job_id=j_671d3cd3`，`job_name=小米破位预警`，`executed_at=2026-04-26T02:30:35.863853+08:00`
    - 本轮落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`should_deliver=0`、`delivered=0`
    - `error_message=max_iterations_exceeded:6`
    - 对比同任务紧邻窗口：
      - `run_id=6400`，`executed_at=2026-04-26T02:00:13.646457+08:00`，仍是 `noop + skipped_noop`
      - `run_id=6420`，`executed_at=2026-04-26T03:00:13.951450+08:00`，又回到 `noop + skipped_noop`
    - 这说明最近一小时内同一 heartbeat 模板仍会在正常 `noop` 与 `max_iterations_exceeded:6 + skipped_error` 之间抖动；用户侧无法区分是“条件未命中”还是“这一轮根本没跑完”
  - 最新真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4715`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-23T01:00:21.205650+08:00`
    - 本轮落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`should_deliver=0`、`delivered=0`
    - `error_message=max_iterations_exceeded:6`，`detail_json={"heartbeat_model":"MiniMax-M2.7-highspeed"}`
    - 对比同一任务前一窗口 `run_id=4698`（`2026-04-23T00:30:06.470670+08:00`）仍是 `noop + skipped_noop`，说明该链路仍会在正常 `noop` 与触顶失败之间抖动
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3618`，`job_id=j_818f0150`，`job_name=TEM大事件心跳监控`，`executed_at=2026-04-20T21:01:11.025741+08:00`
    - 本轮再次落成 `execution_status=execution_failed`、`message_send_status=skipped_error`、`delivered=0`
    - `error_message=已达最大迭代次数 6`
    - 对比同任务前后窗口：
      - `run_id=3599`，`executed_at=2026-04-20T20:30:17.969968+08:00`，仍是 `noop + skipped_noop`
      - 同一 `21:00` 批次里的 `run_id=3612/3613`（`Monitor_Watchlist_11`、`ORCL 大事件监控`）又同时落成 `JsonUnknownStatus + execution_failed`
      - 这说明最新真实窗口里，heartbeat 不只是单条任务持续卡死，而是同一批次里同时出现“结构化状态退化”和“迭代耗尽”两种失败形态；用户侧更无法区分本轮是未触发还是链路根本没跑完
  - `data/runtime/logs/acp-events.log`
    - `2026-04-20 21:01:11.025` 对应台账已落成 `TEM大事件心跳监控 -> 已达最大迭代次数 6`
    - 同批次前序日志还保留了 `21:00:23.979` `Monitor_Watchlist_11` 与 `21:00:25.848` `ORCL 大事件监控` 的 `parse failure escalated`
    - 这说明 `TEM` 任务并不是单独因为调度整体停摆而失败，而是在同一轮 heartbeat 坏态里独立撞到 `max_iterations=6`
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

- `2026-04-27 21:00` 的 `小米破位预警` 最新样本说明，这条缺陷在本轮最近一小时仍活跃：`run_id=7984` / `job_id=j_671d3cd3` 落成 `execution_failed + skipped_error + delivered=0`，`error_message=heartbeat 输出不是结构化 JSON，任务已标记失败`，说明同一 job 在 `20:00` 刚出现 `max_iterations_exceeded:6` 后，下一窗口又漂移回另一类 heartbeat 结构化失败。
- 这意味着“达到最大迭代次数 6 后静默跳过”的根因并没有独立消失，而是继续与 `PlainTextSuppressed` 类 heartbeat 坏态在同一任务上交替出现；用户依然无法区分这类窗口究竟是未触发、结构化失败，还是本轮直接触顶。
- `2026-04-27 20:00` 的 `小米破位预警` 最新样本说明，缺陷在本轮最近一小时仍活跃：前一窗口 `19:30` 还是 `heartbeat 输出不是结构化 JSON`，到 `20:00` 又重新漂回 `max_iterations_exceeded:6 + skipped_error`，而同批其它 heartbeat 仍多为 plain-text/noop 结构化失败。
- 这说明 heartbeat 触顶失败不只没有止血，反而继续与公共 JSON 契约坏态交替出现；用户依然既收不到提醒，也无法从台账区分“该提醒未触发”还是“本轮根本没跑完”。
- `2026-04-27 13:30` 的 `小米破位预警` 再次把这条缺陷带回最近一小时真实窗口：前一窗口 `13:00` 还是正常 `noop`，`13:30` 直接退化成 `max_iterations_exceeded:6 + skipped_error`，到 `14:00` 又漂移成另一类结构化失败。说明 heartbeat 触顶失败截至本轮巡检仍未止血，而且会在同一 job 上与其它 heartbeat 坏态交替出现。
- `2026-04-26 15:00` 的 `小米破位预警` 再次把这条缺陷带回最近一小时真实窗口：前一窗口 `14:30` 还是正常 `noop`，`15:00` 直接退化成 `max_iterations_exceeded:6 + skipped_error`，且同批其它 heartbeat 仍多数只是 `noop`。这说明 heartbeat 触顶失败并未收口，而是在同一 job 上持续抖动复现。
- `2026-04-26 02:30` 的 `小米破位预警` 把这条缺陷重新带回最近一小时真实窗口：前一窗口 `02:00` 还是正常 `noop`，`02:30` 直接退化成 `max_iterations_exceeded:6 + skipped_error`，到 `03:00` 又回到 `noop`。这说明 heartbeat 触顶失败仍在当前生产时段活跃，并且会在同一 job 上抖动复现。
- `2026-04-23 01:00` 的 `Monitor_Watchlist_11` 再次把这条缺陷带回最近一小时真实窗口：前一轮 `00:30` 还是正常 `noop`，下一轮直接退化成 `max_iterations_exceeded:6 + skipped_error`，且 `delivered=0`。这说明 heartbeat 触顶失败仍在生产活跃，并且不局限于单只股票或单一事件监控模板。
- `2026-04-20 21:01` 的 `TEM大事件心跳监控` 再次把这条缺陷带回真实窗口：前一轮 `20:30` 还是正常 `noop`，下一轮直接退化成 `已达最大迭代次数 6 + skipped_error`，而同批次其它 heartbeat 又混有 `JsonUnknownStatus`。这说明 heartbeat 触顶失败会和结构化状态退化叠加出现。
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

- `2026-04-27 21:00` 的 `小米破位预警` 新样本说明，这个根因到本轮巡检结束时仍未从同一任务上退出活跃窗口；`20:00` 是 `max_iterations_exceeded:6`，`21:00` 立即漂移成 `heartbeat 输出不是结构化 JSON`，说明 heartbeat 触顶失败和结构化状态退化仍然共享同一条不稳定收口链路。
- `2026-04-27 20:00` 的 `小米破位预警` 新样本说明，这个根因到本轮巡检窗口仍未止血；同一 job 在 `19:30` 还是另一类 heartbeat 结构化失败，到 `20:00` 又再次独立撞到 `max_iterations=6`，说明 heartbeat 触顶失败与非结构化状态漂移仍在共享同一批次里反复交替。
- `2026-04-27 13:30` 的 `小米破位预警` 新样本说明，这个根因到本轮巡检窗口仍未止血；同一 job 在 `13:00` 还是 `noop`，到 `13:30` 又再次独立撞到 `max_iterations=6`，且 `14:00` 还会漂移成另一类 heartbeat 结构化失败，说明同一链路缺少稳定收口与预算控制。
- `2026-04-26 15:00` 的 `小米破位预警` 新样本说明，这个根因截至当前巡检窗口仍未止血；同一 job 在 `14:30` 还是 `noop`，到 `15:00` 又再次独立撞到 `max_iterations=6`，说明 heartbeat 触顶不是一次性偶发波动。
- `2026-04-26 02:30` 的 `小米破位预警` 新样本说明，这个根因不只影响“大事件监控”或多标的 watchlist；即使是单 ticker 价格阈值 heartbeat，也仍可能在正常 `noop` 与 `max_iterations=6` 静默失败之间摆动。
- `2026-04-23 01:00` 的 `Monitor_Watchlist_11` 新样本说明，这个根因也会影响多标的 watchlist heartbeat；即使前一窗口可正常 `noop`，下一窗口仍可能直接撞到 `max_iterations=6` 后静默失败。
- `2026-04-20 21:01` 的 `TEM大事件心跳监控` 样本说明，这个根因不依赖时间型 heartbeat 或 ASTS 那种重复旧事件；即使是另一条事件监控模板，也可能直接撞到 `max_iterations=6` 后静默失败。
- `2026-04-20 18:00` 的 `全天原油价格3小时播报` 新样本说明，这个根因并不依赖 ASTS 那种“旧事件反复消费”的复杂上下文；即使是时间型 heartbeat，也仍可能在本轮推理中直接撞到 `max_iterations=6` 后静默失败。
- heartbeat/function-calling 链路缺少对 `max_iterations` 触顶的专门恢复与降级处理，高概率仍沿用“直接失败并跳过发送”的默认分支。
- 从 ASTS 最新样本看，同一旧事件已经先触发“跨窗口重复送达”，随后又拖到 `已达最大迭代次数 6`，说明链路既缺少增量判断，也缺少预算控制，最终把本可快速收口的 heartbeat 任务拖成失败。
- 该问题与 `JsonUnknownStatus` 不是同一根因：本轮没有结构化解析失败日志，而是 runner 在更早阶段就直接耗尽迭代并退出。
- 该问题也不同于直聊/定时汇总里常见的 `已达最大迭代次数 8`。heartbeat 当前使用的是 `max_iterations=6`，且失败后没有用户态兜底文本，影响形态更接近“提醒静默消失”。

## 下一步建议

- 为 heartbeat 链路补专门的“达到最大迭代次数”失败兜底，至少把本轮失败显式记录为可区分的状态，并输出用户可理解的失败说明或内部重试。
- 在 heartbeat 台账里补记 `iterations`、`tool_calls`、失败阶段与关键查询摘要，避免后续再次只能看到 `heartbeat_model`。
- 为 `ASTS 重大异动心跳监控` 与 `全天原油价格3小时播报` 增加回归样本，覆盖“旧事件重复检索后触顶”和“时间型 heartbeat 触顶”两类场景。
