# Bug: Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **修复提交**:
  - `ea5229b fix release launch runtime contract`
- **证据来源**:
  - 2026-04-15 用户真实故障与恢复过程
  - 2026-04-16 最近会话“无回复”排查
  - 日志证据:
    - `data/runtime/logs/desktop.log`
    - `data/runtime/logs/web.log`
    - `data/runtime/logs/backend_release_restart.log`
  - 运行约束:
    - `docs/runbooks/desktop-release-app-runtime.md`

## 端到端链路

1. Desktop release app 在 remote backend 模式下固定探测 `http://127.0.0.1:8077/api/meta`，并要求 backend 与启用渠道持续存活。
2. 2026-04-15 的实际故障中，desktop 日志在 `09:03:21` 起连续记录 `remote backend probe failed ... 127.0.0.1:8077/api/meta`，表现为桌面壳还在，但远端 backend 已不可用。
3. 同一时间窗里，`data/runtime/logs/web.log` 记录 `2026-04-15 09:02:37` backend 曾启动到 `http://127.0.0.1:56044`，而不是固定的 `8077`；随后又出现多条向 `http://127.0.0.1:8077/api/runtime/heartbeat` 发送失败的警告。
4. 这说明 release runtime 的某条启动/托管路径没有稳定保住 `HONE_WEB_PORT=8077` 与长期进程生命周期，导致 desktop 继续探测 `8077` 时只能看到 `Connection refused`。
5. 本轮恢复把 backend、channels、desktop 全部放进显式导出环境变量的 detached `screen` 会话后，`/api/meta` 与 `/api/channels` 恢复正常，当前未再观察到同类掉线。
6. 2026-04-16 09:01 的最近一次真实排查里，Feishu 直聊会话 `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 已正常完成 search，并在 `09:01:30` 进入 `opencode` answer 阶段发送 `session/prompt`。
7. 但这条链路在出现 `multi_agent.answer.done`、`session.persist_assistant` 或 `reply.send` 之前，`web.log` 于 `09:01:53` 直接出现新的 backend 启动序列，说明 backend 进程在 answer 执行中途被替换或重启。
8. 同一重启窗口内，两个 scheduler 会话 `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 与 `Actor_feishu__direct__ou_5f95ab3697246ded86446fcc260e27e1e2` 也停留在 `answer.start` 之后，最终都没有 assistant 落库。

## 期望效果

- release runtime 应长期稳定地绑定 `127.0.0.1:8077`，除非显式切换配置，不应随机漂移到其它端口。
- backend、desktop 与启用渠道应由稳定 supervisor 或等价的长期运行托管方式维持，不应因为临时 shell / 启动上下文结束而整组进程消失。
- Desktop 在 remote backend 模式下应始终能连接到与当前运行态一致的固定地址，而不是出现“桌面还在，但 backend 已掉线”的分裂状态。

## 当前实现效果（问题发现时）

- `data/runtime/logs/desktop.log` 在 `2026-04-15 09:03:21` 到 `09:03:47` 持续记录 `127.0.0.1:8077/api/meta` 的 `Connection refused`。
- `data/runtime/logs/web.log` 在 `2026-04-15 09:02:37` 记录 `hone-console-page running at http://127.0.0.1:56044`，说明 backend 的实际监听端口曾与 desktop 约定端口脱节。
- 同一日志文件在 `09:02:56` 到 `09:06:44` 多次记录向 `http://127.0.0.1:8077/api/runtime/heartbeat` 发送失败，进一步证明系统内不同进程对 backend 地址的认知已经分叉。
- 本轮排查中，直接用短生命周期的后台启动方式时，runtime 无法稳定保持；改为 detached `screen` 持有完整进程组与显式环境后，release backend、desktop 与三个启用渠道才稳定恢复。
- 2026-04-16 09:01 的最新样本进一步证明：这个缺陷不只表现为“desktop 探测 8077 失败”，还会直接截断正在执行中的 answer 请求，导致用户看到“最后一条消息一直没回复”。
- 同一次窗口里，`web.log` 在 `09:01:30` 仍能看到多个会话已进入 `opencode session/prompt`，但 `09:01:53` 后日志直接切到新 backend 的 startup banner，中间没有对应会话的完成事件。

