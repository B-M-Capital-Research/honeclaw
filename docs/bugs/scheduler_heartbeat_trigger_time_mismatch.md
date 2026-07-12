# Bug: Heartbeat 触发提醒把实际执行时间写成错误的北京时间

- **发现时间**: 2026-05-29 15:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 最新进展

- 本轮 `2026-07-12 15:02-19:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-12`
    - 本窗 80 条 raw / deliver preview 继续出现与实际 2026-07-12 下午到傍晚窗口不一致的时间口径。
    - 15:30 CST `TEM大事件心跳监控` raw preview 把当前时间写成 `2026-07-13 15:30 Beijing time`；`RKLB异动监控` deliver preview 写 `当前北京时间 2026-07-13 周一午后`；`持仓重大事件心跳检测` deliver preview 写 `2026-07-13 约 15:30`。
    - 16:00 CST `TEM大事件心跳监控` deliver preview 同时写 `2026-07-13 15:30` 和“美股已进入常规交易时段 / 尚未开市”等互相矛盾口径；`持仓财报与重大新闻心跳提醒` deliver preview 写 `检查时间：2026-07-13 北京时间约 09:36`。
    - 19:00 CST `ASTS 重大异动心跳监控` deliver preview 写 `北京时间 2026-07-13 08:50 口径`；`AI与科技持仓观察关键事件心跳提醒` deliver preview 写 `北京时间 2026-07-11 22:02 · 监控扫描`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / deliver preview 仍属于 heartbeat 模型时间上下文 / 执行窗口口径漂移，与本文档既有链路一致。
  - 用户影响：
    - 错误日期继续影响用户对增量扫描、重复抑制、行情新鲜度和是否处于交易时段的判断。
    - 调度和投递主链路仍可运行；因此保持质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-12 07:01-11:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-12`
    - 08:00 CST `AAOI 1.6T 光模块心跳检测` raw preview 把当前时间写成 `2026-07-13 北京时间 15:30`；`光模块板块关键事件心跳提醒` raw preview 写 `2026-07-13 北京时间约 09:35`。
    - 08:00 CST `ORCL 大事件监控` deliver preview 写 `2026-07-13 周一，北京时间约 17:00`，与实际 2026-07-12 08:00 CST 不一致。
    - 11:00 CST `持仓财报与重大新闻心跳提醒` deliver preview 写 `检查时间：2026-07-13 北京时间约 10:00`；`AI与科技持仓观察关键事件心跳提醒` deliver preview 写 `北京时间 2026-07-11 ~21:30`；`FOTO 光子学ETF心跳检测`、`TEM大事件心跳监控`、`ORCL 大事件监控` 多条 deliver preview 继续写 `2026-07-13`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / delivered preview 仍属于 heartbeat 模型时间上下文 / 执行窗口口径漂移，与本文档既有链路一致。
  - 用户影响：
    - 错误日期继续影响用户对增量扫描、重复抑制、行情新鲜度和是否处于交易时段的判断。
    - 调度和投递主链路仍可运行；因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-12 03:02-07:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-11`
    - 05:30 CST `持仓财报与重大新闻心跳提醒` deliver preview 把检查时间写成 `2026-07-13 09:30（北京时间）`，与实际 2026-07-12 05:30 CST 执行窗口不一致。
    - 05:30 CST `AI与科技持仓观察关键事件心跳提醒` deliver preview 写成 `北京时间 2026-07-11 21:30`，早于实际窗口。
    - 06:00 CST `光模块板块关键事件心跳提醒` deliver preview 写成 `2026-07-13 北京时间 09:30`，并把美东时间写成 `2026-07-13 01:30`。
    - 07:00 CST `全天原油价格3小时播报` deliver preview 写成 `北京时间 2026年5月12日 19:42`；同窗多条 `Cerebras IPO与业务进展心跳监控`、`FOTO 光子学ETF心跳检测`、`持仓重大事件心跳检测`、`AAOI 1.6T 光模块心跳检测` 继续写出 `2026-07-13` 的错误检查口径。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview / raw preview 仍属于 heartbeat 模型时间上下文 / 执行窗口口径漂移，与本文档既有链路一致。
  - 用户影响：
    - 错误日期继续影响用户对增量扫描、重复抑制、行情新鲜度和是否处于交易时段的判断。
    - 调度和投递主链路仍可运行；因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 23:02-2026-07-12 03:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-11`
    - 03:00 CST `存储板块关键事件心跳提醒` deliver preview 把检查时间写成 `2026-07-13 北京时间 10:00`，与实际 2026-07-12 03:00 CST 执行窗口不一致。
    - 03:00 CST `ORCL 大事件监控` deliver preview 把当前时间写成 `2026-07-13 周一，北京时间 14:00`，并把 2026-07-10 行情当作尚未进入“今日日内数据”的口径。
    - 03:00 CST `光迅科技关键事件心跳提醒` deliver preview 写成 `2026年4月4日（周六）北京约19:30`，并把 2026-04-22 财报披露描述为晚于当前时间 18 天。
    - 03:00 CST `NVDA 关键事件心跳提醒` deliver preview 写成 `2026-07-11 北京时间 18:30`；`持仓重大事件心跳提醒` raw preview 又写当前时间为 `2026-07-11T22:30:35+08:00`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview / raw preview 仍属于 heartbeat 模型时间上下文 / 执行窗口口径漂移，与本文档既有链路一致。
  - 用户影响：
    - 错误日期继续影响用户对增量扫描、重复抑制、行情新鲜度和是否处于交易时段的判断。
    - 调度和投递主链路仍可运行；因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 19:01-23:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 同窗只有 3 个 user turn / 3 条 assistant 记录；普通 scheduler 用户可见文本没有命中 `<think>`、provider 原始错误、本机路径、`data_fetch`、`quote_short` 或原始工具 JSON 外泄。
  - `data/runtime/logs/web.log.2026-07-11`
    - 19:30 / 20:00 / 20:30 / 21:00 / 21:30 / 22:00 / 22:30 / 23:00 CST `光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`FOTO 光子学ETF心跳检测` 等多条 `deliver_preview` 继续把实际 2026-07-11 夜间窗口写成 `2026-07-12`。
    - 19:00 / 19:30 / 20:30 / 22:00 / 23:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` 多条 `deliver_preview` 写出 `北京时间 2026-07-10 20:30` 或 `2026-07-10 21:00`，早于实际执行窗口。
    - 21:30 / 22:30 / 23:00 CST 多条样本继续把 quote 时间戳或检查时间写成 `2026-07-11 04:00`、`12:31`、`21:30` 等与当前执行时间不一致的口径。
  - 查重结论：
    - 最新证据仍属于同一 heartbeat 时间上下文 / 用户可见检查口径漂移链路，不新建重复缺陷。
  - 用户影响：
    - 本窗主要是 deliver / duplicate suppression 路径中的错误时间口径，未见错投、直聊失败或全渠道不可用；影响触发判断可信度和用户对增量扫描时效性的理解。
    - 因不影响直聊 / 调度 / 投递主功能链路，维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 15:01-19:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 15:01 CST 后没有新增本地 `session_messages` 或 `cron_job_runs`；本轮以 `data/runtime/logs/web.log.2026-07-11` 的 heartbeat runtime 证据为主。
  - `data/runtime/logs/web.log.2026-07-11`
    - 15:30 / 16:30 / 17:00 / 17:30 / 19:00 CST `持仓财报与重大新闻心跳提醒`、`光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒` 多条 `deliver_preview` 继续把实际 2026-07-11 窗口写成 `2026-07-12` 检查时间。
    - 15:00 / 16:30 / 18:01 / 19:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` 多条 `deliver_preview` 写出 `北京时间 2026-07-10 20:30`，早于实际执行窗口。
    - 16:30 CST `光迅科技关键事件心跳提醒` `deliver_preview` 写出“今日为 4月4日”，但同段又引用 2026-07-10 A 股行情数据；19:00 CST `Cerebras IPO与业务进展心跳监控` 写 `北京时间 2026-07-11 21:30`，晚于实际 19:00 CST 执行窗口。
  - 查重结论：
    - 最新证据仍属于同一 heartbeat 时间上下文 / 用户可见检查口径漂移链路，不新建重复缺陷。
  - 用户影响：
    - 本窗主要是 deliver / duplicate suppression 路径中的错误时间口径，未见错投、直聊失败或全渠道不可用；影响触发判断可信度和用户对增量扫描时效性的理解。
    - 因不影响直聊 / 调度 / 投递主功能链路，维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 11:01-15:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 同窗只有 1 个 user turn / 1 条 assistant final，为 12:00 CST Feishu 普通 scheduler `每日公司资讯与分析总结`，12:02 CST 正常收口；assistant final 污染扫描未命中空回复、`reasoning_content`、`<think>`、本机绝对路径、provider 原始错误、panic、quota、`mcpServers`、`data_fetch`、`quote_short`、`company_profiles/` 或原始工具 JSON。
  - `data/runtime/logs/web.log.2026-07-11`
    - 12:00 CST `存储板块关键事件心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 北京时间约 10:00`，与实际 2026-07-11 12:00 CST 执行窗口不一致；随后被 duplicate suppression。
    - 12:00 CST `光模块板块关键事件心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 北京时间约 18:00`，且 raw preview 自称当前时间为 2026-07-12；随后进入 duplicate suppression / 跳过路径。
    - 13:00-14:30 CST `持仓财报与重大新闻心跳提醒`、`光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒` 多条 `deliver_preview` 继续写出 `2026-07-12` 检查时间。
    - 11:31 / 14:30 / 15:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` 多条 `deliver_preview` 写出 `北京时间 2026-07-10 20:30`，早于实际 2026-07-11 执行窗口；均随后被 duplicate suppression 或未确认正式发送。
    - 15:00 CST `RKLB异动监控` `deliver_preview` 写出 `北京时间 2026-07-12 09:00 口径`，与实际 2026-07-11 15:00 CST 执行窗口不一致。
  - 查重结论：
    - 最新证据仍属于同一 heartbeat 时间上下文 / 用户可见检查口径漂移链路，不新建重复缺陷。
  - 用户影响：
    - 本窗主要是 deliver / duplicate suppression 路径中的错误时间口径，未见错投、直聊失败或全渠道不可用；影响触发判断可信度和用户对增量扫描时效性的理解。
    - 因不影响直聊 / 调度 / 投递主功能链路，维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 07:01-11:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 同窗 3 个 user turn / 3 条 assistant final 均正常收口；assistant final 污染扫描未命中空回复、`reasoning_content`、`<think>`、本机绝对路径、provider 原始错误、panic、quota、`mcpServers`、`data_fetch`、`quote_short`、`company_profiles/` 或原始工具 JSON。
  - `data/runtime/logs/web.log.2026-07-11`
    - 11:00 CST `光模块板块关键事件心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 北京时间约 18:00`，与实际 2026-07-11 11:00 CST 执行窗口不一致；随后被 duplicate suppression，未确认正式发送。
    - 11:00 CST `持仓财报与重大新闻心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 09:00 北京时间`，并把美东时间也写成 2026-07-12；随后被 duplicate suppression。
    - 11:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` `deliver_preview` 写出 `本次检查时间为北京时间 2026-07-10 20:30`，早于实际执行窗口；该文本随后被 duplicate suppression。
  - 查重结论：
    - 最新证据仍属于同一 heartbeat 时间上下文 / 用户可见检查口径漂移链路，不新建重复缺陷。
  - 用户影响：
    - 本窗主要是 deliver / duplicate suppression 路径中的错误时间口径，未见错投、直聊失败或全渠道不可用；影响触发判断可信度和用户对增量扫描时效性的理解。
    - 因不影响直聊 / 调度 / 投递主功能链路，维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 `2026-07-11 03:00-07:01 CST` 真实运行态在 03:11 代码提交后继续复发，状态从代码级 `Fixed` 回退为运行态 `New`：
  - `data/runtime/logs/web.log.2026-07-10`
    - 最近四小时仅有 2 个 user turn / 2 条 assistant final，均正常收口；assistant final 污染扫描未命中 `reasoning_content`、`<think>`、本机路径、raw tool JSON、`data_fetch` / `quote_short` 等用户可见内部字段。
    - 06:00 CST `持仓财报与重大新闻心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 北京时间约 15:30`，与实际 2026-07-11 06:00 CST 执行窗口不一致；随后被 duplicate suppression，未确认正式发送。
    - 06:00 CST `光模块板块关键事件心跳提醒` `deliver_preview` 写出 `本轮检查：2026-07-12 北京时间约 03:00`，同样晚于实际执行窗口；随后进入送达/抑制路径。
    - 06:00 CST `存储板块关键事件心跳提醒` `deliver_preview` 写出 `检查时间：2026-07-12 约 15:30 北京时间`，并围绕错误检查日判断新闻增量。
    - 07:00 CST `光模块板块关键事件心跳提醒` `deliver_preview` 继续写 `检查时间：2026-07-12 北京时间约 15:15`，证明 03:09 的 `检查时间 / 核验摘要` 归一化仍未覆盖该标题前缀 / 多段时间口径形态。
  - 查重结论：
    - 最新证据仍属于本文档同一 heartbeat 时间口径漂移链路，不新建重复缺陷。
    - 本轮不是全局 runtime 停摆：`data/runtime/logs/acp-events.log` 推进到 06:01 CST 且 Feishu/Web scheduler final 均有收口样本。
  - 用户影响：
    - 错误检查日期继续影响 heartbeat 触发判断、重复抑制和用户对增量扫描时效性的理解；但未见直聊 / 调度 / 投递主链路不可用、错对象投递或数据安全问题。
    - 因主要影响返回质量和时间口径可信度，不影响主功能链路，维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-11 03:09 CST` 代码级修复并回归通过，状态更新为 `Fixed`：
  - `crates/hone-channels/src/scheduler.rs`
    - heartbeat 出站归一化新增 `normalize_heartbeat_check_time_context(...)`，在现有“当前时间上下文 / 北京时间触发时间”修正之外，额外把 `检查时间：...`、`核验摘要（...）` 这两类用户可见检查口径统一重写到 scheduler 权威北京时间。
    - 新 metadata 增加 `beijing_check_time_context_normalized` 与 `original_beijing_check_time_context`，方便后续巡检区分“模型判断仍漂移”与“用户可见正文已被收口修正”。
  - 新增 / 复跑回归：
    - `cargo test -p hone-channels heartbeat_normalizes_conflicting_check_time_context --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_normalizes_conflicting_verification_summary_time --lib -- --nocapture`
    - `cargo check -p hone-channels --tests`
  - 当前按代码与回归验证更新为 `Fixed`；本轮未重启 live runtime，待后续运行态复核是否仍有其他未覆盖的“日期在前 / 北京时间在中 / 标题前缀”漂移形态。

- 本轮 2026-07-10 19:02-23:03 CST 真实运行态继续复发，状态维持 `New`：
  - cloud PostgreSQL `cloud_cron_job_runs`
    - 当前 runtime 已切到 cloud scheduler 台账；19:02-23:03 CST `cloud_cron_job_runs` 继续推进，证明本轮不是 runtime 全局停摆。
    - 22:30 CST Web heartbeat `光模块板块关键事件心跳提醒` 落成 `completed + sent + delivered=1`，但用户可见 `response_preview` 开头写 `检查时间：2026-07-11 22:30 北京时间（美东 07:30，盘前）`，比实际执行窗口 2026-07-10 22:30 CST 晚 1 天。
    - 23:00 CST Web heartbeat `持仓重大事件心跳提醒` 落成 `completed + sent + delivered=1`，但 preview 写 `核验摘要（2026-07-10 北京时间 22:00）`，与本轮 23:00 执行窗口不一致。
    - runtime 日志同窗还显示 `持仓财报与重大新闻心跳提醒` raw / deliver preview 把检查时间写成 `2026-07-11 10:36 北京时间`，随后因 duplicate suppression 未进入用户可见发送；该样本只作为判断链路时间上下文漂移证据。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview / raw preview 仍属于 heartbeat 模型时间上下文 / 执行窗口口径漂移，与本文档既有链路一致。
  - 用户影响：
    - 本窗已再次确认用户可见 heartbeat 送达正文出现错误检查日期；调度和投递主链路仍可运行，但错误日期会影响用户对增量扫描、重复抑制和行情新鲜度的判断。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-10 07:01-11:02 CST 真实运行态继续观察，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 新增 114 条运行记录，仍有 82 条 `noop + skipped_noop` 与 32 条 `execution_failed + skipped_error`，结构化退化链路继续存在。
    - 本窗未确认新的 heartbeat `completed + sent + delivered=1` 用户可见提醒写错执行日期；时间口径问题主要保留在既有 raw preview / noop / failed 判断风险内，未形成新的正式投递正文样本。
    - 代表样本包括 07:30 CST `全天原油价格3小时播报` raw preview 写 `当前北京时间是 2026-04-04 02:17:07`，与实际 2026-07-10 07:30 CST 执行窗口不一致；08:00 CST `美股盘中科技股机会心跳监控` raw preview 把 2026-07-09 新闻视作相对 `2026年4月4日` 的“未来”数据；11:00 CST `DRAM 心跳监控` raw preview 把 timestamp `1783627201` 解释为 `July 9, 2026` 并继续用于当前价 / 均线判断。
  - 查重结论：
    - 本窗没有新的独立根因，也没有足以回退严重等级或关闭缺陷的止血证据；仍归入本文档既有“触发提醒时间口径漂移”链路。
  - 用户影响：
    - 本窗未见新的用户可见错时间投递，因此不新增独立缺陷；但 heartbeat 判断链路仍可能受错误时间上下文影响触发判断、重复抑制和行情新鲜度判断。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-09 23:02-2026-07-10 03:02 CST 真实运行态继续观察，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 新增 112 条运行记录，仍有 65 条 `noop + skipped_noop`、45 条 `execution_failed + skipped_error`、1 条 `completed + sent` 与 1 条边界在途。
    - 本窗未确认新的 heartbeat 用户可见提醒把执行日期写错；但 raw preview / 失败上下文仍持续出现错误时间口径。
    - 代表样本包括 23:30 CST `DRAM 心跳监控` raw preview 写 `当前北京时间 2026-07-09T03:30:44`，与实际 23:30 CST 执行窗口相差约 20 小时；00:00 CST `RKLB 全面心跳检测` raw preview 写 `Current time: 2025-07-17 13:20:00 CST`；00:30 CST `TSLA 正负触发条件心跳监控` raw preview 写 `当前时间：2026年4月4日 09:30`；02:30 CST `TEM大事件心跳监控` raw preview 写 `当前时间：2025年12月12日 20:35:49 北京时间`。
  - 查重结论：
    - 本窗没有新的独立根因，也没有足以回退严重等级或关闭缺陷的止血证据；仍归入本文档既有“触发提醒时间口径漂移”链路。
  - 用户影响：
    - 本窗未见新的用户可见错时间投递，因此不新增独立缺陷；但 heartbeat 判断链路仍可能受错误时间上下文影响触发判断、重复抑制和行情新鲜度判断。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-09 19:02-23:02 CST 真实运行态继续观察，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 新增 112 条运行记录，仍有 86 条 `noop + skipped_noop` 与 26 条 `execution_failed + skipped_error`，结构化退化链路继续存在。
    - 本窗未确认新的 heartbeat `completed + sent + delivered=1` 用户可见提醒写错执行日期；时间口径问题主要保留在既有结构化失败 / noop 判断风险内，未形成新的正式投递正文样本。
  - 查重结论：
    - 本窗没有新的独立根因，也没有足以回退严重等级或关闭缺陷的止血证据；仍归入本文档既有“触发提醒时间口径漂移”链路。
  - 用户影响：
    - 本窗未见新的用户可见错时间投递，因此不新增独立缺陷；但 heartbeat 判断链路仍可能受错误时间上下文影响触发判断、重复抑制和行情新鲜度判断。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-09 11:01-15:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / delivered preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / 数据日期”口径。
    - 代表样本为 13:00 CST `RKLB 全面心跳检测` `run_id=47365`，执行时间为 `2026-07-09T13:00:43+08:00` 且 `completed + sent + delivered=1`，但用户可见标题写成 `【RKLB 北京时间 2026年7月8日 重要更新 — Morgan Stanley 大幅上调牛市目标价】`，比实际执行日早 1 天。
    - 同窗 raw preview 还有多条错误或漂移口径：15:00 CST `全天原油价格3小时播报` 写 `Current time: 2026-04-04 15:11:26 Beijing time`；15:00 CST `TSLA 正负触发条件心跳监控` 写 `Current time context: July 8, 2026`；14:30 CST `TEM破位预警` noop reason 写 `数据时间戳 2025-11-18`；14:30 CST `全天原油价格3小时播报` 把 `1783578613` 近似换算为 `2026-06-17`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview / raw preview 仍属于 heartbeat 模型时间上下文 / 数据日期漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误日期已进入成功送达的用户可见提醒，可能影响用户对增量扫描时效性的判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-09 07:00-11:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / delivered preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / 数据日期”口径。
    - 代表样本为 08:01 CST `AAOI 全面心跳检测` `run_id=47220`，执行时间为 `2026-07-09T08:01:02+08:00` 且 `completed + sent + delivered=1`，但用户可见正文开头写成 `2026-07-06 北京时间 AAOI 例行增量扫描`，比实际执行日早 3 天。
    - 同窗还有多条 heartbeat 结构化失败 / noop 样本继续围绕错误时间进入判断，但本轮以已经送达的 AAOI 样本作为主证据。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview 仍属于 heartbeat 模型时间上下文 / 数据日期漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误日期已进入成功送达的用户可见提醒，可能影响用户对增量扫描时效性的判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-09 03:02-07:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / delivered preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本为 03:30 CST `DRAM 心跳监控` `run_id=47096`，执行时间为 `2026-07-09T03:30:41+08:00` 且 `completed + sent + delivered=1`，但用户可见正文写出 `Roundhill Memory ETF(DRAM) 现价 $61.565（美东收盘价，数据时间约 2026 年 3 月 6 日盘后）`，随后又把该价格作为已站上 `$60` 触发位的当前判断依据。
    - 同窗另有多条 heartbeat raw / failed preview 继续围绕错误时间或 timestamp 进入判断，但本轮以已经送达的 DRAM 样本作为主证据。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview 仍属于 heartbeat 模型时间上下文 / 数据日期漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误数据日期已进入成功送达的用户可见提醒，可能影响用户对行情新鲜度和触发窗口的判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 23:00-2026-07-09 03:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / noop / failed preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 00:30 CST `全天原油价格3小时播报` raw preview 围绕 `1783528205` 自行换算当前油价时间；02:30 CST `TEM大事件心跳监控` raw preview 写 `Current time: 2026-04-04 (from the system context)`；02:30 CST `伦敦金跌破4100提醒` `PlainTextSuppressed` raw preview 把 `1783535407` 解释为 `2026 年 7 月 10 日美东盘中`；03:00 CST `AAOI 全面心跳检测` raw preview 写 `Current time: 2026-07-07T22:37+08:00`，与实际 2026-07-09 03:00 CST 执行窗口不一致。
    - 同窗没有确认新的用户可见送达错时间样本；本轮复核依据仍是错误时间进入 heartbeat 判断 / noop / failure 原始结果，而非新的正式投递正文。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 19:01-23:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / noop / failed preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 19:30 CST `ASTS 全面心跳检测` raw preview 写 `Time: 2026-04-04 14:28:01 CST`，与实际 2026-07-08 19:30 CST 执行窗口不一致；20:00 CST `TEM大事件心跳监控` raw preview 写 `当前时间：2026年4月4日 15:30（北京时间，周六）`；21:30 CST `AAOI 全面心跳检测` raw preview 把 quote timestamp 判断为 `January 2026` 并与 2026-07-07/08 系统口径冲突；21:30 CST `ASTS 全面心跳检测` raw preview 写 `Current time (system stated): 2025-05-19 08:01:03 CST`。
    - 同窗没有确认新的用户可见送达错时间样本；本轮复核依据仍是错误时间进入 heartbeat 判断 / noop / failure 原始结果，而非新的正式投递正文。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 15:03-19:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / noop / failed preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 16:00 CST `全天原油价格3小时播报` raw preview 把 USO timestamp `1783454400` 误算为 `Jan 13, 2026`，并引用 `2026-04-04` 系统日期；16:30 CST `TSLA 正负触发条件心跳监控` 写 `Current time is 2026-07-08 04:30 Beijing time`，与实际 16:30 CST 执行窗口不一致；17:30 CST `RKLB 全面心跳检测` raw preview 明确写出 `System time: 2026年4月4日` 与 `Reminder date: 2026年7月8日` 冲突；18:00 CST `伦敦金跌破4100提醒` `JsonMalformed` raw preview 写 `triggered_at=2026-04-04 10:00 北京时间`；18:00 CST `TEM大事件心跳监控` raw preview 写 `当前时间：2025-06-04 08:40:02 北京时间`。
    - 同窗 18:00 CST `TSLA 正负触发条件心跳监控`、18:00 CST `heartbeat_绿田机械基本面跟踪`、19:00 CST `伦敦金跌破4100提醒` 3 条 heartbeat 成功送达；未确认新的用户可见送达错时间样本，本轮复核依据仍是错误时间进入 heartbeat 判断 / noop / failure 原始结果。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 11:03-15:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / noop / failed preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 11:30 CST `AAOI 全面心跳检测` raw preview 围绕 `1783454401` 自行换算；12:30 CST `美股盘中科技股机会心跳监控` raw preview 写 `Time: July 7, 2026, US market session (approximately 2:31 PM ET)`；14:00 CST `DRAM 心跳监控` raw preview 把 quote timestamp `1783454401` 判断为 `July 9, 2026`；15:00 CST `ASTS 全面心跳检测` raw preview 写 `Current Data (2026-07-08, based on the system context)` 后又把 quote timestamp 解释为 `2026-07-07 in US market hours`，时间口径仍在判断内漂移。
    - 13:00 CST `RKLB 全面心跳检测` 有 1 条 `completed + sent + delivered=1`，正文未确认新的错误北京时间外露；本轮复核依据仍是错误时间进入 heartbeat 判断 / noop / failure 原始结果，而非新的用户可见送达错时间。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 07:00-11:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / noop / failed preview 继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 07:00 CST `全天原油价格3小时播报` raw preview 把 `1783464278/1783465215` 判断为 `early Feb 25, 2025 Beijing time`；08:00 CST `RKLB 全面心跳检测` raw preview 写 `Current date context: 2025-09-23` 并把 2026-07 行情 / 新闻判为相对系统日期的未来数据；09:30 CST `伦敦金跌破4100提醒` raw preview 写 `检查时间：2025-05-02 23:27 北京时间`；11:00 CST `美股盘中科技股机会心跳监控` raw preview 写 `数据截至美东时间 2025-12-15 收盘`。
    - 同窗没有确认新的用户可见送达错时间样本；本轮复核依据仍是错误时间进入 heartbeat 判断 / noop / failure 原始结果。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

- 本轮 2026-07-08 03:04-07:00 CST 真实运行态继续复发，状态从代码级 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 03:10 CST 非文档提交 `e4a39b98 fix: normalize heartbeat current-time context` 落地后，live heartbeat raw / noop / failed preview 仍继续出现与实际执行窗口不一致的“当前时间 / 检查时间 / quote timestamp”口径。
    - 代表样本包括 05:00 CST `全天原油价格3小时播报` raw preview 称 system prompt 为 `2026年4月4日` 且不知道当前精确时间；06:00 CST `TEM大事件心跳监控` raw preview 写 `当前时间：2025-05-19 北京时间 22:03`；06:00 CST `伦敦金跌破4100提醒` raw preview 写 `检查时间：北京时间 2026-04-25 00:30`；06:00 CST `ASTS 全面心跳检测` raw preview 把行情数据写成 `2025-10-29 当前时间`；07:00 CST `全天原油价格3小时播报` raw preview 把 `1783464278/1783465215` 判断为 `early Feb 25, 2025 Beijing time`。
    - 同窗唯一成功送达 heartbeat 为 04:01 CST `TSLA 正负触发条件心跳监控`，送达 preview 未确认新的错误北京时间外露；本轮回退依据是错误时间仍进入 heartbeat 判断 / noop / failure 原始结果，而非新的用户可见送达错时间。
  - 查重结论：
    - 本窗没有新的独立根因；上述样本仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度和投递主链路未被该问题直接阻断，但错误时间上下文仍可能影响触发判断、重复抑制和行情新鲜度判断。本窗没有错投、数据安全、全渠道不可用或新的用户可见送达错时间证据；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
    - 因该问题不影响直聊 / 调度 / 投递主功能链路，只影响 heartbeat 触发判断质量与用户可见时间口径可信度，所以定级保持 P3。

## 修复记录（2026-07-08 03:04 CST）

- 代码级修复：`crates/hone-channels/src/scheduler.rs` 在 heartbeat 出站归一化阶段新增“当前/系统时间上下文”修正，覆盖 `系统当前时间`、`当前时间上下文`、`Current time context`、`Current date context`、`System context`、`current time is` 等中英文自述时间口径；当模型把这类当前时间写成错误日期、错误北京时间或错误 ISO 时间时，送达前统一改写到 scheduler 权威北京时间。
- 回归验证：
  - `cargo test -p hone-channels heartbeat_normalizes_conflicting_current_time_context --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_normalizes_conflicting_english_current_time_context --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_normalizes_conflicting_beijing_trigger_time --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`
  - `git diff --check`
- 状态更新为 `Fixed`；本轮未重启 live runtime，后续如真实运行态在当前 HEAD 上继续出现相同“当前/系统时间上下文”漂移，再以新样本回退为 `New`。

- 本轮 2026-07-07 23:02-2026-07-08 03:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / delivered preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本为 23:30 CST `DRAM 心跳监控` `run_id=46322`，执行时间为 `2026-07-07T23:30:46+08:00` 且 `completed + sent + delivered=1`，但送达正文开头写出 `2026年7月7日（北京时间约14:30）`，并把该美股常规盘窗口描述成“7月7日亚盘/美股盘前时段”。
  - 查重结论：
    - 本窗没有新的独立根因；上述 delivered preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文已进入成功送达的用户可见提醒，可能影响用户对行情新鲜度和触发窗口的判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-07 19:02-23:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 19:30 CST `TEM大事件心跳监控` raw preview 把 `1783368000` 换算为 `2026年6月14日` 并据此判断行情；20:00 CST `AAOI 全面心跳检测` 把 current time context 写成 `2026-04-04 02:02:52 CST`；20:30 CST `伦敦金跌破4100提醒` 把 `1783427409` 粗略换算到 `December 2025`；22:00 CST `全天原油价格3小时播报` 在同一判断中混入 `2026年6月20日`、`2025年5月` 等错误时间；22:01 CST `美股盘中科技股机会心跳监控` 把今天写成 `2025年9月3日` 并引用 `2025年9月6日` Reddit 讨论；23:00 CST `伦敦金跌破4100提醒` raw preview 写出 `系统当前时间：2025年4月28日 17:50 北京时间`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-07 15:00-19:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 16:30 CST `RKLB 全面心跳检测` raw preview 把当前时间判断成 `2026-07-06` 并沿用前一日价格上下文；17:00 CST `ASTS 全面心跳检测` raw preview 写当前时间为 `2026年7月6日`；17:30 CST 同 job 又把当前时间误判为 `2025年12月17日` 并围绕 2025/2026 发射计划做触发判断；18:30 CST `RKLB 全面心跳检测` raw preview 继续把当前新闻窗口与旧日价格混在一起判断。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-06 23:04-2026-07-07 03:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 23:30 CST `RKLB 全面心跳检测` raw preview 把当前时间写成 `April 6, 2026` 并把 2026-07 新闻误判为相对系统时间的未来数据；00:00 CST 同任务写 `Current time context: 2026-04-06T10:00:27+08:00`；02:00 / 02:30 CST `RKLB 全面心跳检测` 继续把 2026-07 新闻判作相对 `2026-04-06` 的未来日期；03:00 CST `全天原油价格3小时播报` 围绕 `1783364404` Unix timestamp 自行换算当前油价时间。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-06 15:02-19:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 15:30 CST `RKLB异动监控` raw preview 把当前时间上下文写成 `2026年4月6日` 并围绕 `1783022400` 自行换算；16:30 CST `Cerebras IPO与业务进展心跳监控` 把当前时间写成 `2025-06-27 16:20:00 CST` 后又重算 `1783022401`；18:00 CST `DRAM 心跳监控` 写系统当前时间为 `2026年4月4日 16:00 CST`，同时与 2026-07 新闻相互冲突；18:30 CST `伦敦金跌破4100提醒` 将 `1783333806` 粗略换算为 `January 2026` 附近；18:30 CST `RKLB异动监控` 继续把 2026-07 新闻当作相对 `2026-04-06` 的未来新闻。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-06 07:02-11:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / delivered preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 08:30 CST `持仓重大事件心跳检测` raw preview 围绕 `timestamp 1783022401` 做当前行情判断，10:00 CST `RKLB异动监控` delivered preview 把当前行情写成 `2026年4月6日`，10:30 CST `持仓重大事件心跳检测` raw preview 继续使用 `ASTS/RKLB ... timestamp 1783022400/1783022401` 作为当前判断依据。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / delivered preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-06 03:01-07:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 03:30 CST `TEM大事件心跳监控` raw preview 把当前时间写成 `2025-12-20 09:40:05 北京时间` 并把 `1783022401` 解释成 `2025年12月19日`，04:30 CST `Cerebras IPO与业务进展心跳监控` 反复换算 `1783022401` 并写成 `2026-07-01 00:00:01 UTC`，05:00 CST `DRAM 心跳监控` 把当前时间写成 `July 5, 2026, 01:30 Beijing time`，06:30 CST `全天原油价格3小时播报` 把系统上下文写成 `2026-04-04 11:31:19`，07:00 CST `持仓重大事件心跳检测` 写 `current time is 2026-04-04 00:00 Beijing time`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-05 23:01-2026-07-06 03:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 00:30 CST `Cerebras IPO与业务进展心跳监控` raw preview 写 `Current date context: ... 2026/4/4`，01:00 CST 同 job 将 `1783022401` 推断为 `2026-06-30`，02:30 CST `DRAM 心跳监控` 围绕 `1783022401` 自行换算数据日期，03:00 CST `全天原油价格3小时播报` 把当前北京时间写成 `3:32 AM ... July 4, 2026`，与本轮 2026-07-06 03:00 CST 执行窗口不一致。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-05 19:02-23:06 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 19:30 CST `全天原油价格3小时播报` raw preview 围绕 `1783251004` 自行换算 Unix 时间，19:30 CST `Cerebras IPO与业务进展心跳监控` 把 `1783022401` 当作当前检查依据反复重算，20:30 CST `DRAM 心跳监控` 将 `1783022401` 解释为 `July 5, 2026`，20:30 CST `持仓重大事件心跳检测` 又把同类 timestamp 解读为 `2026-04-06`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-05 11:01-15:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 13:30 CST `RKLB异动监控` raw preview 把 2026-07-04 新闻判为相对 `June 30, 2026` 的未来新闻，14:30 CST `全天原油价格3小时播报` raw preview 围绕 `1783098008` / `1783209600` 自行换算，14:30 CST `伦敦金跌破4100提醒` 把 `1783209600` 换算成 `2026年6月12日 00:00:00 UTC`，15:00 CST `DRAM 心跳监控` 继续围绕 MU `$975.56` 与异常时间戳做当前判断。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-05 11:01 CST）

- 本轮 2026-07-05 07:02-11:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 07:02 CST `SIVE POET/Nokia/1.6T DFB 心跳检测` raw preview 写 `Current check time context: 2026-05-29 02:02:29 UTC`，08:01 CST `TEM大事件心跳监控` 把 `1783022401` 推断为 `Jan 2026` 附近，08:30 CST `DRAM 心跳监控` 围绕 `1783022401` 自行换算数据日期，10:00 CST `TSLA 正负触发条件心跳监控` 继续围绕 `1783022400` 做当前日期推断。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-05 07:04 CST）

- 本轮 2026-07-05 03:02-07:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 07:02 CST `SIVE POET/Nokia/1.6T DFB 心跳检测` raw preview 写 `Current check time context: 2026-05-29 02:02:29 UTC`，07:01 CST `heartbeat_绿田机械基本面跟踪` raw preview 写 `System context: 2025-08-07 (Thursday) 09:05 Beijing time`，07:01 CST `持仓重大事件心跳检测` raw preview 把 `timestamp 1743465601` 解读为 `April 4, 2026`，07:00 CST `Cerebras IPO与业务进展心跳监控` raw preview 继续围绕 `1783022401` 做当前数据判断。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-04 19:04 CST）

- 本轮 2026-07-04 23:02-2026-07-05 03:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括 `ORCL 大事件监控` raw preview 把 quote timestamp 解读为 “from January 2026”，`全天原油价格3小时播报` raw preview 把当前时间写成 `2026-04-04` 附近，`DRAM 心跳监控` raw preview 围绕 `1783022401` 自行换算数据时间。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-04 15:02-19:04 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-04` / `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现与实际执行窗口不一致或容易误解的时间口径。
    - 代表样本包括多条 raw preview 围绕 quote timestamp `1783022400` / `1783066093` 自行换算并判断数据时间；另有 `光迅科技关键事件心跳提醒` raw preview 把已存在监控更新时间写成 `2026-07-04T03:30:26+00:00`，与本轮 15:02-19:04 CST 执行窗口不同。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-03 23:02 CST）

- 本轮 2026-07-03 19:02-23:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw / deliver preview 继续出现与实际执行窗口不一致的时间口径。
    - 代表样本包括 23:01 CST `heartbeat_绿田机械基本面跟踪` deliver preview 正文写 `北京时间2026年6月13日`，以及 23:01 CST `Cerebras IPO与业务进展心跳监控` raw preview 把当前时间写成 `2026-04-04 20:40 Beijing time`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / deliver preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-03 19:02 CST）

- 本轮 2026-07-03 15:10-19:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现 `2026年4月4日` / `April 4, 2026` 等与实际执行窗口不一致的时间口径。
    - 15:10-19:01 CST 统计可检出 3 条 `2026年4月4日`、2 条 `April 4, 2026`、21 条 `2026-07-02` 相关时间口径信号；其中部分进入 raw preview / deliver preview 上下文。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / deliver preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-03 11:05 CST）

- 本轮 2026-07-03 11:00-15:10 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-03`
    - 同窗 heartbeat raw preview 继续出现 `2026年4月4日` / `April 4, 2026` 等与实际执行窗口不一致的时间口径。
    - 11:00-15:10 CST 统计可检出 5 条 `2026年4月4日`、3 条 `April 4, 2026`、37 条 `2026-07-02` 相关时间口径信号；其中部分进入 raw preview / deliver preview 上下文。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / deliver preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-03 07:00-11:05 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-03`
    - 同窗 heartbeat raw preview 继续出现 `2026-04-04` / `April 4, 2026` 等与实际执行窗口不一致的时间口径。
    - 11:00 CST 附近 `小米30港元破位预警` 生成 deliver preview，正文仍写出 `数据时间：2026年4月4日`；`TSLA 正负触发条件心跳监控` raw preview 又把当前时间推成 `2026-07-02`。
    - 结构化计数中同窗可检出 6 条 `2026-04-04` 类错误时间信号、2 条把当前时间写成 `2026-07-02` 的信号，并有 1 条错误日期进入 `deliver_preview`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw / deliver preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-02 23:03 CST）

- 本轮 2026-07-02 23:02-2026-07-03 03:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:30 CST `FOTO 光子学ETF心跳检测` raw preview 继续把当前检查时间写成 `2026-04-04 23:44:12`。
    - 02:00 CST `NVDA 关键事件心跳提醒` raw preview 继续把当前时间写成 `2026-04-04T12:37:43+08:00`，并围绕 2026-07-02 新闻与时间戳做错误新鲜度判断。
    - 02:00 CST `存储板块关键事件心跳提醒` raw preview 写出 `Current time: 2026-07-03T06:00:00+08:00`，与实际执行窗口不一致。
    - 03:01 CST `持仓财报与重大新闻心跳提醒` 与 `AAOI 1.6T 光模块心跳检测` raw preview 继续把 2026-07-02 / 2026-07 新闻判断为相对 `2026-04-06` / `2026-04-04` 的 future-dated 信息。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

- 本轮 2026-07-02 19:02-23:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 21:00 CST `全天原油价格3小时播报` raw preview 继续把系统当前时间写成 `2026-04-04T11:02:37.48+08:00`，与实际执行窗口不一致。
    - 21:00 / 21:30 / 22:00 CST `ORCL 大事件监控` raw preview 继续把当前时间判断为 `2026-04-04`，并据此把 2026-07-02 新闻误判为“未来日期/旧新闻”。
    - 22:30 CST `FOTO 光子学ETF心跳检测` 与 `全天原油价格3小时播报` raw preview 继续输出 `2026-04-04` 错误时间口径。
    - 23:00 CST `NBIS关键事件心跳提醒` raw preview 把当前时间推成 `2026-07-03 early morning`；23:01 CST `光模块板块关键事件心跳提醒` raw preview 又写出系统当前日期为 `2026-06-03`。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-02 19:03 CST）

- 本轮 2026-07-02 15:01-19:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现 `2026-04-04` 等与实际执行窗口不一致的时间口径。
    - 代表样本包括 `SIVE POET/Nokia/1.6T DFB 心跳检测`、`ORCL 大事件监控`、`TEM大事件心跳监控`、`NVDA 关键事件心跳提醒`，其中模型把当前时间或 quote timestamp 写成 2026-04-04，再进入 `JsonNoop`、`PlainTextSuppressed` 或后续判断链路。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-02 15:01 CST）

- 本轮 2026-07-02 11:01-15:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现 `2026-04-04`、`2026-07-03` 等与实际执行窗口不一致的时间口径。
    - `Monitor_Watchlist_11`、`持仓财报与重大新闻心跳提醒` 等样本把异常行情时间戳或旧系统日期带入当前触发判断。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-02 07:02 CST）

- 本轮 2026-07-02 03:02-07:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 03:30-06:30 CST 全天原油价格播报多次 raw preview 继续基于 `2026-04-04` 系统日期判断是否静默，与实际执行日 2026-07-02 不一致。
    - 05:30 CST Feishu `FOTO 光子学ETF心跳检测` raw preview 写出 `2026年4月4日 08:02 北京时间`。
    - 04:30 CST `AI与科技持仓观察关键事件心跳提醒` raw preview 也继续从错误系统时间上下文推断当前窗口。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-02 03:03 CST）

- 本轮 2026-07-01 23:01-2026-07-02 03:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 03:00 CST Feishu `小米30港元破位预警` raw preview 继续把 quote timestamp 解释为 `2026年7月3日14:00`，与实际执行日 2026-07-02 不一致。
    - 同窗 `ORCL 大事件监控`、`RKLB异动监控`、`NBIS关键事件心跳提醒` 等 raw preview 继续围绕异常 Unix timestamp 做时间推断，进入 `JsonNoop` / `PlainTextSuppressed` 判定上下文。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-01 23:02 CST）

- 本轮 2026-07-01 19:06-23:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 19:00 CST `小米30港元破位预警` raw preview 继续把 quote timestamp 解释为 2026-07-03，与实际执行日 2026-07-01 不一致。
    - 23:00 CST 同 job raw preview 仍写出“数据时间戳仍为 2026年7月3日14:00”；其它 heartbeat raw preview 也继续从旧 reminder 时间、错误 quote timestamp 或模型自述时间推断当前窗口。
  - 查重结论：
    - 本窗没有新的独立根因；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入触发判断、重复抑制或用户可见提醒新鲜度判断。没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1。

## 最新进展（2026-07-01 15:03 CST）

- 本轮 2026-07-01 11:02-15:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 11:30 CST Feishu `小米30港元破位预警` raw preview 写出系统时间为 `2026-07-03T14:00:00+08:00`，与实际执行日 2026-07-01 不一致。
    - 12:00 CST 同 job 又把 quote timestamp 解释为 `2026年7月3日约14:00 北京时间`；12:30 / 14:30 CST `全天原油价格3小时播报` raw preview 也继续基于 `2026-07-03 14:00` 判断是否静默。
    - 13:30 CST `持仓重大事件心跳检测` raw preview 明确指出 system prompt date 为 `2026-04-04`，14:00 CST `ORCL 大事件监控` raw preview 继续把当前时间推为 `2026-04-04`。
  - 查重结论：
    - 本窗没有新的正式送达错误日期样本；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入后续触发判断、重复抑制或用户可见提醒新鲜度判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-01 11:03 CST）

- 本轮 2026-07-01 07:02-11:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 08:30 CST Feishu `AAOI 1.6T 光模块心跳检测` raw preview 写出 `AAOI Current Status (as of 2026-04-04)`，与实际执行日 2026-07-01 不一致。
    - 同窗多条 heartbeat raw preview 仍从旧 reminder 时间、错误 quote timestamp 或模型自述时间推断当前窗口，进入后续 `JsonNoop` / `PlainTextSuppressed` 判断上下文。
  - 查重结论：
    - 本窗没有新的正式送达错误日期样本；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入后续触发判断、重复抑制或用户可见提醒新鲜度判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-01 07:01 CST）

- 本轮 2026-07-01 03:01-07:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 03:30 CST Feishu `FOTO 光子学ETF心跳检测` raw preview 写出 `triggered_at_check: "2026-04-04T14:30 CST"`，与实际执行窗口 2026-07-01 03:30 CST 不一致。
    - 04:00 CST Feishu `TSLA 正负触发条件心跳监控` raw preview 写出 `北京时间 2026-04-04 21:33`；同窗 `TEM大事件心跳监控` raw preview 写出 `当前时间 2025-08-19 09:20:00`。
    - 04:00 CST Feishu `持仓重大事件心跳检测` raw preview 写出 `Current system time: 2026-04-04T14:31:00+08:00`。
  - 查重结论：
    - 本窗没有新的正式送达错误日期样本；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入后续触发判断、重复抑制或用户可见提醒新鲜度判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-01 03:01 CST）

- 本轮 2026-06-30 23:00-2026-07-01 03:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:00 CST Web `闪迪关键事件心跳提醒` `job_id=j_19dd9a1e` raw preview 写出 `2026-04-04 08:01 北京时间`，与实际执行日 2026-06-30 不一致；该样本最终为 `PlainTextSuppressed + execution_failed`，未确认正式送达。
    - 02:00 CST Web `持仓重大事件心跳提醒` raw preview 写出 `System date context: 2026-04-04`，并基于错误系统日期继续综合 MU 财报、SNDK / GEV / NVDA / AAPL 等事件；该样本未确认正式送达，仅作为时间上下文漂移辅助信号。
  - 查重结论：
    - 本窗没有新的正式送达错误日期样本；上述 raw preview 仍属于 heartbeat 模型时间上下文漂移，与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路仍可运行，但错误时间上下文可能进入后续触发判断、重复抑制或用户可见提醒新鲜度判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 23:01 CST）

- 本轮 2026-06-30 19:02-23:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 21:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `行情（数据时间：2026年7月3日）`，与实际执行日 2026-06-30 不一致。
    - 22:30 CST `全天原油价格3小时播报` raw preview 写出 `2026年7月3日 09:16 北京时间，非检查时间节点，静默`；23:00 CST `闪迪关键事件心跳提醒` raw preview 又写出 `2026-04-04 08:01 北京时间`。这两条未确认正式送达，仅作为时间上下文漂移辅助信号。
  - 查重结论：
    - 小米样本仍属于 heartbeat 成功生成触发提醒后的用户可见数据日期 / 执行日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-30 03:07 CST）

- `crates/hone-channels/src/scheduler.rs` 的 heartbeat 出站归一化新增 `今日（M月D日）` 口径修正：当触发提醒正文把“今日”括号日期写成与 scheduler 权威北京时间不一致的未来/错误日期时，会在送达前自动改写为当前北京时间日期。
- 该修复与既有 `北京时间 YYYY-MM-DD HH:MM` / `北京时间 HH:MM` 归一化并行生效，避免同一提醒同时把绝对检查时间和“今日（…）”相对日期写错。
- 新增回归 `heartbeat_normalizes_conflicting_relative_today_date`，覆盖 `今日（6月30日）` 在 `2026-06-29T13:00:21+08:00` 执行窗口内被归一为 `今日（6月29日）`。
- 验证通过：
  - `cargo check -p hone-channels --tests`
  - `cargo test -p hone-channels heartbeat_normalizes_conflicting_relative_today_date --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- 本轮未重启当前 live runtime；线上送达预览是否完全止血仍待后续巡检窗口复核，因此先更新为代码级 `Fixed`，不直接标 `Closed`。

