# Bug: Heartbeat 监控使用 `mimo-v2.5-pro` 时批量命中 `Param Incorrect` 并漏发

- **发现时间**: 2026-05-12 23:03 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: 无

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-18 03:03 CST` 复核：最近四小时真实运行窗口 `2026-05-17T23:30:09+08:00` 到 `2026-05-18T03:00:23+08:00` 又新增 `82` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 scheduler 有 `3` 条 `completed + sent + delivered=1`，最近四小时 7 个 user turn 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、飞书标签、`reasoning_content` 或 `Param Incorrect` 可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前机器没有可确认已重启到 `2026-05-15 04:05 CST` 当前 HEAD 修复后的 live 进程；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-17 23:04 CST` 复核：最近四小时真实运行窗口 `2026-05-17T19:30:09+08:00` 到 `2026-05-17T23:00:21+08:00` 又新增 `81` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 scheduler 有 `16` 条 `completed + sent + delivered=1`，最近四小时 25 个 user turn 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker、飞书标签、`reasoning_content` 或 `Param Incorrect` 可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-17 15:02 CST` 复核：最近四小时真实运行窗口 `2026-05-17T11:30:08+08:00` 到 `2026-05-17T15:01:30+08:00` 又新增 `82` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 Feishu scheduler `每日公司资讯与分析总结` 为 `completed + sent + delivered=1`，最近四小时 10 个 user turn 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-17 07:02 CST` 复核：最近四小时真实运行窗口 `2026-05-17T03:30:08+08:00` 到 `2026-05-17T07:00:19+08:00` 又新增 `79` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 Feishu scheduler `每日美股盘后收盘复盘` 为 `completed + sent + delivered=1`，最近四小时无 direct 用户新请求；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-17 03:03 CST` 复核：最近四小时真实运行窗口 `2026-05-16T23:30:08+08:00` 到 `2026-05-17T03:00:20+08:00` 又新增 `82` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 Feishu scheduler `RKLB / AAOI / TEM 每日动态监控` 均 `completed + sent + delivered=1`，最近四小时 Feishu direct 2 个用户请求均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 23:03 CST` 复核：最近四小时真实运行窗口 `2026-05-16T19:00:13+08:00` 到 `2026-05-16T23:00:22+08:00` 又新增 `91` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 scheduler 有 `16` 条 `completed + sent + delivered=1`，Feishu / Web direct 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 19:03 CST` 复核：最近四小时真实运行窗口 `2026-05-16T15:30:08+08:00` 到 `2026-05-16T19:00:22+08:00` 又新增 `80` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗无普通 scheduler 终态，最近四小时 17 个 user turn 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 15:03 CST` 复核：最近四小时真实运行窗口 `2026-05-16T11:30:09+08:00` 到 `2026-05-16T15:00:21+08:00` 又新增 `81` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 scheduler `每日公司资讯与分析总结` 有 `completed + sent + delivered=1` 终态，最近四小时 9 个 user turn 均有 assistant final 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update`、compact marker 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 11:03 CST` 复核：最近四小时真实运行窗口 `2026-05-16T07:30:07+08:00` 到 `2026-05-16T11:00:23+08:00` 又新增 `79` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗普通 scheduler 有 `10` 条 `completed + sent`，最近四小时 Feishu / Web / Discord direct 和普通 scheduler 均有 assistant 收口；assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update` 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 `hone-console-page` 仍启动于 `2026-05-13 19:28 CST`，`hone-feishu` 仍启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 07:02 CST` 复核：最近四小时真实运行窗口 `2026-05-16T03:30:09+08:00` 到 `2026-05-16T07:00:18+08:00` 又新增 `81` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 同窗仅看到 1 条普通 scheduler `每日美股盘后收盘复盘` 成功 `completed + sent + delivered=1`；最近四小时 Feishu assistant final 未命中绝对路径、工具轨迹、原始 ACP `session/update` 或飞书标签可见污染。故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前证据继续按当前机器旧/非生产运行态处理，只追加运行态观察，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-16 03:04 CST` 复核：最近四小时真实运行窗口 `2026-05-15T23:30:08+08:00` 到 `2026-05-16T03:00:19+08:00` 又新增 `81` 条 heartbeat `reasoning_content must be passed back` / `Param Incorrect` 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 其中 9 个 job 各失败 `8` 次，`TEM破位预警` 失败 `7` 次，`全天原油价格3小时播报` 失败 `2` 次；03:00 CST 同窗仍连续失败。
  - 同窗 Feishu / Web direct 均有 assistant 收口，未见新的用户可见工具轨迹、绝对路径或原始标签污染；故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - 当前 live `hone-console-page` 启动于 `2026-05-13 19:28 CST`，`hone-feishu` 启动于 `2026-05-13 21:01 CST`，早于 `2026-05-15 04:05 CST` 的当前 HEAD 修复复核；本轮按当前机器旧/非生产运行态补充证据，不把状态从 `Fixed` 回退为 `New`。
  - `2026-05-15 04:05 CST` 复核：当前 HEAD 已包含 reasoning transcript replay 修复，并且 `hone-llm` / `hone-agent` 的定向回归仍通过。本轮不再以当前机器旧/非生产运行态作为继续打开依据，状态从 `New` 更新为 `Fixed`；无关联 GitHub Issue。
  - `2026-05-15 07:02 CST` 复核：当前机器运行态在 03:00-07:00 CST 仍新增 `90` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。但 04:05 CST 已按当前 HEAD 定向回归确认 reasoning transcript replay 修复仍生效，本轮仅补充旧/非生产运行态证据，不把状态从 `Fixed` 回退为 `New`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`持仓重大事件心跳检测`、`小米30港元破位预警` 与 `全天原油价格3小时播报`。
  - 代表性窗口：04:00、04:30、05:00、05:30、06:00、06:30、07:00 持续出现同类失败；其中 9 个 job 各失败 9 次，`小米30港元破位预警` 失败 7 次，`全天原油价格3小时播报` 失败 2 次。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍有 `4` 条 `completed + sent + delivered=1`，Feishu direct 会话也有成功 assistant 回复，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-15 03:03 CST` 复核：该缺陷继续保持活跃 `New`。23:30-03:00 CST 又新增 `82` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`TEM破位预警` 与 `全天原油价格3小时播报`。
  - 代表性窗口：23:30、00:30、01:00、01:30、02:00、02:30、03:00 持续出现同类失败；03:00 CST 同窗 `run_id=21350-21360` 连续失败。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍有 `5` 条 `completed + sent + delivered=1`，Feishu direct 会话也有成功 assistant 回复，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 23:04 CST` 复核：该缺陷继续保持活跃 `New`。19:00-23:01 CST 又新增 `90` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`TEM破位预警` 与 `全天原油价格3小时播报`。
  - 代表性窗口：19:00、19:30、20:30、21:00、21:30、22:00、22:30、23:00 持续出现同类失败；9 个 job 各失败 9 次，`TEM破位预警` 失败 8 次，`全天原油价格3小时播报` 失败 1 次。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - `data/runtime/logs/*` 同窗继续记录 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro ... reasoning_content ... Param Incorrect`；普通 scheduler 与 Feishu / Web direct 仍有成功送达或回复，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 19:04 CST` 复核：该缺陷继续保持活跃 `New`。15:30-19:00 CST 又新增 `80` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`TEM破位预警` 与 `全天原油价格3小时播报`。
  - 代表性窗口：15:30、16:00、16:30、17:00、17:30、18:00、18:30、19:00 持续出现同类失败；9 个 job 各失败 8 次，`TEM破位预警` 失败 7 次，`全天原油价格3小时播报` 失败 1 次。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗未看到非 heartbeat 原油坏播报或观察池击球区送达样本；本轮新增证据继续集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 15:04 CST` 复核：该缺陷继续保持活跃 `New`。11:00-15:00 CST 又新增 `90` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`TEM破位预警`、`伦敦金跌破4500提醒` 与 `全天原油价格3小时播报`。
  - 代表性窗口：11:00、11:30、12:00、12:30、13:00、13:30、14:00、14:30、15:00 持续出现同类失败；8 个 job 各失败 9 次，`TEM破位预警`、`伦敦金跌破4500提醒` 各失败 8 次，`全天原油价格3小时播报` 失败 2 次。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍可成功送达，例如 12:02 CST `每日公司资讯与分析总结` 为 `completed + sent + delivered=1`；Feishu / Web direct 会话镜像也推进到 14:58 CST，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 11:05 CST` 复核：该缺陷继续保持活跃 `New`。07:00-11:00 CST 又新增 `91` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`持仓重大事件心跳检测` 与 `全天原油价格3小时播报`。
  - 代表性窗口：07:00、07:30、08:00、08:30、09:00、09:30、10:00、10:30、11:00 持续出现同类失败；除 `全天原油价格3小时播报` 只有 1 条失败外，其余 10 个 job 各失败 9 次。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 与 Feishu direct 仍可成功送达，例如 08:30-09:31 多条普通 scheduler `completed + sent + delivered=1`，10:59-11:01 Feishu direct 持仓分析也成功发送；故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 07:06 CST` 复核：该缺陷继续保持活跃 `New`。03:00-07:00 CST 又新增 `90` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`TEM破位预警` 与 `全天原油价格3小时播报`。
  - 代表性窗口：03:00 同窗 11 条失败；03:30 同窗 9 条失败并有 2 条正常 `noop`；04:00 同窗 9 条失败；04:30、05:00、05:30、06:30、07:00 各 10 条失败；06:00 同窗 11 条失败。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍有 `Oil_Price_Monitor_Closing`、`OWALERT_PostMarket`、`科技成长赛道大盘极值与情绪监控` 以及 Feishu direct SOFI / AI 股票分析成功送达，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-14 03:03 CST` 复核：该缺陷持续保持活跃 `New`。23:30-03:00 CST 再次新增 `82` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 仍覆盖 `小米30港元破位预警`、`TEM破位预警`、`DRAM 心跳监控`、`RKLB异动监控`、`持仓重大事件心跳检测`、`伦敦金跌破4500提醒`、`TEM大事件心跳监控`、`Cerebras IPO与业务进展心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控` 与 `全天原油价格3小时播报`。
  - 代表性 run：`run_id=20029-20039` 在 23:30 CST 同窗失败；`run_id=20086-20095` 在 00:30 CST 同窗失败；`run_id=20107-20117` 在 01:00 CST 同窗失败；`run_id=20129-20139` 在 01:30 CST 同窗失败；`run_id=20151-20161` 在 02:00 CST 同窗失败；`run_id=20174-20183` 在 02:30 CST 同窗失败；`run_id=20195-20205` 在 03:00 CST 同窗失败。
  - `error_message` 继续为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍有 `科技成长股持仓买卖点日内预警`、`AAOI 每日动态监控`、`TEM 每日动态监控`、`RKLB 每日动态监控` 等 `completed + sent + delivered=1`，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `2026-05-13 23:04 CST` 复核：该缺陷从 `Closed` 调回 `New`。21:02-23:00 CST 再次新增 `51` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`。
  - 失败 job 覆盖 `TEM破位预警`、`DRAM 心跳监控`、`小米30港元破位预警`、`全天原油价格3小时播报`、`TEM大事件心跳监控`、`RKLB异动监控`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测` 与 `Monitor_Watchlist_11`。
  - 代表性 run：`run_id=19902` 到 `19912` 在 21:02 CST 同窗连续失败；`run_id=19937` 到 `19947` 在 21:30 CST 同窗失败；`run_id=19965` 到 `19975` 在 22:00 CST 同窗失败；`run_id=19987` 到 `19997` 在 22:30 CST 同窗失败；`run_id=20010` 到 `20020` 在 23:00 CST 同窗失败。
  - `error_message` 均为 `LLM 错误: : Param Incorrect (param: The reasoning_content in the thinking mode must be passed back to the API.) (code: 400)`。
  - 同窗普通 scheduler 仍有 `核心观察股池晚间快报`、`A股盘后高景气产业链推演`、`美股盘前宏观与财报日历梳理` 等 `completed + sent + delivered=1`，说明故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径。
  - `data/runtime/logs/web.log.2026-05-13` 在 `21:02`、`21:30`、`22:00`、`22:30`、`23:00` 多次记录 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro error="... reasoning_content ... Param Incorrect ..."`。
  - `2026-05-13 11:08 CST` 复核：该缺陷从 `Fixed` 更新为 `Closed`。本轮 07:30-10:00 CST 旧 live 运行态仍新增 `59` 条同类 heartbeat 失败，覆盖 `10` 个 job；错误均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`，终态仍为 `execution_failed + skipped_error + delivered=0`。
  - `2026-05-13 10:22 CST` Feishu runtime 重启后未再看到 `mimo-v2.5-pro` provider 400；10:30 CST heartbeat 窗口已有 `DRAM 心跳监控`、`持仓重大事件心跳检测`、`Cerebras IPO与业务进展心跳监控` 成功 `completed + sent + delivered=1`，其余同窗为正常 `noop + skipped_noop`；11:00 CST 窗口全部为正常 `noop + skipped_noop`。
  - `data/runtime/logs/sidecar.log` 在 10:22 CST 记录 Feishu scheduler 启动并连接，此后同窗仅看到 web_search key quota warning 与正常 heartbeat 收口，未再出现 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro ... Param Incorrect`。
  - `2026-05-13 07:08 CST` 复核：当前 HEAD 已是 `d3dffd6 Fix heartbeat mimo reasoning transcript replay`，但 live channel/backend 仍未确认重启到修复代码；本轮按旧运行态 / 未部署证据补充，不把状态从 `Fixed` 回退为 `New`。
  - 从 `2026-05-13T03:30:09+08:00` 到 `2026-05-13T07:00:18+08:00`，继续新增 `80` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`，错误均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`。
  - 失败覆盖 `DRAM 心跳监控`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`RKLB异动监控` 与 `全天原油价格3小时播报`。
  - 同窗仍有 `Oil_Price_Monitor_Closing`、`OWALERT_PostMarket`、`科技成长赛道大盘极值与情绪监控` 等非 heartbeat / 普通 scheduler 成功 `completed + sent`，故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径，而不是 Feishu 出站或 scheduler 全局停摆。
  - `data/runtime/logs/web.log.2026-05-12` 在 `2026-05-13 07:00 CST` 仍记录多个 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro failure_kind=provider_http_error error="LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)"`。
  - `2026-05-13 03:02 CST` 复核：该缺陷仍为活跃 `New`。从 `2026-05-12T23:30:12+08:00` 到 `2026-05-13T03:00:25+08:00`，最近四小时继续新增 `82` 条同类 heartbeat 失败，覆盖 `11` 个 job；其中 10 个核心 heartbeat job 在 23:30、00:00、00:30、01:00、01:30、02:00、02:30、03:00 窗口基本连续失败。
  - `2026-05-12T22:30 CST`：`7` 条 heartbeat 同窗失败，覆盖 `DRAM 心跳监控`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`。
  - `2026-05-12T23:00 CST`：`9` 条 heartbeat 同窗失败，覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`伦敦金跌破4500提醒`、`TEM大事件心跳监控`、`TEM破位预警`、`RKLB异动监控`、`小米30港元破位预警`、`TSLA 正负触发条件心跳监控`。
  - 上述失败均落成 `execution_failed + skipped_error + delivered=0`。
  - `error_message` 均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`。
  - `detail_json.failure_kind=provider_http_error`，`detail_json.heartbeat_model=mimo-v2.5-pro`。
- 最近四小时同窗仍有非 heartbeat 定时任务成功送达，例如 `run_id=19503` 的 `核心观察股池晚间快报`、`run_id=19517` 的 `科技成长股持仓买卖点日内预警`、`run_id=19521/19526/19528` 的每日动态监控均为 `completed + sent + delivered=1`，说明不是 scheduler 或 Feishu 出站全局停摆。

## 端到端链路

1. Feishu heartbeat 调度在半点 / 整点窗口批量执行多个监控 job。
2. 公共 heartbeat runner 调用 `mimo-v2.5-pro`。
3. 上游返回 `HTTP 400 Param Incorrect`。
4. 本地已能把失败分类为 `provider_http_error`，但没有自动切换模型、重试兼容参数或降级到可用 provider。
5. 多个监控 job 同窗落成 `execution_failed + skipped_error + delivered=0`，用户侧收不到本轮 heartbeat 检查结果。

## 期望效果

- Heartbeat 公共链路遇到 provider 参数兼容类 `HTTP 400` 时，应有稳定止血方案，例如切换已知可用 heartbeat provider、移除不兼容参数后重试，或将整窗故障提升为可观测告警。
- 单个 provider / 模型参数错误不应在连续半点 / 整点窗口批量压垮多个 heartbeat 监控。
- 错误分类已记录后，应进一步避免同一参数组合在同窗重复打爆所有 job。

## 当前实现效果

- 2026-05-18 03:03 CST 的最新复核继续只作为当前机器旧/非生产运行态证据：23:30-03:00 CST 仍新增 `82` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；当前机器没有可确认已重启到 2026-05-15 04:05 CST 当前 HEAD 修复后的 live 进程，本轮不重新打开。
- 2026-05-17 23:04 CST 的复核继续只作为当前机器旧/非生产运行态证据：19:30-23:00 CST 仍新增 `81` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；live 主进程与 Feishu 进程仍早于 2026-05-15 04:05 CST 的当前 HEAD 修复复核，本轮不重新打开。
- 2026-05-17 15:02 CST 的最新复核继续只作为当前机器旧/非生产运行态证据：11:30-15:01 CST 仍新增 `82` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；live Feishu 进程仍早于 2026-05-15 04:05 CST 的当前 HEAD 修复复核，本轮不重新打开。
- 2026-05-17 11:03 CST 的复核继续只作为当前机器旧/非生产运行态证据：07:30-11:00 CST 仍新增 `81` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；live 主进程仍早于 2026-05-15 04:05 CST 的当前 HEAD 修复复核，本轮不重新打开。
- 2026-05-13 23:04 CST 的最新复核显示，本单在 10:22 CST 重启后一度恢复，但 21:02-23:00 CST 又连续复发；因此关闭结论不再成立。
- 2026-05-14 03:03 CST 的最新复核显示，复发继续扩大到 23:30-03:00 CST，新增 82 条同类 heartbeat 失败；普通 scheduler 同窗仍可送达。
- 2026-05-14 07:06 CST 的最新复核显示，03:00-07:00 CST 又新增 `90` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；普通 scheduler 与 Feishu direct 同窗仍可送达。
- 2026-05-14 11:05 CST 的最新复核显示，07:00-11:00 CST 又新增 `91` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；普通 scheduler 与 Feishu direct 同窗仍可送达。
- 2026-05-14 15:04 CST 的最新复核显示，11:00-15:00 CST 又新增 `90` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；普通 scheduler 同窗仍可送达。
- 2026-05-14 19:04 CST 的最新复核显示，15:30-19:00 CST 又新增 `80` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job。
- 2026-05-14 23:04 CST 的最新复核显示，19:00-23:01 CST 又新增 `90` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；同窗普通 scheduler 与 direct 会话仍有成功收口。
- 2026-05-15 03:03 CST 的最新复核显示，23:30-03:00 CST 又新增 `82` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；同窗普通 scheduler 与 direct 会话仍有成功收口。
- 2026-05-15 07:02 CST 的最新复核仅作为当前机器旧/非生产运行态证据：03:00-07:00 CST 仍新增 `90` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；但 04:05 CST 已按当前 HEAD 回归验证确认修复成立，本轮不重新打开。
- 2026-05-16 03:04 CST 的最新复核继续只作为当前机器旧/非生产运行态证据：23:30-03:00 CST 仍新增 `81` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；live 主进程仍早于 04:05 CST 修复复核，不重新打开。
- 2026-05-16 07:02 CST 的最新复核继续只作为当前机器旧/非生产运行态证据：03:30-07:00 CST 仍新增 `81` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；本轮不重新打开。
- 2026-05-16 11:03 CST 的最新复核继续只作为当前机器旧/非生产运行态证据：07:30-11:00 CST 仍新增 `79` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败，覆盖 11 个 job；live 主进程仍早于当前 HEAD 修复复核，本轮不重新打开。
- 失败已被正确记为 `provider_http_error`，没有被伪装成 noop；但业务效果仍是本轮监控漏发。
- 同窗普通 scheduler 仍可送达，故障集中在 heartbeat provider 参数 / 模型兼容路径。

## 用户影响

- 这是功能性 bug：影响自动监控告警链路的执行与送达，不是单纯回答质量问题。
- 多条 heartbeat 在两个连续窗口漏发，用户可能错过价格、重大事件、破位和观察池监控。
- 定级为 `P2`：影响多个监控 job，但当前证据不是全渠道不可用，也没有达到 P1 的整批连续长时间全量失效；同窗仍有普通定时任务成功送达。

## 根因判断

- 直接触发点不是基础 `chat/completions` contract 本身损坏：同一 `mimo-v2.5-pro` 在单轮 text-only 与最小 tools 请求下仍可 `HTTP 200` 返回。
- 真正根因出在 heartbeat 共享的 auxiliary function-calling 多轮 transcript：`mimo-v2.5-pro` 在 thinking mode 下会返回 `reasoning_content`，而 Hone 进入下一轮 tool-result 回传时只保留了 `content/tool_calls`，没有把上一轮 `reasoning_content` 一并回传，导致上游在第二轮开始拒绝请求并报 `Param Incorrect`。
- 活跃窗口里所有失败 heartbeat 都走 `runner=function_calling`，与上述多轮工具调用链路一致；同一时间普通非 heartbeat 定时任务仍可送达，也与“heartbeat 独有 transcript 兼容问题”相符。
- 既有 `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 关注 OpenRouter credits / `HTTP 402` 额度问题；本单是不同 provider / model 的 `HTTP 400` 参数兼容问题。
- 既有 `scheduler_heartbeat_unknown_status_silent_skip.md` 关注模型已产出 triggered 正文但 JSON 结构坏掉导致漏发；本单在模型输出前就被 provider 拒绝，根因不同。

## 修复情况

- 2026-05-15 04:05 CST 复核当前 HEAD 的修复仍生效：function-calling agent 会把 assistant `reasoning_content` 写入后续工具轮，OpenAI-compatible raw request body 也会携带该字段。本轮没有新增代码改动；仅把 bug 状态从旧运行态证据驱动的 `New` 收敛为代码与回归验证驱动的 `Fixed`。
- 2026-05-13 已在 `agents/function_calling/src/lib.rs` 保留 assistant `reasoning_content`，并通过 `AgentMessage.metadata -> hone_llm::Message.reasoning_content` 在多轮 tool loop 中回传给上游。
- 2026-05-13 已在 `crates/hone-llm/src/openai_compatible.rs` 收口 OpenAI-compatible 非流式请求：一旦消息里出现 `reasoning_content`，改走原始 JSON 请求体并显式携带该字段；同时从响应里提取 `reasoning_content` 供下一轮继续使用。
- 2026-05-13 已把 heartbeat auxiliary function-calling 工具集收窄为 `data_fetch` / `web_search` / `portfolio` / `missed_events` / `local_*`，移除 `skill_tool`、`load_skill`、`notification_prefs`、`deep_research` 等与 heartbeat 无关的 schema，降低同类 provider 兼容风险与请求体膨胀。

## 验证

- `cargo test -p hone-llm chat_with_tools_replays_reasoning_content_in_raw_request_body -- --nocapture`
- `cargo test -p hone-agent run_replays_reasoning_content_into_followup_tool_round -- --nocapture`
- 2026-05-15 04:05 CST 复跑上述两条定向回归通过。
- `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- `cargo test -p hone-llm -p hone-agent -p hone-channels --no-run`

## 未验证项 / 后续建议

- 2026-05-13 23:04 CST 已确认真实 heartbeat 窗口复发，应重新检查 live runtime 是否实际运行了 `d3dffd6` 后的修复代码，或是否仍存在另一条 auxiliary/tool transcript 路径没有回传 `reasoning_content`。
- 2026-05-13 11:08 CST 已观察到 live 重启后的 10:30 / 11:00 heartbeat 窗口恢复，因此本单关闭。
- 后续若部署后再次出现同一 `mimo-v2.5-pro` reasoning transcript / `Param Incorrect` 失败，应优先在本单追加复发证据，而不是新建重复文档。
