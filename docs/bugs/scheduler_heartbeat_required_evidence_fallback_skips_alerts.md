# Bug: Heartbeat / scheduler 实时核验门禁失败后批量跳过提醒

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

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-28 06:01-10:02 CST。
  - 同窗 source log 统计 `HeartbeatDiag=207`、`run_start=56`、`run_finish=56`、`deliver=29`、`duplicate_suppressed=10`，但仍有 `execution_failed=7` 与 `runner_error=6`；代表样本包括 07:00 CST `存储板块关键事件心跳提醒` 因 `persistent_tool_failure: read-after-write reconciliation failed` 跳过发送，07:30 CST `光模块板块关键事件心跳提醒` 因 `heartbeat 输出不是结构化 JSON` 跳过发送，09:00 / 10:00 CST 多个 heartbeat 因 MiniMax / OpenAI-compatible HTTP 529 provider 错误跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=58`、`JsonNoop=18`、`JsonTriggered=2`、`PlainTextSuppressed=1`、`JsonEmptyStatus=1`，说明 heartbeat 仍在自然语言触发、结构化 noop、空状态、suppressed 和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / persistent-tool / provider / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-27 18:02-22:02 CST。
  - 同窗 source log 统计 `HeartbeatDiag=236`、`run_start=71`、`run_finish=71`、`deliver=16`、`duplicate_suppressed=4`，但仍有 `execution_failed=28` 与 HTTP 529 相关信号 3 条；代表样本包括 18:30-19:30 CST 多个 heartbeat 因 OpenAI-compatible provider transport error 成批跳过发送，21:00 CST `持仓财报与重大新闻心跳提醒` 因 HTTP 529 跳过发送，21:00 CST `光模块板块关键事件心跳提醒` 因 `heartbeat 输出不是结构化 JSON` 跳过发送，21:30 CST `持仓重大事件心跳提醒` 因 `persistent_tool_failure: read-after-write reconciliation failed` 跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=32`、`JsonNoop=11`、`JsonTriggered=4`、`JsonEmptyStatus=1`、`PlainTextSuppressed=1`，说明 heartbeat 仍在自然语言触发、结构化 noop、空状态、suppressed 和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / persistent-tool / provider / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-27 10:01-14:01 CST。
  - 同窗 source log 统计 `HeartbeatDiag=217`、`run_start=56`、`run_finish=57`、`deliver=32`、`duplicate_suppressed=15`，但仍有 `execution_failed=4` 与 HTTP 529 相关信号 3 条；代表样本包括 12:00 CST `持仓财报与重大新闻心跳提醒` 因 OpenAI-compatible upstream HTTP 529 provider 错误跳过发送，以及 12:00 / 12:31 / 13:30 / 14:01 CST 多条 heartbeat 因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=62`、`JsonNoop=16`、`PlainTextSuppressed=4`、`JsonTriggered=3`、`JsonEmptyStatus=2`、`PlainTextNoop=1`，说明 heartbeat 仍在自然语言触发、结构化 noop、空状态、suppressed 和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / provider / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-27 06:00-10:01 CST。
  - 同窗 source log 统计 `HeartbeatDiag=210`、`run_start=56`、`run_finish=61`、`deliver=27`、`duplicate_suppressed=5`，但仍有 `execution_failed=5` 与 HTTP 529 相关信号 9 条；代表样本包括 09:30 CST `光模块板块关键事件心跳提醒` 因 `persistent_tool_failure: read-after-write reconciliation failed` 跳过发送，以及 10:00 CST 多个 heartbeat 因 OpenAI-compatible upstream HTTP 529 provider 错误跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=54`、`JsonNoop=20`、`PlainTextSuppressed=5`、`PlainTextNoop=3`、`JsonTriggered=1`，说明 heartbeat 仍在自然语言触发、结构化 noop、suppressed 和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / persistent-tool / provider / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-26 18:00-22:01 CST。
  - 同窗 source log 统计 `HeartbeatDiag=213`、`run_start=58`、`run_finish=58`、`deliver=29`、`duplicate_suppressed=8`，但仍有 `execution_failed=3`；代表样本包括 18:00 CST `光模块板块关键事件心跳提醒` 因 `heartbeat 输出包含未知状态，任务已标记失败` 跳过发送，以及 22:00 CST `持仓重大事件心跳提醒` 因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=58`、`JsonNoop=19`、`JsonUnknownStatus=4`、`PlainTextNoop=2`、`JsonEmptyStatus=2`、`JsonTriggered=1`、`PlainTextSuppressed=1`，说明 heartbeat 仍在自然语言触发、结构化 noop、空状态、未知状态和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-26 10:01-14:02 CST。
  - 同窗 source log 统计 `HeartbeatDiag=201`、`run_start=56`、`run_finish=56`、`deliver=24`、`duplicate_suppressed=9`，但仍有 `execution_failed=4`；代表样本包括 10:30、11:00、12:00 CST `持仓重大事件心跳提醒` 多轮因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 落成 `failure_kind=execution_failed` 并跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=48`、`JsonNoop=25`、`PlainTextSuppressed=4`、`PlainTextNoop=2`，说明 heartbeat 仍在自然语言触发、结构化 noop、suppressed 和非结构化失败之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-26 06:01-10:05 CST。
  - 同窗 source log 统计 `HeartbeatDiag=218`、`run_start=56`、`run_finish=58`、`deliver=31`、`duplicate_suppressed=13`，但仍有 `execution_failed=7`；代表样本包括 `持仓重大事件心跳提醒` 多轮 `success=true` 生成自然语言后因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 落成 `failure_kind=execution_failed` 并跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=58`、`JsonNoop=14`、`JsonTriggered=5`、`JsonUnknownStatus=4`、`PlainTextSuppressed=5`、`PlainTextNoop=2`，说明 heartbeat 仍在自然语言触发、结构化 noop、未知状态和协议载荷之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-26 02:01-06:01 CST。
  - 同窗 source log 统计 `HeartbeatDiag=212`、`run_start=56`、`run_finish=58`、`deliver=31`、`duplicate_suppressed=9`，但仍有 `execution_failed=14`；代表样本包括 `持仓重大事件心跳提醒` 先 `success=true` 生成 2608 chars，随后因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 落成 `failure_kind=execution_failed` 并跳过发送。
  - 同窗 parse 分布为 `PlainTextTriggered=60`、`JsonNoop=19`、`PlainTextSuppressed=5`、`PlainTextNoop=3`、`JsonTriggered=2`，说明 heartbeat 仍在自然语言触发、结构化 noop 和协议载荷之间漂移。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-23 10:02-14:02 CST。
  - 12:00 CST `持仓重大事件心跳提醒` 与 14:00 CST `存储板块关键事件心跳提醒` 均记录 `heartbeat 输出不是结构化 JSON，任务已标记失败`，随后 Web events 以 `failure_kind=execution_failed` 跳过发送。
  - 同窗 source runtime 仍有 `HeartbeatDiag=1209`、`run_start=57`、`run_finish=57`、`deliver=57`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message/cron 增量。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-23 02:01-06:02 CST。
  - 同窗 `HeartbeatDiag=207`、`run_start=56`、`run_finish=57`、`deliver=31`，但仍出现 1 条 runner / execution 类失败信号，且 parse 分布包含 `Empty=1`；多轮 heartbeat 在工具预算受限后依赖旧报价或旧轮次信息收口，说明 required-evidence / 输出契约 fail-closed 风险仍未消失。
  - 本轮未见批量 `heartbeat 输出不是结构化 JSON` 明文复发，但 06:00 CST `光模块板块关键事件心跳提醒` 仍尝试调用不存在的 `cron_job` 工具，随后把“要确认监控关系”的直聊式文案作为 heartbeat deliver 候选并被 duplicate suppression 压掉，和既有 heartbeat 执行期任务语义漂移 / evidence fail-closed 问题同链路。
  - 判断：该样本仍属于 heartbeat required-evidence / 工具预算 / 输出契约不稳定后监控轮次降级或跳过的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-22 22:02-2026-08-23 02:01 CST。
  - 23:00、23:30、01:30、02:00 CST 各有 heartbeat 轮次因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 落成 `failure_kind=execution_failed` 并跳过发送，覆盖 `NVDA 关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 等任务。
  - 同窗 source runtime 仍有 `HeartbeatDiag=215`、`run_start=57`、`run_finish=58`、`deliver=31`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message/cron 增量。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-21 18:01-22:02 CST。
  - 20:00 CST `NVDA 关键事件心跳提醒` 落成 OpenAI-compatible upstream HTTP 529 `provider_http_error`，随后 Web events 记录 `定时任务执行失败，跳过发送`；同窗另有 1 条 `heartbeat 输出不是结构化 JSON`，说明 heartbeat / scheduler 在实时核验、provider 或输出契约失败后仍会 fail-closed 到跳过发送。
  - 同窗 source runtime 仍有 `HeartbeatDiag=207`、`run_start=56`、`run_finish=56`、`deliver=27`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message/cron 增量。
  - 判断：该样本仍属于 heartbeat required-evidence / provider / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-13 06:01-10:02 CST。
  - 同窗仍有 3 条 `execution_failed / skipped_error / 任务已标记失败` 类信号，覆盖 heartbeat 非结构化 / 空状态输出后跳过发送；近窗 parse 分布仍含 `PlainTextSuppressed=2`、`JsonUnknownStatus=2`、`JsonEmptyStatus=1`。
  - 同窗 source runtime 仍有 `run_start=64`、`run_finish=64`、`deliver=32`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message/cron 增量。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-13 02:00-06:02 CST。
  - 02:00 / 02:30 / 04:30 / 05:30 CST 多条 heartbeat 记录 `定时任务执行失败，跳过发送`，错误为 `heartbeat 输出不是结构化 JSON，任务已标记失败`；覆盖 `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒`、`AAPL + NVDA + BE 关键事件提醒`、`AI与科技持仓观察关键事件心跳提醒` 等任务。
  - 同窗 source runtime 仍有 `run_start=64`、`run_finish=71`、`deliver=43`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message/cron 增量。
  - 判断：该样本仍属于 heartbeat required-evidence / 输出结构化契约 fail-closed 后用户无法获得本轮监控正文或只看到失败路径的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-12 14:00-18:02 CST。
  - 同窗有 3 条 heartbeat 轮次因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 或同类 `failure_kind=execution_failed` 跳过发送。
  - 同窗 source runtime 仍有 `HeartbeatDiag=271`、`run_start=72`、`run_finish=72`、`deliver=42`，说明不是 Web scheduler / event-engine 全链路不可用；本轮未见 `persistent_tool_failure`、HTTP 429 / 402 或 OpenAI-compatible stream decode 复发。
  - 判断：这些样本仍属于“实时核验 / 输出契约 / runner fail-closed 后监控轮次跳过”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-12 10:01-14:01 CST。
  - 13:00 CST `持仓重大事件心跳提醒` 与 `AAPL + NVDA + BE 关键事件提醒` 均落成 `heartbeat 输出不是结构化 JSON，任务已标记失败`，随后 Web events 记录 `failure_kind=execution_failed` 并跳过发送。
  - 同窗 source runtime 仍有 `HeartbeatDiag=238`、`run_start=64`、`run_finish=65`、`deliver=35`，说明不是 Web scheduler / event-engine 全链路不可用；本轮未见 `persistent_tool_failure` 复发。
  - 判断：这些样本仍属于“实时核验 / 输出契约 / runner fail-closed 后监控轮次跳过”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-12 06:01-10:02 CST。
  - 06:30 CST `持仓财报与重大新闻心跳提醒` 与 09:00 CST `光模块板块关键事件心跳提醒` 均落成 `persistent_tool_failure: read-after-write reconciliation failed`，随后 Web events 记录 `failure_kind=runner_error` 或执行失败并跳过发送。
  - 同窗 06:30 / 08:30 / 09:30 / 10:00 CST 另有 4 轮 heartbeat 因 `heartbeat 输出不是结构化 JSON，任务已标记失败` 跳过发送；source runtime 仍有 `HeartbeatDiag=239`、`run_start=64`、`run_finish=65`、`deliver=38`，说明不是 Web scheduler / event-engine 全链路不可用。
  - 判断：这些样本仍属于“实时核验 / persistent-tool / runner fail-closed 后监控轮次跳过”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-10 14:02-18:02 CST。
  - 17:30 CST `持仓重大事件心跳提醒` 多次落成 `persistent_tool_failure: read-after-write reconciliation failed`，随后 Web events 记录 `failure_kind=runner_error` 或执行失败并跳过发送。
  - 16:30 CST `NVDA 关键事件心跳提醒` 因 OpenAI-compatible HTTP 529 provider 错误跳过发送；同窗 source runtime 仍有 `HeartbeatDiag` 相关行 359 条、`deliver=51`，说明不是 Web scheduler / event-engine 全链路不可用。
  - 判断：这些样本仍属于“实时核验 / persistent-tool / provider runner fail-closed 后监控轮次跳过”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-10 06:00-10:02 CST。
  - 08:00 CST `光模块板块关键事件心跳提醒` 与 08:30 CST `光迅科技关键事件心跳提醒` 均落成 `persistent_tool_failure: read-after-write reconciliation failed`，随后 Web events 只记录 `failure_kind=runner_error` 并跳过发送。
  - 09:32 CST `AI与科技持仓观察关键事件心跳提醒` 又因 OpenAI-compatible stream body decode runner error 跳过发送；同窗 source runtime 仍有 `run_start=96`、`run_finish=105`、`deliver=57`，说明不是 Web scheduler / event-engine 全链路不可用。
  - 判断：这些样本仍属于“实时核验 / persistent-tool / runner fail-closed 后监控轮次跳过”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-05 02:00-06:00 CST。
  - 05:05 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 记录 `[MsgFlow/web] failed ... error="数据时间：北京时间 2026-08-05 05:05；行情口径：报价源时间：北京时间 2026-08-05 04:58 至 2026-08-05 04:59...`；错误字段是一段 SOXX / SMH 投研正文开头，而不是结构化失败分类。
  - 随后仅记录 `step=session.persist_assistant ... detail=failed` 与 `recovered read-only failure answer ... failure_kind=internal_error_suppressed chars=447`；同窗 source runtime 仍有 `run_start=110`、`run_finish=110`、`deliver=57`，说明不是 Web scheduler / event-engine 全链路不可用。
  - 判断：该样本仍属于“投研完整性 / evidence 门禁 fail-closed 后用户无法获得任务正文或只看到失败路径”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-04 22:01-2026-08-05 02:01 CST。
  - 22:30-23:00 CST 多条 Web scheduler / heartbeat 记录 `定时任务执行失败，跳过发送`，错误包括 `heartbeat 输出不是结构化 JSON，任务已标记失败`、`persistent_tool_failure: read-after-write reconciliation failed` 和 runner failure；同窗还有完整业务正文被失败字段承载的同类信号。
  - 同窗 source runtime 仍有 `run_start=96`、`run_finish=98`、`deliver=59`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message 增量。
  - 判断：该样本仍属于“投研完整性 / evidence 门禁 fail-closed 后用户无法获得任务正文或只看到失败路径”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-04 18:00-22:02 CST。
  - 20:02 CST Web scheduler session `Actor_web__direct__web-user-ba50cb9401c0` 记录 `[MsgFlow/web] failed ... error="数据时间：北京时间 2026-08-04 20:02；行情口径：报价源时间：北京时间 2026-08-04 04:00 至 2026-08-04 04:00（最新可得，非逐笔）...`；错误字段是一段完整投研正文开头，而不是结构化失败分类。
  - 21:01 CST Web scheduler `Actor_web__direct__web-user-afc1cabadbf8` 再次记录 `定时任务执行失败: 数据时间：北京时间 2026-08-04 21:01；数据口径：本轮查询时间...`，同样显示业务正文已形成但被失败路径承载。
  - 同窗 source runtime 仍有 `run_start=111`、`run_finish=111`、`deliver=57` 和 event-engine `poller ok=32`，说明不是 Web scheduler / event-engine 全链路不可用；`data/sessions.sqlite3` 仍未追入这些 session/message 增量。
  - 判断：该样本仍属于“投研完整性 / evidence 门禁 fail-closed 后用户无法获得任务正文”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-03 02:02-06:02 CST。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 收到 `[定时任务触发] 任务名称：盘后美股复盘与SNDK/MU存储产业链日报`。
  - runtime 记录 `market_data.preflight ... entities=SOXX,SMH ... origin=Scheduled`，随后执行多轮 `data_fetch quote` 和 `web_search`。
  - 05:03 CST 记录 `investment response contract rejected draft; retrying`，缺失项为“历史、开收盘或高低价表格必须来自本轮专用历史行情证据”。
  - 05:06 CST 该 session 落成 `[MsgFlow/web] failed ... error="数据时间：北京时间 2026-08-03 05:06；行情口径：报价源时间：北京时间 2026-08-01 04:00（最新可得，非逐笔）...`；错误字段是一段完整投研正文开头，而不是结构化失败分类。
  - 随后仅记录 `step=session.persist_assistant ... detail=failed`；`data/sessions.sqlite3` 同窗不可见该 session/message 增量。
  - 同窗其它 heartbeat / scheduler 继续运行，event-engine 也持续 `poller ok`，说明不是 Web scheduler 全链路不可用。
  - 判断：该样本仍属于“投研完整性 / evidence 门禁 fail-closed 后用户无法获得任务正文”的同根缺陷；本轮没有错投、敏感信息泄露、全渠道不可用或 P1 级主链路停摆，维持 `P2 / New`，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-27`
  - 巡检时间窗：2026-07-27 15:03-19:04 CST。
  - `cron_job_runs.run_id=48903` / 18:00 CST `TSLA 正负触发条件心跳监控` 落成 `execution_failed + skipped_error + delivered=0`。
  - runtime 同时记录 `persistent_tool_failure: execution state is uncertain; automatic replay suppressed`，说明 runner 在工具副作用 / 可重放边界 fail-closed，本轮没有生成用户可见 TSLA heartbeat 正文。
  - 同窗 16:30、17:30、18:30、19:00 的 TSLA heartbeat 又能进入 sent / noop / duplicate suppression 分支，且 direct 样本正常收口，因此这更像单轮 evidence / persistent-tool 收口失败，不是全渠道不可用。
  - 判断：该样本与本单同属 heartbeat required-evidence / runner fail-closed 导致监控轮次跳过的表现；目前只有单条 TSLA 样本且未证明应触发提醒，不拆新缺陷。维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-22`
  - 巡检时间窗：2026-07-22 19:01-23:02 CST。
  - `session_id=Actor_web__direct__web-user-0545ade83537`
    - 22:17 CST 用户粘贴 A 股复盘长文并要求回答“周四关注方向，反弹结束还是正常分歧如何理解”，assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”
    - 22:18 CST 用户原文重试，assistant 再次只返回同一失败提示。
    - runtime 两轮均记录 `entity_resolution.agent_loop ... contract_built=false answer_preserved=true ... missing_explicit_seeds=456`，随后 `failed ... error="committed terminal prefix mismatch"` 并 `session.persist_assistant ... detail=committed_prefix_after_terminal_failure`。
    - 22:19 用户追问后自动 compact，22:22 同会话成功输出长正文，说明不是 Web direct 全链路不可用。
  - 判断：该样本与本单同属“已有材料或答案信号后，finalization / evidence 门禁 fail-closed，用户只看到产品化失败提示”的表现；但直接根因是新发现的 terminal prefix mismatch，已单独建档为 [`web_direct_terminal_prefix_mismatch_commits_generic_failure.md`](./web_direct_terminal_prefix_mismatch_commits_generic_failure.md)。本单仅记录其对 required-evidence / fail-closed 用户体验的关联影响，不调整严重等级。由于同窗其它 direct / scheduler 正常收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，非 P1。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-18`
  - 巡检时间窗：2026-07-19 03:00-07:01 CST。
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 6 条 assistant / 2 条 system compact；近期 Web direct canary 03:25 / 04:51 / 06:52 均可成功回答 CRWV/NVDA 关系，说明不是 direct 全链路不可用。
  - 03:00 CST heartbeat 批量以 `chat_with_tools stream ended before Done` 落成 runner_error 并跳过发送，覆盖 TSLA、中际旭创、持仓重大事件、NVDA、NBIS、ASTS、Monitor_Watchlist_11、光模块板块、RKLB、TEM、原油、存储板块、SIVE、光迅科技、闪迪等任务；同批另有 `HTTP 529` provider 错误。
  - 05:00 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` 继续因 `PCE` 无行情覆盖预检失败，用户只看到执行出错。
  - 06:44 CST Web direct canary 同题 CRWV/NVDA 在工具调用已完成后落成 `active business stream timed out`，用户只收到“抱歉，处理超时了。请稍后再试。”；06:52 CST 同题重试成功，说明该超时当前更像单次波动，不单独建档。
  - 同窗统计命中 `runner_error=84`、`chat_with_tools stream ended before Done=63`、`active business stream timed out=2`、`context window exceeds limit=3`；运行态仍有批量 fail-closed 和降级，但后半窗同类 stream-ended 错误未继续批量复发。
  - 判断：该样本仍属于同根 provider / evidence / entity / 输出契约 fail-closed，用户只看到执行出错、超时兜底或无发送；由于同窗 direct 有成功样本、原始 provider 错误未进入用户可见 final、未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-18`
  - 巡检时间窗：2026-07-18 19:02-23:03 CST。
  - `data/sessions.sqlite3` 同窗新增 18 条 user / 11 条 assistant / 4 条 system compact，覆盖 Web regression direct、Web scheduler、Feishu direct 与 Feishu scheduler；近期会话均以 assistant 收口，没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 20:40-22:49 CST 五个非文档修复提交后，20:47 / 20:49 CRWV/NBIS regression direct 与 21:51 / 22:10 / 22:52 CRWV/NVDA regression direct 均成功保留 agent answer，说明 interactive direct 的前窗 fail-closed 已明显止血。
  - 但 21:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘前美股要闻与SNDK/MU存储产业链日报` 仍先返回证券 / 数据覆盖预检失败，随后追加用户可见 `定时任务「盘前美股要闻与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
  - `data/runtime/logs/web.log.2026-07-18` 同窗 heartbeat parse 分布为 `PlainTextTriggered=160`、`JsonNoop=72`、`PlainTextNoop=13`、`PlainTextSuppressed=8`、`JsonTriggered=5`、`JsonMalformed=4`、`JsonEmptyStatus=1`；22:30 CST `美股黄金坑信号心跳检测` 输出 `<think>` + `JsonMalformed` 后标记“heartbeat 输出不是合法 JSON，任务已标记失败”并跳过发送。
  - 22:00 / 22:30 / 23:00 CST AAOI、ORCL 等 heartbeat 继续以实体 / 数据覆盖 runner_error 跳过发送；23:00 多条 heartbeat deliver preview 虽被判为 `PlainTextTriggered`，但内容自称 noop 或数据未核验，仍显示 evidence / 输出契约分流不足。
  - 判断：interactive direct 有止血证据，但 scheduler / heartbeat 仍会因 evidence / entity / 输出契约 fail-closed 让用户只看到执行出错或无发送；由于同窗 direct 和部分 scheduler 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-18`
  - 巡检时间窗：2026-07-18 15:02-19:02 CST。
  - `data/sessions.sqlite3` 同窗新增 2 条 user / 2 条 assistant，覆盖 2 个 Web regression direct session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 18:19 CST Web regression direct session `Actor_web__direct__codex-regression-2d6b4be8-crwv-nbis` 对 `分析下crwv和nbis的估值` 已精确核验 CoreWeave / Nebius 与两者行情，但最终只返回“这次回答未通过投研完整性检查”；18:40 CST 同题 session `Actor_web__direct__codex-regression-8d4fcdd6-crwv-nbis-v2` 在后续提交后成功输出完整估值对比，说明 interactive direct 已部分止血，但 fail-closed 仍在本窗出现过。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续出现 333 条 `runner_error`、120 条定时任务执行失败、34 条“当前数据供应商没有返回”、34 条“已识别证券代码”和 16 条多候选信号，覆盖 Feishu / Web heartbeat 与普通 scheduler。
  - 19:00 CST 代表任务包括 AAOI、ORCL、闪迪、存储板块、TSLA、绿田机械等，分别落成无行情覆盖、多候选、非证券实体、结构化失败或普通问候型 raw preview，用户侧只能看到任务失败或不发送。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed，用户只看到实体解析失败、候选澄清、scheduler 失败提示或无发送；由于同窗没有错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-18`
  - 巡检时间窗：2026-07-18 11:00-15:02 CST。
  - `data/sessions.sqlite3` 同窗新增 11 条 user / 11 条 assistant，近期 Web direct / regression session 均以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 12:49-12:52 CST Web direct session `Actor_web__direct__web-user-4d761588537b` 连续对 `Cohr` / `Coherent Corp` 只返回实体无法确认或实体解析暂时未能确认；12:52 后同一 session 对 `Acls` 可成功核验并输出行情与技术分析，说明 evidence / entity guard 仍会让部分真实投研请求 fail-closed。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续出现 332 条 `runner_error`、173 条定时任务执行失败和 256 条“证券实体解析暂时未能确认”信号，覆盖 Feishu / Web heartbeat 与普通 scheduler。
  - 15:00 CST 代表任务包括 AAOI、ASTS、ORCL、TSLA、RKLB、NVDA、闪迪、中际旭创、光迅科技、存储板块、绿田机械、Monitor_Watchlist_11、Cerebras、SIVE 等，均在实体解析、多候选或行情覆盖门禁阶段 fail-closed，用户侧只能看到任务失败或不发送。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed，用户只看到实体解析失败、候选澄清、scheduler 失败提示或无发送；由于同窗没有错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-18`
  - 巡检时间窗：2026-07-18 07:00-11:01 CST。
  - `data/sessions.sqlite3` 同窗新增 16 条 user / 17 条 assistant，覆盖 11 个近期 Web / Feishu / Discord direct 或 scheduler session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - `data/runtime/logs/web.log.2026-07-18` 同窗继续出现 262 条 `runner_error`、135 条定时任务执行失败和 207 条“证券实体解析暂时未能确认”信号，覆盖 Feishu / Web heartbeat 与普通 scheduler。
  - 08:00-11:00 CST 代表任务包括 AAOI、ASTS、ORCL、TSLA、NVDA、NBIS、闪迪、中际旭创、光迅科技、存储板块、全天原油、绿田机械、Monitor_Watchlist_11、Cerebras、SIVE 等，均在实体解析、多候选或行情覆盖门禁阶段 fail-closed，用户侧只能看到任务失败或不发送。
  - 10:00 CST `RKLB异动监控` 已完成业务判断但输出 `<think>` + `PlainTextSuppressed`，最后落成“heartbeat 输出不是结构化 JSON，任务已标记失败”；说明 evidence / 输出契约失败仍会把有效监控轮次变成跳过发送。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed，用户只看到实体解析失败、候选澄清、scheduler 失败提示或无发送；由于同窗没有错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-17`
  - 巡检时间窗：2026-07-18 03:00-07:01 CST。
  - `data/sessions.sqlite3` 同窗新增 3 条 user / 5 条 assistant，近期 direct / scheduler session 均以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 先写入证券 / 数据覆盖预检失败，随后追加用户可见 `定时任务「盘后美股复盘与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
  - 05:30 CST Feishu scheduler session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 的 `美股收盘后跨市场复盘` 只返回实体解析失败；06:00 CST Feishu scheduler session `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` 的 `每日美股盘后收盘复盘` 只返回 Nasdaq 多候选澄清，均未生成用户请求的复盘主体。
  - `data/runtime/logs/web.log.2026-07-17` 同窗继续出现 341 条 `runner_error`、175 条定时任务执行失败和 340 条实体 / 多候选 / 无覆盖相关信号，覆盖 Feishu / Web heartbeat 与普通 scheduler。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed，用户只看到实体解析失败、候选澄清或 scheduler 失败提示；由于同窗没有错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-17`
  - 巡检时间窗：2026-07-17 15:01-19:02 CST。
  - `data/sessions.sqlite3` 同窗新增 8 条 user / 9 条 assistant，覆盖 8 个近期 Web direct / scheduler session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 18:00 CST Web scheduler session `Actor_web__direct__web-user-ba50cb9401c0` 的 `18:00 美股盘前 X 英文帖` 先写入 `X 的报价没有可用且足够新的数据源时间戳。本轮不会把查询时间冒充行情时间。`，随后写入 `定时任务「18:00 美股盘前 X 英文帖」执行出错，请稍后重试。`
  - 同窗 runtime 继续出现 45 条定时任务执行失败、56 条 `runner_error`、15 条“heartbeat 输出不是结构化 JSON”和 58 条“证券实体解析暂时未能确认”相关信号，覆盖 Feishu / Web heartbeat。
  - 17:35 / 17:47 CST 同题 RKLB Web direct 已成功核验并输出价格区间，说明不是 Web direct 或 scheduler 全链路不可用。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败或 scheduler 失败提示；由于同窗 direct / heartbeat 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-16` / `data/runtime/logs/web.log.2026-07-17`
  - 巡检时间窗：2026-07-17 07:01-11:02 CST。
  - `data/sessions.sqlite3` 同窗新增 10 条 user / 10 条 assistant，覆盖 8 个近期 session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 07:15 CST Web direct session `Actor_web__direct__web-user-be13e1f84d14` 对 ISRG 财报 / 盘后下跌请求已有 ISRG 同代码现价与报价源时间前缀，但最终落成投研完整性失败，runtime 记录 `step=session.persist_assistant detail=failed`。
  - 07:24 CST 同一 Web direct session 对 UNH 追问已有 UNH 同代码现价与报价源时间前缀，但最终再次落成投研完整性失败。
  - 08:30 CST Web direct session `Actor_web__direct__web-user-266454c88ed6` 多标的分析在执行 `data_fetch earnings_calendar` 与 `web_search` 后仍落成失败并未写入本地 SQLite 最新消息。
  - 10:05 CST Feishu direct `nibs` 与 10:53 CST `中船特气` 均只返回产品化实体解析失败；10:06 CST 同用户 `nbis` 重试成功，说明不是 Feishu direct 全链路不可用。
  - 同窗 RMBS / NBIS regression direct、Citrini / SemiAnalysis 文章跟踪 scheduler 均成功收口，说明该缺陷仍是投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败或实体解析失败。
  - 判断：由于同窗 direct / scheduler 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-16`
  - 巡检时间窗：2026-07-17 03:01-07:01 CST。
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 6 条 assistant，覆盖 5 个 session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 05:00 CST Web scheduler session `Actor_web__direct__web-user-afc1cabadbf8` 的 `盘后美股复盘与SNDK/MU存储产业链日报` 先写入 `本轮证券实体与当前数据核验超过 45 秒，已终止本轮预检；请重试。`，随后写入 `定时任务「盘后美股复盘与SNDK/MU存储产业链日报」执行出错，请稍后重试。`
  - 06:11 CST Web direct session `Actor_web__direct__investment-repair-rmbs-1784239766` 对 `现在rmbs怎么看` 已有 RMBS 同代码现价与报价源时间前缀，但最终仍落成投研完整性失败。
  - 06:53 CST Web direct session `Actor_web__direct__web-user-be13e1f84d14` 对 ISRG 财报 / 盘后下跌分析请求已有 ISRG 同代码现价与报价源时间前缀，但最终仍落成投研完整性失败。
  - 同窗 06:31 CST Web scheduler `1亿美元AI科技组合每日跟踪` 与 07:01 CST Feishu scheduler `美股持仓收盘后早报` 均成功输出长正文，说明该缺陷不是 Web / Feishu scheduler 或 direct 全链路不可用。
  - `data/runtime/logs/web.log.2026-07-16` 同窗另有 104 条 runner / 执行失败和多批 heartbeat `runner_error`，覆盖实体 / 核验门禁失败后跳过发送。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示或 scheduler 失败提示；由于同窗 direct / scheduler 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-16`
  - 巡检时间窗：2026-07-16 23:01-2026-07-17 03:03 CST。
  - `data/sessions.sqlite3` 同窗新增 5 条 user / 5 条 assistant，覆盖 5 个 Web regression direct session，全部以 assistant 收口；没有 user-only 悬挂、错投、敏感信息泄露或全渠道不可用证据。
  - 00:59 CST Web direct session `Actor_web__direct__regression-market-final-20260717` 对 `整个都在跌，今天为什么大跌` 只返回 `这次回答未通过投研完整性检查，已停止发送不完整或未经充分核验的结论。请稍后重试。`，`metadata_json` 标记 `run_failed=true` / `AgentFailed`。
  - 02:46 CST Web direct session `Actor_web__direct__regression-rmbs-20260717-0245` 对 `现在rmbs怎么看` 只返回同一投研完整性失败文案，`metadata_json` 标记 `run_failed=true` / `AgentFailed`。
  - 同窗 00:43 / 00:46 / 00:57 CST AAPL 报价样本成功输出行情正文，说明该缺陷不是 Web direct 全链路不可用。
  - `data/runtime/logs/web.log.2026-07-16` 同窗另有 150 条 heartbeat `runner_error`，多批任务因实体 / 核验门禁失败跳过发送。
  - 判断：该样本仍属于同根投研完整性 / evidence 门禁 fail-closed 后用户只看到通用失败提示；由于同窗 direct 可成功收口，未见错投、数据破坏、敏感信息泄露或全渠道不可用，维持功能性 `P2 / New`，不升级为 P1，不创建 GitHub Issue。
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

