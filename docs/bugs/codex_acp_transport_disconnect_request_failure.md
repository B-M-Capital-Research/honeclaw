# Bug: Codex ACP transport 断连导致直聊和定时请求失败且缺少自动恢复

- **发现时间**: 2026-06-06 11:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3` / `data/runtime/logs/acp-events.log` / `data/runtime/logs/backend_screen.log`
  - 2026-07-07 11:02-15:01 CST 同类 scheduler runner timeout 在普通 Feishu scheduler 链路继续出现。
  - 12:00 CST Feishu scheduler `每日公司资讯与分析总结` user turn 落库，12:11 CST assistant 仅写入产品化失败提示“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”，没有生成用户请求的公司资讯 / 分析师总结 / 财报日期正文。
  - `cron_job_runs.run_id=46004` 同步落成 `execution_failed + skipped_error + should_deliver=0 + delivered=0`，`detail_json.failure_kind=scheduler_runner_timeout`。
  - 该样本晚于 2026-07-07 03:06 CST 代码级 ACP retry 修复，且当前 SQLite / cron 台账显示业务正文仍未完成，因此从代码级 `Fixed` 回退为运行态 `New`。
  - 用户可见侧只看到脱敏失败提示，未见 ACP timeout、路径、URL、provider 原始错误、错投或数据破坏；影响仍是请求完成率和 scheduler 结果生成，维持功能性 `P2`，非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/acp-events.log` / `data/runtime/logs/backend_screen.log`
  - 2026-07-06 19:03-23:04 CST 同类 ACP runner timeout 在 Web direct 链路继续出现，且本轮用户请求没有形成 assistant final。
  - 22:35 CST Web direct session `Actor_web__direct__web-user-8988066ef1ac` 收到用户关于特朗普股市表态和市场反应的提问；`sessions.sqlite3` 已持久化该 user turn，但截至 23:01 CST 该 session 最新仍为 user，未追加 assistant final。
  - `acp-events.log` 同轮已出现 `session/prompt`、58 个 `agent_message_chunk` 和 1 个 `tool_call`（`Searching the Web`），最后一条 ACP 事件停在 22:35:41 CST 附近，没有 `stopReason=end_turn`。
  - `backend_screen.log` 在 22:36 / 22:37 / 22:38 CST 连续记录 `agent.run still running`，随后 22:38 CST 记录 `runner.error kind=TimeoutPerLine`，错误为 `codex acp session/prompt idle timeout (180s)`，并在 Web chat 路径落成处理失败。
  - 该错误的内部 stderr 包含本机 plugin manifest 路径，但本轮没有形成用户可见 assistant final，因此没有原始错误外泄到最终回复；问题影响 Web direct 请求完成率，仍为功能性 `P2 / New`，非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/feishu_screen.log`
  - 2026-07-06 07:02-11:02 CST 同类 ACP runner timeout 在 Feishu direct 链路继续出现，但用户可见错误已被净化。
  - 09:09 CST Feishu direct 用户请求“周末财经新闻,预测热门板块...”后，09:14 CST 同一 message_id 落库 assistant text `抱歉，处理超时了。请稍后再试。`；日志侧同轮为 `codex acp session/prompt idle timeout (180s)`，内部 stderr 包含本机 plugin manifest 路径，但未进入用户可见回复。
  - 09:15 CST 用户重发同一请求后，09:17 CST assistant final 正常给出完整板块 / 持仓分析。
  - 同窗普通 scheduler 19 条 `completed + sent + delivered=1`，未见同类普通 scheduler timeout；assistant final 污染扫描未命中内部路径、provider 原始错误、panic、quota、stream disconnect 或 env 字段。
  - 结论：该问题当前表现为“直聊单轮请求未完成但错误提示脱敏，用户重试后可恢复”，仍影响请求完成率，维持 `P2 / New`；非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `cron_job_runs`
  - 2026-07-05 19:02-23:06 CST 同类 ACP runner timeout 在普通 Feishu scheduler 链路继续出现，但用户可见错误已被净化。
  - `run_id=44917` / `job_name=每日美股大跌风险控制检查` 在 20:34 CST 落成 `execution_failed + skipped_error`，`detail_json.failure_kind=scheduler_runner_timeout`；assistant transcript 同步写入产品化失败提示“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”。
  - `run_id=44947` / `job_name=每日美股大盘风控简报` 在 21:52 CST 落成同类 `execution_failed + skipped_error + scheduler_runner_timeout`，同样只写入产品化失败提示。
  - 同窗 25 条 ACP `session/new` 记录共 252 个 env 条目，其中 223 个值为 `<redacted>`，12 个敏感键名均无未脱敏值；未见 `stream disconnected before completion`、response error、runner error、quota、panic 或资源耗尽进入 assistant final。
  - 结论：该问题当前继续表现为“业务正文未完成但失败提示可见且脱敏”，仍影响 scheduler 请求完成率，维持 `P2 / New`；非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `cron_job_runs`
  - 2026-07-04 23:02-2026-07-05 03:02 CST 同类 ACP runner timeout 在普通 Feishu scheduler 链路继续出现，但用户可见错误已被净化。
  - `run_id=44323` / `job_name=SemiAnalysis与Citrini文章晚间跟踪` 在 23:52 CST 落成 `execution_failed + skipped_error`，`detail_json.failure_kind=scheduler_runner_timeout`；assistant transcript 同步写入产品化失败提示“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”。
  - 同窗 44 个 `session/prompt` 覆盖 21 个 prompt session，均有 response 收口；43 个 `stopReason=end_turn`，未见 ACP response error、stream disconnect 原文、provider 原始错误、panic、quota、资源耗尽或本机绝对路径进入 assistant final。
  - 结论：该问题当前表现为“业务正文未完成但失败提示可见且脱敏”，仍影响 scheduler 请求完成率，维持 `P2 / New`；非 P1，不创建 GitHub Issue。
- `data/runtime/logs/acp-events.log` / `data/runtime/logs/hone_cli_screen.log`
  - 2026-07-04 15:02-19:04 CST 同类 ACP runner 长运行风险在 Feishu direct 链路继续出现。
  - 18:47 CST Feishu direct 会话 `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 启动后持续输出 `agent_message_chunk`，到 19:03 CST 仍记录 `agent.run still running`，`elapsed_s=960`、`state="agent_iterating"`。
  - 同一会话在 `acp-events.log` 到 19:03:55 CST 仍只有 chunk，尚未见 `stopReason=end_turn` 或失败终态；`data/sessions.sqlite3` 对应 actor 最新真实消息仍停在 2026-06-17，说明本轮无法从会话镜像确认最终用户可见收口。
  - 当前证据是长运行 / 最终超时风险，还没有确认原始错误外泄；状态维持 `P2 / New`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/hone_cli_screen.log`
  - 2026-07-03 03:00-07:00 CST 同类 ACP runner 请求失败在 Feishu scheduler 链路继续出现，但用户可见错误已被净化。
  - 06:00 CST Feishu scheduler `每日美股盘后收盘复盘` 启动后，06:03 CST 命中 `codex acp session/prompt idle timeout (180s)`，最终落成 `failure_kind=scheduler_runner_timeout`，Feishu 侧记录本轮不发送。
  - 同轮 `data/sessions.sqlite3` 只有 1 个 scheduler user turn 与 1 条 assistant failure final，assistant final 为产品化失败提示，污染扫描未命中本机路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、quota 或资源耗尽原文。
- `data/runtime/logs/hone_cli_screen.log`
  - 2026-07-01 07:02-11:02 CST 同类 ACP runner 请求失败在 scheduler 链路继续出现，但用户可见错误已被净化。
  - 09:03 CST Feishu scheduler `核心观察池早间简报` 命中 `codex acp session/prompt idle timeout (180s)`，最终落成 `failure_kind=scheduler_runner_timeout`，Feishu 侧记录本轮不发送。
  - 同窗错误 stderr 中只在内部日志保留 plugin manifest / 本机路径细节；对外只保留产品化失败提示“定时任务执行环境暂时不可用，系统已记录失败并将在下一次触发时重试。”。
  - `data/sessions.sqlite3` 本窗唯一 direct assistant final 污染扫描未命中内部路径、raw tool 字段、`HONE_MCP_BIN`、binary-not-found、provider 原始错误、panic 或资源耗尽原文。
- `data/runtime/logs/hone_cli_screen.log`
  - 2026-06-29 19:01-23:01 CST 同类 ACP runner 请求失败在 scheduler 链路继续出现，但用户可见错误已被净化。
  - 20:36 CST Feishu scheduler `每日仓位复盘` 命中 `codex acp session/prompt idle timeout (180s)`，最终落成 `failure_kind=scheduler_runner_timeout`，Feishu 侧记录本轮不发送。
  - 同窗错误 stderr 中只在内部日志保留 `Received event for unknown submission ID: auto-compact-* SkillsUpdateAvailable` 细节，用户可见 `agent_message_chunk` 污染扫描未命中绝对路径、raw tool 字段、`HONE_MCP_BIN`、binary-not-found、provider 原始错误或 panic。
- `data/runtime/logs/hone_cli_screen.log`
  - 2026-06-29 07:00-11:01 CST 同类 ACP runner 请求失败在 scheduler 链路继续出现，但用户可见错误已被净化。
  - 07:03 CST Feishu scheduler `美股持仓收盘后早报` 命中 `codex acp session/prompt idle timeout (180s)`，最终落成 `failure_kind=scheduler_runner_timeout`，Feishu 侧记录本轮不发送。
  - 08:10 CST Feishu scheduler `每日美股收盘与持仓早报` 命中同类 `scheduler_runner_timeout`，用户只看到产品化失败提示“定时任务执行环境暂时不可用，系统已记录失败并将在下一次触发时重试”。
  - 同窗错误 stderr 中只在内部日志保留 plugin manifest / MCP startup 细节，用户可见 `agent_message_chunk` 污染扫描未命中绝对路径、raw tool 字段、`HONE_MCP_BIN`、binary-not-found、provider 原始错误或 panic。
- `data/runtime/logs/hone_cli_screen.log`
  - 2026-06-29 03:04-07:02 CST 同类 ACP runner 请求失败在 scheduler 链路继续出现，但用户可见错误已被净化。
  - 04:04 CST Feishu scheduler `Oil_Price_Monitor_Closing` 命中 `codex acp session/prompt idle timeout (180s)`，最终落成 `failure_kind=scheduler_runner_timeout`，Feishu 侧记录本轮不发送。
  - 05:33 CST Feishu scheduler `美股收盘后跨市场复盘` 命中同类 `scheduler_runner_timeout`，用户只看到产品化失败提示“定时任务执行环境暂时不可用，系统已记录失败并将在下一次触发时重试”。
  - 06:33 CST Web scheduler `1亿美元AI科技组合每日跟踪` 同样落成 `scheduler_runner_timeout` 并跳过发送。
  - 同窗错误 stderr 中只在内部日志保留 MCP / plugin startup 细节，用户可见 `agent_message_chunk` 污染扫描未命中绝对路径、raw tool 字段、`HONE_MCP_BIN`、binary-not-found、provider 原始错误或 panic。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-06 07:01-11:01 CST。
  - 本窗共有 12 个 user turn 与 12 个 assistant final，Feishu direct / Discord scheduler 会话均有 assistant 记录收口。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f0bdff19e3e341fbbbffe811abecaac61` 在 2026-06-06 09:25 CST 收到用户追问：小分子化学药 / 生物药用药方式 / 是否借助 AI 研发。
  - 2026-06-06 09:29 CST assistant final 只返回脱敏通用失败文案：`抱歉，这次处理失败了。请稍后再试。`，用户本轮问题没有得到回答。
