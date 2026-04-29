# Bug: ASTS heartbeat 把“接近 8% 警戒阈值”直接当作已触发并送达用户

- **发现时间**: 2026-04-29 10:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9818`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T09:31:25.539312+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `detail_json.scheduler.raw_preview={"status":"noop"}`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9844`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T10:01:20.670987+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`最新价格 $71.88，相对昨收 $77.20 跌幅 -6.89% ... 触发原因：单日涨跌幅（跌）接近 8% 警戒阈值，且距离 8% 仅差约 1.1 个百分点`
  - 同一条消息还把 `盘中低点 $71.00`、`日内振幅 7.81%` 与 FCC / BlueBird 7 旧事件一并拼入触发文案，但正文没有给出任何真实越过 `8%` 的新证据。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 09:31:25.539` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonNoop`，并写出 `心跳任务未命中，本轮不发送`。
  - `2026-04-29 10:01:17.536` 同一 job 又记录 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都把 `跌幅 -6.89%` 包装成“接近 8% 警戒阈值”，随后实际投递。
- 相关缺陷对照：
  - [`scheduler_heartbeat_orcl_intraday_range_false_trigger.md`](./scheduler_heartbeat_orcl_intraday_range_false_trigger.md) 已修复的是“把日内高低点/振幅误当成涨跌幅阈值”。
  - [`scheduler_watchlist_near_threshold_false_trigger.md`](./scheduler_watchlist_near_threshold_false_trigger.md) 是多标的 watchlist 把“接近阈值”包装成触发。
  - 本次样本发生在单标的 heartbeat `ASTS 重大异动心跳监控`，且正文直接以“接近 8%”作为触发理由，属于新的独立受影响链路。

## 端到端链路

1. Feishu heartbeat scheduler 触发 `ASTS 重大异动心跳监控`。
2. runner 查询最新 ASTS 行情与旧新闻背景。
3. `09:31` 窗口同一 job 仍正常返回 `noop`。
4. `10:01` 窗口在最新价格仅 `-6.89%`、仍未越过 `8%` 阈值时，把“接近阈值”收口成 `{"status":"triggered"}`。
5. scheduler 消费该结果后按 `completed + sent + delivered=1` 正式向用户发送告警。

## 期望效果

- 当 heartbeat 条件写的是“单日涨跌幅（跌）达到 8%”时，只有真实越过阈值才允许返回 `triggered`。
- 若仅接近阈值，最多只能作为风险观察或上下文说明，不应进入最终发送链路。
- 同一 job 不应在前一窗口 `noop`、后一窗口没有新增越线证据的情况下，把“接近阈值”直接升级成正式提醒。

## 当前实现效果

- `2026-04-29 10:01` 的 `ASTS 重大异动心跳监控` 把 `跌幅 -6.89%` 解释成“接近 8% 警戒阈值”，并成功送达。
- 这条文案没有声称价格真的越过 `8%`，而是明确承认“仅差约 1.1 个百分点”，却仍返回 `JsonTriggered`。
- 同一 job 在 `09:31` 还只是 `noop`，说明当前链路会把“接近阈值”的自然语言风险提示直接升级成用户可见触发告警。

## 用户影响

- 用户会收到并不存在的 ASTS 触发提醒，以为“单日跌幅达到 8%”这条监控条件已经满足。
- 该问题会直接影响监控可信度和用户后续交易/关注决策，属于功能性告警误报，因此定级为 `P2`。

## 根因判断

- 初步判断不是发送链路或通用 JSON 解析失败，而是 heartbeat 模板仍允许模型把“接近阈值”“建议关注风险”这类观察性表达直接收口成 `triggered`。
- 这与已修复的 ORCL/ASTS 高低点口径混算不同；本次样本里正文已经明确承认没有达到 `8%`，说明缺口更偏向“缺少 triggered 前的数值硬校验”。
- 同时它与 watchlist 的近阈值误报表现相似，提示“接近阈值也算触发”的语义漂移并不只存在于多标的 watchlist。

## 下一步建议

- 为 heartbeat `triggered` 结果增加机器可校验的数值字段，例如 `metric`, `threshold`, `observed_value`, `comparison_passed`，并在发送前校验。
- 在 ASTS / ORCL / watchlist 这类价格阈值模板里明确禁止把“接近阈值”“距离阈值不远”“建议关注波动”解释成 `triggered`。
- 为单标的 heartbeat 增加回归样本：当最新涨跌幅仅 `-6.89%`、阈值为 `-8%` 时必须返回 `noop` 或独立的 `near_threshold`，不得发送正式提醒。
