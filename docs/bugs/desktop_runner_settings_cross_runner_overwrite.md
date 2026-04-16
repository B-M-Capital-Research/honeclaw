# Bug: Desktop Agent 设置会把 `multi-agent.answer` 反写到 `agent.opencode`，导致不同 runner 的独立配置互相覆盖

- **发现时间**: 2026-04-15
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `packages/app/src/pages/settings-model.ts:22-49`
    - `bins/hone-desktop/src/sidecar.rs:831-842`
    - `bins/hone-desktop/src/sidecar.rs:860-870`
    - `bins/hone-desktop/src/sidecar.rs:897-955`
    - `packages/app/src/pages/settings.tsx:486-568`
    - `packages/app/src/pages/settings.tsx:589-728`

## 端到端链路

1. Desktop 设置页同时暴露了两套看起来独立的配置区域：
   - OpenCode / OpenAI-compatible runner 使用的 `openaiUrl` / `openaiModel` / `openaiApiKey`
   - `multi-agent` 的 `Answer Agent` 使用的 `multiAgent.answer.*`
2. 前端默认草稿 `defaultAgentSettings()` 总是带着一个非空的 `multiAgent` 结构，即使当前 runner 不是 `multi-agent`。
3. 后端保存时，先把 `openaiUrl` / `openaiModel` / `openaiApiKey` 写入 `agent.opencode.*`。
4. 但只要 `settings.multi_agent` 存在，后端又会在同一次保存里把 `agent.opencode.api_base_url` / `api_key` / `model` / `variant` 再覆盖成 `multi_agent.answer.*`。
5. 结果是：用户在 OpenCode 区块中保存的 runner 专属配置，最终会被多代理 Answer 区块的值静默覆盖；两个看起来独立的设置面板实际上共用同一份落盘字段。

## 期望效果

- `opencode_acp` / OpenAI-compatible runner 的配置应与 `multi-agent.answer` 保持清晰边界，不应在一次保存里互相反写。
- 用户在设置页看到两个独立区域时，应能可靠地分别保存这两套配置。
- 切换 runner 时，未启用的另一套配置不应偷偷改写当前 runner 的真实落盘值。

## 当前实现效果（问题发现时）

- `defaultAgentSettings()` 在 `packages/app/src/pages/settings-model.ts:22-49` 中默认总会附带 `multiAgent.answer`，并非按当前 runner 懒加载。
- `get_agent_settings_impl()` 在 `bins/hone-desktop/src/sidecar.rs:831-842` 中也总是返回 `Some(seed_multi_agent_settings(&config))`，因此前端保存请求几乎总会带着 `multi_agent`。
- `set_agent_settings_impl()` 在 `bins/hone-desktop/src/sidecar.rs:860-870` 先写 `agent.opencode.* = openai*`，随后又在 `bins/hone-desktop/src/sidecar.rs:897-955` 把 `agent.opencode.*` 覆盖成 `multi_agent.answer.*`。
- 设置页文案和表单布局却把这两套字段拆成了两块独立 UI：`Answer Agent` 区块位于 `packages/app/src/pages/settings.tsx:486-568`，OpenAI-compatible / OpenCode 区块位于 `packages/app/src/pages/settings.tsx:589-728`。这会直接误导用户以为二者独立。

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前 `HEAD` 仍在 `bins/hone-desktop/src/sidecar.rs:861-869` 先写 `agent.opencode.* = openai*`。
- 同一次保存里，`bins/hone-desktop/src/sidecar.rs:940-953` 仍会把 `agent.opencode.*` 再覆盖成 `multi_agent.answer.*`。
- `packages/app/src/pages/settings-model.ts:26-28` 仍默认给 OpenAI-compatible runner 草稿填入独立的 `openaiUrl` / `openaiModel` / `openaiApiKey`，继续强化了“这是另一套独立配置”的 UI 预期。
- 本轮巡检未发现拆分持久化字段或按当前 runner 条件写入的修复，因此该缺陷继续保持 `New`。

## 用户影响

- 用户在 `opencode_acp` runner 下修改模型、Base URL 或 API Key，保存后可能马上被 `multi-agent.answer` 的旧值覆盖，表现为“明明改了，但下次运行还是错模型/错路由”。
- 用户在调多代理 Answer Agent 参数时，也会连带改写普通 OpenCode runner 的配置，导致切换 runner 后行为漂移。
- 这类问题会让“runner 切换”和“模型/路由切换”同时失真，排障成本很高。

## 根因判断

- 当前设置数据结构把 `agent.opencode` 与 `multi-agent.answer` 当成两套可独立编辑的 UI 状态，但保存逻辑又把它们收束到同一组落盘字段。
- 后端保存没有按照“当前正在编辑的是哪一套 runner 配置”做条件写入，而是无差别地同时写两套，并让 `multi_agent.answer` 后写覆盖 `agent.opencode`。
- 因此前端表意和后端持久化语义发生了明显冲突。

## 下一步建议

- 明确 `agent.opencode` 与 `multi-agent.answer` 的产品契约：如果必须共享同一路由，应在 UI 上合并为单一真相源；如果应独立，则必须拆开落盘字段并停止互相覆盖。
- 在修复前，可先补一条最小回归：当 `openai*` 与 `multi_agent.answer.*` 取不同值时，保存后 `agent.opencode.*` 不应被后者静默改写。
- 后续验证 runner 生效时，不能只看 `agent.runner`，还要核对落盘后的 `agent.opencode.*` 是否仍与用户刚保存的那组值一致。

## 修复情况（2026-04-16）

- `bins/hone-desktop/src/sidecar.rs` 已去掉把 `multi_agent.answer.*` 反写到 `agent.opencode.*` 的那组更新
- desktop 设置保存现在分为两套独立落盘：
  - `openaiUrl` / `openaiModel` / `openaiApiKey` -> `agent.opencode.*`
  - `multiAgent.answer.*` -> `agent.multi_agent.answer.*`
- 新增回归测试 `sidecar::tests::build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`，验证两套值不同的时候不会再互相覆盖
- 回归验证：
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`
