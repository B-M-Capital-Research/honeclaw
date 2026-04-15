# Bug: Desktop 设置页重复点击 runner 会触发重入保存与 bundled backend 重启，导致切换过程卡死或表现为“点一下就崩”

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `packages/app/src/pages/settings.tsx:321-329`
    - `bins/hone-desktop/src/sidecar.rs:682-688`
    - `bins/hone-desktop/src/sidecar.rs:970-981`

## 端到端链路

1. 用户进入 Desktop 设置页，在“基础设置”里点击某个 runner 卡片。
2. 前端 `selectRunner(...)` 每次点击都会立刻执行 `saveDesktopAgentSettings(next)`，没有判断“是否点的是当前 runner”，也没有用 `agentSaving` 或其它互斥状态阻止连续点击。
3. Desktop sidecar 在 bundled 模式下收到每一次保存请求后，都会把 runtime 标记为 dirty，并调用 `connect_backend_serialized(...)` 重启内置后端。
4. 当用户快速重复点击同一张卡片，或在上一次切换尚未完成时再次点击，会连续排队触发多次“保存配置 + bundled backend 重连”。
5. 用户侧最终感知为：设置页切换 runner 时明显卡顿、第二次点击后页面像“崩掉”、或切换过程不稳定。

## 期望效果

- 点击已选中的 runner 不应重复触发保存和 backend 重启。
- runner 切换期间应禁止再次发起同类切换，至少要做到串行、幂等、可见反馈明确。
- 设置页不应因为重复点击同一张 runner 卡片而引发 backend 反复重启。

## 当前实现效果（问题发现时）

- `selectRunner(...)` 在 `packages/app/src/pages/settings.tsx:321-329` 中无条件保存，即使用户点击的是当前已选中的 runner，也会再次发请求。
- 这条“点击即保存”的路径没有设置 `agentSaving`，也没有把页面交互切到“切换中”状态；因此用户可以在同一轮切换尚未完成时继续点击。
- `set_agent_settings_impl(...)` 在 `bins/hone-desktop/src/sidecar.rs:970-981` 中对 bundled 模式始终执行一次 `connect_backend_serialized(...)`，意味着每次点击都伴随一次内置后端重启。
- `connect_backend_serialized(...)` 虽然会在 `bins/hone-desktop/src/sidecar.rs:682-688` 内串行化 backend 过渡，但当前前端仍会把重复点击排成一串连续重启任务，最终在用户视角表现为切换卡死、掉线或“再点一下就崩”。

## 用户影响

- 用户在设置页切换 runner 时很容易因为重复点击触发不必要的多次 backend 重启。
- 这类问题发生在核心入口设置页，直接影响“切换执行引擎”这一主流程，故障感知非常强。
- 对 bundled 模式用户来说，这会干扰当前桌面内置后端连接，导致“设置页崩溃 / 卡住 / 无响应”的直接体验问题。

## 根因判断

- 前端把 runner 卡片点击设计成“立即保存”，但没有做“同值点击短路”或“保存中禁止重复触发”。
- 后端保存链路把每次设置都当成需要立即重启 bundled backend 的真实变更处理，缺少前端/后端任一侧的幂等保护。
- `transition_lock` 只能保证后台串行过渡，不能避免前端把无意义的重复切换排队提交。

## 下一步建议

- 前端 runner 卡片点击应先判断目标值是否已与当前 runner 相同；相同则直接短路，不触发保存。
- runner 自动保存链路应复用统一的 “saving / switching” 状态，在切换未完成时禁止再次点击。
- 如需保留“点击即保存”，建议把这条路径的返回结果显式反馈到 UI，避免用户误以为第一次点击没生效而重复触发。
