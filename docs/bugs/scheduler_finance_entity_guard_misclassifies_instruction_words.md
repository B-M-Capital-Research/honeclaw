# Bug: Scheduler finance entity guard misclassifies instruction words or explicit tickers as securities

- **发现时间**: 2026-07-15 19:02 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 运行态复核（2026-07-23 07:01 CST）

- 状态维持 `New/P2`。
- `data/runtime/logs/web.log.2026-07-22` 在 2026-07-23 03:01-07:01 CST 继续复发：
  - 05:00 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` 任务正文要求复盘宏观数据，仍把 `PCE` 识别为证券代码并返回“当前数据供应商没有返回同代码行情覆盖”，用户侧只看到 scheduler 执行出错。
  - 03:30-07:00 CST `AAOI 1.6T 光模块心跳检测` 每半小时继续把任务说明里的 `SEC` 当作证券代码并 fail-closed。
  - 同窗 `ORCL 大事件监控` 每半小时继续落成 Oracle 多上市地候选，需要补交易所后缀或公司全名。
- 判断：最新证据仍是 scheduler / heartbeat 任务正文、宏观词、监管文件词和任务名进入实体 guard / resolver 后误抽或多候选拦截；与既有缺陷同根，不新建重复文档。问题阻断部分 scheduler / heartbeat 正文生成，但同窗 Web / Feishu direct 有正常 assistant 收口，未见全渠道停摆、错投、敏感信息泄露或数据破坏，因此不是 P1。

## 运行态复核（2026-07-19 07:01 CST）

- 本轮 2026-07-22 23:02-2026-07-23 03:01 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/runtime/logs/web.log.2026-07-22`
    - 00:00 CST `ORCL 大事件监控` 仍因 Oracle 多上市地候选落成 `runner_error` 并跳过发送。
    - 00:00 CST `AAOI 1.6T 光模块心跳检测` 继续把任务上下文里的 `SEC` 当证券代码，因数据供应商没有同代码行情覆盖而失败。
    - 03:00 CST 同类 `ORCL` 多候选、`SEC` 误抽仍在 heartbeat runner_error 中复现。
  - `data/sessions.sqlite3`
    - 同窗 ordinary 会话均有 assistant 收口，未见直聊全局不可用、错投或敏感信息外泄。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、监管缩写或上市地候选进入实体 guard 后 fail-closed；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-22 11:03-15:03 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 7 条 user / 4 条 assistant / 2 条 system compact，覆盖 3 个更新 session；最近 assistant 到 `2026-07-22T13:50:00.553517+08:00`，无 user-only 残留、错投、空回复、本机路径、provider 原始错误或全渠道不可用。
  - `data/runtime/logs/web.log.2026-07-22` 同窗仍有 `runner_error=32`，代表样本包括 11:30、12:00、14:30 CST `AAOI 1.6T 光模块心跳检测` 继续把任务上下文里的 `SEC` 当证券代码且无行情覆盖；同窗 `ORCL 大事件监控` 继续因 Oracle 多上市地候选失败。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、监管公告词和公司名进入实体 guard / resolver 后误抽、误拦或多候选 fail-closed；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗 direct / scheduler 用户可见 final 正常收口，未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-22 03:01-07:03 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 14 条 user / 10 条 assistant / 2 条 system compact，覆盖 9 个更新 session；07:00 Feishu scheduler 边界触发已在 07:02 CST assistant 收口，未见长期 user-only 残留、错投、空回复、本机路径、provider 原始错误或全渠道不可用。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 任务正文包含宏观指标 `PCE`，assistant 先返回“已识别证券代码‘PCE’，但当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `定时任务「盘后美股复盘与SNDK/MU存储产业链日报」执行出错，请稍后重试。` 与 scheduler failure 元数据。
  - `data/runtime/logs/web.log.2026-07-21` 03:01-07:03 CST 仍记录 `runner_error=33`，代表样本包括 06:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖，`ORCL 大事件监控` 继续因 Oracle 多上市地候选失败。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、宏观词和监管公告词进入实体 guard / resolver 后误抽、误拦；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗 direct / scheduler 多个 session 正常收口，未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-21 19:01-23:01 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 72 条 user / 49 条 assistant / 18 条 system compact，覆盖 29 个更新 session，最近 assistant 到 `2026-07-21T23:01:21.870583+08:00`；未见全渠道不可用、错投或敏感信息外泄。
  - 21:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘前美股要闻与SNDK/MU存储产业链日报` 任务正文包含宏观指标 `PCE`，assistant 先返回“已识别证券代码‘PCE’，但当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `定时任务「盘前美股要闻与SNDK/MU存储产业链日报」执行出错，请稍后重试。` 与 `scheduler_failure=true` 元数据。
  - `data/runtime/logs/web.log.2026-07-21` 19:01-23:01 CST 仍记录 heartbeat / scheduler 运行态：`runner_error=32`、`PCE/SEC` 证券误抽相关日志 17 条；代表样本包括 19:30、20:00、20:30、21:00、21:30、22:00、22:30、23:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、宏观词和监管公告词进入实体 guard / resolver 后误抽、误拦；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗 direct / scheduler 多个 session 正常收口，未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-21 03:02-07:03 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 15 条 user / 9 条 assistant / 4 条 system compact，覆盖 7 个更新 session；采样点 07:00 Feishu scheduler 已在 07:02 CST assistant 收口。未见全渠道不可用、错投或敏感信息外泄。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 任务正文包含宏观 `PCE` 等指标词，assistant final 最终只向用户写入 `定时任务「盘后美股复盘与SNDK/MU存储产业链日报」执行出错，请稍后重试。`；runtime 同窗记录该任务接入 strict function-calling runner 后快速落成 `session.persist_assistant detail=failed` 和用户可见执行失败。
  - `data/runtime/logs/web.log.2026-07-20` 在 03:02-07:03 CST 仍有 32 条 `runner_error`，代表样本包括 03:30 / 04:00 / 04:30 / 05:00 / 05:30 / 06:00 / 06:30 / 07:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖，`ORCL 大事件监控` 仍因 Oracle 多上市地候选失败。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、宏观指标和上下文词进入实体 guard / resolver 后误抽、误拦或多候选 fail-closed；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗仍有多个 Feishu / Web scheduler assistant final 正常收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-20 19:01-23:02 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 74 条 user / 44 条 assistant / 18 条 system compact，27 个更新 session 均以 assistant 收口；未见直聊全局不可用、错投或敏感信息外泄。
  - 21:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘前美股要闻与SNDK/MU存储产业链日报` 再次把宏观指标 `PCE` 当作证券代码，返回“当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `scheduler_failure=true` 执行出错。
  - `data/runtime/logs/web.log.2026-07-20` 同窗继续记录 36 条 `runner_error`；代表样本包括 19:30-23:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖，`ORCL 大事件监控` 仍因 Oracle 多上市地候选失败。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、宏观词、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成或造成 heartbeat 标的错配，但同窗 27 个更新 session 均以 assistant 收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-20 11:01-15:05 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 19 条 user / 15 条 assistant / 6 条 system compact，6 个更新 session 均以 assistant 收口；未见直聊全局不可用、错投或敏感信息外泄。
  - `data/runtime/logs/web.log.2026-07-20` 同窗继续记录 32 条 `runner_error`，实体 / 候选相关代表样本包括 11:30 和 12:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖；同窗 `ORCL 大事件监控` 仍因 Oracle 多上市地候选失败。
  - 同窗还出现 11:30 CST `光迅科技关键事件心跳提醒` deliver preview 漂移成 NIO 分析、12:00 CST `光迅科技关键事件心跳提醒` 漂移成 SK Hynix / NVIDIA 关系分析，说明问题继续包含 fail-open 的任务主体错配。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成或造成 heartbeat 标的错配，但同窗 Web / Feishu direct 有正常 assistant 收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-20 03:02-07:02 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 23 条 user / 10 条 assistant / 8 条 system compact；除 07:00 Feishu scheduler 在巡检采样点仍运行、随后 07:03 收口外，其余近期 session 均以 assistant 收口，未见直聊全局不可用、错投或敏感信息外泄。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 再次把宏观指标 `PCE` 当作证券代码，返回“当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `scheduler_failure=true` 执行出错。
  - `data/runtime/logs/web.log.2026-07-19` 同窗继续记录实体 / 候选 fail-closed 样本：03:30 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码且无行情覆盖；03:30 CST `ORCL 大事件监控` 仍因 Oracle 多上市地候选失败。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、宏观词、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗仍有多个 Feishu / Web scheduler assistant final 正常收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-19 23:02-2026-07-20 03:02 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 3 条 user / 3 条 assistant，均来自 Feishu scheduler `AAOI/TEM/RKLB 每日动态监控`，同一 session 以 assistant 收口，未见直聊 user-only 残留或 assistant final 原始错误外泄。
  - `data/runtime/logs/web.log.2026-07-19` 同窗有 38 条 `runner_error`，实体 / 候选相关代表样本继续出现：03:00 CST `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码并因无行情覆盖失败；03:00 CST `ORCL 大事件监控` 仍因 Oracle 多上市地候选 fail-closed。
  - 同窗还出现 18 条 OpenAI-compatible upstream HTTP 529 失败，但这些是 provider 负载错误，不改变本缺陷的实体 guard 根因判断。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler / heartbeat 正文生成，但同窗 scheduler session 仍有 AAOI/TEM/RKLB assistant final 收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-19 11:00-15:03 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 69 条 user / 27 条 assistant / 26 条 system compact，近期 Feishu direct / scheduler、Web direct / canary session 均以 assistant 收口，`last_message_role=user` 为 0。
  - 11:10 CST `cron_job_runs.run_id=48133` / Feishu scheduler `美股与A股重点标的跟踪晨报` 把任务上下文里的 `SEC` 当证券代码，返回“当前数据供应商没有返回同代码行情覆盖”，并以 `execution_failed + sent + delivered=1` 进入用户可见失败。
  - 11:11 CST `Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732` 在 `每日SemiAnalysis与Citrini文章追踪` 同一触发窗口先写出“本轮未能发现足够的可核验代表证券，不会用通用标的凑数”，随后同 session 在 11:13 CST 才输出文章追踪正文；说明 scheduler 金融实体预检仍可能抢先生成与用户任务不匹配的失败/短答。
  - `data/runtime/logs/web.log.2026-07-19` 11:00-15:03 CST 继续记录 18 条 `runner_error`，代表包括 15:00 CST `ORCL 大事件监控` 落成 Oracle 多上市地候选，`AAOI 1.6T 光模块心跳检测` 把 `SEC` 当作证券代码且无行情覆盖，多个 heartbeat 因实体/候选/结构化问题跳过发送。
  - 15:01 CST `NBIS关键事件心跳提醒` deliver preview 输出 `NVIDIA 当前 $202.81` 分析，与任务主体 NBIS 错配；15:00 CST `ASTS 重大异动心跳监控` deliver preview 的行情口径误写为 `TEM 报价源`，说明问题仍包含 fail-open 的实体/任务主体漂移。
  - 判断：最新样本仍是 scheduler / heartbeat 任务正文、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复文档。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler 正文并造成 heartbeat 标的错配，但同窗 Web direct canary 13:51 / 14:51 / 14:54 均有 assistant 收口，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- 本轮 2026-07-19 03:00-07:01 CST 真实运行态继续复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 6 条 assistant / 2 条 system compact，近期 Web direct / Web scheduler 会话均有 assistant 收口；`Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 在 07:01 CST 边界新增 user turn，未纳入本轮 07:01 前完整收口判断。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 连续第三个窗口把宏观指标 `PCE` 当作证券代码，返回“当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `scheduler_failure=true` 执行出错。
  - `data/runtime/logs/web.log.2026-07-18` 在 03:00 / 03:30 / 05:30 CST 继续记录 AAOI heartbeat 把任务上下文里的 `SEC` 当证券代码且无行情覆盖；ORCL heartbeat 仍落成 Oracle 多上市地候选。
  - 同窗 06:30 CST `光迅科技关键事件心跳提醒` 可生成并投递正文，但内容主体漂移成 NBIS 投研；07:01 CST `heartbeat_绿田机械基本面跟踪` 又漂移成 LULU 分析，说明问题不只 fail-closed，也会在 heartbeat 上下文中选错实体并产出错配正文。
  - 判断：这些样本仍是 scheduler / heartbeat 任务正文、历史 reminder 与行业词进入实体 guard / resolver 后误抽、误拦或错配实体；与既有缺陷同根，不新建重复文档。
  - 严重等级维持 `P2`：它直接阻断部分 scheduler 正文并造成 heartbeat 标的错配，但同窗 Web direct canary 03:25 / 04:51 / 06:52 均可成功回答 CRWV/NVDA 关系，未见全渠道停摆、错投到其他用户、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