## 最新进展（2026-06-30 15:02 CST）

- 本轮 2026-06-30 15:02-19:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `数据时间 2026年7月3日`，与实际执行日 2026-06-30 不一致。
    - 16:30 CST 同 job 再次生成 `deliver_preview`，正文继续写 `行情（数据时间：2026年7月3日，数据戳 1782806885）`。
    - 19:00 CST Feishu `ORCL 大事件监控` raw preview 写出 `Current time: 2026-04-04 22:17:16 CST (Saturday night)`；该样本最终为 `JsonNoop` 且未送达，本轮仅作为时间上下文漂移辅助信号。
  - 查重结论：
    - 小米样本仍属于 heartbeat 成功生成触发提醒后的用户可见数据日期 / 执行日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

- 本轮 2026-06-30 11:02-15:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 13:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `今日（7月4日，北京时间 14:40）低开低走`，与实际执行日 2026-06-30 不一致。
    - 13:30 / 14:00 CST 同 job 后续预览改写为 `数据截至 2026 年 6 月 26 日 18:35 北京时间`，说明正文仍混用执行日期、数据日期与模型推断日期；15:00 CST 再次生成 `今日（6月30日）`，相对日期口径在同一 job 内漂移。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 07:03 CST）

- 本轮 2026-06-30 03:00-07:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 06:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，正文仍写出 `今日（6月30日）高开高走`。
    - 07:00 CST 同 job 的 `duplicate_suppressed` 继续匹配带 `今日（6月30日）` 的旧 preview。
  - 03:12 CST 非文档提交 `a00e5131 fix: harden heartbeat noop compatibility` 已包含 `今日（M月D日）` 归一化回归，但本窗 live 日志仍出现同类错误日期 preview；当前按运行态 `New` 处理，不能关闭。
  - 调度 / 解析 / 预览生成链路可用，但用户可见提醒新鲜度和交易日判断仍可能被误导；没有错投、数据安全或全渠道不可用证据，因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 03:07 CST）

