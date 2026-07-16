# Bug: Heartbeat 实时核验门禁失败后批量跳过提醒

## 发现时间

2026-07-13 19:01 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-16`
  - 巡检时间窗：2026-07-16 19:02-23:02 CST。
  - `data/sessions.sqlite3` 同窗新增 29 条 user / 29 条 assistant，覆盖 12 个近期 session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 21:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘前美股要闻与SNDK/MU存储产业链日报` 先写入“我暂时无法确认你提到的 原文 对应哪家上市公司或证券”，随后写入 `定时任务「盘前美股要闻与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
  - 21:44 CST Web direct session `Actor_web__direct__web-user-31e5cde131ea` 对 `ARM 到底怎么看，股价持续回落，可以加吗` 只返回 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`
  - 23:00-23:01 CST runtime 继续出现多条 heartbeat `runner_error`，包括原油、Samsung/SNDK、SIVE、光迅科技与美股黄金坑等任务因实体 / 核验门禁失败跳过发送。
  - 同窗 22:57 / 22:59 CST Web direct 仍能输出 KORU / LITE、COHR / MU 调仓分析，21:45 CST Feishu scheduler 也能输出 QQQ / SPY 风控简报，说明该缺陷不是直聊或 scheduler 全链路不可用。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示或 scheduler 失败提示；由于同窗 direct / scheduler 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-16`
  - 巡检时间窗：2026-07-16 15:03-19:02 CST。
  - `data/sessions.sqlite3` 同窗新增 6 条 user / 6 条 assistant，覆盖 5 个 session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 18:24 CST Web direct session `Actor_web__direct__intl_5fasset_5fregression_5f1784197328` 回答“现在intl怎么看”时，只写入 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`，`metadata_json` 标记 `run_failed=true` / `AgentFailed`。
  - 18:31 CST Web direct session `Actor_web__direct__intl_5ffinal_5fregression_5f1784197814` 再次以同一投研完整性失败文案收口。
  - 18:38 CST Web direct session `Actor_web__direct__intl_5fvisible_5ffinal_5f1784198248` 同题最终成功输出 INTL 分析正文，说明该缺陷不是 Web direct 全链路不可用。
  - 同窗 `data/runtime/logs/web.log.2026-07-16` 仍有多批 heartbeat `runner_error` 与投研完整性 / 实体识别 guard 失败；相关实体 / 投研完整性 WARN / ERROR 共 216 条。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示；由于同窗 direct 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-15`
  - 巡检时间窗：2026-07-16 03:02-07:02 CST。
  - `data/sessions.sqlite3` 同窗新增 10 条 user / 11 条 assistant，覆盖 9 个近期 session；07:00 CST 边界任务已在 07:02:55 收口，没有长期 user-only 悬挂、错投或全渠道不可用证据。
  - 04:00 CST Feishu scheduler/direct actor session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 的 `Oil_Price_Monitor_Closing` 只写入 `当前无法稳定核验 USO 的本轮财务数据，已停止生成完整估值结论。`
  - 05:00 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` 先写入 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`，随后追加用户可见 `定时任务「盘后美股复盘与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
  - 05:10-05:14 CST Feishu scheduler `美股收盘资金流向简报` 只写入 `抱歉，这次处理失败了。请稍后再试。` 与 `本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。`
  - 05:00 / 06:00 CST 另有 ARKK / VIXM 财务数据无法稳定核验的产品化失败提示。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示；由于同窗 direct / scheduler 仍有成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-15`
  - 巡检时间窗：2026-07-15 23:02-2026-07-16 03:02 CST。
  - `data/sessions.sqlite3` 同窗新增 6 条 user / 6 条 assistant，覆盖 3 个 session，均以 assistant 收口；没有 user-only 悬挂、错投或全渠道不可用证据。
  - 00:05 CST Feishu scheduler/direct actor session `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 `RKLB 每日动态监控` 只写入 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`，`metadata_json` 标记 `run_failed=true` / `AgentFailed`。
  - 同窗 runtime heartbeat 仍出现 1 条 `context window exceeds limit` 后 `BudgetRecovery`，并有 129 次 `function_calling tool call rejected`；但本窗未见 `当前信息暂时未完成实时核验` 文案批量复发。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示；由于同窗 direct / scheduler 仍有成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-15`
  - 巡检时间窗：2026-07-15 19:01-23:01 CST。
  - `data/sessions.sqlite3` 同窗新增 48 条 user / 55 条 assistant，近期 28 个 session 均以 assistant 收口；没有 user-only 悬挂、错投或全渠道不可用证据。
  - 20:00 CST Web scheduler `英伟达每日消息` 只写入 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`，并追加用户可见 `定时任务「英伟达每日消息」执行出错，请稍后重试。`
  - 20:02-20:03 CST Feishu direct / scheduler 多条会话写入同一投研完整性检查失败文案；20:31 CST Web scheduler `持仓复盘-周三` 也以同一文案失败并写入 `定时任务「持仓复盘-周三」执行出错，请稍后重试。`
  - 21:02 CST Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 落成 `定时任务执行环境暂时不可用，系统已记录失败并将在下一次触发时重试。`
  - 这些样本和既有“实时核验 / required evidence / 完整性 guard fail-closed 后跳过提醒”是同一类用户可见降级：任务未生成业务正文，只给产品化失败提示。由于同窗仍有多条 direct / scheduler 成功收口，维持功能性 `P2 / New`，不升级为 P1。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-07-15 07:04-11:02 CST。
  - 09:00 CST Feishu direct session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 只写入 assistant final `当前信息暂时未完成实时核验，请稍后再试。`
  - 同窗 `data/sessions.sqlite3` 有 29 个 user turn / 29 条 assistant 记录，19 个近期 session 均以 assistant 收口，说明这不是全渠道不可用或未回复 P1。
  - 本地 `cron_job_runs` 同窗无新增，`max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`；真实 heartbeat 运行态继续依赖 `data/runtime/logs/web.log.2026-07-15` 复核。
  - 判断：同根实时核验门禁 fail-closed 仍会进入用户可见 final；但同窗 direct / scheduler 多数正常收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，因此维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-07-14` / `data/sessions.sqlite3`
  - 巡检时间窗：2026-07-14 19:02-23:02 CST。
  - runtime 日志同窗仍有 60 次 `当前信息暂时未完成实时核验`、39 次 required evidence / fallback failed、57 次 Tavily 查询过长、88 次工具预算拒绝、266 条 heartbeat `parse_kind` 诊断、77 条 `deliver_preview`，并有 195 条 `<think>` 出现在 heartbeat raw / preview 相关行。
  - `data/sessions.sqlite3` 同窗新增 58 个 user turn、62 条 assistant 记录和 2 条 system compact 记录；最近会话均以 assistant 收口，`last_message_role=user` 的活跃会话数为 0。
  - 20:00 CST Web scheduler `20:00 持仓股重要新闻晚报` 同时写入通用失败 final、scheduler 失败文本和 Web 出错提示；20:00 CST Feishu `每日20点期权墙简报` 与 `每日美股大盘温度检查` 也写入 `本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。`
  - 本地 `cron_job_runs` 同窗仍无新增行；真实运行态仍需依赖 runtime web log 复核。
  - 判断：同根实时核验门禁 / 工具预算退化仍影响 scheduler 与 heartbeat 覆盖；但 direct 会话和多数 scheduler 仍有 assistant 收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1。
