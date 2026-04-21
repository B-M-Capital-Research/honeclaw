# Bug: Heartbeat 定时任务命中 MiniMax HTTP 发送失败后缺少自动重试与降级，提醒整轮失败

- **发现时间**: 2026-04-17 16:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixing
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-04-21 15:30-16:01 最新巡检样本：
    - `15:30:07` 窗口里，`run_id=4008/4009/4010/4011/4012` 分别对应 `RKLB异动监控`、`ORCL 大事件监控`、`全天原油价格3小时播报`、`小米30港元破位预警`、`TEM大事件心跳监控`，全部落成 `execution_failed + skipped_error + delivered=0`
    - 错误体相同：`LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - `16:00:07` 窗口里，`run_id=4018/4019`（`TEM大事件心跳监控`、`ASTS 重大异动心跳监控`）再次落成同一 MiniMax HTTP 传输失败
    - `16:01:07`，`run_id=4025`（`CAI破位预警`）继续单点复现同一错误
    - 这说明 14:30 之后没有进入稳定恢复；同类传输失败仍会在多个 heartbeat 模板间成批或单点复现。
  - `data/runtime/logs/web.log`
    - `2026-04-21 15:30:07.100 -> 15:30:07.103` 连续记录多条 heartbeat `run_finish ... success=false error="LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)"`
    - `2026-04-21 16:00:07.154` 与 `16:00:07.159` 再次记录 `TEM大事件心跳监控`、`ASTS 重大异动心跳监控` 相同失败
    - `2026-04-21 16:01:07.677` 又记录 `CAI破位预警` 相同失败，每次失败后仍以 `[Feishu] 心跳任务未命中，本轮不发送` 收口。
  - 2026-04-21 14:30-15:00 最新巡检样本：
    - `run_id=3988`（`RKLB异动监控`，`executed_at=2026-04-21T14:30:06.999855+08:00`）再次落成 `execution_failed + skipped_error + delivered=0`
    - `error_message=LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 但到 `15:00` 同一 `RKLB异动监控` 已不再是 HTTP 传输失败，而是切换成 `JsonUnknownStatus`；说明 MiniMax 传输失败仍会单点复现，但最新窗口主要故障又漂移回结构化输出契约。
  - 2026-04-21 14:00 最新巡检样本：
    - `run_id=3978`（`RKLB异动监控`，`executed_at=2026-04-21T14:00:06.953369+08:00`）再次落成 `execution_failed + skipped_error + delivered=0`
    - `error_message=LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - `data/runtime/logs/web.log` 在 `2026-04-21 14:00:06.952` 同步记录 `run_finish ... success=false` 与 `runner_error ... model=MiniMax-M2.7-highspeed`
    - 这次不是 09:00-12:00 那种成批失败，而是单个 heartbeat job 在短时恢复观察后再次复现；因此不能把 `12:30/13:00` 两个窗口无同类错误解释为缺陷已修复。
  - 2026-04-21 11:00-12:00 最新巡检样本：
    - `11:00:05` 窗口里，`run_id=3917/3918/3919/3920/3921/3922/3923/3924` 分别对应 `CAI破位预警`、`TEM破位预警`、`RKLB异动监控`、`ORCL 大事件监控`、`小米30港元破位预警`、`TEM大事件心跳监控`、`全天原油价格3小时播报`、`ASTS 重大异动心跳监控`，全部落成 `execution_failed + skipped_error + delivered=0`，错误体相同：`LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 同一 `11:00` 窗口还出现 `run_id=3925/3926` 的 `JsonUnknownStatus` 失败，说明同批 heartbeat 除了传输失败外仍混有结构化状态退化；但本条缺陷只跟踪 MiniMax HTTP 传输失败。
    - `11:30:05` 窗口里，`run_id=3927-3936` 覆盖 `CAI破位预警`、`Monitor_Watchlist_11`、`全天原油价格3小时播报`、`TEM大事件心跳监控`、`TEM破位预警`、`ASTS 重大异动心跳监控`、`小米破位预警`、`RKLB异动监控`、`ORCL 大事件监控`、`小米30港元破位预警`，再次全部统一落成同一 MiniMax HTTP 传输失败。
    - `12:00:05` 窗口里，`run_id=3937-3946` 又覆盖相同任务族并全部 `execution_failed + skipped_error + delivered=0`，错误仍为 `https://api.minimaxi.com/v1/chat/completions` 发送失败。
    - 这说明 09:00/09:31 后故障没有自然恢复，而是持续影响至少三个后续半小时窗口。
  - 2026-04-21 09:00-09:31 最近一小时最新样本：
    - `09:00:05` 窗口里，`run_id=3873/3874/3875/3876/3877/3878/3879/3880/3881/3882`
      - 分别对应 `小米破位预警`、`全天原油价格3小时播报`、`Monitor_Watchlist_11`、`小米30港元破位预警`、`CAI破位预警`、`TEM破位预警`、`TEM大事件心跳监控`、`ASTS 重大异动心跳监控`、`ORCL 大事件监控`、`RKLB异动监控`
      - 全部统一落成 `execution_failed + skipped_error + delivered=0`
      - `error_message` 全部相同：`LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 到 `09:31:04` 窗口，`run_id=3886/3887/3888/3889/3890/3891`
      - 分别对应 `TEM破位预警`、`TEM大事件心跳监控`、`RKLB异动监控`、`ORCL 大事件监控`、`Monitor_Watchlist_11`、`小米30港元破位预警`
      - 再次统一落成相同的 `execution_failed + skipped_error`
      - 说明这不是单个 heartbeat job 偶发失败，而是在相邻两个半小时窗口里成批复现
  - `data/runtime/logs/web.log`
    - `2026-04-21 09:00:05.350 -> 09:00:05.371` 连续记录 10 条 heartbeat `run_finish ... success=false error="LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)"`
    - `2026-04-21 09:31:04.190 -> 09:31:04.200` 再次连续记录 6 条相同 heartbeat 失败
    - 每条失败后都紧跟 `[Feishu] 心跳任务未命中，本轮不发送`，说明当前链路仍把传输失败折叠成“未命中”式静默跳过，而不是自动重试或明确降级
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2695`
    - `job_id=j_27495ea4`
    - `job_name=TEM破位预警`
    - `executed_at=2026-04-19T01:02:04.297796+08:00`
    - `execution_status=execution_failed`
    - `message_send_status=skipped_error`
    - `delivered=0`
    - `error_message=LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 说明这类传输失败在最近一小时又从 `小米30港元破位预警` 扩散到 `TEM破位预警`，并未随时间自行收口
  - 运行日志：`data/runtime/logs/web.log`
    - `2026-04-19 01:00:59.578` heartbeat 启动：`job_id=j_27495ea4 job=TEM破位预警`
    - `2026-04-19 01:02:04.296` `run_finish ... success=false error="LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)"`
    - `2026-04-19 01:02:04.296` 同步记录 `runner_error ... model=MiniMax-M2.7-highspeed`
  - 同一任务前一轮对比：
    - `run_id=2678`
    - `executed_at=2026-04-19T00:31:08.921323+08:00`
    - `execution_status=noop`
    - `message_send_status=skipped_noop`
    - 说明 `TEM破位预警` 的配置与业务条件未改，只在相邻轮次遭遇 MiniMax 传输层失败
  - 更早首个复现样本：
    - `run_id=2162`
    - `job_id=j_654aef9b`
    - `job_name=小米30港元破位预警`
    - `executed_at=2026-04-17T16:01:08.495993+08:00`
    - `execution_status=execution_failed`
    - `message_send_status=skipped_error`
    - `delivered=0`
    - `error_message=LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
  - 运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-17T08:00:00.257353Z` heartbeat 启动：`job_id=j_654aef9b job=小米30港元破位预警`
    - `2026-04-17T08:01:08.494686Z` `run_finish ... success=false error="LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)"`
    - `2026-04-17T08:01:08.495282Z` `runner_error ... model=MiniMax-M2.7-highspeed`
  - 同一任务前一轮对比：
    - `run_id=2158`
    - `executed_at=2026-04-17T15:30:17.790827+08:00`
    - `execution_status=noop`
    - `message_send_status=skipped_noop`
    - `detail_json.parse_kind=JsonNoop`
    - 说明任务配置与业务条件未改，仅在相邻轮次遭遇传输层失败
  - 相关已知缺陷：
    - [`minimax_search_http_transport_failure_no_retry.md`](./minimax_search_http_transport_failure_no_retry.md) 已记录 Feishu 直聊搜索阶段的同类 MiniMax 传输失败，但当前样本发生在 heartbeat scheduler 链路，影响范围独立

## 端到端链路

1. 用户已配置 Feishu heartbeat 任务 `小米30港元破位预警`，系统按整点启动本轮检查。
2. scheduler 正常进入 MiniMax `chat/completions` 调用阶段。
3. 本轮没有拿到合法 heartbeat 结果，而是在 HTTP 发送阶段直接报 `error sending request for url (...)`。
4. 当前 heartbeat 链路没有自动重试、provider fallback，或基于上一轮状态做保守降级。
5. 本轮最终以 `execution_failed + skipped_error` 收口，用户侧没有收到提醒，也没有自动恢复。

## 期望效果

- heartbeat 定时任务遇到 `error sending request for url (...)` 这类上游传输抖动时，应至少自动重试一次，而不是立即整轮失败。
- 若重试后仍失败，系统应保留更清晰的链路级告警或统一降级策略，而不是让监控任务静默缺口扩大。
- 对同一 MiniMax 传输失败，应在 scheduler 与直聊链路之间共享吸震策略，而不是只有人工重试能恢复。

## 当前实现效果

- `2026-04-21 15:30-16:01` 最新窗口又出现多任务批量和单点 MiniMax HTTP 传输失败；失败对象覆盖 `RKLB / ORCL / 原油 / 小米 / TEM / ASTS / CAI`，说明传输吸震仍未在生产收口。
- `2026-04-21 14:30` 最新窗口又出现 `RKLB异动监控` 单点 MiniMax HTTP 传输失败；`15:00` 同任务虽转为 `JsonUnknownStatus`，但这不能证明传输吸震已收口，只能说明当前 heartbeat 故障在“传输失败”和“结构化输出失败”之间切换。
- `2026-04-21 14:00` 最新窗口又出现 `RKLB异动监控` 单点 MiniMax HTTP 传输失败；虽然不再是 09:00-12:00 的大面积批量失败，但足以推翻“短时恢复可能已收口”的判断，本单继续保持 `Fixing`。
- `2026-04-21 12:30` 与 `13:00` 两个最新 heartbeat 窗口没有继续出现 `https://api.minimaxi.com/v1/chat/completions` 的成批传输失败；同批主要退化为 `JsonUnknownStatus` 或正常 `noop/triggered`。
- 但该观察只覆盖上一次大面积失败后的两个半小时窗口，尚不足以证明重试或降级策略已经在生产收口；本单继续保持 `Fixing`，等待后续窗口确认是否稳定不再复现。
- `2026-04-21 11:00`、`11:30`、`12:00` 的最新真实窗口说明，这条缺陷仍在继续：三个窗口又至少 30 条 heartbeat run 统一命中同一 `chat/completions` 传输失败。
- 加上 `09:00` 与 `09:31` 的 16 条样本，故障已跨越多个半小时检查周期，不能按单次上游抖动处理。
- 失败对象覆盖 `ASTS / RKLB / TEM / ORCL / Watchlist / CAI / 原油 / 小米` 多种 heartbeat 模板与不同 target，不再是单个 job 抖动。
- 更关键的是，`web.log` 仍显示每次失败后直接进入 `心跳任务未命中，本轮不发送`，表明当前生产链路并没有稳定吸收这类传输抖动；README 中先前“2026-04-20 已 provider 级修复”的结论已与最新生产事实不符。
- 当前 heartbeat 链路在 MiniMax HTTP 发送失败后直接落为 `execution_failed + skipped_error`，没有自动重试。
- 从 15:30 的正常 `JsonNoop` 到 16:01 的传输失败之间，没有任务配置变更，说明当前更像是上游/网络抖动没有被 scheduler 吸收。
- 用户本轮虽然没有被误报为成功，但提醒任务仍未完成，功能链路已中断。
- 该问题与 heartbeat 输出协议抖动不同：这里不是 `JsonUnknownStatus`，而是请求尚未完成就直接在传输层失败。

