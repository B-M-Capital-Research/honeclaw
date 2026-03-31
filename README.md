# Hone-Financial

> 本地运行的 AI 投研 / 金融助手，已从 Python 重写为 **Rust**（Tokio 异步 + Cargo Workspace）。

支持多种交互入口：Web 控制台、Tauri 桌面端、CLI、iMessage（macOS）、Telegram Bot、Discord Bot。

---

## 快速开始

### 1. 启动主体

Hone Financial 的核心启动逻辑封装在 `launch.sh` 脚本中。如果你使用的是 MacOS，可以通过以下命令快速启动桌面端：

```bash
./launch.sh --desktop
```

> [!NOTE]
> 第一次启动时，脚本会自动执行以下操作：
> - 下载必要的运行环境（Tauri, Rust 编译环境等）
> - 编译各模块源码（hone-core, hone-integrations, hone-channels 等）
> - 初始化本地数据目录
>
> 首次构建与启动通常需要 **10 分钟** 左右，请耐心等待。

### 2. 配置模型

启动成功后，点击界面左侧导航栏的 **设置 (Settings)**，在 **模型配置** 模块中进行初始化：

- **本地端点**：如果你本地已经安装并运行了 `gemini cli` 或 `codex`，在设置中直接勾选相应项即可开始使用。
- **自定义模型**：如果没有本地端点，推荐使用 OpenAI 协议配置。我们经过大量测试，推荐使用 **OpenRouter + Gemini 2.0 Pro/Flash** (原 3.1 效果较好，目前 2.0 更佳) 的组合。
- **验证连接**：配置完成后，点击「检查连接」确保 API Key 与 Base URL 正确无误。

### 3. API 与渠道配置

- **金融数据 API**：在 API 配置区填写你的 FMP (Financial Modeling Prep) API Key，支持配置多个 Key 进行自动轮询。
- **搜索 API**：填写 Tavily API Key 以启用深度网页搜索功能。
- **渠道接入**：在渠道设置中一键开启飞书、Discord、Telegram 或 iMessage。其中 iMessage 渠道需要你按照系统提示开启 macOS 的 **完全磁盘控制权限**。

或直接用 `cargo`：

```bash
cargo run --release --bin hone-console-page
cargo run --release --bin hone-cli
cargo run --release --bin hone-telegram
cargo run --release --bin hone-discord
```

桌面端开发：

```bash
# 按当前 target triple 预构建桌面 sidecar + 生成 Tauri 配置后启动
bun run dev:desktop

# 构建桌面安装包 / 可执行产物（跨平台标准入口）
bun run build:desktop

# macOS release：同时产出 Apple Silicon / Intel 两套 DMG
bash ./make_dmg_release.sh

# Unix 侧仍可使用包装脚本
bash scripts/build_desktop.sh
```

前端开发：

```bash
# 终端 1：跑 Rust Web API（默认监听 8077）
cargo run --bin hone-console-page

# 终端 2：跑 Vite dev server（默认监听 3000，并代理 /api）
bun run dev:web
```

---

## 项目结构

```
Hone-Financial/
├── Cargo.toml                  # Workspace 根配置
├── package.json                # Bun workspace 根配置
├── bun.lock                    # Bun 锁文件
├── config.yaml                 # 运行时配置（从 config.example.yaml 复制）
│
├── crates/                     # 共享库 crates
│   ├── hone-core/              # 配置、日志、错误类型（HoneConfig）
│   ├── hone-llm/               # LLM 调用层（OpenRouter / Kimi）
│   ├── hone-tools/             # Tool trait、ToolRegistry、内置工具实现
│   ├── hone-integrations/      # 外部集成：X(Twitter)、NanoBanana 图片生成
│   ├── hone-scheduler/         # 定时任务调度器
│   └── hone-channels/          # 渠道运行时（HoneBotCore、流式分段引擎）
│
├── bins/                       # 可执行入口
│   ├── hone-cli/               # 交互式 REPL（本地调试）
│   ├── hone-imessage/          # iMessage Bot（macOS）
│   ├── hone-console-page/      # Web 控制台 API + 静态资源托管
│   ├── hone-telegram/          # Telegram Bot
│   └── hone-discord/           # Discord Bot
│
├── packages/
│   ├── app/                    # SolidJS + Vite Web 控制台
│   └── ui/                     # 共享 UI primitives / theme / markdown
├── bins/hone-desktop/          # Tauri 桌面宿主、窗口配置与打包资源
│
├── agents/function_calling/    # Agent 核心：函数调用、推理循环
├── agents/gemini_cli/          # Agent 适配：Gemini CLI
├── agents/codex_cli/           # Agent 适配：Codex CLI
├── memory/                     # 会话记忆
├── skills/                     # 可加载的 Skill 定义（YAML/Markdown）
├── scripts/                    # 辅助脚本（XHS cookie 同步等）
└── data/                       # 运行时数据（sessions、portfolio、cron_jobs）
```

