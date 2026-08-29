# Bug: Tavily web_search pay-as-you-go quota exhausted degrades realtime research

## 发现时间

- 2026-08-26 02:01 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 2026-08-29 10:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 58 / 103 条，其中 pay-as-you-go limit 19 条，`tool_execute_error name=web_search` 19 条。
  - 代表样本包括 UTC `2026-08-29T00:00:28Z`、`2026-08-29T00:31:00Z`、`2026-08-29T01:00:16Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 DataFetch / web_search 预算或速率限制口径。
  - 同窗 heartbeat 仍有 `run_start=35`、`run_finish=35`、`deliver=19`、`duplicate_suppressed=7`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / 数据未核验”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-29 02:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 114 条，其中 pay-as-you-go limit 25 条，`tool_execute_error name=web_search` 25 条。
  - 代表样本包括 UTC `2026-08-28T14:30:27Z`、`2026-08-28T15:00:28Z`、`2026-08-28T18:00:37Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 DataFetch / web_search 预算或速率限制口径。
  - 同窗 heartbeat 仍有 `run_start=57`、`run_finish=57`、`deliver=37`、`duplicate_suppressed=9`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / 数据未核验”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-28 18:01-22:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 157 条，其中 pay-as-you-go limit 32 条，`tool_execute_error name=web_search` 32 条。
  - 代表样本包括 UTC `2026-08-28T10:30:13Z`、`2026-08-28T11:30:14Z`、`2026-08-28T13:30:40Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 DataFetch / web_search 预算或速率限制口径。
  - 同窗 heartbeat 仍有 `run_start=57`、`run_finish=58`、`deliver=31`、`duplicate_suppressed=18`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-28 14:01-18:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 176 条，其中 pay-as-you-go limit 35 条，`tool_execute_error name=web_search` 35 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-28T06:30:40Z`、`2026-08-28T07:00:23Z`、`2026-08-28T09:00:49Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 `function_calling tool call rejected by global budget tool="data_fetch" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=56`、`deliver=34`、`duplicate_suppressed=16`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-28 10:00-14:02 CST 巡检窗口内，Tavily pay-as-you-go limit 26 条，`tool_execute_error name=web_search` 26 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-28T02:00:55Z`、`2026-08-28T03:00:56Z`、`2026-08-28T06:00:55Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=59`、`deliver=28`、`duplicate_suppressed=12`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-28 06:01-10:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 140 条，其中 pay-as-you-go limit 25 条，`tool_execute_error name=web_search` 25 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-27T22:30:24Z`、`2026-08-27T22:30:34Z`、`2026-08-28T02:00:33Z` 附近多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=56`、`deliver=29`、`duplicate_suppressed=10`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-28 02:01-06:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 171 条，其中 pay-as-you-go limit 35 条，`tool_execute_error name=web_search` 35 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-27T18:30:45Z`、`2026-08-27T19:31:44Z`、`2026-08-27T21:31:47Z` 附近多次 `web_search` 不可用或触发工具预算上限；同轮 deliver preview 继续把“Web 搜索通道不可用 / 工具调用上限 / 未核验实时价格”口径带入心跳正文。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=58`、`deliver=35`、`duplicate_suppressed=9`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-27 22:01-2026-08-28 02:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 107 条，其中 pay-as-you-go limit 22 条，`tool_execute_error name=web_search` 22 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-27T14:30:34Z`、`2026-08-27T15:00:29Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=55`、`deliver=31`、`duplicate_suppressed=4`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用上限 / DataFetch quote 接口触发账户级速率限制”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-27 18:02-22:02 CST 巡检窗口内，Tavily / `web_search` 相关信号 100 条，其中 pay-as-you-go limit 20 条，`tool_execute_error name=web_search` 20 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-27T12:00:32Z`、`2026-08-27T13:30:22Z`、`2026-08-27T14:00:40Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=71`、`run_finish=71`、`deliver=16`、`duplicate_suppressed=4`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“Web 搜索不可用 / quote 不可用 / 工具调用上限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-27 10:01-14:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 133 条，其中 pay-as-you-go limit 30 条，`tool_execute_error name=web_search` 30 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-27T02:30:25Z`、`2026-08-27T03:00:26Z`、`2026-08-27T03:30:27Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 DataFetch / web_search 预算拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=57`、`deliver=32`、`duplicate_suppressed=15`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“Web 搜索不可用 / 工具调用上限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-27 06:00-10:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 215 条，其中 pay-as-you-go limit / unavailable 117 条，`tool_execute_error name=web_search` 39 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T22:30:24Z`、`2026-08-26T23:01:00Z`、`2026-08-27T02:00:27Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 per-tool budget 拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=61`、`deliver=27`、`duplicate_suppressed=5`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用受限 / Web 搜索不可用”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-27 02:01-06:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 119 条，其中 pay-as-you-go limit 25 条，`tool_execute_error name=web_search` 25 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T20:00:30Z`、`2026-08-26T20:30:27Z`、`2026-08-26T22:00:31Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 per-tool budget 拒绝。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=58`、`deliver=33`、`duplicate_suppressed=7`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用受限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 22:00-2026-08-27 02:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 134 条，其中 pay-as-you-go limit 29 条，`tool_execute_error name=web_search` 29 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T14:00:32Z`、`2026-08-26T14:30:35Z`、`2026-08-26T18:00:28Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后继续出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或 per-tool budget 拒绝。
  - 同窗 heartbeat 仍有 `run_start=63`、`run_finish=63`、`deliver=38`、`duplicate_suppressed=10`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“新闻通道暂不可用 / 工具调用受限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 18:00-22:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 151 条，其中 pay-as-you-go limit 29 条，`tool_execute_error name=web_search` 29 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T10:30:27Z`、`2026-08-26T14:00:37Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后进入 `function_calling tool call rejected by global budget tool="web_search" limit=3`。
  - 同窗 heartbeat 仍有 `run_start=58`、`run_finish=58`、`deliver=29`、`duplicate_suppressed=8`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用受限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 14:02-18:01 CST 巡检窗口内，Tavily / `web_search` 相关信号 128 条，其中 pay-as-you-go limit 25 条，`tool_execute_error name=web_search` 25 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T06:30:25Z`、`2026-08-26T06:30:31Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同轮随后进入 `function_calling tool call rejected by global budget tool="web_search" limit=3`。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=56`、`deliver=26`、`duplicate_suppressed=11`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用和工具预算受限后，用旧 quote、旧上下文或“工具调用受限”口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 10:01-14:02 CST 巡检窗口内，Tavily pay-as-you-go limit 23 条，`web_search` unavailable 46 条，`tool_execute_error name=web_search` 23 条，仍只有 `key_count=1`。
  - 代表样本包括 UTC `2026-08-26T02:30:22Z`、`2026-08-26T02:30:31Z`、`2026-08-26T06:00:29Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；同窗 heartbeat 仍有 `run_start=56`、`run_finish=56`、`deliver=24`、`duplicate_suppressed=9`，说明不是全局调度停摆。
  - 多条 deliver preview 继续把“Web 搜索通道暂不可用 / 工具调用额度已达上限 / 无法获取新的独立行情或新闻数据”等执行口径带入用户可见候选；部分轮次沿用 `hone_quote_time` 旧行情锚收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 06:01-10:05 CST 巡检窗口内，Tavily / `web_search` 相关信号继续增至 188 条，仍只有 `key_count=1`，pay-as-you-go limit / unavailable 持续出现。
  - 代表样本包括 UTC `2026-08-25T22:30:21Z`、`2026-08-25T23:30:22Z`、`2026-08-26T02:00:39Z` 多次 `web_search` 因 Tavily pay-as-you-go limit 失败；随后同轮仍出现 `function_calling tool call rejected by global budget tool="web_search" limit=3` 或继续依赖旧 quote / 上下文收口。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=58`、`deliver=31`、`duplicate_suppressed=13`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用、工具预算受限或 quote 时间戳停留后，用旧上下文、quote 或工具上限口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-26 02:01-06:01 CST 巡检窗口内，Tavily / `web_search` 相关信号从上一窗 87 条升至 142 条，仍只有 `key_count=1`，pay-as-you-go limit / unavailable 持续出现。
  - 同窗 heartbeat 仍有 `run_start=56`、`run_finish=58`、`deliver=31`、`duplicate_suppressed=9`，说明不是全局调度停摆；但多轮任务继续在 `web_search` 不可用与工具预算受限后，用旧上下文、quote 或工具上限口径收口。
  - 本窗未见错投、敏感凭据泄露或全渠道不可用；维持功能性 `P2 / New`，非 P1，不创建 GitHub Issue。
  - 2026-08-25 22:01-2026-08-26 02:01 CST 巡检窗口内，`web_search` 持续只有 `key_count=1`，Tavily 返回 pay-as-you-go limit 拒绝。
  - 同窗统计：Tavily limit / unavailable 相关日志 87 条，`tool_execute_error name=web_search` 多次出现，heartbeat 侧仍有 `run_start=56`、`run_finish=62`、`deliver=39`、`duplicate_suppressed=14`，说明不是全局调度停摆。
  - 代表样本：
    - UTC `2026-08-25T16:01:07Z`：`web_search` 因 Tavily pay-as-you-go limit 失败，随后同轮进入 `function_calling tool call rejected by global budget tool="web_search" limit=3`。
    - UTC `2026-08-25T18:00:24Z`：同一错误再次连续出现，`web_search` 被工具层记为执行错误。
    - UTC `2026-08-25T18:00:40Z`：`持仓重大事件心跳提醒` raw preview 明确写出 `Web search is not available`，deliver preview 退化为 Starlink / Tesla 一般说明，而不是稳定围绕持仓重大事件实时核验。
    - UTC `2026-08-25T18:01:08Z`：`持仓财报与重大新闻心跳提醒` raw preview 明确写出 `Web search is also not available`，随后只能依赖剩余 quote / 历史上下文组织 noop。
