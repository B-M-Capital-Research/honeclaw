# Bug: Feishu 普通 scheduler 未触发静默条件时仍发送完整报告

## 发现时间

2026-07-21 03:02 CST

## Bug Type

Business Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，非 P1

## 最新进展

- 2026-08-04 10:01 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-04 06:02-10:01 CST。
    - 近窗继续有 `NOOP/noop/无触发/无新增` 等静默语义或跳过语义信号进入 deliver。
    - 代表样本：06:00 `中际旭创关键事件心跳提醒` 写 `本轮无高权重触发（noop）` 仍进入 deliver；06:30 `NVDA 关键事件心跳提醒` 写 `本轮监测结论：Noop` 仍进入 deliver；08:00 `AI与科技持仓观察关键事件心跳提醒` 写 `本轮检查状态：NOOP — 无新增触发事件` 仍进入 deliver；10:00 `光模块板块关键事件心跳提醒` 写 `状态：noop — 本轮无全新重大事件触发` 仍进入 deliver。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-04 06:02 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-04 02:00-06:02 CST。
    - 近窗继续有 `NOOP/noop/无触发/无新增` 等静默语义或跳过语义信号：`noop` 18 条、`NOOP` 5 条、`无触发` 2 条、`无新增` 5 条。
    - 代表样本：02:00 `中际旭创关键事件心跳提醒` 写 `本轮无高权重触发（noop）` 仍进入 deliver；02:00 `AI与科技持仓观察关键事件心跳提醒` 写 `本轮检查状态：NOOP — 无新增触发事件` 仍进入 deliver；04:00 `持仓财报与重大新闻心跳提醒` 写 `状态：noop — 无全新持仓级别的触发事件` 仍进入 deliver；06:00 `存储板块关键事件心跳提醒` 写 `状态：noop — 本轮无全新重大事件触发` 仍进入 deliver。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-04 02:02 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 22:01-2026-08-04 02:01 CST。
    - 近窗继续有 53 条 `NOOP/noop/无触发/无新增/无全新/未命中` 等静默语义或跳过语义信号。
    - 代表样本：23:30 `AAPL + NVDA + BE 关键事件提醒` 写“过去 24 小时内未见触发事件，持仓和关注标的无新增关键变化”仍进入 deliver；00:30 `持仓财报与重大新闻心跳提醒` 写 `状态：noop — 本轮无全新持仓级别触发事件` 仍进入 deliver；01:30 `光模块板块关键事件心跳提醒` 写 `状态：noop — 本轮无全新重大事件触发` 仍进入 deliver。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 22:02 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 18:02-22:02 CST。
    - 近窗继续有 93 条 `NOOP/noop/无触发/无新增/未命中，跳过发送` 等静默语义或跳过语义信号。
    - 代表样本：18:00 `持仓财报与重大新闻心跳提醒` 写 `状态：noop` 仍进入 deliver；19:30 `持仓财报与重大新闻心跳提醒` 写 `状态：noop — 本轮无全新持仓级别触发事件` 仍进入 deliver；20:00 / 21:30 `持仓财报与重大新闻心跳提醒` 继续把无全新触发或未独立核验报价的正文送入 deliver。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 18:03 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 14:03-18:02 CST。
    - 近窗继续有 `NOOP/noop/无触发/无新增/无高权重新事件` 等静默语义进入 `deliver_preview`。
    - 代表样本：15:00 `闪迪关键事件心跳提醒` 写 `本轮无触发（noop）` 仍进入 deliver；15:00 / 16:00 `持仓财报与重大新闻心跳提醒` 写 `状态：noop` 仍进入 deliver；17:00 `NBIS关键事件心跳提醒` 写“无高权重新事件触发（noop）”仍进入 deliver；17:30 `AAPL + NVDA + BE 关键事件提醒` 写“心跳检查结论：无触发”仍进入 deliver。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 14:03 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 10:01-14:01 CST。
    - 近窗有 20 条 `NOOP/noop/无全新/无新增/无高权重新事件` 等静默语义进入 `deliver_preview`。
    - 代表样本：10:00 `持仓财报与重大新闻心跳提醒` 写 `状态：noop`；10:30 同 job 写 `状态：noop` 且称数据工具调用已达配额；11:00 / 12:00 / 13:00 `NBIS关键事件心跳提醒` 写“无新增 / 无新事件触发（noop）”仍进入 deliver；12:00 `持仓重大事件心跳提醒` 写 `本轮心跳检查结论：noop`。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 10:03 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 06:02-10:03 CST。
    - 近窗仍有 `NOOP/noop/无全新/无触发` 等静默语义进入 `deliver_preview` 或 duplicate suppression 候选。
    - 代表样本：06:00 `持仓财报与重大新闻心跳提醒`、`光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒` deliver preview 明写 `状态：noop` 或“本轮无全新持仓级别触发事件”；09:30 `持仓重大事件心跳提醒` deliver preview 明写 `本轮心跳检查结论：noop`；10:00 `持仓财报与重大新闻心跳提醒` 写 `状态：noop`，`AI与科技持仓观察关键事件心跳提醒` 写 `30 分钟心跳检查：NOOP`。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 06:02 CST 运行态复核：问题继续在 live source heartbeat / scheduler 出站候选中复发，状态维持 `New / P2`。
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：2026-08-03 02:02-06:02 CST。
    - 近窗仍有 `NOOP/noop/无全新/无触发` 等静默语义进入 `deliver_preview`。
    - 代表样本：02:30 `AI与科技持仓观察关键事件心跳提醒` deliver preview 明写 `30 分钟心跳检查：NOOP`；03:30 `持仓财报与重大新闻心跳提醒` / `存储板块关键事件心跳提醒` 明写 `状态：noop`；04:00 / 05:00 / 06:00 多条 `持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒`、`NBIS关键事件心跳提醒` 仍把无新增或 noop 结果送入 deliver / duplicate suppression。
  - 判断：
    - 该样本仍属于未触发静默条件时继续生成并进入投递候选的同根缺陷。
    - 严重等级维持 P2：它造成监控噪音和错误送达语义，但本轮未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用，因此不是 P1。

