# Bug: ASTS 发射链路把预告与停牌前行情误报成已发射后的实时结果

- **发现时间**: 2026-04-19 14:28 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3331`，`executed_at=2026-04-20T08:01:30.414148+08:00`，在最近一小时最新窗口里再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 同轮 `response_preview` 继续写成：`ASTS 今日（4/19）BlueBird 7 由 Blue Origin New Glenn 3 任务发射升空`
    - 但这条提醒仍同时捆绑同一组停牌前 `ASTS $85.53 / 前收 $90.94 / 日跌幅约 5.95%`，说明到 `08:01` 为止，系统仍在把旧价格快照包装成事件后的市场背景，而不是明确标注“最近可得仍是停牌前价格”
  - `data/runtime/logs/web.log`
    - `2026-04-20 08:01:27.976` 的 `HeartbeatDiag deliver` 明确写出：`ASTS 今日（4/19）BlueBird 7 由 Blue Origin New Glenn 3 任务发射升空，但公司官方公告确认：卫星被上级火箭送入低于计划的轨道`
    - 同条 `deliver_preview` 继续附带 `当前价格 $85.53 / 日跌幅约 -5.95%`，说明最新真实出站文本仍把停牌前旧行情包装成事件后的“当前价格”背景
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3281`，`executed_at=2026-04-20T05:31:19.225338+08:00`，在最近一小时最新窗口里继续落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 紧接着到 `run_id=3291`，`executed_at=2026-04-20T06:01:44.164566+08:00`，又退化成 `execution_failed + skipped_error`，`error_message=已达最大迭代次数 6`
    - 但最新成功送达的 `3281` 仍继续把同一 `BlueBird 7` 低轨事件与停牌前 `ASTS $85.53 / 前收 $90.94 / 日跌幅约 5.95%` 绑在一起，说明“价格时间口径错误”在链路退化为执行失败前依旧活跃。
  - `data/runtime/logs/web.log`
    - `2026-04-20 05:31:17.675` 的 `HeartbeatDiag deliver` 仍明确写出：`BlueBird 7 已于 2026-04-19 搭乘 Blue Origin 的 New Glenn 火箭发射，但卫星被放置在【低于计划轨道】... 当前价格 $85.53，前收盘 $90.94，当日跌幅约 -5.95%`
    - `2026-04-20 06:01:44.162` 则同任务直接记录 `success=false error="已达最大迭代次数 6"`，说明这条链路在最近窗口里已经从“错误时态+错误价格口径”进一步退化成直接跑不完，但并没有证明原有事实错误已经修复。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3257`，`executed_at=2026-04-20T04:31:15.440929+08:00`，在最近一小时最新窗口里再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - `run_id=3270`，`executed_at=2026-04-20T05:01:30.466350+08:00`，仅过约 30 分钟又再次 `completed + sent + delivered=1`
    - 两轮 `response_preview` 都继续围绕同一 `BlueBird 7` 低于计划轨道旧事件展开，并继续捆绑同一组停牌前 `ASTS $85.53 / 前收 $90.94 / 日跌幅约 5.95%`
    - 这说明到 `05:01` 为止，ASTS heartbeat 仍在把旧价格快照包装成事件后的市场背景，而不是显式标注“停牌前最后可见价格”
  - `data/runtime/logs/web.log`
    - `2026-04-20 04:31:13.695` 的 `HeartbeatDiag deliver` 明确写出：`BlueBird 7 于4月19日搭乘 New Glenn 发射升空，但卫星被放置至低于计划的轨道 ... ASTS当日下跌约5.95%至$85.53`
    - `2026-04-20 05:01:29.559` 的同任务 `deliver_preview` 又继续写成 `当前价格 $85.53 / Day's Change -5.95%`，但两轮之间没有新的独立价格时间戳或新的市场开盘数据
    - 这说明当前链路仍没有把“低轨事件的事实时间”与“最近可得停牌前价格快照时间”剥离，用户侧收到的仍是旧价格被包装成事件后市场反应
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3197`，`executed_at=2026-04-20T01:31:34.070170+08:00`，在最近一小时最新窗口里再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 同轮 `response_preview` 继续写成：`BlueBird 7 已于2026年4月19日通过 Blue Origin New Glenn 火箭从 Cape Canaveral 成功发射`
    - 仅过约 30 分钟，到 `run_id=3208`，`executed_at=2026-04-20T02:02:14.687350+08:00`，同一任务又回落成 `execution_status=noop`、`message_send_status=skipped_noop`
    - 这说明到 `02:02` 为止，ASTS heartbeat 仍会在“已成功发射，应主动推送”与“无需触发，直接 noop”之间来回摇摆；而 `01:31` 那轮已发送提醒仍继续捆绑停牌前 `ASTS $85.53 / 前收 $90.94 / 日跌幅 -5.95%`
  - `data/runtime/logs/web.log`
    - `2026-04-20 01:31:34.069` 的 `HeartbeatDiag deliver` 明确写出：`ASTS 重大基本面事件触发提醒。BlueBird 7 已于2026年4月19日通过 Blue Origin New Glenn 火箭从 Cape Canaveral 成功发射`
    - `2026-04-20 02:02:14.686` 的同任务 `raw_preview` 又回落为 `parse_kind=JsonNoop`，说明相邻两个窗口对同一事件的结论仍不稳定
    - 两轮之间没有新的独立市场价格时间戳，`01:31` 那轮却继续把同一组停牌前价格写成事件触发时的市场背景，说明“旧价格包装为事件落地后反应”的错误仍未收口
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3115`，`executed_at=2026-04-19T21:30:39.536730+08:00`，在最近一小时最新窗口里再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - `run_id=3123`，`executed_at=2026-04-19T22:00:25.003693+08:00`，仅过约半小时又再次 `completed + sent + delivered=1`
    - `21:30` 这轮 `response_preview` 写成：`BlueBird 7 已于当日（美东 6:45 AM）... 发射升空`；`22:00` 这轮又写成：`BlueBird 7 按计划今日（4月19日）从肯尼迪航天中心发射`
    - 两轮都继续捆绑同一组停牌前价格 `ASTS $85.53 / 前收 $90.94 / 日跌幅约5.95%`，说明系统仍在把旧行情快照包装成事件落地后的市场背景
    - 这说明到 `22:00` 为止，ASTS heartbeat 仍会在“已发射完成”与“今日按计划发射”之间来回摆动，错误时态与价格时间口径都没有收口
  - `data/runtime/logs/web.log`
    - `2026-04-19 21:30:39.535` 的 `HeartbeatDiag deliver` 明确写出：`ASTS 重大基本面事件已触发 ... BlueBird 7 已于当日（美东 6:45 AM）... 发射升空`
    - `2026-04-19 22:00:23.490` 的 `HeartbeatDiag deliver` 又改写成：`BlueBird 7 按计划今日（4月19日）从肯尼迪航天中心发射`
    - 两轮都继续附带 `今日ASTS下跌约5.95%，报$85.53` 这组停牌前价格，说明最新窗口里系统仍没有把“事件时态”与“价格快照时间”分离
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3091`，`executed_at=2026-04-19T20:31:19.466805+08:00`，在最近一小时最新窗口里落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - `run_id=3105`，`executed_at=2026-04-19T21:02:09.819339+08:00`，仅过约半小时又再次 `completed + sent + delivered=1`
    - 两轮最新 `response_preview` 都继续写成：`BlueBird 7 卫星已于今日（2026年4月19日）成功发射升空`
    - 同时又继续搭配同一组 `ASTS $85.53 / 前收 $90.94 / 日跌幅约5.95%` 的停牌前价格，说明链路仍在把旧价格包装成“发射落地后的市场反应”
    - 这说明错误并没有随着前一轮 `19:31 -> 20:01` 的时态摇摆而收口；到 `20:31 -> 21:02` 最新窗口，系统已经连续两轮稳定输出同一错误完成态
  - `data/runtime/logs/web.log`
    - `2026-04-19 20:31` 对应 ASTS heartbeat 继续以 `parse_kind=JsonTriggered` 出站，`deliver_preview` 写成 `BlueBird 7 已于今日（2026年4月19日）...发射升空`
    - `2026-04-19 21:02` 对应 `cron_job_runs.run_id=3105` 再次把同一事件投递为“已成功发射”，并继续附带 `当前价 $85.53 / 盘中最低 $84.91 / 盘中最高 $91.77`
    - 两轮之间没有新的独立价格时间戳或新的市场开盘数据，却继续沿用同一停牌前快照描述“发射后的价格背景”
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3070`，`executed_at=2026-04-19T19:31:19.680900+08:00`，在最近一小时最新窗口里短暂回落成 `execution_status=noop`、`message_send_status=skipped_noop`
    - 仅过约 30 分钟，到 `run_id=3083`，`executed_at=2026-04-19T20:01:33.782226+08:00`，同一任务又重新落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 最新 `response_preview` 再次写成：`BlueBird 7 卫星已于北京时间2026年4月19日从佛罗里达州卡纳维拉尔角成功发射升空`
    - 但仍继续搭配同一组 `ASTS $85.53 / 前收 $90.94 / 日跌幅约5.95%` 的停牌前价格，说明这条链路依然在把旧价格包装成“发射落地后的市场反应”
    - 这说明错误并没有从 `18:31 -> 19:01` 的时态摇摆中收口，而是在 `19:31` 短暂 `noop` 后，于 `20:01` 再次回摆成“已发射完成”的确定性结论
  - `data/runtime/logs/web.log`
    - `2026-04-19 20:01:32.529` 的 `HeartbeatDiag raw_preview` 明确写出：`BlueBird 7 launch appears to have happened today`
    - 同轮 `deliver_preview` 则进一步落成“已于北京时间2026年4月19日成功发射升空”，并继续把 `当前股价 $85.53，日内跌幅约5.95%` 包装成事件后的即时背景
    - 结合 `19:31` 窗口对应 `run_id=3070` 的 `noop + skipped_noop` 可见，当前并不存在稳定的事实基线；系统只是在半小时轮询里继续把同一事件在“暂不触发”和“已确认完成”之间来回改写
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3053`，`executed_at=2026-04-19T18:31:22.747721+08:00`，在最近一小时最新窗口里先落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 同轮 `response_preview` 仍写成：`BlueBird 7 卫星计划于今日（2026年4月19日）从卡纳维拉尔角肯尼迪航天中心发射`
    - 仅过 30 分钟，到 `run_id=3063`，`executed_at=2026-04-19T19:01:27.302512+08:00`，同一任务再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 最新 `response_preview` 又改口成：`BlueBird 7 发射已完成`，并进一步宣称“当前已过窗口期，任务应已完成”
    - 这说明同一 ASTS 事件在最近一小时里仍在“计划于今日发射”与“已发射完成”之间来回摆动，错误时态并未收口
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-19 18:31:19.233` 的 `HeartbeatDiag deliver` 仍对外投递：`BlueBird 7 卫星计划于今日（2026年4月19日）... 发射`
    - `2026-04-19 19:01:24.946` 的 `HeartbeatDiag deliver` 又把同一事件改写成：`BlueBird 7 发射已完成`
    - 两轮 `deliver_preview` 都继续搭配同一组 `ASTS $85.53 / 前收 $90.94 / -5.95%` 的停牌前行情，说明链路仍在把旧价格包装成事件时态切换后的背景
    - 这组最近一小时样本进一步表明：问题不是单次错误措辞，而是同一事件在相邻窗口里被系统反复改写成互相冲突的完成态/预告态结论
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3043`，`executed_at=2026-04-19T18:00:40.304581+08:00`，在最近一小时最新窗口里再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 最新 `response_preview` 仍直接写出：`BlueBird 7卫星已于今日（4月19日）从卡纳维拉尔角成功发射`
    - 同轮继续把 `ASTS $85.53`、`日跌幅 -5.95%` 包装成同一事件后的即时背景，说明在 `17:00` 一度回落为 `noop` 后，`18:00` 又重新把旧事件+旧价格拼成“已发射后的实时提醒”
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-19 18:00:39.158` 的 `HeartbeatDiag raw_preview` 再次明确写出：`BlueBird 7于2026年4月19日成功发射`
    - `2026-04-19 18:00:39.159` 的 `HeartbeatDiag deliver` 继续把该结论投递到用户侧，说明这不是 `cron_job_runs` 的单边摘要误差，而是真实出站文本仍在复现
    - 相比 `17:00` 窗口的“4月19日（明天）发射”错误时间口径，`18:00` 样本已经再次回摆成“已成功发射”，说明当前并非稳定止血，只是在错误时态之间来回漂移
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=3012`，`executed_at=2026-04-19T16:30:37.743228+08:00`，仍落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 同轮 `response_preview` 直接写出：`BlueBird 7 卫星已于今日成功发射升空`，同时又把发射时间写成 `美国东部时间 2026年4月19日早上`，等价于在北京时间 `16:30` 仍提前把当晚窗口判成既成事实
    - 到 `run_id=3020`，`executed_at=2026-04-19T17:00:17.494841+08:00`，同一任务又回落成 `execution_status=noop`、`message_send_status=skipped_noop`
    - 但这不是修复：`17:00` 对应日志 `raw_preview` 反而把同一事件改写成 `BlueBird 7 计划于4月19日（明天）发射`，说明链路只是从“过早宣布成功”漂移成了“时间口径继续错误的 noop”
  - `data/runtime/logs/web.log`
    - `2026-04-19 16:30:37.742` 的 `HeartbeatDiag deliver` 明确写出：`BlueBird 7 卫星已于今日成功发射升空`
    - 同轮 `deliver_preview` 继续把 `当前价$85.53，前收$90.94` 包装成事件后的现时行情，说明停牌前价格仍在被错用为“发射后市场反应”
    - `2026-04-19 17:00:17.493` 的 `HeartbeatDiag raw_preview` 又写出 `BlueBird 7 计划于4月19日（明天）从卡纳维拉尔角发射`
    - 这说明同一根因在最近一小时里已经从“过早触发 completed + sent”漂移成“错误时间理解下的 noop”，并未真正收口
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
    - `2026-04-19T14:23:13.806125+08:00` 用户在 Feishu 直聊提问：`asts发射成功了告诉我一下，帮我留意着。`
    - `2026-04-19T14:26:14.901405+08:00` assistant 直接答复：`BlueBird 7 发射已确认成功`，并进一步宣称 `股价在发射成功后反而下跌了约百分之6，当前价格在 85.53 美元附近`
    - `2026-04-19T14:28:17.518908+08:00` 用户当场纠正：`你说错了 发射成功股价下跌不是这样的，现在的股价是上个星期5收盘时的价格 现在停盘中`
    - `2026-04-19T14:28:41.645159+08:00` assistant 承认：`是我在查阅数据时未严格核对时间戳，将上周五的收盘价误认为了发射后的实时市场反应`
  - `data/runtime/logs/web.log`
    - `2026-04-19 14:23:22.450`、`14:23:39.679` 搜索阶段分别执行 `web_search query="AST SpaceMobile launch success 2026 April"` 与 `web_search query="BlueBird 7 launch result AST SpaceMobile April 19 2026 successful"`
    - `2026-04-19 14:24:27.916` 搜索阶段又执行 `data_fetch snapshot ASTS`
    - `2026-04-19 14:26:14.906` 本轮仍以 `success=true` 收口并正常送达，说明错误不是链路失败，而是把检索与快照结果错误消费成了“已发射后的实时市场反应”
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `job_name=ASTS 重大异动心跳监控`
    - `run_id=2973`，`executed_at=2026-04-19T14:30:40.993802+08:00`，仍只把事件写成“`BlueBird 7 已正式对外官宣将于今日发射`”
    - `run_id=2983`，`executed_at=2026-04-19T15:01:04.584892+08:00`，却直接升级成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 同一条 `response_preview` 明确写出：`BlueBird 7卫星已成功发射`，并同时又写出 `美东时间4月19日06:45（对应北京时间4月19日晚）`，等价于把北京时间晚上才会发生的窗口，在 `15:00` 就判成了既成事实
    - `run_id=2988`，`executed_at=2026-04-19T15:30:17.357261+08:00`，在 `15:30` 仍继续落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`
    - 这轮 `response_preview` 依旧把事件写成：`条件2已满足：BlueBird 7 卫星今日发射`，并继续配上停牌前价格 `ASTS 今日下跌约5.95%（当前价$85.53，前收$90.94）`，说明错误不仅没有在 `15:00` 后被止血，还继续跨半小时窗口重复投递
  - `data/runtime/logs/web.log`
    - `2026-04-19 15:01:03.458` 的 `HeartbeatDiag raw_preview` 明确自相矛盾：
      - 先写 `BlueBird 7于2026年4月19日使用New Glenn火箭成功发射`
      - 又写 `发射窗口是美东时间早上6:45（对应北京时间4月19日晚7:45左右）`
      - 还写 `当前北京时间是2026年4月19日15:00`
    - 该轮仍被解析成 `parse_kind=JsonTriggered` 并送达用户，说明错误结论在出站前没有被时间一致性检查拦住
    - `2026-04-19 15:30:14.362` 的 `HeartbeatDiag deliver` 继续写出：`条件2已满足：BlueBird 7 卫星今日发射`
    - 同一轮 `deliver_preview` 仍把 `当前价$85.53，前收$90.94` 包装成“今日下跌约5.95%”的现时行情，说明“预告事件 + 停牌前价格”的错误拼接还在活跃链路中继续投递
  - 相关已有缺陷：
    - [`scheduler_heartbeat_retrigger_duplicate_alerts.md`](./scheduler_heartbeat_retrigger_duplicate_alerts.md) 记录的是同一旧事件被跨窗口反复重发；本单关注的是“事件时态和价格时间戳本身被读错”，属于独立的正确性缺陷

