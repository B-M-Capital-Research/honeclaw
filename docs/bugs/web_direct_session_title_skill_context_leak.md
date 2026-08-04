# Bug: Web direct 会话标题写入完整技能上下文与本机路径

## 发现时间

- 2026-08-04 14:02 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-08-04 10:02-14:02 CST。
  - session_id: `Actor_web__direct__codex_5fsndk_5fearnings_5fvalidation_5fnative3_5f20260804`
  - `2026-08-04 11:11 CST` 附近，ACP `session_info_update` 的 `title` 字段写入完整 Session 上下文、本轮用户输入、`Invoked Skill Context`、财报 workflow 参数、技能规则块，以及本机 skill base directory。
  - 同一 session 的用户可见 `agent_message_chunk` final 可重构为 7032 字报告，以 `stopReason=end_turn` 收口；未见空回复、错投、response error、provider 原始错误、panic、token、env 字段或原始工具 JSON 进入 final。
  - 同窗 `data/sessions.sqlite3` 仍未追入真实运行，SQLite `session_messages.max(timestamp)` 停在 2026-08-01 14:13 CST，因此本缺陷证据来自 ACP 事件日志重构。

## 端到端链路

1. Web direct 财报工作流创建 Codex ACP 会话。
2. 系统把当前会话上下文、技能上下文和用户任务作为 runner 可用上下文传入。
3. ACP runner 发出 `session_info_update`。
4. `title` 字段没有被压缩成安全标题，而是承载了完整上下文块和本机技能目录。
5. 该字段进入 ACP 事件日志，并可能进入任务标题 / 会话元数据展示面。

## 期望效果

- 会话标题只应是短标题，例如公司名、任务类型或用户请求摘要。
- skill 指令、workflow 参数、内部 guard、服务端候选扫描结果、本机目录和完整用户任务块不应进入 title。
- 如果需要排障，应把完整上下文留在受控日志字段，并对路径和内部 prompt 做最小化 / 脱敏。

## 当前实现效果

- 用户可见 final 主体正常完成财报分析报告，说明直聊回答链路没有整体失败。
- 但 `session_info_update.title` 承载了大量内部上下文和本机路径，标题字段语义被破坏，也扩大了内部 prompt / 本机路径在会话元数据中的可见面。
- 本轮没有证据显示该内容被直接发送到最终回答正文；当前风险集中在标题 / 会话元数据边界。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 主报告已正常收口，未见未回复、空回复、错投、投递失败、会话中断或系统链路不可用证据。
- 暴露内容主要是内部技能上下文、workflow 规则和本机目录，不是凭据或用户跨会话数据；因此不影响主功能链路，按规则定级为 P3。
- 如果 title 被前端侧边栏、历史列表或通知展示，用户会看到极长且包含内部实现细节的标题，影响产品可信度并增加内部 prompt 被误扩散的风险。

## 根因判断

- 直接证据只能证明 ACP `session_info_update.title` 使用了未裁剪的上下文文本。
- 初步判断是标题生成 / 同步路径缺少字段语义约束：把 runner 的完整上下文或 prompt-like 文本当作 title 写回，而不是从用户任务中抽取短标题。
- 该问题不同于用户可见 final sanitizer 漏洞：本轮 final 没有同类内部上下文外泄，泄露点在会话元数据更新。

## 下一步建议

- 收紧 `session_info_update.title` 的生成来源，只允许短摘要、公司名或任务名，限制长度并剥离 prompt-like 块。
- 在写入 title 前统一过滤 `Skill:`、`Invoked Skill Context`、`Base directory`、`【Session 上下文】`、本机绝对路径和服务端工具候选扫描块。
- 增加 ACP event / Web direct 回归：包含技能上下文的投研工作流应只产生短标题，不能把完整上下文写入 `session_info_update.title`。
