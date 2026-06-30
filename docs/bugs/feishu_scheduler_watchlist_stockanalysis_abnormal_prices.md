# Bug: Feishu scheduler 核心观察池使用 StockAnalysis 异常价格作为行情锚

## 发现时间

- 2026-06-20 11:02 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 最新进展（2026-07-01 07:01 CST）

- 本轮 2026-07-01 03:01-07:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU / WDC 等异常数量级价格生成判断。
    - 07:00 CST Web `持仓财报与重大新闻心跳提醒` raw preview 仍写出 SNDK `$2,273.73`、日内高点 `$2,280.52`、前收 `$2,050.39` 等异常价格口径。
    - 同窗 `Monitor_Watchlist_11` raw preview 继续使用 MU `$1,154.29`，`存储板块关键事件心跳提醒` raw preview 继续出现 WDC `$638.72` 等异常数量级。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗没有新的真实 direct message timestamp；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-07-01 03:01 CST）

- 本轮 2026-06-30 23:00-2026-07-01 03:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:00-03:01 CST heartbeat raw preview / 判断上下文继续围绕 SNDK / MU 异常数量级价格生成判断。
    - 23:00 CST 后多条 `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 等仍可见 SNDK 约 `$2,162.54`、MU `$1,142+`、SNDK 52 周高点 `$2,354.39` 等异常价格口径进入 raw preview。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗 direct 会话表继续实时增量，但 `cron_job_runs` 仍停在 `2026-06-30T09:30:52.069168+08:00`；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 23:01 CST）

- 本轮 2026-06-30 19:02-23:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 23:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `核心观察股池晚间快报` 正常收口，assistant final 正式输出多只明显异常数量级价格：`MU $1,142.40`、`SNDK $2,171.23`、`STX $969.64`、`WDC $649.37`、`GEV $1,135.22`、`AMD $559.37` 等。
    - 同条 final 继续围绕这些价格给出击球区、财报日期和观察池纪律说明；主链路正常收口，没有空回复、错投、投递失败或原始工具 JSON。
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现 SNDK 异常价：`$2,050.39`、`previousClose $2,090.71`、`day range $1,895-$2,090.71`、`$2,162.54` 等，并进入触发判断上下文。
- 用户影响：
  - 本窗已有正式用户可见 scheduler final 样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径。
  - 报告主链路正常收口，未见功能阻断、错发对象或数据安全影响，因此仍按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 19:02 CST）

- 本轮 2026-06-30 15:02-19:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU 异常数量级价格生成判断。
    - 15:30-19:00 CST `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 等仍可见 SNDK 约 `$2,050.39`、日内高低区间 `$1,895-$2,090.71`、52 周高点 `$2,354.39` 等异常数量级价格进入 raw preview。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗 direct 会话表继续实时增量，但 `cron_job_runs` 仍停在 09:30；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 15:02 CST）

- 本轮 2026-06-30 11:02-15:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 13:30 CST `Monitor_Watchlist_11` raw preview 继续把 `MU $1145.28` 作为行情锚，并与 `HIMS $33.39`、`RKLB ~$86-98` 等 watchlist 条件一起进入判断上下文。
    - 15:00 前后 `闪迪关键事件心跳提醒` / `持仓财报与重大新闻心跳提醒` 等 heartbeat raw preview 仍可见 SNDK / MU 异常数量级价格信号。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗会话表已恢复实时增量，但 `cron_job_runs` 仍停在 09:30；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 07:03 CST）

- 本轮 2026-06-30 03:00-07:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 05:00 CST `闪迪关键事件心跳提醒` raw preview 继续把 `SNDK $2,050.39`、`previous close $2,090.71`、`yearHigh $2,354.39` 等异常数量级价格作为行情锚。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 03:07 CST）