## 运行态复核（2026-07-18 23:03 CST）

- 20:40-22:49 CST 五个非文档提交已进一步止血 interactive direct 投研输出保留路径：
  - `fcca5a35 fix: preserve interactive answers across search refinement`
  - `54b14068 fix: avoid partial contracts for translated aliases`
  - `25090e88 fix: preserve interactive agent answers`
  - `8df08239 fix: enforce agent-owned time-first research output`
  - `ccb24767 fix: stream agent terminal header safely`
- 本轮 2026-07-18 19:02-23:03 CST 真实运行态显示 direct regression 明显好转但 scheduler / heartbeat 仍复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 18 条 user / 11 条 assistant / 4 条 system compact，近期 Web regression direct、Web scheduler、Feishu direct 与 Feishu scheduler 均以 assistant 收口，`last_message_role=user` 为 0。
  - 20:47 / 20:49 CST CRWV/NBIS regression direct 已能精确核验 CoreWeave / Nebius 并给出估值分析；21:51 / 22:10 / 22:52 CST CRWV/NVDA regression direct 均保留 agent answer，不再落成前窗“投研完整性检查失败”。
  - 但 21:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘前美股要闻与SNDK/MU存储产业链日报` 仍把任务中的 `PCE` 识别成证券代码并返回“当前数据供应商没有返回同代码行情覆盖”，随后写入 `scheduler_failure=true` 的用户可见执行出错。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续记录 AAOI heartbeat 把 `SEC` 当证券代码且无行情覆盖，ORCL heartbeat 落成 Oracle 多上市地候选；22:00、22:30、23:00 批次均复发。
  - 判断：interactive direct 的 CRWV/NBIS、CRWV/NVDA 路径已有明确止血，但 scheduler / heartbeat 任务正文和上下文词仍会被实体 guard / resolver 误抽或多候选拦截，仍是同一链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分 scheduler / heartbeat 正文生成，但同窗 direct regression 和 Feishu direct 均可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 运行态复核（2026-07-18 19:02 CST）

- `2d6b4be8` / `8d4fcdd6` 已部分止血 interactive direct entity / finance answer 路径：18:19 CST CRWV/NBIS regression direct 已能精确核验两只标的但仍被投研完整性检查拦截；18:40 CST 同题在后续提交后成功输出完整估值对比。
- 但本轮 2026-07-18 15:02-19:02 CST 真实运行态仍复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 2 条 user / 2 条 assistant，近期 Web regression direct session 均以 assistant 收口，`last_message_role=user` 为 0；assistant final 污染扫描未命中内部路径、raw tool、`data_fetch`、`cron_job`、SQLite、panic、provider 原始错误或 `<think>`。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续有 333 条 `runner_error`、120 条定时任务执行失败、34 条“当前数据供应商没有返回”、34 条“已识别证券代码”和 16 条多候选信号。
  - 19:00 CST 代表 heartbeat 仍包括 AAOI 被误识别为 `SEC` 且无行情覆盖、ORCL 落成 Oracle 多上市地候选、闪迪 raw 把 `SK` 判断为非证券实体、TSLA raw 试图调用不存在的 heartbeat / cron 工具后落成结构化失败，存储板块继续把产品词和任务上下文混入证券判断。
  - 判断：interactive CRWV/NBIS 修复有止血，但 scheduler / heartbeat 与部分任务上下文仍会被实体 guard / resolver 拦截或误抽任务词，仍是同一链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler / heartbeat 正文生成，但同窗 CRWV/NBIS direct 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 运行态复核（2026-07-18 15:02 CST）

- `b87c4cb7` / `9d030286` 已修复并记录 direct `CRWV` 被 CWY 引用型产品制造假歧义的问题；14:00-14:02 CST 两条 Web regression direct 均能精确核验 CoreWeave/CRWV 并成功收口。
- 但本轮 2026-07-18 11:00-15:02 CST 真实运行态仍复发，状态维持 `New/P2`：
  - `data/sessions.sqlite3` 同窗新增 11 条 user / 11 条 assistant，近期 Web direct / regression session 均以 assistant 收口，`last_message_role=user` 为 0；assistant final 污染扫描未命中内部路径、raw tool、`data_fetch`、`cron_job`、SQLite、panic、provider 原始错误或 `<think>`。
  - Web direct session `Actor_web__direct__web-user-4d761588537b` 在 12:49-12:52 CST 连续对 `Cohr`、`美国公司coherent corp`、`Coherent`、`Cohr` 只返回“无法确认对应哪家上市公司或证券 / 证券实体解析暂时未能确认”，随后同一 session 对 `Acls` 可核验 Axcelis 并成功输出行情 / 技术分析，说明故障是部分实体解析 fail-closed，不是 direct 全链路不可用。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续有 332 条 `runner_error`、173 条定时任务执行失败和 256 条“证券实体解析暂时未能确认”信号。15:00 CST 代表 heartbeat 仍包括 AAOI 被误识别为 `SEC`、ASTS 被截成 `AST`、ORCL 多上市地候选，以及 TSLA/RKLB/绿田机械/Monitor_Watchlist_11/Cerebras/SIVE/NVDA/闪迪/光模块/存储板块等 fail-closed。
  - 判断：CRWV 的引用型产品修复已生效，但 scheduler / heartbeat 与部分 direct 公司名仍会被实体 guard / resolver 拦截或误抽任务上下文词，仍是同一链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler / heartbeat 正文生成，但同窗 CRWV、ACLS 等 direct 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 运行态复核（2026-07-18 14:02 CST）

- `b87c4cb7` 已修复并部署 direct `CRWV` 被 CWY 引用型产品制造假歧义的问题；生产 `crwv当前价` 与 `crwv预计估值多少` 均精确核验 CoreWeave/CRWV 并成功收口，说明 FMP/DataFetch 和 direct 精确 ticker 主链路健康。
- 同一次重启后的 14:00-14:02 CST scheduler / heartbeat 窗口仍复现本 P2：光模块任务将 `800G` 送入行情覆盖，ASTS 任务出现截断 `AST`，AAOI 任务将 `SEC` 当证券代码，存储任务出现 `NAND`，ORCL 任务仍落成多上市地候选。状态维持 `New/P2`；CRWV 的关系分类修复不能作为关闭 scheduler 任务正文边界缺陷的证据。

## 运行态复核（2026-07-18 11:01 CST）

- 运行态在 2026-07-18 07:01 CST 回退后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-18 07:00-11:01 CST`。
  - `data/sessions.sqlite3` 同窗新增 16 条 user / 17 条 assistant，近期 Web / Feishu / Discord direct 或 scheduler session 均以 assistant 收口，`last_message_role=user` 为 0；assistant final 污染扫描未命中 `<think>`、本机路径、SQLite、panic、provider 原始错误、raw tool、`data_fetch`、`cron_job` 或 fenced JSON。
  - 最近四小时无非文档代码提交；问题不是新修复后的代码状态变化，而是当前 live 继续复发。
  - `data/runtime/logs/web.log.2026-07-18` 在 08:00-11:00 CST 继续记录 262 条 `runner_error`、135 条定时任务执行失败和 207 条“证券实体解析暂时未能确认”信号。
  - 代表样本包括：AAOI 1.6T 光模块心跳检测把任务上下文里的 `SEC` 当证券代码且无行情覆盖；ASTS 重大异动心跳监控把 `ASTS` 截成 `AST`；ORCL 大事件监控仍落成 Oracle 多候选；光模块 / 存储板块关键事件提醒把 `800G` / `NAND` 当证券代码；TSLA、NVDA、NBIS、中际旭创、光迅科技、闪迪、全天原油、绿田机械、Monitor_Watchlist_11、Cerebras、SIVE 等任务继续 fail-closed。
  - 判断：最新样本仍是实体 guard / resolver 把任务词、行业词、缩写或显式 ticker 错送进证券核验，随后以实体解析、多候选或无覆盖错误阻断业务正文；与既有缺陷同根，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分 scheduler / heartbeat 正文生成，但同窗 direct / scheduler 均有 assistant 收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 运行态复核（2026-07-18 07:01 CST）

