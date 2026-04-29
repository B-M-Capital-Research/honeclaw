# Daily macOS build release app startup blocked

- 发现时间：2026-04-30 04:23 CST
- Bug Type：Desktop release runtime / daily build validation
- 严重等级：P1
- 状态：New
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

1. 在 `prepare_desktop_startup` 或 `show_startup_error_dialog` 前后补充非 UI 日志，确保启动阻塞的具体 message 写入 runtime `desktop.log` 或 stderr。
2. 复核 packaged app 并行启动时是否会受已有 `/Applications/Hone Financial.app` 日常实例、bundle id、或启动锁判断影响；本轮未停止既有真实桌面进程。
3. 为每日验证增加可自动化的 startup failure probe：错误 dialog 不应是唯一证据出口。
4. 修复后重新执行 `honeclaw-mac` 完整链路，要求 `.app` / `.dmg` / `/api/meta` / public page / channel disabled 状态全部通过。

## 验证结果

- 构建打包：通过。
- `.app` 存在性：通过，mtime `2026-04-30 04:14:01 CST`。
- `.dmg` 存在性：通过，mtime `2026-04-30 04:14:40 CST`。
- 隔离配置生成：通过，外部渠道均为 disabled。
- release app 启动：失败，进程卡在 startup dialog。
- API 验证：失败，`18077` connection refused。
- public 页面验证：失败，`18088` connection refused。
- 清理结果：已终止本轮验证进程并移除本轮 `hone-desktop.lock`；未停止或改动已有日常 Hone 进程。