- 本轮 2026-06-29 23:00-2026-06-30 03:07 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒`、`持仓关键事件心跳检测` 等 heartbeat raw preview 继续把 `SNDK $2,090.71`、`MU $1,132.33` 或同级异常数量级价格作为行情锚。
    - 23:00-03:07 CST 未确认新的正式送达成功样本；异常价格主要停留在 raw preview、结构化失败、未命中或 heartbeat 判断上下文。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 19:01 CST）

- 本轮 2026-06-29 15:00-19:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - `持仓财报与重大新闻心跳提醒`、`闪迪关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓关键事件心跳检测`、`Monitor_Watchlist_11` 等 heartbeat raw preview 继续把 `SNDK $2,090.71`、`MU $1,132.33` 作为行情锚。
    - 17:31 CST `闪迪关键事件心跳提醒` 返回 `JsonTriggered`，raw preview 继续围绕 `SNDK $2,090.71`、`-10.46%`、`52-week high $2,354.39` 等异常数量级价格生成触发判断；本窗未确认新的正式送达成功样本。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗主要停留在 heartbeat raw preview、未命中或结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告，因此不提升严重等级。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 15:07 CST）

- 本轮 2026-06-29 11:00-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-29`
    - 14:30 CST `闪迪关键事件心跳提醒`（`job_id=j_19dd9a1e`，`target=web-user-c2776780c59d`）以 `JsonTriggered` 生成正式 `deliver_preview`。
    - 同条预览写出 `最新价 $2,090.71，日内跌幅 -10.46%`、`日内区间 $2,063-$2,256`，继续把 SNDK 异常数量级价格作为用户可见行情锚。
    - 15:00 CST `持仓关键事件心跳检测` raw preview 又引用 `SNDK -10.46% ($2,090.71)`、`MU -6.69% ($1,132.33)` 等异常价格，但该样本落在 `JsonUnknownStatus + execution_failed` 路径。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 14:30 CST 样本已进入正式送达预览，不只是内部 raw preview，因此继续说明价格 sanity check 未覆盖当前 scheduler / heartbeat 运行路径。
  - 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 11:01 CST）

- 本轮 2026-06-29 07:00-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 10:30 CST `Monitor_Watchlist_11` raw preview 继续把 `MU $1,132.33` 作为价格锚，模型自己也写出 “MU at 1132? That seems wrong for Micron”，但该异常数值仍进入 heartbeat 判定上下文。
    - 本窗该样本落在 heartbeat raw preview / 结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/runtime/logs/acp-events.log`
    - 09:19 CST Web direct 用户可见 chunk 出现 `StockAnalysis SND...` / `StockAnalysis` 来源名片段；该样本只证明来源名净化仍漏网，异常价格主证据仍来自 runtime heartbeat raw preview。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 07:02 CST）

- 本轮 2026-06-29 03:04-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 07:00 CST Web heartbeat `持仓关键事件心跳检测` raw preview 继续把 `SNDK -10.46% ($2,090.71)` 作为行情锚，并围绕该异常数量级价格总结板块抛压。
    - 本窗该样本落在 heartbeat raw preview / 结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 03:01 CST）

- 本轮 2026-06-28 23:02-2026-06-29 03:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28`
    - 多条 heartbeat raw preview 继续把异常数量级价格作为行情锚：`Monitor_Watchlist_11` 多次写出 `MU $1,132.33` 或 `$1132.33`，`闪迪关键事件心跳提醒` / `存储板块关键事件心跳提醒` / `持仓关键事件心跳检测` 多次写出 `SNDK $2,090.71`，`AI与科技持仓观察关键事件心跳提醒` 继续写出 `STX $899.90`，`闪迪关键事件心跳提醒` 继续写出 `WDC $586.45`。
    - 本窗这些样本主要停留在 heartbeat raw preview、未命中或结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-28 11:01 CST）

- 本轮 2026-06-28 07:01-11:01 CST 真实运行态确认同根复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 11:00 CST Web heartbeat `持仓关键事件心跳检测`（`job_id=j_7a2adc11`，`target=web-user-cb1b46a2add4`）以 `JsonTriggered` 成功投递并记录 `定时任务完成`。
    - `deliver_preview` 写出 `SNDK -10.46%（$2,090.71）`、`VRT -6.64%（$303.95）`、`MU -6.69%（$1,132.33）` 等明显异常数量级价格，并把这些价格作为“AI 基础设施板块抛压延续”的用户可见摘要锚点。
    - 同窗其它 heartbeat raw preview 也继续围绕 `SNDK $2,090.71`、`MU $1,132.33` 解释或触发，但多数落为结构化失败 / noop；本条样本已经完成投递，因此不只是内部 raw preview 质量问题。
  - 查重结论：
    - 该样本与 2026-06-20/21 的同一缺陷根因一致：scheduler / heartbeat 对行情数值缺少稳定的数量级 sanity check，异常价仍可进入正式用户可见报告。
    - 这次发生在 Web heartbeat 投递链路，而不是原始 Feishu scheduler 早报，但受影响面仍是 scheduler / heartbeat 观察池行情锚，因此回退本单，不新建重复文档。
  - 用户影响：
    - 调度、收口和投递主链路均可用，没有错投、空回复或系统失败证据。
    - 但用户收到的行情摘要以异常价格作为判断依据，影响投资观察质量和可信度。
    - 因为不阻断功能链路，仍按质量性 `P3`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-28 23:02 CST）