- 运行态在 2026-07-17 23:59 CST 代码级修复后继续复发，状态从 `Fixed` 回退为 `New`：
  - 本轮巡检窗口为 `2026-07-18 03:00-07:01 CST`。
  - `data/sessions.sqlite3` 同窗新增 3 条 user / 5 条 assistant，近期 direct / scheduler session 均以 assistant 收口，`last_message_role=user` 为 0；assistant final 污染扫描未命中 `<think>`、本机路径、SQLite、panic、provider 原始错误、raw tool、`data_fetch`、`cron_job` 或 fenced JSON。
  - 最近非文档提交 `3bf74589 fix(investment): recognize heartbeat ticker task subjects` 发生在 2026-07-18 03:08 CST，晚于上一轮代码级修复；但 05:00-07:00 CST 真实运行态仍出现同根失败：
    - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 先把任务正文中的 `PCE` 识别成证券代码且数据供应商无覆盖，随后写入 `scheduler_failure=true` 的用户可见执行出错。
    - 05:30 CST Feishu scheduler session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 的 `美股收盘后跨市场复盘` 只返回“证券实体解析暂时未能确认当前点名的公司”，任务主体未生成。
    - 06:00 CST Feishu scheduler session `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` 的 `每日美股盘后收盘复盘` 把任务中“纳指或 Nasdaq-100”落成 `NASDAQ 100 / Nasdaq, Inc. / NASDAQ Composite / NASDAQ Biotechnology` 多候选澄清，业务正文未生成。
  - `data/runtime/logs/web.log.2026-07-17` 在 03:00-07:01 CST 继续记录 340 条实体 / 候选 / 无覆盖相关信号、341 条 `runner_error` 与 175 条定时任务执行失败；07:00 CST 代表任务包括 ASTS、AAOI、ORCL、TSLA、NVDA、NBIS、闪迪、光迅科技、Cerebras 与持仓重大事件等 heartbeat 继续在实体解析或多候选阶段 fail-closed。
  - 判断：代码级修复对任务名证券语境有补强，但 live scheduler / heartbeat 仍会把普通宏观指标、指数名、任务上下文词或显式 ticker 误导入证券实体核验并阻断业务正文，仍是同一实体 guard / resolver 链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分投研 scheduler / heartbeat 正文生成，但同窗 direct / scheduler 均有 assistant 收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 代码级修复（2026-07-17 23:59 CST）