## 用户影响

- 这是功能性缺陷。heartbeat 的价值在于定时自动检查并在触发时提醒，一旦 HTTP 传输失败整轮退出，用户就会失去这一轮监控覆盖。
- 之所以定级为 `P2`，是因为当前证据更像是可恢复的上游/网络抖动，而不是持续性全量不可用；但它已经直接影响任务是否成功执行，不能按 `P3` 质量问题处理。
- 若同类失败在多条 heartbeat 任务间扩散，用户会把提醒缺席误判为“条件未触发”，进一步削弱任务可信度。

## 根因判断

- 最新 `09:00/09:31` 成批复现说明，至少在当前仓库对应的线上链路里，并没有看到“provider 级重试已覆盖 heartbeat”的稳定证据；如果相关修补曾存在，也没有被最新运行事实证明已经生效。
- 直接触发点是 heartbeat scheduler 调用 MiniMax `https://api.minimaxi.com/v1/chat/completions` 时发生 HTTP 传输失败。
- 现有 scheduler 链路对这类传输错误缺少自动重试与降级策略，因此单次抖动就会让整轮执行失败。
- 该问题与直聊搜索阶段的 `minimax_search_http_transport_failure_no_retry` 共享上游故障形态，但发生在独立的 heartbeat 执行链路上，说明“缺少吸震”并不是直聊专属问题。