- 本轮 2026-06-29 23:00-2026-06-30 03:07 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口仍为 2026-06-29。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致；随后 `duplicate_suppressed` 也匹配到同一错误日期预览，说明错误时间口径会进入重复抑制判断基线。
    - 00:00 CST 同 job 再次生成 `deliver_preview`，正文写成 `今日（7月1日）高开高走`，继续与实际执行日不一致。
    - 03:00 CST Web `持仓关键事件心跳检测` raw preview 还把检查窗口写成 `北京时间 2026-05-30`，但该样本最终为 `PlainTextNoop` 且未送达，本轮仅作为时间上下文漂移的辅助信号，不单独升严重级别。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 23:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 21:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致；随后 `duplicate_suppressed` 也匹配到同一错误日期预览，说明错误时间口径会进入重复抑制判断基线。
    - 23:00 CST Web `中际旭创关键事件心跳提醒` raw preview 内部还把系统时间写成 `2026-04-04 15:00 CST`，但该样本最终为 `JsonNoop` 且未送达，本轮仅作为时间上下文漂移的辅助信号，不单独升严重级别。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 19:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:00 / 17:30 / 18:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29。
    - 15:00 CST `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`；17:30 CST 又写成 `今日（7月4日）高开高走`；18:00 CST 再写成 `今日（6月30日）高开高走`，均与实际执行日 2026-06-29 不一致。
    - 该样本已进入送达预览文本；同窗还可见同 job 的触发 / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 15:07 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-29` 与 `data/runtime/logs/hone_cli_screen.log`
    - 15:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29 15:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致。
    - 该样本已进入送达预览文本；同窗还可见同 job 的触发 / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 03:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 03:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29 03:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（7月3日）成交量约 1.92 亿股`，与实际执行日 2026-06-29 不一致。
    - 随后的 `duplicate_suppressed` 又匹配到旧预览中的 `今日（7月2日）`，说明错误日期口径会进入重复抑制判断基线。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
    - 本轮同时存在 `小米30港元破位预警` triggered / noop / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`；本单只跟踪已生成提醒的日期口径错误。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-28 19:02 CST）

- 本轮最近四小时真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 17:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成成功送达预览，当前日志窗口为 2026-06-28 17:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（7月1日）低点 21.30 港元已刷新 52 周新低`，与实际执行日 2026-06-28 不一致。
    - 同一轮 raw preview 还包含 `timestamp":"2026-07-02T12:48:09+08:00"`；这说明时间 / 日期口径不只出现在标题型“北京时间”归一化范围内，数据日期与正文“今日”也可能以未来日期进入用户可见提醒。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功投递后用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
    - 本轮同时存在 `小米30港元破位预警` triggered / noop / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`；本单只跟踪已送达提醒的日期口径错误。
  - 用户影响：
    - 调度、解析、投递链路成功，用户能收到提醒。
    - 但用户可见正文把 6 月 28 日执行窗口写成 7 月 1 日，容易误判提醒新鲜度和交易日；没有错投、漏投、数据安全或全渠道不可用证据。
    - 因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-21 23:07 CST）

- 本轮补齐 2026-06-21 19:03 回退样本的日期型标题归一化：
  - `normalize_heartbeat_beijing_trigger_time(...)` 现在不仅处理 `北京时间 HH:MM ...触发`，也处理 `【...监控 · 北京时间 YYYY-MM-DD HH:MM】` 这类 heartbeat 标题时间。
  - 归一化仍限制在 `监控 / 检查 / 心跳 / 任务 / 触发` 上下文，避免把普通数据时间误写成执行时间。
  - 命中日期型触发时间后会把标题改写为 scheduler 权威北京时间日期和分钟，并在 metadata 保留 `beijing_trigger_time_normalized=true` 与原始 `YYYY-MM-DD HH:MM`。
  - 新增回归 `heartbeat_normalizes_conflicting_beijing_trigger_datetime_title`，覆盖 `NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30` 在 2026-06-21 19:01 CST 执行窗口内被归一到 `北京时间 2026-06-21 19:01`。
- 验证：
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- 无关联 GitHub Issue；当前按本地代码和回归验证更新为 `Fixed`，未依赖当前机器生产日志、线上渠道状态或 live 服务重启复核。

## 最新进展（2026-06-21 19:03 CST）

- 本轮最近四小时真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-21`
    - 19:00:02 CST Web heartbeat `NBIS关键事件心跳提醒` 启动，target=`web-user-c2776780c59d`。
    - 19:01:02 CST 同 job 收口为 `success=true`、`parse_kind=JsonTriggered`。
    - 19:01:02 CST deliver preview 开头写成 `【NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30】`。
  - 该送达标题时间与实际调度执行窗口 `2026-06-21 19:01 CST` 明显不一致，且 raw preview 中模型自行推断 `It's approximately 17:30 Beijing time on June 19, 2026`。
  - 代码对照显示当前调度路径仍调用 `heartbeat_execution_from_content(&content, &heartbeat_model)`，没有把 scheduler 当前北京时间传入 `heartbeat_execution_from_content_at_beijing(...)`，因此 2026-05-29 的触发时间归一化修复没有覆盖这条 live 出站路径。