- `data/runtime/logs/acp-events.log`
  - 2026-06-06 09:26 CST 同一 Feishu direct prompt 已启动，随后 runner 输出内部 transport fallback 事件。
  - 2026-06-06 09:29 CST 同一 prompt 返回 `stream disconnected before completion` 内部错误；用户侧没有看到原始 URL 或 transport 细节。
  - 2026-06-06 09:30-09:34 CST Discord scheduler session `Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371` 也出现同类 transport fallback 和 `stream disconnected before completion` 错误。
  - 2026-06-24 09:01-09:33 CST 同根因复发：Feishu direct、Web direct 与 Discord group / scheduler 路径合计 10 个 ACP response 返回 `Internal error`，`payload.error.data.message` 均为 `stream disconnected before completion: error sending request for url (https://chatgpt.com/backend-api/codex/responses)`。
  - 同窗 ACP 可重构 31 次 `session/prompt`、18 个 session、22 次 `stopReason=end_turn`；错误集中在 09:01-09:33 CST，09:52 CST 非文档提交 `3679c4c5 fix: clean up acp mcp process groups` 后到 11:03 CST 未再见同类 response error，但该提交只清理 ACP/MCP 子进程进程组，未证明已覆盖 transport 断连重试 / fallback 根因。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=38431` / `job_name=每日美股降息概率推送` 在 2026-06-06 09:34 CST 落成 `noop + skipped_noop + should_deliver=0 + delivered=0`。
  - `detail_json.failure_kind=internal_error_suppressed`，说明 scheduler 没有把内部失败外发，但这轮定时任务也没有产出正文。
- 最近四小时无非文档代码提交。

## 端到端链路

1. 用户通过 Feishu direct 发起一个连续追问，前几轮同主题药物机制解释均正常回答。
2. runner 初始化、创建 session 并发起 `session/prompt`。
3. Codex ACP 从 WebSocket fallback 到 HTTPS transport 后，最终返回 `stream disconnected before completion`。
4. Feishu direct 外层把内部错误净化成通用失败文案并写入会话。
5. 用户本轮问题没有被完成，只能手动重试。
6. 同窗 Discord scheduler 也命中同类 transport 断连，但当前 scheduler 失败被抑制为不外发、不送达。

## 期望效果

- ACP transport 断连应有自动重试、备用 transport / runner fallback，或至少在保留安全净化的同时给出更可操作的失败分类。
- 对 Feishu direct 用户主动提问，单次可恢复的 transport 抖动不应直接让整轮清零。
- 对 scheduler，内部失败可以不外发，但应保留明确失败终态与可审计分类，避免误判为正常 `noop`。

## 当前实现效果

- Feishu direct 对用户可见的结果是通用失败，原始错误没有外泄，说明错误净化是生效的。
- 但主请求没有完成，且系统没有在同轮自动恢复或基于既有上下文降级回答。
- Discord scheduler 没有外发通用失败，`should_deliver=0` 是正确止血；但 `execution_status=noop` 与 `failure_kind=internal_error_suppressed` 同时出现，容易把 transport 失败和真正无须发送的业务 `noop` 混在一起。
- 2026-06-24 复发窗中，Feishu / Web direct 用户主动请求与 Discord group / scheduler 均仍会在 ACP transport 断连时整轮失败；日志侧保留内部 URL 和 raw error，用户可见侧未见原始 URL 外泄。
- 2026-06-29 07:02 CST 复核窗中，失败形态从 `stream disconnected before completion` 扩展到 `codex acp session/prompt idle timeout (180s)` / `scheduler_runner_timeout`；错误净化生效，但 Feishu / Web scheduler 的业务报告正文没有完成。
- 2026-07-01 11:03 CST 复核窗中，普通 Feishu scheduler 仍可因 `scheduler_runner_timeout` 跳过发送；错误净化继续生效，但该轮业务报告正文仍没有完成。
- 2026-07-03 07:00 CST 复核窗中，Feishu scheduler `每日美股盘后收盘复盘` 再次因 `scheduler_runner_timeout` 跳过发送；错误净化继续生效，但该轮业务报告正文仍没有完成。
- 2026-07-05 03:02 CST 复核窗中，Feishu scheduler `SemiAnalysis与Citrini文章晚间跟踪` 再次因 `scheduler_runner_timeout` 跳过发送；错误净化继续生效并落库产品化失败提示，但该轮业务报告正文仍没有完成。
- 2026-07-06 23:04 CST 复核窗中，Web direct 主动提问在输出开头和发起 Web 搜索工具后命中 `codex acp session/prompt idle timeout (180s)`；该轮没有 assistant final，用户请求未完成。
- 2026-07-07 15:01 CST 复核窗中，Feishu scheduler `每日公司资讯与分析总结` 再次因 `scheduler_runner_timeout` 跳过发送；错误净化继续生效并落库产品化失败提示，但该轮业务报告正文仍没有完成。该样本晚于 03:06 代码级 retry 修复，因此当前状态回退为运行态 `New`。

## 用户影响

- 这是功能性 bug，不是单纯输出质量问题。
- Feishu 用户主动追问没有得到答案，定时任务也有一轮因同类 runner transport 断连未产出正文。
- 定级为 `P2`：影响请求完成率和 scheduler 结果生成，但本窗只有 1 条 Feishu direct 用户可见失败和 1 条 Discord scheduler 抑制失败；没有跨用户大面积不可用、错投、数据破坏或原始错误外泄证据，因此不是 `P1`。
- 2026-06-24 复发窗覆盖 Feishu direct、Web direct 与 Discord group / scheduler 多条请求，影响范围比首发更广；仍未观察到原始错误外泄、错投或数据破坏，因此严重等级保持 `P2`，状态从 `Fixed` 回退为 `New`。
- 2026-07-06 23:04 CST 复发窗再次覆盖 Web direct 主动提问；影响是单轮请求未完成且 transcript 只保留 user turn，未见跨用户错投、数据破坏或大面积全渠道不可用，因此仍保持 `P2` 而不是 `P1`。
- 2026-07-07 15:01 CST 复发窗再次覆盖普通 Feishu scheduler；影响是本轮定时任务只产生失败提示、业务报告正文缺失，未见跨用户错投、数据破坏或大面积全渠道不可用，因此仍保持 `P2` 而不是 `P1`。

## 根因判断

- 直接根因是 Codex ACP transport 在执行中断连，返回 `stream disconnected before completion`。
- 与 `web_scheduler_acp_stream_disconnect_no_final.md` 同属 ACP transport 断连大类，但本轮新增受影响链路是 Feishu direct 用户主动提问和 Discord scheduler 抑制失败，不是原 Web scheduler SSE / 无终态问题。
- 与 `channel_raw_llm_error_exposure.md` 不同：本轮没有把 `chatgpt.com/backend-api/codex/responses`、transport fallback 或内部错误文本暴露给最终用户。
- 与 `feishu_direct_codex_usage_limit_generic_failure.md` 不同：本轮不是 usage limit / quota，错误分类为 transport disconnect。
- 2026-06-24 的非文档提交 `3679c4c5` 改善 ACP/MCP 子进程清理，可能降低进程泄漏导致的运行压力；但本轮证据中的直接失败是 ChatGPT Codex backend transport 断连，当前文档不能仅凭该提交判定根因已修复。
- 2026-06-29 新样本显示，MCP / plugin startup 事件与 ACP `session/prompt` idle timeout 仍可把 scheduler 正文生成拖到超时。它与本单同属 ACP runner 请求失败后缺少同轮自动恢复，但不同于 heartbeat 结构化 JSON 退化，也不同于 Web scheduler SSE 无终态：本窗已有产品化失败终态，只是业务内容未完成。

## 下一步建议

- 在 ACP runner 调用层为 `stream disconnected before completion` 增加一次短重试，或在同类错误下切换备用 runner / transport。
- 将 scheduler 的 transport 失败终态从业务 `noop` 中区分出来，例如保持 `execution_failed + skipped_error + failure_kind=internal_error_suppressed`，避免巡检和用户侧误读为“条件未命中”。
- 保留现有错误净化规则，继续禁止内部 URL、transport fallback、raw error 进入最终用户文本。

## 最新运行态复核（2026-07-07 11:02 CST）

- `data/sessions.sqlite3` / `cron_job_runs`
  - 巡检窗口：2026-07-07 07:01-11:02 CST。
  - 普通 scheduler 18 条中 17 条为 `completed + sent + delivered=1`，1 条 `特斯拉与火箭实验室新闻日报` 在 09:06 CST 落成 `execution_failed + skipped_error + should_deliver=0 + delivered=0`。
  - 该失败的 `detail_json.failure_kind=scheduler_runner_timeout`，assistant transcript 只写入产品化失败提示“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”，没有把 ACP timeout、路径、URL 或 provider 原始错误暴露给用户。
  - 09:00 CST 同一任务的 user turn 已落库，业务正文没有生成，因此仍属于请求完成率问题。
- 本轮判断
  - 该样本发生在 03:06 CST 代码级 retry 修复之后，但上一轮修复记录明确“未重启当前 live 服务”；本轮仅把它作为旧 live 运行态待复核证据，不直接判定代码修复失败。
  - 用户可见侧已脱敏，未见错投、数据破坏或大面积不可用；严重等级不升级，状态保持代码级 `Fixed`，后续需在已加载新代码的 live 窗口继续观察是否还出现 `scheduler_runner_timeout`。

## 最新运行态复核（2026-07-07 15:01 CST）

- `data/sessions.sqlite3` / `cron_job_runs`
  - 巡检窗口：2026-07-07 11:02-15:01 CST。
  - 普通 scheduler 1 条 `每日公司资讯与分析总结` 在 12:11 CST 落成 `execution_failed + skipped_error + should_deliver=0 + delivered=0`。
  - 该失败的 `detail_json.failure_kind=scheduler_runner_timeout`，assistant transcript 只写入产品化失败提示“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”，没有把 ACP timeout、路径、URL 或 provider 原始错误暴露给用户。
  - 12:00 CST 同一任务的 user turn 已落库，用户请求的公司资讯、分析师总结和财报日期正文没有生成，因此仍属于请求完成率问题。
- 本轮判断
  - 该样本晚于 03:06 CST 代码级 retry 修复，且已经是连续两个巡检窗口出现普通 scheduler timeout 运行态证据；不能继续仅按“未重启 live”保留在已修复表。
  - 用户可见侧已脱敏，未见错投、数据破坏或大面积不可用；严重等级不升级，状态从代码级 `Fixed` 回退为运行态 `New`，非 P1，不创建 GitHub Issue。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP 事件错误分类、`cron_job_runs` 终态与最近四小时非文档提交检查。
- 2026-06-24 11:03 CST 复核：SQLite 本地镜像仍停在 2026-06-17，真实会话以 `acp-events.log` / `web.log.2026-06-24` 重构；09:01-09:33 CST 有 10 条同类 ACP response error，09:52 CST 后到 11:03 CST 未见同类 response error。未修改业务代码，未运行代码测试。

## 修复记录

- 2026-07-07 03:06 CST：共享 `AgentSession` 现在会把 ACP 类瞬时失败视为可重试坏态，在保留既有 context-overflow / 空成功兜底之外，新增 1 次轻量重试用于 `codex/opencode acp session/prompt idle timeout`、`stream disconnected before completion`、`stream closed before response`、`acp stream disconnected`、`transport disconnected`。这让 direct 与 scheduler 在同一 runner 边界上共享恢复逻辑，而不是首轮瞬时断连就直接整轮失败。新增回归 `retryable_transient_runner_error_text_matches_acp_disconnect_and_idle_timeout`、`transient_runner_failure_retries_once_before_returning_success`，并复跑 `empty_success_with_tool_calls_uses_fallback_after_retries`、`cargo check -p hone-channels --tests`、`rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/agent_session/helpers.rs crates/hone-channels/src/agent_session/core.rs crates/hone-channels/src/agent_session/tests.rs`、`git diff --check` 通过。本轮未重启 live 服务，先按代码级 `Fixed` 记录，后续仍需运行态复核是否完全止血。
- 2026-06-09 00:12 CST 进入 `Fixing`：已准备代码加固，直聊错误净化会把 `codex acp ... stream disconnected before completion` 映射为安全的“当前本机执行环境暂时不可用”提示；scheduler 内部 ACP transport 断连仍不外发，但 `ScheduledTaskExecution.error` 会保留安全台账文案，`failure_kind=acp_transport_disconnect`，下游应落成 `execution_failed + skipped_error`，不再伪装成业务 `noop + skipped_noop`。
- 验证阻塞：本机 Rust toolchain 当前连 `cargo --version`、直接 toolchain `cargo --version`、`rustc --version` 都会悬挂；已终止悬挂进程并仅完成 `git diff --check`。因此本轮不得标记 `Fixed`、不得提交或推送；下一轮需先恢复 toolchain，再运行 `cargo test -p hone-channels user_visible_error_message_ --lib -- --nocapture`、`cargo test -p hone-channels suppressed_scheduler_failure_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`。
- 2026-06-09 04:43 CST 状态更新为 `Fixed`：Rust toolchain 已恢复，`cargo test -p hone-channels user_visible_error_message_ --lib -- --nocapture`、`cargo test -p hone-channels suppressed_scheduler_failure_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。该修复不依赖当前机器生产运行态或线上日志判定恢复。
- 2026-06-24 11:03 CST 回退为 `New`：07:02-11:03 CST 巡检确认同根因在 Feishu direct、Web direct 与 Discord group / scheduler 多链路复发；错误已净化但请求未完成。09:52 CST 后未见新增同类 response error，需后续巡检确认 `3679c4c5` 或运行态重启是否真正止血。

## 最新运行态复核（2026-07-04 23:02 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-04 19:01-23:02 CST。
  - 同窗重构 20 条 `session/prompt`，20 条均以 `stopReason=end_turn` 收口；未见 `stream disconnected before completion`、runner error、response error、quota 或 panic。
  - 上轮 18:47 CST Feishu direct 长运行样本在本窗内于 23:01 CST 前完成，最终 session/update 尾部返回 `stopReason=end_turn`，并已在 `sessions.sqlite3` 形成 assistant final。
- 本轮判断
  - 本窗没有新的 transport 断连失败证据，且上轮长运行样本最终收口；这是缓解信号，但不足以关闭此前多链路复发的 P2。
  - 状态维持 `P2 / New`；下一步继续观察是否连续多个窗口无 `stream disconnected` / `scheduler_runner_timeout` / 长运行未收口样本。