## 修复进展

- 截至 2026-04-21 14:00，MiniMax HTTP 失败已从 09:00-12:00 的大面积成批故障回落为单点复现，但仍未完全消失；后续需继续观察是否还有新的 `chat/completions` 发送失败。
- 截至 2026-04-21 13:00，最近两个 heartbeat 窗口暂未再现 MiniMax HTTP 传输失败，但仍只算短时恢复观察；若后续连续多个窗口不再复现，可再评估从 `Fixing` 切到 `Fixed`。
- 截至 2026-04-21 12:00，仓库主线仍无法证明这条缺陷已经收口：从 `09:00` 到 `12:00` 的多个 heartbeat 真实窗口持续命中同一 `error sending request for url (...)`。
- 本轮巡检时工作区保持干净，未见能证明“线上已落地吸震补丁”的仓库内新增事实；因此本单只能恢复为 `Fixing`，不能继续记为 `Fixed`。
- 由于 heartbeat scheduler 与直聊搜索阶段共用 MiniMax / OpenAI-compatible provider，后续只有在真实生产窗口不再出现这类成批 `chat/completions` 传输失败时，才可重新评估是否关闭。

## 后续观察点

- 待补丁提交并进入仓库主线后，继续巡检 `cron_job_runs` 是否还出现 `error sending request for url (...)` 的同类 heartbeat 失败样本。
- 评估 scheduler 是否仍需要 provider fallback，或在失败时保留更清晰的运维侧告警聚合。
- 后续巡检继续关注是否还有其它 heartbeat job 在 `cron_job_runs` 中出现相同 `error sending request for url (...)`，若扩散则考虑提升优先级。