- 本轮在 `crates/hone-channels/src/investment_response_guard.rs` 收紧了 heartbeat / scheduler 任务名的证券语境识别：
  - `大事件`、`异动`、`触发条件`、`心跳监控`、`心跳检测`、`破位预警`、`价格播报` 现在会被视为有效的证券讨论上下文，`ORCL 大事件监控`、`TSLA 正负触发条件心跳监控`、`ASTS 重大异动心跳监控`、`光迅科技 002281.SZ 关键事件心跳提醒` 这类任务名不再因为缺少 `股价/股票/报价` 等字样而把显式 ticker 丢掉。
  - 同一批 heartbeat 任务词也加入 `request_may_need_auxiliary_entity_extraction(...)` 的 generic 清洗词表，避免这类已含显式 ticker 的非交互任务再被误判成“必须走辅助实体抽取”。
- 新增回归：
  - `security_identifier::tests::scanner_keeps_bare_ticker_before_chinese_heartbeat_suffix`
  - `investment_response_guard::tests::heartbeat_subject_markers_count_as_security_context`
- 验证通过：
  - `cargo test -p hone-channels scanner_keeps_bare_ticker_before_chinese_heartbeat_suffix --lib -- --nocapture`
  - `cargo test -p hone-channels scheduled_ticker_subject_is_available_without_parsing_the_envelope --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_subject_markers_count_as_security_context --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 当前先按代码级 `Fixed` 记录。本轮没有重启 live 服务，`2026-07-17` 之后的真实运行态是否完全止血，仍需后续巡检窗口复核；若 heartbeat 继续把显式 ticker 任务落成“证券实体解析暂时未能确认”，应基于新样本重新回退。

## 复发记录（2026-07-17 19:02 CST）

- 运行态在新一轮跨市场 ticker 解析修复后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-17 23:00-2026-07-18 03:00 CST`。
  - `data/sessions.sqlite3` 同窗新增 13 条 user / 12 条 assistant，近期 session 均以 assistant 收口，`last_message_role=user` 为 0；assistant final 污染扫描未命中空回复、`<think>`、本机路径、SQLite、panic、provider 原始错误、raw tool、`data_fetch`、`cron_job` 或 fenced JSON。
  - 最近非文档提交 `4d419770 fix(investment): unify cross-market ticker resolution` 发生在 2026-07-17 23:50 CST；该提交后仍有真实用户可见失败样本：
    - 2026-07-18 00:00 CST，Feishu scheduler actor session `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 `AAOI 每日动态监控` 和 `TEM 每日动态监控` 仍返回“证券实体解析暂时未能确认当前点名的公司”，同一串内 `RKLB` 可正常核验，说明不是全链路停摆。
    - 2026-07-18 02:53-02:56 CST，Web direct session `Actor_web__direct__web-user-266454c88ed6` 用户连续输入 `clbk 的基本面扫描`、`CLBK是家什么公司`、`COLUMBIA FINANCIAL`，assistant 三次只返回证券实体解析失败，没有给出候选、澄清或业务解释。
  - `data/runtime/logs/web.log.2026-07-17` 同窗仍有 62 条 heartbeat `runner_error`、49 条定时任务执行失败与 60 条“证券实体解析暂时未能确认”信号，代表 ORCL、ASTS、TSLA、Monitor_Watchlist_11 等 heartbeat 继续被同类 guard / resolver 阻断。
  - 判断：新提交对部分跨市场 / regression ticker 有止血，但 scheduler heartbeat 与普通用户 direct 对显式 ticker / 公司名仍会 fail-closed，仍是同一实体 guard / resolver 链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler 正文生成，但同窗仍有 RKLB 与多条 heartbeat 成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

- 运行态在投研相关修复后部分止血但仍复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-17 15:01-19:02 CST`。
  - `data/sessions.sqlite3` 同窗新增 8 条 user / 9 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - 最近非文档提交 `ff3852c3 fix(investment): preserve exact ticker resolution`、`7d14c87f fix(investment): enforce deep valuation intent` 后，17:35 / 17:47 CST 同题 RKLB Web direct 已能核验 `Rocket Lab USA, Inc.（RKLB；NASDAQ Capital Market）` 并输出价格区间；17:11 CST 修复前同题仍返回“证券实体解析暂时未能确认当前点名的公司”。
  - 但 `data/runtime/logs/web.log.2026-07-17` 17:30-19:00 CST 仍有 ORCL、TSLA、ASTS、Monitor_Watchlist_11 等 heartbeat 被拦成“证券实体解析暂时未能确认”或“无法确认你提到的 Oracle（ORCL）对应哪家上市公司或证券”，并落成 `failure_kind=runner_error` / 定时任务执行失败。
  - 判断：direct 显式 ticker 解析有部分止血，但 heartbeat / scheduler 路径仍被实体优先 / 投研完整性 guard 拦截，仍属于同一链路，不新建重复缺陷。
  - 严重等级维持 `P2`：问题直接阻断部分投研 scheduler / heartbeat 正文生成，但同窗 RKLB direct 和多条 heartbeat 仍有成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 复发记录（2026-07-17 15:01 CST）

