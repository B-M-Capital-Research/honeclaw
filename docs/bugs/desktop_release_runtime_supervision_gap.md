# Bug: Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-15 用户真实故障与恢复过程
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

## 期望效果

- release runtime 应长期稳定地绑定 `127.0.0.1:8077`，除非显式切换配置，不应随机漂移到其它端口。
- backend、desktop 与启用渠道应由稳定 supervisor 或等价的长期运行托管方式维持，不应因为临时 shell / 启动上下文结束而整组进程消失。
- Desktop 在 remote backend 模式下应始终能连接到与当前运行态一致的固定地址，而不是出现“桌面还在，但 backend 已掉线”的分裂状态。

## 当前实现效果（问题发现时）

- `data/runtime/logs/desktop.log` 在 `2026-04-15 09:03:21` 到 `09:03:47` 持续记录 `127.0.0.1:8077/api/meta` 的 `Connection refused`。
- `data/runtime/logs/web.log` 在 `2026-04-15 09:02:37` 记录 `hone-console-page running at http://127.0.0.1:56044`，说明 backend 的实际监听端口曾与 desktop 约定端口脱节。
- 同一日志文件在 `09:02:56` 到 `09:06:44` 多次记录向 `http://127.0.0.1:8077/api/runtime/heartbeat` 发送失败，进一步证明系统内不同进程对 backend 地址的认知已经分叉。
- 本轮排查中，直接用短生命周期的后台启动方式时，runtime 无法稳定保持；改为 detached `screen` 持有完整进程组与显式环境后，release backend、desktop 与三个启用渠道才稳定恢复。

## 当前实现效果（2026-04-15 HEAD 复核）

- 当前源码和 runbook 已明确要求 release lane 固定使用 `.app/Contents/MacOS/hone-desktop` 与显式环境变量，但这套约束尚未被一个稳定的默认 supervisor 路径强制落实。
- `docs/runbooks/desktop-release-app-runtime.md` 也已经记录：如果启动路径没有稳定保留 `HONE_WEB_PORT=8077`，`hone-console-page` 可能静默回退到随机端口，进而让 desktop remote mode 失效。
- 当前运行态在 detached `screen` 模式下是健康的：
  - `/api/meta` 返回正常
  - `/api/channels` 显示 `web`、`feishu`、`telegram`、`discord` 都是 `running`
  - 运行中的 desktop 路径指向打包后的 `.app/Contents/MacOS/hone-desktop`
- 因此现阶段更像“release 运行托管缺口仍存在，但已找到可稳定复现的恢复方式”，而不是本轮代码变动正在持续触发热重启。

## 用户影响

- 用户感知是“系统时不时挂掉”或“桌面明明开着，但服务已经不可用”，直接影响主功能可用性。
- 当 backend 漂移到非 `8077` 端口或整组进程退出时，desktop remote mode、Web API、以及依赖 heartbeat 的启用渠道都会一起受影响。
- 由于故障更多发生在运行托管层而不是显式业务报错层，用户通常只能看到系统断连，却看不到一个明确的“为什么挂了”的前端提示。

## 根因判断

- 当前证据更支持“release runtime 启动/监督链路不稳定”，而不是“代码改动自动把正在运行的 release app 弄崩”。
- release runtime 对启动上下文过于敏感：一旦 `HONE_WEB_PORT` 没被稳定保留，backend 就可能回退到随机端口；一旦长期进程组没有被可靠托管，整组服务就可能在启动后消失。
- desktop remote backend 模式固定探测 `8077`，因此只要 backend 端口漂移或 backend 整体掉线，就会被放大成持续 `Connection refused` 与整机不可用。

## 下一步建议

- 为 release lane 收口到一个仓库内可复用、可验证的稳定 supervisor 入口，避免继续依赖易漂移的临时后台启动方式。
- 在正式托管收口前，继续沿用本轮验证通过的 detached `screen` + 显式环境变量方案，并把 `HONE_DATA_DIR` 固定到 `/Users/ecohnoch/Desktop/honeclaw/data`。
- 后续如果再次复现，优先对照三类证据一起看：`desktop.log` 的 probe 失败时间点、`web.log` 的实际监听端口、以及进程树是否仍由稳定 supervisor 持有。
