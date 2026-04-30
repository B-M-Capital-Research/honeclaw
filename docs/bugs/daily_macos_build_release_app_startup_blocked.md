# Daily macOS build release app startup blocked

- 发现时间：2026-04-30 04:23 CST
- Bug Type：Desktop release runtime / daily build validation
- 严重等级：P1
- 状态：Fixed
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证，本地执行 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop` 后启动打包产物

## 端到端链路

1. 从 `/Users/ecohnoch/Desktop/honeclaw` 的 `main` 分支拉取远端最新代码，结果为 already up to date。
2. 使用 `CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target` 执行完整桌面构建与 Tauri 打包。
3. Tauri 成功生成：
   - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`
   - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.5.0_aarch64.dmg`
4. 基于当前 `config.yaml` 写入隔离验证配置 `data/runtime/daily-build-check/config.yaml`，并显式禁用：
   - `feishu.enabled=false`
   - `telegram.enabled=false`
   - `discord.enabled=false`
   - `imessage.enabled=false`
   - `discord_watch.enabled=false`
   - `event_engine.enabled=false`
5. 使用隔离端口 `HONE_WEB_PORT=18077`、`HONE_PUBLIC_WEB_PORT=18088` 启动打包后的 `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`。

## 期望效果

- release app 主进程启动后应拉起 embedded web runtime。
- `http://127.0.0.1:18077/api/meta` 应返回 JSON。
- public/user-facing 页面应在 `http://127.0.0.1:18088/chat` 返回 200。
- Feishu / Telegram / Discord / iMessage 等外部 IM 渠道在隔离配置下应保持 disabled 或未启动。

## 当前实现效果

- `.app` 与 `.dmg` 均已成功生成，且 mtime 位于本轮构建窗口内。
- release app 进程启动后卡在启动错误 dialog：标准错误输出出现 `CFUserNotificationDisplayAlert: called from main application thread, will block waiting for a response.`。
- 进程持有 `data/runtime/daily-build-check/data/runtime/locks/hone-desktop.lock`，但没有拉起 embedded backend。
- `18077` 与 `18088` 均无监听；`curl http://127.0.0.1:18077/api/meta` 返回 connection refused。
- `data/runtime/daily-build-check/data/runtime/effective-config.yaml` 已确认外部渠道均为 disabled，因此失败不来自真实 IM 渠道启动。

## 用户影响

- 每日 macOS 打包健康验证无法通过完整启动链路。
- 当前代码可以产出 `.app` 与 `.dmg`，但无法证明新打包桌面产物可用，阻断 macOS 产物交付信心。

## 根因判断

- 初步定位为 release app 在 Tauri setup / bundled runtime startup 阶段触发 `Hone Startup Blocked` 类 rfd dialog；该 dialog 在主线程阻塞，导致自动化无法继续启动 embedded web runtime。
- 当前 stdout/stderr 只能捕获到 macOS `CFUserNotificationDisplayAlert` 阻塞提示，具体 dialog message 未落入 `data/runtime/daily-build-check/data/runtime/logs/desktop.log`，需要补充启动失败日志或让 dialog 错误写入 stderr / desktop log。
- 隔离 runtime 目录中仅出现 `hone-desktop.lock`，未出现 `hone-console-page.lock`，说明失败发生在 backend 成功启动之前。

## 下一步建议

1. 下一次每日完整打包验证应重新执行 `.app` / `.dmg` / `/api/meta` / public page / channel disabled 链路，确认 release app 不再被 UI dialog 挂起。
2. 若仍无法拉起 backend，优先查看隔离 runtime 的 `desktop.log` 与 stderr 中的 `desktop startup blocked before backend bootstrap` 记录，而不是只依赖 macOS dialog。
3. 若失败原因指向真实端口或锁冲突，再按 `docs/runbooks/desktop-release-app-runtime.md` 的 stale lock / pid 清理流程定位。

## 修复进展（2026-04-30）

- `bins/hone-desktop/src/sidecar.rs` 将 startup error dialog 改为后台线程显示，避免 `CFUserNotificationDisplayAlert` 在主线程阻塞自动化启动/退出路径。
- `bins/hone-desktop/src/commands.rs` 在 setup 阶段 `prepare_desktop_startup(...)` 失败时先写入 desktop runtime log，并同步输出 stderr：`Hone Startup Blocked: ...`。每日验证后续即使仍失败，也能拿到可判定错误文本。
- 新增 `HONE_SUPPRESS_STARTUP_DIALOG=1` / `CI=1` 抑制开关，供无交互 smoke test 或 CI 环境显式禁用原生 dialog。
- 新增回归测试覆盖 dialog 走非阻塞 spawn 路径，以及抑制开关的 truthy 值解析。

## 验证结果

- 2026-04-30 04:23 原始每日构建：
  - 构建打包：通过。
  - `.app` 存在性：通过，mtime `2026-04-30 04:14:01 CST`。
  - `.dmg` 存在性：通过，mtime `2026-04-30 04:14:40 CST`。
  - 隔离配置生成：通过，外部渠道均为 disabled。
  - release app 启动：失败，进程卡在 startup dialog。
  - API 验证：失败，`18077` connection refused。
  - public 页面验证：失败，`18088` connection refused。
  - 清理结果：已终止本轮验证进程并移除本轮 `hone-desktop.lock`；未停止或改动已有日常 Hone 进程。
- 2026-04-30 11:09 修复验证：
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop startup_error_dialog -- --nocapture`
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`
  - `git diff --check`
