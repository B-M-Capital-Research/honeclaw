# Bug: 单标的 heartbeat 会把“接近阈值”直接当作已触发并送达用户

- **发现时间**: 2026-04-29 10:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=12420`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-05-01T13:01:08.230303+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前价 $82.51，单日涨跌幅 +7.13%（已接近但未达8%阈值）`
  - 同条正文继续把 `4月29日确认获得1.9亿美元国防合同` 包装成“重大订单利好”，说明同一根因在 `09:30` 之后并未消失，而是在 `13:00` 窗口再次把“未达阈值 + 旧消息背景”送成正式提醒。
- `data/runtime/logs/sidecar.log`
  - `2026-05-01 13:00:01.838` 记录同一 `job_id=j_1241aad0` heartbeat 启动。
  - `2026-05-01 13:00:47.850-13:01:08.230` 对应窗口最终落成 `completed + sent`，与 sqlite 中 `run_id=12420` 的正式送达一致。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=12263`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-05-01T09:30:27.966052+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前现货报价$82.51，相对昨收$77.02上涨7.13%，单日涨跌幅接近但未达8%阈值`
  - 同条正文还继续把 `4月29日披露Rocket Lab赢得1.9亿美元国防合同` 拼成“重大订单利好，但非当日新发消息”，说明最新窗口仍把“未达价格阈值 + 旧事件背景”包装成正式 `triggered` 提醒。
- `data/runtime/logs/web.log.2026-05-01`
  - `2026-05-01 09:30:26.415` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `单日涨跌幅接近但未达8%阈值`。
  - 同一 job 在同日更早窗口 `08:00:45.430` 与 `09:00:17.846` 又都正常落成 `parse_kind=JsonNoop` 并记录 `心跳任务未命中，本轮不发送`，说明 `09:30` 这轮不是持续越线后的稳定提醒，而是同一价格条件在相邻窗口间再次漂成正式误触发。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=11216`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-04-30T13:00:31.251295+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`RKLB触发重大订单提醒... 当前股价$77.02... 涨跌幅未超过8%阈值`
  - 同一用户直聊会话 `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 在 `12:17:02-12:29:11 CST` 刚连续反馈“这合同是什么时候的”与“所以这些老新闻不要重复发了 你昨天也发了好多次给我”，说明最近一小时用户侧已明确把这类文案识别为旧新闻噪音；但 heartbeat 仍在 `13:00` 把同一旧合同和未命中阈值的价格状态包装成正式 `triggered`。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 13:00:28.749-13:00:28.750` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `涨跌幅未超过8%阈值`。
  - 这说明 `2026-04-30 08:01` 的 RKLB 误触发并非单次波动；在用户刚明确要求“不要重复发旧新闻”后，同一 job 仍会把“旧合同 + 未命中阈值”继续升级成正式提醒。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10943`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-04-30T08:01:18.374082+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`RKLB异动提醒... 最新价$77.02，较前收$78.59下跌-2.00%，未触发涨跌幅8%阈值`
  - 这说明最新复发已经不再局限于 ASTS / ORCL，而是扩展到 `RKLB异动监控`：正文明确承认“未触发 8% 阈值”，链路仍以正式 `triggered` 提醒送达用户。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 08:01:16.470-08:01:16.473` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `未触发涨跌幅8%阈值`。
  - 这说明当前线上坏态不是单个 ASTS 模板特例，而是“正文否认命中阈值，结构化状态仍给 triggered”这条单标的 heartbeat 误报链路继续扩散。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10643`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-30T02:00:58.810628+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`ASTS 基本面积事件触发... 当前股价 $69.61（昨收 $71.88），日内跌幅 -3.16%，未触及 8% 涨跌幅阈值`
  - 这说明最新复发已经跨日延续到 `2026-04-30 02:00` 窗口，且坏态仍是 `status=triggered` 与正文结论直接自相矛盾。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 02:00:55.564-02:00:55.565` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `未触及 8% 涨跌幅阈值`。
  - 这说明 `2026-04-29 19:04` 补上的近阈值保险闸没有稳定覆盖最新 ASTS 变体；线上仍会把“未命中阈值”的文案作为正式提醒送达。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10183`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T17:01:39.662237+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`触发条件：单日涨跌幅超过 8%`，随后正文又承认 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛`，但本轮仍以正式触发提醒送达。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 17:01:34.563-17:01:34.564` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `当前跌幅未达到 8% 阈值`。
  - 这说明最新复发已不只是“接近 8% 警戒阈值”的措辞漂移，而是 `status=triggered` 与正文结论正面自相矛盾。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9912`
  - `job_id=j_39a96b7a`
  - `job_name=ORCL 大事件监控`
  - `executed_at=2026-04-29T11:30:36.068108+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前价格 $165.92，跌幅 4.07%（相对昨收 $172.96）... 该触发接近 5% 阈值，建议关注`
  - 同一 job 在下一窗口 `run_id=9941`（`2026-04-29T12:01:32.811230+08:00`）又恢复 `noop + skipped_noop`，说明这不是持续越线后的正常提醒，而是“接近 5%”被直接包装成正式触发。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 11:30:32.238-11:30:32.239` 记录同一 `job_id=j_39a96b7a` 收口为 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都明确承认只有 `跌幅 4.07%`，但仍落成正式投递。
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
  - 本次样本已覆盖两个单标的 heartbeat：`ASTS 重大异动心跳监控` 把 `-6.89%` 包装成“接近 8%”，`ORCL 大事件监控` 把 `-4.07%` 包装成“接近 5%”；两者都属于同一条“接近阈值 => triggered”链路。