- 用户影响：
  - 调度、解析、投递链路成功，用户能收到提醒；但用户可见标题把提醒时间写早两天，容易误判提醒新鲜度和交易时段。
  - 该问题不涉及错投、漏投、数据安全或系统级失败，因此保持 `P3 / New`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-05-29 16:35 CST）

- 已修复 heartbeat 用户可见触发时间口径漂移：heartbeat prompt 现在显式注入“本轮权威检查时间（北京时间）”，并要求 `message` 中的检查/触发时间必须使用该权威时间；市场时段、数据时间或美东盘前/盘后不得写成另一个“北京时间触发”。
- 出站前新增轻量归一化：若 `JsonTriggered` 正文出现类似 `北京时间 HH:MM ...监控/检查/心跳/任务触发`，且该时间与 scheduler 当前北京时间不一致，会把该触发时间归一到 scheduler 权威检查时间，并在 metadata 中记录 `beijing_trigger_time_normalized=true` 与原始时间。
- 回归验证：`cargo test -p hone-channels heartbeat_normalizes_conflicting_beijing_trigger_time --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_ --lib -- --nocapture` 通过。
- 状态更新为 `Fixed`；后续如当前 HEAD 运行态仍出现 heartbeat 把美东/UTC/数据时间错误标成“北京时间触发”，再用新样本重新打开。