## 最新运行态复核（2026-08-19 18:02 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-19 14:01-18:02 CST。
  - 同窗有 4 条 `定时任务执行失败，跳过发送: failure_kind=execution_failed err=heartbeat 输出不是结构化 JSON，任务已标记失败`，直接表现为 heartbeat 本轮未发送。
  - 同窗还有 82 条 `function_calling tool call rejected by global budget`，多条 deliver 明写工具调用上限、行情未完整核验或沿用上下文报价；这些仍会放大实时核验 / evidence fail-closed 后跳过或降级的风险。
- 本轮判断
  - 该样本仍落在“实时核验 / 完整性门禁或结构化输出失败后只给失败态、跳过发送”的既有缺陷范围，不是新的独立根因。
  - 同窗仍有其它 heartbeat 成功 deliver，未见错投、敏感信息泄露或全渠道不可用，因此维持功能性 `P2 / New`，非 P1。
## 最新运行态复核（2026-07-17 23:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-17 19:01-23:01 CST。
  - 21:00 CST Web scheduler `盘前美股要闻与SNDK/MU存储产业链日报` 先生成一条 assistant final，正文含已核验 SNDK / MU 行情前缀，随后以“抱歉，这次处理失败了。请稍后再试。”结束。
  - 同 session 下一条 assistant 写入用户可见 scheduler 失败文本：`定时任务「盘前美股要闻与SNDK/MU存储产业链日报」执行出错，请稍后重试。`，metadata 标记 `scheduler_failure=true`，并把失败前生成的业务正文塞入 `error` 字段。
