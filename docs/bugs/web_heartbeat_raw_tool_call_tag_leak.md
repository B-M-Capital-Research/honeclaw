# Bug: Web heartbeat deliver 外露原始工具调用标签

## 发现时间

2026-08-08 10:01 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-08 10:01-14:02 CST（UTC 2026-08-08 02:01-06:02）。
  - 本轮未再检出 `<minimax:tool_call>`、`<invoke name=` 或 `<absolute-path>` 进入 heartbeat `deliver_preview`。
  - 状态仍维持 `New/P3`：10:00 CST 样本仍是未修复的最近真实证据，且本轮没有对应代码修复或清理层闭环；需后续自然窗口继续观察。
- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-08 06:01-10:01 CST（UTC 2026-08-07 22:01-2026-08-08 02:01）。
  - `2026-08-08T02:00:30Z` / 10:00 CST，`job_id=j_348d0f87`、`job=中际旭创关键事件心跳提醒`、`target=web-user-c2776780c59d`。
  - 同轮 `HeartbeatDiag` 记录 `parse_kind=PlainTextTriggered` 并进入 `deliver`；`deliver_preview` 开头先写“已存在心跳任务 ... 我来用你提供的条件更新它”，随后直接包含 `<minimax:tool_call>`、`<invoke name="cron_job">` 与 `<absolute-path>` 占位式协议片段。
- 本轮去重检查：
  - 已搜索 `docs/bugs/*.md` 中的 `minimax:tool_call`、`invoke name=`、`absolute-path`、`原始工具调用`、`工具调用标签`、`tool_call` 等关键词。
  - 既有 `scheduler_heartbeat_trigger_json_payload_leak.md` 覆盖 heartbeat JSON / 协议字段外泄；`feishu_direct_partial_reply_before_tool_completion.md` 覆盖 Feishu direct 在工具未完成时外发 `<function_calls>`。本缺陷发生在 Web heartbeat deliver 出站路径，用户可见形态是 MiniMax 原始工具调用标签和占位路径，属于独立格式污染形态。

## 端到端链路

1. Web heartbeat 定时任务到点触发 `中际旭创关键事件心跳提醒`。
2. 模型把当前 heartbeat 执行误入任务更新 / 工具调用路径。
3. 出站解析将该文本归类为 `PlainTextTriggered`。
4. Scheduler deliver 候选没有剥离 `<minimax:tool_call>` / `<invoke ...>` 原始协议标签。
5. 用户侧可能看到内部工具调用标签和 `<absolute-path>` 占位符，而不是自然语言监控结果。

## 期望效果

- Heartbeat 出站正文应只包含用户可读的监控结论、触发事实或安全的 noop / 未触发说明。
- 原始工具调用标签、provider 特定协议、占位路径和内部工具名不应进入用户可见 deliver。
- 若模型误入工具调用或任务管理路径，应该失败收口、静默跳过或改写成产品化说明，而不是把协议文本发送出去。

## 当前实现效果

- `deliver_preview` 直接包含 `<minimax:tool_call>` 和 `<invoke name="cron_job">`。
- 同轮调度、模型执行和 deliver 候选链路没有整体失败；问题集中在用户可见输出净化和 heartbeat 任务意图边界。
- 该样本还与既有“heartbeat 已创建监控任务仍反复输出任务创建 / 更新说明”相邻，但新增污染点是原始工具调用标签进入用户可见正文。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 用户订阅的是中际旭创关键事件 heartbeat，却可能收到工具调用协议片段，内容不可直接理解，也暴露了内部执行格式。
- 为何不影响功能链路，因此定级为 P3：本轮没有证据显示 Web scheduler 整体未触发、错投对象、数据破坏、敏感凭据泄露或全渠道不可用；主问题是单条 heartbeat 用户可见格式和内部协议边界受损。

## 根因判断

- 直接根因是 heartbeat final / delivery sanitizer 没有覆盖 MiniMax 风格 `<minimax:tool_call>` 与 `<invoke name=...>` 标签。
- 上游根因可能是 heartbeat 执行 prompt 仍允许已创建任务在周期执行时进入任务治理 / 工具调用路径，导致模型把“更新监控条件”当作用户可见结果。
- 该问题与 JSON 协议载荷外泄同属 heartbeat 出站结构化污染族，但协议形态、触发 job 和净化规则缺口不同，因此单独登记。

## 下一步建议

1. 在 heartbeat delivery 出站净化层增加 raw tool-call tag guard，覆盖 `<minimax:tool_call>`、`<function_calls>`、`<invoke name=...>` 和占位路径片段。
2. 对 heartbeat 执行 prompt 增加“周期执行不得输出任务创建 / 更新 / tool call 协议”的硬约束。
3. 增加回归：当 heartbeat result 含 raw tool-call tag 时，不得作为 `PlainTextTriggered` 正常送达；应改写或失败收口。