- `data/runtime/logs/web.log.2026-07-14` / `data/sessions.sqlite3`
  - 巡检时间窗：2026-07-14 15:01-19:02 CST。
  - runtime 日志继续命中 507 次 `当前信息暂时未完成实时核验，请稍后再试。`、169 次 `required tool evidence missing after enforcement retry`、166 次 `tavily request failed ... Query is too long`、166 次 `function_calling required evidence fallback failed`、219 次工具预算拒绝。
  - 同窗 heartbeat `run_finish` 为 `success=false` 169 条、`success=true` 65 条；失败覆盖 Feishu / Web heartbeat，例如 `RKLB异动监控`、`TEM大事件心跳监控`、`持仓财报与重大新闻心跳提醒`、`美股黄金坑信号心跳检测`、`ASTS 重大异动心跳监控`、`FOTO 光子学ETF心跳检测`、`ORCL 大事件监控` 等以 `runner_error` 跳过发送。
  - `data/sessions.sqlite3` 同窗有 5 个 user turn / 5 条 assistant 记录，Web / Feishu direct 与 scheduler 均有 assistant 收口；assistant final 污染扫描未命中 `<think>`、本机绝对路径、原始工具 JSON、`data_fetch`、`company_profiles/`、panic、provider 原始 429 或实时核验失败文案。
  - 本地 `cron_job_runs` 仍无 2026-07-14 15:01 CST 后新增行，`max(executed_at)` 停在 `2026-07-10T14:01:27.621121+08:00`，本轮继续以 runtime web log 作为 heartbeat 真实运行态来源。
  - 判断：同根实时核验门禁 fail-closed 仍活跃，继续影响 heartbeat 覆盖；同窗仍有 direct / scheduler 成功样本、未见错投、数据破坏、敏感信息泄露或全渠道不可用，因此维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-07-14` / `data/sessions.sqlite3`
  - 巡检时间窗：2026-07-14 07:01-11:01 CST。
  - runtime 日志命中 340 次 `当前信息暂时未完成实时核验，请稍后再试。`、126 次 `tavily request failed ... Query is too long`、126 次 `function_calling required evidence fallback failed`、204 次工具预算拒绝。
  - 受影响任务覆盖 Feishu / Web heartbeat：08:30-11:01 CST 多条 `持仓重大事件心跳提醒`、`存储板块关键事件心跳提醒`、`光迅科技关键事件心跳提醒`、`全天原油价格3小时播报`、`Monitor_Watchlist_11`、`AAOI 1.6T 光模块心跳检测`、`SIVE POET/Nokia/1.6T DFB 心跳检测` 等以 `runner_error` 跳过发送。
  - `data/sessions.sqlite3` 同窗有 32 个 user turn / 44 条 assistant 记录，普通 direct / scheduler 仍有成功样本；失败主要表现为产品化失败提示或实时核验失败文案，没有 provider 原始错误、token、本机路径或 panic 进入 assistant final。
  - 判断：同根实时核验门禁 fail-closed 仍活跃，影响 heartbeat 覆盖和部分 scheduler 正文完成率；因同窗仍有多渠道正常收口，维持功能性 `P2 / New`，不升级为 P1。
- `data/sessions.sqlite3`
  - 巡检时间窗：2026-07-14 03:01-07:01 CST。
  - 04:30 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 的 `OWALERT_PostMarket` 先写 assistant final `抱歉，这次处理失败了。请稍后再试。`，随后写 scheduler 文本 `本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。`，`metadata_json` 标记 `AgentFailed` / `scheduler_failure=true`。
  - 06:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` 的 `每日美股盘后收盘复盘` 出现同样的 `AgentFailed` final 与产品化 scheduler 失败文本。
  - 04:03 / 04:06 CST Web direct 图片附件问答也两次只返回 `当前信息暂时未完成实时核验，请稍后再试。`，说明该 fail-closed 文案仍会影响非 heartbeat 的用户可见完成率；图片主链路另归入 Web 图片附件缺陷。
  - 本轮本地 `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，运行态缺少可审计任务粒度失败台账；用户可见侧主要是产品化失败文案，没有 provider 原始错误、token、本机路径或 panic 外泄。
  - 判断：该缺陷仍为功能性 `P2 / New`。它影响普通 scheduler / direct 任务正文完成率，但同窗仍有多个 scheduler 和 direct final 正常收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，因此不升级为 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-07-13`
  - 巡检时间窗：2026-07-13 23:02-2026-07-14 03:01 CST。
  - 同窗日志命中 477 行 `当前信息暂时未完成实时核验，请稍后再试。` 相关文本、157 次 `tavily request failed ... Query is too long`、93 次 `function_calling tool call rejected by global budget`，并有 318 条 heartbeat / scheduler `runner_error` 指向同一实时核验失败文案。
  - 受影响任务继续覆盖 Feishu 与 Web heartbeat：23:30 CST `AAOI 1.6T 光模块心跳检测`、`Monitor_Watchlist_11`、`小米30港元破位预警`、`光模块板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 等批量跳过发送；00:00 CST `美股黄金坑信号心跳检测`、`全天原油价格3小时播报`、`ASTS 重大异动心跳监控`、`FOTO 光子学ETF心跳检测`、`AI与科技持仓观察关键事件心跳提醒` 等继续失败；03:00 CST `Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`FOTO 光子学ETF心跳检测`、`RKLB异动监控`、`NBIS关键事件心跳提醒`、`ASTS 重大异动心跳监控`、`存储板块关键事件心跳提醒` 仍跳过发送。
  - 同窗 heartbeat 可分类信号仍有 `PlainTextTriggered=46`、`JsonNoop=13`、`JsonTriggered=7`、`PlainTextNoop=4`、`JsonMalformed=2`、`PlainTextSuppressed=1`，说明结构化漂移仍在，但本轮主要功能损失仍是 evidence 门禁 fail-closed 后批量 `runner_error`。
  - 判断：该缺陷仍为功能性 `P2 / New`。它影响 heartbeat 覆盖和普通监控任务完成率，但同窗 direct 会话仍有 assistant final 收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，因此不升级为 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-07-13`
  - 巡检时间窗：2026-07-13 19:00-23:02 CST。
  - 同窗日志命中 433 行 `当前信息暂时未完成实时核验，请稍后再试。` 相关文本，并有 156 次 `tavily request failed ... Query is too long`，多次触发 `function_calling required evidence fallback failed` 与 `answer rejected because required tool evidence is missing`。
  - 影响范围继续覆盖 Feishu / Web heartbeat，也扩展到普通 scheduler 用户可见正文完成率：
    - 20:00 CST Web scheduler `20:00 持仓股重要新闻晚报` 先写 assistant final `当前信息暂时未完成实时核验，请稍后再试。`，随后写 `定时任务「20:00 持仓股重要新闻晚报」执行出错，请稍后重试。`
    - 20:30 CST Feishu scheduler `美股纳斯达克盘前简报`、`老王说事与巴芒投资美股财报季个股判断`、`美股盘前宏观与财报日历梳理`、`每日仓位复盘` 均只写产品化失败提示 `本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。`
    - 21:00 CST Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 同时写内部失败 final、通用 scheduler 失败提示和 Web 出错提示。
    - 21:35 / 23:00 CST Feishu scheduler `科技核心股池 · 晚间击球区快报`、`核心观察股池晚间快报` 只落成 `当前信息暂时未完成实时核验，请稍后再试。`
  - 同窗 heartbeat 可分类信号仍有 `PlainTextTriggered=62`、`JsonNoop=11`、`PlainTextNoop=9`、`JsonMalformed=4`、`JsonTriggered=3`、`PlainTextSuppressed=2`、`JsonUnknownStatus=2`，但这批失败的直接表现是 evidence 门禁 fail-closed，而不是单纯结构化 JSON 解析退化。
  - 判断：该缺陷仍为功能性 P2。它影响监控 / 普通 scheduler 任务正文完成率，但直聊与部分 scheduler 仍正常收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，因此不升级为 P1。
- `data/sessions.sqlite3`
  - 19:00-23:02 CST 按真实 `timestamp` 新增 49 个 user turn / 60 条 assistant 记录；Feishu direct、Feishu scheduler、Web direct 与 Web scheduler 均有 assistant 终态。
  - assistant final 污染扫描未命中 `<think>`、本机路径、provider 原始错误、panic、quota、原始工具 JSON 或结构化 JSON 外泄；用户可见侧主要是产品化失败文案。
- `data/runtime/logs/web.log.2026-07-13`
  - 巡检时间窗：2026-07-13 15:01-19:01 CST。
  - 18:00-19:00 CST heartbeat / scheduler 日志出现 123 条 `error="当前信息暂时未完成实时核验，请稍后再试。"`。
  - 受影响任务覆盖 Feishu 与 Web heartbeat：`AAOI 1.6T 光模块心跳检测`、`闪迪关键事件心跳提醒`、`全天原油价格3小时播报`、`持仓财报与重大新闻心跳提醒`、`AI与科技持仓观察关键事件心跳提醒`、`SIVE POET/Nokia/1.6T DFB 心跳检测`、`NVDA 关键事件心跳提醒`、`NBIS关键事件心跳提醒` 等。
  - 同一窗口可分类 heartbeat 信号仍有 `PlainTextTriggered=174`、`JsonNoop=70`、`PlainTextNoop=10`、`JsonTriggered=5`、`JsonMalformed=4`、`PlainTextSuppressed=1`，但新增的核验失败不是结构化 JSON 解析失败，而是 runner fail-closed 后整轮 `runner_error` / 跳过发送。
- `data/sessions.sqlite3`
  - 18:00 CST Web scheduler session `Actor_web__direct__web-user-ba50cb9401c0` 先写 assistant final `当前信息暂时未完成实时核验，请稍后再试。`，随后又写 scheduler 文本 `定时任务「18:00 美股盘前 X 英文帖」执行出错，请稍后重试。`
  - 本地 `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，因此本轮以 runtime 日志为真实 heartbeat 状态来源。