## 端到端链路

1. 系统读取 ASTS 的发射相关新闻和 `data_fetch` 行情快照。
2. 这些证据里同时包含“今日发射窗口/计划发射”的预告信息，以及停牌前最后一个可见市场价格。
3. direct 与 heartbeat 两条链路都没有把“事件预告时间”与“当前北京时间”做一致性核对。
4. 最终用户侧收到的文本把“计划于今晚发射”或“停牌前收盘价”误解释成“已经发射成功后的实时市场反应”。

## 期望效果

- 对发射、财报、业绩会这类时间敏感事件，系统应先判断当前时间是否已越过事件窗口，再决定能否使用过去式结论。
- 当行情快照来自停牌前或上个交易日收盘价时，输出应明确标注“最新可得价格仍为停牌前价格”，不能把它写成事件落地后的即时反应。
- direct 问答与 heartbeat 提醒都应共享同一套时间一致性约束，避免一条链路修正后另一条链路继续误报。

## 当前实现效果

- 到 `2026-04-20 08:01` 的最新窗口，ASTS heartbeat 又一次把 `BlueBird 7` 低轨事件写成“今日（4/19）发射升空”的触发提醒，并继续附带同一组停牌前 `ASTS $85.53 / -5.95%` 价格；这说明误报链路并没有因为前一小时的执行失败或 `noop` 抖动而停止，仍在真实出站文本里活跃。
- 到 `2026-04-20 05:31 -> 06:01` 的最新窗口，ASTS heartbeat 已进一步出现状态漂移：`05:31` 这轮仍继续把同一 `BlueBird 7` 低轨事件与停牌前旧价格一起送达，`06:01` 又直接退化成 `已达最大迭代次数 6` 的执行失败；这说明链路并不是“已停止误报”，而是在旧误报未收口的前提下继续恶化。
- 到 `2026-04-20 04:31 -> 05:01` 的最新窗口，heartbeat 仍连续两轮围绕同一 `BlueBird 7` 低轨事件送达，并继续沿用同一组停牌前 `ASTS $85.53 / -5.95%` 价格来描述事件背景，说明错误价格时间口径到现在依旧没有收口。
- 到 `2026-04-20 01:31 -> 02:02` 的最新窗口，heartbeat 仍在同一旧事件上继续摇摆：`01:31` 把 `BlueBird 7` 写成“已成功发射并主动提醒”，`02:02` 又回退成 `noop`，但发送那轮仍继续搭配同一组停牌前价格，说明错误时态和价格时间口径到现在都还在活跃链路里并存。
- 到 `2026-04-19 21:30 -> 22:00` 的上一组窗口，heartbeat 仍在同一旧事件上继续摇摆：`21:30` 把 `BlueBird 7` 写成“已于当日发射升空”，`22:00` 又回退成“按计划今日发射”，但两轮都继续搭配同一组停牌前价格，说明错误时态和价格时间口径到现在都还在活跃链路里并存。
- 当前 direct 链路会在用户询问发射进展时，直接把预告新闻和旧行情快照收口成“已发射成功 + 发射后股价下跌”。
- 当前 heartbeat 链路在 `14:30` 仍把事件描述为“今日发射”，到 `15:00` 又把同一事件直接升级成“已成功发射”，`15:30` 还继续沿用同一错误条件反复送达，而日志里自己同时承认对应北京时间应是当晚。
- 到 `16:30`，heartbeat 仍继续以 `completed + sent` 对外投递“已于今日成功发射升空”；到 `17:00` 才回落成 `noop`，但同轮 `raw_preview` 又把同一事件写成“4月19日（明天）发射”，说明错误并未修复，只是从误报成功切换成了错误时间口径下的静默跳过。
- `18:00` 的最新 heartbeat 又重新回摆成 `completed + sent`，再次把 `BlueBird 7 已于今日成功发射` 投递给用户，说明所谓 `17:00` 的短暂 `noop` 不是修复，而是错误结论在不同窗口间来回抖动。
- 到 `18:31 -> 19:01` 的最新窗口，这种抖动仍在继续：前一轮还写“计划于今日发射”，后一轮又切回“发射已完成”，同一事件的错误时态在半小时轮询里继续摇摆。
- `19:31 -> 20:01` 的最新窗口说明，这种摇摆还会继续跨窗口自我放大：`19:31` 短暂回落成 `noop` 后，`20:01` 又重新把同一 `BlueBird 7` 事件写成“已于北京时间 2026-04-19 成功发射升空”，同时仍沿用停牌前旧价格来描述所谓“发射后的市场表现”。
- 这不是纯粹的措辞波动，而是用户态事实判断前后矛盾，且已经跨越直聊和定时提醒两条链路。
- 之所以定级为 `P2`，是因为问题直接伤害了投资提醒的事实正确性：用户收到的是可以驱动操作判断的结论性文本，而不是单纯的风格偏差或格式瑕疵。

