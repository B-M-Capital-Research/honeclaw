# Bug: Heartbeat 触发提醒把实际执行时间写成错误的北京时间

- **发现时间**: 2026-05-29 15:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，当前不是 P1。

## 修复记录（2026-05-29 16:35 CST）

- 已修复 heartbeat 用户可见触发时间口径漂移：heartbeat prompt 现在显式注入“本轮权威检查时间（北京时间）”，并要求 `message` 中的检查/触发时间必须使用该权威时间；市场时段、数据时间或美东盘前/盘后不得写成另一个“北京时间触发”。
- 出站前新增轻量归一化：若 `JsonTriggered` 正文出现类似 `北京时间 HH:MM ...监控/检查/心跳/任务触发`，且该时间与 scheduler 当前北京时间不一致，会把该触发时间归一到 scheduler 权威检查时间，并在 metadata 中记录 `beijing_trigger_time_normalized=true` 与原始时间。
- 回归验证：`cargo test -p hone-channels heartbeat_normalizes_conflicting_beijing_trigger_time --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_ --lib -- --nocapture` 通过。
- 状态更新为 `Fixed`；后续如当前 HEAD 运行态仍出现 heartbeat 把美东/UTC/数据时间错误标成“北京时间触发”，再用新样本重新打开。

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=36255`
  - `job_id=j_bb4bbb99`
  - `job_name=AI与科技持仓观察关键事件心跳提醒`
  - `actor_channel=web`
  - `executed_at=2026-05-29T11:31:32.698046+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `response_preview` / `detail_json.scheduler.deliver_preview` 开头写成 `2026年5月29日 北京时间 04:00 盘后监控触发。已核验事实...`
- 最近四小时巡检窗口 `2026-05-29 11:02-15:03 CST`
  - 按消息时间共有 47 个 user turn 与 47 个 assistant final，最新活跃会话均已 assistant final 收口。
  - 普通 scheduler 2 条 `completed + sent + delivered=1`，未见 `commodity_causality_guarded=true`。
  - Heartbeat 新增 1 条 `completed + sent + delivered=1`、81 条 `execution_failed + skipped_error + delivered=0`、40 条 `noop + skipped_noop + delivered=0`。
  - Assistant final 污染扫描未命中空回复、本机绝对路径、`rawOutput`、`tool_call`、`session/update`、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400 Bad Request` 或 `open_id cross app`。

## 端到端链路

1. Web heartbeat scheduler 在 `2026-05-29 11:31 CST` 执行 `AI与科技持仓观察关键事件心跳提醒`。
2. Heartbeat runner 返回 `JsonTriggered`，scheduler 将结果落成 `completed + sent + delivered=1`。
3. 送达正文开头却把触发时间写为 `北京时间 04:00`。
4. 该时间与 `cron_job_runs.executed_at=2026-05-29T11:31:32+08:00` 不一致，用户可见提醒的时间口径错误。

## 期望效果

- Heartbeat 触发提醒应使用调度器权威执行时间或明确的数据时间字段，不能把 UTC 时间、市场时段说明或模型推断时间写成“北京时间”。
- 如果正文需要区分数据时间、交易时段与触发时间，应分别标注，例如“执行时间”“数据口径时间”“美东盘后”。

## 当前实现效果

- 本轮 heartbeat 内容已成功触发并送达，但用户可见首句把实际 `11:31 CST` 执行写成 `北京时间 04:00`。
- 当前证据只覆盖一条 Web heartbeat 成功送达样本；同窗直聊与普通 scheduler 没有同类时间口径污染。

## 用户影响

- 用户看到的 heartbeat 触发时间与系统实际执行时间不一致，可能误判提醒的新鲜度和所处交易时段。
- 该问题不影响主功能链路：任务有正常执行、解析、落库和送达；没有错误投递对象、没有漏发、没有把工具原始输出暴露给用户，也没有直接给出错误交易指令。
- 因此本轮定级为 P3：它是用户可见输出质量 / 时间口径问题，而不是调度、投递、数据安全或交易正确性链路失效。

## 根因判断

- 初步判断是 heartbeat 模型在生成 `JsonTriggered` 正文时把 UTC 时间、市场时段或内部数据时间错误表述为“北京时间”。
- Scheduler 送达前目前没有校验触发正文里的显式北京时间是否与 `executed_at` 一致，也没有强制区分执行时间和数据时间。

## 下一步建议

- 在 heartbeat prompt 或输出 schema 中显式传入并要求使用 `executed_at_beijing`，同时禁止模型自行换算“北京时间”。
- 在 scheduler 出站前增加轻量校验：若 `JsonTriggered` 正文出现“北京时间 HH:MM”且与 `executed_at` 偏差明显，降级为待复核或重写时间口径。
- 后续巡检优先观察其它 `JsonTriggered + delivered=1` heartbeat 是否继续出现类似 UTC/CST 混淆，再决定是否提升严重等级。