- 运行态继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-17 11:01-15:01 CST`。
  - `data/sessions.sqlite3` 同窗新增 16 条 user / 16 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - `data/runtime/logs/web.log.2026-07-17` 同窗有 74 条 `runner_error`、54 条定时任务执行失败和 72 条“证券实体解析”信号。
  - 11:00-15:00 CST heartbeat 继续把 `全天原油价格3小时播报`、`ORCL 大事件监控`、`ASTS 重大异动心跳监控`、`TSLA 正负触发条件心跳监控`、`Monitor_Watchlist_11` 等任务拦成“证券实体解析暂时未能确认当前点名的公司”，其中 12:00 CST 对 `特斯拉 TSLA` 也返回“我暂时无法确认你提到的‘特斯拉 TSLA’对应哪家上市公司或证券”。
  - 13:01 CST Feishu direct 用户说“不再持有绿田机械，不要再推送任何关于绿田机械的相关消息”，assistant 只返回证券实体解析失败，未执行用户明确的取消 / 清理意图。
  - 14:51 CST Feishu direct 用户请求“深度分析下 BE 的投资 thesis 以及买入点”，assistant 只返回证券实体解析失败；同窗 13:22 MU / SNDK 直聊可解析到标的但落成投研完整性失败，说明当前坏态是部分实体 / 投研 guard 阻断，而非全链路不可用。
- 判断：最新样本仍是实体优先 / 投研完整性 guard 过宽、实体解析候选召回不足或上下文输入不干净链路，更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler 正文生成，但同窗仍有定时任务列表查询、FOTO 心跳删除和部分金融 direct 正常收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 复发记录（2026-07-17 07:01 CST）

- 运行态在 07:34 / 08:09 / 09:30 等投研链路修复提交后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-17 07:01-11:02 CST`。
  - `data/sessions.sqlite3` 同窗新增 10 条 user / 10 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - `data/runtime/logs/web.log.2026-07-16` / `web.log.2026-07-17` 同窗有 86 条 runner / 执行失败；07:30-11:00 CST heartbeat 仍把 `TSLA`、`HIMS`、`ORCL`、`ASTS`、`Monitor_Watchlist_11`、原油等明确标的或任务上下文拦成“证券实体解析暂时未能确认”或“无法确认你提到的”。
  - 10:05 CST Feishu direct 用户将 `NBIS` 误拼成 `nibs` 时返回证券实体解析失败；10:06 CST 同用户用 `nbis` 重试成功，说明当前链路对近似拼写仍 fail-closed，但不是全链路不可用。
  - 10:53 CST Feishu direct 用户只发 `中船特气`，assistant 返回“证券实体解析暂时未能确认当前点名的公司”，没有给出 A 股常见中文实体候选或澄清选项。
  - 同窗 RMBS / NBIS regression direct、Citrini / SemiAnalysis 文章跟踪 scheduler 可成功收口，说明故障不是 scheduler/direct 全链路不可用。
- 判断：最新样本仍是实体优先 / 投研完整性 guard 过宽、实体解析候选召回不足或上下文输入不干净链路，更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler 正文生成，但同窗仍有金融 direct / scheduler 成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

- 运行态在最近投研链路修复后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-17 03:01-07:01 CST`。
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 6 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - `data/runtime/logs/web.log.2026-07-16` 同窗有 104 条 runner / 执行失败，代表实体 guard 样本包括 TSLA、AST SpaceMobile、SPY、Samsung、SIVE、TEM、AT&T、VIX、Cerebras Systems、BRK.B、AWS、Nokia、Meta、Nvidia 等被判为无法确认或多候选。
  - 05:00 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` 先因证券实体与当前数据预检超过 45 秒终止，再写入用户可见执行出错；06:11 CST RMBS 和 06:53 CST ISRG direct 也在已核验行情前缀后落成投研完整性失败。
  - 同窗 06:31 CST Web scheduler 组合跟踪和 07:01 CST Feishu 持仓早报可成功输出长正文，说明故障不是 scheduler/direct 全链路不可用。
- 判断：最新样本仍是实体优先 / 投研完整性 guard 过宽或上下文输入不干净链路，更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题直接阻断部分投研 direct / scheduler 正文生成，但同窗仍有金融 scheduler 成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 复发记录（2026-07-17 03:02 CST）

- 运行态在最近投研链路修复后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-16 23:01-2026-07-17 03:03 CST`。
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 5 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - `data/runtime/logs/web.log.2026-07-16` 同窗仍有 150 条 heartbeat `runner_error`、112 条“无法确认你提到的”和 142 条“请补充公司全名”。
  - 代表样本包括 `原文`、原油、AAOI、SNDK、Samsung、光迅科技、VIX、Meta、SIVE、ASTS、TSLA 等明确标的或上下文词被实体识别 / 投研完整性 guard 拦截，任务落成 `failure_kind=runner_error` 并跳过发送。
  - 同窗 Web regression direct 00:59 / 02:46 CST 只返回投研完整性失败文案并标记 `AgentFailed`，但 00:43-00:57 CST AAPL 报价样本可成功收口，说明故障不是全链路不可用。
- 判断：最新样本仍是实体优先 / 投研完整性 guard 过宽或上下文输入不干净链路，更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题直接阻断投研 direct / scheduler 正文生成，但同窗仍有 direct 金融问题成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 复发记录（2026-07-16 23:02 CST）

- 运行态在最近两次投研链路修复后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-16 19:02-23:02 CST`。
  - `data/sessions.sqlite3` 同窗新增 29 条 user / 29 条 assistant，全部以 assistant 收口；未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - 最近非文档提交 `335c4b73 fix: harden investment chat execution and recovery` 发生在 21:23 CST，`4aa21b29 fix(agent): preserve explicit tickers across extraction` 发生在 21:33 CST。
  - 但提交后仍有 direct / scheduler 样本被同类实体识别 / 投研完整性 guard 拦截：
    - 21:41 / 21:44 CST，`session_id=Actor_web__direct__web-user-31e5cde131ea`，用户分别询问 `MRVL（迈威尔科技）` 与 `ARM`，assistant 仍返回“我暂时无法确认你提到的 MRVL（迈威尔科技）对应哪家上市公司或证券”或“这次回答未通过投研完整性检查”。
    - 23:00 CST，`session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`，Feishu scheduler `核心观察股池晚间快报` 的任务正文显式列出核心股 `MSFT、NVDA、GOOGL、AAPL、AVGO、AMZN、META`，assistant 却返回“我暂时无法确认你提到的 AMZN 对应哪家上市公司或证券”。
    - 21:00 CST，`session_id=Actor_web__direct__web-user-afc1cabadbf8`，Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 先返回“我暂时无法确认你提到的 原文 对应哪家上市公司或证券”，随后写入 `定时任务执行出错`。
  - `data/runtime/logs/web.log.2026-07-16` 同窗还有 125 条实体 / 投研完整性失败信号，23:00-23:01 CST 代表 heartbeat runner_error 包括 `全天原油价格3小时播报` 无法确认具体证券、`闪迪关键事件心跳提醒` 被 Samsung 多候选阻断、`SIVE POET/Nokia/1.6T DFB 心跳检测` 无法确认 SIVE、`光迅科技关键事件心跳提醒` / `美股黄金坑信号心跳检测` 无法确认“原文”。
- 判断：这次不只是 `REPEAT` / `EBITDA` 配置词误抽，显式 ticker 和常见上市公司名称也会被 guard / resolver 拦截，仍属于同一条实体优先 / 投研完整性 guard 过宽或上下文输入不干净链路，因此更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题直接阻断投研 direct / scheduler 正文生成，但同窗仍有多个 direct 金融问题成功收口，未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

## 复发记录（2026-07-16 15:03 / 19:02 CST）