- 2026-08-03 02:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/logs/hone-console-page-source.log`
    - 22:01-02:03 CST 近窗统计 `deliver=62`，其中 27 条 deliver preview 明写 `noop`、`NOOP`、`无新增` 或 `无触发` 仍进入 deliver 候选。
    - 01:00 / 01:30 / 02:00 `持仓财报与重大新闻心跳提醒`、`光模块板块关键事件心跳提醒`、`AI与科技持仓观察关键事件心跳提醒` 等 preview 明写 `状态：noop`、`30 分钟心跳检查：NOOP` 或 `本轮无全新持仓级别触发事件`，仍被归类为 `PlainTextTriggered` 并进入 deliver。
    - 02:00 `持仓重大事件心跳提醒` deliver preview 写“本轮心跳检查结论：noop / 本次 30 分钟窗口无值得推送的持仓更新”，随后被 duplicate suppression 压制；同轮 `持仓财报与重大新闻` 和 `光模块板块` 仍完成定时任务。
  - 判断：
    - 最新样本继续来自 heartbeat=1 语义链路：模型 / preview 明确无新增事实或 noop，但出站层仍生成完整正文或送达候选。
    - 严重等级维持 `P2`：该问题会导致监控任务错误投递噪音报告，影响提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露或 P1 级链路故障。

- 2026-08-02 21:00-22:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/logs/hone-console-page-source.log`
    - 21:01 `中际旭创关键事件心跳提醒` deliver preview 明写“本轮检查结果：无高权重新事件触发（noop）”，仍进入 `PlainTextTriggered` deliver 候选，随后被 duplicate suppression 压制。
    - 21:30 `NBIS关键事件心跳提醒` deliver preview 写“本轮检查结果：无高权重新事件触发（noop）”，仍进入 `PlainTextTriggered` deliver。
    - 21:30 `光模块板块关键事件心跳提醒` 与 22:00 同 job 明写 `状态：noop / 本轮无全新触发事件`，仍进入 deliver 候选。
    - 22:00 `持仓财报与重大新闻心跳提醒` 与 `存储板块关键事件心跳提醒` deliver preview 明写 `状态：noop`，仍进入 `PlainTextTriggered` deliver。
  - 判断：
    - 最新样本继续来自 heartbeat=1 语义链路：模型 / preview 明确无新增事实或 noop，但出站层仍生成完整正文或送达候选。
    - 严重等级维持 `P2`：该问题会导致监控任务错误投递噪音报告，影响提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露或 P1 级链路故障。

