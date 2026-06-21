# Daily macOS build Tauri CLI killed before bundling

- 发现时间：2026-06-22 04:08 CST
- Bug Type：Daily macOS build verification / desktop packaging
- 严重等级：P1
- 状态：New
- GitHub Issue：未创建
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证

## 端到端链路

1. 自动化按要求读取 `AGENTS.md`、`docs/repo-map.md`、`docs/invariants.md`、`docs/runbooks/desktop-release-app-runtime.md` 和 `docs/bugs/README.md`。
2. 初始 `git status --short` 无输出，工作区干净。
3. `GIT_SSH_COMMAND='ssh -o BatchMode=yes -o ConnectTimeout=20' git fetch origin` 成功。
4. `git pull --rebase origin main` 成功，本地 `main` 与 `origin/main` 同步到 `cbbd792b`。
5. 执行 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`。
6. `scripts/prepare_tauri_sidecar.mjs release` 完成并准备 6 个桌面 sidecar，但多个 bin 的 `rust-objcopy` strip debug info 被 `SIGKILL`，仅以 warning 形式继续。
7. 进入 `bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json` 后，Tauri CLI 被 `Killed: 9` 终止，`build:desktop` 退出码为 `137`。
8. 按 runbook 允许的直接 Tauri 路径重试 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`，仍立即以退出码 `137` 结束且无输出。
9. 进一步执行 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri --version` 也直接退出 `137`，说明失败点集中在 Tauri CLI 启动 / 执行阶段，而不是单个 Rust crate 编译错误。

## 期望效果

- 每日 macOS 打包验证能够在最新 `main` 上完整执行 `build:desktop`。
- 构建成功后生成并确认：
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/` 下本轮生成的 `.dmg`
- 随后使用隔离配置与固定本地端口启动 `.app/Contents/MacOS/hone-desktop` smoke server，确认 Web/API、用户端页面和渠道 disabled 状态。

## 当前实现效果

- 本轮 sidecar release 编译完成，但 Tauri bundle 阶段没有完成。
- `build:desktop` 失败摘要：
  - 多个 sidecar 出现 warning：`stripping debug info with rust-objcopy failed: signal: 9 (SIGKILL)`。
  - Tauri build 阶段：`/bin/bash: line 1: 52519 Killed: 9 bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`。
  - 最终错误：`error: script "build:desktop" exited with code 137`。
- 直接重试 `bunx tauri build --config ...`：退出码 `137`，无输出。
- `bunx tauri --version`：退出码 `137`，无输出。
- 产物检查显示当前 bundle 仍是旧产物：
  - `.app` mtime：`2026-06-02 04:09:39 CST`
  - `.dmg` mtime：`2026-06-02 04:10:18 CST`
- 本轮没有生成可用于验证的新 `.app` / `.dmg`。

## 用户影响

- 本轮每日 macOS 完整打包验证未闭环，不能证明最新 `main` 可以完整打出 `.app` 与 `.dmg`。
- 因没有本轮新 bundle，未启动 `.app/Contents/MacOS/hone-desktop` 隔离 smoke runtime。
- 未启动真实 Feishu / Telegram / Discord / iMessage 渠道，未检查或重启线上服务。

## 根因判断

- 当前证据不是 GitHub 拉取失败：默认 SSH fetch 成功，`main` 已同步到 `origin/main=cbbd792b`。
- 当前证据也不是明确的 Rust 编译错误：sidecar release 编译已完成，失败集中在 `bunx tauri` CLI 进程被系统 `SIGKILL`。
- 与既有 `daily_macos_build_spawn_resource_exhausted.md` 的 `fork` / `posix_spawn EAGAIN` 不同，本轮进程创建本身可用，进程数约 `709`、僵尸进程约 `1`、磁盘可用约 `36Gi`；但 Tauri CLI 及其版本查询仍被 `137` 终止。
- 由于该问题阻断 macOS 产物交付验证，按 P1 登记。

## 下一步建议

1. 优先复核本机是否有内存压力、进程被 Jetsam / OOM / 安全策略杀死、或 `@tauri-apps/cli` native binding / Bun runtime 被系统拒绝执行。
2. 定向比较 `bunx tauri --version`、`bunx --bun tauri --version`、`bunx tauri build --verbose --config ...` 的行为，确认是否只影响 Node-backed `bunx` 路径。
3. 如确认是本机 runtime / 缓存问题，清理 Bun/Tauri CLI 缓存或固定使用可工作的 Tauri CLI 调用方式。
4. 修复后重新运行完整 `honeclaw-mac` 链路，必须生成本轮 `.app` / `.dmg` 并完成隔离 smoke。

## 验证结果

- `git status --short`：通过，开始时无输出。
- `git fetch origin`：通过。
- `git pull --rebase origin main`：通过，本地和远端均为 `cbbd792b`。
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`：失败，退出码 `137`。
- 直接 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`：失败，退出码 `137`。
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri --version`：失败，退出码 `137`。
- `.app` 产物确认：失败，现有 `.app` 为 2026-06-02 旧产物，不是本轮构建。
- `.dmg` 产物确认：失败，现有 `.dmg` 为 2026-06-02 旧产物，不是本轮构建。
- `.app/Contents/MacOS/hone-desktop` 隔离 smoke：未执行，因为本轮未生成新 bundle。
- 渠道隔离：未启动任何本轮验证 runtime 或真实 IM sidecar。
