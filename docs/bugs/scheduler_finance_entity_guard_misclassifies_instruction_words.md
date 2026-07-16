# Bug: Scheduler finance entity guard misclassifies instruction words as securities

- **发现时间**: 2026-07-15 19:02 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 复发记录（2026-07-16 15:03 CST）

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