- `data/runtime/logs/web.log.2026-07-17`
  - 同窗继续出现 62 条 `runner_error`、57 条定时任务执行失败，以及多条实体核验 / 结构化失败导致的 heartbeat 跳过发送。
- 本轮判断
  - 该样本仍落在“实时核验 / 完整性门禁 fail-closed 后只给产品化失败提示”的既有缺陷范围。
  - 任务没有产出用户原本请求的盘前简报，但同窗其他 scheduler 正常收口，未见错投、敏感信息泄露或全渠道不可用，因此维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-19 03:01 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-18 23:01-2026-07-19 03:01 CST。
  - 同窗新增 2 条 user / 2 条 assistant，覆盖 2 个 Web direct canary / regression session，均以 assistant 收口；23:50 CRWV/NVDA regression 已成功输出完整关系与估值分析。
  - 02:38 Web direct canary 只返回通用失败，已另登记为 OpenAI-compatible stream completion 缺陷；本单不重复登记 direct provider stream 根因。
- `data/runtime/logs/web.log.2026-07-18`
  - 同窗继续记录真实 heartbeat 运行态；本轮关键词命中 1406 行，03:00 CST 多条 heartbeat 在 runner 层失败并跳过发送。
  - 代表样本包括 `美股黄金坑信号心跳检测` 因上游 `HTTP 529` 落成 `provider_http_error`，以及 TSLA / NVDA / ASTS / RKLB / TEM / 存储板块 / 光迅科技等多条 heartbeat 命中 `chat_with_tools stream ended before Done` 后落成 `runner_error`。
  - 同批仍有 AAOI `SEC` 无行情覆盖、ORCL 多上市地候选等实体 / evidence fail-closed 信号，说明 heartbeat 覆盖仍未稳定生成合法 noop / triggered 主体。
- 本轮判断
  - 这些样本继续落在 heartbeat / scheduler 因 runner、evidence 或实体核验失败而跳过发送的既有 P2 范围。
  - 同窗仍有 Web direct 成功样本，且未见错投、敏感信息泄露、全渠道不可用或原始 provider 错误进入用户可见 final，因此维持 `P2 / New`，非 P1。
