# Bug: Heartbeat 定时任务命中 MiniMax HTTP 发送失败后缺少自动重试与降级，提醒整轮失败

- **发现时间**: 2026-04-17 16:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New

## 修复进展（2026-04-26）

- **2026-06-27 11:01 CST 回退为 `New`**：
  - `data/runtime/logs/web.log.2026-06-27` 与 `data/runtime/logs/hone_cli_screen.log`
    - 11:00-11:01 CST heartbeat function-calling 路径成批出现 `MiniMax-M2.7-highspeed` / OpenAI-compatible `error sending request for url (https://api.minimaxi.com/v1/chat/completions)`。
    - 同窗先记录多条 `raw chat_with_tools transport error, retrying`，说明 provider 级短重试已触发；随后仍有多个 Feishu / Web heartbeat 任务落成 `runner_error + execution_failed`，覆盖 `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`NVDA 关键事件心跳提醒`、`小米30港元破位预警`、`SIVE POET/Nokia/1.6T DFB 心跳检测`、`RKLB异动监控`、`全天原油价格3小时播报` 等。
    - 错误均停留在 runtime 日志 / scheduler 失败台账侧，未见用户可见 assistant final 外露原始 provider 错误。
  - 会话质量对照：
    - `data/sessions.sqlite3` 仍停在 2026-06-17；本轮以 runtime 日志和 `data/runtime/logs/acp-events.log` 重构真实运行态。
    - 同窗 ACP 可见 16 次 `session/prompt`、11 个 prompt session、16 次 `stopReason=end_turn`、0 个 response error；没有证据显示 Feishu direct、Web direct 或出站全局不可用。
  - 判断：
    - 2026-06-21 的 `Later` 结论明确写入“若已确认加载当前代码且网络 / 供应商状态稳定的新运行态中仍成批复现，再重新打开”。本轮日志已经显示 provider 短重试触发后仍成批失败，满足回退条件。
    - 该问题影响 heartbeat 监控覆盖但未造成错投、数据破坏或全渠道不可用；严重等级保持功能性 `P2`，非 P1，不创建 GitHub Issue。

- **2026-06-21 19:09 CST 调整为 `Later`**：
  - 本轮复核边界按 bug-2 最新规则执行：当前机器不再作为生产运行态依据，且本单核心失败是 MiniMax / OpenAI-compatible `chat/completions` 外部传输失败。
  - 当前仓库已有 OpenAI-compatible provider 级短重试覆盖 `error sending request`、connection reset、timeout、tcp connect error 等瞬时传输错误；本轮不为供应商偶发失败继续叠加渠道 / 单任务特判。
  - 若未来在已确认加载当前代码、网络与供应商状态稳定的新运行态中仍成批复现，可再重新打开并评估通用退避、provider fallback 或告警聚合，而不是按当前旧窗口证据保持活跃。

- **2026-06-16 11:01 CST 补充复发证据，状态保持 `New`**：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 07:03-11:01 CST heartbeat 新增 `67` 条 `noop + skipped_noop + delivered=0`、`34` 条 `execution_failed + skipped_error + delivered=0` 与 `3` 条 `running + pending + delivered=0`。
    - 09:30 CST 同批 `11` 条 Feishu heartbeat 任务落成 `execution_failed + skipped_error + delivered=0`，覆盖 `TEM破位预警`、`RKLB异动监控`、`全天原油价格3小时播报`、`TEM大事件心跳监控`、`持仓重大事件心跳检测`、`Cerebras IPO与业务进展心跳监控`、`heartbeat_绿田机械基本面跟踪`、`AAOI 1.6T 光模块心跳检测`、`Monitor_Watchlist_11`、`DRAM 心跳监控`、`TSLA 正负触发条件心跳监控`。
    - 代表性样本：`run_id=43531/43533/43535/43536/43537/43539/43540/43541/43542/43543/43544`，均为 `detail_json.failure_kind=runner_error`，`heartbeat_model=MiniMax-M2.7-highspeed`。
    - 错误体均为 MiniMax/OpenAI-compatible `error sending request for url (https://api.minimaxi.com/v1/chat/completions)`，最终没有 heartbeat 用户可见提醒送达。
  - 会话质量对照：
    - 同窗 `session_messages` 有 `26` 个 user turn 与 `26` 个 assistant turn；最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口，无 user-only 残留。
    - 普通 scheduler `16` 条为 `completed + sent + delivered=1`，未见普通 scheduler 全局不可用；本轮另有 1 条 Feishu scheduler `data_fetch` 文案 P3 复发，已登记到独立文档。
    - 最近四小时无非文档代码提交。
  - 判断：
    - 本轮是 2026-06-15 15:30 / 16:00 CST 成批失败后的下一次真实窗口复发，说明 heartbeat MiniMax 传输失败仍未稳定收口。
    - 该问题影响 heartbeat 监控覆盖但未进入用户可见 assistant final；普通 scheduler 和直聊同窗仍收口，严重等级保持功能性 `P2`，非 P1，不创建 GitHub Issue。

- **2026-06-15 19:03 CST 回退为 `New`**：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 15:03-19:03 CST heartbeat 新增 `66` 条 `noop + skipped_noop + delivered=0`、`37` 条 `execution_failed + skipped_error + delivered=0` 与 `1` 条 `completed + sent + delivered=1`。
    - 15:30 CST 同批 `13` 条 Feishu heartbeat 任务落成 `execution_failed + skipped_error + delivered=0`，覆盖 `RKLB异动监控`、`SIVE POET/Nokia/1.6T DFB 心跳检测`、`DRAM 心跳监控`、`全天原油价格3小时播报`、`Monitor_Watchlist_11`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Cerebras IPO与业务进展心跳监控`、`heartbeat_绿田机械基本面跟踪`、`TSLA 正负触发条件心跳监控`、`TEM破位预警`、`AAOI 1.6T 光模块心跳检测`、`伦敦金跌破4100提醒`。
    - 16:00 CST 同类失败继续成批出现，新增 `13` 条 Feishu heartbeat `runner_error`，覆盖 `SIVE POET/Nokia/1.6T DFB 心跳检测`、`Cerebras IPO与业务进展心跳监控`、`Monitor_Watchlist_11`、`伦敦金跌破4100提醒`、`DRAM 心跳监控`、`持仓重大事件心跳检测`、`TEM破位预警`、`RKLB异动监控`、`heartbeat_绿田机械基本面跟踪`、`全天原油价格3小时播报`、`AAOI 1.6T 光模块心跳检测`、`TEM大事件心跳监控`、`TSLA 正负触发条件心跳监控`。
    - 代表性样本：`run_id=43004-43016` 与 `run_id=43017-43029`，`detail_json.failure_kind=runner_error`，`heartbeat_model=MiniMax-M2.7-highspeed`。
    - 错误体均为 MiniMax/OpenAI-compatible `error sending request for url (https://api.minimaxi.com/v1/chat/completions)`，最终没有 heartbeat 用户可见提醒送达。
  - 会话质量对照：
    - 同窗 `session_messages` 有 `5` 个 user turn 与 `5` 个 assistant turn；最近 Feishu direct 与普通 scheduler 会话均以 assistant 收口，无 user-only 残留。
    - 普通 scheduler 仅 `A股港股收盘后跨市场复盘` 1 条，为 `completed + sent + delivered=1`，未见 `commodity_causality_guarded=true`、send_failed 或空回复。
    - 15:10 / 15:15 CST 同一 Feishu direct 请求两次返回产品化通用失败文案，16:15 CST 第三次重试成功完成同一问题；该短时直聊失败缺少独立根因证据，未单独建档。
    - 最近四小时无非文档代码提交。
  - 判断：
    - 2026-06-12 的 `Later` 结论依赖“当前代码已有 provider 级短重试，真实窗口若确认仍成批复现再回退”的条件；本轮真实 heartbeat 窗口连续两个半小时批次共 `26` 条同类传输失败，满足回退条件。
    - 该问题与 heartbeat 结构化 JSON 退化、context window overflow、重复提醒噪音不同；本轮失败发生在 MiniMax/OpenAI-compatible 请求传输层，直接造成 heartbeat 监控覆盖缺口。
    - 没有证据显示普通 scheduler、Feishu direct 或 Feishu 出站整体不可用；严重等级保持功能性 `P2`，非 P1，不创建 GitHub Issue。

- **2026-06-10 19:01 CST 回退为 `New`**：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 15:02-19:01 CST heartbeat 新增 `59` 条 `noop + skipped_noop + delivered=0`、`46` 条 `execution_failed + skipped_error + delivered=0` 与 `1` 条 `completed + sent + delivered=1`。
    - 18:30 CST 同批 `13` 条 Feishu heartbeat 任务落成 `execution_failed + skipped_error + delivered=0`，覆盖 `伦敦金跌破4500提醒`、`DRAM 心跳监控`、`持仓重大事件心跳检测`、`heartbeat_绿田机械基本面跟踪`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`、`SIVE POET/Nokia/1.6T DFB 心跳检测`、`Cerebras IPO与业务进展心跳监控`、`RKLB异动监控`、`AAOI 1.6T 光模块心跳检测` 等任务。
    - 代表性样本：`run_id=39708/39712/39713/39706/39703/39705/39710/39701/39704/39711/39709/39702/39707`，`detail_json.failure_kind=runner_error`，`heartbeat_model=MiniMax-M2.7-highspeed`。
    - 错误体均为 MiniMax/OpenAI-compatible `error sending request for url (https://api.minimaxi.com/v1/chat/completions)`，且最终没有 heartbeat 用户可见提醒送达。
  - 会话质量对照：
    - 同窗 `session_messages` 有 `11` 个 user turn 与 `11` 个 assistant final；最近 Feishu direct / scheduler 会话均以 assistant final 收口。
    - 普通 scheduler 仅 `A股港股收盘后跨市场复盘` 1 条，为 `completed + sent + delivered=1`，未见发送失败或 commodity guard 误替换。
    - assistant final 污染扫描未命中空回复、本机路径、`data/agent-sandboxes`、`company_profiles/...`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic、`enabled=true/false`、`data_fetch` 或技能状态外露；最近四小时无非文档代码提交。
  - 判断：
    - 2026-04-26 的 `Later` 结论基于 provider 级短重试已覆盖主要 `error sending request` 形态；本轮真实生产窗口再次出现成批同类失败，满足“若真实 heartbeat 窗口仍有同类成批传输失败，再改回 `New`”的回退条件。
    - 该问题与 `PlainTextSuppressed` / `JsonMalformed` / `JsonUnknownStatus` 等 heartbeat 输出协议退化不同；本轮 18:30 样本在请求传输层失败，导致 heartbeat 监控覆盖缺口。
    - 没有用户可见 final 污染，也没有普通 scheduler 或 Feishu direct 全局不可用证据；严重等级保持功能性 `P2`，非 P1，不创建 GitHub Issue。
- **2026-06-12 00:07 CST 调整为 `Later`**：
  - 本轮复核确认当前仓库代码已在 OpenAI-compatible `chat` / `chat_with_tools` 两条路径对 `error sending request`、connection reset、timeout 等瞬时传输错误做 provider 级短重试。
  - 2026-06-10 样本属于 MiniMax/OpenAI-compatible 外部传输失败；当前机器不再作为生产运行态证据，且本任务不为单次网络/第三方抖动写特殊兼容。
  - 因此本单不再占活跃修复队列，改为 `Later`：若在已确认加载当前代码且网络/供应商状态稳定的新运行态中仍成批复现，再评估通用 provider fallback、退避或告警聚合，而不是写渠道/单次错误特判。

- 代码层确认 `crates/hone-llm/src/openai_compatible.rs` 已在 `chat` 与 `chat_with_tools` 两条路径对主要瞬时传输错误执行一次短重试，覆盖：
  - `error sending request`
  - `connection reset`
  - `connection closed before message completed`
  - `operation timed out`
  - `tcp connect error`
- heartbeat scheduler 调用 MiniMax 辅助模型走 OpenAI-compatible provider，因此该 provider 级吸震已覆盖本单记录的 `https://api.minimaxi.com/v1/chat/completions` 发送失败形态。
- 2026-04-26 状态曾调整为 `Later`：provider 级吸震已覆盖当前失败形态，不再占活跃修复队列；若真实 heartbeat 窗口仍有同类成批传输失败，再改回 `New` 并评估更多重试、退避或 provider fallback。2026-06-10 18:30 CST 已再次成批复现，因此本单当前为 `New`。
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-04-21 19:30-20:00 最新巡检样本：
    - `run_id=4089`（`Monitor_Watchlist_11`，`executed_at=2026-04-21T19:30:06.654987+08:00`）落成 `execution_failed + skipped_error + delivered=0`
    - 错误体为 `LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 到 `20:00` 同批主要故障又切换为 `JsonUnknownStatus` 与 ASTS 重复触发，说明 MiniMax 传输失败已从 18:30 的小批量故障回落成单点复现，但仍没有稳定消失，不能标记为已修复。
  - 2026-04-21 18:30-19:00 最新巡检样本：
    - `run_id=4069`（`TEM破位预警`，`executed_at=2026-04-21T18:30:06.674728+08:00`）落成 `execution_failed + skipped_error + delivered=0`
    - `run_id=4070`（`小米30港元破位预警`，`executed_at=2026-04-21T18:30:06.675238+08:00`）同批落成 `execution_failed + skipped_error + delivered=0`
    - 两条错误体均为 `LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - 到 `19:00` 同批主要故障切换为 `JsonUnknownStatus` 与 ASTS 重复触发，说明 MiniMax 传输失败不是全批持续不可用，但仍会在真实 heartbeat 窗口单点/小批量复现，不能视为已收口。
  - 2026-04-21 17:30-18:00 最新巡检样本：
    - `run_id=4057`（`ASTS 重大异动心跳监控`，`executed_at=2026-04-21T17:31:04.688322+08:00`）落成 `execution_failed + skipped_error + delivered=0`
    - `run_id=4058`（`小米30港元破位预警`，`executed_at=2026-04-21T18:00:06.533451+08:00`）再次落成 `execution_failed + skipped_error + delivered=0`
    - `run_id=4059`（`小米破位预警`，`executed_at=2026-04-21T18:00:06.543419+08:00`）同批落成 `execution_failed + skipped_error + delivered=0`
    - 三条错误体均为 `LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
    - `data/runtime/logs/sidecar.log` 在 `2026-04-21 18:00:06.531` 与 `18:00:06.542` 记录两条 `run_finish ... success=false` 与 `runner_error ... model=MiniMax-M2.7-highspeed`，失败后仍以“心跳任务未命中，本轮不发送”收口。
    - 这说明 16:01 之后并未稳定收口；MiniMax HTTP 传输失败继续以单点和小批量方式影响 heartbeat。
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

- `2026-06-10 18:30` 最新窗口再次出现 13 条 Feishu heartbeat MiniMax/OpenAI-compatible `chat/completions` 发送失败，均落成 `execution_failed + skipped_error + delivered=0`。这说明 2026-04-26 的 provider 级短重试止血没有在当前真实 heartbeat 窗口里稳定吸收同类传输失败。
- `2026-04-21 19:30` 最新窗口继续出现 `Monitor_Watchlist_11` MiniMax `chat/completions` 发送失败；`20:00` 虽又漂移到结构化状态失败，但同类传输失败仍在生产窗口单点复现，说明 heartbeat 上游吸震仍未稳定证明收口。
- `2026-04-21 18:30` 最新窗口继续出现 `TEM破位预警` 与 `小米30港元破位预警` MiniMax `chat/completions` 发送失败；`19:00` 虽未继续同类传输失败，但漂移到结构化状态失败，说明 heartbeat 上游吸震仍未稳定证明收口。
- `2026-04-21 17:31-18:00` 最新窗口继续出现 `ASTS / 小米30港元 / 小米` 三条 MiniMax `chat/completions` 发送失败；虽不再是 11:00-12:00 的全批次失败，但足以证明吸震策略仍未在生产稳定收口。
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

- 2026-06-10 的新样本表明，当前线上仍会在同一 heartbeat 批次里成批命中 MiniMax/OpenAI-compatible 请求发送失败；即使 provider 层存在短重试，真实结果仍未被稳定吸收。
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