## 端到端链路

1. Heartbeat / scheduler 到点触发持仓、财报、重大事件或市场播报任务。
2. Function Calling runner 尝试执行当前金融 / 事件核验。
3. runner 未能完成满足门禁的实时核验时，返回统一错误 `当前信息暂时未完成实时核验，请稍后再试。`
4. 调度层把该错误记为 `runner_error`，跳过发送或写入 Web scheduler 出错提示。
5. 用户本应收到的 heartbeat 覆盖缺失；部分 Web 会话还能看到失败 final，而不是有用的任务内容。

## 期望效果

- 实时核验门禁应避免无来源的强时效金融幻觉，但不应让大批 heartbeat 在没有区分任务风险、来源可用性和 noop 场景的情况下统一失败。
- 对“无重大事件 / 无需提醒”的 heartbeat，应能稳定落为 noop，而不是因为没有完成实时核验就进入 runner error。
- 对确实需要来源但检索失败的任务，应保留可审计失败原因与任务粒度，便于重试和降级，而不是只留下统一文案。

## 当前实现效果

- 18:00-19:00 CST 同一运行窗出现 123 条统一核验失败文案，覆盖多个用户、多个 heartbeat job 和 Feishu / Web 两类出站链路。
- 错误文本已经产品化，没有外泄 provider 原始错误、token 或本机路径。
- 但主功能链路受影响：监控任务没有产出正常 noop / triggered 结果，用户也收不到本应送达的提醒或确认无事发生的判断。

