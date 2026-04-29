# Bug: Watchlist heartbeat 会把“接近阈值”误判成已触发，价格仍高于配置线也会发提醒

- **发现时间**: 2026-04-29 08:02 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10650`
  - `job_id=j_ab7e8fb1`
  - `job_name=Monitor_Watchlist_11`
  - `executed_at=2026-04-30T02:02:44.245952+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 直接写出：`【ASTS 触发提醒】... 最新价 $69.51，已跌破触发价 $69.83`
  - 这说明最新复发已跨日持续到 `2026-04-30 02:02`，而且 watchlist 仍把“刚好接近配置线”的比较结果直接写成“已跌破触发价”并正式送达。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 02:02:41.951` 同一 `job_id=j_ab7e8fb1` 记录 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都把 `当前价 $69.51` 写成 `已跌破触发价 $69.83`，然后执行实际投递。
  - 这说明 `2026-04-29 19:04` 补上的 watchlist 数值自检没有稳定覆盖当前线上变体；同一根因仍会在跨日窗口继续误触发。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10087`
  - `job_id=j_ab7e8fb1`
  - `job_name=Monitor_Watchlist_11`
  - `executed_at=2026-04-29T15:02:18.827501+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 直接写出：`【价格提醒】ASTS触发买入条件。当前价格$71.88，已低于触发价$69.83`
  - 这次不再只是“距触发价仅差 2.9%”，而是把 `71.88 > 69.83` 的事实直接改写成“已低于触发价”，随后正式送达，说明同一 watchlist 误触发链路在最新窗口仍活跃，而且错误表述更激进。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 15:02:16.188` 同一 `job_id=j_ab7e8fb1` 记录 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都把 `当前价格$71.88` 写成 `已低于触发价$69.83`，然后执行实际投递。
- 最近一小时同链路对照：
  - `2026-04-29 14:30:35.580` 同一 `Monitor_Watchlist_11` 还正常落成 `{"status":"noop"}` 并记录 `心跳任务未命中，本轮不发送`。
  - 仅半小时后 `15:02` 就在没有真实跌破 `69.83` 的前提下改成 `triggered + sent`，说明这不是稳定穿线后的连续提醒，而是 watchlist 条件判断再次漂移。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9692`
  - `job_id=j_ab7e8fb1`
  - `job_name=Monitor_Watchlist_11`
  - `executed_at=2026-04-29T07:31:01.063926+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`ASTS：当前价 71.88，已跌破触发价 69.83 上方区间，距触发价仅差 2.9%`，并据此发送 `【触发提醒】监控条件已满足`。
  - 同一条消息的其它标的仍按明确阈值口径展示，如 `HIMS: 27.91（触发价≤15.75）未触发`、`MU: 504.29（触发价≤252.00）未触发`，说明这不是整条 watchlist 全部乱序，而是 ASTS 被单独放宽成“接近阈值也算触发”。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 07:30:59.518-07:30:59.519` 记录同一 job `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都是 `ASTS 当前价 71.88` 但仍宣称“已跌破触发价 69.83 上方区间，距触发价仅差 2.9%”。
  - 同窗日志还显示 `job_id=j_fc7749ca`（`ASTS 重大异动心跳监控`）在 `07:30:54.479` 只落成 `JsonNoop`，说明并没有一个独立的 ASTS 实际触线事件被其它 heartbeat 一致确认。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9720`
  - `job_id=j_ab7e8fb1`
  - `job_name=Monitor_Watchlist_11`
  - `executed_at=2026-04-29T08:01:15.313387+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - 同一 job 在下一个半小时窗口立刻恢复成 `{"status":"noop"}`，没有看到 ASTS 价格先跌破 `69.83` 再反弹回上的中间链路证据，进一步说明 `07:30` 的提醒更像“接近阈值误报”而不是短暂真实触发。
- 相关已修复缺陷对照：[`scheduler_heartbeat_orcl_intraday_range_false_trigger.md`](./scheduler_heartbeat_orcl_intraday_range_false_trigger.md)
  - 旧缺陷已修复的是“把日内高低点/振幅误当成涨跌幅阈值”。
  - 本次样本并非 high-low 振幅混算，而是明确写着价格仍在阈值上方，却仍把“接近阈值、进入观察区间”包装成 `triggered`，属于新的独立误报形态。

## 端到端链路

1. Feishu heartbeat scheduler 触发多标的 watchlist 任务 `Monitor_Watchlist_11`。
2. runner 拉取各标的最新价格，并生成自然语言判断。
3. ASTS 实际价格仍为 `71.88`，高于配置触发线 `≤69.83`。
4. 最终输出却把“距触发价仅差 2.9%”解释成“监控条件已满足”，并落成 `JsonTriggered`。
5. Feishu scheduler 消费该结果后成功发送用户提醒；下一窗口同一 job 又恢复 `noop`。

