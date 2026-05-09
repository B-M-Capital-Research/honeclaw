# Daily macOS build startup panic leaves stuck process

## Metadata

- 发现时间：2026-05-10 04:18 CST
- Bug Type：Desktop Startup / Runtime Cleanup
- 严重等级：P2
- 状态：New
- 发现来源：`honeclaw-mac` 每日 macOS 完整打包验证
- 关联提交：`ea573565`

## 证据来源

1. 每日验证先完成最新 `main` 拉取、release sidecar / Web / desktop 编译、`.app` bundling 和 `.dmg` bundling。
2. 本轮初次隔离配置生成脚本误把 `max_tokens` 类数值字段清成空字符串；使用打包后的 `.app/Contents/MacOS/hone-desktop` 启动时，release app 在 setup 阶段输出：
   - `Hone Startup Blocked: 无法生成 effective-config.yaml: 配置错误: 配置文件解析失败: invalid type: string "", expected u32`
   - 随后 Tauri setup hook panic：`Failed to setup app: error encountered during setup hook`
3. 该失败进程随后残留为 `UE` 状态，`kill` 与 `kill -9` 均未回收：
   - `pid=51996`
   - `stat=UE`
   - `command=/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app/Contents/MacOS/hone-desktop`
4. 修正隔离配置后，同一个 `.app` 能在 `HONE_WEB_PORT=18077`、`HONE_PUBLIC_WEB_PORT=18088` 下成功启动；`/api/meta`、`/api/channels`、用户端 `/chat` 均验证通过，且四个 IM 渠道 disabled。因此本单不记录 `.app` / `.dmg` 打包失败，而记录 setup 失败路径的 panic / 清理异常。

## 端到端链路

每日验证链路为：拉取最新 `main` -> release sidecar / Web / desktop 编译 -> 生成 `.app` -> 生成 `.dmg` -> 使用隔离配置启动 `.app/Contents/MacOS/hone-desktop` -> 验证管理端 API、用户端页面、渠道 disabled 状态 -> 清理本轮进程。

本轮主体链路在修正隔离配置后通过，但初次 setup 失败路径留下了不可回收的 `hone-desktop` 进程，导致“清理本轮验证进程”不完整。

## 期望效果

- 无效隔离配置应以可诊断错误退出，不应触发不可控 panic。
- setup 失败后不应长期残留 `hone-desktop` 进程。
- 自动化应能通过普通 `kill` 或进程自然退出完成本轮清理。

## 当前实现效果

- setup 失败路径先展示 `Hone Startup Blocked`，随后 Tauri setup hook panic。
- panic 后存在一个 `UE` 状态 `hone-desktop` 进程，当前无法通过 `kill -9` 清理。
- 该残留进程未占用 `18077` / `18088`，也未持有本轮隔离 runtime lock；修正配置后的正式 smoke 进程已成功清理。

## 用户影响

对正常有效配置启动没有直接阻断；本轮修正隔离配置后 release app 可以启动并服务 API / 用户端页面。但每日自动化、手工排障或用户配置损坏场景下，setup 失败可能留下不可回收的桌面进程，影响后续进程检查、启动判断和本机资源清理。

## 根因判断

初步根因在 desktop setup 失败处理路径：`prepare_desktop_startup(...)` 返回错误后，Tauri setup hook 把错误向上返回，最终进入 `tauri::App::run` 的 panic 路径。无效配置本身应被视为输入错误，但 release app 不应在该路径留下不可回收进程。

本轮没有进一步定位 `UE` 状态是否来自 macOS dialog / WebKit / Tauri abort 交互；需要单独复现无效 config，检查 setup error dialog、panic unwind 和进程退出路径。

## 下一步建议

1. 为 desktop startup invalid-config 路径加回归：构造 `max_tokens: ""` 或其它 serde 类型错误，验证能写入诊断日志并干净退出。
2. 避免 setup hook 直接返回会触发 `expect("error while running hone desktop")` panic 的错误形态；改为记录错误、展示非阻塞提示，并让进程可控退出。
3. 复查 `show_startup_error_dialog(...)` 在自动化 / 无交互场景下是否仍可能参与阻塞或异常退出。
4. 修复后重跑 `honeclaw-mac` 完整链路，确认无效配置失败路径不会残留 `hone-desktop` 进程。

## 验证结果

- `git pull --rebase origin main`：通过，更新到 `ea573565`。
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`：通过。
- `.app`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`，mtime `2026-05-10 04:06:54 CST`。
- `.dmg`：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.8.0_aarch64.dmg`，mtime `2026-05-10 04:07:34 CST`，size `103307177` bytes。
- 修正隔离配置后的 release app smoke：通过，`/api/meta` 返回 `version=0.8.0`，用户端 `/chat` 返回 `200`。
- 渠道隔离：`/api/channels` 显示 `web=running`，`imessage/discord/feishu/telegram=disabled`。
- 清理：修正配置后的验证进程与 `18077/18088` 端口已清理；初次无效配置启动留下的 `pid=51996 stat=UE` 仍未能回收。