- 本轮 2026-06-28 19:02-23:02 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28`
    - 21:35 CST Feishu scheduler 会话触发 `科技核心股池 · 晚间击球区快报`，21:36 CST `session.persist_assistant` 与 `done ... success=true` 收口。
    - 同条 final 继续把多只观察池标的输出为明显异常数量级价格：`MU $1,132.33`、`SNDK $2,090.71`、`STX $899.90`、`WDC $586.45` 等，并继续围绕这些价格给出击球区、财报日期和距离表。
    - 同窗 heartbeat raw preview 也继续围绕 `SNDK $2,090.71`、`MU $1,132.33` 解释，但多数未正式送达；本条 Feishu scheduler 样本已经正常收口，因此可作为用户可见质量证据。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
  - 但异常价格继续进入正式观察池快报，说明价格 sanity check 仍未覆盖当前 scheduler / function-calling 路径。

## 最新进展（2026-06-21 23:03 CST）

- 本轮 19:02-23:01 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/acp-events.log`
    - 本窗 ACP 可重构 21 次 `session/prompt`、21 次 `stopReason=end_turn`、0 个 ACP response error。
    - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 的观察池快报以 `end_turn` 收口，正文再次把多只观察池标的输出为明显异常数量级价格：`MU $1,151.95`、`SNDK $2,209.28`、`STX $1,075.77`、`WDC $754.10` 等，并继续给出击球区和财报日期。
    - 同条 final 来源段继续出现 `StockAnalysis 各标的行情页`，说明问题不只是内部来源标签外露，而是异常行情数值仍被当作正式观察池价格锚。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 ACP 日志为准。
- 用户影响：
  - 回复正常收口，观察池主链路仍可用，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
  - 但异常价格已经在 6 月 20 日早间简报和 6 月 21 日晚间观察池快报复现，说明需要优先修复价格 sanity check，而不仅是改写 `StockAnalysis` 这个用户可见标签。

## 修复记录

- 2026-06-28 11:01 CST 状态回退为 `New`：
  - 10:00-11:01 CST 当前 live 运行态再次将异常价格写入 Web heartbeat `deliver_preview` 并完成投递，说明 2026-06-22 的 prompt 级 sanity 约束没有稳定覆盖当前 heartbeat / function_calling 运行路径。
  - 本轮是缺陷台账维护任务，未修改业务代码、测试代码或配置代码。
- 2026-06-22 03:08 CST 状态更新为 `Fixed`：
  - 观察池 scheduler prompt 增加价格 sanity 约束：如果某个标的最新价相对固定击球区或近期有效价明显偏离一个数量级，或疑似把市值、复权 / 拆股口径、页面其它数字误当股价，必须把该标的价格写为“最新行情未完成稳定校验”。
  - 同类异常价不得继续输出为精确价格，也不得基于该异常价计算距离击球区或给出交易判断。
  - 验证：`cargo test -p hone-channels scheduled_watchlist_hit_zone_prompt_keeps_stable_local_fields --lib -- --nocapture` 通过。
  - 无关联 GitHub Issue；本轮按本地代码与回归验证关闭，不依赖生产日志、线上渠道状态或 live 重启。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-20 07:02-11:02 CST。
  - 本窗 ACP 可重构 13 个 session、20 次 `session/prompt`、20 次 `stopReason=end_turn`，没有 ACP response error、空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径或思维痕迹进入用户可见 final。
  - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 的 `核心观察池早间简报` 以 `end_turn` 收口。
  - final 开头明确写出行情口径为 `StockAnalysis 对 25 支标的的最新可得统一口径：2026-06-18 美股盘后 19:59 EDT`，随后将多只观察池标的输出为明显异常价格：`MU $1,151.95`、`SNDK $2,209.28`、`STX $1,075.77`、`WDC $754.10` 等，并继续给出击球区、财报日期和观察池结论。
  - 同条 final 也正确说明 `6月19日为美股休市日`、`不覆盖 6月20日盘前实时价`，说明问题不是时间口径缺失，而是 scheduler 消费/展示的行情数值本身异常。