## 证据来源

- `data/runtime/logs/web.log.2026-06-21`
  - 巡检窗口：2026-06-21 15:03-19:03 CST。
  - 19:00:02 CST `NBIS关键事件心跳提醒` 触发，target=`web-user-c2776780c59d`。
  - 19:01:02 CST `run_finish job_id=j_eab1a3b2 job=NBIS关键事件心跳提醒 ... success=true content_chars=4179`，随后 `parse_kind=JsonTriggered`。
  - 同一秒 `deliver_preview` 开头为 `【NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30】`，但本轮运行日志时间为 2026-06-21 19:01 CST。
- `crates/hone-channels/src/scheduler.rs`
  - 当前调度路径在 heartbeat 内容收口后调用 `heartbeat_execution_from_content(&content, &heartbeat_model)`。
  - 带权威北京时间的 `heartbeat_execution_from_content_at_beijing(...)` 只在测试 / helper 路径出现，未接入本条 live 调度路径。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=36255`
  - `job_id=j_bb4bbb99`
  - `job_name=AI与科技持仓观察关键事件心跳提醒`
  - `actor_channel=web`
  - `executed_at=2026-05-29T11:31:32.698046+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `response_preview` / `detail_json.scheduler.deliver_preview` 开头写成 `2026年5月29日 北京时间 04:00 盘后监控触发。已核验事实...`
