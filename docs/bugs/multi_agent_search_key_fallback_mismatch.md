# Bug: Multi-Agent Search Agent 在 Desktop 设置页显示可继承 auxiliary key，但真实运行时不使用该 fallback，导致看似已配置却直接失败

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `bins/hone-desktop/src/sidecar/settings.rs:4-25`
    - `packages/app/src/pages/settings.tsx:430-478`
    - `crates/hone-channels/src/runners/multi_agent.rs:54-67`
    - `crates/hone-channels/src/core.rs:1047-1050`

## 端到端链路

1. Desktop 设置页会把 Multi-Agent Search Agent 的配置展示给用户，其中 `search.apiKey` 在 UI seed 阶段允许从 `llm.auxiliary.api_key` 回填。
2. 因此对某些用户来说，即使 `agent.multi_agent.search.api_key` 为空，设置页里仍可能显示出一个“已经有值”的 Search Agent key，或者测试链路看起来可以继续操作。
3. 但真实运行 multi-agent 时，`HoneBotCore::create_runner()` 会把原始 `self.config.agent.multi_agent.search.clone()` 直接传给 `MultiAgentRunner`。
4. `MultiAgentRunner::build_search_provider()` 只检查 `search_config.api_key` 本身；如果这个字段为空，就直接返回 `multi-agent search agent API key 为空`。
5. 结果是：Desktop 设置页的“Search Agent 似乎已配置好”和运行时的“Search Agent 直接报空 key”会出现明显割裂。

## 期望效果

- Multi-Agent Search Agent 的 UI 展示、测试入口和真实运行时应该共享同一套 key fallback 语义。
- 如果产品希望 Search Agent 能继承 `llm.auxiliary.api_key`，运行时也必须真正使用这个 fallback。
- 如果产品不希望继承，那么设置页就不应显示或暗示 Search Agent 已经具备该 key。

## 当前实现效果（问题发现时）

- `seed_multi_agent_settings()` 在 `bins/hone-desktop/src/sidecar/settings.rs:11-14` 中，当 `agent.multi_agent.search.api_key` 为空时，会回填 `config.llm.auxiliary.api_key` 到 Desktop 设置页草稿。
- 设置页 Search Agent 区块直接绑定这个草稿值，见 `packages/app/src/pages/settings.tsx:430-478`。
- 但运行时创建 multi-agent runner 时，在 `crates/hone-channels/src/core.rs:1047-1050` 传入的是原始 `self.config.agent.multi_agent.search.clone()`，没有把 UI seed fallback 合并回真实配置。
- `MultiAgentRunner::build_search_provider()` 在 `crates/hone-channels/src/runners/multi_agent.rs:54-67` 中只接受 `search_config.api_key`，为空就直接失败。

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前 `HEAD` 仍在 `bins/hone-desktop/src/sidecar/settings.rs:11-14` 用 `llm.auxiliary.api_key` 回填 Search Agent 草稿。
- 运行时仍在 `crates/hone-channels/src/core.rs:1004-1015` 仅对 OpenRouter answer 路径读取 `effective_key_pool()`，没有把 auxiliary fallback 应用到 `agent.multi_agent.search`。
- `crates/hone-channels/src/runners/multi_agent.rs:57` 仍会在 search key 为空时直接返回 `multi-agent search agent API key 为空`。
- 这部分描述记录的是修复前的 HEAD 复核结论；当前状态以文档顶部 `Fixed` 和下方“修复情况（2026-04-16）”为准。

## 用户影响

- 用户会看到非常误导的状态：设置页里 Search Agent 看起来有 key，但真正执行 multi-agent 时会首轮就失败。
- 这种失败发生在 multi-agent 搜索阶段入口，用户只会感知到“multi-agent 特别不稳定 / 特别难配”，体验明显差于其它 runner。
- 排障时如果只看设置页，很容易误判为 provider 抖动，而不是 fallback 语义前后不一致。

## 根因判断

- Desktop 设置页的 seed 逻辑引入了 `auxiliary -> multi-agent.search` 的 UI fallback。
- 运行时 `create_runner()` 和 `MultiAgentRunner` 并没有遵循同一套 fallback 规则，而是严格读取原始 `agent.multi_agent.search.api_key`。
- 这导致“展示层”和“执行层”对 Search Agent 是否已配置给出相互矛盾的答案。

## 下一步建议

- 明确 Search Agent key 的唯一真相源：要么运行时也支持 `llm.auxiliary.api_key` fallback，要么设置页停止回填这类继承值。
- 修复前可补一条最小回归：当 `agent.multi_agent.search.api_key` 为空且 `llm.auxiliary.api_key` 非空时，UI 展示与实际运行结果必须一致，不能一边显示可用、一边运行报空 key。
- multi-agent 排障时，应单独核对 `agent.multi_agent.search.api_key` 的实际落盘值，而不能只看 Desktop 设置页展示值。

## 修复情况（2026-04-16）

- `crates/hone-channels/src/core.rs` 已为 multi-agent 运行时补上与 Desktop 设置页一致的 Search Agent key fallback 语义：
  - 当 `agent.multi_agent.search.api_key` 为空时，运行时会回退到 `llm.auxiliary.resolved_api_key()`
  - 如果 Search Agent 显式配置了自己的 key，运行时仍优先使用显式 search key，不会被 auxiliary 覆盖
- `create_runner_with_model_override(...)` 在构建 `MultiAgentRunner` 时，已从直接传 `self.config.agent.multi_agent.search.clone()` 改为传入统一收口后的 effective search config
- 这次修复没有改 Desktop 设置页展示层；而是把真实执行层对齐到现有 UI 继承语义，避免“页面看起来已配置、运行时报空 key”
- 新增运行时回归测试：
  - `core::tests::effective_multi_agent_search_config_falls_back_to_auxiliary_api_key`
  - `core::tests::effective_multi_agent_search_config_preserves_explicit_search_api_key`
- 验证命令：
  - `cargo test -p hone-channels effective_multi_agent_search_config_falls_back_to_auxiliary_api_key -- --nocapture`
  - `cargo test -p hone-channels effective_multi_agent_search_config_preserves_explicit_search_api_key -- --nocapture`
