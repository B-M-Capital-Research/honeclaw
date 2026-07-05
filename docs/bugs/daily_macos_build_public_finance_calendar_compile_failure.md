# Daily macOS build public finance calendar compile failure

- 发现时间：2026-07-06 04:04 CST
- Bug Type：Daily macOS build verification / Rust compile failure
- 严重等级：P1
- 状态：Fixed
- GitHub Issue：未创建
- 证据来源：`honeclaw-mac` 每日 macOS 完整打包验证

## 端到端链路

1. 自动化按要求读取 `AGENTS.md`、`docs/repo-map.md`、`docs/invariants.md`、`docs/runbooks/desktop-release-app-runtime.md` 和 `docs/bugs/README.md`。
2. 初始 `git status --short` 无输出，工作区干净。
3. `git fetch origin` 成功，`git pull --rebase origin main` 快进到 `origin/main=1b2025a0`。
4. 生成隔离验证配置 `data/runtime/daily-build-check/config.yaml` 与 `effective-config.yaml`，关闭 Feishu / Telegram / Discord / iMessage / event_engine 等外部投递面，并把 storage 路径指向隔离 data 目录。
5. 执行 `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`。
6. release sidecar 与 desktop Web build 通过，但最终 Tauri build 在编译 `hone-web-api` 时失败。
7. 失败后修正 `crates/hone-web-api/src/routes/public_finance_calendar.rs` 中 `Holding` 类型路径，并重新执行定向编译与完整打包 / smoke 验证。

## 期望效果

- 每日 macOS 打包验证能够在最新 `main` 上完整执行 `build:desktop`。
- 构建成功后生成并确认：
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`
  - `/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/` 下本轮生成的 `.dmg`
- 随后使用隔离配置与固定本地端口启动 `.app/Contents/MacOS/hone-desktop` smoke server，确认 Web/API、用户端页面和渠道 disabled 状态。

## 当前实现效果

- 首次 `build:desktop` 失败在 Rust 编译阶段：
  - `error[E0425]: cannot find type Holding in crate hone_memory`
  - 位置：`crates/hone-web-api/src/routes/public_finance_calendar.rs:367`
  - 测试模块也存在同根因导入：`use hone_memory::Holding`
- 修复后完整 `build:desktop` 成功生成本轮 `.app` 与 `.dmg`。
- 隔离 smoke server 使用打包后的 `.app/Contents/MacOS/hone-desktop` 启动成功，`/api/meta`、用户端 `/`、`/api/channels` 均验证通过。

## 用户影响

- 修复前最新 `main` 不能通过 macOS 桌面打包验证，阻断 `.app` / `.dmg` 交付健康判断。
- 该问题发生在编译期，未启动真实 Feishu / Telegram / Discord / iMessage 渠道，未影响线上服务或真实 IM 投递。

## 根因判断

- `Holding` 实际定义在 `hone_memory::portfolio::Holding`，不是 `hone_memory::Holding`。
- `public_finance_calendar` 新代码同时在业务函数签名与测试模块中使用了 root 级 `hone_memory::Holding` 路径，导致 `hone-web-api` 在 desktop Tauri build 中编译失败。
- 根因是跨 crate 类型路径引用错误，不是 Bun / Tauri / DMG 工具链问题。

## 下一步建议

1. 若未来继续调整 `memory/src/portfolio.rs` 的 public type export，优先通过 `cargo check -p hone-web-api --tests` 捕获 Web API 引用漂移。
2. 每日 macOS build 仍应保留完整 `build:desktop`，因为该路径会覆盖普通 workspace check 之外的 desktop bundle 编译面。
3. 如果后续再次在 public finance calendar 附近失败，优先检查 `hone_memory::portfolio` 的公开 API 和测试导入路径。

## 验证结果

- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/routes/public_finance_calendar.rs`：通过。
- `cargo check -p hone-web-api --tests`：通过。
- `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run build:desktop`：通过。
- `.app` 产物确认：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app`，mtime `2026-07-06 04:09:10 CST`。
- `.dmg` 产物确认：`/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/dmg/Hone Financial_0.12.4_aarch64.dmg`，mtime `2026-07-06 04:09:47 CST`，size `122430834` bytes。
- `.app/Contents/MacOS/hone-desktop` 隔离 smoke：通过，`HONE_WEB_PORT=18077`、`HONE_PUBLIC_WEB_PORT=18088`。
- `/api/meta`：通过，返回 `version=0.12.4`。
- 用户端页面 `/`：通过，HTTP `200 OK`，`content-type: text/html`。
- `/api/channels`：通过，`web` 为 `running`，`imessage` / `discord` / `feishu` / `telegram` 均为 `disabled`。
- 进程形态：通过，运行进程为 `.app/Contents/MacOS/hone-desktop`，未发现 `tauri dev` / `bun --watch` / `cargo watch`。
- 清理：通过，smoke 进程已退出，`18077` / `18088` 无监听残留。