- 最近四小时巡检窗口 `2026-05-29 11:02-15:03 CST`
  - 按消息时间共有 47 个 user turn 与 47 个 assistant final，最新活跃会话均已 assistant final 收口。
  - 普通 scheduler 2 条 `completed + sent + delivered=1`，未见 `commodity_causality_guarded=true`。
  - Heartbeat 新增 1 条 `completed + sent + delivered=1`、81 条 `execution_failed + skipped_error + delivered=0`、40 条 `noop + skipped_noop + delivered=0`。
  - Assistant final 污染扫描未命中空回复、本机绝对路径、`rawOutput`、`tool_call`、`session/update`、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400 Bad Request` 或 `open_id cross app`。

## 端到端链路

1. Web heartbeat scheduler 在 `2026-05-29 11:31 CST` 执行 `AI与科技持仓观察关键事件心跳提醒`。
2. Heartbeat runner 返回 `JsonTriggered`，scheduler 将结果落成 `completed + sent + delivered=1`。
3. 送达正文开头却把触发时间写为 `北京时间 04:00`。
4. 该时间与 `cron_job_runs.executed_at=2026-05-29T11:31:32+08:00` 不一致，用户可见提醒的时间口径错误。

## 期望效果

- Heartbeat 触发提醒应使用调度器权威执行时间或明确的数据时间字段，不能把 UTC 时间、市场时段说明或模型推断时间写成“北京时间”。
- 如果正文需要区分数据时间、交易时段与触发时间，应分别标注，例如“执行时间”“数据口径时间”“美东盘后”。

## 当前实现效果

- 本轮 heartbeat 内容已成功触发并送达，但用户可见首句把实际 `11:31 CST` 执行写成 `北京时间 04:00`。
- 当前证据只覆盖一条 Web heartbeat 成功送达样本；同窗直聊与普通 scheduler 没有同类时间口径污染。

## 用户影响

- 用户看到的 heartbeat 触发时间与系统实际执行时间不一致，可能误判提醒的新鲜度和所处交易时段。
- 该问题不影响主功能链路：任务有正常执行、解析、落库和送达；没有错误投递对象、没有漏发、没有把工具原始输出暴露给用户，也没有直接给出错误交易指令。
- 因此本轮定级为 P3：它是用户可见输出质量 / 时间口径问题，而不是调度、投递、数据安全或交易正确性链路失效。

## 根因判断

- 初步判断是 heartbeat 模型在生成 `JsonTriggered` 正文时把 UTC 时间、市场时段或内部数据时间错误表述为“北京时间”。
- Scheduler 送达前目前没有校验触发正文里的显式北京时间是否与 `executed_at` 一致，也没有强制区分执行时间和数据时间。

## 下一步建议

- 在 heartbeat prompt 或输出 schema 中显式传入并要求使用 `executed_at_beijing`，同时禁止模型自行换算“北京时间”。
- 在 scheduler 出站前增加轻量校验：若 `JsonTriggered` 正文出现“北京时间 HH:MM”且与 `executed_at` 偏差明显，降级为待复核或重写时间口径。
- 后续巡检优先观察其它 `JsonTriggered + delivered=1` heartbeat 是否继续出现类似 UTC/CST 混淆，再决定是否提升严重等级。

## 最新运行态复核（2026-06-28 23:02 CST）

- `data/runtime/logs/web.log.2026-06-28` / `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-06-28 19:02-23:02 CST。
  - 20:00 CST `小米30港元破位预警` 生成 `JsonTriggered + deliver_preview`，但 preview 把当前 2026-06-28 执行窗口写成 `今日（7月3日）`。
  - 21:00 CST 同一 job 再次生成 `JsonTriggered + deliver_preview`，preview 把当前执行窗口写成 `今日（7月2日）`，随后因重复抑制未正式发送。
  - 该样本与 17:00 CST 的 `今日（7月1日）` 同根，均为 heartbeat 触发提醒把数据日期 / 模型推断日期写成用户可见“今日”口径。