## 用户影响

- 这是功能性缺陷。Heartbeat 的价值在于周期性覆盖重大事件和异常变化；批量 `runner_error` 会造成监控盲区。
- 当前证据集中在 heartbeat / scheduler 链路，直聊仍有成功样本，且没有错投、数据破坏或敏感信息泄露，因此定级为 `P2`，不是 `P1`。

## 根因判断

- 初步判断是当前金融实时核验门禁在 heartbeat 场景过于宽泛或缺少分流：没有区分“必须 web evidence 才能回答的强时效财报 / 投资建议”和“可合法 noop 的周期监控”。
- 既有 `scheduler_heartbeat_unknown_status_silent_skip.md` 跟踪的是模型输出结构化状态退化、`<think>` 文本、JSON malformed 或 triggered/noop 解析漂移；本缺陷的主要失败形态是 runner 已经 fail-closed 并返回统一核验失败，影响链路和根因不同，因此单独建档。
- 既有外部模型 / transport / quota 缺陷也不能完全覆盖本轮样本：错误文本不是 MiniMax HTTP transport、OpenRouter 402、429 或 tool-call protocol mismatch，而是业务门禁失败后的用户态错误。

## 下一步建议

- 为 heartbeat 增加专用 evidence policy：只有生成用户可见事实 / 触发提醒时才要求来源闭环；无重大事件应允许基于已执行的查询结果或明确无结果落为 noop。
- 记录门禁失败的结构化原因，例如缺少 `web_search`、检索失败、工具预算耗尽、模型未调用工具，避免统一文案掩盖真实失败点。
- 增加回归样本：重大事件 heartbeat 在无新事件时应输出合法 noop；当前财报类 direct 问答仍必须在缺少实时来源时 fail closed。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/runtime/logs/web.log.2026-07-13` 15:01-19:01 CST heartbeat 日志、`data/sessions.sqlite3` 同窗 session 记录与 `cron_job_runs` 停滞状态。