- `docs/bugs/feishu_direct_storage_price_unverified_before_tool_complete.md`
  - 旧缺陷覆盖 Feishu direct 中 MU / SNDK 异常价格与未充分核验链路，状态已在 2026-06-09 标为 `Fixed`。
  - 本轮样本发生在 Feishu scheduler 的核心观察池早间简报，影响多只观察池标的和定时报告行情锚，属于新的受影响链路，因此单独登记，不复用直聊文档。
- `data/sessions.sqlite3`
  - 只读快照仍显示 `sessions.max(updated_at)=2026-06-17T10:37:37.207669+08:00`、`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`，最近真实会话证据需以 ACP 日志为准。

## 端到端链路

1. Feishu scheduler 触发 `核心观察池早间简报`。
2. runner 为 25 支核心 / 拓展观察池标的获取或整理最新可得行情。
3. final 采用 `StockAnalysis` 作为统一行情口径，并把异常放大的价格数值写入用户可见报告。
4. 报告仍正常完成、收口并展示击球区 / 财报日期 / 观察结论。
5. 用户看到的是一条结构完整但价格锚明显不可信的观察池早报。

## 期望效果

- scheduler 对观察池价格应逐项完成稳定核验，且价格数量级应通过基本 sanity check。
- 当某个行情源返回异常数量级、拆股/复权口径不明或与常识区间明显冲突时，应标注该标的行情未完成稳定校验，而不是继续输出精确价格。
- 定时报告可以说明休市和盘后口径，但不能把异常放大的价格当作最新行情锚。

## 当前实现效果

- 报告链路没有中断，用户可见 final 结构完整并正常 `end_turn`。
- final 同时输出了明显异常的多标的精确价格，并继续围绕这些价格展示击球区和观察池简表。
- 该问题不同于单纯 `StockAnalysis` / `data_fetch` 名称外露：本轮实际影响了用户可见行情数值质量。
- 该问题也不同于旧的 direct MU / SNDK 文档：当前样本发生在 scheduler 观察池批量早报链路，影响范围更偏定时报告质量。

## 用户影响

- 用户仍收到核心观察池早间简报，调度、收口和投递主链路没有证据显示失败。
- 但观察池报告里的多只价格锚明显异常，会降低击球区、价格距离和风险判断的参考价值。
- 本轮没有看到错误交易指令、持久化写坏、投递失败或错发对象证据，因此不按 P2/P1 处理。
- 因为主功能链路可用，问题主要影响行情质量和用户决策参考可信度，所以定级为质量性 `P3`。

## 根因判断

- 初步判断 scheduler 对 `StockAnalysis` 或中间行情摘要的数值缺少跨源 / 数量级 sanity check。
- 现有金融 prompt 的“多标的最新行情约束”更偏要求独立核验来源、时间戳和交易时段口径；本轮说明即使给出统一口径，仍需要在批量 scheduler 层校验价格数量级是否异常。
- 现有 `feishu_scheduler_data_fetch_tool_name_exposed.md` 跟踪的是内部工具名 / 数据源名外露；本单跟踪的是异常价格被当作正式行情锚。

## 下一步建议

- 在 scheduler 观察池行情整理层增加价格 sanity check：同一标的最新价若相对固定击球区、历史画像价格或前次有效价偏离异常倍数，应降级为“未完成稳定校验”。
- 对批量行情报告增加回归样本：当 MU / SNDK / WDC / STX 等价格出现异常数量级时，final 不应输出精确价格或基于该价格判断距离击球区。
- 若继续使用 `StockAnalysis` 页面作为补充校验源，需明确解析字段来源，避免把市值、拆股/复权口径或其它页面数字误当股价。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`docs/bugs/README.md` / 既有 bug 文档查重、`data/sessions.sqlite3` 上界、`data/runtime/logs/acp-events.log` 07:02-11:02 CST 结构化解析、用户可见 final 关键词扫描、最近四小时非文档代码提交检查。