## 用户影响

- 用户可能把“今晚才发生的发射窗口”误认为已经确认完成，并基于错误时点判断仓位或风险。
- 用户还会把停牌前价格误读成事件后的市场投票，从而错误理解市场对发射结果的反应。
- 同一根错误若继续存在，会同时污染直聊问答和自动提醒，削弱整条 ASTS 跟踪链路的可信度。

## 根因判断

- 当前链路缺少“事件时间是否已到达”的一致性校验，模型只要看到“今天发射/发射成功”的搜索片段，就可能直接输出完成态。
- 行情快照的时间语义没有被强制暴露到最终答案，导致停牌前价格、上周五收盘价被误包装成事件落地后的即时反应。
- 最近一小时里同一 heartbeat 在 `17:00` 与 `18:00` 之间在“明天发射”与“今日已成功发射”之间摇摆，说明除了缺少时间校验外，系统也没有为同一事件维持稳定的事实状态基线。
- 最新 `18:31 -> 19:01` 窗口又把同一事件在“今日计划发射”与“当前已完成”之间来回改写，说明该事实状态基线到现在仍未稳定。
- `19:31 -> 20:01` 窗口进一步表明，哪怕中间短暂回落成 `noop`，下一轮仍会再次把同一旧事实升级成“已完成”，且继续消费同一停牌前价格快照；也就是说，系统既没有稳定的时间一致性校验，也没有稳定的事件状态记忆。
- 该缺陷与消息投递、工具失败、权限或 compact 无关；search/answer 都正常完成，问题发生在工具结果解释与最终结论收口阶段。

## 下一步建议

- 为事件型问答和 heartbeat 补一条共享回归：当日志里同时出现“当前北京时间 < 事件北京时间”时，禁止输出“已成功发射/已落地”等完成态。
- 在行情工具结果进入 answer 前强制暴露交易时段与快照时间，若市场停牌或价格来自上一交易日，应明确标记为“停牌前最后价格”。
- 针对 `ASTS / BlueBird 7` 这类已复现样本增加回归用例，覆盖“预告新闻 + 停牌前价格 + 用户追问发射进展”的组合场景。
