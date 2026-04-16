# Bug: Multi-Agent Answer Agent 在设置页允许 `maxToolCalls=0`，但运行时强制提升为至少 1，用户无法真正禁用补充工具调用

- **发现时间**: 2026-04-15
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: Fixed
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

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前 `HEAD` 仍在 `packages/app/src/pages/settings.tsx:550-557` 允许用户输入 `0` 并把它保存进草稿。
- 运行时仍在 `crates/hone-channels/src/core.rs:1052` 使用 `max_tool_calls.max(1)`，没有尊重 `0` 的配置值。
- 本轮巡检时尚未发现收紧前端最小值或放宽运行时语义的修复；随后在同日自动修复轮次中完成了下面记录的本地代码修复。

## 修复情况（2026-04-16）

- `crates/hone-channels/src/core.rs` 已去掉 multi-agent answer `max_tool_calls` 的 `.max(1)` 强制提升，运行时现在会原样保留 `0`。
- `crates/hone-channels/src/runners/multi_agent.rs` 的 answer-stage handoff 文本不再把“最多一次补充工具调用”写死，而是和配置值保持一致；`0` 会明确传达为 `at most 0 supplemental tool call(s)`。
- 这样前端设置页、落盘配置和运行时行为重新对齐：当用户显式设置 `maxToolCalls=0` 时，Answer Agent 将真正禁用补充工具调用，而不是被静默提升到 `1`。

## 回归验证

- `cargo test -p hone-channels handoff_text_respects_zero_supplemental_tool_limit -- --nocapture`
- `cargo test -p hone-channels multi_agent_answer_zero_tool_limit_is_preserved -- --nocapture`
- `cargo test -p hone-channels multi_agent -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/core.rs crates/hone-channels/src/runners/multi_agent.rs`
- `git diff --check`

## 用户影响

- 用户明明在设置页里把补充工具调用关掉了，但 multi-agent answer 阶段仍可能继续触发一次额外工具调用。
- 这会直接放大 multi-agent 和其它 runner 的体验差异，尤其在用户希望 answer 阶段“只整理、不再发散搜索”的场景下最明显。
- 由于 UI 看起来保存成功，用户很难意识到是运行时偷偷改写了配置语义。

## 根因判断

- 设置页和运行时对 `maxToolCalls=0` 的含义没有达成一致。
- 前端把 `0` 作为合法产品语义暴露给用户，而运行时把它视为非法/不支持并静默改成 `1`。
- 这属于典型的“配置可填，但语义不兑现”的行为偏差。

## 后续建议

- 如果后续还需要把这条配置语义上升到更贴近真实执行链路的证明，可以再补一条 integration/regression，验证 Answer Agent 在 `max_tool_calls=0` 时不会发起任何新的 MCP tool call。
