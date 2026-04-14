# Bug: 会话上下文溢出未自动恢复且向用户泄露底层报错

- **发现时间**: 2026-04-14
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近修复提交: `1a65ce0 fix: recover from context overflow`
  - 关联归档: `docs/archive/plans/context-overflow-recovery.md`

## 端到端链路

1. 用户在 Feishu / iMessage / Web 等渠道持续追问，当前 session 历史逐渐变长。
2. `AgentSession` 组装 prompt 时把恢复出的上下文、compact summary 与本轮消息一并送入 runner。
3. 上游 provider 因 token 超限返回 `context window exceeds limit` / `maximum context length` / `too many tokens` 一类错误。
4. 旧实现没有在会话层自动 compact 并重试，而是直接把底层 provider 错误返回给用户。

## 期望效果

- 当会话历史过长导致上下文超限时，系统应优先自动压缩历史并重试至少一次。
- 如果自动恢复后仍失败，用户应看到稳定、产品化的提示，例如“当前会话过长，请 compact 或开启新会话”，而不是 provider 内部错误细节。

## 当前实现效果（问题发现时）

- 旧实现会直接向用户暴露类似 `bad_request_error: invalid params, context window exceeds limit (...)` 的底层报错。
- 用户视角会感知为“系统突然坏了”，而不是“当前会话需要压缩”。
- 这会中断一整条会话链路，且用户无法判断该怎么继续。

## 用户影响

- 长会话在高频问答、长文分析、带工具调用的链路里会更容易触发。
- 一旦触发，用户无法拿到当前问题答案，且会被误导为系统异常或模型服务不可用。
- 该问题会显著拉低会话连续性，属于高频长对话场景下的可感知故障。

## 根因判断

- 会话层只识别到 runner 失败，但没有把“上下文超限”当成可恢复错误处理。
- 错误文案没有经过产品化改写，直接透传了 provider 的原始实现细节。

## 修复线索

- `1a65ce0` 已补上上下文超限识别、强制 compact、单次自动重试和最终友好错误改写。
- 当前 bug 台账先以 `New` 登记，等待人工确认是否按现网状态转 `Fixed` / `Closed`。
