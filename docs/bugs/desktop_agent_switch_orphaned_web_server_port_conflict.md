# Bug: Desktop 基础设置切换 Agent 后旧内嵌 Web server 未停止，重启时撞上 8077 端口占用并让页面掉线

- **发现时间**: 2026-04-25
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-25 用户反馈：基础设置选择 Agent 配置后出现红色弹窗，提示 `后端未连接：无法绑定端口 127.0.0.1:8077: Address already in use (os error 48)`，随后页面挂掉
  - 2026-04-27 21:33:00-21:33:25 `data/runtime/logs/desktop.log` 连续 22 次报 `embedded web server start failed: 无法绑定端口 127.0.0.1:8077: Address already in use (os error 48)`；直到 `21:33:26.688` 才重新拿回端口
  - 2026-04-27 21:33:27-21:33:30 `data/runtime/logs/sidecar.log` 显示 bundled runtime 在同一轮恢复里再次批量拉起 `hone-feishu` / `hone-discord` / `hone-telegram`，说明这不是单次手工误操作，而是 runtime restart 路径仍会进入端口冲突后重试
  - 代码证据：
    - `crates/hone-web-api/src/lib.rs`
    - `bins/hone-desktop/src/sidecar.rs`
    - `bins/hone-desktop/src/sidecar/processes.rs`

## 端到端链路

1. 用户在 Desktop 基础设置里切换 Agent runner。
2. `set_agent_settings_impl(...)` 写入配置后，会把 bundled runtime 标记为 dirty，并调用 `connect_backend_serialized(...)` 让内置后端重启生效。
3. 重启前 `stop_managed_children(...)` 试图停止旧的 bundled runtime。
4. 但 `hone_web_api::start_server(...)` 启动 Axum 管理端、用户端、scheduler、event engine 等后台 task 后，返回值只包含 state 和端口，没有返回这些 task 的 `JoinHandle`。
5. Desktop sidecar 的 `web_server_task` 因此从未被赋值，`stop_web_server(...)` 实际无法 abort 旧的 Axum listener。
6. 下一次切换 Agent 或保存设置时，新 server 再次绑定固定管理端口 `127.0.0.1:8077`，旧 listener 仍占用端口，于是返回 `Address already in use`。
7. 前端收到 disconnected backend status 后显示红色错误，设置页与后续 API 请求一起失联。

## 期望效果

- 切换 Agent 时，旧的内嵌 Web API listener 必须在新 listener 绑定前真实停止。
- `8077` / `8088` 固定端口不应因为同一 desktop 进程内的 runtime restart 留下孤儿 listener。
- 设置页即使显示 runtime 重启失败，也不应由“旧 task 没停掉”造成必现端口冲突。

## 当前实现效果

- 2026-04-25 的修复后，这条缺陷一度被标记为 `Fixed`。
- 但 2026-04-27 21:33 的真实日志表明，bundled runtime 仍会在重启窗口里连续撞上 `127.0.0.1:8077`，并让 Desktop backend 在约 26 秒内反复启动失败。
- 最终虽然自动恢复，但故障窗口内前端会经历实际掉线，说明“旧 server/task 未完全释放或停止时序仍有竞态”这个问题仍未彻底消失。

## 用户影响

- Desktop 用户在切换配置、自动重连或 runtime 自恢复期间，仍可能看到后端断连、页面短时不可用或设置保存失败。
- 这类故障直接影响 Desktop 主入口可用性，因此继续维持 `P1`。

## 根因判断

- `StartedServer` 没有把 per-startup 后台 task handles 暴露给调用方。
- Desktop sidecar 虽然定义了 `web_server_task` 并在停止路径 abort，但启动路径没有任何 handle 可写入。
- 固定端口发布后，runner 切换从“重启但旧 listener 残留”升级为稳定复现的端口冲突。

## 修复情况

- 2026-04-25：
  - `crates/hone-web-api/src/lib.rs` 的 `StartedServer` 新增 `task_handles`，并收集本次 `start_server(...)` 创建的 UDP log server、event engine、scheduler、scheduler event handler、管理端 Axum、用户端 Axum task。
  - 管理端或用户端端口绑定失败时，会 abort 已经启动的 per-startup task，避免失败启动也留下后台任务。
  - `bins/hone-desktop/src/sidecar.rs` 改为把 `started.task_handles` 存入 `DesktopBackendManager`。
  - `bins/hone-desktop/src/sidecar/processes.rs` 的 `stop_web_server(...)` 现在会 abort 全部已记录 task，再释放 bundled web lock。
- 2026-04-27：
  - 最新真实窗口再次出现同型 `8077` 端口占用回归，说明既有修复没有覆盖全部 restart 时序；状态从 `Fixed` 改回 `New`，等待重新定位剩余竞态或重复启动入口。
  - 已登记 GitHub issue：[Issue #24](https://github.com/B-M-Capital-Research/honeclaw/issues/24)（脱敏摘要）

## 验证

- `cargo check -p hone-web-api`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop sidecar::tests -- --nocapture`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- `git diff --check`

说明：本次自动化验证覆盖了 web-api 启动返回 task handles、desktop sidecar 停止路径清理 handles、以及 Agent settings 重启链路的既有回归测试。完整 GUI 点击验证需要使用本次提交重新打包后的 Desktop app。

## 下一步建议

- 先回查 `21:33` 这轮 runtime restart 的触发入口，确认是 Desktop 自动重连、设置变更还是 release helper 反复拉起导致的重复启动。
- 在 `stop_web_server(...)` 与 `start_server(...)` 两端补充更细粒度的 task/port 生命周期日志，确认是旧 listener 未退出，还是新一轮启动被重复触发。
- 复核 `task_handles` 之外是否仍有未纳管的 server/task 会占住 `8077`，尤其是失败重试分支与多次 `connect_backend_serialized(...)` 竞态。