## 修复情况（2026-04-16 HEAD）

- `launch.sh --release` 现在会通过 `bun run build:desktop` 产出正式 Tauri bundle，并在 macOS 上直接启动 `Hone Financial.app/Contents/MacOS/hone-desktop`，不再把裸 `target/release/hone-desktop` 当成正式 release 运行态。
- `launch.sh` 现在会把自身 supervisor pid 写入 `data/runtime/current.pid`，使 `restart_hone` 和其它后台重启链路可以先终止旧 supervisor，再清理整组 child process，避免旧实例未停、新实例又启动造成的 split-brain、端口冲突和中途替换。
- `launch.sh` 与 `scripts/build_desktop.sh` 的默认 `CARGO_TARGET_DIR` 已统一到 `~/Library/Caches/honeclaw/target`，不再与 runbook 认可的 cache target 目录分叉；这减少了“重建了错误 target 树，运行中的 release app / backend 仍旧是旧二进制”的误操作窗口。
- 新增 CI-safe 回归 `tests/regression/ci/test_release_launch_runtime_contract.sh`，锁住三条关键契约：
  - release helper 必须指向 `.app/Contents/MacOS/hone-desktop`
  - 默认 cache target 必须是 `honeclaw/target`
  - `launch.sh` 必须持续写入 `data/runtime/current.pid`
- 因此，这条缺陷里最核心的“缺少稳定、可复用、受 runbook 约束的 release 启动入口”已经收口；后续如果再出现掉线，更应先排查新的子进程崩溃或渠道级故障，而不是继续沿用旧的启动链路缺口结论。

## 用户影响

- 用户感知是“系统时不时挂掉”或“桌面明明开着，但服务已经不可用”，直接影响主功能可用性。
- 在更隐蔽的形态下，用户不会立刻看到掉线，而是会看到某条消息永远卡在“处理中”或“始终没有最终回复”，因为 answer 阶段中的 backend 被直接切断。
- 当 backend 漂移到非 `8077` 端口或整组进程退出时，desktop remote mode、Web API、以及依赖 heartbeat 的启用渠道都会一起受影响。
- 由于故障更多发生在运行托管层而不是显式业务报错层，用户通常只能看到系统断连，却看不到一个明确的“为什么挂了”的前端提示。

## 根因判断

- 当前证据更支持“release runtime 启动/监督链路不稳定”，而不是“代码改动自动把正在运行的 release app 弄崩”。
- release runtime 对启动上下文过于敏感：一旦 `HONE_WEB_PORT` 没被稳定保留，backend 就可能回退到随机端口；一旦长期进程组没有被可靠托管，整组服务就可能在启动后消失。
- 现有链路也缺少“backend 正在替换/重启时，如何处理飞行中的 answer 请求”的保护，因此进程级中断会直接表现成无 assistant 落库、无错误回填、无最终发送。
- desktop remote backend 模式固定探测 `8077`，因此只要 backend 端口漂移或 backend 整体掉线，就会被放大成持续 `Connection refused` 与整机不可用。

## 回归验证

- `bash -n launch.sh`
- `bash -n scripts/build_desktop.sh`
- `bash tests/regression/ci/test_release_launch_runtime_contract.sh`
- `bash tests/regression/run_ci.sh`

## 结论

- release runtime 现在有了仓库内可复用、默认对齐 runbook 的正式入口：统一 cache target、正式 `.app` 启动形态、以及可被重启工具引用的 supervisor pid 契约。
- 若后续仍复现“answer 中途断掉”或“Desktop 还在但 backend 掉了”，应转向新的崩溃证据排查，而不是继续把问题归因于旧的 release 启动方式。