- `data/sessions.sqlite3`
  - 本轮无法从 SQLite 交叉验证最新用户会话：`sessions.updated_at`、`sessions.last_message_at`、`session_messages.timestamp` 仍停在 `2026-08-01T14:13:46+08:00`，`session_messages.imported_at` 停在 `2026-08-02T20:59:58+08:00`。

## 端到端链路

1. scheduler / heartbeat 进入 function-calling runner。
2. runner 需要 `web_search` 补实时新闻、事件或外部来源。
3. `hone_tools::web_search` 调 Tavily，当前仅有 1 个 key，provider 返回额度拒绝。
4. `ToolRegistry` 将 `web_search` 记为工具执行错误，后续同轮还会触发 per-tool / global budget 拒绝。
5. heartbeat 仍继续尝试收口，部分任务退化为“无法搜索 / 只能用旧上下文 / 只查报价”的输出，放大既有 heartbeat 目标漂移、格式漂移和实时核验降级。

## 期望效果

- web_search 额度耗尽时，系统应有清晰的 provider/account 健康状态、可见降级策略和可恢复路径。
- 对强实时任务，应避免把无搜索能力的结果包装成已完成实时核验；必要时应产品化标记本轮因搜索源不可用而跳过或降级。
- 如果配置支持多 key，应能在安全边界内轮转可用 key；如果只有单 key，应让运营能及时发现并补充额度。