---

## Crate 架构

```
hone-core          ← 基础设施（配置、日志、错误）
    ↑
hone-llm           ← LLM 调用抽象
    ↑
hone-tools         ← 工具层（Tool trait + 内置工具）
    ↑
hone-integrations  ← 外部服务（X / XHS / NanoBanana）
    ↑
hone-channels      ← 渠道运行时（HoneBotCore）
    ↑
bins/*             ← 各平台入口二进制
```

**内置工具**（`hone-tools`）：

| 工具 | 说明 |
|------|------|
| `DataFetchTool` | 金融数据拉取（FMP API） |
| `WebSearchTool` | 网页搜索 |
| `PortfolioTool` | 持仓管理（读写本地 JSON） |
| `CronJobTool` | 定时任务注册 / 管理 |
| `LoadSkillTool` | 动态加载 Skill 文件 |

---

## 配置文件

`config.yaml` 现在只作为只读种子模板，保留完整注释和默认值；桌面端与 `./launch.sh` 首次启动时会把它复制到 `data/runtime/config_runtime.yaml`，后续可写变更会落到同目录的 `config_runtime.overrides.yaml`。

在 `config.yaml` 中填写密钥与鉴权信息：

- `llm.openrouter.api_key`
- `fmp.api_key`
- `telegram.bot_token`
- `discord.bot_token`
- `feishu.app_id` / `feishu.app_secret`
- `x.oauth1.consumer_key` / `x.oauth1.consumer_secret` / `x.oauth1.access_token` / `x.oauth1.access_token_secret`
- `web.auth_token`（远程部署建议设置）
- `web.research_api_key`（研究报告 API，如不使用可留空）

如果你想重置运行时配置，删除 `data/runtime/config_runtime.yaml` 后重新启动即可，启动过程会重新生成基底并清理旧覆盖层。

---

## 主要功能

### 📱 iMessage Bot（macOS）

1. 确保终端有「完全磁盘访问权限」（用于读取 `~/Library/Messages/chat.db`）。
2. 在 `config.yaml` 中配置：
   ```yaml
   imessage:
     enabled: true
     target_handle: ""   # 留空监听所有发件人，或指定手机号/Apple ID
   ```
3. 启动：`./launch_imessage.sh`

### 🤖 Telegram / Discord Bot

在 `config.yaml` 中分别配置 `telegram` / `discord` 节点（`enabled: true`、`bot_token`、`allow_from`），然后运行对应 launch 脚本。

支持统一的 `chat_scope` 配置：

- `DM_ONLY`：只收私聊
- `GROUPCHAT_ONLY`：只收群聊
- `ALL`：私聊和群聊都收

其中群聊仍沿用显式触发模型：只有 `@bot` 或 reply-to-bot 才会真正执行 agent，未触发消息会先进群聊预触发窗口；若同一群上一条还在处理，新来的 `@bot` 会收到“请等待前一条完成”的提示，同时问题本身仍会保留在窗口里。

### 🏠 CLI（本地调试）

```bash
cargo run --release --bin hone-cli
```

交互式 REPL，输入 `quit` 退出。

### 🖥️ Web 控制台

```bash
# 一键启动（推荐）
./launch.sh

# 前端单独开发
bun run dev:web

# 前端检查
bun run typecheck:web
bun run test:web
bun run build:web
```