## 端到端链路

1. Feishu heartbeat scheduler 触发单标的价格监控任务（如 `ASTS 重大异动心跳监控`、`ORCL 大事件监控`）。
2. runner 查询最新价格与相关背景信息。
3. 某些窗口会正常返回 `noop`。
4. 一旦自然语言里出现“接近 5% / 8% 阈值，建议关注”之类表述，链路会把未越线的观察性提示收口成 `{"status":"triggered"}`。
5. scheduler 消费该结果后按 `completed + sent + delivered=1` 正式向用户发送告警；下一窗口又可能回到 `noop`。

## 期望效果

- 当 heartbeat 条件写的是“单日涨跌幅（跌）达到 5% / 8%”时，只有真实越过阈值才允许返回 `triggered`。
- 若仅接近阈值，最多只能作为风险观察或上下文说明，不应进入最终发送链路。
- 同一 job 不应在前一窗口 `noop`、后一窗口没有新增越线证据的情况下，把“接近阈值”直接升级成正式提醒。

## 当前实现效果

- `2026-05-01 13:00` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 正式送达给用户，但正文明确承认 `单日涨跌幅 +7.13%（已接近但未达8%阈值）`，同时继续把 `4月29日` 的旧合同包装成本轮触发理由。
- 这说明 `09:30` 样本并非孤立波动；同一 job 在同一个交易日午后窗口仍会把未越线价格和旧新闻背景升级成正式告警，本单继续保持活跃 `New` 状态。
- `2026-05-01 09:30` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 正式送达给用户，但正文明确承认 `单日涨跌幅接近但未达8%阈值`，还补充 `4月29日` 旧合同只是“非当日新发消息”。
- 这说明 `2026-04-30` 标记为 `Fixed` 的修复结论没有在线上稳定生效；最新窗口里，同一 job 在 `08:00/09:00` 还能正常 `noop`，到 `09:30` 又把未达阈值的观察性提示升级成正式告警。
- `2026-04-30 13:00` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `涨跌幅未超过8%阈值`，且主题仍是用户在同一小时刚追问过时间点的旧合同。
- 这说明线上最新坏态不只是“接近阈值也算触发”，还会把已被用户指出是旧新闻的事件继续叠加到误触发告警里，形成“错误触发 + 重复旧闻”双重噪音。
- `2026-04-30 08:01` 的 `RKLB异动监控` 把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `较前收下跌 -2.00%，未触发涨跌幅8%阈值`。
- 这说明线上最新坏态已从 ASTS / ORCL 继续扩展到 RKLB：只要存在重大事件叙述，模型仍会把“事件成立但价格条件未命中”的观察性提示升级成正式触发提醒。
- `2026-04-30 02:00` 的 `ASTS 重大异动心跳监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `日内跌幅 -3.16%，未触及 8% 涨跌幅阈值`。
- 这说明线上最新坏态没有收口到单日样本，而是跨日后继续把“事件存在但价格条件未命中”的观察性提示升级成正式触发提醒。
- `2026-04-29 17:01` 的 `ASTS 重大异动心跳监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛`。
- 这说明线上最新坏态已经不只是“接近阈值也算触发”，而是触发状态与结论文本直接自相矛盾，用户会收到一条自称“已触发”但正文说“未触发”的告警。
- `2026-04-29 10:01` 的 `ASTS 重大异动心跳监控` 把 `跌幅 -6.89%` 解释成“接近 8% 警戒阈值”，并成功送达。
- `2026-04-29 11:30` 的 `ORCL 大事件监控` 又把 `跌幅 4.07%` 解释成“接近 5% 阈值”，同样成功送达；`12:01` 下一窗口立即恢复 `noop`。
- 两条文案都没有声称价格真的越过阈值，而是明确承认“接近阈值”，却仍返回 `JsonTriggered`，说明当前链路会把观察性提示直接升级成用户可见触发告警。

## 用户影响

- 用户会收到并不存在的 ASTS / ORCL 触发提醒，以为“单日跌幅达到 8% / 5%”这类监控条件已经满足。
- 该问题会直接影响监控可信度和用户后续交易/关注决策，属于功能性告警误报，因此定级为 `P2`。

## 根因判断

- 初步判断不是发送链路或通用 JSON 解析失败，而是单标的 heartbeat 模板仍允许模型把“接近阈值”“建议关注风险”这类观察性表达直接收口成 `triggered`。
- 这与已修复的 ORCL/ASTS 高低点口径混算不同；本次样本里正文已经明确承认没有达到 `5% / 8%`，说明缺口更偏向“缺少 triggered 前的数值硬校验”。
- 同时它与 watchlist 的近阈值误报表现相似，提示“接近阈值也算触发”的语义漂移并不只存在于多标的 watchlist。

## 修复记录

- 2026-05-01 13:03: 最新真实窗口再次确认本单仍活跃：`run_id=12420` 把 `RKLB异动监控` 的 `已接近但未达8%阈值` 文案再次落成 `completed + sent + delivered=1`；说明同一根因在 `09:30` 之后没有收敛，继续维持 `New`。
- 2026-05-01 10:02: 最新真实窗口再次确认本单回归：`run_id=12263` 把 `RKLB异动监控` 的 `接近但未达8%阈值` 文案又落成 `completed + sent + delivered=1`；此前 `Fixed` 结论失效，本单状态改回 `New` 并重新进入活跃队列。
- 2026-04-30 15:05: 本轮继续补强同一送达前保险闸，新增 `未触发 / 没有触发 / 尚未触发` 以及 `未超过 / 没有超过 / 尚未超过` 等直接否认触发的阈值措辞覆盖；`RKLB异动监控` 中 `未触发涨跌幅8%阈值`、`涨跌幅未超过8%阈值` 这类 `triggered` 输出会被落成 `near_threshold_suppressed`，不再投递。回归验证：`cargo test -p hone-channels heartbeat_ -- --nocapture`。
- 2026-04-30 13:00 最新真实窗口再次确认本单仍在扩散：`run_id=11216` 把 `RKLB异动监控` 的 `涨跌幅未超过8%阈值` 文案再次落成 `completed + sent + delivered=1`，且同一小时用户已直接反馈“老新闻不要重复发”；说明当前保护仍未覆盖“旧事件 + 价格条件明确否认命中”的单标的 heartbeat 变体，本单继续保持 `New`。
- 2026-04-30 08:01 最新真实窗口再次确认本单仍在扩散：`run_id=10943` 把 `RKLB异动监控` 的 `未触发涨跌幅8%阈值` 文案仍落成 `completed + sent + delivered=1`；说明当前保护没有稳定覆盖“事件触发 + 价格阈值明确否认命中”的单标的 heartbeat 新变体，本单继续保持 `New`。
- 2026-04-30 02:00 最新真实窗口再次确认 ASTS 仍复发：`run_id=10643` 在正文已明确写出 `日内跌幅 -3.16%，未触及 8% 涨跌幅阈值` 的前提下，仍落成 `completed + sent + delivered=1`；说明当前保护仍未稳定覆盖“事件触发 + 价格阈值否认命中”的跨日变体，本单继续保持 `New`。
- 2026-04-29: `crates/hone-channels/src/scheduler.rs` 在 heartbeat 送达前增加近阈值保险闸：`跌幅 -6.89% 接近 8% / 仅差约 1.1 个百分点` 这类承认未达到阈值的 `triggered` 文案会被抑制，不再进入用户可见发送链路。
- 回归验证：`cargo test -p hone-channels heartbeat_near_threshold_trigger_is_suppressed -- --nocapture`。
- 2026-04-29 17:01 最新真实窗口再次确认 ASTS 仍复发：`run_id=10183` 在正文已明确写出 `当前跌幅未达到 8% 阈值` 的前提下，仍落成 `completed + sent + delivered=1`；说明当前保护尚未覆盖“触发条件声明 + 正文否认命中”这一新变体。
- 2026-04-29 11:30-12:01 最新真实窗口仍复现回归：`run_id=9912` 把 ORCL `跌幅 4.07%` 写成“接近 5% 阈值”并送达，下一窗口 `run_id=9941` 才恢复 `noop`；说明近阈值保险闸尚未稳定覆盖所有单标的 heartbeat 变体，本单改回 `New`。
- 2026-04-29 19:04: 本轮补强同一保险闸，新增 `门槛 / 未触及 / 未命中 / 未满足 / 未达` 等否认命中措辞覆盖；`触发条件：超过 8%` 但正文写出 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛` 的 `triggered` 输出会被落成 `near_threshold_suppressed`，不再投递。回归验证：`cargo test -p hone-channels heartbeat_ -- --nocapture`。

## 后续建议

- 后续仍可把 heartbeat `triggered` 结果升级成机器可校验的数值字段，例如 `metric`, `threshold`, `observed_value`, `comparison_passed`，进一步减少模型自由文本判断空间。
- 在 ASTS / ORCL / watchlist 这类价格阈值模板里明确禁止把“接近阈值”“距离阈值不远”“建议关注波动”解释成 `triggered`。
- 为单标的 heartbeat 增加回归样本：当最新涨跌幅仅 `-6.89%` 对 `-8%`、或仅 `-4.07%` 对 `-5%` 时，必须返回 `noop` 或独立的 `near_threshold`，不得发送正式提醒。