- 2026-08-01 14:00-18:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs` / `data/runtime/logs/web.log.2026-08-01`
    - SQLite 近窗 heartbeat run 共 9 条：`noop/skipped_noop=5`、`completed/sent=4`；普通 scheduler 无新增 run。
    - 14:00 `TEM大事件心跳监控` `run_id=51428` deliver preview 明写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - 14:00 `AAOI 全面心跳检测` `run_id=51431` 写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - 14:00 `RKLB 全面心跳检测` `run_id=51424` 写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - 14:00 `ASTS 全面心跳检测` `run_id=51425` 写 `无新突破性事实，noop`，仍 `completed/sent/delivered=1`。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / deliver preview 已明确无变化、无新增触发事实或 noop，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露或新的 P1 级投递错误。

- 2026-08-01 10:00-14:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs` / `data/runtime/logs/web.log.2026-08-01`
    - SQLite 近窗 heartbeat run 共 45 条：`noop/skipped_noop=30`、`completed/sent=13`、`execution_failed/skipped_error=2`；普通 scheduler 3 条均 `completed/sent`。
    - 10:30 `TEM大事件心跳监控` `run_id=51365` 写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - 10:30 `德业股份加仓信号心跳检测` `run_id=51368` 写 `结论：NOOP` 且数据源尚未更新，仍送达完整检查表。
    - 10:30 `ASTS 全面心跳检测` `run_id=51361`、10:30 `AAOI 全面心跳检测` `run_id=51364`、11:30 `ASTS 全面心跳检测` `run_id=51381`、12:00 `RKLB 全面心跳检测` `run_id=51396` 均明写 `noop/无新增触发事实`，仍 `completed/sent/delivered=1`。
    - runtime 14:00 `TEM / AAOI / RKLB / ASTS` heartbeat deliver preview 继续写 `noop/无新增触发事实` 后发送。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-08-01 06:00-10:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - SQLite 近窗 heartbeat run 共 72 条：`completed/sent=8`、`noop/skipped_noop=18`、`execution_failed/skipped_error=46`。
    - 09:30 `珠海冠宇加仓信号心跳检测` `run_id=51339` 明写 `结论：NOOP`，且说明“四条件无变化，维持观察”，仍 `completed/sent/delivered=1`。
    - 09:30 `ASTS 全面心跳检测` `run_id=51344` 写 `无新突破性事实，noop`，仍 `completed/sent/delivered=1`。
    - 10:00 `珠海冠宇加仓信号心跳检测` `run_id=51354` 写 `结论：NOOP` 与“不推送加仓结论”，仍送达完整检查表。
    - 10:00 `AAOI 全面心跳检测` `run_id=51353` 写 `本轮无新增触发事实，noop`，仍进入完整报告投递。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-31 10:00-14:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - SQLite 近窗 heartbeat run 共 84 条：`completed/sent=23`、`noop/skipped_noop=50`、`execution_failed/skipped_error=11`。
    - 11:30 `德业股份加仓信号心跳检测` `run_id=50908` 明写 `结论：NOOP` 且“四条件未同时成立，维持观察，不推送加仓结论”，仍 `completed/sent/delivered=1`。
    - 11:30 `RKLB 全面心跳检测` `run_id=50907`、12:00 `AAOI 全面心跳检测` `run_id=50918`、12:30 / 13:00 `RKLB 全面心跳检测` `run_id=50926/50931`、13:00 `ASTS 全面心跳检测` `run_id=50935` 均明写 `noop/无新增触发事实`，仍送达完整检查表。
    - 12:00 / 13:00 / 13:30 `珠海冠宇加仓信号心跳检测` `run_id=50919/50938/50948` 继续写 `NOOP` 与“不推送加仓结论”，但仍发送完整报告。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-31 06:00-10:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - SQLite 近窗 heartbeat run 共 90 条：`noop/skipped_noop=62`、`completed/sent=17`、`execution_failed/skipped_error=10`、`running/pending=1`。
    - 06:30 `德业股份加仓信号心跳检测` `run_id=50794` 写 `结论：NOOP` 且“四条件未同时成立，维持观察，不推送加仓结论”，仍 `completed/sent/delivered=1`。
    - 06:30 `AAOI 全面心跳检测` `run_id=50796`、07:00 `RKLB 全面心跳检测` `run_id=50797`、07:30 `德业股份加仓信号心跳检测` `run_id=50809` 与 `RKLB 全面心跳检测` `run_id=50816` 均明写 `noop/NOOP/无新增触发事实/不推送`，仍送达或进入完整报告形态。
    - 08:30 `ASTS 全面心跳检测` `run_id=50842`、09:30 `珠海冠宇加仓信号心跳检测` `run_id=50862` 继续写 `noop/NOOP` 后送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-31 02:02-06:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - SQLite 近窗 heartbeat run 共 80 条：`completed/sent=21`、`noop/skipped_noop=54`、`execution_failed/skipped_error=5`。
    - 03:01 `TSLA 正负触发条件心跳监控` `run_id=50717` 明写 `本轮 TSLA 监控检查结论：noop`，仍 `completed/sent/delivered=1`。
    - 03:30 `珠海冠宇加仓信号心跳检测` `run_id=50730` 写 `结论：NOOP` 且“四条件未同时成立，维持观察，不推送加仓结论”，仍送达完整报告。
    - 04:30 `德业股份加仓信号心跳检测` `run_id=50752` 写 `结论：NOOP` 且“不推送加仓结论”，仍 `completed/sent/delivered=1`。
    - 04:31 `RKLB 全面心跳检测` `run_id=50748`、05:00 `TSLA 正负触发条件心跳监控` `run_id=50755`、05:30 `德业股份加仓信号心跳检测` `run_id=50768` 与 `RKLB 全面心跳检测` `run_id=50774` 均明写 `noop/NOOP/无新增触发事实/不推送`，仍送达或进入完整报告形态。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 22:01-2026-07-31 02:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - SQLite 近窗 heartbeat run 共 42 条：`completed/sent=9`、`noop/skipped_noop=30`、`execution_failed/skipped_error=3`。
    - 23:30 `德业股份加仓信号心跳检测` `run_id=50645` 写 `结论：NOOP`，仍 `completed/sent/delivered=1`。
    - 00:00 `ASTS 全面心跳检测` `run_id=50660` 写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
  - `data/runtime/logs/web.log.2026-07-30`
    - 01:30 `德业股份加仓信号心跳检测`、`珠海冠宇加仓信号心跳检测`、`RKLB 全面心跳检测`、`ASTS 全面心跳检测` 均在 deliver preview 中明写 `NOOP/noop/不推送/无新增触发事实`，仍进入 sent / deliver preview。
    - 02:00 `德业股份加仓信号心跳检测`、`AAOI 全面心跳检测`、`珠海冠宇加仓信号心跳检测` 继续写 `NOOP/noop/不推送加仓结论` 后送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 18:00-22:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 90 条：`completed/sent=25`、`noop/skipped_noop=59`、`execution_failed/skipped_error=6`；其中 11 条 `delivered=1` 的 heartbeat preview 明写 `NOOP`、`noop`、`不推送`、`无新增触发事实` 或“本轮无新价格变化”。
    - 18:00 `TEM大事件心跳监控` 写 `本轮无新增触发事实，noop`，18:00 / 19:00 / 19:30 `德业股份加仓信号心跳检测` 多次写 `结论：NOOP` 且四条件未同时成立，仍 `completed/sent/delivered=1`。
    - 19:30 `RKLB 全面心跳检测` 写同一 7/29 收盘价已连续推送十余轮、本轮无新价格变化，仍发送完整正文。
    - 21:30 `TEM大事件心跳监控`、21:30 / 22:00 `珠海冠宇加仓信号心跳检测` 继续写 `noop` / `NOOP` / `不推送加仓结论` 后送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不重复触发，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 14:01-18:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 80 条：`completed/sent=20`、`noop/skipped_noop=54`、`execution_failed/skipped_error=6`；其中 12 条 `delivered=1` 的 heartbeat preview 明写 `NOOP`、`noop`、`无新增触发事实`、`不推送` 或“四条件未同时成立”。
    - 14:30 `德业股份加仓信号心跳检测` `run_id=50437` 写 `结论：NOOP` 且“四条件未同时成立，维持观察，不推送加仓结论”，仍 `completed/sent/delivered=1`。
    - 15:00 `珠海冠宇加仓信号心跳检测` `run_id=50448` 写 `结论：NOOP`，15:00 `ASTS 全面心跳检测` `run_id=50444` 与 `RKLB 全面心跳检测` `run_id=50447` 写 `本轮无新增触发事实，noop`，均仍送达。
    - 17:30 `RKLB 全面心跳检测` `run_id=50495` 写同一价格已连续推送、本轮无新价格变化；18:00 `TEM大事件心跳监控` `run_id=50509` 写 `本轮无新增触发事实，noop`，18:00 `德业股份加仓信号心跳检测` `run_id=50508` 写 `结论：NOOP` 且 `不推送加仓结论`，均仍送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不重复触发，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 10:01-14:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 83 条：`completed/sent=27`、`noop/skipped_noop=49`、`execution_failed/skipped_error=4`；其中 20+ 条 `delivered=1` 的 heartbeat preview 明写 `noop`、`NOOP`、`无新增触发事实` 或 `无新增触发`。
    - `RKLB 全面心跳检测` 在 10:30 `run_id=50352`、11:00 `run_id=50362`、14:00 `run_id=50429` 均写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - `ASTS 全面心跳检测` 在 11:00 `run_id=50360`、11:30 `run_id=50372`、12:30 `run_id=50397`、13:30 `run_id=50417`、14:00 `run_id=50428` 均写 `无新增触发事实 / noop`，仍送达。
    - `TEM大事件心跳监控` 12:00 `run_id=50386` 写 `本轮无新增触发事实，noop`，仍送达。
    - `德业股份加仓信号心跳检测` 与 `珠海冠宇加仓信号心跳检测` 在 10:30-13:30 多轮写 `结论：NOOP` 且四条件未同时成立，仍送达完整报告。
    - 14:01 `AAOI 全面心跳检测` `run_id=50425` 写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不重复触发，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 06:02-10:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 80 条：`completed/sent=21`、`noop/skipped_noop=58`、`execution_failed/skipped_error=1`；其中 10 条 `delivered=1` 的 heartbeat preview 明写 `noop`、`NOOP`、`无新增触发事实`、`无新触发事实` 或 `无高权重增量事实`。
    - `TEM大事件心跳监控` 在 06:30 `run_id=50258`、09:00 `run_id=50326`、09:30 `run_id=50337` 均写 `本轮无新增触发事实，noop`，仍 `completed/sent/delivered=1`。
    - `RKLB 全面心跳检测` 在 06:30 `run_id=50255` 与 07:30 `run_id=50278` 写 `本轮无高权重增量事实 / 无新增触发事实，noop`，仍送达。
    - `ASTS 全面心跳检测` 在 06:30 `run_id=50263` 与 10:00 `run_id=50345` 写 `本轮检查结论：noop / 本轮无新增触发事实，noop`，仍送达。
    - `德业股份加仓信号心跳检测` 在 06:30 `run_id=50260` 与 `珠海冠宇加仓信号心跳检测` 在 07:00 `run_id=50274`、07:30 `run_id=50283` 写 `结论：NOOP` 且四条件未同时成立，仍送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不重复触发，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-30 02:01-06:04 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 80 条：`completed/sent=20`、`noop/skipped_noop=58`、`execution_failed/skipped_error=2`；其中 9 条 `delivered=1` 的 heartbeat preview 明写 `noop`、`NOOP`、`不推送`、`无新触发事实`、`无高权重增量事实` 或 `无新增触发事实`。
    - `TEM大事件心跳监控` 在 02:30 `run_id=50177` 写 `本轮无触发，不推送`，仍 `completed/sent/delivered=1`；06:00 `run_id=50251` 写 `本轮无新增触发事实，noop`，仍送达。
    - `RKLB 全面心跳检测` 在 02:30 `run_id=50176` 写 `本轮检查结论：noop`，03:00 `run_id=50189` 与 05:30 `run_id=50239` 写 `本轮无高权重增量事实，不推送`，均仍送达。
    - `ASTS 全面心跳检测` 在 02:30 `run_id=50172` 与 03:30 `run_id=50193` 写 `无新触发事实 / 无高权重增量事实，不推送`，仍 `completed/sent/delivered=1`。
    - `德业股份加仓信号心跳检测` 在 03:00 `run_id=50183` 写 `结论：NOOP` 且四条件未同时成立，仍送达；`TSLA 正负触发条件心跳监控` 在 06:00 `run_id=50244` 写 `触发判断：noop`，仍送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或不推送，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-29 14:01-18:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 80 条：`completed/sent=23`、`noop/skipped_noop=54`、`execution_failed/skipped_error=3`。
    - `TSLA 正负触发条件心跳监控` 在 16:30 `run_id=49945` 写出 `触发判断：noop`，仍落成 `completed/sent/delivered=1`。
    - `德业股份加仓信号心跳检测` 在 17:00 `run_id=49948` 写 `结论：NOOP` 且四条件未同时成立，仍 `completed/sent/delivered=1`。
    - `珠海冠宇加仓信号心跳检测` 在 17:00 `run_id=49955` 写 `结论：NOOP` 且维持观察，仍 `completed/sent/delivered=1`。
  - 判断：
    - 这次证据继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无触发或维持观察，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-29 06:01-10:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - `AAOI 全面心跳检测` 在 07:30 写出“本轮检查结论：noop”，仍落成 `heartbeat=1`、`completed/sent/delivered=1`。
    - `珠海冠宇加仓信号心跳检测` 在 09:00 写出“结论：NOOP”，仍 `completed/sent/delivered=1`。
    - `TEM大事件心跳监控` 在 09:30 标题写 `TEM 30分钟心跳检查（09:30）— noop`，仍 `completed/sent/delivered=1`。
  - 判断：
    - 这次证据继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-29 02:00-06:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 85 条：`completed/sent=18`、`execution_failed/skipped_error=7`、`noop/skipped_noop=60`；其中至少 11 条 `delivered=1` 的 heartbeat preview 明写 `noop`、`无新增触发`、`无新触发`、`无触发` 或“保持静默”。
    - 02:00 CST `TSLA 正负触发条件心跳监控` `run_id=49634` 写 `触发判断：noop`，仍 `completed/sent/delivered=1`。
    - 03:00 CST `全天原油价格3小时播报` `run_id=49655` 写“本轮无法完成原油价格播报，保持静默”，仍 `completed/sent/delivered=1`。
    - 03:00 / 03:30 CST `AAOI 全面心跳检测` `run_id=49649/49662` 标题或结论含 `noop`，仍送达。
    - 04:30 CST `TEM大事件心跳监控` `run_id=49679`、`TSLA 正负触发条件心跳监控` `run_id=49678` 继续写 `noop` / 无触发后送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1 路径，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化、无触发或保持静默，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；同窗无错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-28 18:01-22:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 80 条：`completed/sent=20`、`noop/skipped_noop=55`、`execution_failed/skipped_error=5`。
    - 19:00 CST `德业股份加仓信号心跳检测` `run_id=49459` 明确写 `结论：NOOP——大跌 -5.69% 但量能未显著放大`，仍 `completed/sent/delivered=1`。
    - 21:30 CST 同 job `run_id=49534` 再次写 `NOOP`，仍送达；22:00 `run_id=49547` 写 `结论：NOOP——今日续跌 -5.69%...无新催化`，仍 `completed/sent/delivered=1`。
    - 22:00 CST `珠海冠宇加仓信号心跳检测` `run_id=49549` 写 `结论：NOOP——价格小幅收跌（-0.14%），量能持续萎缩（均量 40%），无新催化`，仍 `completed/sent/delivered=1`。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-28 14:01-18:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat run 共 81 条：`completed/sent=22`、`noop/skipped_noop=47`、`execution_failed/skipped_error=12`。
    - 15:00 CST `RKLB 全面心跳检测` `run_id=49382` 标题写 `RKLB 30分钟心跳检查（15:00）— noop`，仍 `completed/sent/delivered=1`。
    - 15:30 CST `德业股份加仓信号心跳检测` `run_id=49387` 写 `结论：NOOP——大跌 -5.69% 但量能未显著放大`，仍送达。
    - 15:30 CST `TSLA 正负触发条件心跳监控` `run_id=49388` 写 `触发判断：noop`，仍送达。
    - 17:30 CST `RKLB 全面心跳检测` `run_id=49431` 与 18:00 CST `ASTS 全面心跳检测` `run_id=49442` 均在标题写 `noop`，仍 `completed/sent/delivered=1`。
    - 18:00 CST `珠海冠宇加仓信号心跳检测` `run_id=49436` 写 `结论：NOOP——价格基本收平，无新催化，量能仍处低位`，仍送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-28 10:01-14:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 10:30 CST `珠海冠宇加仓信号心跳检测` `run_id=49289` 的 preview 明确写 `结论：NOOP——缩量反弹延续，未见放量止跌或新催化`，但 cron 仍记录 `heartbeat=1`、`completed/sent/delivered=1`。
    - 11:00 / 11:30 CST `RKLB 全面心跳检测` `run_id=49300/49307` 分别写 `noop`、`无新增触发事实`，仍为 `completed/sent/delivered=1`。
    - 11:30 CST `ASTS 全面心跳检测` `run_id=49310` 写 `ASTS 30分钟心跳检查（11:30）— noop` 与 `无新增触发事实`，仍发送完整报告。
    - 13:00 / 13:30 CST `TEM大事件心跳监控` `run_id=49336/49344` 写 `TEM 30分钟心跳检查 — noop`，仍 `completed/sent/delivered=1`。
    - 14:00 CST `ASTS 全面心跳检测` `run_id=49356` 写 `ASTS 30分钟心跳检查（14:00）— noop`；14:00 `TSLA 正负触发条件心跳监控` `run_id=49359` 写 `触发判断：noop`，两条均送达。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-27 15:03-19:04 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 16:00 CST `珠海冠宇加仓信号心跳检测` `run_id=48862` 的 preview 明确写 `结论：NOOP`，但 cron 仍记录 `heartbeat=1`、`completed/sent/delivered=1`。
    - 16:30 / 17:00 / 18:00 CST `RKLB 全面心跳检测` `run_id=48870/48886/48907` 均写 `noop`、`本轮无新增触发` 或“沿用 7/24 收盘，与今日各轮完全一致”，但仍发送完整报告。
    - 18:00 CST `德业股份加仓信号心跳检测` `run_id=48908` 写 `结论：NOOP——缩量续跌，无明确加仓信号`，仍落成 `completed/sent/delivered=1`。
    - 19:00 CST `TSLA 正负触发条件心跳监控` `run_id=48928` 写 `触发判断：noop`、无独立新行情时间戳或新事件节点，仍发送给用户。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-27 11:01-15:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 12:00 CST `AAOI 全面心跳检测` `run_id=48773` 的 preview 标题明确写 `本轮 AAOI 心跳检查 — noop`，正文也写“本轮新增可核验事实：无”，但 cron 仍记录 `heartbeat=1`、`completed/sent/delivered=1`。
    - 12:01 CST `RKLB 全面心跳检测` `run_id=48778` 写 `本轮无新增触发`，但仍为 `completed/sent/delivered=1`。
    - 14:30 / 15:00 CST `RKLB 全面心跳检测` `run_id=48835/48842` 均写 `本轮无新增触发` 或 `无进一步恶化`，仍发送完整报告。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-27 07:02-11:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 07:30 CST heartbeat job `RKLB 全面心跳检测` 的 `response_preview` 写“本轮无新触发 / 无新增可核验价格变化或基本面增量”，但 `run_id=48659` 仍记录 `heartbeat=1`、`completed/sent/delivered=1`。
    - 08:00 CST `珠海冠宇加仓信号心跳检测` 写 `结论：NOOP——休市无新报价，无新催化，止跌信号仍不成立`，但 `run_id=48673` 仍为 `completed/sent/delivered=1`。
    - 08:00 / 09:00 / 09:30 / 11:00 CST `AAOI / RKLB` heartbeat 多次在 preview 中写 `noop`、无变化或无新增触发，仍进入 sent / delivered 或先进入 deliver preview 再 duplicate suppression。
  - 判断：
    - 本轮样本继续来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP`、无变化或无触发增量，出站层仍将正文送达或进入送达候选。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-27 03:01-07:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 05:00 CST heartbeat job `德业股份加仓信号心跳检测` 的 `response_preview` 明确写 `结论：NOOP——休市无新报价，上轮数据无更新`，但 cron 仍记录 `heartbeat=1`、`execution_status=completed`、`message_send_status=sent`、`delivered=1`。
    - 05:30 CST 同一 `德业股份加仓信号心跳检测` 再次写 `结论：NOOP——休市无新报价，上轮数据无更新，上轮已判定不满足加仓条件`，仍落成 `completed/sent/delivered=1`。
    - 07:00 CST `RKLB 全面心跳检测` `response_preview` 写 `本轮触发评估：noop` 且行情快照无变化，仍落成 `completed/sent/delivered=1`。
    - 07:00 CST `珠海冠宇加仓信号心跳检测` 写 `结论：NOOP——无量续跌，担保公告为常规融资，非基本面容量变化`，仍落成 `completed/sent/delivered=1`。
  - 判断：
    - 本轮样本来自 heartbeat=1，但坏语义与本单相同：模型 / preview 已明确 `NOOP` 或无新报价、无触发增量，出站层仍向用户发送完整正文。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

- 2026-07-26 23:02-2026-07-27 03:02 CST 真实运行态复发，状态从代码级 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3`
    - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
    - `ordinal=10` / `timestamp=2026-07-27T00:00:00.622286+08:00`：Feishu scheduler 任务 `RKLB 每日动态监控` 明确要求“发现实质性催化或风险证伪信号时，第一时间推送简报；若当日无重要更新，可跳过不推送”。
    - `ordinal=11` / `timestamp=2026-07-27T00:00:57.028818+08:00`：assistant final 自行判断“今日无新增实质变化，跳过主动推送”，但仍输出完整 `RKLB 每日动态监控简报`。
    - `ordinal=12-15`：`AAOI 每日动态监控` 与 `TEM 每日动态监控` 同样包含“若当日无重要更新，可跳过不推送”，assistant final 分别写“今日跳过主动推送”，但仍发送完整长报告。
  - `cron_job_runs`
    - `run_id=48486` (`RKLB 每日动态监控`)、`run_id=48488` (`AAOI 每日动态监控`)、`run_id=48497` (`TEM 每日动态监控`) 均记录 `heartbeat=0`、`execution_status=completed`、`message_send_status=sent`、`delivered=1`。
  - 判断：
    - 这是普通 scheduler 的静默 / no-op 语义在 live 链路中的同根复发；模型已明确判断应跳过主动推送，但出站层仍按完成发送处理。
    - 严重等级维持 `P2`：问题会导致监控任务错误投递噪音报告，影响功能语义和用户决策提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-20 23:02-2026-07-21 03:02 CST。
  - 窗口内按真实 `timestamp` 新增 6 条 user / 6 条 assistant，覆盖 3 个 session，均以 assistant 收口。
  - `session_id=Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44`
    - `ordinal=18` / `timestamp=2026-07-20T23:30:01.398154+08:00`：Feishu scheduler 任务 `科技成长股持仓买卖点日内预警` 明确要求校验 BE / RKLB / TEM / MSFT 的触发位，并写明“若未触发，则保持静默”。
    - `ordinal=19` / `timestamp=2026-07-20T23:30:26.018222+08:00`：assistant final 仍生成完整持仓报告，并在正文中自行判断 `TEM — $40 未破，静默`、`RKLB — $60 未破，静默`、`MSFT — $380 未破，静默`、`无纪律触发，全部静默`。
  - 同窗 `cron_job_runs` 无新增，`max(executed_at)` 仍停在 `2026-07-19T13:31:15.040172+08:00`；本条用户可见证据以 `session_messages` 为准。

