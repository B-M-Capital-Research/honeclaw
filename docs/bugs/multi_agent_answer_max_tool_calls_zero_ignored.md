# Bug: Multi-Agent Answer Agent 在设置页允许 `maxToolCalls=0`，但运行时强制提升为至少 1，用户无法真正禁用补充工具调用

- **发现时间**: 2026-04-15
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `packages/app/src/pages/settings.tsx:548-558`
    - `crates/hone-channels/src/core.rs:1047-1053`
    - `crates/hone-channels/src/runners/multi_agent.rs:343-358`

## 端到端链路

1. Desktop 设置页的 Multi-Agent Answer Agent 区块允许用户把 `maxToolCalls` 设置为 `0`，输入框最小值也是 `0`。
2. 用户据此会自然理解为：可以完全禁用 answer 阶段的额外工具调用，让 Answer Agent 只基于 Search Agent 结果收束答案。
3. 但运行时创建 multi-agent runner 时，会把 `self.config.agent.multi_agent.answer.max_tool_calls.max(1)` 传给 `MultiAgentRunner`。
4. 这意味着即使用户显式保存了 `0`，运行时也会把它提升成 `1`。
5. 后续 answer 阶段提示词和 `answer_request.max_tool_calls` 都基于这个被提升后的值，因此用户实际上无法得到“0 次补充工具调用”的行为。

## 期望效果

- 如果设置页允许输入 `0`，运行时就必须尊重 `0`，真正禁用 answer 阶段补充工具调用。
- 如果产品上不支持 `0`，设置页输入控件和文案应明确限制最小值为 `1`，不能让用户以为自己能关闭该能力。
- UI、配置文件和运行时的工具调用上限语义必须完全一致。

## 当前实现效果（问题发现时）

- 设置页输入控件在 `packages/app/src/pages/settings.tsx:548-558` 中明确允许 `min=\"0\"`，并会把 `0` 按正常数值写入草稿。
- 运行时在 `crates/hone-channels/src/core.rs:1047-1053` 中却使用 `self.config.agent.multi_agent.answer.max_tool_calls.max(1)`，把任何小于 `1` 的值都强行提升为 `1`。
- `MultiAgentRunner` 在 answer 阶段 handoff prompt 中会把这个提升后的值写进 “at most N extra tool call(s)” 提示，并同步写入 `answer_request.max_tool_calls`，见 `crates/hone-channels/src/runners/multi_agent.rs:343-358`。

## 用户影响

- 用户明明在设置页里把补充工具调用关掉了，但 multi-agent answer 阶段仍可能继续触发一次额外工具调用。
- 这会直接放大 multi-agent 和其它 runner 的体验差异，尤其在用户希望 answer 阶段“只整理、不再发散搜索”的场景下最明显。
- 由于 UI 看起来保存成功，用户很难意识到是运行时偷偷改写了配置语义。

## 根因判断

- 设置页和运行时对 `maxToolCalls=0` 的含义没有达成一致。
- 前端把 `0` 作为合法产品语义暴露给用户，而运行时把它视为非法/不支持并静默改成 `1`。
- 这属于典型的“配置可填，但语义不兑现”的行为偏差。

## 下一步建议

- 决定唯一语义：要么运行时尊重 `0`，要么前端禁止输入 `0` 并明确文案说明最小值是 `1`。
- 修复前应补一条回归：当用户保存 `max_tool_calls=0` 时，最终 answer 阶段不得再发生补充工具调用，或 UI 必须明确拒绝该配置。
- multi-agent 相关体验排查里，像这类“表单允许但运行时 silently 改写”的设置项值得继续系统清点一遍。
