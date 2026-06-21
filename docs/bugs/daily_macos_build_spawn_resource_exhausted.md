# Daily macOS build blocked by local process spawn resource exhaustion

- 发现时间：2026-06-12 04:03 CST
- Bug Type：Daily macOS build verification / local build environment
- 严重等级：P2
- 状态：Later
- GitHub Issue：未创建
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证

## 端到端链路

1. 自动化按要求读取 `AGENTS.md`、`docs/repo-map.md`、`docs/invariants.md`、`docs/runbooks/desktop-release-app-runtime.md` 和 `docs/bugs/README.md`。
2. 初始 `git status --short` 无输出，工作区干净。
3. 首次 `git fetch origin` 因本机无法 fork SSH 子进程失败；向 Codex 父进程发送 `SIGCHLD` 后，重试 `git fetch origin` 成功，远端 `main` 从 `79fcc583` 更新到 `a1da0e9a`。
4. `git pull --rebase origin main` 又因 `merge-base` / `fetch` fork 失败；改用已刷新的 `origin/main` 执行 `git rebase origin/main` 成功，本地 `HEAD` 与 `origin/main` 均为 `a1da0e9a`。
5. 执行 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`，构建脚本在启动 `tauri:prep:build` 时失败，未进入实际 Tauri/Rust 打包。
6. 再次发送 `SIGCHLD` 并重试同一构建命令，仍因 `posix_spawn()` 返回 `EAGAIN` 失败。
7. 因 `.app` / `.dmg` 未能生成或确认，本轮未启动隔离 smoke runtime，也未触碰真实 Feishu / Telegram / Discord / iMessage 渠道。

## 期望效果

- 每日 macOS 打包验证能够在拉取远端最新 `main` 后运行完整 `build:desktop`。
- 构建成功后应生成并确认：
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/` 下的本轮 `.dmg`
- 随后使用隔离配置和固定本地端口启动 `.app/Contents/MacOS/hone-desktop` smoke server，确认 Web/API、用户端页面和渠道 disabled 状态。

## 当前实现效果

- 本轮已同步到远端最新 `main`：`HEAD=a1da0e9a`，`origin/main=a1da0e9a`。
- 构建命令失败两次：
  - 首次：`/bin/bash: fork: Resource temporarily unavailable`，`script "build:desktop" exited with code 128`
  - 重试：`EAGAIN: /bin/bash: non-blocking and interrupt i/o. Resource temporarily unavailable (posix_spawn())`
- 进程表显示大量 `<defunct>` 子进程，父进程集中在 `/Applications/Codex.app/Contents/MacOS/Codex`，同时存在多组 Codex `node_repl` / `app-server` / Computer Use MCP 子进程。
- `SIGCHLD` 只能让部分命令偶发越过 fork 点，不能保证 `bun run build:desktop` 进入真实打包。

## 用户影响

- 本轮每日 macOS 完整打包验证未闭环，不能证明最新 `main` 可以完整打出 `.app` 与 `.dmg`。
- 未启动本轮验证 runtime，未启动真实 IM sidecar，未影响线上服务或真实 Feishu / Telegram / Discord / iMessage 渠道。
- 当前证据指向本机自动化运行环境资源耗尽；若不清理 Codex 僵尸/旧子进程或重启宿主应用，后续每日打包仍可能在 git、bun、Tauri 或 Rust 任一需要 spawn 子进程的阶段失败。

## 根因判断

- 直接代码编译尚未开始，因此没有证据证明这是仓库代码或 Tauri 配置回归。
- 失败集中在 macOS 进程创建资源：`fork`、`posix_spawn()`、`EAGAIN`、`Resource temporarily unavailable`。
- 本轮 `ps` 输出显示超过两千个进程/僵尸进程，且大量 `<defunct>` 的父进程为 Codex 桌面主进程；僵尸进程不能由本轮构建命令直接回收。
- 因该问题阻断每日 macOS 产物交付验证，但尚未证明影响用户安装包或仓库代码，暂按 `P2 / New` 登记。
- 2026-06-21 19:09 CST 复核：本单证据仍指向宿主机进程 / 磁盘等本机自动化环境问题，而不是仓库代码、Tauri 配置或桌面 runtime 行为回归；当前 bug-2 规则也明确不再依赖本机线上运行态作为缺陷依据。本轮仅清理可再生 Cargo build artifacts 以恢复测试空间，没有重启 Codex 桌面父进程或执行完整 macOS 打包验证；状态改为 `Later`，待每日 macOS 构建链路在资源恢复后重新复现代码 / 打包阶段错误时再进入活跃修复队列。

## 下一步建议

1. 重启或修复 Codex 桌面父进程，使其回收 `<defunct>` 子进程，并确认 `ps` 进程数显著回落。
2. 清理不再使用的 Codex `node_repl` / `app-server` / Computer Use MCP 子进程；清理前需避免杀掉当前会话或真实 Hone 服务。
3. 资源恢复后重跑：
   - `git fetch origin`
   - `git rebase origin/main`
   - `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`
4. 若资源恢复后构建进入 Rust/Tauri 阶段并出现新的代码或打包错误，应按新的失败阶段另建或更新更具体的缺陷文档。

## 验证结果

- `git status --short`：通过，开始时无输出。
- `git fetch origin`：首次因 `cannot fork() for ssh` 失败；发送 `SIGCHLD` 后重试成功。
- `git rebase origin/main`：通过，本地 `main` 更新到 `a1da0e9a`。
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`：失败，`/bin/bash: fork: Resource temporarily unavailable`。
- 构建重试：失败，`EAGAIN ... Resource temporarily unavailable (posix_spawn())`。
- `.app` 产物确认：未执行，因为构建未进入打包。
- `.dmg` 产物确认：未执行，因为构建未进入打包。
- `.app/Contents/MacOS/hone-desktop` 隔离 smoke：未执行。
- 渠道隔离：未启动任何本轮验证 runtime 或真实 IM sidecar。