- 运行态在最近非文档修复后继续复发，状态维持 `New`：
  - 本轮巡检窗口为 `2026-07-16 15:03-19:02 CST`。
  - `data/sessions.sqlite3` 同窗新增 6 条 user / 6 条 assistant，全部以 assistant 收口，未见长期 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - 最近非文档提交 `fa65dfef fix(agent): resolve ordinary ticker entities first` 发生在 16:23 CST，修改 `investment_response_guard` 与 live entity-search 手工回归脚本。
  - 但 `data/runtime/logs/web.log.2026-07-16` 在 18:30 / 19:00 CST 后仍记录同类实体识别 / 投研完整性 guard 失败；同窗实体 / 投研完整性相关 WARN / ERROR 共 216 条，`HeartbeatDiag runner_error` 共 112 条。
  - 代表样本包括 `全天原油价格3小时播报` 无法确认具体公司或证券、`存储板块关键事件心跳提醒` 无法确认 `SNDK`、`TEM大事件心跳监控` 无法确认 `TEM`、`ASTS 重大异动心跳监控` 被上下文中的 `AT&T` 多候选阻断、`光迅科技关键事件心跳提醒` 无法确认 `光迅科技 002281.SZ`、`Cerebras IPO与业务进展心跳监控` 无法确认 `Meta`、`SIVE POET/Nokia/1.6T DFB 心跳检测` 无法确认 `Sivers Semiconductors（SIVE）`。
  - 判断：这次说明 live heartbeat 路径在“普通 ticker 先解析”修复后仍会被实体优先 / 投研完整性 guard 批量拦截；当前仍属同一 scheduler finance entity guard 过宽 / 上下文输入不干净链路，因此补充原文档，不新建重复缺陷。
  - 严重等级维持 `P2`：问题影响 heartbeat 提醒生成和发送，但本窗 direct 会话仍能收口，且 18:38 CST 同题 `INTL` direct 已成功输出业务正文；未见错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`，不创建 GitHub Issue。

- 运行态在 11:02 CST live probe 止血结论后再次复发，状态从 `Fixed` 回退为 `New`：
  - 本轮巡检窗口为 `2026-07-16 11:00-15:01 CST`。
  - `data/sessions.sqlite3` 在 11:00 CST 后没有新增 user / assistant 消息，`sessions.last_message_role=user` 新增为 0；未见直聊 user-only 悬挂、错投、空回复、内部路径 / raw tool / `<think>` 外泄或全渠道不可用。
  - `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，因此本轮以 `data/runtime/logs/web.log.2026-07-16` 的 live runtime 证据为准。
  - `data/runtime/logs/web.log.2026-07-16` 在 11:30-15:01 CST 连续记录 400 条 `证券实体识别结果不完整。请补充公司全名或明确 ticker。` 相关 WARN / ERROR 行，覆盖 11:30、12:00、12:30、13:00、13:30、14:00、14:30、15:00 多个 heartbeat 批次；每个失败通常同时写一条 `HeartbeatDiag runner_error` 和一条渠道跳过发送日志。
  - 代表样本包括 `全天原油价格3小时播报`、`小米30港元破位预警`、`AAOI 1.6T 光模块心跳检测`、`TSLA 正负触发条件心跳监控`、`TEM大事件心跳监控`、`NBIS关键事件心跳提醒`、`RKLB异动监控`、`ORCL 大事件监控`、`Monitor_Watchlist_11` 等 Feishu / Web heartbeat 任务，均在 runner / guard 阶段落成 `failure_kind=runner_error` 并跳过发送。
  - 最近非文档代码提交 `7a18f552 fix(agent): enforce entity-first investment analysis` 发生在 13:30 CST；15:00 CST 之后 runtime 日志仍继续出现同类 guard 失败。当前尚不能证明 live 进程已加载该提交，但从用户影响角度看，运行态仍是活跃坏态。
- 判断：这次不再只是把 `REPEAT` / `EBITDA` 单个配置词或指标词误抽为证券实体，而是实体优先 / 投研完整性 guard 在多条 heartbeat 任务上批量无法完成实体解析，导致提醒整轮不发送。根因仍属于同一条 scheduler finance entity guard 过宽 / 上下文输入不干净链路，因此更新原文档，不新建重复缺陷。
- 严重等级维持 `P2`：问题影响 heartbeat 提醒是否能生成和发送，但本窗没有直聊未回复、跨用户错投、数据破坏、敏感信息泄露或全渠道停摆证据，因此不是 `P1`，不创建 GitHub Issue。

## 最新进展（2026-07-16 11:02 CST）

- 运行态已通过 live probe 复核，状态从 `New` 更新为 `Fixed`：
  - 最近非文档代码提交 `9a1cceb7 fix(agent): keep scheduler metadata out of stock routing` 已在 2026-07-16 09:44 CST 落地，明确把 scheduler metadata 从 stock routing / finance entity guard 路径剥离。
  - `data/sessions.sqlite3` 在 `2026-07-16 07:02-11:02 CST` 新增两条专门的 live repeat guard probe：
    - `2026-07-16T09:38:35.841480+08:00`，`session_id=Actor_web__direct__live-repeat-guard-probe-20260716`，用户触发文本含 `权威触发配置：repeat=daily，北京时间 09:40`，assistant 在 `09:38:41.010007+08:00` 返回 `调度路由健康检查通过。`
    - `2026-07-16T09:43:21.445555+08:00`，`session_id=Actor_web__direct__live-repeat-guard-probe-20260716-final`，用户触发文本含 `权威触发配置：repeat=daily，北京时间 09:44`，assistant 在 `09:43:27.310289+08:00` 返回 `调度路由健康检查通过。`
  - 两条 probe 均未再把 `REPEAT` 当作不可核验证券实体，也没有出现 `EBITDA` / 指标词误杀、通用失败、错投或未收口。
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 新增 5 条 user / 5 条 assistant，最近 4 个 session 全部以 assistant 收口；未见长期 user-only 悬挂、空回复、内部实现外露或全渠道不可用。
- 判断：本轮证据已经覆盖导致 07:02 回退的 `repeat=daily` 运行态路径，且新的 commit 专门处理 scheduler metadata 路由，因此当前按运行态止血更新为 `Fixed`。若后续真实 scheduler 再出现 `REPEAT` / `EBITDA` 或同类配置 / 指标词误杀，应基于新样本重新回退。

## 复发记录（2026-07-16 07:02 CST）

- 运行态在代码级修复后继续复发，状态从 `Fixed` 回退为 `New`：
  - 最近非文档代码提交 `c776b808 fix(agent): ignore scheduler config tokens in investment guard` 已在 2026-07-16 03:04 CST 落地，包含 `key=value` 配置上下文过滤、财务指标词排除和 3 条 guard 回归。
  - 但 `data/sessions.sqlite3` 在 `2026-07-16 03:02-07:02 CST` 的真实运行窗口继续出现同根用户可见失败。
  - `2026-07-16T04:30:01.134+08:00`，`session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`，Feishu scheduler 触发 `OWALERT_PostMarket`，权威触发配置含 `repeat=trading_day`；assistant 在 `04:30:04.171082+08:00` 返回：`当前无法稳定核验证券实体 `REPEAT`，已停止生成可能指向错误公司的分析。`
  - `2026-07-16T05:30:01.180780+08:00`，`session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`，Feishu scheduler 触发 `美股收盘后跨市场复盘`，任务正文包含普通估值指标；assistant 在 `05:30:04.494984+08:00` 返回：`当前无法稳定核验证券实体 `EBITDA`，已停止生成可能指向错误公司的分析。`
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 有 10 条 user / 11 条 assistant，07:00 边界任务也在 07:02:55 收口，未见长期 user-only 悬挂、错投、敏感信息泄露或全渠道不可用，因此维持功能性 `P2`，非 P1，不创建 GitHub Issue。
- 判断：当前证据证明 live 运行态仍会影响用户；可能是修复尚未进入运行进程，也可能是 guard 仍有未覆盖路径。无论哪种情况，缺陷对当前用户仍为活跃状态。

