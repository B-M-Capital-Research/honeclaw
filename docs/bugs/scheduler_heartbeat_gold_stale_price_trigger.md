# Bug: Heartbeat 金价阈值提醒把旧日期价格当作当前触发价送达

- **发现时间**: 2026-05-27 19:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=34789`
  - `job_name=伦敦金跌破4500提醒`
  - `actor_channel=feishu`
  - `heartbeat=1`
  - `executed_at=2026-05-27T16:00:22.743867+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - 用户可见 `response_preview` / `detail_json.scheduler.deliver_preview` 写出：`XAU/USD 现货黄金当前价格已跌破 $4,500 阈值，现报 $4,483.12（2026年4月4日），较昨收下跌约 0.54%。`
- 同任务在本轮 15:00-19:03 CST 窗口里的其它运行：
  - `15:00` 为 `noop + skipped_noop`
  - `15:30` 为 `execution_failed + skipped_error`
  - `16:00` 为上述坏样本 `completed + sent + delivered=1`
  - `16:30` 为 `execution_failed + skipped_error`
  - `17:00 / 17:30 / 18:00` 为 `noop + skipped_noop`
  - `18:30 / 19:00` 为 `execution_failed + skipped_error`

## 端到端链路

1. Feishu heartbeat scheduler 执行 `伦敦金跌破4500提醒`。
2. heartbeat runner 返回结构化触发态，scheduler 解析为 `JsonTriggered`。
3. scheduler 未校验用户可见触发证据中的价格时间戳与当前执行窗口是否一致。
4. Feishu 出站成功，台账记录 `completed + sent + delivered=1`。
5. 用户收到一条以 2026-04-04 价格作为 2026-05-27 当前跌破阈值证据的自动提醒。

## 期望效果

- heartbeat 价格阈值提醒只能用当前执行窗口内可核验的最新价格、价格时间戳和阈值关系触发。
- 如果数据源只返回旧日期、缺少时间戳、或价格时间戳与当前提醒窗口明显不一致，应落成 `noop` 或用户态数据不可用提示，不应发送正式触发提醒。
- 出站前应对价格类 heartbeat 做轻量时间一致性校验：当前时间、价格时间、交易时段和阈值触发条件必须彼此一致。

## 当前实现效果

- `2026-05-27 16:00 CST` 的自动提醒把 `2026年4月4日` 的 XAU/USD 价格写成当前触发价。
- 链路没有失败，反而以 `completed + sent + delivered=1` 送达，说明现有校验只覆盖结构化状态和发送结果，没有覆盖价格证据的新鲜度。
- 同窗多数 heartbeat 仍在结构化解析失败 / noop 间摆动，但该样本是成功送达的用户可见错误触发，不属于单纯解析失败。

## 用户影响

- 用户可能误以为金价在 2026-05-27 16:00 CST 已经根据最新行情跌破 `$4,500`，而实际文本自带的价格日期是 `2026年4月4日`。
- 这是自动化阈值预警，用户通常不会在收到提醒前主动提供上下文或二次确认；错误时间口径会直接影响仓位风险判断。
- 定为 `P2`：主投递链路可用，但价格阈值触发正确性被破坏，影响金融提醒的可靠性和风险管理判断；不是只影响表达观感的 P3。

## 根因判断

- heartbeat runner / prompt 允许模型在触发正文中使用与当前窗口不一致的旧价格时间戳。
- scheduler 出站前没有对价格类 `JsonTriggered` 结果做“价格时间戳是否过旧 / 是否与当前提醒窗口一致”的硬校验。
- 该问题不同于 `scheduler_heartbeat_unknown_status_silent_skip.md` 的结构化状态退化；本样本已经成功解析并送达。
- 该问题也不同于 `scheduler_heartbeat_near_threshold_false_trigger.md` 的阈值语义误判；本样本的核心是旧日期价格被当作当前触发证据。

## 下一步建议

- 为 heartbeat 价格阈值类任务增加出站前时间新鲜度 guard：若触发正文包含明确价格日期且早于当前执行日期，降级为 `noop` 或 `execution_failed`，并写入 `failure_kind=stale_price_timestamp`。
- 在 heartbeat prompt 中要求价格触发必须同时输出 `price_timestamp`，且不能用过期价格作为当前触发依据。
- 增加回归样本，覆盖 `XAU/USD ... 现报 $4483.12（2026年4月4日）` 在 `2026-05-27` 执行时不得送达。