## 当前实现效果

- 工具层已经正确把 Tavily 不可用记为错误，没有再伪装成空成功。
- 但业务链路没有可用替代搜索源或运营侧健康门禁，heartbeat 仍会继续生成用户可见候选或 noop，且正文会混入“Web search is not available / 工具上限 / 无法获取新数据”等执行口径。
- 这会和既有 `scheduler_heartbeat_unknown_status_silent_skip.md`、`scheduler_heartbeat_trigger_json_payload_leak.md`、`web_scheduler_ai_watchlist_kweb_topic_drift.md` 叠加，但根因是当前 live Tavily provider/account 额度耗尽，链路独立于 heartbeat JSON 协议本身。

## 用户影响

- 依赖外部实时搜索的持仓重大事件、财报新闻、行业事件 heartbeat 可能漏掉真实新增事件，或基于旧上下文 / 部分报价给出低质量结论。
- 用户不会看到完整 provider 机密或 token，但可能看到“无法搜索、工具上限、未能获取新数据”等执行口径，降低产品可信度。
- 同窗 scheduler、data_fetch 与部分 heartbeat 投递仍在运行，未见错投、全渠道不可用、敏感凭据泄露或数据破坏；因此定级为功能性 P2，非 P1，不创建 GitHub Issue。

## 根因判断

- 直接根因是 Tavily provider/account 当前额度耗尽，且 live 配置只有 1 个可用 key。
- 系统层缺口是缺少面向强实时业务的搜索源健康隔离与产品化降级策略：工具错误虽已脱敏上抛，但 scheduler/heartbeat 仍把同轮任务推入普通生成路径。

## 下一步建议

1. 先确认 Tavily 额度 / key 池配置是否符合当前生产负载；必要时补额度或配置可用 key。
2. 为 `web_search` 增加可观测健康指标，区分 provider quota、auth、temporary transport 和 local config 缺失。
3. 在 scheduler / heartbeat 的强实时任务里识别 `web_search` provider quota exhausted，优先跳过或输出产品化降级结论，不继续让模型用旧上下文伪实时收口。
4. 后续巡检若 `web_search` 恢复且不再出现 Tavily limit，可保持 `New` 到代码/配置侧完成并有自然窗口验证后再改为 `Fixed` 或 `Closed`。
