# Bug: Heartbeat 定时任务命中 MiniMax HTTP 发送失败后缺少自动重试与降级，提醒整轮失败

- **发现时间**: 2026-04-17 16:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
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

- 当前 heartbeat 链路在 MiniMax HTTP 发送失败后直接落为 `execution_failed + skipped_error`，没有自动重试。
- 从 15:30 的正常 `JsonNoop` 到 16:01 的传输失败之间，没有任务配置变更，说明当前更像是上游/网络抖动没有被 scheduler 吸收。
- 用户本轮虽然没有被误报为成功，但提醒任务仍未完成，功能链路已中断。
- 该问题与 heartbeat 输出协议抖动不同：这里不是 `JsonUnknownStatus`，而是请求尚未完成就直接在传输层失败。

## 用户影响

- 这是功能性缺陷。heartbeat 的价值在于定时自动检查并在触发时提醒，一旦 HTTP 传输失败整轮退出，用户就会失去这一轮监控覆盖。
- 之所以定级为 `P2`，是因为当前证据更像是可恢复的上游/网络抖动，而不是持续性全量不可用；但它已经直接影响任务是否成功执行，不能按 `P3` 质量问题处理。
- 若同类失败在多条 heartbeat 任务间扩散，用户会把提醒缺席误判为“条件未触发”，进一步削弱任务可信度。

## 根因判断

- 直接触发点是 heartbeat scheduler 调用 MiniMax `https://api.minimaxi.com/v1/chat/completions` 时发生 HTTP 传输失败。
- 现有 scheduler 链路对这类传输错误缺少自动重试与降级策略，因此单次抖动就会让整轮执行失败。
- 该问题与直聊搜索阶段的 `minimax_search_http_transport_failure_no_retry` 共享上游故障形态，但发生在独立的 heartbeat 执行链路上，说明“缺少吸震”并不是直聊专属问题。

## 修复进展

- 截至 2026-04-19 01:02，仓库主线仍无法证明这条缺陷已经收口：最新 `TEM破位预警` 真实样本再次命中同一 `error sending request for url (...)`。
- 本轮巡检时工作区保持干净，未见可据此认定“修复已在本地完成但未提交”的新增证据；因此仍只能按 `Fixing` 持续跟踪，而不能升级为 `Fixed`。
- 由于 heartbeat scheduler 与直聊搜索阶段共用 MiniMax / OpenAI-compatible provider，后续若主线引入 provider 级重试或 fallback，才可能同时吸收这条 heartbeat 故障。

## 后续观察点

- 待补丁提交并进入仓库主线后，继续巡检 `cron_job_runs` 是否还出现 `error sending request for url (...)` 的同类 heartbeat 失败样本。
- 评估 scheduler 是否仍需要 provider fallback，或在失败时保留更清晰的运维侧告警聚合。
- 后续巡检继续关注是否还有其它 heartbeat job 在 `cron_job_runs` 中出现相同 `error sending request for url (...)`，若扩散则考虑提升优先级。