- 本轮判断
  - 最新证据仍属于 heartbeat 成功生成触发提醒后的日期 / 时间口径错误，不新建重复缺陷。
  - 调度和解析链路仍可运行，问题主要影响用户对提醒新鲜度和交易日的判断，因此维持质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新运行态复核（2026-07-01 19:06 CST）

- `data/runtime/logs/web.log.2026-07-01`
  - 巡检窗口：2026-07-01 15:00-19:05 CST。
  - 15:30 CST `小米30港元破位预警` 生成 `JsonTriggered + deliver_preview`，在 2026-07-01 执行窗口内继续把行情数据时间写成 `约对应 2026 年 7 月 3 日 14:00 北京时间`。
  - 19:01 CST `AAOI 1.6T 光模块心跳检测` 经 `BudgetRecovery { reason: ContextOverflow }` 恢复后生成 deliver preview，标题写成 `AAOI 1.6T 批量订单确认 — 2026-07-01 北京时间 19:00 检查`，正文核心事件却是 `2026年3月9日` 官方公告，容易把旧事件包装成当前检查时点的新触发。
- 本轮判断
  - 最新证据仍属于 heartbeat 触发提醒的时间 / 日期口径错误，不新建重复缺陷。
  - 调度、解析和投递链路仍可运行；问题主要影响用户对提醒新鲜度和事件时点的判断，因此维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 最新运行态复核（2026-07-02 11:01 CST）