## 端到端链路

1. Feishu 普通 scheduler 触发 `科技成长股持仓买卖点日内预警`。
2. 用户任务正文定义一组价格 / 技术条件，并要求未触发时静默。
3. assistant 执行行情与持仓判断。
4. assistant 在 final 中确认没有纪律触发，但仍把完整报告写入会话。
5. 用户收到一条本应静默的报告。

## 期望效果

当普通 scheduler 任务明确要求“若未触发，则保持静默”且模型判断没有触发条件时，链路应落成不投递或 no-op；最多只在内部台账记录本轮检查结果，不应向用户发送完整正文。

## 当前实现效果

截至 2026-07-21 的代码修复前，模型能识别未触发条件，但输出层没有把“全部静默 / 未触发”转成跳过发送，仍把完整分析正文作为 final 落库并面向用户可见。

## 用户影响

- 用户会收到本应静默的噪音提醒，削弱价格预警任务的可信度。
- 高频交易日任务可能反复推送“未触发”长报告，用户难以区分真正触发的买卖点提醒。
- 这是功能性缺陷：静默 / no-op 是该类任务的核心交付语义，不只是文字质量问题。

## 根因判断

当前证据指向普通 scheduler 的 skip-delivery 判定没有覆盖“模型 final 已确认未触发但仍生成正文”的场景。已有 heartbeat 结构化状态退化文档覆盖的是 `heartbeat=1` 的 JSON / noop 协议漂移；本次样本来自 `heartbeat=0` 普通 Feishu scheduler，链路和受影响范围不同，因此独立登记。

