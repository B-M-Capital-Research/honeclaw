# Daily macOS build DMG bundle failed

## Metadata

- 发现时间：2026-05-05 04:07 CST
- Bug Type：Build / Packaging
- 严重等级：P1
- 状态：New
- 发现来源：`honeclaw-mac` 每日 macOS 完整打包验证
- 关联提交：`26f4ddf`

## 证据来源

1. 工作区干净，`git fetch origin && git pull --rebase origin main` 返回 `Already up to date`。
2. 首选命令 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop` 在当前自动化 shell 中先因 PATH 缺少 Bun 失败；补 `PATH=$HOME/.bun/bin:$PATH` 后进入构建。
3. Node 执行 Tauri CLI 时命中 native binding 签名加载问题：`cli.darwin-arm64.node not valid for use in process: mapping process and mapped file (non-platform) have different Team IDs`。
4. 使用等价 fallback `env PATH="$HOME/.bun/bin:$PATH" CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx --bun tauri build --config bins/hone-desktop/tauri.generated.conf.json` 后，Rust release 编译、Web build、sidecar 准备和 `.app` bundling 完成，但 DMG bundling 失败。
5. 关键错误摘要：`failed to bundle project error running bundle_dmg.sh: failed to run /Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/bundle_dmg.sh`。

## 端到端链路

每日验证需要完成：拉取最新 `main` -> release sidecar / Web / desktop 编译 -> 生成 `.app` -> 生成 `.dmg` -> 使用 `.app/Contents/MacOS/hone-desktop` 以隔离配置启动 -> 验证 `/api/meta`、public 页面与渠道禁用状态。

本轮链路在 DMG bundling 阶段中断，未进入隔离启动验证。

## 期望效果

- `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app` 存在且为本轮产物。
- `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.7.0_aarch64.dmg` 或同版本最新 `.dmg` 存在且 mtime 位于本轮构建窗口。
- 后续可使用 `.app/Contents/MacOS/hone-desktop` 启动隔离 runtime 并完成 web/API/channel disabled smoke test。

## 当前实现效果

- `.app` 已生成：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`，mtime `2026-05-05 04:06:31 CST`。
- 最终 `bundle/dmg/` 下没有本轮 `.dmg` 文件。
- 发现一个临时读写镜像残留在 macOS bundle 目录：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/rw.60835.Hone Financial_0.7.0_aarch64.dmg`，mtime `2026-05-05 04:06:52 CST`，大小 `305173504` bytes。
- 启动验证未执行，因为 `.dmg` 缺失已经使完整打包验证失败。

## 用户影响

macOS 桌面产物无法完成每日交付形态验证。即使 `.app` 已生成，也不能证明安装分发所需的 `.dmg` 可以被稳定产出，阻断 macOS 打包健康判断。

## 根因判断

根因尚未定位到代码。当前证据显示失败发生在 Tauri 生成的 `bundle_dmg.sh` 执行期间，且脚本留下了 `rw.*.dmg` 中间产物但没有生成最终 DMG。需要进一步复现时打开 `hdiutil` verbose 或直接捕获 `bundle_dmg.sh` 内部失败点。

另一个独立环境风险是当前自动化 PATH 不包含 `/Users/ecohnoch/.bun/bin`，且 Node 运行 Tauri CLI 会触发 native binding Team ID 校验失败；本轮通过 `bunx --bun tauri` 绕过后仍失败在 DMG bundling，因此主阻断仍是 DMG 产出失败。

## 下一步建议

1. 重新运行直接 Tauri build，并让 DMG 阶段输出 `hdiutil` 详细日志，确认失败是在 create、attach、Finder AppleScript、detach 还是 convert。
2. 检查 Tauri create-dmg 在当前 macOS / Codex 自动化进程环境下是否需要 `--skip-jenkins`、APFS/HFS+ 调整或 sandbox-safe 配置。
3. 若问题只出现在 Node CLI 路径，调整 `build:desktop` 脚本或自动化环境，固定使用 Bun runtime 执行 Tauri CLI。
4. 修复后重新执行完整每日链路，包括 `.app`、`.dmg`、隔离启动、`/api/meta`、public 页面和渠道 disabled 验证。

## 验证结果

- 拉取最新 `main`：通过。
- release sidecar 编译：通过。
- Web desktop build：通过。
- `hone-desktop` release 编译：通过。
- `.app` 生成：通过。
- `.dmg` 生成：失败。
- `.app/Contents/MacOS/hone-desktop` 隔离启动验证：未执行，因 `.dmg` 缺失提前失败。
- 渠道禁用状态确认：未执行，因 `.dmg` 缺失提前失败。
