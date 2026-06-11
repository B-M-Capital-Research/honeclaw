# Bug: Heartbeat 金价阈值提醒把旧日期/非当前窗口价格当作当前触发价送达

- **发现时间**: 2026-05-27 19:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 修复记录（2026-06-11 03:03 CST）

- 已补齐 heartbeat 价格时间戳 guard 的未来日期分支：价格阈值触发文案若把未来日期价格包装成“当前价 / 现报 / 数据时间”证据，本轮同样抑制送达并继续落 `failure_kind=stale_price_timestamp`，避免把未来窗口或错窗口价格当作当前阈值触发依据。
- 新增回归覆盖：
  - `heartbeat_future_price_timestamp_trigger_is_suppressed`
- 验证：`cargo test -p hone-channels heartbeat_stale_price_timestamp --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_prompt_clarifies_price_threshold_semantics --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_near_threshold_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。

## 修复记录（2026-05-28 03:11 CST）

- 已修复。heartbeat `JsonTriggered` 送达前新增旧日期价格 guard：价格阈值触发文案若把当前 / 最新 / 现报价格绑定到明显过旧的显式价格日期，则本轮抑制送达并写入 `failure_kind=stale_price_timestamp`、`stale_price_timestamp` 与 `stale_price_timestamp_suppressed=true`，避免把历史价格包装成当前阈值触发证据。
- 新增回归覆盖：
  - `heartbeat_stale_price_timestamp_trigger_is_suppressed`
  - `heartbeat_recent_price_timestamp_trigger_is_allowed`
  - `heartbeat_prompt_clarifies_price_threshold_semantics`
- 验证：`cargo test -p hone-channels heartbeat_stale_price_timestamp --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_prompt_clarifies_price_threshold_semantics --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_near_threshold_ --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`、`rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs` 通过。

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
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-06-10 23:02 CST 巡检窗口：2026-06-10 19:01-23:02 CST。
  - `run_id=39847`
  - `job_name=伦敦金跌破4500提醒`
  - `actor_channel=feishu`
  - `heartbeat=1`
  - `executed_at=2026-06-10T22:30:20.885601+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - 用户可见 `response_preview` / `detail_json.scheduler.deliver_preview` 写出：`【黄金急跌预警】XAU/USD 现货黄金当前价 4161.56 美元/盎司（数据时间：2026年6月18日 北京时间 13:10，盘中日低 4130.62 美元/盎司），已跌破 4500 美元/盎司阈值。`
  - 该执行窗口真实当前日期为 2026-06-10，但用户可见价格时间戳是未来日期 2026-06-18；系统仍以 `completed + sent + delivered=1` 正常送达。
