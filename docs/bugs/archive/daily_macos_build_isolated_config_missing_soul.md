# Daily macOS build isolated config missing soul prompt

- 发现时间：2026-05-02 04:08 CST
- Bug Type：Desktop release runtime / daily build validation
- 严重等级：P3
- 状态：Fixed
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证，使用隔离 `data/runtime/daily-build-check/config.yaml` 启动打包产物

## 端到端链路

1. 从 `/Users/ecohnoch/Desktop/honeclaw` 的 `main` 分支拉取远端最新代码，结果为 already up to date。
2. 使用 `CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target` 执行 `bun run build:desktop`。
3. Tauri 成功生成：
   - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`
   - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.5.1_aarch64.dmg`
4. 基于 `config.example.yaml` 生成隔离验证配置 `data/runtime/daily-build-check/config.yaml`，并显式禁用：
   - `feishu.enabled=false`
   - `telegram.enabled=false`
   - `discord.enabled=false`
   - `imessage.enabled=false`
   - `discord_watch.enabled=false`
   - `event_engine.enabled=false`
5. 使用隔离端口 `HONE_WEB_PORT=18077`、`HONE_PUBLIC_WEB_PORT=18088` 启动打包后的 `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`。

## 期望效果

- release app 应能使用隔离配置目录启动 embedded web runtime。
- 缺少可派生的资源文件时，desktop startup 应自动复制、使用 bundle/repo 内置资源，或返回可恢复错误。
- 隔离配置不应因为 `system_prompt_path: ./soul.md` 相对路径缺少同目录文件而 panic。

## 当前实现效果

- 首次隔离启动失败，stderr 输出：
  - `Hone Startup Blocked: 配置错误: 无法读取 system_prompt_path (/Users/ecohnoch/Desktop/honeclaw/data/runtime/daily-build-check/./soul.md)：No such file or directory (os error 2)`
  - `Failed to setup app: error encountered during setup hook`
- 失败后本轮隔离目录残留 `data/runtime/daily-build-check/data/runtime/locks/hone-desktop.lock`，记录的 pid 已退出，需要按 stale lock 规则清理。
- 手工复制 `/Users/ecohnoch/Desktop/honeclaw/soul.md` 到 `data/runtime/daily-build-check/soul.md` 后，同一个 `.app` 可正常启动并通过后续验证。

## 用户影响

- 每日 macOS 打包验证在使用干净隔离配置目录时可能先失败，需要人工补齐 `soul.md`。
- 真实日常配置目录若已有 `soul.md` 或使用 repo 根目录配置，不会直接影响正常桌面使用。
- 该问题不启动真实 Feishu / Telegram / Discord / iMessage 渠道；本轮配置已确认这些渠道保持 disabled。

## 根因判断

- `config.example.yaml` 的 `system_prompt_path` 默认是 `./soul.md`。
- 当自动化把 config 放到 `data/runtime/daily-build-check/config.yaml` 时，该相对路径按临时 config 所在目录解析。
- Desktop startup 在生成或读取有效 runtime config 前会校验该 prompt 文件；`ensure_runtime_paths` 虽会把 `soul.md` 复制到 runtime 子目录，但不会补到 canonical config 同级目录，因此首次启动仍失败。

## 修复情况（2026-05-02）

- `bins/hone-desktop/src/sidecar/runtime_env.rs` 在 desktop runtime path 物料化时读取 canonical config 的 `agent.system_prompt_path`。
- 若该路径是安全的相对路径、config 同级目标文件不存在、bundle/repo 里存在同名资源，则先把资源复制到 canonical config 同级目录，再生成 `effective-config.yaml`。
- 这覆盖隔离 `config.yaml` 指向 `./soul.md` 但目录干净的启动路径，同时不自动复制 `../soul.md` 这类会逃出 config 目录的路径。
- 修复不依赖当前机器线上运行态，也不启动生产渠道。
- 修复提交：`5bf2ccb`。

## 验证结果

- 2026-05-02 07:10 代码回归：
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`：通过，5 个 `runtime_env` 单测全部通过。
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`：通过。
  - `rustfmt --edition 2024 bins/hone-desktop/src/sidecar/runtime_env.rs` 与 `git diff --check`：通过。
- 2026-05-02 04:08 首次启动：
  - 构建打包：通过。
  - `.app` 存在性：通过，mtime `2026-05-02 04:05:57 CST`。
  - `.dmg` 存在性：通过，mtime `2026-05-02 04:06:35 CST`。
  - release app 启动：失败，缺少临时配置目录下的 `soul.md`。
  - 清理结果：已移除 dead `hone-desktop.lock`，未停止或改动已有日常 Hone 进程。
- 2026-05-02 04:09 规避后复测：
  - 将 `/Users/ecohnoch/Desktop/honeclaw/soul.md` 复制到 `data/runtime/daily-build-check/soul.md`。
  - release app 启动：通过，进程路径为 `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`。
  - `curl http://127.0.0.1:18077/api/meta`：通过，返回 `api_version=desktop-v1`、`version=0.5.1`。
  - `curl http://127.0.0.1:18088/chat`：通过，返回 200。
  - 用户端 JS / CSS 静态资源：通过，返回 200。
  - `/api/channels`：通过，`web` running，`imessage` / `discord` / `feishu` / `telegram` 均为 `disabled`。
  - watcher 检查：通过，未发现 `tauri dev`、`bun --watch`、`cargo watch`。
  - 清理结果：已终止本轮验证进程并移除本轮 stale locks；未停止或改动已有日常 Hone 进程。