严重等级定为 P2：问题会导致监控任务错误投递噪音报告，影响功能语义和用户决策提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

## 下一步建议

1. 在普通 scheduler 出站前增加 skip-delivery 判定，识别“未触发 / 保持静默 / 全部静默 / 今日跳过推送”等明确 no-op 语义。
2. 区分用户要求的“静默不推送”和普通报告任务的“无重大更新但仍需简报”，避免误杀日常摘要。
3. 为 Feishu 普通 scheduler 增加回归：当任务正文包含“若未触发则保持静默”且 final 判断“全部静默”时，应记录 no-op 或 skipped，不发送用户可见正文。

## 修复记录

- 2026-07-21：普通 scheduler 出站链路已补“静默 no-op”判定。
  - 代码位置：`crates/hone-channels/src/scheduler.rs`
  - 修复内容：当任务正文明确要求“若未触发则保持静默/静默不推送”时，若 final 同时表达“未触发/未破/无纪律触发”与“静默/不推送”，出站层会回滚本轮 assistant 持久化并按 `should_deliver=false` 收口，不再向用户发送完整报告。
  - 回归覆盖：新增正反两条单元测试，覆盖“静默任务 + 全部静默”命中 skip，以及普通复盘任务不被误判为 skip。
  - 验证：`cargo test -p hone-channels silent_noop_signal_ --lib -- --nocapture`、`cargo test -p hone-channels skip_delivery_signal_detected --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 说明：本轮未重启当前 Feishu / scheduler live 服务，因此状态先记为代码级 `Fixed`；若后续 2026-07-21 之后的真实运行窗仍出现同类“全部静默但照样投递”样本，再按新证据重新打开。

## 最新运行态复核（2026-07-28 10:02 CST）

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 巡检窗口：2026-07-28 06:02-10:02 CST。
  - 同窗 heartbeat run 共 80 条：`completed/sent=23`、`noop/skipped_noop=50`、`execution_failed/skipped_error=7`。
  - `run_id=49188`，`TEM大事件心跳监控`，`executed_at=2026-07-28T06:30:11.228527+08:00`，终态 `completed/sent/delivered=1`，用户可见 preview 标题含 `TEM 30分钟心跳检查（06:30）— noop`，正文仍作为消息送达。
  - `TSLA 正负触发条件心跳监控` 在 `06:30`、`07:00`、`08:00`、`08:30` 多次 `completed/sent/delivered=1`，preview 写出 `触发判断：noop` 或等价未触发判断后仍送达。
- 本轮判断
  - 这次证据来自 heartbeat=1 路径，但用户需求语义同样是未触发时静默；现象与“NOOP / 未触发仍发送报告”同根，不新建重复文档。
  - 影响是用户收到本应静默的噪音提醒，功能语义仍受损；同窗无错投、数据破坏、敏感信息泄露或全渠道不可用，维持 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-29 02:01 CST）

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 巡检窗口：2026-07-28 22:00-2026-07-29 02:01 CST。
  - `run_id=49549` / `珠海冠宇加仓信号心跳检测` 在 22:00 明写 `结论：NOOP` 且无新催化，仍落成 `completed/sent/delivered=1`。
  - `run_id=49547` / `德业股份加仓信号心跳检测` 在 22:00 明写 `结论：NOOP`，仍落成 `completed/sent/delivered=1`。
  - `run_id=49558` / `TEM大事件心跳监控` 在 22:30 标题含 `noop`，仍落成 `completed/sent/delivered=1`。
  - `run_id=49620` / `ASTS 全面心跳检测` 在 01:30 标题含 `noop`，仍落成 `completed/sent/delivered=1`。
- 判断：
  - 这次证据来自 heartbeat=1 路径，但用户需求语义同样是未触发时静默；现象与“NOOP / 未触发仍发送报告”同根，不新建重复文档。
  - 影响是用户收到本应静默的噪音提醒，功能语义仍受损；同窗无错投、数据破坏、敏感信息泄露或全渠道不可用，维持 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-29 14:02 CST）

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 巡检窗口：2026-07-29 10:01-14:02 CST。
  - `run_id=49840` / `德业股份加仓信号心跳检测` 在 11:00 明写 `结论：NOOP`，仍落成 `completed/sent/delivered=1`。
  - `run_id=49860` / 同一 job 在 11:31 明写 `结论：NOOP` 且“无新公开信息支撑”，仍落成 `completed/sent/delivered=1`。
  - `run_id=49920` / 同一 job 在 14:00 明写 `结论：NOOP`，仍落成 `completed/sent/delivered=1`。
- 本轮判断
  - 最新证据仍来自 heartbeat=1 路径，但用户需求语义同样是未触发时静默；现象与“NOOP / 未触发仍发送报告”同根，不新建重复文档。
  - 影响是用户收到本应静默的噪音提醒，功能语义仍受损；同窗无错投、数据破坏、敏感信息泄露或全渠道不可用，维持 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-29 22:03 CST）

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 巡检窗口：2026-07-29 18:01-22:03 CST。
  - `run_id=49980` / `珠海冠宇加仓信号心跳检测` 在 18:30 明写 `结论：NOOP`、无新行情变化、维持观察，仍落成 `completed/sent/delivered=1`。
  - `run_id=50031` / `德业股份加仓信号心跳检测` 在 21:30 明写 `结论：NOOP`、无新公告或硬催化，仍落成 `completed/sent/delivered=1`。
  - `run_id=50032` / `AAOI 全面心跳检测` 在 21:30 明写“本轮无新触发事实，不推送”，仍落成 `completed/sent/delivered=1`。
  - `run_id=50043` / `珠海冠宇加仓信号心跳检测`、`run_id=50045` / `ASTS 全面心跳检测`、`run_id=50046` / `RKLB 全面心跳检测` 在 22:00 明写 `NOOP/noop/不重复触发` 或全部条件未触发，仍送达。
- 判断：
  - 这次证据继续来自 heartbeat=1 路径，但用户需求语义同样是未触发时静默；现象与“NOOP / 未触发仍发送报告”同根，不新建重复文档。
  - 影响是用户收到本应静默的噪音提醒，功能语义仍受损；同窗无错投、数据破坏、敏感信息泄露或全渠道不可用，维持 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-30 02:03 CST）

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 巡检窗口：2026-07-29 22:01:29-2026-07-30 02:03 CST。
  - 同窗 heartbeat run 中有 15 条 `completed/sent/delivered=1` 的用户可见 preview 明写 `noop`、`NOOP`、`不推送` 或 `无新触发事实`。
  - 代表样本：`run_id=50085` / `TEM大事件心跳监控` 在 22:30 标题含 `本轮检查结论：noop`，仍送达。
  - `run_id=50088` / `珠海冠宇加仓信号心跳检测` 在 22:30 明写 `结论：NOOP` 且“四条件未同时成立”，仍送达。
  - `run_id=50143` / `AAOI 全面心跳检测` 在 01:00 明写“本轮无高权重增量事实，不推送”，仍落成 `completed/sent/delivered=1`。
  - `run_id=50158` / `ASTS 全面心跳检测` 与 `run_id=50159` / `RKLB 全面心跳检测` 在 01:30 明写“本轮无新触发事实 / 无高权重增量事实，不推送”，仍送达。
- 本轮判断
  - 这次证据继续来自 heartbeat=1 路径，但用户需求语义同样是未触发时静默；现象与“NOOP / 未触发仍发送报告”同根，不新建重复文档。
  - 影响是用户收到本应静默的噪音提醒，功能语义仍受损；同窗无错投、数据破坏、敏感信息泄露或全渠道不可用，维持 `P2 / New`，非 P1。