- 同窗摘要：
  - 2026-06-10 19:01-23:02 CST `data/sessions.sqlite3` 有 53 个 user turn 与 54 个 assistant 记录，最近 Feishu direct / scheduler 会话均以 assistant final 收口。
  - 普通 Feishu scheduler 33 条均为 `completed + sent + delivered=1`。
  - heartbeat 新增 71 条 `noop + skipped_noop + delivered=0`、33 条 `execution_failed + skipped_error + delivered=0`、2 条 `completed + sent + delivered=1` 与 1 条 `running + pending`；除本条金价成功误送达外，其它失败仍落在既有结构化 / 状态解析信号范围。
  - 最近四小时无非文档代码提交。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-06-11 11:01 CST 巡检窗口：2026-06-11 07:01-11:01 CST。
  - `run_id=40172`
  - `job_name=伦敦金跌破4500提醒`
  - `actor_channel=feishu`
  - `heartbeat=1`
  - `executed_at=2026-06-11T10:00:48.163093+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `should_deliver=1`
  - `delivered=1`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - 用户可见 `response_preview` / `detail_json.scheduler.deliver_preview` 写出：`北京时间 2026年6月13日 20:20，现货黄金（XAU/USD）最新价格为 $4098.71/盎司。当前价格已低于您设置的 $4500 预警阈值。`
  - 该执行窗口真实当前日期为 2026-06-11，但用户可见价格时间戳是未来日期 2026-06-13；系统仍以 `completed + sent + delivered=1` 正常送达。
- 同窗摘要：
  - 2026-06-11 07:01-11:01 CST `data/sessions.sqlite3` 有 19 个 user turn 与 20 个 assistant 记录，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant final 收口。
  - 普通 scheduler 17 条为 `completed + sent + delivered=1`，另有 1 条 Discord `completed + send_failed + delivered=0`。
  - heartbeat 新增 74 条 `noop + skipped_noop + delivered=0`、29 条 `execution_failed + skipped_error + delivered=0`、2 条 `completed + sent + delivered=1` 与 1 条 `running + pending`；除本条金价成功误送达外，其它失败仍落在既有结构化 / context window / runner error 信号范围。
  - 最近四小时无新的非文档代码提交；03:04 CST 的 `ecb993aa` 已在上一轮作为未来日期 guard 修复确认，但本窗 live 样本晚于该修复确认后仍复发。

## 端到端链路

1. Feishu heartbeat scheduler 执行 `伦敦金跌破4500提醒`。
2. heartbeat runner 返回结构化触发态，scheduler 解析为 `JsonTriggered`。
3. scheduler 未校验用户可见触发证据中的价格时间戳与当前执行窗口是否一致；旧日期和未来日期都可能被当作当前触发依据。
4. Feishu 出站成功，台账记录 `completed + sent + delivered=1`。
5. 用户收到一条以 2026-04-04 价格作为 2026-05-27 当前跌破阈值证据的自动提醒。

## 期望效果

- heartbeat 价格阈值提醒只能用当前执行窗口内可核验的最新价格、价格时间戳和阈值关系触发。
- 如果数据源只返回旧日期、缺少时间戳、或价格时间戳与当前提醒窗口明显不一致，应落成 `noop` 或用户态数据不可用提示，不应发送正式触发提醒。
- 出站前应对价格类 heartbeat 做轻量时间一致性校验：当前时间、价格时间、交易时段和阈值触发条件必须彼此一致。

## 当前实现效果

- `2026-05-27 16:00 CST` 的自动提醒把 `2026年4月4日` 的 XAU/USD 价格写成当前触发价。
- `2026-06-10 22:30 CST` 的自动提醒把未来日期 `2026年6月18日` 的 XAU/USD 价格写成当前触发价。
- `2026-06-11 10:00 CST` 的自动提醒在 03:04 CST 未来日期 guard 修复确认后，仍把未来日期 `2026年6月13日` 的 XAU/USD 价格写成当前触发价。
- 链路没有失败，反而以 `completed + sent + delivered=1` 送达，说明现有校验只覆盖结构化状态和发送结果，没有覆盖价格证据的新鲜度。
- 同窗多数 heartbeat 仍在结构化解析失败 / noop 间摆动，但该样本是成功送达的用户可见错误触发，不属于单纯解析失败。

## 用户影响

- 用户可能误以为金价在 2026-05-27 16:00 CST 已经根据最新行情跌破 `$4,500`，而实际文本自带的价格日期是 `2026年4月4日`。
- 2026-06-10 22:30 CST 的复发样本则把未来日期 `2026年6月18日` 写成当前价格时间戳，会让用户以为系统持有跨日期或更高优先级的实时金价证据。
- 2026-06-11 10:00 CST 的复发样本继续把未来日期 `2026年6月13日` 写成当前最新价格时间戳，说明用户仍可能收到已送达的错误阈值触发。
- 这是自动化阈值预警，用户通常不会在收到提醒前主动提供上下文或二次确认；错误时间口径会直接影响仓位风险判断。
- 定为 `P2`：主投递链路可用，但价格阈值触发正确性被破坏，影响金融提醒的可靠性和风险管理判断；不是只影响表达观感的 P3。

## 根因判断

- heartbeat runner / prompt 允许模型在触发正文中使用与当前窗口不一致的旧价格时间戳。
- scheduler 出站前没有对价格类 `JsonTriggered` 结果做“价格时间戳是否过旧、是否未来日期、是否与当前提醒窗口一致”的硬校验。
- 2026-05-28 的修复只覆盖“早于当前北京时间的显式价格日期”，没有覆盖未来日期或其它非当前窗口时间戳，因此本轮从 `Fixed` 回退为 `New`。
- 2026-06-11 03:04 CST 的未来日期 guard 代码级修复结论没有在 10:00 CST live 样本中兑现；优先判断真实 heartbeat 运行态未加载最新修复，或仍存在未覆盖的未来日期文案形态 / 出站路径。
- 该问题不同于 `scheduler_heartbeat_unknown_status_silent_skip.md` 的结构化状态退化；本样本已经成功解析并送达。
- 该问题也不同于 `scheduler_heartbeat_near_threshold_false_trigger.md` 的阈值语义误判；本样本的核心是旧日期价格被当作当前触发证据。

## 下一步建议

- 为 heartbeat 价格阈值类任务增加出站前时间一致性 guard：若触发正文包含明确价格日期且不等于当前执行窗口日期，或落在明显未来日期 / 过旧日期，降级为 `noop` 或 `execution_failed`，并写入 `failure_kind=stale_price_timestamp` 或更准确的时间戳错误分类。
- 在 heartbeat prompt 中要求价格触发必须同时输出 `price_timestamp`，且不能用过期价格作为当前触发依据。
- 增加回归样本，覆盖 `XAU/USD ... 现报 $4483.12（2026年4月4日）` 在 `2026-05-27` 执行时不得送达，以及 `XAU/USD ... 当前价 4161.56（数据时间：2026年6月18日）` 在 `2026-06-10` 执行时不得送达。

## 复发记录

- 2026-06-10 23:02 CST 状态从 `Fixed` 回退为 `New`：
  - 22:30 CST `伦敦金跌破4500提醒` `run_id=39847` 成功送达，正文把 `2026年6月18日 北京时间 13:10` 的价格时间戳作为当前跌破阈值证据。
  - 这不是单纯格式问题，而是自动金融阈值提醒的触发证据与执行窗口不一致，仍按功能性 `P2` 处理；非 P1，不创建 GitHub Issue。
- 2026-06-11 11:01 CST 再次复发：
  - 10:00 CST `伦敦金跌破4500提醒` `run_id=40172` 成功送达，正文把 `2026年6月13日 20:20` 的价格时间戳作为当前最新金价和跌破阈值证据。
  - 该样本晚于 2026-06-11 03:04 CST 未来日期 guard 修复确认，说明 live 路径仍会把未来日期价格证据送达给用户；状态保持 `New`，仍为功能性 `P2`；非 P1，不创建 GitHub Issue。
