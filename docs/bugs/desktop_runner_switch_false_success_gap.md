# Bug: Desktop 设置页切换 runner 后可能显示已切换，但 bundled runtime 重启失败会被静默吞掉，实际仍跑旧 runner 或未完成切换

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-15 当前源码复核
  - 代码证据:
    - `packages/app/src/pages/settings.tsx:321-339`
    - `packages/app/src/lib/backend.ts:197-202`
    - `packages/app/src/context/backend.tsx:304-316`
    - `bins/hone-desktop/src/sidecar.rs:957-981`
    - `crates/hone-web-api/src/routes/chat.rs:169-172`

## 端到端链路

1. 用户在 Desktop 设置页切换 runner，前端立即调用 `saveDesktopAgentSettings(...)`。
2. sidecar 会先把新的 `agent.runner` 写入配置文件，再在 bundled 模式下尝试通过 `connect_backend_serialized(...)` 重启内置后端以立即生效。
3. 但这条重启结果在后端被 `let _ = connect_backend_serialized(...)` 直接丢弃；即使重启失败，`set_agent_settings_impl(...)` 仍然返回 `Ok(())`。
4. 前端 `saveDesktopAgentSettings(...)` 只声明返回 `void`，设置页也没有像 channel settings 那样拿到 `backendStatus` 并刷新 backend 连接状态。
5. 结果是：UI 很可能已经显示新的 runner 配置，但当前正在服务聊天请求的 backend 未必已经成功切到新 runner。
6. 聊天流的 `run_started` 事件仍直接暴露 `arc.core.config.agent.runner`，因此用户会看到实际运行中的 runner 继续是旧值，形成“设置改了，但实际还是上一个 runner”的故障感知。

## 期望效果

- Desktop 设置页在保存 runner 后，应明确知道 bundled backend 是否已成功重启并实际载入新配置。
- 如果 backend 重启失败，前端应展示错误或至少显示“配置已写入，但当前 runtime 尚未生效”，而不是把本次切换当成成功完成。
- bundled runtime 成功切换后，设置页和实际聊天运行时看到的 runner 应保持一致。

## 当前实现效果（问题发现时）

- 后端 `set_agent_settings_impl(...)` 在 `bins/hone-desktop/src/sidecar.rs:957-981` 中会先写配置，再尝试 bundled backend 重启，但使用 `let _ = connect_backend_serialized(&app, &state).await;` 静默吞掉结果。
- 前端 `saveDesktopAgentSettings(...)` 在 `packages/app/src/lib/backend.ts:201-202` 只接收 `void`，没有承接 backend status。
- 设置页 `selectRunner(...)` 和 `submitAgentSettings(...)` 在 `packages/app/src/pages/settings.tsx:321-339` 中都只关心“invoke 是否抛错”；其中 `selectRunner(...)` 甚至会静默吞掉异常。
- 作为对比，channel settings 已在 `packages/app/src/context/backend.tsx:304-316` 中通过 `backendStatus` 回写 frontend runtime 状态，而 agent settings 没有这套收口。
- 聊天实际使用的 runner 来自运行中 backend 的 `arc.core.config.agent.runner`，见 `crates/hone-web-api/src/routes/chat.rs:169-172`；因此当 runtime 未成功重启或尚未切换完成时，用户会继续看到旧 runner。

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前 `HEAD` 仍在 `bins/hone-desktop/src/sidecar.rs:980` 以 `let _ = connect_backend_serialized(&app, &state).await;` 的方式吞掉 runner 保存后的 bundled backend 重启结果。
- `packages/app/src/lib/backend.ts:201` 的 `saveDesktopAgentSettings(...)` 仍只返回 `void`，没有把 runtime 重启状态带回前端。
- `packages/app/src/pages/settings.tsx:322-338` 的 `selectRunner(...)` / `submitAgentSettings(...)` 仍只根据是否抛错判断保存结果，未展示“配置写入成功但 runtime 未切换”的中间态。
- 本轮巡检未发现把 backend status 回传到 agent settings 保存链路的修复，因此该缺陷继续保持 `New`。

## 用户影响

- 用户会遇到非常误导的状态：设置页看起来已经选中了新 runner，但新对话仍由旧 runner 执行。
- 由于保存路径把 backend 重启失败吞掉，用户和排障者都很难第一时间判断问题出在“配置未写入”还是“runtime 未重启成功”。
- 这会直接破坏 Desktop 设置页“修改后立即生效”的产品承诺。

## 根因判断

- agent settings 保存链路没有把 “配置写入成功” 和 “bundled backend 已成功按新配置重启” 区分成两个结果层级。
- 后端静默吞掉了 runtime 重启结果，前端也没有像 channel settings 那样同步 backend status。
- 因此前端只能看到新配置值，不能确认实际运行中的 backend 是否已经切到新 runner。

## 下一步建议

- 让 agent settings 保存命令返回和 channel settings 一致的 backend status / restart 结果，而不是 `void`。
- 如果 bundled backend 重启失败，前端应明确展示“未立即生效”的状态，并允许用户手动重连或重试。
- 排查时可用聊天流 `run_started.runner` 作为实际生效 runner 的外显证据，避免只看设置页选中状态。