## 期望效果

- watchlist 中形如 `触发价≤69.83` 的条件应按配置值严格判定。
- 若价格仅接近阈值、仍高于触发线，最多只能表达为“接近观察位”，不应输出 `triggered` 或发送“条件已满足”的提醒。
- 同一任务不应在没有真实越线证据的前提下，在一个窗口发出触发提醒、下一窗口又回到 `noop`。

## 当前实现效果

- `2026-04-30 02:02` 的 `Monitor_Watchlist_11` 再次把 `ASTS 69.51` 与阈值 `≤69.83` 组织成正式 `triggered + sent`，且文案直接写成“已跌破触发价”。
- 这说明此前的 watchlist 数值自检没有稳定证明线上收口；跨日后这条链路仍会把临界附近样本直接升级成正式触发提醒。
- `2026-04-29 15:02` 的 `Monitor_Watchlist_11` 再次把 `ASTS 71.88` 与阈值 `≤69.83` 误报成“已低于触发价 $69.83”，并成功发送 `completed + sent`。
- 这说明此前的近阈值保险闸并没有稳定覆盖 watchlist 当前变体；问题不仅复发，而且已经从“接近阈值”升级成直接篡改比较方向。
- `2026-04-29 07:30` 的 `Monitor_Watchlist_11` 把 `ASTS 71.88` 与阈值 `≤69.83` 的差距 `2.9%` 包装成“已跌破触发价上方区间”，并成功发送 `completed + sent`。
- `2026-04-29 08:01` 的同一 job 又立即恢复 `noop + skipped_noop`。
- 同窗其它 ASTS heartbeat 没有给出一致的真实触发证据，说明当前 watchlist 模板允许“接近阈值”的自然语言被错误收口成正式触发结果。

## 用户影响

- 用户会收到错误的 watchlist 触发提醒，以为 ASTS 已达到预设加仓/警戒价位，实际价格仍高于配置线。
- 这会直接影响监控可信度和后续交易判断，属于功能性告警误报，因此定级为 `P2`。

## 根因判断

- 初步判断不是发送链路或通用 JSON 解析失败，而是 watchlist heartbeat 的条件语义仍被模型放宽了：从“价格必须穿过阈值”漂成“进入阈值上方观察区间也算触发”，最新样本甚至把比较方向直接写反。
- 由于输出最终仍符合结构化 `{"status":"triggered"}`，调度器没有额外做数值校验，导致错误判断被直接送达用户。
- 这与已修复的“用日内高低点/振幅代替涨跌幅阈值”不同，本次问题集中在 watchlist 模板把“接近阈值”当成“命中阈值”。

## 修复记录

- 2026-04-30 02:02 最新真实窗口确认线上仍复发：`run_id=10650` 把 `ASTS 69.51` 直接写成“已跌破触发价 69.83”并送达，说明当前 watchlist 保护没有稳定覆盖最新 prompt/文案变体；本单继续保持 `New`。
- 2026-04-29: `crates/hone-channels/src/scheduler.rs` 在 heartbeat 送达前增加近阈值保险闸：触发文案如果同时包含阈值/触发价语义与“接近、距离、仅差、仍高于、观察区间”等未越线表述，会落为 `near_threshold_suppressed`，不再作为正式提醒发送。
- 回归验证：`cargo test -p hone-channels heartbeat_watchlist_above_trigger_price_is_suppressed -- --nocapture`。
- 2026-04-29 15:02 最新真实窗口确认线上仍复发：`run_id=10087` 把 `ASTS 71.88` 明确写成“已低于触发价 69.83”并送达，说明当前保护没有稳定覆盖 watchlist 最新 prompt/文案变体；本单状态改回 `New`。
- 2026-04-29 19:04: 本轮在近阈值保险闸中加入 watchlist 数值自检：当消息声明低于/跌破 `触发价≤...`，但同一文本里的当前价仍高于触发价时，调度器会把该 `triggered` 输出抑制为 `near_threshold_suppressed`。回归覆盖 `当前价格$71.88，已低于触发价$69.83` 变体；验证命令：`cargo test -p hone-channels heartbeat_ -- --nocapture`。

## 后续建议

- 后续仍可把 watchlist 条件升级成机器可校验字段，例如 `comparator`, `threshold`, `observed_value`, `distance_pct`，进一步减少模型自由文本判断空间。