- `data/runtime/logs/web.log.2026-07-02`
  - 巡检窗口：2026-07-02 07:01-11:01 CST。
  - 10:30 CST `AAOI 1.6T 光模块心跳检测` 经 `BudgetRecovery { reason: ContextOverflow }` 后生成 deliver preview，正文以当前检查时间包装旧订单事实，仍容易把 2026-03-09 旧公告表达成当前触发事实。
  - 11:00 CST `NVDA 关键事件心跳提醒` raw preview 把当前检查口径写成 `2026年4月4日 15:05 北京时间`。
  - 11:00 CST `TEM大事件心跳监控` raw preview 把工具时间戳解释为 `2026-04-09` 并称这是系统给定当前时间。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-03 07:00 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-03 03:00-07:00 CST。
  - 本窗至少 10 条 heartbeat raw preview 继续出现错误当前时间或日期口径。
  - 代表样本包括 `光迅科技关键事件心跳提醒` 把当前检查写成 `2026年4月4日 10:30（北京时间）`，`TEM大事件心跳监控` 把当前时间写成 `April 4, 2026, 09:43 Beijing time`，以及 `TSLA 正负触发条件心跳监控` 把当前时间推断为 `April 30, 2026`。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-04 03:05 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-03 23:02-2026-07-04 03:05 CST。
  - 本窗 heartbeat raw / deliver preview 继续出现错误当前时间或日期口径。
  - 代表样本包括 `美股黄金坑信号心跳检测` 把 market data 写成 `April 2026`，`光模块板块关键事件心跳提醒` 在 2026-07-04 03:01 CST 执行窗口中写出 `current date of January 27, 2026`，以及多条 heartbeat 将 quote timestamp 自行推断成与调度执行时点不一致的当前口径。
- `data/sessions.sqlite3`
  - 同窗 00:00-00:05 Feishu scheduler final 的显式北京时间口径与触发窗口基本一致，未见同类错误进入这 3 条 assistant final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-04 11:01 CST）

- `data/runtime/logs/web.log.2026-07-04` / `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-04 07:01-11:01 CST。
  - 本窗 heartbeat raw / deliver preview 继续出现错误或混乱的当前时间口径。
  - 代表样本包括 `全天原油价格3小时播报` 在 08:00、09:00、10:30 CST 执行窗口中仍把系统时间读成 `2026-07-04T02:00` 或 `01:57`；`小米30港元破位预警` 在 07:30-11:00 CST 窗口继续基于 2026-07-03 HK 18:28 前后行情时间输出触发判断。
- `data/sessions.sqlite3`
  - 同窗 Feishu scheduler final 的显式北京时间口径与触发窗口基本一致，未见同类错误进入这 3 条 assistant final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-04 15:02 CST）

- `data/runtime/logs/web.log.2026-07-04`
  - 巡检窗口：2026-07-04 11:02-15:02 CST。
  - 本窗 heartbeat raw / deliver preview 继续出现错误或混乱的当前时间口径。
  - 代表样本包括 `美股黄金坑信号心跳检测` 把指数数据时间戳推断成 `2026年12月`，`持仓重大事件心跳检测` 在 13:30 CST 执行窗口中写出 `2026年4月4日 23:10 北京时间`，以及部分 heartbeat 把当前检查口径推断为 2026-07-03 或更早新闻日期。
- `data/sessions.sqlite3`
  - 同窗 Feishu direct final 的显式北京时间口径与用户请求窗口基本一致，未见同类错误进入该 assistant final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-04 23:02 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-04 19:01-23:02 CST。
  - 本窗 heartbeat raw / deliver preview 继续出现错误或混乱的当前时间 / 数据时间口径。
  - 代表样本包括多条 heartbeat 围绕 `1783022400` 自行换算交易日期，`中际旭创关键事件心跳提醒` 将 `1783062316` 推断为约 `2026-04`，以及 `存储板块关键事件心跳提醒` 把 `SNDK` snapshot 时间口径写成 `July 3/4` 混用。
- `data/sessions.sqlite3`
  - 同窗 20:00 Web scheduler、21:35 / 23:00 Feishu scheduler final 均正确识别 2026-07-04 为周六 / 美股休市；未见同类错误进入这 3 条 assistant final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-05 19:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-05 15:00-19:02 CST。
  - 本窗 heartbeat raw preview 继续出现错误或混乱的当前时间 / 数据时间口径。
  - 代表样本包括 18:30 CST `TSLA 正负触发条件心跳监控` 把当前检查写成 `July 6, 2026 (Monday, Beijing time)`，19:00 CST 同 job 继续写 `Current date: July 6, 2026`。
  - 18:30 CST `持仓重大事件心跳检测` raw preview 把系统时间写成 `July 4, 2026 20:25`，与 2026-07-05 18:30 CST 执行窗口不一致。
  - `全天原油价格3小时播报` 与 `伦敦金跌破4100提醒` 多次围绕 `1783098008` / `1783209600` 自行换算，仍把数据时间、执行时间和交易时段混在同一判断里。
- `session_messages`
  - 同窗 4 条 assistant final 的显式时间口径与用户请求或调度窗口基本一致，未见同类错误进入这些 final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-12 15:01 CST）

- `data/runtime/logs/web.log.2026-07-12`
  - 巡检窗口：2026-07-12 11:00-15:01 CST。
  - 本窗 82 条 heartbeat raw / deliver preview 命中明显错误或互相矛盾的当前时间 / 数据时间口径。
  - 代表样本包括 11:00 CST `Cerebras IPO与业务进展心跳监控` raw / deliver preview 把当前检查写成 `2026-07-13`；12:00 CST `FOTO 光子学ETF心跳检测` 和 `ORCL 大事件监控` 把实际周日中午窗口写成 `2026-07-13 周一`；15:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` deliver preview 写成 `北京时间 2026-07-14`。
  - 同窗多条 preview 还把 2026-07-12 周日窗口写成周一盘前 / 常规交易时段，或混用 `2026-07-10` 收盘快照与未来检查时间。
- `data/sessions.sqlite3`
  - 11:00-15:01 CST 按真实 `timestamp` 没有新增 assistant final；本轮未确认同类时间口径错误进入新的普通 direct / scheduler final。
- 本轮判断
  - 最新证据仍属于 heartbeat 时间 / 日期口径错误，不新建重复缺陷。
  - 调度与解析链路仍可运行，问题主要影响用户对提醒新鲜度和事件时点的判断；维持质量性 `P3 / New`，非 P1。