默认地址：
- Rust 控制台服务：[http://127.0.0.1:8077](http://127.0.0.1:8077)
- Vite dev server：[http://127.0.0.1:3000](http://127.0.0.1:3000)

### 🪟 Tauri 桌面端

- 桌面端宿主位于 `bins/hone-desktop/`，复用 `packages/app` 前端与 `hone-console-page` 后端
- 支持两种 backend 模式：
  - `bundled`：Tauri 在宿主进程内启动内置 `hone-console-page` backend，并托管渠道 sidecar
  - `remote`：桌面端连接任意兼容 `/api/meta` 握手协议的远程 HTTP backend
- 设置入口在前端 `/settings` 页面，支持切换模式、填写 remote base URL、Bearer token、查看 capability 列表
- 当前桌面 sidecar 由 `scripts/prepare_tauri_sidecar.mjs` 统一准备，并生成 `bins/hone-desktop/tauri.generated.conf.json`
- macOS release/DMG 打包会额外把 `hone-mcp` 与 `opencode` 一起打进包内，避免 bundled runtime 内部 agent 缺少本地 CLI/MCP binary
- Windows 打包只包含 `hone-discord` / `hone-feishu` / `hone-telegram` 三个 sidecar；`hone-imessage` 保持 macOS-only，不进入 Windows 包
- macOS 安装包启动后默认使用应用自己的 app sandbox 数据目录，并在首次启动时初始化 `runtime/`、`locks/`、`logs/` 与 `agent-sandboxes/`
- GUI 启动时会先补一轮 login shell 环境变量，再优先解析包内 `opencode` / `hone-mcp`，减少 Finder 启动场景下 `PATH` 缺失导致的 CLI 拉起失败
- 桌面端会把运行时诊断信息写到应用数据目录下的 `logs/`：
  - `desktop.log`：Tauri 宿主连接、probe、配置切换、sidecar 生命周期
  - `sidecar.log`：`hone-console-page` 的 stdout / stderr / 退出事件
- 具体日志路径可在桌面端「设置」页面查看

Windows 打包前置：

```powershell
# 需要先具备 Rust MSVC toolchain、Bun、WebView2 运行时/SDK 前置
cargo check -p hone-desktop
cargo check -p hone-discord -p hone-feishu -p hone-telegram
bun run build:web
bun run tauri:prep:build
bun run build:desktop
```

产物验证重点：

- `bins/hone-desktop/binaries/` 中应出现 `hone-discord-<target>.exe`、`hone-feishu-<target>.exe`、`hone-telegram-<target>.exe`
- `bins/hone-desktop/tauri.generated.conf.json` 中的 `bundle.externalBin` 不应包含 `hone-imessage`
- Tauri Windows 构建输出中应包含 `hone-desktop.exe`

macOS 回归自检：

```bash
# 仅验证生成配置，不实际编译；输出应仍包含 hone-imessage
bun scripts/prepare_tauri_sidecar.mjs debug --target-triple aarch64-apple-darwin --skip-build --json

# 构建 Apple Silicon + Intel 两套 DMG（会自动补齐 hone-mcp / opencode）
bash ./make_dmg_release.sh
```

---

## 测试与发布

```bash
# Rust 全量检查（本地等同 CI 主流程）
bash scripts/ci/check_fmt_changed.sh
cargo check --workspace --all-targets
cargo test --workspace --all-targets

# Web 前端检查
bun run typecheck:web
bun run test:web

# CI-safe 回归脚本（无外部账号依赖）
bash tests/regression/run_ci.sh

# 手工回归脚本（依赖本机已登录的外部 CLI，如 codex/gemini）
bash tests/regression/run_manual.sh
```

发布策略：
- CI：PR / push 运行 Rust 检查与 CI-safe 回归。
- CD：推送 `v*` tag 触发 Release，产出多平台二进制压缩包。

---

## X（Twitter）发布（两阶段确认）

流程：用户发起请求 → Agent 生成草稿（推文/线程 + 可选配图）→ 用户确认口令 → 发布。

### 配置

```yaml
# config.yaml
x:
  enabled: true
  dry_run: false          # 首次建议设为 true 联调
  default_image_count: 3
```

### 使用流程

1. 用户：`帮我分析英伟达然后发个推`
2. Agent 生成草稿 + 配图并返回确认口令
3. 用户：`确认发布X <token>`
4. Agent 调用 `x_publish` 完成发布

---

## Discord 频道 Watcher

对指定 Discord 频道做 REST 轮询，支持「被 @ 必回 + 低频集中参与」模式：

```yaml
# config.yaml
discord:
  watch:
    enabled: true
    channel_ids:
      - "123456789012345678"
    loop: true
```

在 Discord Bot 启动后，`hone-discord` 会按配置自动启用 watcher。

---

## 构建

```bash
# 构建所有 crate
cargo build --release

# 只构建特定 bin
cargo build --release --bin hone-cli

# 运行测试
cargo test
```

---

## 技术栈

| 层 | 主要依赖 |
|----|---------|
| 异步运行时 | `tokio` |
| HTTP 客户端 / 服务端 | `reqwest` / `axum` |
| LLM | `async-openai`（OpenRouter 兼容端点） |
| 序列化 | `serde` + `serde_json` + `serde_yaml` |
| 错误处理 | `anyhow` + `thiserror` |
| 日志追踪 | `tracing` + `tracing-subscriber` |
| 时间 | `chrono` |
