# Daily macOS build release app API not persistent

- 发现时间：2026-05-15 04:17 CST
- Bug Type：Desktop release runtime / daily macOS build verification
- 严重等级：P1
- 状态：Fixed
- GitHub Issue：[#42](https://github.com/B-M-Capital-Research/honeclaw/issues/42)
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证

## 修复记录（2026-05-15 08:07 CST）

- `bins/hone-desktop/src/commands.rs` 新增显式 smoke server 模式：设置 `HONE_DESKTOP_SMOKE_SERVER=1` 后，打包出的 `.app/Contents/MacOS/hone-desktop` 会绕过 Tauri 窗口生命周期，直接启动同进程 `hone_web_api::start_server(...)` 并保持进程直到 Ctrl-C。
- smoke 模式会按 `HONE_CONFIG_PATH` -> `HONE_USER_CONFIG_PATH` 选择配置，按 `HONE_DESKTOP_DATA_DIR` -> `HONE_DATA_DIR` 选择数据目录，并强制设置 `HONE_DISABLE_AUTO_OPEN=1`，因此每日验证可以继续使用隔离 config、固定 `HONE_WEB_PORT` / `HONE_PUBLIC_WEB_PORT` 和 disabled channels。
- 本地 smoke 已用 debug binary 复现验证链路：`/api/meta` 返回 `desktop-v1`，用户端 `18088` 可响应，`/api/channels` 显示 `web=running` 且 `imessage/discord/feishu/telegram=disabled`。
- 该修复不宣称 LaunchServices 窗口启动路径已恢复；它补齐的是每日 macOS release build 所需的自动化、可判定 Web/API smoke 路径。

## 端到端链路

1. 更新到 `main` 最新提交 `e04ae34e`。
2. 使用共享 target cache 执行完整桌面打包。
   - 首次 `bun run build:desktop` 因自动化 shell 的 `PATH` 未包含 `/Users/ecohnoch/.bun/bin` 失败。
   - 补齐 `PATH` 后，`bunx tauri` 在 Node runtime 下命中 `@tauri-apps/cli` native binding code signature 错误。
   - 改用 `bunx --bun tauri build --config bins/hone-desktop/tauri.generated.conf.json` 后打包成功。
3. 产物已生成：
   - `.app`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`，mtime `2026-05-15 04:09:17 CST`
   - `.dmg`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.12.2_aarch64.dmg`，mtime `2026-05-15 04:09:56 CST`
4. 准备隔离配置目录 `data/runtime/daily-build-check/`，确认：
   - `feishu.enabled=false`
   - `telegram.enabled=false`
   - `discord.enabled=false`
   - `discord.watch.enabled=false`
   - `imessage.enabled=false`
   - `event_engine.enabled=false`
   - `event_engine.global_digest.enabled=false`
   - `event_engine.thresholds.price_close_direct_enabled=false`
5. 使用隔离端口启动打包产物：
   - `HONE_WEB_PORT=18077`
   - `HONE_PUBLIC_WEB_PORT=18088`
   - `HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/daily-build-check/config.yaml`
   - `HONE_DESKTOP_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data/runtime/daily-build-check/data`
   - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`

## 期望效果

- 打包后的 `.app/Contents/MacOS/hone-desktop` 进程保持运行。
- `http://127.0.0.1:18077/api/meta` 返回 JSON。
- `http://127.0.0.1:18088/` 返回用户端页面。
- `/api/channels` 可确认 Feishu / Telegram / Discord / iMessage 均 disabled 或未启动。
- 验证完成后能清理本轮验证进程和端口。

## 当前实现效果

- 直接执行 `.app/Contents/MacOS/hone-desktop` 时，日志显示 embedded Web API 曾绑定 `18077/18088`：
  - `hone_web_api::start_server returned port=18077`
  - `embedded web server ready: http://127.0.0.1:18077`
- 但进程随后退出，`curl http://127.0.0.1:18077/api/meta` 持续 `Connection refused`，端口未保持监听。
- 使用 LaunchServices `open -n Hone Financial.app` 时，`hone-desktop` 进程可以保持运行，且环境变量中能看到 `HONE_WEB_PORT=18077`、`HONE_PUBLIC_WEB_PORT=18088` 与隔离配置路径；但 bundled Web/API 未启动，`18077/18088` 仍无监听。
- 两条启动路径均无法同时满足“release app 进程保持运行”和“本轮 Web/API 可响应”的最终验证条件。

## 用户影响

- 每日 macOS 完整打包验证无法成功闭环，当前 `.app/.dmg` 虽已生成，但不能证明打包后的桌面 runtime 可用。
- 若该行为在用户双击 / 正常 release 启动路径中复现，桌面壳可能显示为进程存在但本地 backend 未就绪，或直接启动后退出。

## 根因判断

- 初步怀疑存在桌面启动路径与 bundled backend bootstrap 生命周期错位：
  - 直接二进制启动路径能进入 backend bootstrap，但 app 进程随后结束，导致内嵌 Web server 被一并清理。
  - LaunchServices 启动路径能保持 app 进程，但没有触发同一套 backend bootstrap 日志与端口绑定。
- 这不是本轮渠道隔离配置导致的外部 IM 风险：日志显示 `event engine disabled via config`，且进程列表中没有 `hone-feishu`、`hone-telegram`、`hone-discord`、`hone-imessage` sidecar。

## 下一步建议

1. 给 desktop startup 增加可自动化的 headless/smoke 启动模式，或保证 direct `.app/Contents/MacOS/hone-desktop` 启动不会在 backend ready 后退出。
2. 检查 `bootstrap_backend_on_startup(...)` 是否依赖 frontend/window 事件；LaunchServices 进程保持但不启动 backend 的路径需要写入 `desktop.log`。
3. 在 release app smoke test 中补一条稳定回归：隔离 config + fixed ports + disabled channels 下，`/api/meta` 必须在限定时间内可访问。

## 验证结果

- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_smoke -- --nocapture`：通过。
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`：通过。
- `HONE_DESKTOP_SMOKE_SERVER=1 HONE_WEB_PORT=18077 HONE_PUBLIC_WEB_PORT=18088 HONE_USER_CONFIG_PATH=... HONE_DESKTOP_DATA_DIR=... target/debug/hone-desktop` + `curl /api/meta`、`curl :18088/`、`curl /api/channels`：通过，进程保持到 Ctrl-C，渠道 disabled 状态可审计。
- `env PATH="$HOME/.bun/bin:$PATH" CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build && env PATH="$HOME/.bun/bin:$PATH" CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx --bun tauri build --config bins/hone-desktop/tauri.generated.conf.json`：通过。
- `.app` 存在：通过。
- `.dmg` 存在：通过。
- 直接 `.app/Contents/MacOS/hone-desktop` 启动：失败，进程退出且 `/api/meta` connection refused。
- LaunchServices `.app` 启动复核：失败，进程保持但 `/api/meta` connection refused。
- 渠道隔离：配置层验证通过；未观察到真实 IM sidecar 进程。
- 清理：本轮验证进程已停止，`18077/18088` 未被占用。