## 代码级修复记录（2026-07-16 03:03 CST）

- 本轮已在 `crates/hone-channels/src/investment_response_guard.rs` 收紧证券实体 hint 抽取：
  - `key=value` 形态里的配置键不再参与证券实体识别，`repeat=daily` 这类 scheduler 权威配置不会再把 `REPEAT` 当作标的。
  - 常见财务指标词新增排除，并对 `repeat=trading_day`、`repeat=daily` 这类调度频率值做上下文过滤，避免 `DAILY` / `TRADING` 一类调度词继续误入 guard。
  - `EV/EBITDA` 一类估值指标片段不再触发证券实体核验。
- 新增回归：
  - `repeat_assignment_is_not_treated_as_security_hint`
  - `metric_tokens_are_not_treated_as_security_hint`
  - `real_ticker_still_wins_over_repeat_assignment_noise`
- 验证通过：
  - `cargo test -p hone-channels investment_response_guard::tests --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 该修复已通过代码级验证，但 2026-07-16 07:02 CST 巡检确认 live 运行态仍复发，因此当前状态不再保持 `Fixed`。

## 证据来源

- `data/sessions.sqlite3`，本轮真实消息窗口 `2026-07-15 19:01-23:01 CST`。
- `session_messages` 按真实 `timestamp` 新增 48 条 user / 55 条 assistant，近期 28 个 session 均以 assistant 收口，`last_message_role=user` 为 0；未见全渠道未回复、错投或敏感信息泄露。
- `2026-07-15T20:30:00.576659+08:00`，`session_id=Actor_feishu__direct__ou_5fbceebf26fcbb242fd6585745222c8063`，Feishu scheduler 触发 `老王说事与巴芒投资美股财报季个股判断`。任务正文是每天 20:30 的财报季个股判断，权威触发配置含 `repeat=trading_day`；assistant 在 `20:30:04.772759+08:00` 返回“当前无法稳定核验证券实体 `REPEAT`，已停止生成可能指向错误公司的分析。”
- `2026-07-15T21:00:01` 附近，Feishu scheduler `session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768` 与 `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 也分别返回同一 `REPEAT` 实体核验失败文案。
- 同窗仍有 20:30 / 21:35 / 23:00 成功 scheduler 样本，因此维持功能性 `P2 / New`，不升级为 P1。

- `data/sessions.sqlite3`，本轮真实消息窗口 `2026-07-15 15:02-19:02 CST`。
- 近期非文档代码提交：`c29de55c fix(agent): enforce verified stock response contracts`，提交时间 `2026-07-15 16:44:30 +0800`，早于本轮两条误杀样本。
- 同窗 `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，因此本轮以 `session_messages` 中的真实 scheduler/direct transcript 为准，不依赖本地 `cron_job_runs` 判断送达状态。

关键样本：

- `2026-07-15T17:30:00.856071+08:00`，`session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`，`ordinal=527`，Feishu scheduler 触发 `A股港股收盘后跨市场复盘`。任务正文要求 A/H 收盘复盘，内容里包含估值指标 `EV/EBITDA` 一类普通金融术语。`ordinal=528` assistant 在 `17:30:04.210692+08:00` 返回：`当前无法稳定核验证券实体 `EBITDA`，已停止生成可能指向错误公司的分析。`
- `2026-07-15T18:00:00.987468+08:00`，`session_id=Actor_web__direct__web-user-ba50cb9401c0`，`ordinal=13`，Web scheduler 触发 `18:00 美股盘前 X 英文帖`。权威触发配置中含 `repeat=daily`，任务是生成英文 X 帖草稿，不是分析 `REPEAT` 证券。`ordinal=14/15` assistant 在 `18:00:04` 返回实体核验失败，并落成用户可见调度失败：`当前无法稳定核验证券实体 `REPEAT`，已停止生成可能指向错误公司的分析。`
- 本轮同窗 `session_messages` 按真实 `timestamp` 有 8 条 user / 9 条 assistant；近期会话均有 assistant 收口，没有发现全局未回复或错投，因此该问题按功能性 P2，而不是 P1。

## 端到端链路

1. 用户或 scheduler 触发金融相关定时任务。
2. Runner / answer contract 执行新增的证券实体核验 guard。
3. Guard 从完整任务文本中抽取大写词或指令字段，误把 `EBITDA`、`REPEAT` 这类指标 / 配置词当作证券实体。
4. 实体核验失败后直接中止生成，并向用户写入失败提示。
5. 原本应生成的市场复盘或英文 X 帖草稿没有产出。

## 期望效果

- 金融实体核验 guard 应只校验用户真正要求分析、定价、建仓或推送的证券实体。
- `EBITDA`、`EV/EBITDA`、`repeat=daily`、任务配置字段、格式指令和普通指标词不应被当作 ticker / 股票实体。
- 当实体识别存在歧义时，应优先忽略配置词，或把歧义限制在当前用户请求的真实标的候选上，而不是阻断整条 scheduler 链路。

## 当前实现效果

- Guard 把 `EBITDA` 和 `REPEAT` 当作需要稳定核验的证券实体。
- 两条不同渠道的 scheduler 样本均在数秒内失败，未生成业务正文。
- Web scheduler 还额外写入一条用户可见失败消息，明确暴露错误实体名 `REPEAT`。

## 用户影响

- 定时任务按时触发但产出被错误拦截，用户收不到应有的复盘或内容草稿。
- 失败信息让用户误以为任务中包含了不可核验的证券实体，实际只是系统把指标 / 配置词误识别。
- 当前证据覆盖 Feishu scheduler 与 Web scheduler 两条链路，但同窗仍有其他直聊正常收口，未见全渠道不可用或数据泄露，因此定为 P2。

## 根因判断

- 与 `c29de55c fix(agent): enforce verified stock response contracts` 时间相邻，且失败文案来自新增的投研完整性 / 证券实体核验 guard，疑似实体抽取范围过宽。
- 抽取逻辑没有区分任务配置、金融指标、正文格式指令和真正的证券 ticker。
- Scheduler 任务会把权威触发配置、任务说明和用户原始正文一起送入模型 / guard，增大了误抽取 `REPEAT` 这类配置词的概率。
- 2026-07-16 03:04 CST 代码级修复后仍有 live 复发；若确认运行进程已加载 `c776b808`，则说明修复只覆盖了部分 guard 路径，仍有另一条实体抽取 / 完整性检查路径会扫描完整 scheduler prompt。

## 下一步建议

- 先确认当前 Feishu / Web scheduler runtime 是否已重启到包含 `c776b808` 的二进制或解释层配置；如果未生效，重启后用 `repeat=trading_day` 与 `EV/EBITDA` scheduler 样本做 live 复核。
- 如果 runtime 已是最新代码，继续沿调用栈定位 `当前无法稳定核验证券实体` 的实体来源，确认是否还有第二套 guard / answer contract 在扫描完整 prompt。
- 若后续再出现误伤，优先继续沿“上下文过滤”而不是堆全局 denylist，避免把真实 ticker 一并排除。
- 更稳妥的长期方向仍是让 guard 只校验模型准备输出交易结论的实体，或只校验工具已解析出的候选 ticker，而不是扫描完整 prompt 文本。
## 最新运行态复核（2026-07-17 23:02 CST）

- `data/runtime/logs/web.log.2026-07-17`
  - 巡检窗口：2026-07-17 19:01-23:01 CST。
  - 23:00 CST heartbeat 继续出现同根实体核验阻断：`ORCL 大事件监控`、`ASTS 重大异动心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控` 均落成 `failure_kind=runner_error`，错误为“证券实体解析暂时未能确认当前点名的公司。请稍后重试，或补充明确 ticker。”
  - 同窗统计命中 64 行实体 guard 相关日志、62 条 `runner_error`、57 条定时任务执行失败。
- `data/sessions.sqlite3`
  - 同窗 2 个近期 scheduler session 均以 assistant 收口，未见长期 user-only 悬挂、错投或全渠道不可用。
- 本轮判断
  - 这不是新的根因，而是既有投研实体 guard 在 heartbeat / scheduler 链路中继续误拦显式监控对象或任务上下文。
  - 影响是部分定时监控跳过发送；同窗仍有 Feishu / Web scheduler 正常收口，因此维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-19 03:01 CST）

- `data/runtime/logs/web.log.2026-07-18`
  - 巡检窗口：2026-07-18 23:01-2026-07-19 03:01 CST。
  - 03:00 CST `AAOI 1.6T 光模块心跳检测` 继续把任务上下文里的 `SEC` 当证券代码核验，并因当前数据供应商没有返回同代码行情覆盖而落成 `runner_error`，本轮不发送。
  - 03:00 CST `ORCL 大事件监控` 仍把 Oracle 解析为多个上市地候选：`ORCP.L`、`ORCL.SW`、`ORCL`、`ORC.DE`，要求补交易所后缀或公司全名，导致本轮不发送。
  - 同批仍有多条显式监控对象 heartbeat 因实体 / 上下文核验失败或 runner error 跳过发送。
- `data/sessions.sqlite3`
  - 同窗 Web direct 23:50 CRWV/NVDA regression 成功输出完整业务正文，说明当前不是全链路投研不可用，而是 scheduler / heartbeat 的实体与上下文 guard 仍会在部分任务上 fail-closed。
- 本轮判断
  - 该样本继续落在既有实体 guard / context extraction 误拦范围，不新建重复缺陷。
  - 用户影响仍是部分 heartbeat 监控缺口；同窗未见错投、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-19 23:01 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-19 19:01-23:01 CST。
  - 21:00 CST Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 继续把宏观指标 `PCE` 当证券代码核验，assistant final 写出“已识别证券代码 `PCE`，但当前数据供应商没有返回同代码行情覆盖”，随后落成用户可见定时任务执行出错提示。
- `data/runtime/logs/web.log.2026-07-19`
  - 19:30 / 20:30 / 23:00 CST `AAOI 1.6T 光模块心跳检测` 继续把任务正文里的 `SEC` 当证券代码并 fail-closed。
  - 19:30 / 20:30 / 23:00 CST `ORCL 大事件监控` 继续把 Oracle 解析为多个上市地候选：`ORCP.L`、`ORCL.SW`、`ORCL`、`ORC.DE`，要求补充交易所后缀或公司全名。
- 本轮判断
  - 这些样本仍落在既有实体 guard / scheduler context extraction 误拦范围，不新建重复缺陷。
  - 影响是部分 Web / Feishu scheduler 和 heartbeat 监控任务跳过或发送失败提示；同窗仍有 direct 和 scheduler 成功收口样本，因此维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-22 11:03 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-22 07:03-11:03 CST。
  - 08:30 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报` 的任务正文包含存储行业词 `NAND`，assistant final 返回“已识别证券代码 `NAND`，但当前数据供应商没有返回同代码行情覆盖”，没有生成 SNDK 行情与行业简报。
  - 08:30 CST Web scheduler `187只关注股临近财报日提醒` 写入用户可见 `执行出错` 补偿消息；本轮未确认该条同样来自实体 guard，暂不作为新根因建档。
