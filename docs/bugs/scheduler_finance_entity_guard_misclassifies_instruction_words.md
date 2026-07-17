# Bug: Scheduler finance entity guard misclassifies instruction words or explicit tickers as securities

- **发现时间**: 2026-07-15 19:02 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 复发记录（2026-07-17 19:02 CST）

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