- `data/runtime/logs/web.log.2026-07-22`
  - 08:00-11:00 CST heartbeat 仍有 14 行 `SEC` 误抽日志，代表样本为 `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码并 fail-closed。
  - 同窗还有 14 行 Oracle 多候选日志，`ORCL 大事件监控` 继续要求补充交易所后缀或公司全名，导致本轮不发送。
- 本轮判断
  - `NAND` 是行业 / 技术名词，不是本轮用户要求分析的证券标的；该样本与此前 `PCE`、`SEC`、`REPEAT`、`EBITDA` 同属 guard 扫描完整 scheduler/heartbeat 文本导致的误拦。
  - 影响是部分 scheduler / heartbeat 任务跳过或给用户失败提示；同窗 direct 与多个 scheduler 仍正常收口，未见错投、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-22 23:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-22 19:01-23:02 CST。
  - 21:00 CST Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 的任务正文要求汇总宏观数据，包含 `PCE` 指标；assistant final 先返回“已识别证券代码 `PCE`，但当前数据供应商没有返回同代码行情覆盖”，随后写入用户可见 `定时任务「盘前美股要闻与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
- `data/runtime/logs/web.log.2026-07-22`
  - 同窗 AAOI `SEC` 误抽日志继续出现 18 行，代表样本为 `AAOI 1.6T 光模块心跳检测` 把任务上下文里的 `SEC` 当证券代码并 fail-closed。
  - 同窗 ORCL 多候选日志继续出现 18 行，`ORCL 大事件监控` 仍要求补充交易所后缀或公司全名，本轮不发送。
- 本轮判断
  - 最新样本仍是 scheduler / heartbeat 任务正文、宏观指标和监管公告词被实体 guard 当作证券实体，或把公司名解析成多上市地候选后 fail-closed。
  - 影响是部分 Web / Feishu scheduler 和 heartbeat 监控任务失败或跳过；同窗 28 个更新 session 没有长期 user-only 残留、错投、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。

## 最新运行态复核（2026-07-23 11:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-23 07:01-11:02 CST。
  - 08:30 CST Web scheduler `187只关注股临近财报日提醒` 要求检查用户列出的 187 只 ticker 的未来 14 天财报日历；assistant final 却先核验 `Beijing Jingcheng Machinery Electric Company Limited（0187.HK）` 和 `MU`，随后落成用户可见 `定时任务「187只关注股临近财报日提醒」执行出错，请稍后重试。`
  - 08:30 CST Feishu scheduler `存储板块关键事件心跳提醒` / 相关 heartbeat 上下文仍包含行业词 `NAND`，但本窗没有把 `NAND` 单独作为新根因建档。
- `data/runtime/logs/web.log.2026-07-23`
  - 08:00、08:30、09:00、09:30、10:00、10:30 CST `AAOI 1.6T 光模块心跳检测` 继续把任务正文中的 `SEC` 当证券代码，并因无行情覆盖落成 `runner_error`，本轮不发送。
  - 同一批次 `ORCL 大事件监控` 继续把 Oracle 解析为 `ORCP.L`、`ORCL.SW`、`ORC.DE`、`ORCL` 多候选并 fail-closed。
- 本轮判断
  - `187只关注股` 被抽成 `0187.HK`，与 `PCE`、`SEC`、`NAND`、`REPEAT`、`EBITDA` 一样，仍属于 scheduler / heartbeat guard 扫描完整任务标题或正文而非真实待分析证券集合的同根问题。
  - 影响是部分 scheduler / heartbeat 任务失败或跳过；同窗 20 个更新 session 中其它直聊和定时报告可正常收口，未见错投、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，非 P1。
